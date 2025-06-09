use tantivy::schema::{Schema, SchemaBuilder, TEXT, STORED};

/// Create the search schema
pub fn create_schema() -> Schema {
    let mut schema_builder = SchemaBuilder::default();

    // Add fields to the schema
    schema_builder.add_text_field("path", TEXT | STORED);
    schema_builder.add_text_field("filename", TEXT | STORED);
    schema_builder.add_text_field("content", TEXT);
    schema_builder.add_text_field("file_type", TEXT | STORED);
    schema_builder.add_u64_field("size_bytes", STORED);
    schema_builder.add_u64_field("modified", STORED);

    schema_builder.build()
}