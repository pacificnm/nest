//! Multi-step agent loop.

use std::time::Instant;

use nest_ai::{AiService, ChatMessage, CompletionRequest, ToolCall};
use tokio::sync::mpsc;
use tracing::debug;

use crate::cancel::CancelToken;
use crate::config::AgentConfig;
use crate::event::AgentEvent;
use crate::policy::may_auto_run;
use crate::registry::ToolRegistry;
use crate::tools::ToolSource;
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
        let mcp_tools = tools.list_tools().await?;
        let registry = ToolRegistry::from_mcp_tools(mcp_tools);
        ensure_system_prompt(&mut messages, registry.tools().len());

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

            let response = self.ai.complete(request).await.map_err(ai_to_nest)?;

            if !response.content.is_empty() {
                let _ = tx
                    .send(AgentEvent::TextDelta(response.content.clone()))
                    .await;
            }

            if response.tool_calls.is_empty() {
                let _ = tx
                    .send(AgentEvent::Finished {
                        metrics: None,
                        content: response.content,
                    })
                    .await;
                return Ok(());
            }

            let assistant = if response.content.is_empty() {
                ChatMessage::assistant_tool_calls(response.tool_calls.clone())
            } else {
                let mut message = ChatMessage::assistant(&response.content);
                message.tool_calls = Some(response.tool_calls.clone());
                message
            };
            messages.push(assistant);

            for call in response.tool_calls {
                if cancel.is_cancelled() {
                    let _ = tx
                        .send(AgentEvent::Failed("agent run cancelled".into()))
                        .await;
                    return Ok(());
                }

                match self.run_tool_call(tools, &registry, &call, &tx).await {
                    Ok(result) => messages.push(ChatMessage::tool_result(&call.name, result)),
                    Err(error) => {
                        let _ = tx.send(AgentEvent::Failed(error.to_string())).await;
                        return Ok(());
                    }
                }
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

        let Some(mcp_tool) = registry.mcp_tool(&call.name) else {
            let _ = tx
                .send(AgentEvent::ToolCallFailed {
                    tool: call.name.clone(),
                    error: format!("unknown tool: {}", call.name),
                })
                .await;
            return Err(
                nest_error::NestError::network(format!("unknown tool: {}", call.name))
                    .with_module("nest-agent"),
            );
        };

        if !may_auto_run(self.config.auto_run_policy, mcp_tool) {
            let _ = tx
                .send(AgentEvent::ToolCallFailed {
                    tool: call.name.clone(),
                    error: "tool requires approval".into(),
                })
                .await;
            return Err(
                nest_error::NestError::network(format!(
                    "tool {} requires approval",
                    call.name
                ))
                .with_module("nest-agent"),
            );
        }

        let qualified = registry
            .qualified_name(&call.name)
            .expect("tool exists in registry")
            .to_string();

        debug!(tool = %call.name, qualified = %qualified, "executing MCP tool");
        let started = Instant::now();
        let result = match tokio::time::timeout(
            self.config.tool_timeout,
            tools.call_tool(&qualified, call.arguments.clone()),
        )
        .await
        {
            Ok(Ok(text)) => text,
            Ok(Err(error)) => {
                let _ = tx
                    .send(AgentEvent::ToolCallFailed {
                        tool: call.name.clone(),
                        error: error.to_string(),
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
                return Err(
                    nest_error::NestError::network(message).with_module("nest-agent"),
                );
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

fn ensure_system_prompt(messages: &mut Vec<ChatMessage>, tool_count: usize) {
    if messages
        .first()
        .is_some_and(|message| message.role == nest_ai::ChatRole::System)
    {
        return;
    }

    messages.insert(
        0,
        ChatMessage::system(format!(
            "You are Kiwi, a coding assistant with access to {tool_count} tools. \
             Call tools to gather information when needed. You may call tools multiple times. \
             When you have enough context, reply with a clear final answer."
        )),
    );
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
    use async_trait::async_trait;
    use nest_ai::{AiProvider, AiResult, CompletionResponse};
    use nest_mcp::McpTool;
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    struct MockTools {
        tools: Vec<McpTool>,
        calls: Vec<(String, serde_json::Value)>,
    }

    #[async_trait]
    impl ToolSource for MockTools {
        async fn list_tools(&mut self) -> NestResult<Vec<McpTool>> {
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
                });
            }
            Ok(responses.remove(0))
        }
    }

    fn search_tool() -> McpTool {
        McpTool {
            server: "nest-memory".into(),
            name: "search_project_memory".into(),
            qualified_name: "nest-memory/search_project_memory".into(),
            description: "Search".into(),
            input_schema: json!({"type": "object"}),
        }
    }

    fn save_tool() -> McpTool {
        McpTool {
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
                },
                CompletionResponse {
                    model: "mock".into(),
                    content: "nest-core is the module system".into(),
                    done: true,
                    tool_calls: vec![],
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
        assert_eq!(
            finished.as_deref(),
            Some("nest-core is the module system")
        );
        assert_eq!(tools.calls.len(), 1);
    }

    #[tokio::test]
    async fn agent_blocks_unapproved_tools() {
        let ai = AiService::new(Arc::new(MockAi {
            responses: Mutex::new(vec![CompletionResponse {
                model: "mock".into(),
                content: String::new(),
                done: true,
                tool_calls: vec![ToolCall::new(
                    "nest_context_memory__save_context_memory",
                    json!({"content": "x"}),
                )],
            }]),
        }));

        let (tx, mut rx) = mpsc::channel(8);
        let mut tools = MockTools {
            tools: vec![save_tool()],
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

        let mut failed = false;
        let mut policy_block = false;
        while let Ok(event) = rx.try_recv() {
            match event {
                AgentEvent::ToolCallFailed { error, .. } if error.contains("approval") => {
                    policy_block = true;
                }
                AgentEvent::Failed(_) => failed = true,
                _ => {}
            }
        }
        assert!(policy_block);
        assert!(failed);
        assert!(tools.calls.is_empty());
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
                },
                CompletionResponse {
                    model: "mock".into(),
                    content: String::new(),
                    done: true,
                    tool_calls: vec![ToolCall::new(
                        "nest_memory__search_project_memory",
                        json!({"query": "b"}),
                    )],
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
