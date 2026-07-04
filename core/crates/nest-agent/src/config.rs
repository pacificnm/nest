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
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_steps: 10,
            tool_timeout: Duration::from_secs(60),
            auto_run_policy: AutoRunPolicy::ReadOnlyOnly,
        }
    }
}

impl AgentConfig {
    /// Creates config with the given step cap.
    pub fn with_max_steps(mut self, max_steps: u32) -> Self {
        self.max_steps = max_steps;
        self
    }
}
