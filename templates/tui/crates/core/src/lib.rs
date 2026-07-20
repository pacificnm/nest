//! {{display_title}} core library.
//!
//! Domain logic and shared services live here. The TUI crate in `crates/tui`
//! is a thin host adapter that delegates to this library.

/// Returns a greeting for the given name.
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}
