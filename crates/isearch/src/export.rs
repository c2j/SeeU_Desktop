use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;
use chrono::{DateTime, Utc};
use serde_json;
use crate::{SearchResult, SearchStats};

/// Export format options
#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Csv,
    Json,
    Text,
    Html,
    Markdown,
}

impl ExportFormat {
    /// Get file extension for the format
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Json => "json",
            ExportFormat::Text => "txt",
            ExportFormat::Html => "html",
            ExportFormat::Markdown => "md",
        }
    }

    /// Get MIME type for the format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "text/csv",
            ExportFormat::Json => "application/json",
            ExportFormat::Text => "text/plain",
            ExportFormat::Html => "text/html",
            ExportFormat::Markdown => "text/markdown",
        }
    }

    /// Get human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "CSV (逗号分隔值)",
            ExportFormat::Json => "JSON (结构化数据)",
            ExportFormat::Text => "文本文件",
            ExportFormat::Html => "HTML (网页)",
            ExportFormat::Markdown => "Markdown (标记文档)",
        }
    }
}

/// Export configuration
#[derive(Debug, Clone)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub include_stats: bool,
    pub include_content_preview: bool,
    pub max_preview_length: usize,
    pub sort_by_relevance: bool,
    pub include_metadata: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::Csv,
            include_stats: true,
            include_content_preview: true,
            max_preview_length: 200,
            sort_by_relevance: true,
            include_metadata: true,
        }
    }
}

/// Export metadata
#[derive(Debug, Clone)]
pub struct ExportMetadata {
    pub query: String,
    pub export_time: DateTime<Utc>,
    pub total_results: usize,
    pub format: ExportFormat,
    pub stats: Option<SearchStats>,
}

/// Search result exporter
pub struct SearchResultExporter;

impl SearchResultExporter {
    /// Export search results to a file
    pub fn export_to_file<P: AsRef<Path>>(
        path: P,
        results: &[SearchResult],
        config: &ExportConfig,
        metadata: &ExportMetadata,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        match config.format {
            ExportFormat::Csv => Self::export_csv(&mut writer, results, config, metadata)?,
            ExportFormat::Json => Self::export_json(&mut writer, results, config, metadata)?,
            ExportFormat::Text => Self::export_text(&mut writer, results, config, metadata)?,
            ExportFormat::Html => Self::export_html(&mut writer, results, config, metadata)?,
            ExportFormat::Markdown => Self::export_markdown(&mut writer, results, config, metadata)?,
        }

        writer.flush()?;
        Ok(())
    }

    /// Export search results to string
    pub fn export_to_string(
        results: &[SearchResult],
        config: &ExportConfig,
        metadata: &ExportMetadata,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();

        match config.format {
            ExportFormat::Csv => Self::export_csv(&mut buffer, results, config, metadata)?,
            ExportFormat::Json => Self::export_json(&mut buffer, results, config, metadata)?,
            ExportFormat::Text => Self::export_text(&mut buffer, results, config, metadata)?,
            ExportFormat::Html => Self::export_html(&mut buffer, results, config, metadata)?,
            ExportFormat::Markdown => Self::export_markdown(&mut buffer, results, config, metadata)?,
        }

        Ok(String::from_utf8(buffer)?)
    }

    /// Export as CSV format
    fn export_csv<W: Write>(
        writer: &mut W,
        results: &[SearchResult],
        config: &ExportConfig,
        metadata: &ExportMetadata,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Write metadata as comments
        if config.include_metadata {
            writeln!(writer, "# 搜索查询: {}", metadata.query)?;
            writeln!(writer, "# 导出时间: {}", metadata.export_time.format("%Y-%m-%d %H:%M:%S UTC"))?;
            writeln!(writer, "# 结果数量: {}", metadata.total_results)?;
            if let Some(stats) = &metadata.stats {
                writeln!(writer, "# 搜索时间: {}ms", stats.search_time_ms)?;
            }
            writeln!(writer)?;
        }

        // Write CSV header
        if config.include_content_preview {
            writeln!(writer, "文件名,路径,文件类型,大小(字节),修改时间,相关性评分,内容预览")?;
        } else {
            writeln!(writer, "文件名,路径,文件类型,大小(字节),修改时间,相关性评分")?;
        }

        // Write data rows
        for result in results {
            let filename = Self::escape_csv_field(&result.filename);
            let path = Self::escape_csv_field(&result.path);
            let file_type = Self::escape_csv_field(&result.file_type);
            let modified = result.modified.format("%Y-%m-%d %H:%M:%S");

            if config.include_content_preview {
                let preview = Self::escape_csv_field(&Self::truncate_preview(&result.content_preview, config.max_preview_length));
                writeln!(writer, "{},{},{},{},{},{:.3},{}", 
                    filename, path, file_type, result.size_bytes, modified, result.score, preview)?;
            } else {
                writeln!(writer, "{},{},{},{},{},{:.3}", 
                    filename, path, file_type, result.size_bytes, modified, result.score)?;
            }
        }

        Ok(())
    }

