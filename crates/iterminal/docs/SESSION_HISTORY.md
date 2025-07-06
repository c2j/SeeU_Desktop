# iTerminal Session History Implementation

## Overview

The iTerminal session history feature allows users to save, manage, and restore terminal sessions. This provides continuity across application restarts and enables users to return to previous work contexts with their command history, working directory, and session content intact.

## Features

### 🔄 Session Management
- **Save Sessions**: Automatically or manually save terminal sessions with full context
- **Restore Sessions**: Create new terminal sessions from saved history
- **Session Metadata**: Store titles, tags, notes, timestamps, and working directories
- **Content Preservation**: Save terminal output and command history

### 🔍 Search and Organization
- **Full-Text Search**: Search across session titles, content, tags, and notes
- **Tag System**: Organize sessions with custom tags
- **Notes**: Add descriptive notes to sessions for better organization
- **Chronological Sorting**: Sessions sorted by last activity time

### 🖥️ User Interface
- **History Dialog**: Comprehensive UI for browsing and managing sessions
- **Quick Actions**: Save current session with one click
- **Edit Mode**: Modify session metadata (title, tags, notes)
- **Confirmation Dialogs**: Safe deletion and restoration with user confirmation

## Architecture

### Core Components

#### 1. `SavedSession` Structure
```rust
pub struct SavedSession {
    pub id: Uuid,                    // Unique identifier
    pub title: String,               // Display name
    pub created_at: DateTime<Utc>,   // Creation timestamp
    pub last_activity: DateTime<Utc>, // Last activity time
    pub saved_at: DateTime<Utc>,     // When saved to history
    pub working_directory: Option<PathBuf>, // Working directory
    pub content: String,             // Terminal output
    pub command_history: Vec<String>, // Command history
    pub environment: HashMap<String, String>, // Safe env vars
    pub tags: Vec<String>,           // Organization tags
    pub notes: String,               // User notes
}
```

#### 2. `SessionHistoryManager`
- **Storage**: JSON files in user config directory
- **Caching**: In-memory cache for fast access
- **Limits**: Configurable maximum session count with automatic cleanup
- **Search**: Full-text search across all session data

#### 3. `SessionHistoryUI`
- **State Management**: Dialog state, selection, editing mode
- **User Interactions**: Browse, search, edit, delete, restore
- **Error Handling**: User-friendly error and success messages

### Data Flow

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Terminal      │    │  Session         │    │  Storage        │
│   Session       │    │  History         │    │  (JSON Files)   │
│                 │───►│  Manager         │◄──►│                 │
│ - Save Session  │    │                  │    │ - Persistent    │
│ - Get Content   │    │ - Save/Load      │    │ - Searchable    │
└─────────────────┘    │ - Search         │    │ - Organized     │
                       │ - Manage         │    └─────────────────┘
                       └──────────────────┘
                              ▲
                              │
                       ┌──────────────────┐
                       │  Session         │
                       │  History UI      │
                       │                  │
                       │ - Browse         │
                       │ - Search         │
                       │ - Edit           │
                       │ - Restore        │
                       └──────────────────┘
```

## Usage

### Basic Operations

#### Saving a Session
```rust
// Save current active session
if let Some(history_manager) = &mut state.session_history_manager {
    match state.egui_terminal_manager.save_active_session_to_history(history_manager) {
        Ok(session_id) => println!("Session saved: {}", session_id),
        Err(e) => println!("Failed to save: {}", e),
    }
}
```

#### Restoring a Session
```rust
// Restore from saved session
if let Some(saved_session) = history_manager.get_session(&session_id) {
    match terminal_manager.restore_session_from_history(saved_session, &ctx) {
        Ok(new_session_id) => println!("Session restored: {}", new_session_id),
        Err(e) => println!("Failed to restore: {}", e),
    }
}
```

#### Searching Sessions
```rust
// Search for sessions
let results = history_manager.search_sessions("development");
for session in results {
    println!("Found: {} - {}", session.title, session.get_description());
}
```

### UI Integration

#### Opening Session History
```rust
// In terminal UI
if ui.button("📚 Session History").clicked() {
    state.session_history_ui.open();
}

