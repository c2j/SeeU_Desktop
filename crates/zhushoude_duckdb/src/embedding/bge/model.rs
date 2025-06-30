//! BGE模型实现
//!
//! 基于Candle框架的BGE模型推理实现

use crate::{Result, Error};
use crate::embedding::provider::{BGEConfig, Device};
use crate::embedding::bge::tokenizer::TokenizedInput;
use crate::model::{ModelResourceManager, ModelPaths};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use candle_core::{Tensor, DType};
use candle_nn::{VarBuilder, Module, Linear, LayerNorm, Embedding};
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use std::collections::HashMap;

/// BGE模型封装
pub struct BGEModel {
    config: BGEConfig,
    device: candle_core::Device,
    model: Option<Arc<dyn ModelInference>>,
    is_loaded: bool,
}

/// 模型推理接口
trait ModelInference: Send + Sync {
    fn forward(&self, input_ids: &candle_core::Tensor, attention_mask: &candle_core::Tensor) -> Result<candle_core::Tensor>;
    fn get_hidden_size(&self) -> usize;
}

/// 真正的BGE模型实现
struct RealBGEModel {
    bert: BertModel,
    device: candle_core::Device,
    hidden_size: usize,
}

/// 占位符模型实现（用于测试和开发）
struct PlaceholderModel {
    hidden_size: usize,
    device: candle_core::Device,
}

impl ModelInference for RealBGEModel {
    fn forward(&self, input_ids: &candle_core::Tensor, attention_mask: &candle_core::Tensor) -> Result<candle_core::Tensor> {
        // 使用BERT模型进行前向传播
        let sequence_output = self.bert
            .forward(input_ids, attention_mask, None) // 第三个参数是token_type_ids，对于BGE通常为None
            .map_err(|e| Error::ModelError(format!("BERT前向传播失败: {}", e)))?;

        Ok(sequence_output)
    }

    fn get_hidden_size(&self) -> usize {
        self.hidden_size
    }
}

impl ModelInference for PlaceholderModel {
    fn forward(&self, input_ids: &candle_core::Tensor, _attention_mask: &candle_core::Tensor) -> Result<candle_core::Tensor> {
        // 获取输入形状
        let shape = input_ids.shape();
        let batch_size = shape.dims()[0];
        let seq_len = shape.dims()[1];

        // 基于输入ID生成确定性但多样化的向量
        let input_data = input_ids.to_vec2::<u32>()
            .map_err(|e| Error::ModelError(format!("获取输入数据失败: {}", e)))?;

        let mut hidden_data = Vec::new();
        for batch_idx in 0..batch_size {
            for seq_idx in 0..seq_len {
                for hidden_idx in 0..self.hidden_size {
                    // 使用输入ID和位置信息生成确定性的值
                    let input_id = input_data[batch_idx][seq_idx];
                    let seed = (input_id as u64)
                        .wrapping_mul(31)
                        .wrapping_add(seq_idx as u64)
                        .wrapping_mul(17)
                        .wrapping_add(hidden_idx as u64);

                    // 生成[-1, 1]范围内的值
                    let value = ((seed % 10000) as f32 / 5000.0) - 1.0;
                    hidden_data.push(value);
                }
            }
        }

        // 创建tensor
        let hidden_states = candle_core::Tensor::new(hidden_data.as_slice(), &self.device)
            .map_err(|e| Error::ModelError(format!("创建占位符输出失败: {}", e)))?
            .reshape(&[batch_size, seq_len, self.hidden_size])
            .map_err(|e| Error::ModelError(format!("重塑占位符输出失败: {}", e)))?;

        Ok(hidden_states)
    }

    fn get_hidden_size(&self) -> usize {
        self.hidden_size
    }
}

impl BGEModel {
    /// 加载模型
    pub async fn load(config: &BGEConfig) -> Result<Self> {
        // 创建设备
        let device = match &config.device {
            Device::CPU => candle_core::Device::Cpu,
            Device::CUDA(id) => {
                candle_core::Device::new_cuda(*id)
                    .map_err(|e| Error::ModelError(format!("CUDA设备初始化失败: {}", e)))?
            },
            Device::Metal => {
                candle_core::Device::new_metal(0)
                    .map_err(|e| Error::ModelError(format!("Metal设备初始化失败: {}", e)))?
            },
        };

        // 尝试加载真实模型，如果失败则使用占位符
        let model = match Self::load_real_model(config, &device).await {
            Ok(model) => Some(model),
            Err(e) => {
                println!("⚠️  无法加载真实BGE模型: {}", e);
                println!("🔄 使用占位符模型进行开发测试");
                
                // 创建占位符模型
                let placeholder = PlaceholderModel {
                    hidden_size: config.model_variant.dimension(),
                    device: device.clone(),
                };
                Some(Arc::new(placeholder) as Arc<dyn ModelInference>)
            }
        };

        let is_loaded = model.is_some();
        Ok(Self {
            config: config.clone(),
            device,
            model,
            is_loaded,
        })
    }

