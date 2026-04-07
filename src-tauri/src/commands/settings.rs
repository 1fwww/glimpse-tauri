use serde_json::{json, Value};
use std::fs;

use crate::get_data_dir;

fn keys_path() -> std::path::PathBuf {
    get_data_dir().join("api-keys.json")
}

fn prefs_path() -> std::path::PathBuf {
    get_data_dir().join("preferences.json")
}

// ── API Keys ──

#[tauri::command]
pub fn get_api_keys() -> Result<Value, String> {
    let path = keys_path();
    if !path.exists() {
        return Ok(json!({
            "ANTHROPIC_API_KEY": "",
            "GEMINI_API_KEY": "",
            "OPENAI_API_KEY": "",
            "hasAnyKey": false,
            "isInvite": false
        }));
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let keys: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let mask = |key_name: &str| -> String {
        keys.get(key_name)
            .and_then(|v| v.as_str())
            .filter(|k| !k.is_empty())
            .map(|k| format!("••••{}", &k[k.len().saturating_sub(4)..]))
            .unwrap_or_default()
    };

    let anthropic = mask("ANTHROPIC_API_KEY");
    let gemini = mask("GEMINI_API_KEY");
    let openai = mask("OPENAI_API_KEY");
    let has_any = !anthropic.is_empty() || !gemini.is_empty() || !openai.is_empty();
    let is_invite = keys.get("_invite").is_some();

    Ok(json!({
        "ANTHROPIC_API_KEY": anthropic,
        "GEMINI_API_KEY": gemini,
        "OPENAI_API_KEY": openai,
        "hasAnyKey": has_any,
        "isInvite": is_invite
    }))
}

#[tauri::command]
pub async fn save_api_keys(keys: Value) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let mut errors: Vec<String> = Vec::new();

    // Validate Anthropic key
    if let Some(key) = keys.get("ANTHROPIC_API_KEY").and_then(|v| v.as_str()).filter(|k| !k.is_empty()) {
        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&json!({
                "model": "claude-haiku-4-5-20251001",
                "max_tokens": 10,
                "messages": [{"role": "user", "content": "hi"}]
            }))
            .send()
            .await;
        if let Ok(r) = resp {
            if r.status().as_u16() == 401 {
                errors.push("Anthropic key is invalid".to_string());
            }
        }
    }

    // Validate OpenAI key
    if let Some(key) = keys.get("OPENAI_API_KEY").and_then(|v| v.as_str()).filter(|k| !k.is_empty()) {
        let resp = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", key))
            .header("content-type", "application/json")
            .json(&json!({
                "model": "gpt-4o-mini",
                "max_tokens": 10,
                "messages": [{"role": "user", "content": "hi"}]
            }))
            .send()
            .await;
        if let Ok(r) = resp {
            if r.status().as_u16() == 401 {
                errors.push("OpenAI key is invalid".to_string());
            }
        }
    }

    // Validate Gemini key
    if let Some(key) = keys.get("GEMINI_API_KEY").and_then(|v| v.as_str()).filter(|k| !k.is_empty()) {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            key
        );
        let resp = client
            .post(&url)
            .header("content-type", "application/json")
            .json(&json!({
                "contents": [{"parts": [{"text": "hi"}]}]
            }))
            .send()
            .await;
        if let Ok(r) = resp {
            let status = r.status().as_u16();
            if status == 400 || status == 403 {
                errors.push("Gemini key is invalid".to_string());
            }
        }
    }

    if !errors.is_empty() {
        return Ok(json!({
            "success": false,
            "error": format!("{}. Please check and try again.", errors.join(". "))
        }));
    }

    // Merge into existing keys
    let path = keys_path();
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;

    let mut current: Value = if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };

    if let (Some(obj), Some(new_keys)) = (current.as_object_mut(), keys.as_object()) {
        for (k, v) in new_keys {
            if let Some(s) = v.as_str() {
                if !s.is_empty() {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }
    }

    fs::write(&path, serde_json::to_string_pretty(&current).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(json!({ "success": true }))
}

#[tauri::command]
pub fn delete_api_key(provider: String) -> Result<(), String> {
    let path = keys_path();
    if !path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut keys: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    if provider == "invite" {
        // Remove invite flag and all invite-provided keys
        if let Some(obj) = keys.as_object_mut() {
            obj.remove("_invite");
            obj.remove("ANTHROPIC_API_KEY");
            obj.remove("GEMINI_API_KEY");
            obj.remove("OPENAI_API_KEY");
        }
    } else {
        let key_name = match provider.as_str() {
            "anthropic" => "ANTHROPIC_API_KEY",
            "gemini" => "GEMINI_API_KEY",
            "openai" => "OPENAI_API_KEY",
            _ => return Err(format!("unknown provider: {}", provider)),
        };
        if let Some(obj) = keys.as_object_mut() {
            obj.remove(key_name);
        }
    }
    fs::write(&path, serde_json::to_string_pretty(&keys).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_available_providers() -> Result<Value, String> {
    let keys = get_api_keys().unwrap_or(json!({}));
    let mut providers = Vec::new();

    if keys.get("ANTHROPIC_API_KEY").and_then(|v| v.as_str()).map_or(false, |k| !k.is_empty()) {
        providers.push(json!({
            "id": "claude",
            "name": "Claude",
            "models": [
                { "id": "claude-haiku-4-5-20251001", "name": "Haiku 4.5" },
                { "id": "claude-sonnet-4-20250514", "name": "Sonnet 4" }
            ]
        }));
    }

    if keys.get("GEMINI_API_KEY").and_then(|v| v.as_str()).map_or(false, |k| !k.is_empty()) {
        providers.push(json!({
            "id": "gemini",
            "name": "Gemini",
            "models": [
                { "id": "gemini-2.5-flash", "name": "2.5 Flash" },
                { "id": "gemini-2.5-pro", "name": "2.5 Pro" }
            ]
        }));
    }

    if keys.get("OPENAI_API_KEY").and_then(|v| v.as_str()).map_or(false, |k| !k.is_empty()) {
        providers.push(json!({
            "id": "openai",
            "name": "OpenAI",
            "models": [
                { "id": "gpt-4o-mini", "name": "GPT-4o Mini" },
                { "id": "gpt-4o", "name": "GPT-4o" }
            ]
        }));
    }

    Ok(json!(providers))
}

#[tauri::command]
pub fn validate_invite_code(code: String) -> Result<Value, String> {
    if code.trim() != "KPIMG" {
        return Ok(json!({ "valid": false, "error": "Invalid invite code" }));
    }

    // Save built-in keys for invite users (baked in at compile time)
    let mut keys = json!({ "_invite": true });
    if let Some(obj) = keys.as_object_mut() {
        if let Some(k) = option_env!("GLIMPSE_ANTHROPIC_KEY") { obj.insert("ANTHROPIC_API_KEY".into(), json!(k)); }
        if let Some(k) = option_env!("GLIMPSE_OPENAI_KEY") { obj.insert("OPENAI_API_KEY".into(), json!(k)); }
        if let Some(k) = option_env!("GLIMPSE_GEMINI_KEY") { obj.insert("GEMINI_API_KEY".into(), json!(k)); }
    }

    let path = keys_path();
    fs::create_dir_all(path.parent().unwrap()).ok();
    fs::write(&path, serde_json::to_string_pretty(&keys).unwrap_or_default()).ok();

    Ok(json!({ "valid": true, "success": true }))
}

// ── Preferences ──

#[tauri::command]
pub fn get_preferences() -> Result<Value, String> {
    let path = prefs_path();
    if !path.exists() {
        return Ok(json!({
            "launchAtLogin": false,
            "saveLocation": "ask",
            "savePath": ""
        }));
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_preference(key: String, value: Value) -> Result<Value, String> {
    let path = prefs_path();
    let mut prefs = get_preferences().unwrap_or(json!({}));
    let is_launch = key == "launchAtLogin";
    let launch_val = if is_launch { value.as_bool() } else { None };
    if let Some(obj) = prefs.as_object_mut() {
        obj.insert(key, value);
    }
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(&path, serde_json::to_string_pretty(&prefs).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    if let Some(enabled) = launch_val {
        set_launch_at_login(enabled);
    }
    Ok(json!({ "success": true }))
}

fn set_launch_at_login(enabled: bool) {
    let script = if enabled {
        r#"tell application "System Events" to make login item at end with properties {path:"/Applications/Glimpse.app", hidden:false}"#
    } else {
        r#"tell application "System Events" to delete login item "Glimpse""#
    };
    let _ = std::process::Command::new("osascript")
        .args(["-e", script])
        .spawn();
}
