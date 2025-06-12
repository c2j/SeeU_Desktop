/// File type classification and handling utilities

#[derive(Debug, Clone, PartialEq)]
pub enum FileCategory {
    Text,
    Code,
    Document,
    Spreadsheet,
    Presentation,
    Image,
    Video,
    Audio,
    Archive,
    Executable,
    Binary,
}

impl FileCategory {
    /// Get the display name for the file category
    pub fn display_name(&self) -> &'static str {
        match self {
            FileCategory::Text => "文本文件",
            FileCategory::Code => "代码文件",
            FileCategory::Document => "文档文件",
            FileCategory::Spreadsheet => "表格文件",
            FileCategory::Presentation => "演示文稿",
            FileCategory::Image => "图片文件",
            FileCategory::Video => "视频文件",
            FileCategory::Audio => "音频文件",
            FileCategory::Archive => "压缩文件",
            FileCategory::Executable => "可执行文件",
            FileCategory::Binary => "二进制文件",
        }
    }

    /// Get the emoji icon for the file category
    pub fn icon(&self) -> &'static str {
        match self {
            FileCategory::Text => "📃",
            FileCategory::Code => "💻",
            FileCategory::Document => "📄",
            FileCategory::Spreadsheet => "📊",
            FileCategory::Presentation => "📽",
            FileCategory::Image => "🖼",
            FileCategory::Video => "🎬",
            FileCategory::Audio => "🎵",
            FileCategory::Archive => "📦",
            FileCategory::Executable => "⚙️",
            FileCategory::Binary => "📄",
        }
    }

    /// Check if this file category supports content preview
    pub fn is_previewable(&self) -> bool {
        matches!(self, 
            FileCategory::Text | 
            FileCategory::Code | 
            FileCategory::Document // PDF with proper extraction
        )
    }

    /// Check if this file category should have its content indexed for search
    pub fn should_index_content(&self) -> bool {
        self.is_previewable()
    }
}

/// File type utilities
pub struct FileTypeUtils;

impl FileTypeUtils {
    /// Get file category from file extension
    pub fn get_category(file_extension: &str) -> FileCategory {
        let ext = file_extension.to_lowercase();
        match ext.as_str() {
            // Text files
            "txt" | "md" | "markdown" | "rst" | "rtf" => FileCategory::Text,
            
            // Code files
            "rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" | "h" | "hpp" | 
            "cs" | "php" | "rb" | "go" | "swift" | "kt" | "scala" | "clj" |
            "html" | "htm" | "css" | "scss" | "sass" | "less" |
            "json" | "xml" | "yaml" | "yml" | "toml" | "ini" | "cfg" |
            "sh" | "bash" | "zsh" | "fish" | "bat" | "ps1" | "cmd" |
            "sql" | "sqlite" | "csv" | "tsv" | "log" => FileCategory::Code,
            
            // Document files
            "pdf" => FileCategory::Document,
            "doc" | "docx" | "odt" => FileCategory::Document,
            
            // Spreadsheet files
            "xls" | "xlsx" | "ods" => FileCategory::Spreadsheet,
            
            // Presentation files
            "ppt" | "pptx" | "odp" => FileCategory::Presentation,
            
            // Image files (SVG is handled separately as it's text-based)
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" |
            "ico" | "tiff" | "tif" | "raw" | "cr2" | "nef" | "arw" => FileCategory::Image,

            // SVG is text-based and should be treated as code for preview purposes
            "svg" => FileCategory::Code,
            
            // Video files
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | 
            "m4v" | "3gp" | "ogv" | "f4v" => FileCategory::Video,
            
            // Audio files
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | 
            "opus" | "ape" | "ac3" => FileCategory::Audio,
            
            // Archive files
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | 
            "lz" | "lzma" | "z" | "cab" | "iso" | "dmg" => FileCategory::Archive,
            
            // Executable files
            "exe" | "msi" | "app" | "deb" | "rpm" | "pkg" | 
            "run" | "bin" | "com" | "scr" => FileCategory::Executable,
            
            // Default to binary for unknown types
            _ => FileCategory::Binary,
        }
    }

    /// Check if a file type is previewable
    pub fn is_previewable(file_extension: &str) -> bool {
        Self::get_category(file_extension).is_previewable()
    }

    /// Check if a file type should have its content indexed
    pub fn should_index_content(file_extension: &str) -> bool {
        Self::get_category(file_extension).should_index_content()
    }

