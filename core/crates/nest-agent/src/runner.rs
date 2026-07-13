//! Multi-step agent loop.

use std::time::Instant;

use futures_util::StreamExt;
use nest_ai::{merge_tool_calls, AiService, ChatMessage, ChatRole, CompletionRequest, ToolCall};
use tokio::sync::mpsc;
use tracing::debug;

use crate::cancel::CancelToken;
use crate::config::AgentConfig;
use crate::event::AgentEvent;
use crate::policy::{is_file_tool, may_auto_run};
use crate::registry::ToolRegistry;
use crate::tools::{SharedMcpHub, ToolSource};
use crate::validation::{parse_tool_calls_from_content, validate_tool_arguments};
use crate::{ai_to_nest, NestResult};

/// Orchestrates LLM completions with MCP tool execution.
pub struct AgentLoop {
    ai: AiService,
    config: AgentConfig,
}

impl AgentLoop {
    /// Creates an agent loop with AI service and config.
    pub fn new(ai: AiService, config: AgentConfig) -> Self {
        Self { ai, config }
    }

    /// Runs the agent until the model produces a final answer, fails, or hits limits.
    pub async fn run(
        &self,
        tools: &mut dyn ToolSource,
        mut messages: Vec<ChatMessage>,
        model: Option<String>,
        tx: mpsc::Sender<AgentEvent>,
        cancel: CancelToken,
    ) -> NestResult<()> {
        let agent_tools = tools.list_tools().await?;
        let attached_files = has_attached_files(&messages);
        let search_with_attachments =
            attached_files && latest_user_message_requests_external_search(&messages);
        let edit_with_attachments =
            attached_files && latest_user_message_requests_file_edit(&messages);
        let persist_requested = latest_user_message_requests_file_persist(&messages);
        let exposed_tools: Vec<_> = agent_tools
            .into_iter()
            .filter(|tool| {
                if !may_auto_run(&self.config, tool) {
                    return false;
                }
                if attached_files && !search_with_attachments {
                    // Keep file tools when the user attached files — especially for edits.
                    return is_file_tool(&tool.server, &tool.name);
                }
                true
            })
            .collect();
        let registry = ToolRegistry::from_tools(exposed_tools);
        ensure_system_prompt(
            &mut messages,
            registry.tools(),
            attached_files,
            edit_with_attachments,
            persist_requested,
            &self.config,
        );

        for step in 1..=self.config.max_steps {
            if cancel.is_cancelled() {
                let _ = tx
                    .send(AgentEvent::Failed("agent run cancelled".into()))
                    .await;
                return Ok(());
            }

            let _ = tx.send(AgentEvent::StepStarted { step }).await;

            let request = CompletionRequest {
                model: model.clone(),
                messages: messages.clone(),
                format: None,
                tools: registry.definitions(),
            };

            let (content, tool_calls, metrics) = self.complete_step(&request, &tx, &cancel).await?;

            if tool_calls.is_empty() {
                let _ = tx.send(AgentEvent::Finished { metrics, content }).await;
                return Ok(());
            }

            let assistant = if content.is_empty() {
                ChatMessage::assistant_tool_calls(tool_calls.clone())
            } else {
                let mut message = ChatMessage::assistant(&content);
                message.tool_calls = Some(tool_calls.clone());
                message
            };
            messages.push(assistant);

            let tool_results = self
                .execute_tool_calls(tools, &registry, tool_calls, &tx, &cancel)
                .await;
            for (call, result) in tool_results {
                messages.push(ChatMessage::tool_result(&call.name, result));
            }
        }

        let _ = tx
            .send(AgentEvent::Failed(format!(
                "agent exceeded max steps ({})",
                self.config.max_steps
            )))
            .await;
        Ok(())
    }

