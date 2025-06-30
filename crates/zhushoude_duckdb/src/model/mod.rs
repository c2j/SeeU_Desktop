//! 模型资源管理模块
//! 
//! 负责模型下载、验证和缓存管理

pub mod manager;
pub mod download;
pub mod validation;

pub use manager::ModelResourceManager;
pub use download::{ModelDownloader, DownloadProgress, DownloadStatus};
pub use validation::ModelValidator;

use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 模型文件路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPaths {
    pub model_file: PathBuf,
    pub tokenizer_file: PathBuf,
    pub config_file: PathBuf,
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub filename: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

/// 模型下载信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadInfo {
    pub variant: crate::embedding::provider::BGEVariant,
    pub files: Vec<FileInfo>,
}

impl ModelDownloadInfo {
    /// 获取BGE模型的下载信息
    pub fn for_bge_variant(variant: &crate::embedding::provider::BGEVariant) -> Self {
        use crate::embedding::provider::BGEVariant;
        
        let base_url = "https://huggingface.co/BAAI";
        let model_name = variant.model_name();
        
        let files = vec![
            FileInfo {
                filename: "pytorch_model.bin".to_string(),
                url: format!("{}/{}/resolve/main/pytorch_model.bin", base_url, model_name),
                sha256: Self::get_model_sha256(variant),
                size: Self::get_model_size(variant),
            },
            FileInfo {
                filename: "tokenizer.json".to_string(),
                url: format!("{}/{}/resolve/main/tokenizer.json", base_url, model_name),
                sha256: Self::get_tokenizer_sha256(variant),
                size: Self::get_tokenizer_size(variant),
            },
            FileInfo {
                filename: "config.json".to_string(),
                url: format!("{}/{}/resolve/main/config.json", base_url, model_name),
                sha256: Self::get_config_sha256(variant),
                size: Self::get_config_size(variant),
            },
        ];

        Self {
            variant: variant.clone(),
            files,
        }
    }

    // 注意：这些是占位符哈希值，实际使用时需要替换为真实的文件哈希
    fn get_model_sha256(variant: &crate::embedding::provider::BGEVariant) -> String {
        match variant {
            crate::embedding::provider::BGEVariant::Small => "placeholder_small_model_hash".to_string(),
            crate::embedding::provider::BGEVariant::Base => "placeholder_base_model_hash".to_string(),
            crate::embedding::provider::BGEVariant::Large => "placeholder_large_model_hash".to_string(),
        }
    }

    fn get_tokenizer_sha256(_variant: &crate::embedding::provider::BGEVariant) -> String {
        "placeholder_tokenizer_hash".to_string()
    }

    fn get_config_sha256(_variant: &crate::embedding::provider::BGEVariant) -> String {
        "placeholder_config_hash".to_string()
    }

    fn get_model_size(variant: &crate::embedding::provider::BGEVariant) -> u64 {
        match variant {
            crate::embedding::provider::BGEVariant::Small => 130 * 1024 * 1024,  // ~130MB
            crate::embedding::provider::BGEVariant::Base => 400 * 1024 * 1024,   // ~400MB
            crate::embedding::provider::BGEVariant::Large => 1300 * 1024 * 1024, // ~1.3GB
        }
    }

    fn get_tokenizer_size(_variant: &crate::embedding::provider::BGEVariant) -> u64 {
        2 * 1024 * 1024 // ~2MB
    }

    fn get_config_size(_variant: &crate::embedding::provider::BGEVariant) -> u64 {
        4 * 1024 // ~4KB
    }
}

/// 模型状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    NotDownloaded,
    Downloading(f32), // 进度百分比
    Downloaded,
    Validated,
    Loaded,
    Error(String),
}

/// 模型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub variant: crate::embedding::provider::BGEVariant,
    pub status: ModelStatus,
    pub download_time: Option<chrono::DateTime<chrono::Utc>>,
    pub validation_time: Option<chrono::DateTime<chrono::Utc>>,
    pub file_paths: Option<ModelPaths>,
    pub total_size_bytes: u64,
}

impl ModelMetadata {
    pub fn new(variant: crate::embedding::provider::BGEVariant) -> Self {
        let download_info = ModelDownloadInfo::for_bge_variant(&variant);
        let total_size = download_info.files.iter().map(|f| f.size).sum();

        Self {
            variant,
            status: ModelStatus::NotDownloaded,
            download_time: None,
            validation_time: None,
            file_paths: None,
            total_size_bytes: total_size,
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.status, ModelStatus::Validated | ModelStatus::Loaded)
    }

    pub fn is_downloading(&self) -> bool {
        matches!(self.status, ModelStatus::Downloading(_))
    }

    pub fn has_error(&self) -> bool {
        matches!(self.status, ModelStatus::Error(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::provider::BGEVariant;

    #[test]
    fn test_model_download_info() {
        let variant = BGEVariant::Small;
        let info = ModelDownloadInfo::for_bge_variant(&variant);
        
        assert_eq!(info.files.len(), 3);
        assert!(info.files.iter().any(|f| f.filename == "pytorch_model.bin"));
        assert!(info.files.iter().any(|f| f.filename == "tokenizer.json"));
        assert!(info.files.iter().any(|f| f.filename == "config.json"));
    }

    #[test]
    fn test_model_metadata() {
        let variant = BGEVariant::Base;
        let metadata = ModelMetadata::new(variant);
        
        assert!(matches!(metadata.status, ModelStatus::NotDownloaded));
        assert!(!metadata.is_ready());
        assert!(!metadata.is_downloading());
        assert!(!metadata.has_error());
        assert!(metadata.total_size_bytes > 0);
    }

    #[test]
    fn test_model_status_transitions() {
        let mut metadata = ModelMetadata::new(BGEVariant::Small);
        
        // 开始下载
        metadata.status = ModelStatus::Downloading(0.0);
        assert!(metadata.is_downloading());
        assert!(!metadata.is_ready());

        // 下载完成
        metadata.status = ModelStatus::Downloaded;
        assert!(!metadata.is_downloading());
        assert!(!metadata.is_ready());

        // 验证完成
        metadata.status = ModelStatus::Validated;
        assert!(metadata.is_ready());

        // 出现错误
        metadata.status = ModelStatus::Error("测试错误".to_string());
        assert!(metadata.has_error());
        assert!(!metadata.is_ready());
    }
}
