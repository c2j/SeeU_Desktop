use std::path::{Path, PathBuf};
use std::fs;
use std::io::Read;
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

/// File indexer
pub struct Indexer {
    base_path: PathBuf,
    max_file_size: u64,
    index: Arc<Mutex<Option<Index>>>,
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

    /// Get index reader
    fn get_index_reader(&self) -> Result<IndexReader, Box<dyn std::error::Error>> {
        let index_lock = self.index.lock().unwrap();
        if let Some(index) = &*index_lock {
            Ok(index.reader_builder().reload_policy(ReloadPolicy::OnCommit).try_into()?)
        } else {
            Err("Index not initialized".into())
        }
    }

    /// Index a directory
    pub fn index_directory(&self, directory: &IndexedDirectory) -> Result<IndexStats, Box<dyn std::error::Error>> {
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

            // Skip hidden files and directories (files starting with .)
            if self.is_hidden_file(entry_path) {
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

        Ok(IndexStats {
            total_files,
            total_size_bytes: total_size,
            last_updated: Some(Utc::now()),
        })
    }

    /// Read file content
    fn read_file_content(&self, path: &Path, file_type: &str) -> String {
        // Simple text-based file reading
        // In a production app, you would use specialized libraries for different file types
        match file_type {
            "txt" | "md" | "rs" | "js" | "py" | "cpp" | "h" | "c" | "java" | "html" | "css" | "json" | "toml" => {
                fs::read_to_string(path).unwrap_or_default()
            },
            _ => {
                // For binary files, just read a small portion to extract potential text
                let file = match fs::File::open(path) {
                    Ok(file) => file,
                    Err(_) => return String::new(),
                };

                let mut buffer = Vec::new();
                let _ = file.take(8192).read_to_end(&mut buffer); // Read first 8KB

                // Try to extract text from binary
                String::from_utf8_lossy(&buffer).to_string()
            }
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

        // Parse query
        let main_query = if !query.is_empty() {
            query_parser.parse_query(query)?
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

            // Create content preview
            let content_preview = self.create_content_preview(content, final_query.as_ref());

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

        Ok(results)
    }

    /// Create content preview with highlighted matches
    fn create_content_preview(&self, content: &str, _query: &dyn Query) -> String {
        // Simple preview extraction
        // In a production app, you would use more sophisticated highlighting

        // Use the truncate_with_ellipsis function from utils
        // This handles both ASCII and Unicode text correctly
        crate::utils::truncate_with_ellipsis(content, 200)
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

        log::info!("Removed {} documents from directory '{}' in index", deleted_count, directory_path);
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
}