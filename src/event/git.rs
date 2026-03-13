use std::process::Command;

use anyhow::Result;
use serde::Serialize;

#[derive(Serialize)]
struct GitStatus {
    branch: String,
    sha: String,
    sync: String,
    dirty: String,
}

fn run_git(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn gather() -> GitStatus {
    let branch = match run_git(&["rev-parse", "--abbrev-ref", "HEAD"]) {
        Some(b) if !b.is_empty() => b,
        _ => return GitStatus {
            branch: "???".into(),
            sha: String::new(),
            sync: String::new(),
            dirty: String::new(),
        },
    };

    // Short SHA
    let sha = run_git(&["rev-parse", "--short", "HEAD"]).unwrap_or_default();

    // Ahead/behind
    let sync = match run_git(&["rev-parse", "--abbrev-ref", "@{u}"]) {
        Some(upstream) if !upstream.is_empty() => {
            match run_git(&["rev-list", "--left-right", "--count", &format!("HEAD...{upstream}")]) {
                Some(lr) => {
                    let parts: Vec<&str> = lr.split_whitespace().collect();
                    if parts.len() == 2 {
                        let ahead: i64 = parts[0].parse().unwrap_or(0);
                        let behind: i64 = parts[1].parse().unwrap_or(0);
                        let mut ab = Vec::new();
                        if ahead > 0 { ab.push(format!("\u{2191}{ahead}")); }  // ↑
                        if behind > 0 { ab.push(format!("\u{2193}{behind}")); } // ↓
                        ab.join("")
                    } else {
                        String::new()
                    }
                }
                None => String::new(),
            }
        }
        _ => String::new(),
    };

    // Porcelain status
    let status_output = run_git(&["status", "--porcelain"]).unwrap_or_default();
    let mut modified = 0u32;
    let mut staged = 0u32;
    let mut untracked = 0u32;
    let mut deleted = 0u32;

    for line in status_output.lines() {
        let bytes = line.as_bytes();
        if bytes.len() < 2 { continue; }
        let (x, y) = (bytes[0], bytes[1]);
        if x == b'?' {
            untracked += 1;
        } else {
            if matches!(x, b'A' | b'M' | b'R' | b'C' | b'T') {
                staged += 1;
            }
            if x == b'D' {
                deleted += 1;
            }
            if matches!(y, b'M' | b'R' | b'T') {
                modified += 1;
            }
            if y == b'D' {
                deleted += 1;
            }
        }
    }

    // Symbols: ~ modified, + staged, - deleted, ? untracked
    let mut counts = Vec::new();
    if modified > 0 { counts.push(format!("~{modified}")); }
    if staged > 0 { counts.push(format!("+{staged}")); }
    if deleted > 0 { counts.push(format!("-{deleted}")); }
    if untracked > 0 { counts.push(format!("?{untracked}")); }

    let dirty = if counts.is_empty() {
        "\u{2728}".to_string()
    } else {
        counts.join(" ")
    };

    GitStatus { branch, sha, sync, dirty }
}

pub fn run() -> Result<()> {
    let data = gather();
    println!("{}", serde_json::to_string(&data)?);
    Ok(())
}
