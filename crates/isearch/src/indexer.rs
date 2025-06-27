use std::path::{Path, PathBuf};
use std::fs;

use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc, TimeZone};
use ignore::WalkBuilder;
use tantivy::schema::{Schema, Field, Document, IndexRecordOption};
use tantivy::{Index, IndexWriter, IndexReader, ReloadPolicy, Term};
use tantivy::collector::TopDocs;
use tantivy::query::{QueryParser, Query};
use tantivy::tokenizer::{TextAnalyzer, SimpleTokenizer, LowerCaser, RemoveLongFilter};
use uuid::Uuid;
use crate::{IndexedDirectory, IndexStats, SearchResult};
use crate::schema::create_schema;
use crate::file_types::FileTypeUtils;

/// Represents a match of a search term in content
#[derive(Debug, Clone)]
struct TermMatch {
    term: String,
    start: usize,
    end: usize,
    score: f32,
}

/// Match with its index in the original matches array
#[derive(Debug, Clone)]
struct MatchWithIndex {
    match_info: TermMatch,
    index: usize,
}

/// File indexer
pub struct Indexer {
    base_path: PathBuf,
    max_file_size: u64,
    index: Arc<Mutex<Option<Index>>>,
    reader: Arc<Mutex<Option<IndexReader>>>,
    schema: Schema,
    path_field: Field,
    filename_field: Field,
    content_field: Field,
    file_type_field: Field,
    size_bytes_field: Field,
    modified_field: Field,
}

impl Indexer {
    /// Create a new indexer
    pub fn new() -> Self {
        let base_path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let base_path = base_path.join("seeu_desktop").join("isearch");

        // Create directories if they don't exist
        fs::create_dir_all(&base_path).ok();

        // Create schema
        let schema = create_schema();

        // Get field references
        let path_field = schema.get_field("path").unwrap();
        let filename_field = schema.get_field("filename").unwrap();
        let content_field = schema.get_field("content").unwrap();
        let file_type_field = schema.get_field("file_type").unwrap();
        let size_bytes_field = schema.get_field("size_bytes").unwrap();
        let modified_field = schema.get_field("modified").unwrap();

        Self {
            base_path,
            max_file_size: 10 * 1024 * 1024, // 10 MB
            index: Arc::new(Mutex::new(None)),
            reader: Arc::new(Mutex::new(None)),
            schema,
            path_field,
            filename_field,
            content_field,
            file_type_field,
            size_bytes_field,
            modified_field,
        }
    }

    /// Initialize the index
    pub fn initialize_index(&self) -> Result<(), Box<dyn std::error::Error>> {
        let index_path = self.base_path.join("index");
        fs::create_dir_all(&index_path)?;

        let index = match Index::open_in_dir(&index_path) {
            Ok(index) => index,
            Err(_) => Index::create_in_dir(&index_path, self.schema.clone())?,
        };

        // Register tokenizers
        index.tokenizers().register(
            "default",
            TextAnalyzer::from(SimpleTokenizer)
                .filter(LowerCaser)
                .filter(RemoveLongFilter::limit(40)),
        );

        let mut index_lock = self.index.lock().unwrap();
        *index_lock = Some(index);

        Ok(())
    }

    /// Get index writer
    fn get_index_writer(&self) -> Result<IndexWriter, Box<dyn std::error::Error>> {
        let index_lock = self.index.lock().unwrap();
        if let Some(index) = &*index_lock {
            Ok(index.writer(50_000_000)?) // 50MB buffer
        } else {
            Err("Index not initialized".into())
        }
    }

    /// Get index reader (cached for performance)
    fn get_index_reader(&self) -> Result<IndexReader, Box<dyn std::error::Error>> {
        // Check if we have a cached reader first
        {
            let reader_lock = self.reader.lock().unwrap();
            if let Some(reader) = &*reader_lock {
                // Try to reload the reader to get latest changes
                if reader.reload().is_ok() {
                    return Ok(reader.clone());
                }
            }
        }

        // Create new reader if cache miss or reload failed
        let index_lock = self.index.lock().unwrap();
        if let Some(index) = &*index_lock {
            let new_reader = index.reader_builder().reload_policy(ReloadPolicy::OnCommit).try_into()?;

            // Cache the new reader
            {
                let mut reader_lock = self.reader.lock().unwrap();
                *reader_lock = Some(new_reader.clone());
            }

            Ok(new_reader)
        } else {
            Err("Index not initialized".into())
        }
    }

    /// Clear reader cache (call after index updates)
    fn clear_reader_cache(&self) {
        let mut reader_lock = self.reader.lock().unwrap();
        *reader_lock = None;
    }

