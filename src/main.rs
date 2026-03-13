mod config;
mod event;
mod events;
mod format;
mod meter;
mod payload;
mod ratelimit;
mod template;

use std::collections::HashMap;
use std::io::Read;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};

use payload::StatusPayload;

const VERSION: &str = env!("BUILD_VERSION");
const DEFAULT_STATE_DIR: &str = ".data/statusline";

#[derive(Parser)]
#[command(name = "3am-statusline", about = "Configurable status display for Claude Code")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an event subcommand
    Event {
        #[command(subcommand)]
        kind: EventKind,
    },
}

#[derive(Subcommand)]
enum EventKind {
    /// Git branch, ahead/behind, dirty counts
    Git,
    /// World clock times
    Time,
    /// CPU/memory stats
    Sys,
    /// Claude API status
    Status,
    /// Check latest Claude Code version from npm
    Version,
    /// Current weather conditions
    Weather {
        /// US zip code for location
        #[arg(long)]
        zip: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Event { kind }) => {
            let result = match kind {
                EventKind::Git => event::git::run(),
                EventKind::Time => event::time::run(),
                EventKind::Sys => event::sys::run(),
                EventKind::Status => event::status::run(),
                EventKind::Version => event::version::run(),
                EventKind::Weather { ref zip } => event::weather::run(zip),
            };
            if let Err(e) = result {
                eprintln!("statusline: {e:#}");
                std::process::exit(1);
            }
        }
        None => {
            if let Err(e) = render() {
                let view_path = format!("{DEFAULT_STATE_DIR}/last_view.txt");
                if let Ok(cached) = std::fs::read_to_string(&view_path) {
                    print!("{cached}");
                }
                eprintln!("statusline: {e:#}");
                std::process::exit(1);
            }
        }
    }
}

fn render() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("reading stdin")?;

    let cfg = config::load().context("loading config")?;
    let state_dir = cfg.as_ref().map(|c| c.state_dir()).unwrap_or(DEFAULT_STATE_DIR);
    let view_cache_path = format!("{state_dir}/last_view.txt");

    // Empty stdin → serve cached view
    if input.trim().is_empty() {
        if let Ok(cached) = std::fs::read_to_string(&view_cache_path) {
            print!("{cached}");
        }
        return Ok(());
    }

    // Log raw payload before deserialization (fire-and-forget)
    if let Some(json_cfg) = cfg.as_ref().and_then(|c| c.logging.as_ref()).and_then(|l| l.json.as_ref()) {
        let _ = write_log(&input, &json_cfg.dir, json_cfg.keep);
    }

    let payload: StatusPayload =
        serde_json::from_str(&input).context("parsing JSON payload")?;
    let use_color = meter::should_use_color();
    let config_columns = cfg.as_ref().and_then(|c| c.columns);
    let width = template::terminal_width(config_columns);

    let meter_config = cfg
        .as_ref()
        .map(|c| c.to_meter_config())
        .unwrap_or_default();

    let theme = cfg
        .as_ref()
        .map(|c| c.theme.clone())
        .unwrap_or_default();

    let lines = cfg
        .as_ref()
        .and_then(|c| {
            let t = c.line_templates();
            if t.is_empty() { None } else { Some(t) }
        })
        .unwrap_or_else(config::default_lines);

    let (mut context, mut strings) = build_context(&payload);

    // Discover active plan slug from .claude/plans/
    if let Some(slug) = discover_plan_slug() {
        strings.insert("plan.slug".into(), slug);
    }

    // Load cached rate limit data
    let rl_cache = ratelimit::load_cached(state_dir);
    let rl_stale = ratelimit::is_stale(&rl_cache);
    if let Some(ref cache) = rl_cache {
        ratelimit::inject_fields(cache, &mut context, &mut strings);
    }

    // Load cached event data and inject into template context
    let event_configs = cfg.as_ref().map(|c| &c.events[..]).unwrap_or(&[]);
    let mut event_cache = events::load_cache(state_dir).unwrap_or_default();
    if !event_configs.is_empty() {
        events::inject_fields(&event_cache, &mut context, &mut strings);
    }

    // Version check: compare payload version against latest from event cache
    if let Some(latest) = strings.get("event.version.latest") {
        if event::version::is_current(&payload.version, latest) {
            strings.insert("version.ok".into(), "\u{2713}".into());
            strings.insert("version.outdated".into(), String::new());
        } else {
            strings.insert("version.ok".into(), String::new());
            strings.insert("version.outdated".into(), "\u{21e1}".into());
        }
    }

    // Process color fields: threshold-based coloring for inline values
    // Also build meter_overrides so {meter:field} uses matching thresholds
    let mut meter_overrides = HashMap::new();
    if let Some(ref cfg) = cfg {
        for cf in &cfg.color_fields {
            meter_overrides.insert(cf.source.clone(), (cf.yellow, cf.red));
            if let Some(&val) = context.get(&cf.source) {
                let formatted = cf.format.as_deref()
                    .map(|f| format::apply(val, f))
                    .unwrap_or_else(|| val.to_string());
                let (g, y, r) = if val < cf.yellow {
                    (formatted.clone(), String::new(), String::new())
                } else if val < cf.red {
                    (String::new(), formatted.clone(), String::new())
                } else {
                    (String::new(), String::new(), formatted)
                };
                strings.insert(format!("{}_green", cf.name), g);
                strings.insert(format!("{}_yellow", cf.name), y);
                strings.insert(format!("{}_red", cf.name), r);
            }
        }
    }

    let mut output = String::new();
    for (i, (left_tpl, right_tpl)) in lines.iter().enumerate() {
        let left = template::resolve(left_tpl, &context, &strings, &meter_config, use_color, &theme, &meter_overrides);
        let right = template::resolve(right_tpl, &context, &strings, &meter_config, use_color, &theme, &meter_overrides);
        let line = template::pad_line(&left, &right, width);
        // Preserve blank lines using zero-width space (U+200B)
        // NBSP (U+00A0) gets stripped by JavaScript's trim()
        if line.chars().all(|c| c == ' ') {
            output.push('\u{200B}');
        } else {
            output.push_str(&line);
        }
        if i < lines.len() - 1 {
            output.push('\n');
        }
    }

    // Render to stdout first (latency-critical path)
    print!("{output}");

    // Cache the rendered view for fallback on next failed invocation
    let _ = cache_view(&output, &view_cache_path);

    // Write compact event log (fire-and-forget, after stdout)
    if let Some(logging) = cfg.as_ref().and_then(|c| c.logging.as_ref()) {
        let _ = write_event_log(&payload, &context, &strings, &logging.file);
    }

    // Fire due events and persist cache (after stdout)
    if !event_configs.is_empty() {
        let due = events::due_events(event_configs, &event_cache);
        if !due.is_empty() {
            events::fire(&due, &mut event_cache);
            events::save_cache(&event_cache, state_dir);
        }
    }

    // Refresh rate limit cache if stale (runs curl, ~500ms, after stdout)
    if rl_stale {
        ratelimit::refresh(state_dir);
    }

    Ok(())
}