    async fn complete_step(
        &self,
        request: &CompletionRequest,
        tx: &mpsc::Sender<AgentEvent>,
        cancel: &CancelToken,
    ) -> NestResult<(String, Vec<ToolCall>, Option<nest_ai::CompletionMetrics>)> {
        // Ollama tool calling is more reliable with a single non-stream response.
        if !request.tools.is_empty() {
            if cancel.is_cancelled() {
                return Err(
                    nest_error::NestError::network("agent run cancelled").with_module("nest-agent")
                );
            }

            let response = self
                .ai
                .complete(request.clone())
                .await
                .map_err(ai_to_nest)?;

            let mut content = response.content;
            let mut tool_calls = response.tool_calls;
            if tool_calls.is_empty() {
                if let Some(parsed) = parse_tool_calls_from_content(&content) {
                    tool_calls = parsed;
                    content.clear();
                }
            }

            if !content.is_empty() {
                let _ = tx.send(AgentEvent::TextDelta(content.clone())).await;
            }

            return Ok((content, tool_calls, response.metrics));
        }

        if let Ok(mut stream) = self.ai.stream_complete(request.clone()).await {
            let mut content = String::new();
            let mut tool_calls = Vec::new();
            let mut metrics = None;

            while let Some(chunk) = stream.next().await {
                if cancel.is_cancelled() {
                    return Err(nest_error::NestError::network("agent run cancelled")
                        .with_module("nest-agent"));
                }
                match chunk {
                    Ok(chunk) => {
                        if !chunk.content_delta.is_empty() {
                            content.push_str(&chunk.content_delta);
                            let _ = tx.send(AgentEvent::TextDelta(chunk.content_delta)).await;
                        }
                        if !chunk.tool_calls.is_empty() {
                            merge_tool_calls(&mut tool_calls, &chunk.tool_calls);
                        }
                        if chunk.metrics.is_some() {
                            metrics = chunk.metrics;
                        }
                        if chunk.done {
                            break;
                        }
                    }
                    Err(error) => return Err(ai_to_nest(error)),
                }
            }

            if tool_calls.is_empty() {
                if let Some(parsed) = parse_tool_calls_from_content(&content) {
                    tool_calls = parsed;
                    content.clear();
                }
            }

            return Ok((content, tool_calls, metrics));
        }

        let response = self
            .ai
            .complete(request.clone())
            .await
            .map_err(ai_to_nest)?;

        let mut content = response.content;
        let mut tool_calls = response.tool_calls;
        if tool_calls.is_empty() {
            if let Some(parsed) = parse_tool_calls_from_content(&content) {
                tool_calls = parsed;
                content.clear();
            }
        }

        if !content.is_empty() {
            let _ = tx.send(AgentEvent::TextDelta(content.clone())).await;
        }

        Ok((content, tool_calls, response.metrics))
    }

    async fn execute_tool_calls(
        &self,
        tools: &mut dyn ToolSource,
        registry: &ToolRegistry,
        tool_calls: Vec<ToolCall>,
        tx: &mpsc::Sender<AgentEvent>,
        cancel: &CancelToken,
    ) -> Vec<(ToolCall, String)> {
        if cancel.is_cancelled() {
            return Vec::new();
        }

        if self.config.parallel_tool_calls && tool_calls.len() > 1 {
            if let Some(shared) = tools.shared_mcp() {
                return self
                    .run_tool_calls_parallel(shared, registry, tool_calls, tx.clone())
                    .await;
            }
        }

        let mut results = Vec::new();
        for call in tool_calls {
            if cancel.is_cancelled() {
                break;
            }
            let result = match self.run_tool_call(tools, registry, &call, tx).await {
                Ok(text) => text,
                Err(error) => format!("Tool error: {error}"),
            };
            results.push((call, result));
        }
        results
    }