// Render the dialog
if let Some(action) = state.session_history_ui.render(&ctx, history_manager) {
    handle_session_history_action(state, action);
}
```

#### Handling Actions
```rust
fn handle_session_history_action(state: &mut ITerminalState, action: SessionHistoryAction) {
    match action {
        SessionHistoryAction::RestoreSession(id) => {
            // Restore session logic
        }
        SessionHistoryAction::DeleteSession(id) => {
            // Delete session logic
        }
        SessionHistoryAction::UpdateSession { id, title, notes, tags } => {
            // Update session metadata
        }
        // ... other actions
    }
}
```

## Storage Format

### File Structure
```
~/.config/seeu_desktop/iterminal/sessions/
├── {session-uuid-1}.json
├── {session-uuid-2}.json
└── {session-uuid-3}.json
```

### JSON Format
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "Development Session",
  "created_at": "2025-07-05T22:00:00Z",
  "last_activity": "2025-07-05T22:30:00Z",
  "saved_at": "2025-07-05T22:30:15Z",
  "working_directory": "/home/user/projects",
  "content": "$ npm install\n$ npm run dev\nDevelopment server started...",
  "command_history": ["npm install", "npm run dev"],
  "environment": {
    "PATH": "/usr/local/bin:/usr/bin",
    "HOME": "/home/user",
    "SHELL": "/bin/zsh"
  },
  "tags": ["development", "nodejs"],
  "notes": "Frontend development session with hot reload"
}
```

## Configuration

### Session Limits
```rust
// Set maximum number of sessions to keep
history_manager.set_max_sessions(100);
```

### Storage Directory
The storage directory is automatically determined:
- **Linux/macOS**: `~/.config/seeu_desktop/iterminal/sessions/`
- **Windows**: `%APPDATA%\seeu_desktop\iterminal\sessions\`

## Security Considerations

### Environment Variables
Only safe environment variables are stored:
- `PATH`, `HOME`, `USER`, `SHELL`, `TERM`
- `LANG`, `LC_ALL`, `PWD`, `OLDPWD`
- `EDITOR`, `VISUAL`, `PAGER`

Sensitive variables (passwords, tokens, etc.) are excluded.

### File Permissions
Session files are stored with user-only read/write permissions to protect sensitive terminal content.

## Testing

### Unit Tests
```bash
cargo test --package iterminal --test session_history_tests
```

### Demo Application
```bash
cargo run --package iterminal --example session_history_demo
```

## Future Enhancements

### Planned Features
- **Session Templates**: Create reusable session templates
- **Export/Import**: Export sessions to share or backup
- **Session Groups**: Organize related sessions into groups
- **Automatic Tagging**: AI-powered automatic tag suggestions
- **Session Snapshots**: Save multiple snapshots of the same session

### Integration Opportunities
- **Note System**: Export sessions directly to note-taking system
- **Project Management**: Link sessions to project contexts
- **Cloud Sync**: Synchronize sessions across devices
- **Team Sharing**: Share sessions with team members

## Troubleshooting

### Common Issues

#### Storage Directory Creation Failed
```
Error: Failed to create storage directory: Permission denied
```
**Solution**: Ensure the application has write permissions to the config directory.

#### Session Not Found
```
Error: Session not found
```
**Solution**: The session may have been deleted or the storage file corrupted. Check the storage directory.

#### Failed to Load Session
```
Error: Serialization error: invalid JSON
```
**Solution**: Session file may be corrupted. Delete the problematic file or restore from backup.

#### UI Warning: "First use of ScrollArea ID"
```
Warning: First use of ScrollArea ID xxxx
```
**Solution**: This warning has been fixed by adding unique IDs to all ScrollArea and TextEdit components in the session history UI.

### Debug Information
Enable debug logging to troubleshoot issues:
```rust
log::set_max_level(log::LevelFilter::Debug);
```

## Performance

### Memory Usage
- Sessions are cached in memory for fast access
- Large session content is handled efficiently
- Configurable session limits prevent memory bloat

### Disk Usage
- JSON format provides good compression
- Automatic cleanup of old sessions
- Typical session file size: 1-10 KB

### Search Performance
- In-memory search for fast results
- Full-text search across all fields
- Results sorted by relevance and recency

## Conclusion

The iTerminal session history feature provides a comprehensive solution for managing terminal session continuity. With its robust storage system, intuitive UI, and powerful search capabilities, users can efficiently organize and restore their terminal work contexts, significantly improving productivity and workflow continuity.
