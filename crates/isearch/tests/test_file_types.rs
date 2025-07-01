use isearch::file_types::{FileCategory, FileTypeUtils};

#[test]
fn test_file_category_detection() {
    // 测试各种文件类型检测
    assert_eq!(FileTypeUtils::get_category("rs"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("py"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("js"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("ts"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("java"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("cpp"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("c"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("h"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("go"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("php"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("rb"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("swift"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("kt"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("scala"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("html"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("css"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("scss"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("less"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("xml"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("json"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("yaml"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("yml"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("toml"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("md"), FileCategory::Text);
    assert_eq!(FileTypeUtils::get_category("txt"), FileCategory::Text);
    assert_eq!(FileTypeUtils::get_category("log"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("sql"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("sh"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("bat"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("ps1"), FileCategory::Code);
    assert_eq!(FileTypeUtils::get_category("pdf"), FileCategory::Document);
    assert_eq!(FileTypeUtils::get_category("jpg"), FileCategory::Image);
    assert_eq!(FileTypeUtils::get_category("mp4"), FileCategory::Video);
    assert_eq!(FileTypeUtils::get_category("zip"), FileCategory::Archive);
    assert_eq!(FileTypeUtils::get_category("exe"), FileCategory::Executable);
    assert_eq!(FileTypeUtils::get_category("unknown"), FileCategory::Binary);
}

#[test]
fn test_file_category_display() {
    assert_eq!(FileCategory::Code.display_name(), "代码文件");
    assert_eq!(FileCategory::Text.display_name(), "文本文件");
    assert_eq!(FileCategory::Document.display_name(), "文档文件");
    assert_eq!(FileCategory::Image.display_name(), "图片文件");
    assert_eq!(FileCategory::Binary.display_name(), "二进制文件");
}

#[test]
fn test_file_previewable() {
    // 测试可预览的文件类型
    assert!(FileTypeUtils::is_previewable("txt"));
    assert!(FileTypeUtils::is_previewable("rs"));
    assert!(FileTypeUtils::is_previewable("py"));
    assert!(FileTypeUtils::is_previewable("js"));
    assert!(FileTypeUtils::is_previewable("md"));
    assert!(FileTypeUtils::is_previewable("json"));
    assert!(FileTypeUtils::is_previewable("xml"));
    assert!(FileTypeUtils::is_previewable("html"));
    assert!(FileTypeUtils::is_previewable("css"));
    assert!(FileTypeUtils::is_previewable("yaml"));
    assert!(FileTypeUtils::is_previewable("toml"));
    assert!(FileTypeUtils::is_previewable("sql"));
    assert!(FileTypeUtils::is_previewable("sh"));
    assert!(FileTypeUtils::is_previewable("pdf"));
    assert!(FileTypeUtils::is_previewable("svg"));
    
    // 测试不可预览的文件类型
    assert!(!FileTypeUtils::is_previewable("exe"));
    assert!(!FileTypeUtils::is_previewable("bin"));
    assert!(!FileTypeUtils::is_previewable("dll"));
    assert!(!FileTypeUtils::is_previewable("so"));
    assert!(!FileTypeUtils::is_previewable("jpg"));
    assert!(!FileTypeUtils::is_previewable("png"));
    assert!(!FileTypeUtils::is_previewable("gif"));
    assert!(!FileTypeUtils::is_previewable("zip"));
    assert!(!FileTypeUtils::is_previewable("tar"));
    assert!(!FileTypeUtils::is_previewable("gz"));
    assert!(!FileTypeUtils::is_previewable("mp3"));
    assert!(!FileTypeUtils::is_previewable("mp4"));
    assert!(!FileTypeUtils::is_previewable("avi"));
}

#[test]
fn test_file_content_indexing() {
    // 测试应该索引内容的文件类型
    assert!(FileTypeUtils::should_index_content("txt"));
    assert!(FileTypeUtils::should_index_content("rs"));
    assert!(FileTypeUtils::should_index_content("py"));
    assert!(FileTypeUtils::should_index_content("js"));
    assert!(FileTypeUtils::should_index_content("md"));
    assert!(FileTypeUtils::should_index_content("json"));
    assert!(FileTypeUtils::should_index_content("xml"));
    assert!(FileTypeUtils::should_index_content("html"));
    assert!(FileTypeUtils::should_index_content("css"));
    assert!(FileTypeUtils::should_index_content("yaml"));
    assert!(FileTypeUtils::should_index_content("toml"));
    assert!(FileTypeUtils::should_index_content("sql"));
    assert!(FileTypeUtils::should_index_content("sh"));
    assert!(FileTypeUtils::should_index_content("svg"));
    
    // 测试不应该索引内容的文件类型
    assert!(!FileTypeUtils::should_index_content("exe"));
    assert!(!FileTypeUtils::should_index_content("bin"));
    assert!(!FileTypeUtils::should_index_content("dll"));
    assert!(!FileTypeUtils::should_index_content("so"));
    assert!(!FileTypeUtils::should_index_content("jpg"));
    assert!(!FileTypeUtils::should_index_content("png"));
    assert!(!FileTypeUtils::should_index_content("gif"));
    assert!(!FileTypeUtils::should_index_content("zip"));
    assert!(!FileTypeUtils::should_index_content("tar"));
    assert!(!FileTypeUtils::should_index_content("gz"));
    assert!(!FileTypeUtils::should_index_content("mp3"));
    assert!(!FileTypeUtils::should_index_content("mp4"));
    assert!(!FileTypeUtils::should_index_content("avi"));
}

#[test]
fn test_file_type_icon() {
    // 测试文件类型图标（使用实际的图标）
    assert_eq!(FileTypeUtils::get_icon("rs"), "💻");
    assert_eq!(FileTypeUtils::get_icon("py"), "💻");
    assert_eq!(FileTypeUtils::get_icon("js"), "💻");
    assert_eq!(FileTypeUtils::get_icon("html"), "💻");
    assert_eq!(FileTypeUtils::get_icon("css"), "💻");
    assert_eq!(FileTypeUtils::get_icon("json"), "💻");
    assert_eq!(FileTypeUtils::get_icon("md"), "📃");
    assert_eq!(FileTypeUtils::get_icon("txt"), "📃");
    assert_eq!(FileTypeUtils::get_icon("pdf"), "📄");
    assert_eq!(FileTypeUtils::get_icon("jpg"), "🖼");
    assert_eq!(FileTypeUtils::get_icon("mp4"), "🎬");
    assert_eq!(FileTypeUtils::get_icon("zip"), "📦");
    assert_eq!(FileTypeUtils::get_icon("exe"), "⚙️");
    assert_eq!(FileTypeUtils::get_icon("unknown"), "📄");
}

#[test]
fn test_file_extensions_by_category() {
    let text_extensions = FileTypeUtils::get_extensions_by_category(FileCategory::Text);
    assert!(text_extensions.contains(&"txt"));
    assert!(text_extensions.contains(&"md"));
    assert!(text_extensions.contains(&"markdown"));
    
    let code_extensions = FileTypeUtils::get_extensions_by_category(FileCategory::Code);
    assert!(code_extensions.contains(&"rs"));
    assert!(code_extensions.contains(&"py"));
    assert!(code_extensions.contains(&"js"));
    assert!(code_extensions.contains(&"html"));
    
    let image_extensions = FileTypeUtils::get_extensions_by_category(FileCategory::Image);
    assert!(image_extensions.contains(&"jpg"));
    assert!(image_extensions.contains(&"png"));
    assert!(image_extensions.contains(&"gif"));
}
