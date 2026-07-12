use crate::agent::{AgentEventKind, AgentRunner, Session};
use crate::error::Result;
use crate::theme;
use crate::usage::UsageTracker;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::stdout;
use std::time::Duration;

struct TranscriptLine {
    kind: LineKind,
    text: String,
}

enum LineKind {
    User,
    Assistant,
    Tool,
    Status,
    System,
}

pub async fn run_tui(
    runner: AgentRunner,
    mut session: Session,
    mut usage: UsageTracker,
    initial_prompt: Option<String>,
) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut input = String::new();
    let mut transcript: Vec<TranscriptLine> = Vec::new();
    let mut scroll: u16 = 0;
    let mut busy = false;
    let mut status = format!(
        "muse · {} · session {}…",
        runner.config.model,
        &session.id[..8.min(session.id.len())]
    );

    transcript.push(TranscriptLine {
        kind: LineKind::System,
        text: "Muse Spark coding agent — type a prompt, Enter to send, Ctrl+C / /exit to quit. /usage for tokens."
            .into(),
    });

    if let Some(p) = initial_prompt {
        if !p.trim().is_empty() {
            input = p;
            // Will send on first loop iteration via force_send
        }
    }
    let mut force_send = !input.is_empty();

    let result = async {
        loop {
            // Render
            let u = usage.session_usage();
            let usage_line = format!(
                "tokens in={} out={} total={}  ~${:.4}  |  {}",
                u.input_tokens,
                u.output_tokens,
                u.total_tokens,
                u.estimated_cost_usd(),
                status
            );

            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(3),
                        Constraint::Length(1),
                    ])
                    .split(f.area());

                let title = Paragraph::new(Line::from(vec![
                    Span::styled(" MUSE ", theme::style_title().add_modifier(Modifier::REVERSED)),
                    Span::raw(" "),
                    Span::styled("Spark", theme::style_title()),
                    Span::styled(" · Meta Model API", theme::style_status()),
                ]))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::META_BLUE)),
                );
                f.render_widget(title, chunks[0]);

                let lines: Vec<Line> = transcript
                    .iter()
                    .map(|t| {
                        let (prefix, style) = match t.kind {
                            LineKind::User => ("you", theme::style_user()),
                            LineKind::Assistant => ("muse", theme::style_assistant()),
                            LineKind::Tool => ("tool", theme::style_tool()),
                            LineKind::Status => ("…", theme::style_status()),
                            LineKind::System => ("sys", theme::style_status()),
                        };
                        Line::from(vec![
                            Span::styled(format!("{prefix:>4} "), style),
                            Span::styled(t.text.clone(), style),
                        ])
                    })
                    .collect();

                let para = Paragraph::new(lines)
                    .wrap(Wrap { trim: false })
                    .scroll((scroll, 0))
                    .block(
                        Block::default()
                            .title(" transcript ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::META_BLUE)),
                    );
                f.render_widget(para, chunks[1]);

                let input_title = if busy { " thinking… " } else { " prompt " };
                let input_w = Paragraph::new(input.as_str())
                    .block(
                        Block::default()
                            .title(input_title)
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme::META_BLUE_BRIGHT)),
                    );
                f.render_widget(input_w, chunks[2]);

                let status_w = Paragraph::new(usage_line).style(theme::style_status());
                f.render_widget(status_w, chunks[3]);
            })?;

            // Handle force send of initial prompt
            if force_send && !busy {
                force_send = false;
                let prompt = input.clone();
                input.clear();
                if let Err(e) = handle_prompt(
                    &runner,
                    &mut session,
                    &mut usage,
                    &mut transcript,
                    &mut status,
                    &mut busy,
                    &prompt,
                )
                .await
                {
                    transcript.push(TranscriptLine {
                        kind: LineKind::Status,
                        text: format!("error: {e}"),
                    });
                    busy = false;
                }
                continue;
            }

            // Poll events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Esc => break,
                        KeyCode::Enter if !busy => {
                            let prompt = input.trim().to_string();
                            input.clear();
                            if prompt.is_empty() {
                                continue;
                            }
                            if prompt == "/exit" || prompt == "/quit" {
                                break;
                            }
                            if prompt == "/usage" {
                                let u = usage.session_usage();
                                transcript.push(TranscriptLine {
                                    kind: LineKind::System,
                                    text: format!(
                                        "session tokens in={} out={} total={} ~${:.6} | status {}",
                                        u.input_tokens,
                                        u.output_tokens,
                                        u.total_tokens,
                                        u.estimated_cost_usd(),
                                        crate::config::status_path().display()
                                    ),
                                });
                                continue;
                            }
                            if prompt == "/help" {
                                transcript.push(TranscriptLine {
                                    kind: LineKind::System,
                                    text: "/usage · /exit · Enter send · PgUp/PgDn scroll".into(),
                                });
                                continue;
                            }
                            if let Err(e) = handle_prompt(
                                &runner,
                                &mut session,
                                &mut usage,
                                &mut transcript,
                                &mut status,
                                &mut busy,
                                &prompt,
                            )
                            .await
                            {
                                transcript.push(TranscriptLine {
                                    kind: LineKind::Status,
                                    text: format!("error: {e}"),
                                });
                                busy = false;
                            }
                        }
                        KeyCode::Backspace if !busy => {
                            input.pop();
                        }
                        KeyCode::Char(c) if !busy => {
                            input.push(c);
                        }
                        KeyCode::PageUp => {
                            scroll = scroll.saturating_add(5);
                        }
                        KeyCode::PageDown => {
                            scroll = scroll.saturating_sub(5);
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok::<(), crate::error::MuseError>(())
    }
    .await;

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    result?;
    Ok(())
}

async fn handle_prompt(
    runner: &AgentRunner,
    session: &mut Session,
    usage: &mut UsageTracker,
    transcript: &mut Vec<TranscriptLine>,
    status: &mut String,
    busy: &mut bool,
    prompt: &str,
) -> Result<()> {
    *busy = true;
    transcript.push(TranscriptLine {
        kind: LineKind::User,
        text: prompt.to_string(),
    });

    let mut events = Vec::new();
    let text = runner
        .run_turn(session, prompt, usage, |ev| {
            events.push(ev);
        })
        .await?;

    for ev in events {
        let kind = match ev.kind {
            AgentEventKind::Assistant => LineKind::Assistant,
            AgentEventKind::ToolStart | AgentEventKind::ToolEnd => LineKind::Tool,
            AgentEventKind::Status => LineKind::Status,
            AgentEventKind::Error => LineKind::Status,
        };
        // Avoid duplicating final assistant text if we push again
        if matches!(kind, LineKind::Assistant) && ev.text == text {
            continue;
        }
        if matches!(kind, LineKind::Status) {
            *status = ev.text.clone();
        }
        transcript.push(TranscriptLine {
            kind,
            text: ev.text,
        });
    }

    if !text.is_empty() {
        transcript.push(TranscriptLine {
            kind: LineKind::Assistant,
            text,
        });
    }

    let u = usage.session_usage();
    *status = format!(
        "idle · tokens {} · ~${:.4}",
        u.total_tokens,
        u.estimated_cost_usd()
    );
    *busy = false;
    Ok(())
}

