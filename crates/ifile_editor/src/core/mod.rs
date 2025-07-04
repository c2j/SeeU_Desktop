//! 核心模块

pub mod file_manager;
pub mod text_buffer;
pub mod text_buffer_adapter;
pub mod syntax;

pub use file_manager::FileManager;
pub use text_buffer::TextBufferManager;
pub use syntax::SyntaxHighlighter;
