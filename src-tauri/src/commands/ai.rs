use serde_json::{json, Value};

use crate::get_data_dir;

fn get_api_key(provider: &str) -> Option<String> {
    let keys_path = get_data_dir().join("api-keys.json");
    let content = std::fs::read_to_string(&keys_path).ok()?;
    let keys: Value = serde_json::from_str(&content).ok()?;
    let key_name = match provider {
        "claude" => "ANTHROPIC_API_KEY",
        "gemini" => "GEMINI_API_KEY",
        "openai" => "OPENAI_API_KEY",
        _ => return None,
    };
    keys.get(key_name)?.as_str().map(|s| s.to_string())
}

/// Check if messages contain referenced text and return system prompt if so
fn get_system_prompt(messages: &Value) -> Option<String> {
    let has_ref = messages.as_array()?.iter().any(|m| {
        let text = if let Some(content) = m.get("content") {
            if let Some(arr) = content.as_array() {
                arr.iter()
                    .filter_map(|b| b.get("text")?.as_str())
                    .collect::<Vec<_>>()
                    .join("")
            } else {
                content.as_str().unwrap_or("").to_string()
            }
        } else {
            String::new()
        };
        text.contains("[Referenced text:")
    });
    if has_ref {
        Some("When the user shares referenced text for proofreading, editing, or rewriting, always put your revised/rewritten version inside a markdown blockquote (lines starting with >). Keep your explanations outside the blockquote. This makes it easy for the user to identify and copy the rewritten text.".to_string())
    } else {
        None
    }
}

#[tauri::command]
pub async fn chat_with_ai(messages: Value, provider: String, model_id: String) -> Result<Value, String> {
    let api_key = get_api_key(&provider).ok_or("API key not found")?;
    let system_prompt = get_system_prompt(&messages);

    // Strip extra fields — only pass role + content to API
    let clean_messages: Vec<Value> = messages.as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|m| json!({
            "role": m.get("role").cloned().unwrap_or(json!("user")),
            "content": m.get("content").cloned().unwrap_or(json!(""))
        }))
        .collect();
    let clean = json!(clean_messages);

    match provider.as_str() {
        "claude" => {
            let text = chat_claude(&api_key, &clean, &model_id, system_prompt.as_deref()).await?;
            Ok(json!({ "success": true, "content": [{ "type": "text", "text": text }] }))
        }
        "openai" => {
            let text = chat_openai(&api_key, &clean, &model_id, system_prompt.as_deref()).await?;
            Ok(json!({ "success": true, "content": [{ "type": "text", "text": text }] }))
        }
        "gemini" => {
            let text = chat_gemini(&api_key, &clean, &model_id, system_prompt.as_deref()).await?;
            Ok(json!({ "success": true, "content": [{ "type": "text", "text": text }] }))
        }
        _ => Ok(json!({ "success": false, "error": format!("unknown provider: {}", provider) })),
    }
}

#[tauri::command]
pub async fn generate_title(messages: Value, provider: String, model_id: String) -> Result<Value, String> {
    // Append title generation prompt to existing conversation (same as Electron)
    let mut title_messages: Vec<Value> = messages.as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|m| json!({
            "role": m.get("role").cloned().unwrap_or(json!("user")),
            "content": m.get("content").cloned().unwrap_or(json!(""))
        }))
        .collect();
    title_messages.push(json!({
        "role": "user",
        "content": [{ "type": "text", "text": "Generate a very short title (3-6 words) for this conversation. Reply with ONLY the title, nothing else." }]
    }));

    let api_key = get_api_key(&provider).ok_or("API key not found")?;

    let result = match provider.as_str() {
        "claude" => chat_claude(&api_key, &json!(title_messages), &model_id, None).await,
        "openai" => chat_openai(&api_key, &json!(title_messages), &model_id, None).await,
        "gemini" => chat_gemini(&api_key, &json!(title_messages), &model_id, None).await,
        _ => Err(format!("unknown provider: {}", provider)),
    }?;

    let mut title = result.trim().trim_matches('"').to_string();
    // Truncate overly long titles (AI sometimes returns full sentences)
    if title.len() > 40 {
        title = title.chars().take(36).collect::<String>() + "...";
    }
    Ok(json!({ "success": true, "title": title }))
}

