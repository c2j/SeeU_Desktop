use iterminal::help_content::{TerminalHelpContent, HelpSection, HelpSubsection};
use iterminal::help_ui::TerminalHelpUI;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_content_creation() {
        let help_content = TerminalHelpContent::new();
        
        // Check that default sections are created
        let sections = help_content.get_sections();
        assert!(!sections.is_empty(), "Help content should have sections");
        
        // Check specific sections exist
        assert!(help_content.get_section("overview").is_some(), "Overview section should exist");
        assert!(help_content.get_section("alacritty").is_some(), "Alacritty section should exist");
        assert!(help_content.get_section("sessions").is_some(), "Sessions section should exist");
        assert!(help_content.get_section("export").is_some(), "Export section should exist");
        assert!(help_content.get_section("shortcuts").is_some(), "Shortcuts section should exist");
        assert!(help_content.get_section("config").is_some(), "Config section should exist");
        assert!(help_content.get_section("tips").is_some(), "Tips section should exist");
    }

    #[test]
    fn test_help_section_content() {
        let help_content = TerminalHelpContent::new();
        
        // Test overview section
        if let Some(overview) = help_content.get_section("overview") {
            assert!(!overview.title.is_empty(), "Overview title should not be empty");
            assert!(!overview.content.is_empty(), "Overview content should not be empty");
            assert!(!overview.subsections.is_empty(), "Overview should have subsections");
        }

        // Test Alacritty section
        if let Some(alacritty) = help_content.get_section("alacritty") {
            assert!(alacritty.title.contains("Alacritty"), "Alacritty section should mention Alacritty");
            assert!(!alacritty.subsections.is_empty(), "Alacritty section should have subsections");
            
            // Check for specific Alacritty features
            let has_gpu_section = alacritty.subsections.iter().any(|sub| 
                sub.title.contains("GPU") || sub.content.contains("GPU"));
            assert!(has_gpu_section, "Should have GPU acceleration information");
            
            let has_unicode_section = alacritty.subsections.iter().any(|sub| 
                sub.title.contains("Unicode") || sub.content.contains("Unicode"));
            assert!(has_unicode_section, "Should have Unicode support information");
        }
    }

    #[test]
    fn test_help_subsection_structure() {
        let help_content = TerminalHelpContent::new();
        
        for section in help_content.get_sections() {
            for subsection in &section.subsections {
                assert!(!subsection.title.is_empty(), "Subsection title should not be empty");
                assert!(!subsection.content.is_empty(), "Subsection content should not be empty");
                // Examples are optional, so we don't assert they exist
            }
        }
    }

    #[test]
    fn test_help_section_keys_order() {
        let help_content = TerminalHelpContent::new();
        let keys = help_content.get_section_keys();
        
        // Check that keys are in logical order
        assert_eq!(keys[0], "overview", "First section should be overview");
        assert_eq!(keys[1], "alacritty", "Second section should be alacritty");
        assert!(keys.contains(&"sessions"), "Should include sessions");
        assert!(keys.contains(&"export"), "Should include export");
        assert!(keys.contains(&"shortcuts"), "Should include shortcuts");
        assert!(keys.contains(&"config"), "Should include config");
        assert!(keys.contains(&"tips"), "Should include tips");
    }

    #[test]
    fn test_help_ui_initialization() {
        let help_ui = TerminalHelpUI::default();
        
        assert!(!help_ui.is_open, "Help UI should start closed");
        assert_eq!(help_ui.selected_section, Some("overview".to_string()), "Should default to overview section");
        assert!(help_ui.search_query.is_empty(), "Search query should start empty");
        assert!(!help_ui.show_search_results, "Search results should start hidden");
    }

    #[test]
    fn test_help_ui_open_close() {
        let mut help_ui = TerminalHelpUI::default();
        
        // Test opening
        help_ui.open();
        assert!(help_ui.is_open, "Help UI should be open after calling open()");
        assert!(help_ui.selected_section.is_some(), "Should have a selected section when open");
        
        // Test closing
        help_ui.close();
        assert!(!help_ui.is_open, "Help UI should be closed after calling close()");
        assert!(help_ui.search_query.is_empty(), "Search query should be cleared on close");
        assert!(!help_ui.show_search_results, "Search results should be hidden on close");
    }

    #[test]
    fn test_alacritty_features_coverage() {
        let help_content = TerminalHelpContent::new();
        
        if let Some(alacritty_section) = help_content.get_section("alacritty") {
            let content_text = format!("{} {}", 
                alacritty_section.content,
                alacritty_section.subsections.iter()
                    .map(|sub| format!("{} {}", sub.title, sub.content))
                    .collect::<Vec<_>>()
                    .join(" ")
            ).to_lowercase();
            
            // Check for key Alacritty features
            assert!(content_text.contains("gpu") || content_text.contains("硬件加速"), 
                "Should mention GPU acceleration");
            assert!(content_text.contains("unicode") || content_text.contains("字符"), 
                "Should mention Unicode support");
            assert!(content_text.contains("color") || content_text.contains("颜色"), 
                "Should mention color support");
            assert!(content_text.contains("font") || content_text.contains("字体"), 
                "Should mention font rendering");
        }
    }

    #[test]
    fn test_session_management_help() {
        let help_content = TerminalHelpContent::new();
        
        if let Some(sessions_section) = help_content.get_section("sessions") {
            let content_text = format!("{} {}", 
                sessions_section.content,
                sessions_section.subsections.iter()
                    .map(|sub| format!("{} {}", sub.title, sub.content))
                    .collect::<Vec<_>>()
                    .join(" ")
            ).to_lowercase();
            
            // Check for session management features
            assert!(content_text.contains("save") || content_text.contains("保存"), 
                "Should mention session saving");
            assert!(content_text.contains("restore") || content_text.contains("恢复"), 
                "Should mention session restoration");
            assert!(content_text.contains("history") || content_text.contains("历史"), 
                "Should mention session history");
        }
    }

    #[test]
    fn test_export_features_help() {
        let help_content = TerminalHelpContent::new();
        
        if let Some(export_section) = help_content.get_section("export") {
            let content_text = format!("{} {}", 
                export_section.content,
                export_section.subsections.iter()
                    .map(|sub| format!("{} {}", sub.title, sub.content))
                    .collect::<Vec<_>>()
                    .join(" ")
            ).to_lowercase();
            
            // Check for export features
            assert!(content_text.contains("markdown"), "Should mention Markdown export");
            assert!(content_text.contains("html"), "Should mention HTML export");
            assert!(content_text.contains("text") || content_text.contains("文本"), 
                "Should mention text export");
            assert!(content_text.contains("clipboard") || content_text.contains("剪贴板"), 
                "Should mention clipboard functionality");
        }
    }

    #[test]
    fn test_keyboard_shortcuts_help() {
        let help_content = TerminalHelpContent::new();
        
        if let Some(shortcuts_section) = help_content.get_section("shortcuts") {
            let content_text = format!("{} {}", 
                shortcuts_section.content,
                shortcuts_section.subsections.iter()
                    .map(|sub| format!("{} {} {}", sub.title, sub.content, sub.examples.join(" ")))
                    .collect::<Vec<_>>()
                    .join(" ")
            ).to_lowercase();
            
            // Check for common shortcuts
            assert!(content_text.contains("ctrl+c"), "Should mention Ctrl+C");
            assert!(content_text.contains("ctrl+r"), "Should mention Ctrl+R");
            assert!(content_text.contains("ctrl+l"), "Should mention Ctrl+L");
        }
    }

    #[test]
    fn test_help_content_examples() {
        let help_content = TerminalHelpContent::new();
        
        let mut total_examples = 0;
        for section in help_content.get_sections() {
            for subsection in &section.subsections {
                total_examples += subsection.examples.len();
                
                // Check that examples are not empty strings
                for example in &subsection.examples {
                    assert!(!example.is_empty(), "Examples should not be empty");
                }
            }
        }
        
        assert!(total_examples > 0, "Help content should include examples");
    }

    #[test]
    fn test_help_content_chinese_support() {
        let help_content = TerminalHelpContent::new();
        
        // Check that content includes Chinese text (indicating localization)
        let mut has_chinese = false;
        for section in help_content.get_sections() {
            if section.title.chars().any(|c| c as u32 > 127) ||
               section.content.chars().any(|c| c as u32 > 127) {
                has_chinese = true;
                break;
            }
            
            for subsection in &section.subsections {
                if subsection.title.chars().any(|c| c as u32 > 127) ||
                   subsection.content.chars().any(|c| c as u32 > 127) {
                    has_chinese = true;
                    break;
                }
            }
            
            if has_chinese {
                break;
            }
        }
        
        assert!(has_chinese, "Help content should include Chinese text");
    }
}
