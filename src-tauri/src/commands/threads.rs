use serde_json::Value;
use std::fs;

use crate::get_data_dir;

fn threads_dir() -> std::path::PathBuf {
    get_data_dir().join("threads")
}

#[tauri::command]
pub fn get_threads() -> Result<Vec<Value>, String> {
    let dir = threads_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut threads: Vec<Value> = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(thread) = serde_json::from_str::<Value>(&content) {
                    threads.push(thread);
                }
            }
        }
    }

    // Sort by updatedAt descending
    threads.sort_by(|a, b| {
        let a_time = a.get("updatedAt").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let b_time = b.get("updatedAt").and_then(|v| v.as_f64()).unwrap_or(0.0);
        b_time.partial_cmp(&a_time).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit to 5 most recent
    threads.truncate(5);
    Ok(threads)
}

#[tauri::command]
pub fn save_thread(thread: Value) -> Result<(), String> {
    let dir = threads_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let id = thread.get("id").and_then(|v| v.as_str()).ok_or("missing thread id")?;
    let path = dir.join(format!("{}.json", id));
    let content = serde_json::to_string_pretty(&thread).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;

    // Prune old threads — keep only 5 most recent
    prune_threads();
    Ok(())
}

#[tauri::command]
pub fn delete_thread(id: String) -> Result<(), String> {
    let path = threads_dir().join(format!("{}.json", id));
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn prune_threads() {
    let dir = threads_dir();
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .collect();

    if entries.len() <= 5 {
        return;
    }

    // Sort by modified time, newest first
    entries.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        b_time.cmp(&a_time)
    });

    // Remove everything after the 5th
    for entry in entries.into_iter().skip(5) {
        fs::remove_file(entry.path()).ok();
    }
}
