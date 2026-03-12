use serde::Deserialize;

/// Top-level JSON payload piped to stdin by Claude Code's statusLine setting.
/// Uses `deny_unknown_fields` nowhere — unknown fields are silently ignored,
/// so this struct survives minor schema additions without breaking.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StatusPayload {
    pub model: Model,
    pub cost: Cost,
    pub context_window: ContextWindow,
    pub session_id: String,
    pub version: String,
    pub cwd: Option<String>,
    pub workspace: Option<Workspace>,
    pub output_style: Option<OutputStyle>,
    pub transcript_path: Option<String>,
    pub exceeds_200k_tokens: Option<bool>,
    pub vim: Option<Vim>,
    pub agent: Option<Agent>,
    pub worktree: Option<Worktree>,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Cost {
    pub total_cost_usd: f64,
    #[serde(default)]
    pub total_api_duration_ms: Option<u64>,
    #[serde(default)]
    pub total_duration_ms: Option<u64>,
    #[serde(default)]
    pub total_lines_added: Option<u64>,
    #[serde(default)]
    pub total_lines_removed: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ContextWindow {
    pub used_percentage: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub context_window_size: u64,
    #[serde(default)]
    pub remaining_percentage: Option<f64>,
    pub current_usage: Option<CurrentUsage>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CurrentUsage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Workspace {
    pub current_dir: Option<String>,
    pub project_dir: Option<String>,
    pub added_dirs: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OutputStyle {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Vim {
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Agent {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Worktree {
    pub name: Option<String>,
    pub path: Option<String>,
    pub branch: Option<String>,
    pub original_cwd: Option<String>,
    pub original_branch: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_real_payload() {
        let json = r#"{
            "model": {"id": "claude-opus-4-6", "display_name": "Opus 4.6"},
            "cost": {
                "total_cost_usd": 1.02,
                "total_api_duration_ms": 176079,
                "total_duration_ms": 360503,
                "total_lines_added": 60,
                "total_lines_removed": 2
            },
            "context_window": {
                "used_percentage": 30.0,
                "total_input_tokens": 11568,
                "total_output_tokens": 7957,
                "context_window_size": 200000,
                "remaining_percentage": 70,
                "current_usage": {
                    "cache_creation_input_tokens": 194,
                    "cache_read_input_tokens": 59770,
                    "input_tokens": 1,
                    "output_tokens": 239
                }
            },
            "session_id": "abc-123",
            "version": "2.1.72",
            "cwd": "/mnt/d/piam/devbox",
            "workspace": {
                "current_dir": "/mnt/d/piam/devbox",
                "project_dir": "/mnt/d/piam/devbox",
                "added_dirs": ["/workspace"]
            },
            "output_style": {"name": "ciso"},
            "transcript_path": ".data/foo.jsonl",
            "exceeds_200k_tokens": false
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p.model.display_name, "Opus 4.6");
        assert_eq!(p.model.id, "claude-opus-4-6");
        assert!((p.cost.total_cost_usd - 1.02).abs() < f64::EPSILON);
        assert_eq!(p.context_window.context_window_size, 200000);
        assert_eq!(p.context_window.total_input_tokens, 11568);
        let usage = p.context_window.current_usage.unwrap();
        assert_eq!(usage.cache_read_input_tokens, 59770);
        let ws = p.workspace.unwrap();
        assert_eq!(ws.project_dir.unwrap(), "/mnt/d/piam/devbox");
    }

    #[test]
    fn deserialize_minimal_payload() {
        let json = r#"{
            "model": {"id": "claude-opus-4-6", "display_name": "Opus"},
            "cost": {"total_cost_usd": 0.0},
            "context_window": {
                "used_percentage": 0.0,
                "total_input_tokens": 0,
                "total_output_tokens": 0,
                "context_window_size": 200000
            },
            "session_id": "def-456",
            "version": "2.1.72"
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        assert!(p.workspace.is_none());
        assert!(p.vim.is_none());
        assert!(p.cwd.is_none());
    }

    #[test]
    fn deserialize_with_agent() {
        let json = r#"{
            "model": {"id": "claude-opus-4-6", "display_name": "Opus"},
            "cost": {"total_cost_usd": 1.23},
            "context_window": {
                "used_percentage": 45.0,
                "total_input_tokens": 90000,
                "total_output_tokens": 5000,
                "context_window_size": 200000
            },
            "session_id": "ghi-789",
            "version": "2.1.72",
            "agent": {"name": "security-reviewer"}
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        let agent = p.agent.unwrap();
        assert_eq!(agent.name.unwrap(), "security-reviewer");
    }

    #[test]
    fn ignores_unknown_fields() {
        let json = r#"{
            "model": {"id": "claude-opus-4-6", "display_name": "Opus"},
            "cost": {"total_cost_usd": 0.0},
            "context_window": {
                "used_percentage": 0.0,
                "total_input_tokens": 0,
                "total_output_tokens": 0,
                "context_window_size": 200000
            },
            "session_id": "test",
            "version": "2.1.72",
            "some_future_field": true,
            "another_new_thing": {"nested": 42}
        }"#;
        let p: StatusPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p.session_id, "test");
    }
}
