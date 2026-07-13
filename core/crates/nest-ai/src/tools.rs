//! Tool definitions and model-requested invocations.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Provider-agnostic tool schema passed to the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Function name exposed to the model.
    pub name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// JSON Schema object describing arguments.
    pub parameters: Value,
}

impl ToolDefinition {
    /// Creates a tool definition.
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}

/// One function invocation requested by the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Stable call id when provided by the backend.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// Function name to invoke.
    pub name: String,
    /// Parsed JSON arguments.
    #[serde(default)]
    pub arguments: Value,
}

impl ToolCall {
    /// Creates a tool call with object arguments.
    pub fn new(name: impl Into<String>, arguments: Value) -> Self {
        Self {
            id: String::new(),
            name: name.into(),
            arguments,
        }
    }
}

/// Merges incremental tool-call fragments from streaming responses.
pub fn merge_tool_calls(into: &mut Vec<ToolCall>, delta: &[ToolCall]) {
    for call in delta {
        if let Some(existing) = into.iter_mut().find(|item| item.name == call.name) {
            if !call.id.is_empty() {
                existing.id.clone_from(&call.id);
            }
            merge_arguments(&mut existing.arguments, &call.arguments);
        } else {
            into.push(call.clone());
        }
    }
}

fn merge_arguments(into: &mut Value, delta: &Value) {
    if delta.is_null() {
        return;
    }
    match (into, delta) {
        (Value::Object(existing), Value::Object(additions)) => {
            for (key, value) in additions {
                existing.insert(key.clone(), value.clone());
            }
        }
        (slot @ &mut Value::Null, delta) => {
            *slot = delta.clone();
        }
        (existing, delta) => {
            *existing = delta.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merge_tool_calls_combines_arguments() {
        let mut calls = vec![ToolCall::new("search", json!({"query": "nest"}))];
        merge_tool_calls(&mut calls, &[ToolCall::new("search", json!({"limit": 3}))]);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].arguments, json!({"query": "nest", "limit": 3}));
    }
}
