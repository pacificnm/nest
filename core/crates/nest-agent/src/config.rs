//! Agent loop configuration.

use std::time::Duration;

/// Policy for executing tool calls without user approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoRunPolicy {
    /// Auto-run read-only Nest memory search tools only.
    #[default]
    ReadOnlyOnly,
    /// Require explicit approval before any tool call (v1.1).
    Ask,
}

/// Limits and behavior for [`super::AgentLoop`].
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Maximum model ↔ tool iterations before failing.
    pub max_steps: u32,
    /// Per-tool MCP call timeout.
    pub tool_timeout: Duration,
    /// Which tools may run without approval.
    pub auto_run_policy: AutoRunPolicy,
    /// When true, `save_context_memory` may auto-run under [`AutoRunPolicy::ReadOnlyOnly`].
    pub allow_save_context: bool,
    /// When true, Nest file write/delete tools may auto-run under [`AutoRunPolicy::ReadOnlyOnly`].
    pub allow_file_writes: bool,
    /// When true, independent tool calls may run concurrently (different MCP servers).
    pub parallel_tool_calls: bool,
    /// Open workspace root shown to the model for file tool paths.
    pub workspace_root: Option<std::path::PathBuf>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_steps: 10,
            tool_timeout: Duration::from_secs(60),
            auto_run_policy: AutoRunPolicy::ReadOnlyOnly,
            allow_save_context: false,
            allow_file_writes: false,
            parallel_tool_calls: true,
            workspace_root: None,
        }
    }
}

impl AgentConfig {
    /// Creates config with the given step cap.
    pub fn with_max_steps(mut self, max_steps: u32) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// Enables or disables auto-run for `save_context_memory`.
    pub fn with_allow_save_context(mut self, allow: bool) -> Self {
        self.allow_save_context = allow;
        self
    }

    /// Enables or disables auto-run for Nest file write/delete tools.
    pub fn with_allow_file_writes(mut self, allow: bool) -> Self {
        self.allow_file_writes = allow;
        self
    }

    /// Sets the workspace root injected into the agent system prompt for file tools.
    pub fn with_workspace_root(mut self, root: impl Into<std::path::PathBuf>) -> Self {
        self.workspace_root = Some(root.into());
        self
    }

    /// Sets per-tool call timeout (for example `cargo check`).
    pub fn with_tool_timeout(mut self, timeout: Duration) -> Self {
        self.tool_timeout = timeout;
        self
    }
}
