use super::mode::{PermissionMode, SharedMode};
use super::prompt::system_instructions;
use super::session::Session;
use super::subagent;
use crate::api::types::{
    function_call_output_item, replay_output_items, user_text_item, FunctionCallRef,
    ReasoningConfig, ResponseRequest,
};
use crate::api::{ApiResponse, MetaClient, StreamEvent};
use crate::config::Config;
use crate::error::{MuseError, Result};
use crate::tools::{ToolContext, ToolHost};
use crate::usage::{TokenUsage, UsageTracker};
use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

/// Events emitted while an agent turn runs.
pub enum AgentEvent {
    Status(String),
    ReasoningDelta(String),
    TextDelta(String),
    AssistantMessage(String),
    ToolStart { id: u64, name: String, args: String },
    ToolEnd {
        id: u64,
        name: String,
        result: String,
        ok: bool,
    },
    /// Todo list changed — TUI should refresh.
    TodosChanged(String),
    /// Plan written via submit_plan.
    PlanSubmitted(String),
    ApprovalRequest {
        name: String,
        args: String,
        respond: oneshot::Sender<ApprovalDecision>,
    },
    Usage { session: TokenUsage, last: TokenUsage },
    Done {
        session: Box<Session>,
        usage: Box<UsageTracker>,
        result: std::result::Result<String, String>,
        interrupted: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approve,
    ApproveAlways,
    Deny,
}

pub const READ_ONLY_TOOLS: &[&str] = &[
    "read_file",
    "grep",
    "glob",
    "web_fetch",
    "git_status",
    "memory",
    "todo_write",
    "submit_plan",
];

pub const MUTATING_TOOLS: &[&str] = &[
    "write_file",
    "edit_file",
    "multi_edit",
    "apply_patch",
    "bash",
];

pub struct AgentRunner {
    pub client: MetaClient,
    pub config: Config,
    pub cwd: PathBuf,
    pub permission_mode: SharedMode,
    #[allow(dead_code)]
    pub verbose: bool,
    pub approved_tools: Arc<Mutex<HashSet<String>>>,
    pub tools: ToolHost,
    /// Nested subagents cannot spawn further agents (depth limit 1).
    pub is_subagent: bool,
}

pub fn spawn_turn(
    runner: Arc<AgentRunner>,
    mut session: Session,
    mut usage: UsageTracker,
    prompt: String,
    tx: mpsc::UnboundedSender<AgentEvent>,
    cancel: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let res = runner
            .run_turn_events(&mut session, &prompt, &mut usage, &tx, &cancel)
            .await;
        usage.set_state("idle");
        let _ = session.save();
        let interrupted = matches!(res, Err(MuseError::Interrupted));
        let result = res.map_err(|e| e.to_string());
        let _ = tx.send(AgentEvent::Done {
            session: Box::new(session),
            usage: Box::new(usage),
            result,
            interrupted,
        });
    })
}

