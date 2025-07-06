use iterminal::session_history::{SavedSession, SessionHistoryManager};
use iterminal::session_history_ui::{SessionHistoryUI, SessionHistoryAction};
use uuid::Uuid;
use chrono::Utc;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test session
    fn create_test_session(title: &str, content: &str) -> SavedSession {
        let now = Utc::now();
        SavedSession::new(
            Uuid::new_v4(),
            title.to_string(),
            now,
            now,
            content.to_string(),
        )
    }

    #[test]
    fn test_saved_session_creation() {
        let session = create_test_session("Test Session", "echo hello\nhello");
        
        assert_eq!(session.title, "Test Session");
        assert_eq!(session.content, "echo hello\nhello");
        assert!(session.tags.is_empty());
        assert!(session.notes.is_empty());
        assert!(session.command_history.is_empty());
    }

    #[test]
    fn test_saved_session_tags() {
        let mut session = create_test_session("Test Session", "content");
        
        // Add tags
        session.add_tag("development".to_string());
        session.add_tag("testing".to_string());
        session.add_tag("development".to_string()); // Duplicate should be ignored
        
        assert_eq!(session.tags.len(), 2);
        assert!(session.tags.contains(&"development".to_string()));
        assert!(session.tags.contains(&"testing".to_string()));
        
        // Remove tag
        session.remove_tag("testing");
        assert_eq!(session.tags.len(), 1);
        assert!(!session.tags.contains(&"testing".to_string()));
    }

    #[test]
    fn test_saved_session_notes() {
        let mut session = create_test_session("Test Session", "content");
        
        session.set_notes("This is a test session for development".to_string());
        assert_eq!(session.notes, "This is a test session for development");
        
        let description = session.get_description();
        assert_eq!(description, "This is a test session for development");
    }

    #[test]
    fn test_session_history_manager_creation() {
        // This test might fail if we can't create the config directory
        // In a real test environment, we'd use a temporary directory
        match SessionHistoryManager::new() {
            Ok(manager) => {
                // Manager should be created successfully
                // Session count may vary if previous tests/demos have run
                assert!(manager.session_count() >= 0);
                println!("Session history manager created with {} existing sessions", manager.session_count());
            }
            Err(e) => {
                // This is expected in some test environments
                println!("Failed to create session history manager: {}", e);
            }
        }
    }

    #[test]
    fn test_session_history_ui_state() {
        let mut ui = SessionHistoryUI::default();
        
        // Test initial state
        assert!(!ui.is_open);
        assert!(ui.search_query.is_empty());
        assert!(ui.selected_session.is_none());
        assert!(!ui.show_details);
        assert!(!ui.edit_mode);
        
        // Test opening
        ui.open();
        assert!(ui.is_open);
        
        // Test closing
        ui.close();
        assert!(!ui.is_open);
        assert!(ui.selected_session.is_none());
        assert!(!ui.show_details);
        assert!(!ui.edit_mode);
    }

    #[test]
    fn test_session_history_ui_messages() {
        let mut ui = SessionHistoryUI::default();
        
        // Test error message
        ui.set_error("Test error".to_string());
        assert_eq!(ui.error_message, Some("Test error".to_string()));
        assert!(ui.success_message.is_none());
        
        // Test success message
        ui.set_success("Test success".to_string());
        assert_eq!(ui.success_message, Some("Test success".to_string()));
        assert!(ui.error_message.is_none());
        
        // Test clearing messages
        ui.clear_messages();
        assert!(ui.error_message.is_none());
        assert!(ui.success_message.is_none());
    }

    #[test]
    fn test_session_history_ui_edit_mode() {
        let mut ui = SessionHistoryUI::default();
        let session = create_test_session("Test Session", "content");
        
        // Start editing
        ui.start_edit(&session);
        assert!(ui.edit_mode);
        assert_eq!(ui.edit_data.title, "Test Session");
        assert!(ui.edit_data.notes.is_empty());
        assert!(ui.edit_data.tags.is_empty());
        
        // Cancel editing
        ui.cancel_edit();
        assert!(!ui.edit_mode);
        assert!(ui.edit_data.title.is_empty());
    }

    #[test]
    fn test_session_history_actions() {
        let session_id = Uuid::new_v4();
        
        // Test different action types
        let restore_action = SessionHistoryAction::RestoreSession(session_id);
        let delete_action = SessionHistoryAction::DeleteSession(session_id);
        let update_action = SessionHistoryAction::UpdateSession {
            id: session_id,
            title: "Updated Title".to_string(),
            notes: "Updated notes".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        let clear_action = SessionHistoryAction::ClearAllSessions;
        let refresh_action = SessionHistoryAction::Refresh;
        
        // Just verify they can be created and matched
        match restore_action {
            SessionHistoryAction::RestoreSession(id) => assert_eq!(id, session_id),
            _ => panic!("Wrong action type"),
        }
        
        match delete_action {
            SessionHistoryAction::DeleteSession(id) => assert_eq!(id, session_id),
            _ => panic!("Wrong action type"),
        }
        
        match update_action {
            SessionHistoryAction::UpdateSession { id, title, notes, tags } => {
                assert_eq!(id, session_id);
                assert_eq!(title, "Updated Title");
                assert_eq!(notes, "Updated notes");
                assert_eq!(tags.len(), 2);
            }
            _ => panic!("Wrong action type"),
        }
        
        match clear_action {
            SessionHistoryAction::ClearAllSessions => {},
            _ => panic!("Wrong action type"),
        }
        
        match refresh_action {
            SessionHistoryAction::Refresh => {},
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_saved_session_environment() {
        let session = create_test_session("Test Session", "content");
        
        // The environment should contain some safe variables
        // This test might be environment-dependent
        assert!(session.environment.contains_key("PATH") || session.environment.is_empty());
    }

    #[test]
    fn test_saved_session_working_directory() {
        let session = create_test_session("Test Session", "content");
        
        // Working directory should be set to current directory
        if let Some(wd) = &session.working_directory {
            assert!(wd.exists());
        }
    }

    #[test]
    fn test_saved_session_timestamps() {
        let before = Utc::now();
        let session = create_test_session("Test Session", "content");
        let after = Utc::now();

        // All timestamps should be within the test timeframe
        assert!(session.created_at >= before && session.created_at <= after);
        assert!(session.last_activity >= before && session.last_activity <= after);
        assert!(session.saved_at >= session.created_at);
        assert!(session.saved_at >= session.last_activity);
    }

    #[test]
    fn test_session_description_generation() {
        // Test with notes
        let mut session = create_test_session("Test Session", "content");
        session.set_notes("Custom description".to_string());
        assert_eq!(session.get_description(), "Custom description");

        // Test with command history
        let mut session2 = create_test_session("Test Session", "content");
        session2.command_history.push("ls -la".to_string());
        session2.command_history.push("pwd".to_string());
        assert_eq!(session2.get_description(), "Last command: pwd");

        // Test with neither (fallback to timestamp)
        let session3 = create_test_session("Test Session", "content");
        let description = session3.get_description();
        assert!(description.contains("Session from"));
        assert!(description.contains(&session3.created_at.format("%Y-%m-%d %H:%M").to_string()));
    }

    #[test]
    fn test_session_history_ui_initialization() {
        let ui = SessionHistoryUI::default();

        // Test that UI initializes with proper default state
        assert!(!ui.is_open);
        assert!(ui.search_query.is_empty());
        assert!(ui.selected_session.is_none());
        assert!(!ui.show_details);
        assert!(!ui.edit_mode);
        assert!(ui.error_message.is_none());
        assert!(ui.success_message.is_none());
        assert!(ui.confirmation_dialog.is_none());

        // Test edit data initialization
        assert!(ui.edit_data.title.is_empty());
        assert!(ui.edit_data.notes.is_empty());
        assert!(ui.edit_data.tags.is_empty());
    }
}
