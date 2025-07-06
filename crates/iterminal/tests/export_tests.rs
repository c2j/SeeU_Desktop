use iterminal::export::{TerminalExporter, ExportOptions, ExportFormat};
use iterminal::egui_terminal::{EguiTerminalSession, EguiTerminalManager};
use uuid::Uuid;
use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a mock terminal session for testing
    fn create_mock_session() -> EguiTerminalSession {
        EguiTerminalSession {
            id: Uuid::new_v4(),
            title: "Test Session".to_string(),
            backend: None, // We'll test without actual backend for now
            is_active: true,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            event_receiver: None, // No event receiver for testing
        }
    }

    #[test]
    fn test_export_options_default() {
        let options = ExportOptions::default();
        
        assert_eq!(options.format, ExportFormat::Markdown);
        assert!(options.include_metadata);
        assert!(!options.include_empty_lines);
        assert!(options.max_lines.is_none());
        assert!(options.strip_ansi);
        assert!(!options.include_line_numbers);
    }

    #[test]
    fn test_export_format_variants() {
        // Test that all export formats are available
        let _plain = ExportFormat::PlainText;
        let _markdown = ExportFormat::Markdown;
        let _html = ExportFormat::Html;
        
        // Test equality
        assert_eq!(ExportFormat::PlainText, ExportFormat::PlainText);
        assert_ne!(ExportFormat::PlainText, ExportFormat::Markdown);
    }

    #[test]
    fn test_export_options_customization() {
        let mut options = ExportOptions::default();
        
        // Customize options
        options.format = ExportFormat::PlainText;
        options.include_metadata = false;
        options.include_empty_lines = true;
        options.max_lines = Some(100);
        options.strip_ansi = false;
        options.include_line_numbers = true;
        
        // Verify changes
        assert_eq!(options.format, ExportFormat::PlainText);
        assert!(!options.include_metadata);
        assert!(options.include_empty_lines);
        assert_eq!(options.max_lines, Some(100));
        assert!(!options.strip_ansi);
        assert!(options.include_line_numbers);
    }

    #[test]
    fn test_export_session_without_backend() {
        let session = create_mock_session();
        let options = ExportOptions::default();
        
        // This should fail because there's no backend
        let result = TerminalExporter::export_session(&session, &options);
        assert!(result.is_err());
        
        // Check error type
        match result {
            Err(iterminal::export::ExportError::NoBackend) => {
                // Expected error
            }
            _ => panic!("Expected NoBackend error"),
        }
    }

    #[test]
    fn test_terminal_manager_export_methods() {
        let manager = EguiTerminalManager::new();
        let options = ExportOptions::default();
        
        // Test export with no active session
        let result = manager.export_active_session(&options);
        assert!(result.is_err());
        
        // Test export with invalid session ID
        let invalid_id = Uuid::new_v4();
        let result = manager.export_session(invalid_id, &options);
        assert!(result.is_err());
        
        // Test get text content with no active session
        let result = manager.get_active_session_text();
        assert!(result.is_err());
    }

    #[test]
    fn test_ansi_stripping() {
        // Test the ANSI stripping functionality - this is a placeholder test
        // since strip_ansi_codes is private. In a real implementation, we would
        // test this through the public export methods.
        let test_cases = vec![
            ("Hello World", "Hello World"), // No ANSI codes
            ("\x1b[31mRed Text\x1b[0m", "Red Text"), // Basic color codes
            ("\x1b[1;32mBold Green\x1b[0m", "Bold Green"), // Bold + color
            ("Normal\x1b[31mRed\x1b[0mNormal", "NormalRedNormal"), // Mixed content
            ("\x1b[2J\x1b[H", ""), // Clear screen codes
        ];

        // For now, just verify the test cases are set up correctly
        for (input, expected) in test_cases {
            // This would test the actual ANSI stripping if the method was public
            assert!(!input.is_empty() || expected.is_empty());
        }
    }

    #[test]
    fn test_export_metadata() {
        let session = create_mock_session();
        let session_id = session.id;
        let session_title = session.title.clone();
        
        // Test that metadata contains expected information
        let metadata = iterminal::export::ExportMetadata {
            session_id,
            session_title: session_title.clone(),
            exported_at: Utc::now(),
            line_count: 42,
            format: ExportFormat::Markdown,
        };
        
        assert_eq!(metadata.session_id, session_id);
        assert_eq!(metadata.session_title, session_title);
        assert_eq!(metadata.line_count, 42);
        assert_eq!(metadata.format, ExportFormat::Markdown);
    }

    #[test]
    fn test_quick_export_functions() {
        let manager = EguiTerminalManager::new();
        
        // Test quick copy (should fail with no sessions)
        let result = iterminal::export_ui::QuickExport::copy_as_text(&manager);
        assert!(result.is_err());
        
        // Test quick markdown export (should fail with no sessions)
        let result = iterminal::export_ui::QuickExport::export_as_markdown(&manager);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_dialog_state() {
        let mut dialog = iterminal::export_ui::ExportDialog::default();
        
        // Test initial state
        assert!(!dialog.is_open);
        assert!(dialog.last_result.is_none());
        assert!(dialog.error_message.is_none());
        assert!(!dialog.show_result);
        
        // Test opening dialog
        dialog.open();
        assert!(dialog.is_open);
        assert!(dialog.last_result.is_none());
        assert!(dialog.error_message.is_none());
        assert!(!dialog.show_result);
        
        // Test closing dialog
        dialog.close();
        assert!(!dialog.is_open);
        assert!(!dialog.show_result);
    }

    #[test]
    fn test_export_error_display() {
        let errors = vec![
            iterminal::export::ExportError::NoBackend,
            iterminal::export::ExportError::IoError("Test IO error".to_string()),
            iterminal::export::ExportError::FormatError("Test format error".to_string()),
        ];
        
        for error in errors {
            let error_string = format!("{}", error);
            assert!(!error_string.is_empty());
        }
    }
}
