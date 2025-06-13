/// Demo showing the optimized file type handling and search functionality
use isearch::file_types::{FileTypeUtils, FileCategory};

fn main() {
    println!("=== SeeU Desktop 搜索功能优化演示 ===\n");

    // 演示文件类型分类
    println!("1. 文件类型分类演示:");
    let test_files = vec![
        "document.txt",
        "code.rs", 
        "image.jpg",
        "video.mp4",
        "archive.zip",
        "presentation.pdf",
        "spreadsheet.xlsx",
        "vector.svg",
        "data.csv",
        "unknown.xyz",
    ];

    for file in &test_files {
        let extension = file.split('.').last().unwrap_or("");
        let category = FileTypeUtils::get_category(extension);
        let is_previewable = FileTypeUtils::is_previewable(extension);
        let should_index = FileTypeUtils::should_index_content(extension);
        let icon = FileTypeUtils::get_icon(extension);
        
        println!("  {} {} - 分类: {:?}, 可预览: {}, 索引内容: {}", 
                 icon, file, category, 
                 if is_previewable { "✓" } else { "✗" },
                 if should_index { "✓" } else { "✗" });
    }

    println!("\n2. 预览消息演示:");
    let non_previewable_files = vec!["image.jpg", "video.mp4", "archive.zip", "app.exe"];
    for file in &non_previewable_files {
        let extension = file.split('.').last().unwrap_or("");
        let message = FileTypeUtils::get_non_previewable_message(extension);
        println!("  {} -> {}", file, message);
    }

    println!("\n3. 内容占位符演示:");
    let binary_files = vec!["photo.png", "movie.avi", "data.zip", "program.exe"];
    for file in &binary_files {
        let extension = file.split('.').last().unwrap_or("");
        let placeholder = FileTypeUtils::get_content_placeholder(extension);
        println!("  {} -> {}", file, placeholder);
    }

    println!("\n4. 可预览文件扩展名列表:");
    let previewable_exts = FileTypeUtils::get_previewable_extensions();
    println!("  支持的扩展名 ({} 种): {}", previewable_exts.len(), previewable_exts.join(", "));

    println!("\n5. 按分类查看文件扩展名:");
    let categories = vec![
        FileCategory::Text,
        FileCategory::Code, 
        FileCategory::Document,
        FileCategory::Image,
        FileCategory::Video,
        FileCategory::Audio,
        FileCategory::Archive,
    ];

    for category in categories {
        let extensions = FileTypeUtils::get_extensions_by_category(category.clone());
        println!("  {} {}: {}", 
                 category.icon(), 
                 category.display_name(), 
                 extensions.join(", "));
    }

    println!("\n=== 优化效果总结 ===");
    println!("✓ 只对可预览文件进行内容索引，节省存储空间");
    println!("✓ 对不可预览文件显示合适的提示信息");
    println!("✓ 支持 {} 种可预览文件格式", FileTypeUtils::get_previewable_extensions().len());
    println!("✓ 智能文件类型分类，提升用户体验");
    println!("✓ 内容大小限制，防止索引膨胀");
}