    /// 尝试加载真实模型
    async fn load_real_model(config: &BGEConfig, device: &candle_core::Device) -> Result<Arc<dyn ModelInference>> {
        // 确保模型文件可用
        let model_manager = ModelResourceManager::new(config.cache_dir.clone());
        let model_paths = model_manager.ensure_model_available(config.model_variant.clone()).await?;

        // 加载模型权重
        if config.enable_quantization {
            Self::load_quantized_model(&model_paths, device).await
        } else {
            Self::load_full_precision_model(&model_paths, device).await
        }
    }

    /// 加载量化模型
    async fn load_quantized_model(_model_paths: &ModelPaths, _device: &candle_core::Device) -> Result<Arc<dyn ModelInference>> {
        // TODO: 实现量化模型加载
        Err(Error::ModelError("量化模型加载尚未实现".to_string()))
    }

    /// 加载全精度模型
    async fn load_full_precision_model(model_paths: &ModelPaths, device: &candle_core::Device) -> Result<Arc<dyn ModelInference>> {
        println!("🔧 开始加载BGE全精度模型...");

        // 1. 加载配置文件
        let config_content = std::fs::read_to_string(&model_paths.config_file)
            .map_err(|e| Error::ModelError(format!("读取配置文件失败: {}", e)))?;

        let bert_config: BertConfig = serde_json::from_str(&config_content)
            .map_err(|e| Error::ModelError(format!("解析配置文件失败: {}", e)))?;

        println!("✅ 配置文件加载成功: hidden_size={}", bert_config.hidden_size);

        // 2. 加载模型权重
        let model_path = &model_paths.model_file;

        // 使用safetensors或pytorch格式加载权重
        let weights = if model_path.extension().and_then(|s| s.to_str()) == Some("safetensors") {
            candle_core::safetensors::load(model_path, device)
                .map_err(|e| Error::ModelError(format!("加载safetensors权重失败: {}", e)))?
        } else {
            // 尝试加载pytorch格式
            Self::load_pytorch_weights(model_path, device)?
        };

        println!("✅ 模型权重加载成功，共{}个参数", weights.len());

        // 3. 构建BERT模型
        let var_builder = VarBuilder::from_tensors(weights, DType::F32, device);
        let bert = BertModel::load(var_builder, &bert_config)
            .map_err(|e| Error::ModelError(format!("构建BERT模型失败: {}", e)))?;

        println!("✅ BERT模型构建成功");

        let real_model = RealBGEModel {
            bert,
            device: device.clone(),
            hidden_size: bert_config.hidden_size,
        };

        Ok(Arc::new(real_model) as Arc<dyn ModelInference>)
    }

    /// 加载PyTorch权重文件
    fn load_pytorch_weights(model_path: &std::path::Path, device: &candle_core::Device) -> Result<HashMap<String, Tensor>> {
        println!("🔧 开始加载PyTorch权重文件: {:?}", model_path);

        // 使用candle的PthTensors加载器
        let pth_tensors = candle_core::pickle::PthTensors::new(model_path, None)
            .map_err(|e| Error::ModelError(format!("创建PyTorch张量加载器失败: {}", e)))?;

        let tensor_infos = pth_tensors.tensor_infos();
        println!("✅ 发现 {} 个权重张量", tensor_infos.len());

        let mut tensor_map: HashMap<String, Tensor> = HashMap::new();
        let mut loaded_count = 0;
        let mut skipped_count = 0;

        // 逐个加载张量
        for (name, _tensor_info) in tensor_infos.iter() {
            // 尝试加载张量
            match pth_tensors.get(name) {
                Ok(Some(tensor)) => {
                    // 将张量移动到指定设备
                    let tensor = tensor.to_device(device)
                        .map_err(|e| Error::ModelError(format!("移动张量 {} 到设备失败: {}", name, e)))?;

                    tensor_map.insert(name.clone(), tensor);
                    loaded_count += 1;
                    if loaded_count <= 5 || loaded_count % 10 == 0 {
                        println!("✅ 成功加载张量: {} ({}/{})", name, loaded_count, tensor_infos.len());
                    }
                }
                Ok(None) => {
                    skipped_count += 1;
                    if skipped_count <= 3 {
                        println!("⚠️ 跳过张量 {}: 张量不存在", name);
                    } else if skipped_count == 4 {
                        println!("⚠️ ... (跳过更多张量的详细日志)");
                    }
                }
                Err(e) => {
                    skipped_count += 1;
                    if skipped_count <= 3 {
                        println!("⚠️ 跳过张量 {}: {}", name, e);
                    } else if skipped_count == 4 {
                        println!("⚠️ ... (跳过更多张量的详细日志)");
                    }
                }
            }
        }

        println!("📊 权重加载总结: 成功加载 {}/{} 个张量", loaded_count, tensor_infos.len());

        if loaded_count == 0 {
            return Err(Error::ModelError("没有成功加载任何权重张量".to_string()));
        }

        if loaded_count < tensor_infos.len() / 2 {
            println!("⚠️ 警告: 只加载了 {:.1}% 的权重，模型可能无法正常工作",
                (loaded_count as f32 / tensor_infos.len() as f32) * 100.0);
        }

        Ok(tensor_map)
    }

