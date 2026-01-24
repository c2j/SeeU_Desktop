# SeeU Desktop - Agent Development Guide

## Build / Test / Lint Commands

### Building
```bash
# Build debug version
cargo build

# Build release (optimized)
cargo build --release

# Run the application
cargo run --release
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p inote
cargo test -p aiAssist
cargo test -p itools

# Run a single test
cargo test test_function_name

# Run tests with output
cargo test -- --nocapture

# Run tests for a specific file
cargo test --test integration_tests
```

### Linting / Formatting
```bash
# Format code with rustfmt (80 character max width)
cargo fmt

# Run clippy linter
cargo clippy

# Run clippy with custom thresholds (defined in crates/egui_term/clippy.toml)
cargo clippy -- -D warnings
```

### Cross-Platform Building
```bash
# vcpkg-based cross-compilation (recommended)
./scripts/setup-vcpkg.sh
./scripts/build-vcpkg.sh --target linux-x64
./scripts/build-vcpkg.sh --target windows-x64

# Traditional build scripts
./scripts/build-linux.sh
./scripts/build-windows.sh
./scripts/build-macos-native.sh
```

---

## Code Style Guidelines

### Imports
- Group imports logically: external crates → std → local modules
- External imports typically first, organized by crate
- Local crate imports use `use crate_name::module`
- Re-export common types in `lib.rs` with `pub use`

Example:
```rust
// External crates
use eframe::egui;
use anyhow::Result;
use serde::{Deserialize, Serialize};

// Standard library
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Local modules
use crate::ui::render_workspace;
use inote::db_state::DbINoteState;
```

### Naming Conventions
- **Functions/Variables**: `snake_case`
- **Structs/Enums**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`
- **Private fields**: `snake_case`

Example:
```rust
pub struct MyStruct {
    pub field_name: String,
    internal_value: usize,
}

pub const MAX_SIZE: usize = 100;

pub fn my_function(param: String) -> Result<()> {
    let local_var = param;
    Ok(())
}
```

### Error Handling
- Use `anyhow::Result<T>` in main application code for flexible error handling
- Define custom error types with `thiserror` in crates
- Use `?` operator for Result propagation
- Avoid `unwrap()` except in tests or when logically impossible to fail

Example:
```rust
// In application code
use anyhow::Result;

pub fn do_something() -> Result<()> {
    let data = read_file()?;  // Propagate errors with ?
    Ok(())
}

// In crate code
#[derive(Debug, thiserror::Error)]
pub enum CrateError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type CrateResult<T> = Result<T, CrateError>;
```

### Structs / Enums / Types
- Use `#[derive(Debug, Clone, Serialize, Deserialize)]` for data structures
- Implement `Default` trait for state structs
- Use `pub` re-exports for important types in `lib.rs`
- Add doc comments with `///` for public APIs

Example:
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
}

impl Default for Note {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: String::new(),
            content: String::new(),
        }
    }
}
```

### Async / Concurrency
- Use `tokio` as async runtime with full features
- Most UI code is synchronous (egui immediate mode pattern)
- Use `tokio::sync::mpsc` for inter-task communication
- Background tasks spawned with `tokio::spawn`

```rust
// Async function
pub async fn fetch_data() -> Result<String> {
    let response = reqwest::get("https://api.example.com").await?;
    Ok(response.text().await?)
}

// Spawn background task
tokio::spawn(async move {
    // Background work
});
```

### Module Organization
- Main app: `src/` with `mod.rs` files for subdirectories
- Crates: `crates/{crate_name}/src/lib.rs`
- Each crate exports types via `pub use`
- Module pattern: separate files for `state`, `ui`, `settings`

```
src/
├── main.rs
├── app.rs
├── mod.rs (if needed)
├── ui/
│   └── mod.rs
├── services/
│   └── mod.rs
└── utils/
    └── mod.rs

crates/my_crate/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── mod.rs
│   ├── state.rs
│   ├── ui.rs
│   └── tests/
```

### Testing
- Use `#[test]` attribute for unit tests
- Integration tests in `tests/` directory at crate root
- Unit tests in `src/tests/` within crates
- Test naming: `test_<functionality>`

Example:
```rust
#[test]
fn test_note_creation() {
    let note = Note::new("Test".to_string(), "Content".to_string());
    assert_eq!(note.title, "Test");
}

#[tokio::test]
async fn test_async_operation() {
    let result = fetch_data().await.unwrap();
    assert!(!result.is_empty());
}
```

### Documentation
- Module-level docs with `//!`
- Public APIs with `///`
- Chinese comments acceptable (project uses mixed Chinese/English)
- Include examples in doc comments when useful

```rust
//! This module handles note storage and retrieval.

/// Creates a new note with the given title and content.
///
/// # Example
/// ```
/// let note = Note::new("Title".to_string(), "Content".to_string());
/// assert_eq!(note.title, "Title");
/// ```
pub fn new(title: String, content: String) -> Self { ... }
```

### Formatting Rules (from crates/egui_term/rustfmt.toml)
- Max width: 80 characters
- Match block trailing comma: true
- Reorder imports: true

### Linting Rules (from crates/egui_term/clippy.toml)
- too-many-arguments-threshold: 20
- enum-variant-name-threshold: 10

---

## Workspace Structure

This is a Cargo workspace with the following crates:
- `inote`: Note-taking module with SQLite storage
- `isearch`: File search with tantivy indexing
- `aiAssist`: AI assistant with LLM integration
- `itools`: Tools and utilities module
- `iterminal`: Terminal emulator integration
- `ifile_editor`: File editor with ROPE data structure
- `egui_term`: Terminal widget (forked from iced_term)
- `egui_ltreeview`: Tree view component
- `egui_code_editor`: Code editor component
- `alacritty_terminal`: Terminal backend

### Dependencies
- egui 0.28.1 (DO NOT CHANGE - IME support issues in newer versions)
- eframe 0.28.1
- tokio 1.32.0 (full features)
- serde 1.0.188
- anyhow 1.0.75
- thiserror 1.0.49

### Platform Support
- Windows, macOS, Linux
- Cross-compilation via vcpkg or Docker
- vcpkg for native dependencies (openssl, sqlite3, libpng, etc.)
