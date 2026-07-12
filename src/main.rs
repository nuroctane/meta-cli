mod ade;
mod agent;
mod api;
mod auth;
mod cli;
mod config;
mod error;
mod theme;
mod tools;
mod tui;
mod usage;

use agent::session::{print_sessions, Session};
use agent::AgentRunner;
use api::MetaClient;
use auth::{auth_status, login_interactive, logout, resolve_api_key, save_api_key};
use clap::Parser;
use cli::{AuthCmd, Cli, Commands};
use config::{load_config, Config};
use error::Result;
use std::path::PathBuf;
use usage::{print_usage_summary, UsageTracker};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_target(false)
        .init();

    if let Err(e) = real_main().await {
        theme::print_err(&e.to_string());
        std::process::exit(1);
    }
}

async fn real_main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Auth { action }) => {
            match action {
                AuthCmd::Login { key } => login_interactive(key.clone())?,
                AuthCmd::Status => auth_status()?,
                AuthCmd::Logout => {
                    logout()?;
                    theme::print_ok("logged out (removed ~/.muse/auth.json)");
                }
            }
            return Ok(());
        }
        Some(Commands::Usage) => {
            print_usage_summary()?;
            return Ok(());
        }
        Some(Commands::Sessions { limit }) => {
            print_sessions(*limit)?;
            return Ok(());
        }
        Some(Commands::InstallHook) => {
            ade::install_orca_hook()?;
            return Ok(());
        }
        _ => {}
    }

    let api_key = match resolve_api_key() {
        Ok(k) => k,
        Err(_) => {
            if let Ok(k) = std::env::var("MODEL_API_KEY").or_else(|_| std::env::var("MUSE_API_KEY"))
            {
                if !k.trim().is_empty() {
                    let _ = save_api_key(k.trim());
                    k
                } else {
                    return Err(error::MuseError::NotAuthenticated);
                }
            } else {
                return Err(error::MuseError::NotAuthenticated);
            }
        }
    };

    let mut cfg = load_config()?;
    if let Some(m) = &cli.model {
        cfg.model = m.clone();
    }
    if let Some(e) = &cli.effort {
        cfg.reasoning_effort = e.clone();
    }
    if let Some(t) = cli.max_turns {
        cfg.max_turns = t;
    }

    let cwd = cli
        .cwd
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?);
    let cwd_str = cwd.display().to_string();

    let client = MetaClient::new(&cfg.base_url, &api_key)?;

    let mut session = if let Some(id) = &cli.resume {
        theme::print_info(&format!("resuming session {id}…"));
        Session::load(id)?
    } else if cli.continue_session {
        theme::print_info("continuing last session for this directory…");
        Session::continue_for_cwd(&cwd_str)?
    } else {
        Session::new(&cfg.model, &cwd_str)
    };

    // Keep model in sync if user overrode
    if session.model != cfg.model {
        session.model = cfg.model.clone();
    }
    session.cwd = cwd_str.clone();

    let mut usage = UsageTracker::new(session.id.clone(), cfg.model.clone(), cwd.clone());
    // Seed tracker with prior session usage so ADE totals stay cumulative
    if session.usage.total_tokens > 0 {
        usage.seed_session(session.usage.clone());
    }

    std::env::set_var("MUSE_STATUS_PATH", config::status_path().display().to_string());
    std::env::set_var(
        "MUSE_USAGE_LOG_PATH",
        config::usage_log_path().display().to_string(),
    );
    std::env::set_var("MUSE_SESSION_ID", &session.id);
    std::env::set_var("MUSE_MODEL", &cfg.model);
    std::env::set_var("MUSE_PROVIDER", "meta");
    std::env::set_var("MUSE_HOME", config::muse_home().display().to_string());

    ade::write_ade_manifest(&session.id, &cfg.model, &cwd_str, usage.session_usage());
    let _ = session.save();

    match &cli.command {
        Some(Commands::Run { prompt, yes }) => {
            let prompt = prompt.join(" ");
            let auto = *yes || cli.yes;
            run_headless(client, cfg, cwd, session, usage, &prompt, auto, cli.verbose).await?;
        }
        None => {
            ade::set_terminal_title(&format!(
                "muse · {}",
                &session.id[..8.min(session.id.len())]
            ));
            theme::banner();
            theme::print_info(&format!(
                "session {} · model {} · usage → {}",
                &session.id[..8.min(session.id.len())],
                cfg.model,
                config::status_path().display()
            ));
            let runner = AgentRunner {
                client,
                config: cfg,
                cwd,
                auto_approve: cli.yes,
                verbose: false,
            };
            tui::run_tui(runner, session, usage, cli.prompt.clone()).await?;
        }
        Some(Commands::Auth { .. })
        | Some(Commands::Usage)
        | Some(Commands::Sessions { .. })
        | Some(Commands::InstallHook) => unreachable!(),
    }

    Ok(())
}

async fn run_headless(
    client: MetaClient,
    cfg: Config,
    cwd: PathBuf,
    mut session: Session,
    mut usage: UsageTracker,
    prompt: &str,
    auto_approve: bool,
    verbose: bool,
) -> Result<()> {
    let runner = AgentRunner {
        client,
        config: cfg,
        cwd,
        auto_approve,
        verbose,
    };

    let text = runner
        .run_turn(&mut session, prompt, &mut usage, |ev| {
            if verbose {
                match ev.kind {
                    agent::AgentEventKind::ToolStart => theme::print_tool("→", &ev.text),
                    agent::AgentEventKind::ToolEnd => theme::print_info(&ev.text),
                    agent::AgentEventKind::Status => theme::print_info(&ev.text),
                    agent::AgentEventKind::Error => theme::print_err(&ev.text),
                    agent::AgentEventKind::Assistant => {}
                }
            }
        })
        .await?;

    println!("{text}");

    let u = usage.session_usage();
    if verbose {
        eprintln!(
            "\n--- usage: in={} out={} total={} ~${:.6} ---",
            u.input_tokens,
            u.output_tokens,
            u.total_tokens,
            u.estimated_cost_usd()
        );
        eprintln!("status: {}", config::status_path().display());
    }
    Ok(())
}
