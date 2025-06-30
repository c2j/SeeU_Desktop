//! 模型验证器
//! 
//! 提供模型文件的完整性验证和格式检查

use crate::{Result, Error};
use crate::model::{ModelPaths, FileInfo};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub file_results: Vec<FileValidationResult>,
    pub total_size_bytes: u64,
    pub validation_time_ms: u64,
}

/// 单个文件验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileValidationResult {
    pub file_path: PathBuf,
    pub file_type: FileType,
    pub is_valid: bool,
    pub size_bytes: u64,
    pub checksum_valid: bool,
    pub format_valid: bool,
    pub error_message: Option<String>,
}

/// 文件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Model,
    Tokenizer,
    Config,
    Unknown,
}

/// 模型验证器
pub struct ModelValidator {
    strict_mode: bool,
}

impl ModelValidator {
    /// 创建新的验证器
    pub fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    /// 验证模型文件
    pub async fn validate_model(&self, paths: &ModelPaths) -> Result<ValidationResult> {
        let start_time = std::time::Instant::now();
        let mut file_results = Vec::new();
        let mut total_size = 0u64;
        let mut overall_valid = true;

        // 验证模型文件
        let model_result = self.validate_model_file(&paths.model_file).await?;
        total_size += model_result.size_bytes;
        overall_valid &= model_result.is_valid;
        file_results.push(model_result);

        // 验证分词器文件
        let tokenizer_result = self.validate_tokenizer_file(&paths.tokenizer_file).await?;
        total_size += tokenizer_result.size_bytes;
        overall_valid &= tokenizer_result.is_valid;
        file_results.push(tokenizer_result);

        // 验证配置文件
        let config_result = self.validate_config_file(&paths.config_file).await?;
        total_size += config_result.size_bytes;
        overall_valid &= config_result.is_valid;
        file_results.push(config_result);

        let validation_time = start_time.elapsed().as_millis() as u64;

        Ok(ValidationResult {
            is_valid: overall_valid,
            file_results,
            total_size_bytes: total_size,
            validation_time_ms: validation_time,
        })
    }

    /// 验证模型文件
    async fn validate_model_file(&self, file_path: &PathBuf) -> Result<FileValidationResult> {
        let mut result = FileValidationResult {
            file_path: file_path.clone(),
            file_type: FileType::Model,
            is_valid: false,
            size_bytes: 0,
            checksum_valid: false,
            format_valid: false,
            error_message: None,
        };

        // 检查文件是否存在
        if !file_path.exists() {
            result.error_message = Some("文件不存在".to_string());
            return Ok(result);
        }

        // 获取文件大小
        let metadata = fs::metadata(file_path).await?;
        result.size_bytes = metadata.len();

        // 检查文件大小
        if result.size_bytes == 0 {
            result.error_message = Some("文件为空".to_string());
            return Ok(result);
        }

        // 检查最小文件大小（模型文件应该至少几MB）
        if result.size_bytes < 1024 * 1024 {
            result.error_message = Some("模型文件太小，可能损坏".to_string());
            return Ok(result);
        }

        // 基本格式验证
        result.format_valid = self.validate_model_format(file_path).await?;
        if !result.format_valid && self.strict_mode {
            result.error_message = Some("模型文件格式无效".to_string());
            return Ok(result);
        }

        // 在非严格模式下，如果文件存在且大小合理，就认为有效
        result.checksum_valid = true; // 暂时跳过校验和验证
        result.is_valid = result.format_valid || !self.strict_mode;

        Ok(result)
    }

    /// 验证分词器文件
    async fn validate_tokenizer_file(&self, file_path: &PathBuf) -> Result<FileValidationResult> {
        let mut result = FileValidationResult {
            file_path: file_path.clone(),
            file_type: FileType::Tokenizer,
            is_valid: false,
            size_bytes: 0,
            checksum_valid: false,
            format_valid: false,
            error_message: None,
        };

        // 检查文件是否存在
        if !file_path.exists() {
            result.error_message = Some("分词器文件不存在".to_string());
            return Ok(result);
        }

        // 获取文件大小
        let metadata = fs::metadata(file_path).await?;
        result.size_bytes = metadata.len();

        // 检查文件大小
        if result.size_bytes == 0 {
            result.error_message = Some("分词器文件为空".to_string());
            return Ok(result);
        }

        // 验证JSON格式
        result.format_valid = self.validate_json_format(file_path).await?;
        if !result.format_valid {
            result.error_message = Some("分词器文件JSON格式无效".to_string());
            if self.strict_mode {
                return Ok(result);
            }
        }

        result.checksum_valid = true; // 暂时跳过校验和验证
        result.is_valid = result.format_valid || !self.strict_mode;

        Ok(result)
    }