async fn chat_claude(api_key: &str, messages: &Value, model_id: &str, system: Option<&str>) -> Result<String, String> {
    let client = reqwest::Client::new();

    let mut body = json!({
        "model": model_id,
        "max_tokens": 4096,
        "messages": messages,
    });

    if let Some(sys) = system {
        body.as_object_mut().unwrap().insert("system".to_string(), json!(sys));
    }

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(error) = data.get("error") {
        return Err(error.get("message").and_then(|m| m.as_str()).unwrap_or("API error").to_string());
    }

    let text = data
        .get("content")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|b| b.get("text")?.as_str())
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    Ok(text)
}

async fn chat_openai(api_key: &str, messages: &Value, model_id: &str, system: Option<&str>) -> Result<String, String> {
    let client = reqwest::Client::new();

    let mut openai_messages: Vec<Value> = Vec::new();

    if let Some(sys) = system {
        openai_messages.push(json!({ "role": "system", "content": sys }));
    }

    // Convert messages to OpenAI format
    if let Some(arr) = messages.as_array() {
        for msg in arr {
            if let Some(content) = msg.get("content") {
                if let Some(blocks) = content.as_array() {
                    let mut parts = Vec::new();
                    for block in blocks {
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            parts.push(json!({ "type": "text", "text": text }));
                        } else if block.get("type").and_then(|t| t.as_str()) == Some("image") {
                            if let Some(source) = block.get("source") {
                                let media_type = source.get("media_type").and_then(|m| m.as_str()).unwrap_or("image/jpeg");
                                let data = source.get("data").and_then(|d| d.as_str()).unwrap_or("");
                                parts.push(json!({
                                    "type": "image_url",
                                    "image_url": { "url": format!("data:{};base64,{}", media_type, data) }
                                }));
                            }
                        }
                    }
                    openai_messages.push(json!({
                        "role": msg.get("role").and_then(|r| r.as_str()).unwrap_or("user"),
                        "content": parts
                    }));
                } else {
                    openai_messages.push(json!({
                        "role": msg.get("role").and_then(|r| r.as_str()).unwrap_or("user"),
                        "content": content
                    }));
                }
            }
        }
    }

    let body = json!({
        "model": model_id,
        "max_tokens": 4096,
        "messages": openai_messages,
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(error) = data.get("error") {
        return Err(error.get("message").and_then(|m| m.as_str()).unwrap_or("API error").to_string());
    }

    let text = data
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    Ok(text)
}

async fn chat_gemini(api_key: &str, messages: &Value, model_id: &str, system: Option<&str>) -> Result<String, String> {
    let client = reqwest::Client::new();

    // Build Gemini format: history + last message
    let mut history: Vec<Value> = Vec::new();
    if let Some(arr) = messages.as_array() {
        for msg in arr {
            let role = match msg.get("role").and_then(|r| r.as_str()).unwrap_or("user") {
                "assistant" => "model",
                _ => "user",
            };
            let mut parts = Vec::new();
            if let Some(content) = msg.get("content") {
                if let Some(blocks) = content.as_array() {
                    for block in blocks {
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            parts.push(json!({ "text": text }));
                        } else if block.get("type").and_then(|t| t.as_str()) == Some("image") {
                            if let Some(source) = block.get("source") {
                                parts.push(json!({
                                    "inlineData": {
                                        "data": source.get("data").and_then(|d| d.as_str()).unwrap_or(""),
                                        "mimeType": source.get("media_type").and_then(|m| m.as_str()).unwrap_or("image/jpeg")
                                    }
                                }));
                            }
                        }
                    }
                } else if let Some(text) = content.as_str() {
                    parts.push(json!({ "text": text }));
                }
            }
            history.push(json!({ "role": role, "parts": parts }));
        }
    }

    let mut body = json!({
        "contents": history,
    });

    if let Some(sys) = system {
        body.as_object_mut().unwrap().insert(
            "systemInstruction".to_string(),
            json!({ "parts": [{ "text": sys }] }),
        );
    }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model_id, api_key
    );

    let resp = client
        .post(&url)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(error) = data.get("error") {
        return Err(error.get("message").and_then(|m| m.as_str()).unwrap_or("API error").to_string());
    }

    let text = data
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    Ok(text)
}
