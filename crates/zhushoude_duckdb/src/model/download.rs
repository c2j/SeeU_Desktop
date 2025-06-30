//! 模型下载器
//! 
//! 提供模型文件的下载和进度跟踪功能

use crate::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

/// 下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: Option<u64>,
}

/// 下载状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed(String),
    Cancelled,
}

/// 模型下载器
pub struct ModelDownloader {
    http_client: reqwest::Client,
    download_progress: Arc<Mutex<HashMap<String, DownloadProgress>>>,
    max_concurrent_downloads: usize,
}

impl ModelDownloader {
    /// 创建新的下载器
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(600)) // 10分钟超时
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            http_client,
            download_progress: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent_downloads: 3,
        }
    }

    /// 下载文件并跟踪进度
    pub async fn download_with_progress(
        &self,
        url: &str,
        file_path: &PathBuf,
        file_id: &str,
    ) -> Result<()> {
        // 初始化进度
        {
            let mut progress_map = self.download_progress.lock().unwrap();
            progress_map.insert(file_id.to_string(), DownloadProgress {
                total_bytes: 0,
                downloaded_bytes: 0,
                status: DownloadStatus::Pending,
                speed_bytes_per_sec: 0,
                eta_seconds: None,
            });
        }

        // 开始下载
        self.update_status(file_id, DownloadStatus::Downloading);

        let result = self.download_file_internal(url, file_path, file_id).await;

        // 更新最终状态
        match &result {
            Ok(_) => self.update_status(file_id, DownloadStatus::Completed),
            Err(e) => self.update_status(file_id, DownloadStatus::Failed(e.to_string())),
        }

        result
    }

    /// 内部下载实现
    async fn download_file_internal(
        &self,
        url: &str,
        file_path: &PathBuf,
        file_id: &str,
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        use futures_util::StreamExt;

        // 创建父目录
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // 发送请求
        let response = self.http_client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::NetworkError(format!(
                "HTTP错误: {}", 
                response.status()
            )));
        }

        // 获取文件大小
        let total_size = response.content_length().unwrap_or(0);
        self.update_total_size(file_id, total_size);

        // 创建文件
        let mut file = tokio::fs::File::create(file_path).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let start_time = std::time::Instant::now();

        // 下载数据
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| Error::NetworkError(e.to_string()))?;
            
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // 更新进度
            let elapsed = start_time.elapsed().as_secs();
            let speed = if elapsed > 0 { downloaded / elapsed } else { 0 };
            let eta = if speed > 0 && total_size > downloaded {
                Some((total_size - downloaded) / speed)
            } else {
                None
            };

            self.update_progress(file_id, downloaded, speed, eta);
        }

        file.flush().await?;
        Ok(())
    }

    /// 更新下载状态
    fn update_status(&self, file_id: &str, status: DownloadStatus) {
        let mut progress_map = self.download_progress.lock().unwrap();
        if let Some(progress) = progress_map.get_mut(file_id) {
            progress.status = status;
        }
    }

    /// 更新总大小
    fn update_total_size(&self, file_id: &str, total_bytes: u64) {
        let mut progress_map = self.download_progress.lock().unwrap();
        if let Some(progress) = progress_map.get_mut(file_id) {
            progress.total_bytes = total_bytes;
        }
    }

    /// 更新下载进度
    fn update_progress(&self, file_id: &str, downloaded: u64, speed: u64, eta: Option<u64>) {
        let mut progress_map = self.download_progress.lock().unwrap();
        if let Some(progress) = progress_map.get_mut(file_id) {
            progress.downloaded_bytes = downloaded;
            progress.speed_bytes_per_sec = speed;
            progress.eta_seconds = eta;
        }
    }

    /// 获取下载进度
    pub fn get_progress(&self, file_id: &str) -> Option<DownloadProgress> {
        let progress_map = self.download_progress.lock().unwrap();
        progress_map.get(file_id).cloned()
    }

    /// 获取所有下载进度
    pub fn get_all_progress(&self) -> HashMap<String, DownloadProgress> {
        let progress_map = self.download_progress.lock().unwrap();
        progress_map.clone()
    }

    /// 取消下载
    pub fn cancel_download(&self, file_id: &str) {
        self.update_status(file_id, DownloadStatus::Cancelled);
    }

    /// 清理完成的下载记录
    pub fn cleanup_completed(&self) {
        let mut progress_map = self.download_progress.lock().unwrap();
        progress_map.retain(|_, progress| {
            !matches!(progress.status, DownloadStatus::Completed | DownloadStatus::Failed(_))
        });
    }

    /// 并发下载多个文件
    pub async fn download_multiple(
        &self,
        downloads: Vec<(String, PathBuf, String)>, // (url, path, file_id)
    ) -> Result<Vec<Result<()>>> {
        use futures_util::stream::{self, StreamExt};

        let results = stream::iter(downloads)
            .map(|(url, path, file_id)| async move {
                self.download_with_progress(&url, &path, &file_id).await
            })
            .buffer_unordered(self.max_concurrent_downloads)
            .collect::<Vec<_>>()
            .await;

        Ok(results)
    }
}

impl Default for ModelDownloader {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadProgress {
    /// 获取下载百分比
    pub fn percentage(&self) -> f32 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f32 / self.total_bytes as f32) * 100.0
        }
    }

    /// 格式化速度显示
    pub fn format_speed(&self) -> String {
        format_bytes_per_sec(self.speed_bytes_per_sec)
    }

    /// 格式化ETA显示
    pub fn format_eta(&self) -> String {
        match self.eta_seconds {
            Some(seconds) => format_duration(seconds),
            None => "未知".to_string(),
        }
    }

    /// 格式化大小显示
    pub fn format_size(&self) -> String {
        format!("{} / {}", 
            format_bytes(self.downloaded_bytes),
            format_bytes(self.total_bytes)
        )
    }
}

/// 格式化字节数
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

/// 格式化速度
fn format_bytes_per_sec(bytes_per_sec: u64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec))
}

/// 格式化时间
fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_progress() {
        let progress = DownloadProgress {
            total_bytes: 1024 * 1024, // 1MB
            downloaded_bytes: 512 * 1024, // 512KB
            status: DownloadStatus::Downloading,
            speed_bytes_per_sec: 1024 * 100, // 100KB/s
            eta_seconds: Some(5),
        };

        assert_eq!(progress.percentage(), 50.0);
        assert_eq!(progress.format_speed(), "100.0 KB/s");
        assert_eq!(progress.format_eta(), "5s");
        assert_eq!(progress.format_size(), "512.0 KB / 1.0 MB");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
    }
}
