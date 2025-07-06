use iterminal::session_history::{SavedSession, SessionHistoryManager};
use iterminal::session_history_ui::{SessionHistoryUI, SessionHistoryAction};
use uuid::Uuid;
use chrono::Utc;

fn main() {
    println!("iTerminal Session History Demo");
    println!("==============================");
    
    // Initialize session history manager
    let mut history_manager = match SessionHistoryManager::new() {
        Ok(manager) => {
            println!("✅ Session history manager initialized successfully");
            manager
        }
        Err(e) => {
            println!("❌ Failed to initialize session history manager: {}", e);
            return;
        }
    };
    
    println!("📊 Current session count: {}", history_manager.session_count());
    
    // Create some demo sessions
    println!("\n🔧 Creating demo sessions...");
    
    let demo_sessions = vec![
        ("Development Session", "cd ~/projects\nnpm install\nnpm run dev\n# Development server started"),
        ("Database Backup", "pg_dump mydb > backup.sql\nls -la backup.sql\n# Backup completed successfully"),
        ("System Monitoring", "top\nhtop\ndf -h\nfree -m\n# System resources checked"),
        ("Git Operations", "git status\ngit add .\ngit commit -m 'Update features'\ngit push origin main\n# Changes pushed"),
    ];
    
    for (title, content) in demo_sessions {
        let now = Utc::now();
        let mut session = SavedSession::new(
            Uuid::new_v4(),
            title.to_string(),
            now,
            now,
            content.to_string(),
        );
        
        // Add some tags and notes
        match title {
            "Development Session" => {
                session.add_tag("development".to_string());
                session.add_tag("nodejs".to_string());
                session.set_notes("Frontend development session with hot reload".to_string());
            }
            "Database Backup" => {
                session.add_tag("database".to_string());
                session.add_tag("backup".to_string());
                session.set_notes("Regular database backup procedure".to_string());
            }
            "System Monitoring" => {
                session.add_tag("monitoring".to_string());
                session.add_tag("system".to_string());
                session.set_notes("System health check and resource monitoring".to_string());
            }
            "Git Operations" => {
                session.add_tag("git".to_string());
                session.add_tag("version-control".to_string());
                session.set_notes("Standard git workflow for feature deployment".to_string());
            }
            _ => {}
        }
        
        match history_manager.save_session(session) {
            Ok(_) => println!("  ✅ Saved session: {}", title),
            Err(e) => println!("  ❌ Failed to save session '{}': {}", title, e),
        }
    }
    
    println!("\n📊 Session count after adding demos: {}", history_manager.session_count());
    
    // Demonstrate session listing
    println!("\n📋 All saved sessions:");
    let sessions = history_manager.get_all_sessions();
    let session_data: Vec<_> = sessions.iter().map(|s| (s.id, s.title.clone(), s.created_at, s.tags.clone(), s.notes.clone(), s.content.clone())).collect();

    for (i, (id, title, created_at, tags, notes, content)) in session_data.iter().enumerate() {
        println!("  {}. {} (ID: {})", i + 1, title, id);
        println!("     Created: {}", created_at.format("%Y-%m-%d %H:%M:%S"));
        if !tags.is_empty() {
            println!("     Tags: {}", tags.join(", "));
        }
        if !notes.is_empty() {
            println!("     Notes: {}", notes);
        }
        println!("     Content preview: {}",
            content.lines().take(2).collect::<Vec<_>>().join(" | "));
        println!();
    }
    
    // Demonstrate search functionality
    println!("🔍 Search demonstrations:");
    
    let search_queries = vec!["development", "git", "database", "monitoring"];
    for query in search_queries {
        let results = history_manager.search_sessions(query);
        println!("  Search for '{}': {} results", query, results.len());
        for result in results {
            println!("    - {}", result.title);
        }
    }
    
    // Demonstrate session operations
    println!("\n🔧 Session operations demo:");

    if let Some((session_id, _, _, _, _, _)) = session_data.first() {
        let session_id = *session_id;
        
        // Load specific session
        match history_manager.load_session(session_id) {
            Ok(Some(session)) => {
                println!("  ✅ Successfully loaded session: {}", session.title);
            }
            Ok(None) => {
                println!("  ❌ Session not found");
            }
            Err(e) => {
                println!("  ❌ Failed to load session: {}", e);
            }
        }
        
        // Update session (simulate editing)
        if let Some(session) = history_manager.get_session_mut(&session_id) {
            session.add_tag("demo".to_string());
            session.set_notes(format!("{} (Updated in demo)", session.notes));
            
            let updated_session = session.clone();
            match history_manager.save_session(updated_session) {
                Ok(_) => println!("  ✅ Successfully updated session with demo tag"),
                Err(e) => println!("  ❌ Failed to update session: {}", e),
            }
        }
    }
    
    // Demonstrate UI state management
    println!("\n🖥️  UI State Management Demo:");
    let mut ui_state = SessionHistoryUI::default();
    
    println!("  Initial state - Open: {}", ui_state.is_open);
    
    ui_state.open();
    println!("  After open() - Open: {}", ui_state.is_open);
    
    ui_state.set_success("Demo operation completed successfully!".to_string());
    if let Some(msg) = &ui_state.success_message {
        println!("  Success message: {}", msg);
    }
    
    ui_state.set_error("Demo error message".to_string());
    if let Some(msg) = &ui_state.error_message {
        println!("  Error message: {}", msg);
    }
    
    ui_state.clear_messages();
    println!("  Messages cleared - Error: {:?}, Success: {:?}", 
        ui_state.error_message, ui_state.success_message);
    
    ui_state.close();
    println!("  After close() - Open: {}", ui_state.is_open);
    
    // Demonstrate action types
    println!("\n⚡ Action Types Demo:");
    let mut demo_actions = vec![
        SessionHistoryAction::Refresh,
        SessionHistoryAction::ClearAllSessions,
    ];

    if let Some((session_id, _, _, _, _, _)) = session_data.first() {
        let session_id = *session_id;
        demo_actions.extend(vec![
            SessionHistoryAction::RestoreSession(session_id),
            SessionHistoryAction::DeleteSession(session_id),
            SessionHistoryAction::UpdateSession {
                id: session_id,
                title: "Updated Title".to_string(),
                notes: "Updated notes".to_string(),
                tags: vec!["new-tag".to_string()],
            },
        ]);
    }
    
    for (i, action) in demo_actions.iter().enumerate() {
        match action {
            SessionHistoryAction::RestoreSession(id) => {
                println!("  {}. Restore Session: {}", i + 1, id);
            }
            SessionHistoryAction::DeleteSession(id) => {
                println!("  {}. Delete Session: {}", i + 1, id);
            }
            SessionHistoryAction::UpdateSession { id, title, .. } => {
                println!("  {}. Update Session: {} -> {}", i + 1, id, title);
            }
            SessionHistoryAction::ClearAllSessions => {
                println!("  {}. Clear All Sessions", i + 1);
            }
            SessionHistoryAction::Refresh => {
                println!("  {}. Refresh Session List", i + 1);
            }
        }
    }
    
    // Final statistics
    println!("\n📈 Final Statistics:");
    println!("  Total sessions: {}", history_manager.session_count());
    println!("  Sessions with tags: {}",
        session_data.iter().filter(|(_, _, _, tags, _, _)| !tags.is_empty()).count());
    println!("  Sessions with notes: {}",
        session_data.iter().filter(|(_, _, _, _, notes, _)| !notes.is_empty()).count());
    
    println!("\n🎉 Session History Demo Completed!");
    println!("In a real application:");
    println!("  - Sessions would be automatically saved when terminals close");
    println!("  - Users can browse and restore sessions through the UI");
    println!("  - Sessions can be organized with tags and notes");
    println!("  - Search functionality helps find specific sessions");
    println!("  - Session content can be exported or copied");
}
