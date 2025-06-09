use std::collections::HashMap;
use uuid::Uuid;
use crate::session::TerminalSession;
use crate::command::{CommandExecutor, CommandMessage};
use crate::config::TerminalConfig;

/// Terminal manager that handles multiple sessions
#[derive(Debug)]
pub struct TerminalManager {
    /// All terminal sessions
    sessions: HashMap<Uuid, TerminalSession>,
    /// Currently active session ID
    active_session_id: Option<Uuid>,
    /// Command executor
    command_executor: CommandExecutor,
    /// Pending commands (command_id -> session_id)
    pending_commands: HashMap<Uuid, Uuid>,
    /// Terminal configuration
    config: TerminalConfig,
}

impl TerminalManager {
    /// Create a new terminal manager
    pub fn new(config: TerminalConfig) -> Self {
        let mut manager = Self {
            sessions: HashMap::new(),
            active_session_id: None,
            command_executor: CommandExecutor::new(),
            pending_commands: HashMap::new(),
            config,
        };

        // Create initial session
        manager.create_session("Terminal 1".to_string());
        
        manager
    }

    /// Create a new terminal session
    pub fn create_session(&mut self, title: String) -> Uuid {
        let working_dir = self.config.get_working_directory().to_string_lossy().to_string();
        let session = TerminalSession::new(title, working_dir, &self.config);
        let session_id = session.id;
        
        // Add welcome message
        let mut session = session;
        session.add_output("Welcome to SeeU Terminal!".to_string(), false);
        session.add_output(format!("Working directory: {}", session.working_directory), false);
        session.add_output("Type 'help' for available commands.".to_string(), false);
        session.add_output("".to_string(), false);
        
        self.sessions.insert(session_id, session);
        
        // Set as active if it's the first session
        if self.active_session_id.is_none() {
            self.active_session_id = Some(session_id);
        }
        
        session_id
    }

    /// Close a terminal session
    pub fn close_session(&mut self, session_id: Uuid) -> bool {
        if self.sessions.remove(&session_id).is_some() {
            // If this was the active session, switch to another one
            if self.active_session_id == Some(session_id) {
                self.active_session_id = self.sessions.keys().next().copied();
            }
            true
        } else {
            false
        }
    }

    /// Get the active session
    pub fn get_active_session(&self) -> Option<&TerminalSession> {
        self.active_session_id
            .and_then(|id| self.sessions.get(&id))
    }

    /// Get the active session mutably
    pub fn get_active_session_mut(&mut self) -> Option<&mut TerminalSession> {
        self.active_session_id
            .and_then(|id| self.sessions.get_mut(&id))
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: Uuid) -> Option<&TerminalSession> {
        self.sessions.get(&session_id)
    }

    /// Get a session by ID mutably
    pub fn get_session_mut(&mut self, session_id: Uuid) -> Option<&mut TerminalSession> {
        self.sessions.get_mut(&session_id)
    }

    /// Set the active session
    pub fn set_active_session(&mut self, session_id: Uuid) -> bool {
        if self.sessions.contains_key(&session_id) {
            self.active_session_id = Some(session_id);
            true
        } else {
            false
        }
    }

    /// Get all sessions
    pub fn get_all_sessions(&self) -> &HashMap<Uuid, TerminalSession> {
        &self.sessions
    }

    /// Execute a command in the active session
    pub fn execute_command(&mut self, command: String) -> bool {
        if let Some(session_id) = self.active_session_id {
            self.execute_command_in_session(session_id, command)
        } else {
            false
        }
    }

    /// Execute a command in a specific session
    pub fn execute_command_in_session(&mut self, session_id: Uuid, command: String) -> bool {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            let command = command.trim().to_string();
            
            if command.is_empty() {
                return true;
            }

            // Handle special commands
            match command.as_str() {
                "clear" => {
                    session.clear_output();
                    return true;
                }
                "exit" => {
                    // Close this session
                    self.close_session(session_id);
                    return true;
                }
                _ => {}
            }

            // Check if it's a built-in command
            if CommandExecutor::is_builtin(&command) {
                let result = CommandExecutor::execute_builtin(&command, &session.working_directory);
                
                // Handle cd command specially to update working directory
                if command.starts_with("cd ") && result.exit_code == Some(0) {
                    let path = command.strip_prefix("cd ").unwrap_or("").trim();
                    if path.is_empty() {
                        // cd with no arguments goes to home directory
                        if let Some(home) = dirs::home_dir() {
                            session.working_directory = home.to_string_lossy().to_string();
                        }
                    } else {
                        let target_path = if path.starts_with('/') || path.contains(':') {
                            std::path::PathBuf::from(path)
                        } else {
                            std::path::PathBuf::from(&session.working_directory).join(path)
                        };
                        
                        if target_path.exists() && target_path.is_dir() {
                            session.working_directory = target_path.to_string_lossy().to_string();
                        }
                    }
                }
                
                // Add output to session
                for line in result.stdout {
                    session.add_output(line, false);
                }
                for line in result.stderr {
                    session.add_output(line, false);
                }
                
                if let Some(exit_code) = result.exit_code {
                    session.history.update_last_exit_code(exit_code);
                }
            } else {
                // Execute as external command
                let command_id = Uuid::new_v4();
                self.pending_commands.insert(command_id, session_id);
                self.command_executor.execute_async(command_id, command, session.working_directory.clone());
            }
            
            true
        } else {
            false
        }
    }

    /// Update the terminal manager (process command results)
    pub fn update(&mut self) {
        let messages = self.command_executor.check_messages();
        
        for (command_id, message) in messages {
            if let Some(session_id) = self.pending_commands.get(&command_id).copied() {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    match message {
                        CommandMessage::Stdout(line) => {
                            session.add_output(line, false);
                        }
                        CommandMessage::Stderr(line) => {
                            session.add_output(line, false);
                        }
                        CommandMessage::Finished(exit_code) => {
                            session.history.update_last_exit_code(exit_code);
                            self.pending_commands.remove(&command_id);
                            
                            // Add exit code info if non-zero
                            if exit_code != 0 {
                                session.add_output(format!("Process exited with code: {}", exit_code), false);
                            }
                        }
                        CommandMessage::Error(error) => {
                            session.add_output(format!("Error: {}", error), false);
                            self.pending_commands.remove(&command_id);
                        }
                    }
                }
            }
        }
    }

    /// Get the terminal configuration
    pub fn get_config(&self) -> &TerminalConfig {
        &self.config
    }

    /// Update the terminal configuration
    pub fn update_config(&mut self, config: TerminalConfig) {
        self.config = config;
        
        // Update existing sessions with new config
        for session in self.sessions.values_mut() {
            session.max_output_lines = self.config.scrollback_lines;
        }
    }

    /// Get the number of sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Check if there are any active commands
    pub fn has_active_commands(&self) -> bool {
        !self.pending_commands.is_empty()
    }
}
