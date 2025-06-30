//! 模型资源管理器
//! 
//! 负责模型的下载、缓存和生命周期管理

use crate::{Result, Error};
use crate::model::{ModelPaths, ModelDownloadInfo, ModelMetadata, ModelStatus};
use crate::embedding::provider::BGEVariant;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::fs;

/// 模型资源管理器
pub struct ModelResourceManager {
    cache_dir: PathBuf,
    http_client: reqwest::Client,
    models: Arc<Mutex<HashMap<String, ModelMetadata>>>,
}

impl ModelResourceManager {
    /// 创建新的模型资源管理器
    pub fn new(cache_dir: PathBuf) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5分钟超时
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            cache_dir,
            http_client,
            models: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 确保模型可用
    pub async fn ensure_model_available(&self, variant: BGEVariant) -> Result<ModelPaths> {
        let model_key = variant.model_name().to_string();
        
        // 检查模型状态
        {
            let models = self.models.lock().unwrap();
            if let Some(metadata) = models.get(&model_key) {
                if metadata.is_ready() {
                    if let Some(ref paths) = metadata.file_paths {
                        return Ok(paths.clone());
                    }
                }
            }
        }

        // 初始化模型元数据
        {
            let mut models = self.models.lock().unwrap();
            models.insert(model_key.clone(), ModelMetadata::new(variant.clone()));
        }

        // 获取模型路径
        let model_paths = self.get_model_paths(&variant);

        // 检查模型文件是否存在且有效
        if !self.validate_model_files(&model_paths).await? {
            self.download_model(&variant).await?;
        }

        // 更新模型状态
        {
            let mut models = self.models.lock().unwrap();
            if let Some(metadata) = models.get_mut(&model_key) {
                metadata.status = ModelStatus::Validated;
                metadata.file_paths = Some(model_paths.clone());
                metadata.validation_time = Some(chrono::Utc::now());
            }
        }

        Ok(model_paths)
    }

    /// 获取模型文件路径
    pub fn get_model_paths(&self, variant: &BGEVariant) -> ModelPaths {
        let model_dir = self.cache_dir.join(variant.model_name());
        
        ModelPaths {
            model_file: model_dir.join("pytorch_model.bin"),
            tokenizer_file: model_dir.join("tokenizer.json"),
            config_file: model_dir.join("config.json"),
        }
    }

    /// 验证模型文件
    async fn validate_model_files(&self, paths: &ModelPaths) -> Result<bool> {
        // 检查所有必需文件是否存在
        let files = [&paths.model_file, &paths.tokenizer_file, &paths.config_file];
        
        for file_path in &files {
            if !file_path.exists() {
                return Ok(false);
            }
            
            // 检查文件是否为空
            let metadata = fs::metadata(file_path).await?;
            if metadata.len() == 0 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// 下载模型
    async fn download_model(&self, variant: &BGEVariant) -> Result<()> {
        let model_key = variant.model_name().to_string();
        let model_info = ModelDownloadInfo::for_bge_variant(variant);
        let model_dir = self.cache_dir.join(variant.model_name());

        // 创建模型目录
        fs::create_dir_all(&model_dir).await?;

        // 更新下载状态
        {
            let mut models = self.models.lock().unwrap();
            if let Some(metadata) = models.get_mut(&model_key) {
                metadata.status = ModelStatus::Downloading(0.0);
            }
        }

        // 下载所有文件
        let total_files = model_info.files.len();
        for (i, file_info) in model_info.files.iter().enumerate() {
            let file_path = model_dir.join(&file_info.filename);
            
            // 检查文件是否已存在且有效
            if self.is_file_valid(&file_path, &file_info.sha256).await? {
                continue;
            }

            // 下载文件
            self.download_file(file_info, &file_path).await?;

            // 更新进度
            let progress = ((i + 1) as f32 / total_files as f32) * 100.0;
            {
                let mut models = self.models.lock().unwrap();
                if let Some(metadata) = models.get_mut(&model_key) {
                    metadata.status = ModelStatus::Downloading(progress);
                }
            }
        }

        // 验证下载完整性
        self.verify_model_integrity(&model_info, &model_dir).await?;

        // 更新状态
        {
            let mut models = self.models.lock().unwrap();
            if let Some(metadata) = models.get_mut(&model_key) {
                metadata.status = ModelStatus::Downloaded;
                metadata.download_time = Some(chrono::Utc::now());
            }
        }

        Ok(())
    }

    /// 下载单个文件
    async fn download_file(&self, file_info: &crate::model::FileInfo, file_path: &PathBuf) -> Result<()> {
        println!("📥 下载文件: {}", file_info.filename);

        let response = self.http_client
            .get(&file_info.url)
            .send()
            .await
            .map_err(|e| Error::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::NetworkError(format!(
                "下载失败，HTTP状态码: {}", 
                response.status()
            )));
        }

        let mut file = fs::File::create(file_path).await?;
        let mut stream = response.bytes_stream();

        use tokio::io::AsyncWriteExt;
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| Error::NetworkError(e.to_string()))?;
            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        println!("✅ 文件下载完成: {}", file_info.filename);

        Ok(())
    }

