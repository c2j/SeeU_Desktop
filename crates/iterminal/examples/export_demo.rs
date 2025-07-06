use iterminal::export::{ExportOptions, ExportFormat};
use iterminal::egui_terminal::EguiTerminalManager;

fn main() {
    println!("iTerminal Export Demo");
    println!("====================");
    
    // Create a terminal manager
    let manager = EguiTerminalManager::new();
    
    // Create export options for different formats
    let markdown_options = ExportOptions {
        format: ExportFormat::Markdown,
        include_metadata: true,
        include_empty_lines: false,
        max_lines: Some(1000),
        strip_ansi: true,
        include_line_numbers: false,
    };
    
    let plain_text_options = ExportOptions {
        format: ExportFormat::PlainText,
        include_metadata: false,
        include_empty_lines: true,
        max_lines: None,
        strip_ansi: true,
        include_line_numbers: true,
    };
    
    let html_options = ExportOptions {
        format: ExportFormat::Html,
        include_metadata: true,
        include_empty_lines: false,
        max_lines: Some(500),
        strip_ansi: false, // Keep ANSI for HTML conversion
        include_line_numbers: true,
    };
    
    println!("\n1. Markdown Export Options:");
    println!("   Format: {:?}", markdown_options.format);
    println!("   Include metadata: {}", markdown_options.include_metadata);
    println!("   Strip ANSI: {}", markdown_options.strip_ansi);
    println!("   Max lines: {:?}", markdown_options.max_lines);
    
    println!("\n2. Plain Text Export Options:");
    println!("   Format: {:?}", plain_text_options.format);
    println!("   Include line numbers: {}", plain_text_options.include_line_numbers);
    println!("   Include empty lines: {}", plain_text_options.include_empty_lines);
    
    println!("\n3. HTML Export Options:");
    println!("   Format: {:?}", html_options.format);
    println!("   Strip ANSI: {}", html_options.strip_ansi);
    println!("   Include line numbers: {}", html_options.include_line_numbers);
    
    // Try to export (will fail since no sessions exist)
    println!("\n4. Testing Export Functions:");
    
    match manager.export_active_session(&markdown_options) {
        Ok(result) => {
            println!("   ✓ Export successful: {} lines exported", result.metadata.line_count);
        }
        Err(e) => {
            println!("   ✗ Export failed (expected): {}", e);
        }
    }
    
    match manager.get_active_session_text() {
        Ok(text) => {
            println!("   ✓ Got session text: {} characters", text.len());
        }
        Err(e) => {
            println!("   ✗ Get text failed (expected): {}", e);
        }
    }
    
    println!("\n5. Export Dialog Demo:");
    let mut dialog = iterminal::export_ui::ExportDialog::default();
    println!("   Initial state - Open: {}", dialog.is_open);
    
    dialog.open();
    println!("   After open() - Open: {}", dialog.is_open);
    
    dialog.close();
    println!("   After close() - Open: {}", dialog.is_open);
    
    println!("\n6. Quick Export Demo:");
    match iterminal::export_ui::QuickExport::copy_as_text(&manager) {
        Ok(_) => println!("   ✓ Quick copy successful"),
        Err(e) => println!("   ✗ Quick copy failed (expected): {}", e),
    }
    
    match iterminal::export_ui::QuickExport::export_as_markdown(&manager) {
        Ok(result) => println!("   ✓ Quick markdown export: {} characters", result.len()),
        Err(e) => println!("   ✗ Quick markdown export failed (expected): {}", e),
    }
    
    println!("\nDemo completed! In a real application:");
    println!("- Create terminal sessions using EguiTerminalManager::create_session()");
    println!("- Run commands in the terminal");
    println!("- Use export functions to save terminal output");
    println!("- Display export dialogs in the UI for user interaction");
}