fn cache_view(output: &str, view_cache_path: &str) -> Result<()> {
    let path = std::path::Path::new(view_cache_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("creating cache directory")?;
    }
    std::fs::write(path, output).context("writing view cache")?;
    Ok(())
}

const EVENT_LOG_MAX_LINES: usize = 100;

fn write_event_log(
    payload: &StatusPayload,
    nums: &HashMap<String, f64>,
    strings: &HashMap<String, String>,
    log_file: &str,
) -> Result<()> {
    let log_path = std::path::Path::new(log_file);
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).context("creating log directory")?;
    }

    let now = Utc::now();
    let ts = now.format("%Y-%m-%d %H:%M:%S");

    let model = &payload.model.display_name;
    let cost = format::apply(payload.cost.total_cost_usd, "currency");
    let ctx_pct = format::apply(payload.context_window.used_percentage, "pct");
    let in_tok = format::apply(payload.context_window.total_input_tokens as f64, "tokens");
    let out_tok = format::apply(payload.context_window.total_output_tokens as f64, "tokens");

    let mut parts = vec![format!(
        "{ts} | ctx: {ctx_pct} | cost: {cost} | {in_tok} in / {out_tok} out | {model}"
    )];

    // Rate limit info if available
    if let (Some(&rl5h), Some(&rl7d)) = (nums.get("ratelimit.5h"), nums.get("ratelimit.7d")) {
        let eta_5h = strings.get("ratelimit.5h_eta").map(|s| s.as_str()).unwrap_or("?");
        let eta_7d = strings.get("ratelimit.7d_eta").map(|s| s.as_str()).unwrap_or("?");
        parts.push(format!(
            "rl: {}% 5h ({}), {}% 7d ({})",
            rl5h.round() as i64,
            eta_5h,
            rl7d.round() as i64,
            eta_7d,
        ));
    }

    // Plan slug if present
    if let Some(slug) = strings.get("plan.slug") {
        parts.push(format!("plan: {slug}"));
    }

    let line = parts.join(" | ");

    // Read existing lines, prepend new line, truncate
    let existing = std::fs::read_to_string(log_path).unwrap_or_default();
    let lines: Vec<&str> = std::iter::once(line.as_str())
        .chain(existing.lines())
        .take(EVENT_LOG_MAX_LINES)
        .collect();
    // Ensure trailing newline
    let content = lines.join("\n") + "\n";

    std::fs::write(log_path, content).context("writing event log")?;
    Ok(())
}

