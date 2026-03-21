use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::error::AppError;

pub fn state_dir() -> PathBuf {
    let base = std::env::var("XDG_STATE_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").expect("HOME not set");
        format!("{home}/.local/state")
    });
    PathBuf::from(base).join("slacli")
}

fn log_path(log_type: &str) -> PathBuf {
    state_dir().join("logs").join(format!("{log_type}.jsonl"))
}

pub fn append_log(log_type: &str, entry: &serde_json::Value) -> Result<(), AppError> {
    let path = log_path(log_type);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::Io(format!("Failed to create {}: {e}", parent.display())))?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| AppError::Io(format!("Failed to open {}: {e}", path.display())))?;
    let line = serde_json::to_string(entry)
        .map_err(|e| AppError::Io(format!("Failed to serialize log entry: {e}")))?;
    writeln!(file, "{line}")
        .map_err(|e| AppError::Io(format!("Failed to write to {}: {e}", path.display())))?;
    Ok(())
}

pub fn print_logs(log_type: &str) -> Result<(), AppError> {
    let path = log_path(log_type);
    if !path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| AppError::Io(format!("Failed to read {}: {e}", path.display())))?;
    print!("{content}");
    Ok(())
}

pub fn remove_log_entry(log_type: &str, channel: &str, ts: &str) -> Result<(), AppError> {
    let path = log_path(log_type);
    if !path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| AppError::Io(format!("Failed to read {}: {e}", path.display())))?;
    let filtered: Vec<&str> = content
        .lines()
        .filter(|line| {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                !(v.get("channel").and_then(|c| c.as_str()) == Some(channel)
                    && v.get("ts").and_then(|t| t.as_str()) == Some(ts))
            } else {
                true
            }
        })
        .collect();
    if filtered.is_empty() {
        fs::remove_file(&path)
            .map_err(|e| AppError::Io(format!("Failed to remove {}: {e}", path.display())))?;
    } else {
        let mut out = filtered.join("\n");
        out.push('\n');
        fs::write(&path, out)
            .map_err(|e| AppError::Io(format!("Failed to write {}: {e}", path.display())))?;
    }
    Ok(())
}

pub fn clear_logs(log_type: &str) -> Result<(), AppError> {
    let path = log_path(log_type);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| AppError::Io(format!("Failed to remove {}: {e}", path.display())))?;
    }
    Ok(())
}
