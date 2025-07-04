//! 文件管理器

use std::path::PathBuf;
use std::fs;
use crate::{FileEditorError, FileEditorResult};

/// 文件管理器
pub struct FileManager {
    /// 当前工作目录
    pub current_dir: Option<PathBuf>,
}

impl FileManager {
    pub fn new() -> Self {
        Self {
            current_dir: std::env::current_dir().ok(),
        }
    }
    
    /// 设置工作目录
    pub fn set_working_directory(&mut self, path: PathBuf) -> FileEditorResult<()> {
        if !path.exists() {
            return Err(FileEditorError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }
        
        if !path.is_dir() {
            return Err(FileEditorError::FileNotFound {
                path: format!("{} is not a directory", path.display()),
            });
        }
        
        self.current_dir = Some(path);
        Ok(())
    }
    
    /// 读取文件内容
    pub fn read_file(&self, path: &PathBuf) -> FileEditorResult<String> {
        if !path.exists() {
            return Err(FileEditorError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }
        
        fs::read_to_string(path).map_err(|e| FileEditorError::IoError(e))
    }
    
    /// 写入文件内容
    pub fn write_file(&self, path: &PathBuf, content: &str) -> FileEditorResult<()> {
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        fs::write(path, content).map_err(|e| FileEditorError::IoError(e))
    }
    
    /// 创建新文件
    pub fn create_file(&self, path: &PathBuf) -> FileEditorResult<()> {
        if path.exists() {
            return Err(FileEditorError::IoError(
                std::io::Error::new(std::io::ErrorKind::AlreadyExists, "File already exists")
            ));
        }
        
        self.write_file(path, "")
    }
    
    /// 删除文件
    pub fn delete_file(&self, path: &PathBuf) -> FileEditorResult<()> {
        if !path.exists() {
            return Err(FileEditorError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }
        
        if path.is_file() {
            fs::remove_file(path)?;
        } else if path.is_dir() {
            fs::remove_dir_all(path)?;
        }
        
        Ok(())
    }
    
    /// 重命名文件
    pub fn rename_file(&self, old_path: &PathBuf, new_path: &PathBuf) -> FileEditorResult<()> {
        if !old_path.exists() {
            return Err(FileEditorError::FileNotFound {
                path: old_path.to_string_lossy().to_string(),
            });
        }
        
        if new_path.exists() {
            return Err(FileEditorError::IoError(
                std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Target file already exists")
            ));
        }
        
        fs::rename(old_path, new_path)?;
        Ok(())
    }
    
    /// 复制文件
    pub fn copy_file(&self, src: &PathBuf, dst: &PathBuf) -> FileEditorResult<()> {
        if !src.exists() {
            return Err(FileEditorError::FileNotFound {
                path: src.to_string_lossy().to_string(),
            });
        }
        
        if src.is_file() {
            fs::copy(src, dst)?;
        } else {
            return Err(FileEditorError::IoError(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Cannot copy directory")
            ));
        }
        
        Ok(())
    }
    
    /// 获取文件信息
    pub fn get_file_info(&self, path: &PathBuf) -> FileEditorResult<FileInfo> {
        let metadata = fs::metadata(path)?;
        
        Ok(FileInfo {
            path: path.clone(),
            size: metadata.len(),
            modified: metadata.modified().ok(),
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
            readonly: metadata.permissions().readonly(),
        })
    }
    
    /// 列出目录内容
    pub fn list_directory(&self, path: &PathBuf) -> FileEditorResult<Vec<FileInfo>> {
        if !path.exists() {
            return Err(FileEditorError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }
        
        if !path.is_dir() {
            return Err(FileEditorError::IoError(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Path is not a directory")
            ));
        }
        
        let mut files = Vec::new();
        let entries = fs::read_dir(path)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if let Ok(info) = self.get_file_info(&path) {
                files.push(info);
            }
        }
        
        // 排序：目录在前，文件在后
        files.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.path.file_name().cmp(&b.path.file_name()),
            }
        });
        
        Ok(files)
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 文件信息
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<std::time::SystemTime>,
    pub is_dir: bool,
    pub is_file: bool,
    pub readonly: bool,
}
