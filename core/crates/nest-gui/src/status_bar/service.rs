//! Status bar message bus.

use std::sync::{Arc, Mutex};

use super::config::StatusBarConfig;
use super::kind::StatusKind;

/// Right-aligned status bar content (e.g. server connection).
#[derive(Debug, Clone)]
pub struct StatusBarRight {
    /// Label text (e.g. `Server: http://host:3000`).
    pub label: String,
    /// `true` = connected (green dot), `false` = disconnected (red dot).
    pub connected: bool,
    /// When false, only the label is shown (no connection dot).
    pub show_connection_dot: bool,
}

/// Current status bar content.
#[derive(Debug, Clone)]
pub struct StatusBarState {
    /// User-visible message on the left (empty hides the left section).
    pub message: String,
    /// Whether to show a spinner beside the message.
    pub busy: bool,
    /// Message styling.
    pub kind: StatusKind,
    /// Optional right-aligned section.
    pub right: Option<StatusBarRight>,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            message: String::new(),
            busy: false,
            kind: StatusKind::Info,
            right: None,
        }
    }
}

impl StatusBarRight {
    /// Label-only right section without a connection dot.
    pub fn text(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            connected: true,
            show_connection_dot: false,
        }
    }
}

impl StatusBarState {
    /// Whether the bar should be visible this frame.
    pub fn is_visible(&self) -> bool {
        !self.message.is_empty() || self.right.is_some()
    }
}

/// Application-wide status bar (register via [`super::StatusBarModule`]).
#[derive(Clone)]
pub struct StatusBarService {
    inner: Arc<Mutex<StatusBarInner>>,
}

struct StatusBarInner {
    config: StatusBarConfig,
    state: StatusBarState,
}

impl StatusBarService {
    /// Creates a service with the given defaults.
    pub fn new(config: StatusBarConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(StatusBarInner {
                config,
                state: StatusBarState::default(),
            })),
        }
    }

    /// Current configuration.
    pub fn config(&self) -> StatusBarConfig {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .config
            .clone()
    }

    /// Bar height in points.
    pub fn height(&self) -> f32 {
        self.config().height
    }

    /// Current state for rendering.
    pub fn state(&self) -> StatusBarState {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .state
            .clone()
    }

    /// Sets a neutral message and clears the busy spinner.
    pub fn set(&self, message: impl Into<String>) {
        self.update(message, StatusKind::Info, false);
    }

    /// Sets a message and shows the busy spinner.
    pub fn loading(&self, message: impl Into<String>) {
        self.update(message, StatusKind::Loading, true);
    }

    /// Sets a success-style message (e.g. after load or refresh).
    pub fn loaded(&self, message: impl Into<String>) {
        self.update(message, StatusKind::Success, false);
    }

    /// Sets a success-style refresh message.
    pub fn refreshed(&self, message: impl Into<String>) {
        self.loaded(message);
    }

    /// Sets an error message.
    pub fn error(&self, message: impl Into<String>) {
        self.update(message, StatusKind::Error, false);
    }

    /// Sets the default idle message.
    pub fn ready(&self) {
        self.set("Ready");
    }

    /// Clears the left message and busy state (right section is unchanged).
    pub fn clear(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.state.message.clear();
            inner.state.busy = false;
            inner.state.kind = StatusKind::Info;
        }
    }

    /// Sets the right section label and connection dot color.
    pub fn set_right(&self, label: impl Into<String>, connected: bool) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.state.right = Some(StatusBarRight {
                label: label.into(),
                connected,
                show_connection_dot: true,
            });
        }
    }

    /// Sets a label-only right section (no connection dot).
    pub fn set_right_text(&self, label: impl Into<String>) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.state.right = Some(StatusBarRight::text(label));
        }
    }

    /// Sets `Server: {url}` on the right with a connection dot.
    pub fn set_server(&self, url: impl Into<String>, connected: bool) {
        self.set_right(format!("Server: {}", url.into()), connected);
    }

    /// Updates only the connection dot on the right section (if present).
    pub fn set_server_connected(&self, connected: bool) {
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(right) = inner.state.right.as_mut() {
                right.connected = connected;
            }
        }
    }

    /// Removes the right section.
    pub fn clear_right(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.state.right = None;
        }
    }

    fn update(&self, message: impl Into<String>, kind: StatusKind, busy: bool) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.state.message = message.into();
            inner.state.busy = busy;
            inner.state.kind = kind;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_and_loaded() {
        let service = StatusBarService::new(StatusBarConfig::default());
        service.loading("Fetching…");
        let state = service.state();
        assert!(state.busy);
        assert_eq!(state.message, "Fetching…");

        service.loaded("Done");
        let state = service.state();
        assert!(!state.busy);
        assert_eq!(state.kind, StatusKind::Success);
    }

    #[test]
    fn clear_hides_left_only() {
        let service = StatusBarService::new(StatusBarConfig::default());
        service.set("Hello");
        service.set_server("http://localhost", true);
        service.clear();
        let state = service.state();
        assert!(state.message.is_empty());
        assert!(state.right.is_some());
        assert!(state.is_visible());
    }

    #[test]
    fn server_connected_updates_dot() {
        let service = StatusBarService::new(StatusBarConfig::default());
        service.set_server("http://localhost", false);
        service.set_server_connected(true);
        assert!(service.state().right.unwrap().connected);
    }
}
