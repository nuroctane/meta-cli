use super::prompt::system_instructions;
use super::session::Session;
use crate::api::types::{
    function_call_output_item, replay_output_items, user_text_item, ReasoningConfig,
    ResponseRequest,
};
use crate::api::{ApiResponse, MetaClient};
use crate::config::Config;
use crate::error::{MuseError, Result};
use crate::theme;
use crate::tools::{dispatch, tool_defs, ToolContext};
use crate::usage::{TokenUsage, UsageTracker};
use serde_json::Value;
use std::path::PathBuf;

pub struct AgentEvent {
    pub kind: AgentEventKind,
    pub text: String,
}

#[allow(dead_code)]
pub enum AgentEventKind {
    Assistant,
    ToolStart,
    ToolEnd,
    Status,
    Error,
}

pub struct AgentRunner {
    pub client: MetaClient,
    pub config: Config,
    pub cwd: PathBuf,
    pub auto_approve: bool,
    pub verbose: bool,
}

impl AgentRunner {
    pub async fn run_turn(
        &self,
        session: &mut Session,
        user_text: &str,
        usage: &mut UsageTracker,
        mut on_event: impl FnMut(AgentEvent),
    ) -> Result<String> {
        session.push_user(user_text);
        session.input_items.push(user_text_item(user_text));

        let instructions = system_instructions(&self.cwd);
        let tools = tool_defs();
        let tool_ctx = ToolContext {
            cwd: self.cwd.clone(),
            auto_approve: self.auto_approve,
        };

        let mut final_text = String::new();
        let mut turns = 0u32;

        loop {
            turns += 1;
            if turns > self.config.max_turns {
                return Err(MuseError::MaxTurns(self.config.max_turns));
            }

            usage.set_state(format!("thinking (turn {turns})"));
            on_event(AgentEvent {
                kind: AgentEventKind::Status,
                text: format!("Spark thinking… (turn {turns})"),
            });

            let req = ResponseRequest {
                model: self.config.model.clone(),
                input: Value::Array(session.input_items.clone()),
                instructions: Some(instructions.clone()),
                tools: Some(tools.clone()),
                tool_choice: Some("auto".into()),
                store: Some(false),
                include: Some(vec!["reasoning.encrypted_content".into()]),
                reasoning: Some(ReasoningConfig {
                    effort: Some(self.config.reasoning_effort.clone()),
                    summary: Some("auto".into()),
                }),
                stream: Some(false),
                parallel_tool_calls: Some(true),
            };

            let resp: ApiResponse = self.client.create_response(&req).await?;

            if let Some(u) = &resp.usage {
                let tu: TokenUsage = u.into();
                usage.record_request(tu.clone(), resp.id.clone());
                session.usage.add(&tu);
            }

            // Replay model output into history for next turn
            let replayed = replay_output_items(&resp.output);
            session.input_items.extend(replayed);

            let calls = resp.function_calls();
            let text = resp.output_text();

            if !text.is_empty() && calls.is_empty() {
                final_text = text.clone();
                on_event(AgentEvent {
                    kind: AgentEventKind::Assistant,
                    text: text.clone(),
                });
            } else if !text.is_empty() {
                on_event(AgentEvent {
                    kind: AgentEventKind::Assistant,
                    text: text.clone(),
                });
            }

            if calls.is_empty() {
                usage.set_state("idle");
                if final_text.is_empty() {
                    final_text = text;
                }
                session.push_assistant(&final_text);
                let _ = session.save();
                return Ok(final_text);
            }

            // Execute tools (parallel-capable sequential for safety)
            for call in calls {
                let detail = truncate_args(&call.arguments, 120);
                on_event(AgentEvent {
                    kind: AgentEventKind::ToolStart,
                    text: format!("{} {}", call.name, detail),
                });
                if self.verbose {
                    theme::print_tool(&call.name, &detail);
                }

                usage.set_state(format!("tool:{}", call.name));
                let result = match dispatch(&call.name, &call.arguments, &tool_ctx) {
                    Ok(s) => s,
                    Err(e) => format!("error: {e}"),
                };

                on_event(AgentEvent {
                    kind: AgentEventKind::ToolEnd,
                    text: format!("{} → {}", call.name, truncate_args(&result, 200)),
                });

                session
                    .input_items
                    .push(function_call_output_item(&call.call_id, &result));
            }

            let _ = session.save();
        }
    }
}

fn truncate_args(s: &str, max: usize) -> String {
    let s = s.replace('\n', " ");
    if s.len() <= max {
        s
    } else {
        format!("{}…", &s[..max])
    }
}
