use std::path::Path;
use std::fs;
use std::io::Read;

/// Document conversion errors
#[derive(Debug)]
pub enum ConversionError {
    UnsupportedFormat(String),
    FileNotFound(String),
    ConversionFailed(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::UnsupportedFormat(format) => write!(f, "不支持的文件格式: {}", format),
            ConversionError::FileNotFound(path) => write!(f, "文件未找到: {}", path),
            ConversionError::ConversionFailed(msg) => write!(f, "转换失败: {}", msg),
            ConversionError::IoError(err) => write!(f, "IO错误: {}", err),
        }
    }
}

impl std::error::Error for ConversionError {}

impl From<std::io::Error> for ConversionError {
    fn from(err: std::io::Error) -> Self {
        ConversionError::IoError(err)
    }
}

/// Document converter for various formats
pub struct DocumentConverter;

impl DocumentConverter {
    /// Create a new document converter
    pub fn new() -> Self {
        Self
    }

    /// Convert a document to markdown format
    pub fn convert_to_markdown<P: AsRef<Path>>(&self, file_path: P) -> Result<String, ConversionError> {
        let path = file_path.as_ref();
        
        if !path.exists() {
            return Err(ConversionError::FileNotFound(path.display().to_string()));
        }

        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or_else(|| ConversionError::UnsupportedFormat("无法确定文件格式".to_string()))?;

        match extension.as_str() {
            "docx" => self.convert_docx_to_markdown(path),
            "pptx" => self.convert_pptx_to_markdown(path),
            "pdf" => self.convert_pdf_to_markdown(path),
            "txt" => self.convert_txt_to_markdown(path),
            "md" => self.convert_md_to_markdown(path),
            _ => Err(ConversionError::UnsupportedFormat(extension)),
        }
    }

    /// Convert DOCX to markdown
    fn convert_docx_to_markdown(&self, path: &Path) -> Result<String, ConversionError> {
        // DOCX is essentially a ZIP file with XML content
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ConversionError::ConversionFailed(format!("DOCX文件读取失败: {}", e)))?;

        let mut markdown = String::new();