    /// Get display icon for file extension
    pub fn get_icon(file_extension: &str) -> &'static str {
        Self::get_category(file_extension).icon()
    }

    /// Get display name for file extension
    pub fn get_display_name(file_extension: &str) -> String {
        let category = Self::get_category(file_extension);
        format!("{} ({})", category.display_name(), file_extension.to_uppercase())
    }

    /// Get preview message for non-previewable files
    pub fn get_non_previewable_message(file_extension: &str) -> String {
        let category = Self::get_category(file_extension);
        format!("{} - 无法预览内容", category.display_name())
    }

    /// Get content placeholder for indexing non-previewable files
    pub fn get_content_placeholder(file_extension: &str) -> String {
        let category = Self::get_category(file_extension);
        format!("{} ({})", category.display_name(), file_extension.to_uppercase())
    }

    /// Get list of all supported previewable file extensions
    pub fn get_previewable_extensions() -> Vec<&'static str> {
        vec![
            // Text files
            "txt", "md", "markdown", "rst", "rtf",
            
            // Code files
            "rs", "js", "ts", "py", "java", "cpp", "c", "h", "hpp",
            "cs", "php", "rb", "go", "swift", "kt", "scala", "clj",
            "html", "htm", "css", "scss", "sass", "less",
            "json", "xml", "yaml", "yml", "toml", "ini", "cfg",
            "sh", "bash", "zsh", "fish", "bat", "ps1", "cmd",
            "sql", "sqlite", "csv", "tsv", "log", "svg",
            
            // PDF (with proper text extraction)
            "pdf",
        ]
    }

    /// Get list of common file extensions by category
    pub fn get_extensions_by_category(category: FileCategory) -> Vec<&'static str> {
        match category {
            FileCategory::Text => vec!["txt", "md", "markdown", "rst", "rtf"],
            FileCategory::Code => vec![
                "rs", "js", "ts", "py", "java", "cpp", "c", "h", "hpp",
                "cs", "php", "rb", "go", "swift", "kt", "scala", "clj",
                "html", "htm", "css", "scss", "sass", "less",
                "json", "xml", "yaml", "yml", "toml", "ini", "cfg",
                "sh", "bash", "zsh", "fish", "bat", "ps1", "cmd",
                "sql", "sqlite", "csv", "tsv", "log", "svg"
            ],
            FileCategory::Document => vec!["pdf", "doc", "docx", "odt"],
            FileCategory::Spreadsheet => vec!["xls", "xlsx", "ods"],
            FileCategory::Presentation => vec!["ppt", "pptx", "odp"],
            FileCategory::Image => vec![
                "jpg", "jpeg", "png", "gif", "bmp", "webp",
                "ico", "tiff", "tif", "raw", "cr2", "nef", "arw"
            ],
            FileCategory::Video => vec![
                "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm",
                "m4v", "3gp", "ogv", "f4v"
            ],
            FileCategory::Audio => vec![
                "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma",
                "opus", "ape", "ac3"
            ],
            FileCategory::Archive => vec![
                "zip", "rar", "7z", "tar", "gz", "bz2", "xz",
                "lz", "lzma", "z", "cab", "iso", "dmg"
            ],
            FileCategory::Executable => vec![
                "exe", "msi", "app", "deb", "rpm", "pkg",
                "run", "bin", "com", "scr"
            ],
            FileCategory::Binary => vec![], // No specific extensions for binary
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_category_classification() {
        assert_eq!(FileTypeUtils::get_category("txt"), FileCategory::Text);
        assert_eq!(FileTypeUtils::get_category("rs"), FileCategory::Code);
        assert_eq!(FileTypeUtils::get_category("pdf"), FileCategory::Document);
        assert_eq!(FileTypeUtils::get_category("jpg"), FileCategory::Image);
        assert_eq!(FileTypeUtils::get_category("mp4"), FileCategory::Video);
        assert_eq!(FileTypeUtils::get_category("mp3"), FileCategory::Audio);
        assert_eq!(FileTypeUtils::get_category("zip"), FileCategory::Archive);
        assert_eq!(FileTypeUtils::get_category("exe"), FileCategory::Executable);
        assert_eq!(FileTypeUtils::get_category("unknown"), FileCategory::Binary);
    }

    #[test]
    fn test_previewable_files() {
        assert!(FileTypeUtils::is_previewable("txt"));
        assert!(FileTypeUtils::is_previewable("rs"));
        assert!(FileTypeUtils::is_previewable("pdf"));
        assert!(FileTypeUtils::is_previewable("svg"));
        
        assert!(!FileTypeUtils::is_previewable("jpg"));
        assert!(!FileTypeUtils::is_previewable("mp4"));
        assert!(!FileTypeUtils::is_previewable("zip"));
        assert!(!FileTypeUtils::is_previewable("exe"));
    }

    #[test]
    fn test_content_indexing() {
        assert!(FileTypeUtils::should_index_content("txt"));
        assert!(FileTypeUtils::should_index_content("rs"));
        assert!(FileTypeUtils::should_index_content("json"));
        
        assert!(!FileTypeUtils::should_index_content("jpg"));
        assert!(!FileTypeUtils::should_index_content("mp4"));
        assert!(!FileTypeUtils::should_index_content("zip"));
    }
}