impl AgentRunner {
    pub async fn run_turn_events(
        &self,
        session: &mut Session,
        user_text: &str,
        usage: &mut UsageTracker,
        tx: &mpsc::UnboundedSender<AgentEvent>,
        cancel: &CancellationToken,
    ) -> Result<String> {
        session.push_user(user_text);
        session.input_items.push(user_text_item(user_text));

        let tools = self.tools.tool_defs();
        let mut turns = 0u32;
        let mut tool_seq: u64 = 0;

        loop {
            if cancel.is_cancelled() {
                return Err(MuseError::Interrupted);
            }
            turns += 1;
            if turns > self.config.max_turns {
                return Err(MuseError::MaxTurns(self.config.max_turns));
            }

            // Auto-compact when context is hot (Claude-style).
            if should_auto_compact(usage, &self.config) {
                let _ = tx.send(AgentEvent::Status("auto-compacting context…".into()));
                if let Ok(summary) = compact_session(self, session, usage).await {
                    let _ = tx.send(AgentEvent::Status(
                        "context compacted — continuing".into(),
                    ));
                    let _ = summary;
                }
            }

            let mode_now = self.permission_mode.get();
            let instructions = system_instructions(
                &self.cwd,
                mode_now,
                self.is_subagent,
                &self.tools.todos_snapshot().render(),
            );

            usage.set_state(format!("thinking (turn {turns})"));
            let _ = tx.send(AgentEvent::Status(format!(
                "thinking · turn {turns} · {}",
                mode_now.label()
            )));

            let req = ResponseRequest {
                model: self.config.model.clone(),
                input: Value::Array(session.input_items.clone()),
                instructions: Some(instructions),
                tools: Some(tools.clone()),
                tool_choice: Some("auto".into()),
                store: Some(false),
                include: Some(vec!["reasoning.encrypted_content".into()]),
                reasoning: Some(ReasoningConfig {
                    effort: Some(self.config.reasoning_effort.clone()),
                    summary: Some("auto".into()),
                }),
                stream: Some(self.config.stream && !self.is_subagent),
                parallel_tool_calls: Some(true),
            };

            let mut text_deltas = 0usize;
            let resp: ApiResponse = if req.stream == Some(true) {
                self.client
                    .create_response_stream(
                        &req,
                        |ev| match ev {
                            StreamEvent::TextDelta(d) => {
                                text_deltas += 1;
                                let _ = tx.send(AgentEvent::TextDelta(d));
                            }
                            StreamEvent::ReasoningDelta(d) => {
                                let _ = tx.send(AgentEvent::ReasoningDelta(d));
                            }
                            StreamEvent::Completed(_) => {}
                        },
                        cancel,
                    )
                    .await?
            } else {
                tokio::select! {
                    _ = cancel.cancelled() => return Err(MuseError::Interrupted),
                    r = self.client.create_response(&req) => r?,
                }
            };

            if let Some(u) = &resp.usage {
                let tu: TokenUsage = u.into();
                usage.record_request(tu.clone(), resp.id.clone());
                session.usage.add(&tu);
                let _ = tx.send(AgentEvent::Usage {
                    session: usage.session_usage().clone(),
                    last: tu,
                });
            }

            let replayed = replay_output_items(&resp.output);
            session.input_items.extend(replayed);

            let calls = resp.function_calls();
            let text = resp.output_text();

            if text_deltas == 0 && !text.is_empty() {
                let _ = tx.send(AgentEvent::AssistantMessage(text.clone()));
            }

            if calls.is_empty() {
                usage.set_state("idle");
                session.push_assistant(&text);
                let _ = session.save();
                return Ok(text);
            }

            // Partition: run read-only tools in parallel; mutating / agent sequential.
            let (readonly, sequential): (Vec<_>, Vec<_>) = calls
                .into_iter()
                .partition(|c| is_parallel_safe(&c.name) && c.name != "agent");

            // --- parallel readonly batch ---
            if !readonly.is_empty() {
                let mut handles = Vec::new();
                for call in &readonly {
                    if cancel.is_cancelled() {
                        return Err(MuseError::Interrupted);
                    }
                    tool_seq += 1;
                    let id = tool_seq;
                    let _ = tx.send(AgentEvent::ToolStart {
                        id,
                        name: call.name.clone(),
                        args: call.arguments.clone(),
                    });
                    // Readonly always approved
                    let host = ToolHost {
                        todos: self.tools.todos.clone(),
                        plan: self.tools.plan.clone(),
                    };
                    let cwd = self.cwd.clone();
                    let name = call.name.clone();
                    let args = call.arguments.clone();
                    let call_id = call.call_id.clone();
                    handles.push(tokio::task::spawn_blocking(move || {
                        let ctx = ToolContext {
                            cwd,
                            auto_approve: true,
                        };
                        let res = host.dispatch(&name, &args, &ctx);
                        (id, call_id, name, res)
                    }));
                }

                for h in handles {
                    let (id, call_id, name, res) = tokio::select! {
                        _ = cancel.cancelled() => return Err(MuseError::Interrupted),
                        r = h => r.map_err(|e| MuseError::Other(e.to_string()))?,
                    };
                    let (body, ok) = match res {
                        Ok(s) => (s, true),
                        Err(e) => (format!("error: {e}"), false),
                    };
                    emit_side_effects(tx, &name, &body);
                    let _ = tx.send(AgentEvent::ToolEnd {
                        id,
                        name,
                        result: body.clone(),
                        ok,
                    });
                    session
                        .input_items
                        .push(function_call_output_item(&call_id, &body));
                }
            }

            // --- sequential mutating / agent ---
            for call in sequential {
                if cancel.is_cancelled() {
                    session.input_items.push(function_call_output_item(
                        &call.call_id,
                        "[interrupted by user]",
                    ));
                    return Err(MuseError::Interrupted);
                }

                tool_seq += 1;
                let id = tool_seq;
                let _ = tx.send(AgentEvent::ToolStart {
                    id,
                    name: call.name.clone(),
                    args: call.arguments.clone(),
                });

                let mode_at_gate = self.permission_mode.get();
                let approved = self.check_approval(&call.name, &call.arguments, tx).await;
                if !approved {
                    let (msg, result_label) = if mode_at_gate.is_read_only_enforced()
                        && (MUTATING_TOOLS.contains(&call.name.as_str()) || call.name == "agent")
                    {
                        (
                            format!(
                                "blocked: plan mode. Only read-only tools allowed. \
                                 Switch to manual/auto (Shift+Tab) for {}.",
                                call.name
                            ),
                            "blocked · plan mode".into(),
                        )
                    } else {
                        (
                            "user denied this tool call".into(),
                            "denied by user".into(),
                        )
                    };
                    let _ = tx.send(AgentEvent::ToolEnd {
                        id,
                        name: call.name.clone(),
                        result: result_label,
                        ok: false,
                    });
                    session
                        .input_items
                        .push(function_call_output_item(&call.call_id, &msg));
                    continue;
                }

                usage.set_state(format!("tool:{}", call.name));

                let (body, ok) = if call.name == "agent" {
                    if self.is_subagent {
                        (
                            "error: nested subagents are not allowed (depth limit)".into(),
                            false,
                        )
                    } else {
                        match run_agent_tool(self, &call, cancel, tx).await {
                            Ok(s) => (s, true),
                            Err(MuseError::Interrupted) => return Err(MuseError::Interrupted),
                            Err(e) => (format!("error: {e}"), false),
                        }
                    }
                } else {
                    let host = ToolHost {
                        todos: self.tools.todos.clone(),
                        plan: self.tools.plan.clone(),
                    };
                    let cwd = self.cwd.clone();
                    let name = call.name.clone();
                    let args = call.arguments.clone();
                    let exec = tokio::task::spawn_blocking(move || {
                        host.dispatch(
                            &name,
                            &args,
                            &ToolContext {
                                cwd,
                                auto_approve: true,
                            },
                        )
                    });
                    tokio::select! {
                        _ = cancel.cancelled() => {
                            session.input_items.push(function_call_output_item(
                                &call.call_id,
                                "[interrupted by user]",
                            ));
                            return Err(MuseError::Interrupted);
                        }
                        r = exec => match r {
                            Ok(Ok(s)) => (s, true),
                            Ok(Err(e)) => (format!("error: {e}"), false),
                            Err(e) => (format!("error: tool panicked: {e}"), false),
                        },
                    }
                };

                emit_side_effects(tx, &call.name, &body);
                let _ = tx.send(AgentEvent::ToolEnd {
                    id,
                    name: call.name.clone(),
                    result: body.clone(),
                    ok,
                });
                session
                    .input_items
                    .push(function_call_output_item(&call.call_id, &body));
            }

            let _ = session.save();
        }
    }