    /// 检查文件是否有效
    async fn is_file_valid(&self, file_path: &PathBuf, expected_sha256: &str) -> Result<bool> {
        if !file_path.exists() {
            return Ok(false);
        }

        // 暂时跳过SHA256验证，因为我们使用的是占位符哈希
        // 在实际部署时，应该使用真实的文件哈希进行验证
        if expected_sha256.starts_with("placeholder_") {
            // 只检查文件大小是否大于0
            let metadata = fs::metadata(file_path).await?;
            return Ok(metadata.len() > 0);
        }

        // 实际的SHA256验证逻辑
        let checksum = self.calculate_file_checksum(file_path).await?;
        Ok(checksum == expected_sha256)
    }

    /// 计算文件校验和
    async fn calculate_file_checksum(&self, file_path: &PathBuf) -> Result<String> {
        use sha2::{Sha256, Digest};
        use tokio::io::AsyncReadExt;

        let mut file = fs::File::open(file_path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// 验证模型完整性
    async fn verify_model_integrity(
        &self, 
        model_info: &ModelDownloadInfo, 
        model_dir: &PathBuf
    ) -> Result<()> {
        for file_info in &model_info.files {
            let file_path = model_dir.join(&file_info.filename);
            
            if !self.is_file_valid(&file_path, &file_info.sha256).await? {
                return Err(Error::ModelError(format!(
                    "文件校验失败: {}", 
                    file_info.filename
                )));
            }
        }

        Ok(())
    }

    /// 获取模型状态
    pub fn get_model_status(&self, variant: &BGEVariant) -> Option<ModelStatus> {
        let models = self.models.lock().unwrap();
        models.get(variant.model_name()).map(|m| m.status.clone())
    }

    /// 清理模型缓存
    pub async fn cleanup_model(&self, variant: &BGEVariant) -> Result<()> {
        let model_dir = self.cache_dir.join(variant.model_name());
        
        if model_dir.exists() {
            fs::remove_dir_all(&model_dir).await?;
        }

        // 从内存中移除
        {
            let mut models = self.models.lock().unwrap();
            models.remove(variant.model_name());
        }

        Ok(())
    }

    /// 获取所有模型状态
    pub fn get_all_model_status(&self) -> HashMap<String, ModelStatus> {
        let models = self.models.lock().unwrap();
        models.iter()
            .map(|(k, v)| (k.clone(), v.status.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_model_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelResourceManager::new(temp_dir.path().to_path_buf());
        
        assert!(manager.get_all_model_status().is_empty());
    }

    #[tokio::test]
    async fn test_model_paths() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelResourceManager::new(temp_dir.path().to_path_buf());
        
        let variant = BGEVariant::Small;
        let paths = manager.get_model_paths(&variant);
        
        assert!(paths.model_file.to_string_lossy().contains("bge-small-zh-v1.5"));
        assert!(paths.tokenizer_file.to_string_lossy().contains("tokenizer.json"));
        assert!(paths.config_file.to_string_lossy().contains("config.json"));
    }
}