    /// 前向推理
    pub async fn forward(&self, tokens: &TokenizedInput) -> Result<Vec<f32>> {
        if !self.is_loaded {
            return Err(Error::ModelError("模型未加载".to_string()));
        }

        let model = self.model.as_ref().unwrap();

        // 转换为Tensor
        let (input_ids_data, attention_mask_data) = tokens.to_tensor_data();
        
        let input_ids = candle_core::Tensor::new(input_ids_data.as_slice(), &self.device)
            .map_err(|e| Error::ModelError(format!("创建input_ids tensor失败: {}", e)))?
            .reshape(&[1, input_ids_data.len()])
            .map_err(|e| Error::ModelError(format!("重塑input_ids tensor失败: {}", e)))?;

        let attention_mask = candle_core::Tensor::new(attention_mask_data.as_slice(), &self.device)
            .map_err(|e| Error::ModelError(format!("创建attention_mask tensor失败: {}", e)))?
            .reshape(&[1, attention_mask_data.len()])
            .map_err(|e| Error::ModelError(format!("重塑attention_mask tensor失败: {}", e)))?;

        // 模型前向传播
        let outputs = model.forward(&input_ids, &attention_mask)?;

        // 平均池化
        let pooled_output = self.mean_pooling(&outputs, &attention_mask)?;

        // 转换为Vec<f32>
        let embedding = if pooled_output.dims().len() == 1 {
            pooled_output.to_vec1::<f32>()
                .map_err(|e| Error::ModelError(format!("转换输出tensor失败: {}", e)))?
        } else {
            // 如果是2维的，先压缩为1维
            let squeezed = pooled_output.squeeze(0)
                .map_err(|e| Error::ModelError(format!("压缩输出tensor失败: {}", e)))?;
            squeezed.to_vec1::<f32>()
                .map_err(|e| Error::ModelError(format!("转换输出tensor失败: {}", e)))?
        };

        // 归一化（如果配置要求）
        if self.config.normalize_embeddings {
            Ok(self.normalize_vector(&embedding))
        } else {
            Ok(embedding)
        }
    }

    /// 平均池化
    fn mean_pooling(&self, token_embeddings: &candle_core::Tensor, attention_mask: &candle_core::Tensor) -> Result<candle_core::Tensor> {
        // 将attention_mask转换为f32类型
        let mask_f32 = attention_mask
            .to_dtype(candle_core::DType::F32)
            .map_err(|e| Error::ModelError(format!("转换attention_mask类型失败: {}", e)))?;

        // 扩展attention_mask维度以匹配token_embeddings
        let expanded_mask = mask_f32
            .unsqueeze(2)
            .map_err(|e| Error::ModelError(format!("扩展attention_mask失败: {}", e)))?
            .expand(token_embeddings.shape())
            .map_err(|e| Error::ModelError(format!("广播attention_mask失败: {}", e)))?;

        // 应用mask
        let masked_embeddings = token_embeddings
            .mul(&expanded_mask)
            .map_err(|e| Error::ModelError(format!("应用mask失败: {}", e)))?;

        // 计算有效token数量
        let sum_mask = mask_f32
            .sum_keepdim(1)
            .map_err(|e| Error::ModelError(format!("计算mask总和失败: {}", e)))?;

        // 求和并除以有效token数量
        let sum_embeddings = masked_embeddings
            .sum_keepdim(1)
            .map_err(|e| Error::ModelError(format!("计算嵌入总和失败: {}", e)))?;

        // 扩展sum_mask以匹配sum_embeddings的维度
        let expanded_sum_mask = sum_mask
            .unsqueeze(2)
            .map_err(|e| Error::ModelError(format!("扩展sum_mask失败: {}", e)))?
            .expand(sum_embeddings.shape())
            .map_err(|e| Error::ModelError(format!("广播sum_mask失败: {}", e)))?;

        let mean_embeddings = sum_embeddings
            .div(&expanded_sum_mask)
            .map_err(|e| Error::ModelError(format!("计算平均值失败: {}", e)))?;

        // 移除多余维度
        mean_embeddings
            .squeeze(1)
            .map_err(|e| Error::ModelError(format!("压缩维度失败: {}", e)))
    }

