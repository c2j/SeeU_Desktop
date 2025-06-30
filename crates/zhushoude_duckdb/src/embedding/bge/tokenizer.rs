//! BGE分词器实现
//! 
//! 基于HuggingFace tokenizers库的BGE分词器封装

use crate::{Result, Error};
use crate::embedding::provider::BGEConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// BGE分词器封装
pub struct BGETokenizer {
    tokenizer: tokenizers::Tokenizer,
    max_length: usize,
    pad_token_id: u32,
    cls_token_id: u32,
    sep_token_id: u32,
    config: BGEConfig,
}

/// 分词结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizedInput {
    pub input_ids: Vec<u32>,
    pub attention_mask: Vec<u32>,
    pub sequence_length: usize,
}

impl BGETokenizer {
    /// 加载分词器
    pub async fn load(config: &BGEConfig) -> Result<Self> {
        let tokenizer_path = config.cache_dir.join(config.model_variant.model_name()).join("tokenizer.json");

        // 检查分词器文件是否存在
        if !tokenizer_path.exists() {
            return Err(Error::ModelError(format!(
                "分词器文件不存在: {}",
                tokenizer_path.display()
            )));
        }

        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| Error::ModelError(format!("加载分词器失败: {}", e)))?;

        // 获取特殊token ID
        let pad_token_id = tokenizer.token_to_id("[PAD]").unwrap_or(0);
        let cls_token_id = tokenizer.token_to_id("[CLS]").unwrap_or(101);
        let sep_token_id = tokenizer.token_to_id("[SEP]").unwrap_or(102);

        Ok(Self {
            tokenizer,
            max_length: config.max_length,
            pad_token_id,
            cls_token_id,
            sep_token_id,
            config: config.clone(),
        })
    }

    /// 创建占位符分词器（用于测试）
    pub fn placeholder(config: &BGEConfig) -> Result<Self> {
        // 创建一个简单的占位符分词器
        let mut tokenizer = tokenizers::Tokenizer::new(
            tokenizers::models::wordpiece::WordPiece::builder()
                .vocab(std::collections::HashMap::from([
                    ("[PAD]".to_string(), 0),
                    ("[CLS]".to_string(), 101),
                    ("[SEP]".to_string(), 102),
                    ("[UNK]".to_string(), 100),
                    ("测".to_string(), 1000),
                    ("试".to_string(), 1001),
                    ("文".to_string(), 1002),
                    ("本".to_string(), 1003),
                ]))
                .unk_token("[UNK]".to_string())
                .build()
                .map_err(|e| Error::ModelError(format!("创建占位符分词器失败: {}", e)))?
        );

        // 添加基本的预处理器
        tokenizer.with_pre_tokenizer(
            tokenizers::pre_tokenizers::whitespace::Whitespace {}
        );

        // 添加后处理器
        tokenizer.with_post_processor(
            tokenizers::processors::bert::BertProcessing::new(
                ("[SEP]".to_string(), 102),
                ("[CLS]".to_string(), 101),
            )
        );

        Ok(Self {
            tokenizer,
            max_length: config.max_length,
            pad_token_id: 0,
            cls_token_id: 101,
            sep_token_id: 102,
            config: config.clone(),
        })
    }

    /// 编码文本
    pub fn encode(&self, text: &str) -> Result<TokenizedInput> {
        // 预处理文本
        let processed_text = self.preprocess_text(text);

        // 使用分词器编码
        let encoding = self.tokenizer
            .encode(processed_text, false)
            .map_err(|e| Error::ModelError(format!("分词失败: {}", e)))?;

        // 构建输入序列
        let mut input_ids = vec![self.cls_token_id];
        input_ids.extend_from_slice(encoding.get_ids());

        // 截断到最大长度-1，为SEP token留空间
        if input_ids.len() >= self.max_length {
            input_ids.truncate(self.max_length - 1);
        }

        input_ids.push(self.sep_token_id);

        // 创建attention mask
        let attention_mask = vec![1u32; input_ids.len()];

        // 填充到最大长度
        let padding_length = self.max_length - input_ids.len();
        input_ids.extend(vec![self.pad_token_id; padding_length]);
        let mut attention_mask_padded = attention_mask;
        attention_mask_padded.extend(vec![0u32; padding_length]);

        Ok(TokenizedInput {
            input_ids,
            attention_mask: attention_mask_padded,
            sequence_length: self.max_length,
        })
    }

    /// 批量编码文本
    pub fn encode_batch(&self, texts: &[&str]) -> Result<Vec<TokenizedInput>> {
        let mut results = Vec::with_capacity(texts.len());
        
        for text in texts {
            results.push(self.encode(text)?);
        }
        
        Ok(results)
    }

    /// 预处理文本
    pub fn preprocess_text(&self, text: &str) -> String {
        // 1. 文本清理：移除多余空白字符
        let cleaned = text.trim();

        // 2. 合并多个连续空白字符为单个空格
        let normalized = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

        // 3. 长度检查和截断（保守估计，实际token数量可能更多）
        let estimated_tokens = normalized.chars().count();
        if estimated_tokens > self.max_length - 2 {
            // 简单截断，实际应用中可能需要更智能的截断策略
            normalized.chars().take(self.max_length - 2).collect()
        } else {
            normalized
        }
    }

    /// 解码token序列
    pub fn decode(&self, token_ids: &[u32]) -> Result<String> {
        self.tokenizer
            .decode(token_ids, true)
            .map_err(|e| Error::ModelError(format!("解码失败: {}", e)))
    }

    /// 获取词汇表大小
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.get_vocab_size(true)
    }

    /// 获取特殊token ID
    pub fn get_special_tokens(&self) -> SpecialTokens {
        SpecialTokens {
            pad_token_id: self.pad_token_id,
            cls_token_id: self.cls_token_id,
            sep_token_id: self.sep_token_id,
        }
    }

    /// 检查分词器是否已加载
    pub fn is_loaded(&self) -> bool {
        self.vocab_size() > 0
    }

    /// 获取配置
    pub fn get_config(&self) -> &BGEConfig {
        &self.config
    }
}