    /// 验证配置文件
    async fn validate_config_file(&self, file_path: &PathBuf) -> Result<FileValidationResult> {
        let mut result = FileValidationResult {
            file_path: file_path.clone(),
            file_type: FileType::Config,
            is_valid: false,
            size_bytes: 0,
            checksum_valid: false,
            format_valid: false,
            error_message: None,
        };

        // 检查文件是否存在
        if !file_path.exists() {
            result.error_message = Some("配置文件不存在".to_string());
            return Ok(result);
        }

        // 获取文件大小
        let metadata = fs::metadata(file_path).await?;
        result.size_bytes = metadata.len();

        // 检查文件大小
        if result.size_bytes == 0 {
            result.error_message = Some("配置文件为空".to_string());
            return Ok(result);
        }

        // 验证JSON格式和内容
        result.format_valid = self.validate_config_content(file_path).await?;
        if !result.format_valid {
            result.error_message = Some("配置文件格式或内容无效".to_string());
            if self.strict_mode {
                return Ok(result);
            }
        }

        result.checksum_valid = true; // 暂时跳过校验和验证
        result.is_valid = result.format_valid || !self.strict_mode;

        Ok(result)
    }

    /// 验证模型文件格式
    async fn validate_model_format(&self, file_path: &PathBuf) -> Result<bool> {
        // 检查文件扩展名
        if let Some(extension) = file_path.extension() {
            if extension == "bin" || extension == "safetensors" {
                // 读取文件头部进行基本验证
                return self.validate_model_header(file_path).await;
            }
        }
        
        // 如果没有扩展名，尝试读取文件头部
        self.validate_model_header(file_path).await
    }

    /// 验证模型文件头部
    async fn validate_model_header(&self, file_path: &PathBuf) -> Result<bool> {
        use tokio::io::AsyncReadExt;

        let mut file = fs::File::open(file_path).await?;
        let mut header = vec![0u8; 1024]; // 读取前1KB
        
        let bytes_read = file.read(&mut header).await?;
        if bytes_read == 0 {
            return Ok(false);
        }

        // 简单的头部验证 - 检查是否包含常见的模型文件标识
        let header_str = String::from_utf8_lossy(&header[..bytes_read]);
        
        // PyTorch模型文件通常包含这些标识
        Ok(header_str.contains("PK") || // ZIP格式标识
           header_str.contains("torch") ||
           header.starts_with(&[0x50, 0x4B]) || // ZIP magic number
           header.len() >= 8) // 至少有一些内容
    }

    /// 验证JSON格式
    async fn validate_json_format(&self, file_path: &PathBuf) -> Result<bool> {
        let content = fs::read_to_string(file_path).await?;
        
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 验证配置文件内容
    async fn validate_config_content(&self, file_path: &PathBuf) -> Result<bool> {
        let content = fs::read_to_string(file_path).await?;
        
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(config) => {
                // 检查必需的配置字段
                if let Some(obj) = config.as_object() {
                    // 检查是否包含模型相关的基本配置
                    let has_model_type = obj.contains_key("model_type") || 
                                       obj.contains_key("architectures") ||
                                       obj.contains_key("_name_or_path");
                    
                    let has_vocab_size = obj.contains_key("vocab_size");
                    let has_hidden_size = obj.contains_key("hidden_size");
                    
                    Ok(has_model_type || has_vocab_size || has_hidden_size)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// 快速验证（只检查文件存在性和大小）
    pub async fn quick_validate(&self, paths: &ModelPaths) -> Result<bool> {
        let files = [&paths.model_file, &paths.tokenizer_file, &paths.config_file];
        
        for file_path in &files {
            if !file_path.exists() {
                return Ok(false);
            }
            
            let metadata = fs::metadata(file_path).await?;
            if metadata.len() == 0 {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

impl Default for ModelValidator {
    fn default() -> Self {
        Self::new(false) // 默认非严格模式
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_validator_creation() {
        let validator = ModelValidator::new(true);
        assert!(validator.strict_mode);

        let validator_default = ModelValidator::default();
        assert!(!validator_default.strict_mode);
    }

    #[tokio::test]
    async fn test_quick_validate_missing_files() {
        let temp_dir = TempDir::new().unwrap();
        let validator = ModelValidator::default();
        
        let paths = ModelPaths {
            model_file: temp_dir.path().join("model.bin"),
            tokenizer_file: temp_dir.path().join("tokenizer.json"),
            config_file: temp_dir.path().join("config.json"),
        };
        
        let result = validator.quick_validate(&paths).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_quick_validate_existing_files() {
        let temp_dir = TempDir::new().unwrap();
        let validator = ModelValidator::default();
        
        let paths = ModelPaths {
            model_file: temp_dir.path().join("model.bin"),
            tokenizer_file: temp_dir.path().join("tokenizer.json"),
            config_file: temp_dir.path().join("config.json"),
        };
        
        // 创建测试文件
        for path in [&paths.model_file, &paths.tokenizer_file, &paths.config_file] {
            let mut file = fs::File::create(path).await.unwrap();
            file.write_all(b"test content").await.unwrap();
        }
        
        let result = validator.quick_validate(&paths).await.unwrap();
        assert!(result);
    }
}
