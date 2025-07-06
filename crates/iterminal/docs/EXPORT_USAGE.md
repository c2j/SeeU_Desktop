# iTerminal Export Functionality

This document describes how to use the terminal export functionality in the iTerminal crate.

## Overview

The iTerminal export system allows users to export terminal session content in multiple formats:
- **Markdown**: Formatted text with metadata and code blocks
- **Plain Text**: Raw text output with optional line numbers
- **HTML**: Web-formatted output with styling support

## Core Components

### 1. Export Options (`ExportOptions`)

Configure how the export should be performed:

```rust
use iterminal::export::{ExportOptions, ExportFormat};

let options = ExportOptions {
    format: ExportFormat::Markdown,
    include_metadata: true,        // Include session info
    include_empty_lines: false,    // Skip blank lines
    max_lines: Some(1000),         // Limit output lines
    strip_ansi: true,              // Remove color codes
    include_line_numbers: false,   // Add line numbers
};
```

### 2. Export Formats

Three formats are supported:

- `ExportFormat::Markdown` - Best for documentation and notes
- `ExportFormat::PlainText` - Raw text output
- `ExportFormat::Html` - Web-ready format with styling

### 3. Terminal Manager Integration

Export functions are integrated into `EguiTerminalManager`:

```rust
use iterminal::egui_terminal::EguiTerminalManager;

let manager = EguiTerminalManager::new();

// Export active session
match manager.export_active_session(&options) {
    Ok(result) => {
        println!("Exported {} lines", result.metadata.line_count);
        println!("Content: {}", result.content);
    }
    Err(e) => eprintln!("Export failed: {}", e),
}

// Export specific session
let session_id = uuid::Uuid::new_v4(); // Your session ID
match manager.export_session(session_id, &options) {
    Ok(result) => println!("Export successful"),
    Err(e) => eprintln!("Export failed: {}", e),
}

// Get raw text content
match manager.get_active_session_text() {
    Ok(text) => println!("Raw text: {}", text),
    Err(e) => eprintln!("Failed to get text: {}", e),
}
```

## Quick Export Functions

For common use cases, use the quick export functions:

```rust
use iterminal::export_ui::QuickExport;

// Copy to clipboard as plain text
match QuickExport::copy_as_text(&manager) {
    Ok(_) => println!("Copied to clipboard"),
    Err(e) => eprintln!("Copy failed: {}", e),
}

// Export as markdown
match QuickExport::export_as_markdown(&manager) {
    Ok(markdown) => {
        // Save to file or display
        std::fs::write("terminal_output.md", markdown)?;
    }
    Err(e) => eprintln!("Markdown export failed: {}", e),
}
```

## UI Integration

### Export Dialog

Use `ExportDialog` for user-friendly export interface:

```rust
use iterminal::export_ui::ExportDialog;

let mut dialog = ExportDialog::default();

// In your UI update loop
if ui.button("Export Terminal").clicked() {
    dialog.open();
}

// Render the dialog
dialog.show(ctx, &terminal_manager);
```

### Export Results

The export system provides detailed results:

```rust
// ExportResult contains:
// - content: String (the exported text)
// - metadata: ExportMetadata (session info, line count, etc.)

match manager.export_active_session(&options) {
    Ok(result) => {
        println!("Session: {}", result.metadata.session_title);
        println!("Lines: {}", result.metadata.line_count);
        println!("Format: {:?}", result.metadata.format);
        println!("Exported at: {}", result.metadata.exported_at);
        
        // Use the content
        std::fs::write("output.txt", result.content)?;
    }
    Err(e) => eprintln!("Export error: {}", e),
}
```

## Error Handling

The export system provides specific error types:

```rust
use iterminal::export::ExportError;

match manager.export_active_session(&options) {
    Err(ExportError::NoBackend) => {
        eprintln!("No terminal backend available");
    }
    Err(ExportError::IoError(msg)) => {
        eprintln!("IO error: {}", msg);
    }
    Err(ExportError::FormatError(msg)) => {
        eprintln!("Format error: {}", msg);
    }
    Ok(result) => {
        // Handle success
    }
}
```

## Best Practices

1. **Choose appropriate formats**:
   - Use Markdown for documentation and sharing
   - Use Plain Text for simple text processing
   - Use HTML for web display

2. **Configure options wisely**:
   - Strip ANSI codes for clean text output
   - Limit lines for large sessions
   - Include metadata for context

3. **Handle errors gracefully**:
   - Always check for active sessions before exporting
   - Provide user feedback for export operations
   - Handle file I/O errors appropriately

4. **Performance considerations**:
   - Use `max_lines` to limit large exports
   - Consider async operations for large sessions
   - Cache export results when appropriate

## Example: Complete Export Workflow

```rust
use iterminal::export::{ExportOptions, ExportFormat};
use iterminal::egui_terminal::EguiTerminalManager;

fn export_terminal_session() -> Result<(), Box<dyn std::error::Error>> {
    let manager = EguiTerminalManager::new();
    
    // Configure export options
    let options = ExportOptions {
        format: ExportFormat::Markdown,
        include_metadata: true,
        include_empty_lines: false,
        max_lines: Some(1000),
        strip_ansi: true,
        include_line_numbers: false,
    };
    
    // Export the active session
    let result = manager.export_active_session(&options)?;
    
    // Generate filename with timestamp
    let filename = format!(
        "terminal_export_{}.md",
        result.metadata.exported_at.format("%Y%m%d_%H%M%S")
    );
    
    // Save to file
    std::fs::write(&filename, result.content)?;
    
    println!("Exported {} lines to {}", result.metadata.line_count, filename);
    Ok(())
}
```

## Testing

Run the export tests to verify functionality:

```bash
cargo test --package iterminal --test export_tests
```

Run the export demo to see the functionality in action:

```bash
cargo run --package iterminal --example export_demo
```