    async fn check_approval(
        &self,
        name: &str,
        args: &str,
        tx: &mpsc::UnboundedSender<AgentEvent>,
    ) -> bool {
        let mode = self.permission_mode.get();
        let read_only = READ_ONLY_TOOLS.contains(&name) || name == "submit_plan";

        match mode {
            PermissionMode::Auto => true,
            PermissionMode::Plan => {
                if read_only && name != "agent" {
                    true
                } else {
                    let _ = tx.send(AgentEvent::Status(format!("plan mode blocked · {name}")));
                    false
                }
            }
            PermissionMode::Manual => {
                if read_only {
                    return true;
                }
                // explore-style agent is ok after approval; general agent too
                if let Ok(set) = self.approved_tools.lock() {
                    if set.contains(name) {
                        return true;
                    }
                }
                let (otx, orx) = oneshot::channel();
                if tx
                    .send(AgentEvent::ApprovalRequest {
                        name: name.to_string(),
                        args: args.to_string(),
                        respond: otx,
                    })
                    .is_err()
                {
                    return false;
                }
                match orx.await {
                    Ok(ApprovalDecision::Approve) => true,
                    Ok(ApprovalDecision::ApproveAlways) => {
                        if let Ok(mut set) = self.approved_tools.lock() {
                            set.insert(name.to_string());
                        }
                        true
                    }
                    Ok(ApprovalDecision::Deny) => false,
                    Err(_) => self.permission_mode.get().auto_approves(),
                }
            }
        }
    }
}

