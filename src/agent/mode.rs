//! Session permission modes — modeled after Claude Code / Codex practice.
//!
//! | Mode          | Behavior |
//! |---------------|----------|
//! | **manual**    | Read-only tools free; write/bash need approval |
//! | **plan**      | Explore + shell freely (incl. scratch/media compute); code authoring & repo/VCS commits blocked |
//! | **auto**      | Auto-approve tools (full auto-approve) |
//!
//! Mode is held in an `Arc<AtomicU8>` so Shift+Tab / `/mode` takes effect
//! **immediately**, including for an in-flight agent turn's next tool call.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PermissionMode {
    /// Review each mutating tool (default). Labeled "manual" in the UI.
    Manual = 0,
    /// Explore only — no writes, no shell that can mutate.
    Plan = 1,
    /// Auto-approve tools without prompts.
    Auto = 2,
}

impl PermissionMode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Plan,
            2 => Self::Auto,
            _ => Self::Manual,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "manual" | "default" | "ask" | "normal" => Some(Self::Manual),
            "plan" | "readonly" | "read-only" | "read_only" => Some(Self::Plan),
            "auto" | "auto-approve" | "autoapprove" | "yolo" | "bypass" => Some(Self::Auto),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Plan => "plan",
            Self::Auto => "auto",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::Manual => "⏸ manual",
            Self::Plan => "◈ plan",
            Self::Auto => "⏵ auto",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Manual => "approve writes & shell; reads free",
            Self::Plan => "explore + run shell freely; no code edits or repo commits",
            Self::Auto => "auto-approve tools (full auto)",
        }
    }

    /// Cycle manual → plan → auto → manual (Claude Code Shift+Tab pattern).
    pub fn next(self) -> Self {
        match self {
            Self::Manual => Self::Plan,
            Self::Plan => Self::Auto,
            Self::Auto => Self::Manual,
        }
    }

    pub fn is_read_only_enforced(self) -> bool {
        matches!(self, Self::Plan)
    }

    pub fn auto_approves(self) -> bool {
        matches!(self, Self::Auto)
    }
}

/// Thread-safe, immediately-visible permission mode for the session.
#[derive(Clone, Debug)]
pub struct SharedMode {
    inner: Arc<AtomicU8>,
}

impl SharedMode {
    pub fn new(mode: PermissionMode) -> Self {
        Self {
            inner: Arc::new(AtomicU8::new(mode.as_u8())),
        }
    }

    /// Current mode — cheap atomic load; safe mid-turn.
    pub fn get(&self) -> PermissionMode {
        PermissionMode::from_u8(self.inner.load(Ordering::SeqCst))
    }

    /// Set mode immediately (visible to in-flight tool gates on next check).
    pub fn set(&self, mode: PermissionMode) {
        self.inner.store(mode.as_u8(), Ordering::SeqCst);
    }

    pub fn cycle(&self) -> PermissionMode {
        let next = self.get().next();
        self.set(next);
        next
    }
}

impl Default for SharedMode {
    fn default() -> Self {
        Self::new(PermissionMode::Manual)
    }
}