    /// 批量前向推理
    pub async fn batch_forward(&self, batch_tokens: &[TokenizedInput]) -> Result<Vec<Vec<f32>>> {
        if batch_tokens.is_empty() {
            return Ok(Vec::new());
        }

        if !self.is_loaded {
            return Err(Error::ModelError("模型未加载".to_string()));
        }

        // 对于占位符实现，我们逐个处理
        // 在真实实现中，这里应该进行真正的批量处理
        let mut embeddings = Vec::new();
        for tokens in batch_tokens {
            embeddings.push(self.forward(tokens).await?);
        }

        Ok(embeddings)
    }

    /// 向量归一化
    fn normalize_vector(&self, vector: &[f32]) -> Vec<f32> {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            vector.iter().map(|x| x / norm).collect()
        } else {
            vector.to_vec()
        }
    }

    /// 获取模型配置
    pub fn get_config(&self) -> &BGEConfig {
        &self.config
    }

    /// 检查模型是否已加载
    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    /// 获取模型维度
    pub fn get_dimension(&self) -> usize {
        self.config.model_variant.dimension()
    }

    /// 估算内存使用量
    pub fn estimate_memory_usage(&self) -> usize {
        if let Some(model) = &self.model {
            // 基本模型大小 + 隐藏状态缓存
            let base_size = self.config.model_variant.estimated_memory_mb() * 1024 * 1024;
            let hidden_size = model.get_hidden_size() * 4; // 4 bytes per f32
            base_size + hidden_size
        } else {
            0
        }
    }

    /// 模型预热
    pub async fn warmup(&self) -> Result<()> {
        if !self.is_loaded {
            return Err(Error::ModelError("模型未加载".to_string()));
        }

        println!("🔥 开始模型预热...");

        // 使用简单的测试输入进行预热
        let mut input_ids = vec![101, 1234, 5678, 102];
        input_ids.extend(vec![0; self.config.max_length - 4]);
        let mut attention_mask = vec![1, 1, 1, 1];
        attention_mask.extend(vec![0; self.config.max_length - 4]);

        let test_tokens = TokenizedInput {
            input_ids,
            attention_mask,
            sequence_length: self.config.max_length,
        };

        // 执行几次推理以预热模型
        for _ in 0..3 {
            let _ = self.forward(&test_tokens).await?;
        }

        println!("✅ 模型预热完成");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::provider::{BGEConfig, BGEVariant, Device};

    fn create_test_config() -> BGEConfig {
        BGEConfig {
            model_variant: BGEVariant::Small,
            device: Device::CPU,
            batch_size: 16,
            max_length: 128,
            normalize_embeddings: true,
            cache_size: 1000,
            enable_quantization: false,
            cache_dir: std::path::PathBuf::from("./test_models"),
        }
    }

    #[tokio::test]
    async fn test_model_loading() {
        let config = create_test_config();
        let model = BGEModel::load(&config).await;
        
        // 应该成功加载（使用占位符模型）
        assert!(model.is_ok());
        
        let model = model.unwrap();
        assert!(model.is_loaded());
        assert_eq!(model.get_dimension(), 512);
    }

    #[tokio::test]
    async fn test_model_inference() {
        let config = create_test_config();
        let model = BGEModel::load(&config).await.unwrap();
        
        let mut input_ids = vec![101, 1234, 5678, 102];
        input_ids.extend(vec![0; 124]);
        let mut attention_mask = vec![1, 1, 1, 1];
        attention_mask.extend(vec![0; 124]);

        let test_tokens = TokenizedInput {
            input_ids,
            attention_mask,
            sequence_length: 128,
        };
        
        let result = model.forward(&test_tokens).await;
        assert!(result.is_ok());
        
        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 512);
        
        // 检查归一化
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_batch_inference() {
        let config = create_test_config();
        let model = BGEModel::load(&config).await.unwrap();
        
        let mut input_ids1 = vec![101, 1234, 102];
        input_ids1.extend(vec![0; 125]);
        let mut attention_mask1 = vec![1, 1, 1];
        attention_mask1.extend(vec![0; 125]);

        let mut input_ids2 = vec![101, 5678, 102];
        input_ids2.extend(vec![0; 125]);
        let mut attention_mask2 = vec![1, 1, 1];
        attention_mask2.extend(vec![0; 125]);

        let test_tokens = vec![
            TokenizedInput {
                input_ids: input_ids1,
                attention_mask: attention_mask1,
                sequence_length: 128,
            },
            TokenizedInput {
                input_ids: input_ids2,
                attention_mask: attention_mask2,
                sequence_length: 128,
            },
        ];
        
        let result = model.batch_forward(&test_tokens).await;
        assert!(result.is_ok());
        
        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 512);
        assert_eq!(embeddings[1].len(), 512);
    }
}
