//! iFile Editor - 高性能文件编辑器模块
//! 
//! 基于 ROPE 数据结构和 egui_ltreeview 的现代文件编辑器，
//! 支持大文件编辑、语法高亮、文件树浏览等功能。

pub mod state;
pub mod ui;
pub mod core;
pub mod settings;

// 重新导出主要类型
pub use state::*;
pub use ui::render_file_editor;
pub use settings::FileEditorSettingsModule;

use anyhow::Result;

/// 初始化文件编辑器模块
pub fn initialize() -> IFileEditorState {
    IFileEditorState::new()
}

/// 文件编辑器错误类型
#[derive(Debug, thiserror::Error)]
pub enum FileEditorError {
    #[error("文件不存在: {path}")]
    FileNotFound { path: String },
    
    #[error("文件过大: {size_mb}MB (最大: {max_mb}MB)")]
    FileTooLarge { size_mb: usize, max_mb: usize },
    
    #[error("编码错误: {encoding}")]
    EncodingError { encoding: String },
    
    #[error("权限错误: {path}")]
    PermissionDenied { path: String },
    
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("ROPE 操作错误: {0}")]
    RopeError(String),
    
    #[error("语法高亮错误: {0}")]
    SyntaxError(String),
}

/// 文件编辑器结果类型
pub type FileEditorResult<T> = Result<T, FileEditorError>;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let state = initialize();
        assert!(!state.initialized);
        assert!(state.file_tree.root_path.is_none());
        assert!(state.editor.buffers.is_empty());
    }
}
