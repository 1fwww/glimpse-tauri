/// Get cursor position and find target monitor info
fn get_display_info() -> Result<serde_json::Value, String> {
    let monitors = xcap::Monitor::all().map_err(|e| format!("Failed to list monitors: {}", e))?;
    if monitors.is_empty() {
        return Err("No monitors found".to_string());
    }

    let cursor_pos = {
        let source = core_graphics::event_source::CGEventSource::new(
            core_graphics::event_source::CGEventSourceStateID::CombinedSessionState,
        ).map_err(|_| "Failed to create event source".to_string())?;
        core_graphics::event::CGEvent::new(source)
            .map_err(|_| "Failed to create CGEvent".to_string())?
            .location()
    };

    let target = monitors
        .iter()
        .find(|m| {
            let mx = m.x().unwrap_or(0) as f64;
            let my = m.y().unwrap_or(0) as f64;
            let mw = m.width().unwrap_or(0) as f64;
            let mh = m.height().unwrap_or(0) as f64;
            cursor_pos.x >= mx && cursor_pos.x < mx + mw && cursor_pos.y >= my && cursor_pos.y < my + mh
        })
        .unwrap_or(&monitors[0]);

    Ok(serde_json::json!({
        "width": target.width().unwrap_or(1920),
        "height": target.height().unwrap_or(1080),
        "x": target.x().unwrap_or(0),
        "y": target.y().unwrap_or(0),
    }))
}

/// Fast screen capture using macOS native screencapture command.
/// Returns (file_path, display_info).
pub fn capture_screen_to_file() -> Result<(String, serde_json::Value), String> {
    let display_info = get_display_info()?;
    let temp_path = std::env::temp_dir().join("glimpse-capture.jpg");
    let path_str = temp_path.to_string_lossy().to_string();

    // Use macOS native screencapture — much faster than xcap + image crate
    let output = std::process::Command::new("screencapture")
        .args(["-x", "-t", "jpg", "-C", &path_str])
        .output()
        .map_err(|e| format!("screencapture failed: {}", e))?;

    if !output.status.success() {
        return Err("screencapture returned non-zero".to_string());
    }

    Ok((path_str, display_info))
}
