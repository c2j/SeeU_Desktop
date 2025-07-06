use crate::export::{ExportFormat, ExportOptions, ExportResult};
use std::path::PathBuf;
use arboard::Clipboard;

/// Export dialog state
#[derive(Debug, Clone)]
pub struct ExportDialog {
    /// Whether the dialog is open
    pub is_open: bool,
    /// Export options
    pub options: ExportOptions,
    /// Export result (if any)
    pub last_result: Option<ExportResult>,
    /// Error message (if any)
    pub error_message: Option<String>,
    /// File path for saving
    pub save_path: Option<PathBuf>,
    /// Whether to show the export result
    pub show_result: bool,
}

impl Default for ExportDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            options: ExportOptions::default(),
            last_result: None,
            error_message: None,
            save_path: None,
            show_result: false,
        }
    }
}

impl ExportDialog {
    /// Open the export dialog
    pub fn open(&mut self) {
        self.is_open = true;
        self.error_message = None;
        self.last_result = None;
        self.show_result = false;
    }

    /// Close the export dialog
    pub fn close(&mut self) {
        self.is_open = false;
        self.show_result = false;
    }

    /// Copy content to clipboard
    pub fn copy_to_clipboard(&self, content: &str) -> Result<(), String> {
        match Clipboard::new() {
            Ok(mut clipboard) => {
                match clipboard.set_text(content.to_string()) {
                    Ok(_) => {
                        log::info!("Content copied to clipboard successfully");
                        Ok(())
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to copy to clipboard: {}", e);
                        log::error!("{}", error_msg);
                        Err(error_msg)
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to access clipboard: {}", e);
                log::error!("{}", error_msg);
                Err(error_msg)
            }
        }
    }


}

/// Quick export functions for common use cases
pub struct QuickExport;

impl QuickExport {
    /// Quick copy to clipboard as plain text
    pub fn copy_as_text(terminal_manager: &crate::egui_terminal::EguiTerminalManager) -> Result<(), String> {
        let options = ExportOptions {
            format: ExportFormat::PlainText,
            include_metadata: false,
            include_empty_lines: false,
            max_lines: None,
            strip_ansi: true,
            include_line_numbers: false,
        };

        match terminal_manager.export_active_session(&options) {
            Ok(result) => {
                // Copy to clipboard using arboard
                match Clipboard::new() {
                    Ok(mut clipboard) => {
                        match clipboard.set_text(result.content) {
                            Ok(_) => {
                                log::info!("Quick export: {} lines copied to clipboard", result.metadata.line_count);
                                Ok(())
                            }
                            Err(e) => {
                                let error_msg = format!("Failed to copy to clipboard: {}", e);
                                log::error!("{}", error_msg);
                                Err(error_msg)
                            }
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to access clipboard: {}", e);
                        log::error!("{}", error_msg);
                        Err(error_msg)
                    }
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Quick export as markdown
    pub fn export_as_markdown(terminal_manager: &crate::egui_terminal::EguiTerminalManager) -> Result<String, String> {
        let options = ExportOptions {
            format: ExportFormat::Markdown,
            include_metadata: true,
            include_empty_lines: false,
            max_lines: None,
            strip_ansi: true,
            include_line_numbers: false,
        };

        match terminal_manager.export_active_session(&options) {
            Ok(result) => Ok(result.content),
            Err(e) => Err(e.to_string()),
        }
    }
}