fn is_parallel_safe(name: &str) -> bool {
    matches!(
        name,
        "read_file" | "grep" | "glob" | "web_fetch" | "git_status" | "memory"
    )
}

fn should_auto_compact(usage: &UsageTracker, cfg: &Config) -> bool {
    let last = usage.last_usage();
    let used = last.input_tokens.max(last.total_tokens);
    let window = cfg.context_window.max(1);
    // When a single request's input exceeds ~55% of the context window, compact.
    used > (window as f64 * 0.55) as u64 && used > 40_000
}

fn emit_side_effects(tx: &mpsc::UnboundedSender<AgentEvent>, name: &str, body: &str) {
    if name == "todo_write" {
        let _ = tx.send(AgentEvent::TodosChanged(body.to_string()));
    }
    if name == "submit_plan" {
        let _ = tx.send(AgentEvent::PlanSubmitted(body.to_string()));
    }
}

async fn run_agent_tool(
    runner: &AgentRunner,
    call: &FunctionCallRef,
    cancel: &CancellationToken,
    tx: &mpsc::UnboundedSender<AgentEvent>,
) -> Result<String> {
    let v: Value = serde_json::from_str(&call.arguments).unwrap_or(serde_json::json!({}));
    let prompt = v
        .get("prompt")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    if prompt.is_empty() {
        return Err(MuseError::Tool("agent.prompt required".into()));
    }
    let kind = v
        .get("subagent_type")
        .and_then(|x| x.as_str())
        .unwrap_or("explore");
    let desc = v
        .get("description")
        .and_then(|x| x.as_str())
        .unwrap_or(kind);
    let _ = tx.send(AgentEvent::Status(format!("subagent · {desc}")));

    subagent::run_subagent(
        runner.client.clone(),
        runner.config.clone(),
        runner.cwd.clone(),
        runner.permission_mode.clone(),
        &prompt,
        kind,
        cancel,
    )
    .await
}

pub async fn compact_session(
    runner: &AgentRunner,
    session: &mut Session,
    usage: &mut UsageTracker,
) -> Result<String> {
    let mut items = session.input_items.clone();
    items.push(user_text_item(
        "Summarize this conversation for a fresh context window. Capture: goals, decisions, \
         files touched, current state, pending next steps. Dense bullets.",
    ));
    let req = ResponseRequest {
        model: runner.config.model.clone(),
        input: Value::Array(items),
        instructions: Some("You compress agent conversations into handoff summaries.".into()),
        tools: None,
        tool_choice: None,
        store: Some(false),
        include: Some(vec!["reasoning.encrypted_content".into()]),
        reasoning: Some(ReasoningConfig {
            effort: Some("low".into()),
            summary: None,
        }),
        stream: Some(false),
        parallel_tool_calls: None,
    };
    let resp = runner.client.create_response(&req).await?;
    if let Some(u) = &resp.usage {
        let tu: TokenUsage = u.into();
        usage.record_request(tu.clone(), resp.id.clone());
        session.usage.add(&tu);
    }
    let summary = resp.output_text();
    if summary.is_empty() {
        return Err(MuseError::Other("compaction produced no summary".into()));
    }
    session.input_items = vec![user_text_item(&format!(
        "[Context compacted. Summary of the conversation so far:]\n\n{summary}"
    ))];
    let _ = session.save();
    Ok(summary)
}