    async fn run_tool_calls_parallel(
        &self,
        hub: SharedMcpHub,
        registry: &ToolRegistry,
        tool_calls: Vec<ToolCall>,
        tx: mpsc::Sender<AgentEvent>,
    ) -> Vec<(ToolCall, String)> {
        let config = self.config.clone();
        let registry = registry.clone();
        let mut handles = Vec::new();

        for call in tool_calls {
            let hub = hub.clone();
            let registry = registry.clone();
            let tx = tx.clone();
            let config = config.clone();
            handles.push(tokio::spawn(async move {
                parallel_tool_call(&config, hub, &registry, call, tx).await
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(pair) = handle.await {
                results.push(pair);
            }
        }
        results
    }

    async fn run_tool_call(
        &self,
        tools: &mut dyn ToolSource,
        registry: &ToolRegistry,
        call: &ToolCall,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> NestResult<String> {
        let _ = tx
            .send(AgentEvent::ToolCallStarted {
                tool: call.name.clone(),
                arguments: call.arguments.clone(),
            })
            .await;

        let Some(_agent_tool) = registry.agent_tool(&call.name) else {
            let error = format!("unknown tool: {}", call.name);
            let _ = tx
                .send(AgentEvent::ToolCallFailed {
                    tool: call.name.clone(),
                    error: error.clone(),
                })
                .await;
            return Err(nest_error::NestError::network(error).with_module("nest-agent"));
        };

        if let Err(error) = validate_tool_arguments(&call.arguments) {
            let _ = tx
                .send(AgentEvent::ToolCallFailed {
                    tool: call.name.clone(),
                    error: error.clone(),
                })
                .await;
            return Err(nest_error::NestError::network(error).with_module("nest-agent"));
        }

        let qualified = registry
            .qualified_name(&call.name)
            .expect("tool exists in registry")
            .to_string();

        debug!(tool = %call.name, qualified = %qualified, "executing agent tool");
        let started = Instant::now();
        let result = match tokio::time::timeout(
            self.config.tool_timeout,
            tools.call_tool(&qualified, call.arguments.clone()),
        )
        .await
        {
            Ok(Ok(text)) => text,
            Ok(Err(error)) => {
                let message = error.to_string();
                let _ = tx
                    .send(AgentEvent::ToolCallFailed {
                        tool: call.name.clone(),
                        error: message.clone(),
                    })
                    .await;
                return Err(error);
            }
            Err(_) => {
                let message = format!("tool {} timed out", call.name);
                let _ = tx
                    .send(AgentEvent::ToolCallFailed {
                        tool: call.name.clone(),
                        error: message.clone(),
                    })
                    .await;
                return Err(nest_error::NestError::network(message).with_module("nest-agent"));
            }
        };

        let _ = tx
            .send(AgentEvent::ToolCallFinished {
                tool: call.name.clone(),
                result: truncate_preview(&result),
                duration: started.elapsed(),
            })
            .await;

        Ok(result)
    }
}

async fn parallel_tool_call(
    config: &AgentConfig,
    hub: SharedMcpHub,
    registry: &ToolRegistry,
    call: ToolCall,
    tx: mpsc::Sender<AgentEvent>,
) -> (ToolCall, String) {
    let _ = tx
        .send(AgentEvent::ToolCallStarted {
            tool: call.name.clone(),
            arguments: call.arguments.clone(),
        })
        .await;

    let Some(_agent_tool) = registry.agent_tool(&call.name) else {
        let error = format!("unknown tool: {}", call.name);
        let _ = tx
            .send(AgentEvent::ToolCallFailed {
                tool: call.name.clone(),
                error: error.clone(),
            })
            .await;
        return (call, format!("Tool error: {error}"));
    };

    if let Err(error) = validate_tool_arguments(&call.arguments) {
        let _ = tx
            .send(AgentEvent::ToolCallFailed {
                tool: call.name.clone(),
                error: error.clone(),
            })
            .await;
        return (call, format!("Tool error: {error}"));
    }

    let qualified = registry
        .qualified_name(&call.name)
        .expect("tool exists in registry")
        .to_string();

    debug!(tool = %call.name, qualified = %qualified, "executing agent tool (parallel)");
    let started = Instant::now();
    let mut tools = hub;
    let outcome = match tokio::time::timeout(
        config.tool_timeout,
        tools.call_tool(&qualified, call.arguments.clone()),
    )
    .await
    {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(error)) => Err(error.to_string()),
        Err(_) => Err(format!("tool {} timed out", call.name)),
    };

    match outcome {
        Ok(text) => {
            let _ = tx
                .send(AgentEvent::ToolCallFinished {
                    tool: call.name.clone(),
                    result: truncate_preview(&text),
                    duration: started.elapsed(),
                })
                .await;
            (call, text)
        }
        Err(error) => {
            let _ = tx
                .send(AgentEvent::ToolCallFailed {
                    tool: call.name.clone(),
                    error: error.clone(),
                })
                .await;
            (call, format!("Tool error: {error}"))
        }
    }
}

fn has_attached_files(messages: &[ChatMessage]) -> bool {
    messages
        .iter()
        .any(|message| message.content.contains("<file path="))
}

fn latest_user_message_requests_external_search(messages: &[ChatMessage]) -> bool {
    let Some(message) = messages
        .iter()
        .rev()
        .find(|message| message.role == ChatRole::User)
    else {
        return false;
    };

    let lower = message.content.to_ascii_lowercase();
    [
        "search project",
        "search memory",
        "project memory",
        "knowledge base",
        "search_knowledge",
        "search_project",
        "look up in the repo",
        "find in the repo",
        "find in project",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
}

fn latest_user_message_requests_file_edit(messages: &[ChatMessage]) -> bool {
    let Some(message) = messages
        .iter()
        .rev()
        .find(|message| message.role == ChatRole::User)
    else {
        return false;
    };

    let lower = message.content.to_ascii_lowercase();
    [
        "edit ",
        "update ",
        "change ",
        "modify ",
        "fix ",
        "write ",
        "rewrite ",
        "refactor ",
        "add ",
        "remove ",
        "delete ",
        "create file",
        "save ",
        "patch ",
        "implement ",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
}

fn latest_user_message_requests_file_persist(messages: &[ChatMessage]) -> bool {
    let Some(message) = messages
        .iter()
        .rev()
        .find(|message| message.role == ChatRole::User)
    else {
        return false;
    };

    let lower = message.content.to_ascii_lowercase();
    [
        "save ",
        "save the",
        "write_file",
        "write file",
        "create file",
        "create the file",
        "document ",
        "document the",
        "markdown",
        "write to ",
        "persist ",
        "docs/",
        "docs/agent",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
        || latest_user_message_requests_file_edit(messages)
}

fn ensure_system_prompt(
    messages: &mut Vec<ChatMessage>,
    tools: &[crate::tool::AgentTool],
    attached_files: bool,
    edit_with_attachments: bool,
    persist_requested: bool,
    config: &AgentConfig,
) {
    if messages
        .first()
        .is_some_and(|message| message.role == ChatRole::System)
    {
        return;
    }

    let tool_count = tools.len();
    let has_file_tools = tools
        .iter()
        .any(|tool| is_file_tool(&tool.server, &tool.name));
    let has_file_write_tools = tools.iter().any(|tool| {
        is_file_tool(&tool.server, &tool.name)
            && tool.name != "read_file"
            && tool.name != "list_directory"
    });

    let workspace = workspace_context(config.workspace_root.as_deref());
    let file_guidance = if has_file_write_tools {
        file_tool_guidance()
    } else {
        String::new()
    };
    let task_hint = task_hint_from_messages(messages);

    let prompt = if persist_requested && has_file_write_tools {
        format!(
            "You are Kiwi, a coding assistant with file write tools. The user asked you to create \
             or save files on disk. You MUST: (1) read every source file the user named with \
             read_file; (2) call write_file for new paths or update_file for existing paths; \
             (3) only reply after the write tool succeeds. Never claim a file was saved unless \
             a write tool returned success in this run.{file_guidance}{task_hint}{workspace}"
        )
    } else if attached_files && has_file_write_tools && edit_with_attachments {
        format!(
            "You are Kiwi, a coding assistant with file editing tools. The user's message includes \
             attached file contents in <file path=\"...\"> blocks. When asked to edit, fix, or update \
             those files, you MUST persist changes using write_file or update_file with the exact path \
             from the file tag — do not only describe changes in text. Use structured tool calls with \
             concrete argument values. After editing, briefly confirm what changed.{file_guidance}{workspace}"
        )
    } else if attached_files && has_file_tools {
        format!(
            "You are Kiwi, a coding assistant with access to {tool_count} file tools. The user's \
             message includes attached file contents in <file path=\"...\"> blocks — read those \
             first. Use read_file, write_file, update_file, list_directory, create_directory, or \
             delete_path when you need to inspect or persist workspace changes. Use the path from \
             the file tag or a project-relative path. Use search_files with a query to locate paths. \
             Use structured tool calls with concrete argument values. When you have enough context, \
             reply with a clear final answer.{file_guidance}{workspace}"
        )
    } else if attached_files && tool_count == 0 {
        "You are Kiwi, a coding assistant. The user's message includes attached file contents \
         inside <file path=\"...\">...</file> blocks. Read and answer from those attachments \
         directly. Do not claim you cannot access the files."
            .to_string()
    } else if has_file_write_tools {
        format!(
            "You are Kiwi, a coding assistant with access to {tool_count} tools including file \
             read/write/update/delete. When the user asks you to change code or files, use \
             write_file or update_file to persist edits — do not only show code in the reply. \
             Use search_files with a query to locate paths, then read_file before editing. Paths \
             are relative to the workspace root below — never use .. to escape it. For update_file: \
             call read_file first, then pass old_string copied exactly from the file (whitespace \
             included). To insert at the start of a file, use empty old_string. Use structured tool \
             calls with concrete argument values, never JSON Schema fragments. When done, reply with \
             a brief summary.{file_guidance}{task_hint}{workspace}"
        )
    } else {
        format!(
            "You are Kiwi, a coding assistant with access to {tool_count} tools. Use structured \
             tool calls with concrete argument values (strings, numbers), never JSON Schema \
             fragments. When you have enough context, reply with a clear final answer in plain text."
        )
    };

    messages.insert(0, ChatMessage::system(prompt));
}

fn file_tool_guidance() -> String {
    "\n\nFile tools — pick the right one:\n\
     • search_files(query): locate paths when unsure\n\
     • search_code(query): find lines in source files by content\n\
     • read_file: always before editing an existing file\n\
     • write_file: create a NEW file (path must not exist yet) or replace a file after read_file\n\
     • update_file: partial edits to an EXISTING file (read_file first; copy old_string exactly)\n\
     • create_directory: create a folder tree before writing files inside it\n\
     • delete_path: remove a file or directory\n\
     • cargo_check: run after Rust edits to verify compilation\n\
     Parent directories are created automatically on write_file."
        .to_string()
}

fn task_hint_from_messages(messages: &[ChatMessage]) -> String {
    let Some(text) = messages
        .iter()
        .rev()
        .find(|message| message.role == ChatRole::User)
        .map(|message| message.content.as_str())
    else {
        return String::new();
    };

    let lower = text.to_ascii_lowercase();
    if lower.contains("document") || lower.contains("markdown") || lower.contains("docs/") {
        let output = text
            .split_whitespace()
            .map(|token| token.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.'))
            .find(|token| token.starts_with("docs/") && token.ends_with(".md"))
            .unwrap_or("docs/agent/agent.md");
        return format!(
            "\n\nTask hint: the user wants markdown documentation written to `{output}` (or another \
             path under docs/). Read the named source files; do not overwrite them — write the doc \
             to the output path."
        );
    }

    String::new()
}

fn workspace_context(workspace_root: Option<&std::path::Path>) -> String {
    let Some(root) = workspace_root else {
        return String::new();
    };
    format!(
        "\n\nWorkspace root: {}. All file tool paths are relative to this directory (not the Kiwi \
         app folder). Do not use .. segments. Call search_files with query before read/write when \
         you are unsure of the path.",
        root.display()
    )
}

fn truncate_preview(text: &str) -> String {
    const LIMIT: usize = 240;
    if text.len() <= LIMIT {
        text.to_string()
    } else {
        format!("{}…", &text[..LIMIT])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{AgentTool, ToolOrigin};
    use async_trait::async_trait;
    use nest_ai::{AiProvider, AiResult, CompletionResponse};
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    struct MockTools {
        tools: Vec<AgentTool>,
        calls: Vec<(String, serde_json::Value)>,
    }

    #[async_trait]
    impl ToolSource for MockTools {
        async fn list_tools(&mut self) -> NestResult<Vec<AgentTool>> {
            Ok(self.tools.clone())
        }

        async fn call_tool(
            &mut self,
            qualified_name: &str,
            arguments: serde_json::Value,
        ) -> NestResult<String> {
            self.calls.push((qualified_name.to_string(), arguments));
            Ok("memory hit".into())
        }
    }

    struct MockAi {
        responses: Mutex<Vec<CompletionResponse>>,
    }

    #[async_trait]
    impl AiProvider for MockAi {
        fn provider_id(&self) -> &'static str {
            "mock"
        }

        async fn complete(&self, _request: CompletionRequest) -> AiResult<CompletionResponse> {
            let mut responses = self.responses.lock().unwrap();
            if responses.is_empty() {
                return Ok(CompletionResponse {
                    model: "mock".into(),
                    content: "fallback".into(),
                    done: true,
                    tool_calls: vec![],
                    metrics: None,
                });
            }
            Ok(responses.remove(0))
        }
    }

    fn search_tool() -> AgentTool {
        AgentTool {
            origin: ToolOrigin::Mcp,
            server: "nest-memory".into(),
            name: "search_project_memory".into(),
            qualified_name: "nest-memory/search_project_memory".into(),
            description: "Search".into(),
            input_schema: json!({"type": "object"}),
        }
    }

    fn save_tool() -> AgentTool {
        AgentTool {
            origin: ToolOrigin::Mcp,
            server: "nest-context-memory".into(),
            name: "save_context_memory".into(),
            qualified_name: "nest-context-memory/save_context_memory".into(),
            description: "Save".into(),
            input_schema: json!({"type": "object"}),
        }
    }

    #[tokio::test]
    async fn agent_runs_tool_then_finishes() {
        let ai = AiService::new(Arc::new(MockAi {
            responses: Mutex::new(vec![
                CompletionResponse {
                    model: "mock".into(),
                    content: String::new(),
                    done: true,
                    tool_calls: vec![ToolCall::new(
                        "nest_memory__search_project_memory",
                        json!({"query": "nest-core"}),
                    )],
                    metrics: None,
                },
                CompletionResponse {
                    model: "mock".into(),
                    content: "nest-core is the module system".into(),
                    done: true,
                    tool_calls: vec![],
                    metrics: None,
                },
            ]),
        }));

        let (tx, mut rx) = mpsc::channel(32);
        let mut tools = MockTools {
            tools: vec![search_tool()],
            calls: vec![],
        };
        AgentLoop::new(ai, AgentConfig::default())
            .run(
                &mut tools,
                vec![ChatMessage::user("What is nest-core?")],
                None,
                tx,
                CancelToken::new(),
            )
            .await
            .unwrap();

        let mut saw_tool = false;
        let mut finished = None;
        while let Ok(event) = rx.try_recv() {
            match event {
                AgentEvent::ToolCallFinished { .. } => saw_tool = true,
                AgentEvent::Finished { content, .. } => finished = Some(content),
                _ => {}
            }
        }

        assert!(saw_tool);
        assert_eq!(finished.as_deref(), Some("nest-core is the module system"));
        assert_eq!(tools.calls.len(), 1);
    }

    #[tokio::test]
    async fn agent_excludes_unapproved_tools_from_registry() {
        let ai = AiService::new(Arc::new(MockAi {
            responses: Mutex::new(vec![
                CompletionResponse {
                    model: "mock".into(),
                    content: String::new(),
                    done: true,
                    tool_calls: vec![ToolCall::new(
                        "nest_context_memory__save_context_memory",
                        json!({"content": "x"}),
                    )],
                    metrics: None,
                },
                CompletionResponse {
                    model: "mock".into(),
                    content: "done".into(),
                    done: true,
                    tool_calls: vec![],
                    metrics: None,
                },
            ]),
        }));

        let (tx, mut rx) = mpsc::channel(8);
        let mut tools = MockTools {
            tools: vec![search_tool(), save_tool()],
            calls: vec![],
        };

        AgentLoop::new(ai, AgentConfig::default())
            .run(
                &mut tools,
                vec![ChatMessage::user("save")],
                None,
                tx,
                CancelToken::new(),
            )
            .await
            .unwrap();

        let mut unknown_tool = false;
        let mut finished = false;
        while let Ok(event) = rx.try_recv() {
            match event {
                AgentEvent::ToolCallFailed { error, .. } if error.contains("unknown tool") => {
                    unknown_tool = true;
                }
                AgentEvent::Finished { .. } => finished = true,
                _ => {}
            }
        }
        assert!(unknown_tool);
        assert!(finished);
        assert!(tools.calls.is_empty());
    }

    #[tokio::test]
    async fn agent_recovers_from_invalid_schema_arguments() {
        let ai = AiService::new(Arc::new(MockAi {
            responses: Mutex::new(vec![
                CompletionResponse {
                    model: "mock".into(),
                    content: String::new(),
                    done: true,
                    tool_calls: vec![ToolCall::new(
                        "nest_memory__search_project_memory",
                        json!({"query": {"type": "string"}}),
                    )],
                    metrics: None,
                },
                CompletionResponse {
                    model: "mock".into(),
                    content: "answer".into(),
                    done: true,
                    tool_calls: vec![],
                    metrics: None,
                },
            ]),
        }));

        let (tx, mut rx) = mpsc::channel(16);
        let mut tools = MockTools {
            tools: vec![search_tool()],
            calls: vec![],
        };
        AgentLoop::new(ai, AgentConfig::default())
            .run(
                &mut tools,
                vec![ChatMessage::user("search")],
                None,
                tx,
                CancelToken::new(),
            )
            .await
            .unwrap();

        let mut schema_error = false;
        let mut finished = false;
        while let Ok(event) = rx.try_recv() {
            match event {
                AgentEvent::ToolCallFailed { error, .. } if error.contains("JSON Schema") => {
                    schema_error = true;
                }
                AgentEvent::Finished { .. } => finished = true,
                _ => {}
            }
        }
        assert!(schema_error);
        assert!(finished);
        assert!(tools.calls.is_empty());
    }

    #[tokio::test]
    async fn agent_parses_tool_call_from_assistant_content() {
        let ai = AiService::new(Arc::new(MockAi {
            responses: Mutex::new(vec![
                CompletionResponse {
                    model: "mock".into(),
                    content: r#"{"name":"nest_memory__search_project_memory","arguments":{"query":"nest"}}"#.into(),
                    done: true,
                    tool_calls: vec![],
                    metrics: None,
                },
                CompletionResponse {
                    model: "mock".into(),
                    content: "found it".into(),
                    done: true,
                    tool_calls: vec![],
                    metrics: None,
                },
            ]),
        }));

        let (tx, mut rx) = mpsc::channel(16);
        let mut tools = MockTools {
            tools: vec![search_tool()],
            calls: vec![],
        };
        AgentLoop::new(ai, AgentConfig::default())
            .run(
                &mut tools,
                vec![ChatMessage::user("search nest")],
                None,
                tx,
                CancelToken::new(),
            )
            .await
            .unwrap();

        let mut finished = None;
        while let Ok(event) = rx.try_recv() {
            if let AgentEvent::Finished { content, .. } = event {
                finished = Some(content);
            }
        }
        assert_eq!(tools.calls.len(), 1);
        assert_eq!(finished.as_deref(), Some("found it"));
    }

    #[tokio::test]
    async fn agent_keeps_file_tools_when_file_attached_in_message() {
        fn write_tool() -> AgentTool {
            AgentTool {
                origin: ToolOrigin::Native,
                server: "nest-file".into(),
                name: "write_file".into(),
                qualified_name: "nest-file/write_file".into(),
                description: "Write".into(),
                input_schema: json!({"type": "object"}),
            }
        }

        let ai = AiService::new(Arc::new(MockAi {
            responses: Mutex::new(vec![
                CompletionResponse {
                    model: "mock".into(),
                    content: String::new(),
                    done: true,
                    tool_calls: vec![ToolCall::new(
                        "nest_file__write_file",
                        json!({"path": "foo.rs", "content": "fn main() {}"}),
                    )],
                    metrics: None,
                },
                CompletionResponse {
                    model: "mock".into(),
                    content: "updated foo.rs".into(),
                    done: true,
                    tool_calls: vec![],
                    metrics: None,
                },
            ]),
        }));

        let (tx, mut rx) = mpsc::channel(8);
        let mut tools = MockTools {
            tools: vec![search_tool(), write_tool()],
            calls: vec![],
        };
        let message = ChatMessage::user("Edit this file\n<file path=\"foo.rs\">\nold\n</file>");
        AgentLoop::new(ai, AgentConfig::default().with_allow_file_writes(true))
            .run(&mut tools, vec![message], None, tx, CancelToken::new())
            .await
            .unwrap();

        let mut finished = None;
        while let Ok(event) = rx.try_recv() {
            if let AgentEvent::Finished { content, .. } = event {
                finished = Some(content);
            }
        }
        assert_eq!(tools.calls.len(), 1);
        assert_eq!(tools.calls[0].0, "nest-file/write_file");
        assert_eq!(finished.as_deref(), Some("updated foo.rs"));
    }

    #[tokio::test]
    async fn agent_skips_tools_when_file_attached_in_message() {
        let ai = AiService::new(Arc::new(MockAi {
            responses: Mutex::new(vec![CompletionResponse {
                model: "mock".into(),
                content: "This README describes the Nest framework.".into(),
                done: true,
                tool_calls: vec![],
                metrics: None,
            }]),
        }));

        let (tx, mut rx) = mpsc::channel(8);
        let mut tools = MockTools {
            tools: vec![search_tool()],
            calls: vec![],
        };
        let message = ChatMessage::user(
            "Summarize this readme\n<file path=\"README.md\">\n# Nest\nA framework.\n</file>",
        );
        AgentLoop::new(ai, AgentConfig::default())
            .run(&mut tools, vec![message], None, tx, CancelToken::new())
            .await
            .unwrap();

        let mut finished = None;
        while let Ok(event) = rx.try_recv() {
            if let AgentEvent::Finished { content, .. } = event {
                finished = Some(content);
            }
        }
        assert!(tools.calls.is_empty());
        assert_eq!(
            finished.as_deref(),
            Some("This README describes the Nest framework.")
        );
    }

    #[tokio::test]
    async fn agent_respects_max_steps() {
        let ai = AiService::new(Arc::new(MockAi {
            responses: Mutex::new(vec![
                CompletionResponse {
                    model: "mock".into(),
                    content: String::new(),
                    done: true,
                    tool_calls: vec![ToolCall::new(
                        "nest_memory__search_project_memory",
                        json!({"query": "a"}),
                    )],
                    metrics: None,
                },
                CompletionResponse {
                    model: "mock".into(),
                    content: String::new(),
                    done: true,
                    tool_calls: vec![ToolCall::new(
                        "nest_memory__search_project_memory",
                        json!({"query": "b"}),
                    )],
                    metrics: None,
                },
            ]),
        }));

        let (tx, mut rx) = mpsc::channel(16);
        let mut tools = MockTools {
            tools: vec![search_tool()],
            calls: vec![],
        };
        AgentLoop::new(ai, AgentConfig::default().with_max_steps(1))
            .run(
                &mut tools,
                vec![ChatMessage::user("loop")],
                None,
                tx,
                CancelToken::new(),
            )
            .await
            .unwrap();

        let mut failed = false;
        while let Ok(event) = rx.try_recv() {
            if matches!(event, AgentEvent::Failed(_)) {
                failed = true;
            }
        }
        assert!(failed);
    }
}