        // Extract title from file name
        if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
            markdown.push_str(&format!("# {}\n\n", file_stem));
        }

        // Try to extract text from document.xml
        match archive.by_name("word/document.xml") {
            Ok(mut document_file) => {
                let mut document_content = String::new();
                document_file.read_to_string(&mut document_content)?;

                if let Some(extracted_text) = self.extract_docx_text_from_xml(&document_content) {
                    markdown.push_str(&extracted_text);
                } else {
                    markdown.push_str("无法提取文档内容");
                }
            },
            Err(_) => {
                return Err(ConversionError::ConversionFailed("无法找到文档内容".to_string()));
            }
        }

        Ok(markdown)
    }

    /// Convert PPTX to markdown
    fn convert_pptx_to_markdown(&self, path: &Path) -> Result<String, ConversionError> {
        // PPTX is essentially a ZIP file with XML content
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ConversionError::ConversionFailed(format!("PPTX文件读取失败: {}", e)))?;

        let mut markdown = String::new();
        markdown.push_str("# PowerPoint 演示文稿\n\n");

        // Extract slides
        let mut slide_count = 1;
        loop {
            let slide_path = format!("ppt/slides/slide{}.xml", slide_count);
            
            match archive.by_name(&slide_path) {
                Ok(mut slide_file) => {
                    let mut slide_content = String::new();
                    slide_file.read_to_string(&mut slide_content)?;
                    
                    if let Some(slide_text) = self.extract_pptx_slide_text(&slide_content) {
                        markdown.push_str(&format!("## 幻灯片 {}\n\n", slide_count));
                        markdown.push_str(&slide_text);
                        markdown.push_str("\n\n---\n\n");
                    }
                    
                    slide_count += 1;
                },
                Err(_) => break, // No more slides
            }
        }

        if slide_count == 1 {
            return Err(ConversionError::ConversionFailed("未找到有效的幻灯片内容".to_string()));
        }

        Ok(markdown)
    }

    /// Convert PDF to markdown
    fn convert_pdf_to_markdown(&self, path: &Path) -> Result<String, ConversionError> {
        use pdf_extract::*;

        match extract_text(path) {
            Ok(text) => {
                let mut markdown = String::new();
                markdown.push_str("# PDF 文档\n\n");

                // Clean up the extracted text and convert to markdown
                let cleaned_text = self.clean_pdf_text(&text);
                markdown.push_str(&cleaned_text);

                Ok(markdown)
            },
            Err(e) => Err(ConversionError::ConversionFailed(format!("PDF解析失败: {}", e))),
        }
    }

    /// Convert TXT to markdown
    fn convert_txt_to_markdown(&self, path: &Path) -> Result<String, ConversionError> {
        let content = fs::read_to_string(path)?;
        
        let mut markdown = String::new();
        
        // Try to detect if it's already structured text
        let lines: Vec<&str> = content.lines().collect();
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                markdown.push('\n');
                continue;
            }
            
            // Convert simple patterns to markdown
            if trimmed.chars().all(|c| c == '=' || c == '-') && trimmed.len() > 3 {
                markdown.push_str("---\n");
            } else if trimmed.starts_with("第") && (trimmed.contains("章") || trimmed.contains("节")) {
                markdown.push_str(&format!("## {}\n", trimmed));
            } else {
                markdown.push_str(line);
                markdown.push('\n');
            }
        }
        
        Ok(markdown)
    }

    /// Convert MD to markdown (just read the file)
    fn convert_md_to_markdown(&self, path: &Path) -> Result<String, ConversionError> {
        let content = fs::read_to_string(path)?;
        Ok(content)
    }

    /// Extract text from DOCX document XML
    fn extract_docx_text_from_xml(&self, xml_content: &str) -> Option<String> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);

        let mut text_content = String::new();
        let mut in_text_element = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    if e.name().as_ref() == b"w:t" {
                        in_text_element = true;
                    }
                },
                Ok(Event::Text(e)) => {
                    if in_text_element {
                        if let Ok(text) = e.unescape() {
                            text_content.push_str(&text);
                        }
                    }
                },
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"w:t" {
                        in_text_element = false;
                    } else if e.name().as_ref() == b"w:p" {
                        // End of paragraph, add line break
                        text_content.push('\n');
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    log::warn!("DOCX XML解析错误: {}", e);
                    break;
                },
                _ => {},
            }
        }

        let cleaned_text = text_content.trim();
        if cleaned_text.is_empty() {
            None
        } else {
            Some(cleaned_text.to_string())
        }
    }

    /// Extract text from PPTX slide XML
    fn extract_pptx_slide_text(&self, xml_content: &str) -> Option<String> {
        use quick_xml::Reader;
        use quick_xml::events::Event;
        
        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);
        
        let mut text_content = String::new();
        let mut in_text_element = false;
        
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    if e.name().as_ref() == b"a:t" {
                        in_text_element = true;
                    }
                },
                Ok(Event::Text(e)) => {
                    if in_text_element {
                        if let Ok(text) = e.unescape() {
                            text_content.push_str(&text);
                            text_content.push(' ');
                        }
                    }
                },
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"a:t" {
                        in_text_element = false;
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    log::warn!("PPTX XML解析错误: {}", e);
                    break;
                },
                _ => {},
            }
        }
        
        let cleaned_text = text_content.trim();
        if cleaned_text.is_empty() {
            None
        } else {
            Some(cleaned_text.to_string())
        }
    }

    /// Clean up PDF extracted text
    fn clean_pdf_text(&self, text: &str) -> String {
        let mut cleaned = String::new();
        let lines: Vec<&str> = text.lines().collect();
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                cleaned.push('\n');
                continue;
            }
            
            // Remove common PDF artifacts
            if trimmed.chars().count() < 3 || 
               trimmed.chars().all(|c| c.is_numeric() || c.is_whitespace()) {
                continue;
            }
            
            cleaned.push_str(trimmed);
            cleaned.push('\n');
        }
        
        cleaned
    }

    /// Check if a file format is supported
    pub fn is_supported_format<P: AsRef<Path>>(file_path: P) -> bool {
        let path = file_path.as_ref();
        
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            matches!(extension.to_lowercase().as_str(), "docx" | "pptx" | "pdf" | "txt" | "md")
        } else {
            false
        }
    }
}
