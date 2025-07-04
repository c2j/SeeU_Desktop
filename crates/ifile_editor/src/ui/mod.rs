//! UI 模块

pub mod main_ui;
pub mod file_tree;
pub mod editor;
pub mod code_editor;
pub mod tabs;

pub use main_ui::render_file_editor;
pub use file_tree::render_file_tree;