fn write_log(raw_input: &str, dir: &str, keep: usize) -> Result<()> {
    let dir_path = std::path::Path::new(dir);
    std::fs::create_dir_all(dir_path).context("creating log directory")?;

    let now = Utc::now();
    let filename = format!("{}-{}.json", now.format("%Y%m%d"), now.timestamp());
    let file_path = dir_path.join(filename);

    // Pretty-print the raw JSON
    let value: serde_json::Value =
        serde_json::from_str(raw_input).context("parsing JSON for logging")?;
    let pretty = serde_json::to_string_pretty(&value).context("pretty-printing JSON")?;
    std::fs::write(&file_path, pretty).context("writing log file")?;

    // Prune old files beyond keep limit
    let mut entries: Vec<_> = std::fs::read_dir(dir_path)
        .context("reading log directory")?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();

    if entries.len() > keep {
        entries.sort_by_key(|e| e.file_name());
        let to_remove = entries.len() - keep;
        for entry in entries.into_iter().take(to_remove) {
            let _ = std::fs::remove_file(entry.path());
        }
    }

    Ok(())
}

/// Find the most recently modified .md file in .claude/plans/ (top-level only).
fn discover_plan_slug() -> Option<String> {
    let plans_dir = std::path::Path::new(".claude/plans");
    let entries = std::fs::read_dir(plans_dir).ok()?;
    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.is_file() && path.extension().map(|x| x == "md").unwrap_or(false)
        })
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .and_then(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str().map(String::from))
        })
}

fn build_context(payload: &StatusPayload) -> (HashMap<String, f64>, HashMap<String, String>) {
    let mut nums = HashMap::new();
    let mut strs = HashMap::new();

    // Model
    strs.insert("model.display_name".into(), payload.model.display_name.clone());
    strs.insert("model.id".into(), payload.model.id.clone());

    // Cost
    nums.insert("cost.total_cost_usd".into(), payload.cost.total_cost_usd);
    if let Some(ms) = payload.cost.total_duration_ms {
        nums.insert("cost.total_duration_secs".into(), ms as f64 / 1000.0);
    }
    if let Some(n) = payload.cost.total_lines_added {
        nums.insert("cost.total_lines_added".into(), n as f64);
    }
    if let Some(n) = payload.cost.total_lines_removed {
        nums.insert("cost.total_lines_removed".into(), n as f64);
    }

    // Context window
    nums.insert("context_window.used_percentage".into(), payload.context_window.used_percentage);
    nums.insert("context_window.total_input_tokens".into(), payload.context_window.total_input_tokens as f64);
    nums.insert("context_window.total_output_tokens".into(), payload.context_window.total_output_tokens as f64);
    nums.insert("context_window.context_window_size".into(), payload.context_window.context_window_size as f64);
    if let Some(pct) = payload.context_window.remaining_percentage {
        nums.insert("context_window.remaining_percentage".into(), pct);
    }
    if let Some(ref cu) = payload.context_window.current_usage {
        nums.insert("current_usage.input_tokens".into(), cu.input_tokens as f64);
        nums.insert("current_usage.output_tokens".into(), cu.output_tokens as f64);
        nums.insert("current_usage.cache_creation_input_tokens".into(), cu.cache_creation_input_tokens as f64);
        nums.insert("current_usage.cache_read_input_tokens".into(), cu.cache_read_input_tokens as f64);
    }

    // Session
    strs.insert("session_id".into(), payload.session_id.clone());
    strs.insert("version".into(), payload.version.clone());
    strs.insert("build_version".into(), VERSION.to_string());

    // Optional fields
    if let Some(ref ws) = payload.workspace {
        if let Some(ref dir) = ws.project_dir {
            strs.insert("workspace.project_dir".into(), dir.clone());
        }
        if let Some(ref dir) = ws.current_dir {
            strs.insert("workspace.current_dir".into(), dir.clone());
        }
    }
    if let Some(ref cwd) = payload.cwd {
        strs.insert("cwd".into(), cwd.clone());
    }
    if let Some(ref vim) = payload.vim {
        if let Some(ref mode) = vim.mode {
            strs.insert("vim.mode".into(), mode.clone());
        }
    }
    if let Some(ref agent) = payload.agent {
        if let Some(ref name) = agent.name {
            strs.insert("agent.name".into(), name.clone());
        }
    }
    if let Some(ref wt) = payload.worktree {
        if let Some(ref name) = wt.name {
            strs.insert("worktree.name".into(), name.clone());
        }
    }

    (nums, strs)
}
