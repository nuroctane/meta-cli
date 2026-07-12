//! Session todo list — Claude Code-style task tracking the model updates via tool.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl TodoStatus {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "pending" | "todo" => Some(Self::Pending),
            "in_progress" | "in-progress" | "doing" | "active" => Some(Self::InProgress),
            "completed" | "done" => Some(Self::Completed),
            "cancelled" | "canceled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn glyph(&self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::InProgress => "●",
            Self::Completed => "✓",
            Self::Cancelled => "✗",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: TodoStatus,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TodoList {
    pub items: Vec<TodoItem>,
}

impl TodoList {
    pub fn render(&self) -> String {
        if self.items.is_empty() {
            return "(no todos)".into();
        }
        let mut s = String::new();
        for t in &self.items {
            s.push_str(&format!(
                "{} [{}] {} — {}\n",
                t.status.glyph(),
                t.status.as_str(),
                t.id,
                t.content
            ));
        }
        s
    }

    #[allow(dead_code)] // kept for future statusline use
    pub fn summary(&self) -> String {
        let total = self.items.len();
        let done = self
            .items
            .iter()
            .filter(|t| t.status == TodoStatus::Completed)
            .count();
        let active = self
            .items
            .iter()
            .filter(|t| t.status == TodoStatus::InProgress)
            .count();
        format!("{done}/{total} done · {active} active")
    }

    /// Replace or merge. If `merge` false, replace entire list.
    pub fn apply(&mut self, items: Vec<TodoItem>, merge: bool) {
        if !merge {
            self.items = items;
            return;
        }
        for it in items {
            if let Some(existing) = self.items.iter_mut().find(|e| e.id == it.id) {
                *existing = it;
            } else {
                self.items.push(it);
            }
        }
    }

    #[allow(dead_code)] // kept for future persistence
    pub fn save(&self, path: &PathBuf) {
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::create_dir_all(path.parent().unwrap_or(path));
            let _ = std::fs::write(path, text);
        }
    }

    #[allow(dead_code)] // kept for future persistence
    pub fn load(path: &PathBuf) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }
}

pub type SharedTodos = Arc<Mutex<TodoList>>;

pub fn shared_empty() -> SharedTodos {
    Arc::new(Mutex::new(TodoList::default()))
}