    /// Index a directory
    pub fn index_directory(&self, directory: &IndexedDirectory) -> Result<IndexStats, Box<dyn std::error::Error>> {
        self.index_directory_with_options(directory, false)
    }

    /// Index a directory with options
    pub fn index_directory_with_options(&self, directory: &IndexedDirectory, include_hidden: bool) -> Result<IndexStats, Box<dyn std::error::Error>> {
        let path = Path::new(&directory.path);

        if !path.exists() || !path.is_dir() {
            return Err(format!("Directory does not exist: {}", directory.path).into());
        }

        // Initialize index if not already initialized
        if self.index.lock().unwrap().is_none() {
            self.initialize_index()?;
        }

        let mut index_writer = self.get_index_writer()?;

        // Clear existing documents for this directory
        let dir_path_term = Term::from_field_text(self.path_field, &directory.path);
        index_writer.delete_term(dir_path_term);

        let mut total_files = 0;
        let mut total_size = 0;

        // Create a WalkBuilder that respects .gitignore files
        let walker = WalkBuilder::new(path)
            .hidden(false)  // Don't automatically skip hidden files (we'll handle this manually)
            .git_ignore(true)  // Respect .gitignore files
            .git_global(true)  // Respect global git ignore
            .git_exclude(true)  // Respect .git/info/exclude
            .ignore(true)  // Respect .ignore files
            .follow_links(true)  // Follow symbolic links
            .build();

        // Walk the directory using the ignore-aware walker
        for result in walker {
            let entry = match result {
                Ok(entry) => entry,
                Err(err) => {
                    log::warn!("Error walking directory: {}", err);
                    continue;
                }
            };

            let entry_path = entry.path();

            // Skip directories
            if entry_path.is_dir() {
                continue;
            }

            // Skip hidden files and directories (files starting with .) unless explicitly included
            if !include_hidden && self.is_hidden_file(entry_path) {
                continue;
            }

            // Get file metadata
            if let Ok(metadata) = entry.metadata() {
                let file_size = metadata.len();

                // Skip files that are too large
                if file_size > self.max_file_size {
                    continue;
                }

                // Get file information
                let file_path = entry_path.to_string_lossy().to_string();
                let filename = entry_path.file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default();

                let file_type = entry_path.extension()
                    .map(|ext| ext.to_string_lossy().to_lowercase())
                    .unwrap_or_default();

                let modified = metadata.modified()
                    .map(|time| DateTime::<Utc>::from(time))
                    .unwrap_or_else(|_| Utc::now());

                // Convert to timestamp for tantivy
                let modified_timestamp = modified.timestamp() as u64;

                // Read file content
                let content = self.read_file_content(entry_path, &file_type);

                // Debug: Log content being indexed (UTF-8 safe)
                let content_preview = if content.chars().count() > 100 {
                    let truncated: String = content.chars().take(100).collect();
                    format!("{}...", truncated)
                } else {
                    content.clone()
                };
                log::info!("Indexing file '{}' (type: {}) with {} characters of content: '{}'",
                           filename, file_type, content.chars().count(), content_preview);

                // Create document
                let mut doc = Document::default();
                doc.add_text(self.path_field, &file_path);
                doc.add_text(self.filename_field, &filename);
                doc.add_text(self.content_field, &content);
                doc.add_text(self.file_type_field, &file_type);
                doc.add_u64(self.size_bytes_field, file_size);
                doc.add_u64(self.modified_field, modified_timestamp);

                // Add document to index
                index_writer.add_document(doc)?;

                total_files += 1;
                total_size += file_size;
            }
        }

        // Commit changes
        index_writer.commit()?;

        // Clear reader cache after index update
        self.clear_reader_cache();

        Ok(IndexStats {
            total_files,
            total_size_bytes: total_size,
            last_updated: Some(Utc::now()),
        })
    }

    /// Check if a file type is previewable
    fn is_previewable_file_type(&self, file_type: &str) -> bool {
        FileTypeUtils::is_previewable(file_type)
    }

    /// Check if a file type should be indexed for content search
    fn should_index_content(&self, file_type: &str) -> bool {
        FileTypeUtils::should_index_content(file_type)
    }

    /// Get file type category for display
    fn get_file_type_category(&self, file_type: &str) -> String {
        FileTypeUtils::get_display_name(file_type)
    }