    /// Export as JSON format
    fn export_json<W: Write>(
        writer: &mut W,
        results: &[SearchResult],
        config: &ExportConfig,
        metadata: &ExportMetadata,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut export_data = serde_json::Map::new();

        // Add metadata
        if config.include_metadata {
            let mut meta = serde_json::Map::new();
            meta.insert("query".to_string(), serde_json::Value::String(metadata.query.clone()));
            meta.insert("export_time".to_string(), serde_json::Value::String(metadata.export_time.to_rfc3339()));
            meta.insert("total_results".to_string(), serde_json::Value::Number(metadata.total_results.into()));
            meta.insert("format".to_string(), serde_json::Value::String(format!("{:?}", metadata.format)));
            
            if let Some(stats) = &metadata.stats {
                let mut stats_obj = serde_json::Map::new();
                stats_obj.insert("search_time_ms".to_string(), serde_json::Value::Number(stats.search_time_ms.into()));
                stats_obj.insert("total_results".to_string(), serde_json::Value::Number(stats.total_results.into()));
                stats_obj.insert("total_matches".to_string(), serde_json::Value::Number(stats.total_matches.into()));
                meta.insert("stats".to_string(), serde_json::Value::Object(stats_obj));
            }
            
            export_data.insert("metadata".to_string(), serde_json::Value::Object(meta));
        }

        // Add results
        let mut results_array = Vec::new();
        for result in results {
            let mut result_obj = serde_json::Map::new();
            result_obj.insert("filename".to_string(), serde_json::Value::String(result.filename.clone()));
            result_obj.insert("path".to_string(), serde_json::Value::String(result.path.clone()));
            result_obj.insert("file_type".to_string(), serde_json::Value::String(result.file_type.clone()));
            result_obj.insert("size_bytes".to_string(), serde_json::Value::Number(result.size_bytes.into()));
            result_obj.insert("modified".to_string(), serde_json::Value::String(result.modified.to_rfc3339()));
            result_obj.insert("score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(result.score as f64).unwrap_or(serde_json::Number::from(0))));
            
            if config.include_content_preview {
                let preview = Self::truncate_preview(&result.content_preview, config.max_preview_length);
                result_obj.insert("content_preview".to_string(), serde_json::Value::String(preview));
            }
            
            results_array.push(serde_json::Value::Object(result_obj));
        }
        
        export_data.insert("results".to_string(), serde_json::Value::Array(results_array));

        let json_string = serde_json::to_string_pretty(&export_data)?;
        writer.write_all(json_string.as_bytes())?;

        Ok(())
    }

    /// Export as plain text format
    fn export_text<W: Write>(
        writer: &mut W,
        results: &[SearchResult],
        config: &ExportConfig,
        metadata: &ExportMetadata,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Write header
        if config.include_metadata {
            writeln!(writer, "搜索结果导出")?;
            writeln!(writer, "============")?;
            writeln!(writer)?;
            writeln!(writer, "搜索查询: {}", metadata.query)?;
            writeln!(writer, "导出时间: {}", metadata.export_time.format("%Y-%m-%d %H:%M:%S UTC"))?;
            writeln!(writer, "结果数量: {}", metadata.total_results)?;
            
            if let Some(stats) = &metadata.stats {
                writeln!(writer, "搜索时间: {}ms", stats.search_time_ms)?;
                writeln!(writer, "总匹配数: {}", stats.total_matches)?;
            }
            
            writeln!(writer)?;
            writeln!(writer, "搜索结果:")?;
            writeln!(writer, "--------")?;
            writeln!(writer)?;
        }

        // Write results
        for (i, result) in results.iter().enumerate() {
            writeln!(writer, "{}. {}", i + 1, result.filename)?;
            writeln!(writer, "   路径: {}", result.path)?;
            writeln!(writer, "   类型: {}", result.file_type)?;
            writeln!(writer, "   大小: {} 字节", result.size_bytes)?;
            writeln!(writer, "   修改时间: {}", result.modified.format("%Y-%m-%d %H:%M:%S"))?;
            writeln!(writer, "   相关性: {:.3}", result.score)?;
            
            if config.include_content_preview && !result.content_preview.is_empty() {
                let preview = Self::truncate_preview(&result.content_preview, config.max_preview_length);
                writeln!(writer, "   预览: {}", preview)?;
            }
            
            writeln!(writer)?;
        }

        Ok(())
    }

    /// Export as HTML format
    fn export_html<W: Write>(
        writer: &mut W,
        results: &[SearchResult],
        config: &ExportConfig,
        metadata: &ExportMetadata,
    ) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(writer, "<!DOCTYPE html>")?;
        writeln!(writer, "<html lang=\"zh-CN\">")?;
        writeln!(writer, "<head>")?;
        writeln!(writer, "    <meta charset=\"UTF-8\">")?;
        writeln!(writer, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(writer, "    <title>搜索结果 - {}</title>", Self::escape_html(&metadata.query))?;
        writeln!(writer, "    <style>")?;
        writeln!(writer, "        body {{ font-family: Arial, sans-serif; margin: 20px; }}")?;
        writeln!(writer, "        .header {{ border-bottom: 2px solid #ccc; padding-bottom: 10px; margin-bottom: 20px; }}")?;
        writeln!(writer, "        .result {{ border: 1px solid #ddd; margin: 10px 0; padding: 15px; border-radius: 5px; }}")?;
        writeln!(writer, "        .filename {{ font-weight: bold; font-size: 1.1em; color: #2c5aa0; }}")?;
        writeln!(writer, "        .path {{ color: #666; font-size: 0.9em; }}")?;
        writeln!(writer, "        .metadata {{ color: #888; font-size: 0.8em; }}")?;
        writeln!(writer, "        .preview {{ background: #f5f5f5; padding: 10px; margin-top: 10px; border-radius: 3px; }}")?;
        writeln!(writer, "    </style>")?;
        writeln!(writer, "</head>")?;
        writeln!(writer, "<body>")?;

        // Header
        if config.include_metadata {
            writeln!(writer, "    <div class=\"header\">")?;
            writeln!(writer, "        <h1>搜索结果</h1>")?;
            writeln!(writer, "        <p><strong>查询:</strong> {}</p>", Self::escape_html(&metadata.query))?;
            writeln!(writer, "        <p><strong>导出时间:</strong> {}</p>", metadata.export_time.format("%Y-%m-%d %H:%M:%S UTC"))?;
            writeln!(writer, "        <p><strong>结果数量:</strong> {}</p>", metadata.total_results)?;
            
            if let Some(stats) = &metadata.stats {
                writeln!(writer, "        <p><strong>搜索时间:</strong> {}ms</p>", stats.search_time_ms)?;
            }
            
            writeln!(writer, "    </div>")?;
        }

        // Results
        for result in results {
            writeln!(writer, "    <div class=\"result\">")?;
            writeln!(writer, "        <div class=\"filename\">{}</div>", Self::escape_html(&result.filename))?;
            writeln!(writer, "        <div class=\"path\">{}</div>", Self::escape_html(&result.path))?;
            writeln!(writer, "        <div class=\"metadata\">")?;
            writeln!(writer, "            类型: {} | 大小: {} 字节 | 修改时间: {} | 相关性: {:.3}",
                Self::escape_html(&result.file_type),
                result.size_bytes,
                result.modified.format("%Y-%m-%d %H:%M:%S"),
                result.score
            )?;
            writeln!(writer, "        </div>")?;
            
            if config.include_content_preview && !result.content_preview.is_empty() {
                let preview = Self::truncate_preview(&result.content_preview, config.max_preview_length);
                writeln!(writer, "        <div class=\"preview\">{}</div>", Self::escape_html(&preview))?;
            }
            
            writeln!(writer, "    </div>")?;
        }

        writeln!(writer, "</body>")?;
        writeln!(writer, "</html>")?;

        Ok(())
    }

    /// Export as Markdown format
    fn export_markdown<W: Write>(
        writer: &mut W,
        results: &[SearchResult],
        config: &ExportConfig,
        metadata: &ExportMetadata,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Header
        if config.include_metadata {
            writeln!(writer, "# 搜索结果")?;
            writeln!(writer)?;
            writeln!(writer, "**查询:** {}", metadata.query)?;
            writeln!(writer, "**导出时间:** {}", metadata.export_time.format("%Y-%m-%d %H:%M:%S UTC"))?;
            writeln!(writer, "**结果数量:** {}", metadata.total_results)?;
            
            if let Some(stats) = &metadata.stats {
                writeln!(writer, "**搜索时间:** {}ms", stats.search_time_ms)?;
                writeln!(writer, "**总匹配数:** {}", stats.total_matches)?;
            }
            
            writeln!(writer)?;
            writeln!(writer, "---")?;
            writeln!(writer)?;
        }

        // Results
        for (i, result) in results.iter().enumerate() {
            writeln!(writer, "## {}. {}", i + 1, result.filename)?;
            writeln!(writer)?;
            writeln!(writer, "- **路径:** `{}`", result.path)?;
            writeln!(writer, "- **类型:** {}", result.file_type)?;
            writeln!(writer, "- **大小:** {} 字节", result.size_bytes)?;
            writeln!(writer, "- **修改时间:** {}", result.modified.format("%Y-%m-%d %H:%M:%S"))?;
            writeln!(writer, "- **相关性:** {:.3}", result.score)?;
            
            if config.include_content_preview && !result.content_preview.is_empty() {
                let preview = Self::truncate_preview(&result.content_preview, config.max_preview_length);
                writeln!(writer)?;
                writeln!(writer, "**内容预览:**")?;
                writeln!(writer, "```")?;
                writeln!(writer, "{}", preview)?;
                writeln!(writer, "```")?;
            }
            
            writeln!(writer)?;
        }

        Ok(())
    }

    /// Escape CSV field
    fn escape_csv_field(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }

    /// Escape HTML content
    fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    /// Truncate preview text
    fn truncate_preview(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else {
            format!("{}...", &text[..max_length])
        }
    }
}