/// 特殊token ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialTokens {
    pub pad_token_id: u32,
    pub cls_token_id: u32,
    pub sep_token_id: u32,
}

impl TokenizedInput {
    /// 获取有效token数量（不包括padding）
    pub fn effective_length(&self) -> usize {
        self.attention_mask.iter().sum::<u32>() as usize
    }

    /// 检查是否包含padding
    pub fn has_padding(&self) -> bool {
        self.attention_mask.contains(&0)
    }

    /// 获取非padding的token IDs
    pub fn get_valid_tokens(&self) -> Vec<u32> {
        self.input_ids.iter()
            .zip(self.attention_mask.iter())
            .filter_map(|(&token, &mask)| if mask == 1 { Some(token) } else { None })
            .collect()
    }

    /// 转换为Tensor格式（为模型推理准备）
    pub fn to_tensor_data(&self) -> (Vec<u32>, Vec<u32>) {
        (self.input_ids.clone(), self.attention_mask.clone())
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

    #[test]
    fn test_placeholder_tokenizer() {
        let config = create_test_config();
        let tokenizer = BGETokenizer::placeholder(&config);
        assert!(tokenizer.is_ok());

        let tokenizer = tokenizer.unwrap();
        assert!(tokenizer.is_loaded());
        assert_eq!(tokenizer.get_config().max_length, 128);
    }

    #[test]
    fn test_tokenized_input() {
        let input = TokenizedInput {
            input_ids: vec![101, 1234, 5678, 102, 0, 0],
            attention_mask: vec![1, 1, 1, 1, 0, 0],
            sequence_length: 6,
        };

        assert_eq!(input.effective_length(), 4);
        assert!(input.has_padding());
        assert_eq!(input.get_valid_tokens(), vec![101, 1234, 5678, 102]);
    }

    #[test]
    fn test_special_tokens() {
        let config = create_test_config();
        let tokenizer = BGETokenizer::placeholder(&config).unwrap();
        let special_tokens = tokenizer.get_special_tokens();
        
        assert_eq!(special_tokens.cls_token_id, 101);
        assert_eq!(special_tokens.sep_token_id, 102);
        assert_eq!(special_tokens.pad_token_id, 0);
    }

    #[test]
    fn test_text_preprocessing() {
        let config = create_test_config();
        let tokenizer = BGETokenizer::placeholder(&config).unwrap();
        
        let text = "  这是一个   测试文本  ";
        let processed = tokenizer.preprocess_text(text);
        assert_eq!(processed, "这是一个 测试文本");
    }
}