    /// Read file content with optimized strategy
    fn read_file_content(&self, path: &Path, file_type: &str) -> String {
        // Only read content for previewable files to save space and improve performance
        if !self.should_index_content(file_type) {
            // For non-previewable files, return a simple description
            return FileTypeUtils::get_content_placeholder(file_type);
        }

        // Read content for previewable files
        let content = match file_type {
            // Text-based files that can be read directly
            "txt" | "md" | "rs" | "js" | "py" | "cpp" | "h" | "c" | "java" |
            "html" | "css" | "json" | "toml" | "xml" | "yml" | "yaml" |
            "ini" | "cfg" | "log" | "sh" | "bat" | "ps1" | "sql" | "csv" => {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        // Limit content size to prevent index bloat
                        const MAX_CONTENT_SIZE: usize = 50000; // 50KB limit
                        if content.chars().count() > MAX_CONTENT_SIZE {
                            let truncated: String = content.chars().take(MAX_CONTENT_SIZE).collect();
                            log::debug!("Content truncated for file {:?}: {} chars -> {} chars", path, content.chars().count(), MAX_CONTENT_SIZE);
                            format!("{}...[文件内容已截断，原长度: {}字符]", truncated, content.chars().count())
                        } else {
                            content
                        }
                    },
                    Err(e) => {
                        log::debug!("Failed to read text file {:?}: {}", path, e);
                        String::new()
                    }
                }
            },
            // SVG files can be read as text
            "svg" => {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        // For SVG, extract meaningful text content
                        self.extract_svg_text_content(&content)
                    },
                    Err(e) => {
                        log::debug!("Failed to read SVG file {:?}: {}", path, e);
                        "SVG图像文件".to_string()
                    }
                }
            },
            // PDF files - placeholder for future PDF text extraction
            "pdf" => {
                // TODO: Implement PDF text extraction using a library like pdf-extract
                "PDF文档 - 需要PDF文本提取库支持".to_string()
            },
            // This shouldn't happen due to the should_index_content check above
            _ => {
                log::warn!("Unexpected file type for content reading: {}", file_type);
                format!("未知文件类型 ({})", file_type.to_uppercase())
            }
        };

        // Debug: Log content reading results
        if content.is_empty() {
            log::debug!("Read empty content from file: {:?} (type: {})", path, file_type);
        } else {
            log::debug!("Read {} characters from file: {:?} (type: {})", content.chars().count(), path, file_type);
        }

        content
    }

    /// Extract text content from SVG files
    fn extract_svg_text_content(&self, svg_content: &str) -> String {
        let mut text_content = String::new();
        let mut in_text_tag = false;
        let mut current_text = String::new();

        // Simple SVG text extraction - look for <text> and <title> tags
        let lines = svg_content.lines();
        for line in lines {
            let line_lower = line.to_lowercase();

            if line_lower.contains("<text") {
                in_text_tag = true;
                current_text.clear();
            } else if line_lower.contains("</text>") {
                in_text_tag = false;
                if !current_text.trim().is_empty() {
                    if !text_content.is_empty() {
                        text_content.push(' ');
                    }
                    text_content.push_str(current_text.trim());
                }
                current_text.clear();
            } else if in_text_tag {
                // Extract text between tags
                if let Some(start) = line.find('>') {
                    if let Some(end) = line.rfind('<') {
                        if start < end {
                            current_text.push_str(&line[start + 1..end]);
                        }
                    } else {
                        current_text.push_str(&line[start + 1..]);
                    }
                }
            }

            // Also extract title and desc tags
            if line_lower.contains("<title>") || line_lower.contains("<desc>") {
                if let Some(start) = line.find('>') {
                    if let Some(end) = line.rfind('<') {
                        if start < end {
                            let title_text = &line[start + 1..end];
                            if !title_text.trim().is_empty() {
                                if !text_content.is_empty() {
                                    text_content.push(' ');
                                }
                                text_content.push_str(title_text.trim());
                            }
                        }
                    }
                }
            }
        }

        if text_content.trim().is_empty() {
            "SVG图像文件".to_string()
        } else {
            format!("SVG图像文件: {}", text_content.trim())
        }
    }

    /// Search the index with basic query
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        self.search_advanced(query, None, None)
    }

    /// Search the index with advanced options
    pub fn search_advanced(
        &self,
        query: &str,
        file_type_filter: Option<&str>,
        filename_filter: Option<&str>
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        if self.index.lock().unwrap().is_none() {
            return Ok(Vec::new());
        }

        let reader = self.get_index_reader()?;
        let searcher = reader.searcher();

        // Create query parser
        let index_lock = self.index.lock().unwrap();
        let index = index_lock.as_ref().unwrap();

        let query_parser = QueryParser::for_index(
            index,
            vec![self.path_field, self.filename_field, self.content_field],
        );

        // Check if this is a potential filename pattern search
        let is_filename_pattern = self.is_potential_filename_pattern(query);

        // Parse query
        let main_query = if !query.is_empty() {
            if is_filename_pattern {
                // For filename patterns, create a combined query that searches both filename and content
                self.create_enhanced_filename_query(&query_parser, query)?
            } else {
                query_parser.parse_query(query)?
            }
        } else {
            // If query is empty but we have filters, use a match all query
            use tantivy::query::AllQuery;
            Box::new(AllQuery) as Box<dyn Query>
        };

        // Create combined query with filters if needed
        let final_query: Box<dyn Query> = if file_type_filter.is_some() || filename_filter.is_some() {
            use tantivy::query::{BooleanQuery, Occur, TermQuery};

            let mut clauses = vec![(Occur::Must, main_query)];

            // Add file type filter if specified
            if let Some(file_type) = file_type_filter {
                let term = Term::from_field_text(self.file_type_field, file_type);
                let file_type_query = Box::new(TermQuery::new(term, IndexRecordOption::Basic));
                clauses.push((Occur::Must, file_type_query));
            }

            // Add filename filter if specified
            if let Some(filename) = filename_filter {
                // For filename, we'll use the query parser to allow for wildcards
                let filename_query_str = format!("filename:{}", filename);
                let filename_query = query_parser.parse_query(&filename_query_str)?;
                clauses.push((Occur::Must, filename_query));
            }

            Box::new(BooleanQuery::new(clauses))
        } else {
            main_query
        };

        // Search with limit of 101 to detect if there are more results
        let top_docs = searcher.search(&final_query, &TopDocs::with_limit(101))?;

        // Convert results
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            // Limit to 100 results
            if results.len() >= 100 {
                break;
            }

            let retrieved_doc = searcher.doc(doc_address)?;

            // Extract fields
            let path = retrieved_doc.get_first(self.path_field)
                .and_then(|f| f.as_text())
                .unwrap_or_default()
                .to_string();

            let filename = retrieved_doc.get_first(self.filename_field)
                .and_then(|f| f.as_text())
                .unwrap_or_default()
                .to_string();

            let content = retrieved_doc.get_first(self.content_field)
                .and_then(|f| f.as_text())
                .unwrap_or_default();

            let file_type = retrieved_doc.get_first(self.file_type_field)
                .and_then(|f| f.as_text())
                .unwrap_or_default()
                .to_string();

            // Debug: Log content extraction from index (UTF-8 safe)
            let content_preview = if content.chars().count() > 100 {
                let truncated: String = content.chars().take(100).collect();
                format!("{}...", truncated)
            } else {
                content.to_string()
            };
            log::info!("Extracted content from index for file '{}' (type: {}): {} characters: '{}'",
                       filename, file_type, content.chars().count(), content_preview);

            let size_bytes = retrieved_doc.get_first(self.size_bytes_field)
                .and_then(|f| f.as_u64())
                .unwrap_or_default();

            let modified_timestamp = retrieved_doc.get_first(self.modified_field)
                .and_then(|f| f.as_u64())
                .unwrap_or_default();

            // Convert timestamp back to DateTime<Utc>
            let modified = Utc.timestamp_opt(modified_timestamp as i64, 0)
                .single()
                .unwrap_or_else(|| Utc::now());

            // Create content preview with query context - only for previewable files
            let content_preview = if self.is_previewable_file_type(&file_type) {
                self.create_content_preview_with_query(content, query)
            } else {
                // For non-previewable files, show file type description
                FileTypeUtils::get_non_previewable_message(&file_type)
            };

            // Create search result
            let result = SearchResult {
                id: Uuid::new_v4().to_string(),
                filename,
                path,
                file_type,
                size_bytes,
                modified,
                content_preview,
                score,  // Include the score from tantivy
            };

            results.push(result);
        }

        // If this was a filename pattern search, sort results to prioritize filename matches
        if self.is_potential_filename_pattern(query) {
            results.sort_by(|a, b| {
                let a_filename_match = self.filename_matches_pattern(&a.filename, query);
                let b_filename_match = self.filename_matches_pattern(&b.filename, query);

                match (a_filename_match, b_filename_match) {
                    (true, false) => std::cmp::Ordering::Less,    // a has filename match, prioritize
                    (false, true) => std::cmp::Ordering::Greater, // b has filename match, prioritize
                    _ => b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal), // same priority, sort by score
                }
            });
        }

        Ok(results)
    }

    /// Check if the query is potentially a filename pattern
    fn is_potential_filename_pattern(&self, query: &str) -> bool {
        let query = query.trim();

        // Skip if it's an advanced query with operators
        if query.contains(':') || query.contains('"') || query.contains('+') {
            return false;
        }

        // Check if it's a simple term that could be part of a filename
        // Filename patterns are typically:
        // - Single words without spaces
        // - May contain numbers, letters, dots, dashes, underscores
        // - Relatively short (less than 50 characters)
        if query.len() > 50 || query.contains(' ') {
            return false;
        }

        // Check if it contains typical filename characters
        let filename_chars = query.chars().all(|c| {
            c.is_alphanumeric() || c == '.' || c == '-' || c == '_'
        });

        filename_chars
    }

    /// Create an enhanced query that prioritizes filename matches
    fn create_enhanced_filename_query(
        &self,
        query_parser: &QueryParser,
        query: &str
    ) -> Result<Box<dyn Query>, Box<dyn std::error::Error>> {
        use tantivy::query::{BooleanQuery, Occur, RegexQuery};

        // Create a regex pattern for filename matching
        // Convert the query to a case-insensitive regex that matches anywhere in the filename
        let regex_pattern = format!("(?i).*{}.*", regex::escape(query));

        // Create regex query for filename field
        let filename_regex_query = RegexQuery::from_pattern(&regex_pattern, self.filename_field)?;

        // Create regular text query for content
        let content_query = query_parser.parse_query(query)?;

        // Combine with OR logic, but filename matches will get higher scores due to sorting
        let clauses = vec![
            (Occur::Should, Box::new(filename_regex_query) as Box<dyn Query>),
            (Occur::Should, content_query),
        ];

        Ok(Box::new(BooleanQuery::new(clauses)))
    }

    /// Check if a filename matches the pattern
    fn filename_matches_pattern(&self, filename: &str, pattern: &str) -> bool {
        filename.to_lowercase().contains(&pattern.to_lowercase())
    }

    /// Create content preview with highlighted matches
    fn create_content_preview(&self, content: &str, query: &dyn Query) -> String {
        if content.is_empty() {
            log::debug!("Content is empty for preview generation, returning empty preview");
            return String::new();
        }

        // Try to extract relevant snippets based on the query
        let preview = self.extract_relevant_snippet(content, query);

        log::debug!("Created preview from content with {} characters -> preview: '{}'",
                   content.chars().count(),
                   if preview.len() > 100 { format!("{}...", &preview[..100]) } else { preview.clone() });

        preview
    }

    /// Extract relevant snippet from content based on query
    fn extract_relevant_snippet(&self, content: &str, _query: &dyn Query) -> String {
        // For now, we'll implement a simple approach
        // In a production app, you would use more sophisticated snippet extraction

        // Clean up the content first (remove excessive whitespace, HTML tags for HTML files)
        let cleaned_content = self.clean_content_for_preview(content);

        // For now, just return the first meaningful paragraph or sentence
        // TODO: In the future, we can implement proper query-based snippet extraction
        self.extract_first_meaningful_content(&cleaned_content, 200)
    }

    /// Clean content for preview (remove HTML tags, excessive whitespace, etc.)
    fn clean_content_for_preview(&self, content: &str) -> String {
        let mut cleaned = content.to_string();

        // Remove HTML tags (simple regex-like approach)
        cleaned = self.remove_html_tags(&cleaned);

        // Normalize whitespace
        cleaned = self.normalize_whitespace(&cleaned);

        cleaned
    }

    /// Remove HTML tags from content
    fn remove_html_tags(&self, content: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        let mut in_script_or_style = false;
        let mut tag_name = String::new();

        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch == '<' && i + 1 < chars.len() {
                in_tag = true;
                tag_name.clear();

                // Check if it's a script or style tag
                let remaining: String = chars[i..].iter().take(20).collect();
                if remaining.to_lowercase().starts_with("<script") ||
                   remaining.to_lowercase().starts_with("<style") {
                    in_script_or_style = true;
                }
            } else if ch == '>' && in_tag {
                in_tag = false;

                // Check if it's a closing script or style tag
                if tag_name.to_lowercase() == "/script" || tag_name.to_lowercase() == "/style" {
                    in_script_or_style = false;
                }

                tag_name.clear();
            } else if in_tag {
                tag_name.push(ch);
            } else if !in_tag && !in_script_or_style {
                result.push(ch);
            }

            i += 1;
        }

        result
    }

    /// Normalize whitespace (collapse multiple spaces, newlines, etc.)
    fn normalize_whitespace(&self, content: &str) -> String {
        let mut result = String::new();
        let mut prev_was_space = false;

        for ch in content.chars() {
            if ch.is_whitespace() {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(ch);
                prev_was_space = false;
            }
        }

        result.trim().to_string()
    }

    /// Extract first meaningful content (skip common headers, navigation, etc.)
    fn extract_first_meaningful_content(&self, content: &str, max_length: usize) -> String {
        let lines: Vec<&str> = content.split('\n').collect();
        let mut meaningful_content = String::new();

        for line in lines {
            let trimmed = line.trim();

            // Skip very short lines, common navigation text, etc.
            if trimmed.len() < 10 {
                continue;
            }

            // Skip common navigation/header patterns
            if trimmed.to_lowercase().contains("navigation") ||
               trimmed.to_lowercase().contains("menu") ||
               trimmed.to_lowercase().contains("header") ||
               trimmed.to_lowercase().starts_with("home") ||
               trimmed.to_lowercase().starts_with("about") {
                continue;
            }

            // Add this line to meaningful content
            if !meaningful_content.is_empty() {
                meaningful_content.push(' ');
            }
            meaningful_content.push_str(trimmed);

            // Stop if we have enough content
            if meaningful_content.len() >= max_length {
                break;
            }
        }

        // If we didn't find meaningful content, just use the beginning
        if meaningful_content.trim().is_empty() {
            meaningful_content = content.chars().take(max_length).collect();
        }

        // Truncate to max length
        crate::utils::truncate_with_ellipsis(&meaningful_content, max_length)
    }

    /// Create content preview with query-based snippet extraction
    fn create_content_preview_with_query(&self, content: &str, query: &str) -> String {
        if content.is_empty() {
            return String::new();
        }

        // Clean the content first
        let cleaned_content = self.clean_content_for_preview(content);

        // Extract query terms (simple approach)
        let query_terms = self.extract_query_terms(query);

        if query_terms.is_empty() {
            // No specific terms, use general content extraction
            return self.extract_first_meaningful_content(&cleaned_content, 200);
        }

        // Try to find snippets containing query terms
        if let Some(snippet) = self.find_snippet_with_terms(&cleaned_content, &query_terms, 200) {
            snippet
        } else {
            // Fallback to general content extraction
            self.extract_first_meaningful_content(&cleaned_content, 200)
        }
    }

    /// Extract query terms from search query
    fn extract_query_terms(&self, query: &str) -> Vec<String> {
        crate::utils::extract_search_terms(query)
    }

    /// Find snippets containing query terms with intelligent context extraction
    fn find_snippet_with_terms(&self, content: &str, terms: &[String], max_length: usize) -> Option<String> {
        if terms.is_empty() {
            return None;
        }

        // Find all matches with their positions and context
        let matches = self.find_all_term_matches(content, terms);

        if matches.is_empty() {
            return None;
        }

        // Group nearby matches and create snippets
        let snippets = self.create_context_snippets(content, &matches, max_length);

        if snippets.is_empty() {
            return None;
        }

        // Combine snippets with ellipsis if multiple
        Some(self.combine_snippets(&snippets, max_length))
    }

    /// Find all positions where query terms appear in the content
    fn find_all_term_matches(&self, content: &str, terms: &[String]) -> Vec<TermMatch> {
        let content_lower = content.to_lowercase();
        let content_chars: Vec<char> = content.chars().collect();
        let content_lower_chars: Vec<char> = content_lower.chars().collect();
        let mut matches = Vec::new();

        for term in terms {
            let term_chars: Vec<char> = term.chars().collect();
            if term_chars.is_empty() {
                continue;
            }

            let mut char_start = 0;
            while char_start + term_chars.len() <= content_lower_chars.len() {
                // Check if term matches at current character position
                let mut matches_here = true;
                for (i, &term_char) in term_chars.iter().enumerate() {
                    if content_lower_chars[char_start + i] != term_char {
                        matches_here = false;
                        break;
                    }
                }

                if matches_here {
                    // Convert character positions back to byte positions for compatibility
                    let byte_start = content_chars[..char_start].iter().collect::<String>().len();
                    let byte_end = content_chars[..char_start + term_chars.len()].iter().collect::<String>().len();

                    matches.push(TermMatch {
                        term: term.clone(),
                        start: byte_start,
                        end: byte_end,
                        score: self.calculate_term_score(term, content, byte_start),
                    });
                    char_start += term_chars.len();
                } else {
                    char_start += 1;
                }
            }
        }

        // Sort by position
        matches.sort_by_key(|m| m.start);
        matches
    }

    /// Calculate relevance score for a term match based on context
    fn calculate_term_score(&self, term: &str, content: &str, byte_position: usize) -> f32 {
        let mut score = 1.0;

        // Bonus for longer terms (use character count, not byte count)
        score += (term.chars().count() as f32) * 0.1;

        // Safe character position calculation
        let char_pos = if byte_position == 0 {
            0
        } else if byte_position >= content.len() {
            content.chars().count()
        } else {
            // Find the character position safely
            let mut char_count = 0;
            let mut byte_count = 0;
            for ch in content.chars() {
                if byte_count >= byte_position {
                    break;
                }
                byte_count += ch.len_utf8();
                char_count += 1;
            }
            char_count
        };

        // Bonus for word boundaries
        let chars: Vec<char> = content.chars().collect();

        // Check if it's at word boundary (start)
        if char_pos == 0 || chars.get(char_pos.saturating_sub(1)).map_or(true, |c| !c.is_alphanumeric()) {
            score += 0.5;
        }

        // Check if it's at word boundary (end)
        let term_char_len = term.chars().count();
        if char_pos + term_char_len >= chars.len() ||
           chars.get(char_pos + term_char_len).map_or(true, |c| !c.is_alphanumeric()) {
            score += 0.5;
        }

        // Bonus for being near the beginning of the document (use character-based calculation)
        let total_chars = content.chars().count();
        if total_chars > 0 {
            let relative_pos = char_pos as f32 / total_chars as f32;
            if relative_pos < 0.1 {
                score += 0.3;
            }
        }

        score
    }

    /// Create context snippets around matches
    fn create_context_snippets(&self, content: &str, matches: &[TermMatch], max_total_length: usize) -> Vec<String> {
        let mut snippets = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let context_size = 40; // Characters of context on each side

        let mut i = 0;
        while i < matches.len() {
            let match_group = self.group_nearby_matches(matches, i, context_size * 2);
            let snippet = self.extract_snippet_for_group(content, &chars, &match_group, context_size);

            if !snippet.trim().is_empty() {
                snippets.push(snippet);
            }

            i = match_group.last().map(|m| m.index + 1).unwrap_or(i + 1);

            // Stop if we have enough content
            let current_length: usize = snippets.iter().map(|s| s.len()).sum();
            if current_length >= max_total_length * 2 / 3 {
                break;
            }
        }

        snippets
    }

    /// Group nearby matches together
    fn group_nearby_matches(&self, matches: &[TermMatch], start_idx: usize, max_distance: usize) -> Vec<MatchWithIndex> {
        let mut group = vec![MatchWithIndex { match_info: matches[start_idx].clone(), index: start_idx }];

        for i in (start_idx + 1)..matches.len() {
            let current_match = &matches[i];
            let last_match = &group.last().unwrap().match_info;

            if current_match.start <= last_match.end + max_distance {
                group.push(MatchWithIndex { match_info: current_match.clone(), index: i });
            } else {
                break;
            }
        }

        group
    }

    /// Extract snippet for a group of matches with context
    fn extract_snippet_for_group(&self, content: &str, chars: &[char], match_group: &[MatchWithIndex], context_size: usize) -> String {
        if match_group.is_empty() {
            return String::new();
        }

        let first_match = &match_group[0].match_info;
        let last_match = &match_group.last().unwrap().match_info;

        // Convert byte positions to character positions safely
        let first_char_pos = self.byte_to_char_position(content, first_match.start);
        let last_char_pos = self.byte_to_char_position(content, last_match.end);

        // Calculate snippet boundaries with context
        let snippet_start = first_char_pos.saturating_sub(context_size);
        let snippet_end = (last_char_pos + context_size).min(chars.len());

        // Find good break points (word boundaries)
        let actual_start = self.find_word_boundary(chars, snippet_start, true);
        let actual_end = self.find_word_boundary(chars, snippet_end, false);

        // Extract the snippet
        let snippet: String = chars[actual_start..actual_end].iter().collect();

        // Add ellipsis if we're not at the document boundaries
        let mut result = String::new();
        if actual_start > 0 {
            result.push_str("...");
        }
        result.push_str(snippet.trim());
        if actual_end < chars.len() {
            result.push_str("...");
        }

        result
    }

    /// Safely convert byte position to character position
    fn byte_to_char_position(&self, content: &str, byte_pos: usize) -> usize {
        if byte_pos == 0 {
            return 0;
        }
        if byte_pos >= content.len() {
            return content.chars().count();
        }

        let mut char_count = 0;
        let mut byte_count = 0;

        for ch in content.chars() {
            if byte_count >= byte_pos {
                break;
            }
            byte_count += ch.len_utf8();
            char_count += 1;
        }

        char_count
    }

    /// Find word boundary near the given position
    fn find_word_boundary(&self, chars: &[char], pos: usize, find_start: bool) -> usize {
        let search_range = 10; // Look within 10 characters for a good boundary

        if find_start {
            // Look backwards for word start
            for i in 0..search_range {
                let check_pos = pos.saturating_sub(i);
                if check_pos == 0 || chars.get(check_pos.saturating_sub(1)).map_or(true, |c| c.is_whitespace()) {
                    return check_pos;
                }
            }
        } else {
            // Look forwards for word end
            for i in 0..search_range {
                let check_pos = pos + i;
                if check_pos >= chars.len() || chars.get(check_pos).map_or(true, |c| c.is_whitespace()) {
                    return check_pos;
                }
            }
        }

        pos
    }

    /// Combine multiple snippets with appropriate separators
    fn combine_snippets(&self, snippets: &[String], max_length: usize) -> String {
        if snippets.is_empty() {
            return String::new();
        }

        if snippets.len() == 1 {
            return crate::utils::truncate_with_ellipsis(&snippets[0], max_length);
        }

        // Combine snippets with separator
        let separator = " ... ";
        let mut result = String::new();
        let mut current_length = 0;

        for (i, snippet) in snippets.iter().enumerate() {
            if i > 0 {
                if current_length + separator.len() + snippet.len() > max_length {
                    break;
                }
                result.push_str(separator);
                current_length += separator.len();
            }

            let available_space = max_length.saturating_sub(current_length);
            if snippet.len() <= available_space {
                result.push_str(snippet);
                current_length += snippet.len();
            } else {
                result.push_str(&crate::utils::truncate_with_ellipsis(snippet, available_space));
                break;
            }
        }

        result
    }

    /// Remove all documents from a directory from the index
    pub fn remove_directory_from_index(&self, directory_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize index if not already initialized
        if self.index.lock().unwrap().is_none() {
            self.initialize_index()?;
        }

        let mut index_writer = self.get_index_writer()?;

        // Get all documents in the index and filter by directory path
        let reader = self.get_index_reader()?;
        let searcher = reader.searcher();

        // Use a match-all query to get all documents, then filter by path
        use tantivy::query::AllQuery;
        let all_query = AllQuery;
        let top_docs = searcher.search(&all_query, &TopDocs::with_limit(100000))?;

        let mut deleted_count = 0;

        // Check each document to see if it belongs to the directory being removed
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            if let Some(path_value) = retrieved_doc.get_first(self.path_field) {
                if let Some(path_text) = path_value.as_text() {
                    // Check if this path is within the directory being removed
                    // Normalize paths to handle different path separators
                    let normalized_dir = directory_path.replace('\\', "/");
                    let normalized_path = path_text.replace('\\', "/");

                    if normalized_path.starts_with(&normalized_dir) {
                        // Make sure it's actually within the directory (not just a prefix match)
                        let remaining = &normalized_path[normalized_dir.len()..];
                        if remaining.is_empty() || remaining.starts_with('/') {
                            let term = Term::from_field_text(self.path_field, path_text);
                            index_writer.delete_term(term);
                            deleted_count += 1;
                        }
                    }
                }
            }
        }

        // Commit the deletions
        index_writer.commit()?;

        // Clear reader cache after index update
        self.clear_reader_cache();

        log::info!("Successfully removed {} documents from directory '{}' in index", deleted_count, directory_path);

        if deleted_count == 0 {
            log::warn!("No documents found to delete for directory: {}", directory_path);
        }
        Ok(())
    }

    /// Check if a file or any of its parent directories is hidden (starts with .)
    fn is_hidden_file(&self, path: &Path) -> bool {
        // Check if the file itself starts with .
        if let Some(filename) = path.file_name() {
            if filename.to_string_lossy().starts_with('.') {
                return true;
            }
        }

        // Check if any parent directory starts with .
        for component in path.components() {
            if let Some(name) = component.as_os_str().to_str() {
                if name.starts_with('.') && name != "." && name != ".." {
                    return true;
                }
            }
        }

        false
    }

    /// Clear all documents from the index
    pub fn clear_all_documents(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Clearing all documents from index");

        // Initialize index if not already initialized
        if self.index.lock().unwrap().is_none() {
            self.initialize_index()?;
        }

        let mut index_writer = self.get_index_writer()?;

        // Delete all documents
        index_writer.delete_all_documents()?;

        // Commit the deletions
        index_writer.commit()?;

        // Clear reader cache after index update
        self.clear_reader_cache();

        log::info!("Successfully cleared all documents from index");
        Ok(())
    }

    /// Completely rebuild the index (delete index directory and recreate)
    pub fn rebuild_index(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Completely rebuilding index (deleting index directory)");

        // Clear any existing index in memory
        {
            let mut index_lock = self.index.lock().unwrap();
            *index_lock = None;
        }

        // Clear reader cache
        self.clear_reader_cache();

        // Delete the entire index directory
        let index_path = self.base_path.join("index");
        if index_path.exists() {
            fs::remove_dir_all(&index_path)?;
            log::info!("Deleted existing index directory: {:?}", index_path);
        }

        // Recreate the index with the current schema
        self.initialize_index()?;

        log::info!("Successfully rebuilt index with new schema");
        Ok(())
    }
}