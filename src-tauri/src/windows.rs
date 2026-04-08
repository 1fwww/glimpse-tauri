use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::get_data_dir;

static SETTINGS_TOGGLING: AtomicBool = AtomicBool::new(false);
pub static CHAT_READY: AtomicBool = AtomicBool::new(false);

// ── Window creation ──

pub fn create_home_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(w) = app.get_webview_window("home") {
        w.set_focus()?;
        return Ok(());
    }
    let win = WebviewWindowBuilder::new(app, "home", WebviewUrl::App("index.html#home".into()))
        .title("Glimpse")
        .inner_size(380.0, 560.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .accept_first_mouse(true)
        .build()?;
    let w = win.clone();
    let _ = win.run_on_main_thread(move || {
        crate::native_mac::set_transparent_background(&w);
    });
    if crate::IS_ACCESSORY.load(std::sync::atomic::Ordering::SeqCst) {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        crate::IS_ACCESSORY.store(false, std::sync::atomic::Ordering::SeqCst);
    }
    Ok(())
}

pub fn create_welcome_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Center on screen
    let (x, y) = app.primary_monitor().ok().flatten()
        .map(|m| {
            let s = m.scale_factor();
            let sw = m.size().width as f64 / s;
            let sh = m.size().height as f64 / s;
            ((sw - 400.0) / 2.0, (sh - 510.0) / 2.0)
        })
        .unwrap_or((500.0, 300.0));

    let win = WebviewWindowBuilder::new(app, "welcome", WebviewUrl::App("index.html#welcome".into()))
        .title("Welcome to Glimpse")
        .inner_size(400.0, 510.0)
        .position(x, y)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .build()?;
    let w = win.clone();
    let _ = win.run_on_main_thread(move || {
        crate::native_mac::set_transparent_background(&w);
    });
    Ok(())
}

pub fn create_settings_window(app: &AppHandle, panel_bounds: Option<&serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
    // Don't create if already exists
    if app.get_webview_window("settings").is_some() {
        return Ok(());
    }

    let is_overlay_mode = app.get_webview_window("overlay")
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false);

    // Position beside parent window or use panel_bounds hint
    fn position_beside(px: f64, py: f64, pw: f64, mon_x: f64, mon_w: f64) -> (f64, f64) {
        let right_space = mon_x + mon_w - (px + pw);
        if right_space >= 440.0 {
            (px + pw + 20.0, py)
        } else {
            ((px - 440.0).max(mon_x), py)
        }
    }

    let (x, y) = if let Some(parent) = app.get_webview_window("chat").or_else(|| app.get_webview_window("home")) {
        // Standalone mode: position beside chat/home
        if let (Ok(pos), Ok(size)) = (parent.outer_position(), parent.outer_size()) {
            let scale = parent.current_monitor().ok().flatten().map(|m| m.scale_factor()).unwrap_or(2.0);
            let px = pos.x as f64 / scale;
            let py = pos.y as f64 / scale;
            let pw = size.width as f64 / scale;
            let (mon_x, mon_w) = parent.current_monitor().ok().flatten()
                .map(|m| (m.position().x as f64 / scale, m.size().width as f64 / scale))
                .unwrap_or((0.0, 1920.0));
            position_beside(px, py, pw, mon_x, mon_w)
        } else {
            (500.0, 100.0)
        }
    } else if let Some(hint) = panel_bounds {
        // Overlay mode: use panel bounds hint from frontend
        let hx = hint.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let hw = hint.get("w").and_then(|v| v.as_f64()).unwrap_or(380.0);
        // Get monitor info
        let (mon_x, mon_w, mon_h) = app.primary_monitor().ok().flatten()
            .map(|m| {
                let s = m.scale_factor();
                (m.position().x as f64 / s, m.size().width as f64 / s, m.size().height as f64 / s)
            })
            .unwrap_or((0.0, 1920.0, 1080.0));
        let centered_y = mon_h / 2.0 - 260.0;
        position_beside(hx, centered_y, hw, mon_x, mon_w)
    } else {
        (500.0, 100.0)
    };

    eprintln!("[Settings] positioning at ({}, {}), overlay_mode={}", x, y, is_overlay_mode);

    let win = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html#settings".into()))
        .title("Settings")
        .inner_size(420.0, 520.0)
        .position(x, y)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .visible(false)
        .build()?;

    // In overlay mode, settings must be on same level as overlay
    if is_overlay_mode {
        let _ = win.set_always_on_top(true);
    }
    let w = win.clone();
    let _ = win.run_on_main_thread(move || {
        crate::native_mac::set_transparent_background(&w);
        crate::native_mac::set_visible_on_fullscreen(&w, true);
        if is_overlay_mode {
            crate::native_mac::set_window_level_screen_saver(&w);
        }
    });

    let _ = win.show();
    let _ = win.set_focus();
    Ok(())
}

pub fn create_chat_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(w) = app.get_webview_window("chat") {
        w.set_focus()?;
        return Ok(());
    }
    let win = WebviewWindowBuilder::new(app, "chat", WebviewUrl::App("index.html#chat-only".into()))
        .title("Glimpse Chat")
        .inner_size(420.0, 400.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .accept_first_mouse(true)
        .build()?;

    let w = win.clone();
    let _ = win.run_on_main_thread(move || {
        crate::native_mac::set_transparent_background(&w);
    });
    Ok(())
}

pub fn create_overlay_window(app: &AppHandle, display_info: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    // Close existing overlay if any
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.close();
    }

    let x = display_info.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y = display_info.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let w = display_info.get("width").and_then(|v| v.as_f64()).unwrap_or(1920.0);
    let h = display_info.get("height").and_then(|v| v.as_f64()).unwrap_or(1080.0);

    let win = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("index.html".into()))
        .title("Glimpse Overlay")
        .position(x, y)
        .inner_size(w, h)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .accept_first_mouse(true)
        .visible(false)
        .build()?;

    // Set fullscreen behavior SYNCHRONOUSLY before showing
    let w2 = win.clone();
    let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let done2 = done.clone();
    let _ = win.run_on_main_thread(move || {
        crate::native_mac::set_visible_on_fullscreen(&w2, true);
        crate::native_mac::set_window_level_screen_saver(&w2);
        done2.store(true, std::sync::atomic::Ordering::Release);
    });
    for _ in 0..200 {
        if done.load(std::sync::atomic::Ordering::Acquire) { break; }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let _ = win.show();
    let _ = win.set_focus();

    Ok(())
}

/// Pre-warm an overlay window (hidden, full screen size) so screenshot is instant
pub fn prewarm_overlay(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if app.get_webview_window("overlay").is_some() {
        return Ok(()); // Already exists
    }
    // Use primary monitor size for pre-warm
    let (w, h) = if let Some(monitor) = app.primary_monitor().ok().flatten() {
        (monitor.size().width as f64, monitor.size().height as f64)
    } else {
        (1920.0, 1080.0)
    };
    WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("index.html".into()))
        .title("Glimpse Overlay")
        .position(0.0, 0.0)
        .inner_size(w, h)
        .decorations(false)
        .transparent(true)
        .visible(false) // Hidden until needed
        .skip_taskbar(true)
        .accept_first_mouse(true)
        .build()?;
    // Pre-set fullscreen behavior so it's ready when we show
    if let Some(win) = app.get_webview_window("overlay") {
        let w = win.clone();
        let _ = win.run_on_main_thread(move || {
            crate::native_mac::set_visible_on_fullscreen(&w, true);
            crate::native_mac::set_window_level_screen_saver(&w);
        });
    }
    Ok(())
}

// ── Window management commands ──

#[tauri::command]
pub fn close_home(app: AppHandle) {
    if let Some(w) = app.get_webview_window("home") {
        let _ = w.close();
    }
    // Close settings if it was opened from home
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.close();
    }
}

#[tauri::command]
pub fn close_welcome(app: AppHandle) {
    if let Some(w) = app.get_webview_window("welcome") {
        let _ = w.close();
    }
}

#[tauri::command]
pub fn close_overlay(app: AppHandle) {
    crate::native_mac::remove_esc_monitor();
    if let Some(w) = app.get_webview_window("overlay") {
        // Hide immediately — non-fullscreen can reuse this instantly
        // Don't emit reset-overlay here — avoids blank flash on reuse.
        // screen-captured event will overwrite state when shown again.
        let _ = w.set_always_on_top(false);
        let _ = w.hide();
        // After a delay, destroy and prewarm fresh (needed for fullscreen Space association)
        let app2 = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            if let Some(ow) = app2.get_webview_window("overlay") {
                if !ow.is_visible().unwrap_or(true) {
                    let _ = ow.close();
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    let _ = prewarm_overlay(&app2);
                }
            }
        });
    }
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.close();
    }
    let has_other = ["home", "chat", "welcome"].iter().any(|l| {
        app.get_webview_window(l).map(|w| w.is_visible().unwrap_or(false)).unwrap_or(false)
    });
    if !has_other {
        crate::native_mac::hide_app();
    } else if crate::IS_ACCESSORY.load(std::sync::atomic::Ordering::SeqCst) {
        crate::native_mac::hide_app();
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        crate::IS_ACCESSORY.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

#[tauri::command]
pub fn input_focus(app: AppHandle) {
    // Lower overlay to floating level so IME candidate windows can appear above
    if let Some(w) = app.get_webview_window("overlay") {
        let w2 = w.clone();
        let _ = w.run_on_main_thread(move || {
            crate::native_mac::set_window_level_floating(&w2);
        });
    }
}

#[tauri::command]
pub fn lower_overlay(app: AppHandle) {
    // Lower overlay below normal windows (for settings, save dialog, etc.)
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.set_always_on_top(false);
    }
}

#[tauri::command]
pub fn restore_overlay(app: AppHandle) {
    // Restore overlay to screen-saver level
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.set_always_on_top(true);
        let w2 = w.clone();
        let _ = w.run_on_main_thread(move || {
            crate::native_mac::set_window_level_screen_saver(&w2);
        });
        let _ = w.set_focus();
    }
}

/// Shared helper: close settings window, emit event, restore overlay if needed
fn close_settings_inner(app: &AppHandle) {
    let had_settings = app.get_webview_window("settings").is_some();
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.close();
    }
    if had_settings {
        let _ = app.emit("settings-closed", ());
        // If overlay is visible, restore its level and focus it
        if let Some(ow) = app.get_webview_window("overlay") {
            if ow.is_visible().unwrap_or(false) {
                let _ = ow.set_always_on_top(true);
                let ow2 = ow.clone();
                let _ = ow.run_on_main_thread(move || {
                    crate::native_mac::set_window_level_screen_saver(&ow2);
                });
                let _ = ow.set_focus();
            }
        }
    }
}

#[tauri::command]
pub fn close_settings(app: AppHandle) {
    close_settings_inner(&app);
}

#[tauri::command]
pub fn close_chat_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window("chat") {
        let _ = w.close();
    }
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.close();
    }
    // If no other visible windows, hide app first then restore policy
    let has_other = ["home", "overlay", "welcome"].iter().any(|l| {
        app.get_webview_window(l).map(|w| w.is_visible().unwrap_or(false)).unwrap_or(false)
    });
    if !has_other {
        crate::native_mac::hide_app();
    }
    // Restore Regular policy AFTER hide_app (restoring before causes Space switch)
    if !has_other && crate::IS_ACCESSORY.load(std::sync::atomic::Ordering::SeqCst) {
        // Don't restore here — let create_home_window do it to avoid Space switch
        crate::IS_ACCESSORY.store(false, std::sync::atomic::Ordering::SeqCst);
    } else if crate::IS_ACCESSORY.load(std::sync::atomic::Ordering::SeqCst) {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        crate::IS_ACCESSORY.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

#[tauri::command]
pub fn welcome_done(app: AppHandle) {
    let data_dir = get_data_dir();
    let onboarding_path = data_dir.join("onboarding-done");
    fs::write(&onboarding_path, chrono_now()).ok();

    // Get welcome window center so home opens at the same position
    let welcome_center = app.get_webview_window("welcome").and_then(|w| {
        let pos = w.outer_position().ok()?;
        let size = w.outer_size().ok()?;
        let scale = w.current_monitor().ok().flatten().map(|m| m.scale_factor()).unwrap_or(2.0);
        Some((
            pos.x as f64 / scale + size.width as f64 / scale / 2.0,
            pos.y as f64 / scale + size.height as f64 / scale / 2.0,
        ))
    });

    if let Some(w) = app.get_webview_window("welcome") {
        let _ = w.close();
    }

    // Position home centered on where welcome was
    if let Some((cx, cy)) = welcome_center {
        let home_w = 380.0;
        let home_h = 560.0;
        let x = cx - home_w / 2.0;
        let y = cy - home_h / 2.0;
        if let Ok(win) = WebviewWindowBuilder::new(&app, "home", WebviewUrl::App("index.html#home".into()))
            .title("Glimpse")
            .inner_size(home_w, home_h)
            .position(x, y)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .accept_first_mouse(true)
            .build()
        {
            let w = win.clone();
            let _ = win.run_on_main_thread(move || {
                crate::native_mac::set_transparent_background(&w);
            });
        }
    } else {
        let _ = create_home_window(&app);
    }
}

#[tauri::command]
pub fn open_settings(app: AppHandle, panel_bounds: Option<serde_json::Value>) {
    if let Err(e) = create_settings_window(&app, panel_bounds.as_ref()) {
        eprintln!("[Settings] failed to create: {}", e);
    }
}

#[tauri::command]
pub async fn toggle_settings(app: AppHandle, panel_bounds: Option<serde_json::Value>) {
    // Atomic flag — if already toggling, skip (prevents rapid double-click issues)
    if SETTINGS_TOGGLING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return;
    }

    if app.get_webview_window("settings").is_some() {
        close_settings_inner(&app);
        // Wait for window to fully destroy so next toggle sees it as gone
        for _ in 0..20 {
            if app.get_webview_window("settings").is_none() { break; }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    } else {
        let _ = create_settings_window(&app, panel_bounds.as_ref());
    }

    SETTINGS_TOGGLING.store(false, Ordering::SeqCst);
}

#[tauri::command]
pub fn open_thread_in_chat(app: AppHandle, thread_id: String) {
    if let Some(w) = app.get_webview_window("home") {
        let _ = w.close();
    }

    // Read full thread data from disk
    let thread_path = get_data_dir().join("threads").join(format!("{}.json", thread_id));
    let thread_data = std::fs::read_to_string(&thread_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());

    // Create chat hidden, send thread data, then show
    if app.get_webview_window("chat").is_none() {
        let _ = WebviewWindowBuilder::new(&app, "chat", WebviewUrl::App("index.html#chat-only".into()))
            .title("Glimpse Chat")
            .inner_size(420.0, 550.0)
            .resizable(true)
            .decorations(false)
            .transparent(true)
            .accept_first_mouse(true)
            .visible(false)
            .build();
        if let Some(win) = app.get_webview_window("chat") {
            let w = win.clone();
            let _ = win.run_on_main_thread(move || {
                crate::native_mac::set_transparent_background(&w);
                crate::native_mac::set_visible_on_fullscreen(&w, true);
            });
        }
    }
    let app_clone = app.clone();
    std::thread::spawn(move || {
        // Wait for chat frontend to signal ready
        CHAT_READY.store(false, Ordering::SeqCst);
        for _ in 0..80 {
            if CHAT_READY.load(Ordering::SeqCst) { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        if let Some(w) = app_clone.get_webview_window("chat") {
            if let Some(data) = thread_data {
                let _ = w.emit("load-thread-data", &data);
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
            let _ = w.show();
            let _ = w.set_focus();
        }
    });
}

#[tauri::command]
pub fn chat_ready(_app: AppHandle) {
    CHAT_READY.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub fn pin_chat(app: AppHandle, thread_data: Option<serde_json::Value>, bounds: Option<serde_json::Value>) {
    // Get overlay position to calculate absolute chat panel position
    let overlay_pos = app.get_webview_window("overlay")
        .and_then(|w| w.outer_position().ok());
    let overlay_scale = app.get_webview_window("overlay")
        .and_then(|w| w.current_monitor().ok().flatten())
        .map(|m| m.scale_factor()).unwrap_or(2.0);

    // Calculate absolute chat position from overlay-relative bounds
    let (cx, cy, cw, ch) = if let Some(b) = &bounds {
        let ox = overlay_pos.map(|p| p.x as f64 / overlay_scale).unwrap_or(0.0);
        let oy = overlay_pos.map(|p| p.y as f64 / overlay_scale).unwrap_or(0.0);
        (
            ox + b.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
            oy + b.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
            b.get("width").and_then(|v| v.as_f64()).unwrap_or(420.0),
            b.get("height").and_then(|v| v.as_f64()).unwrap_or(550.0),
        )
    } else {
        (200.0, 200.0, 420.0, 550.0)
    };

    // Create chat window at exact position BEFORE closing overlay
    if app.get_webview_window("chat").is_none() {
        let _ = WebviewWindowBuilder::new(&app, "chat", WebviewUrl::App("index.html#chat-only".into()))
            .title("Glimpse Chat")
            .position(cx, cy)
            .inner_size(cw, ch)
            .resizable(true)
            .decorations(false)
            .transparent(true)
            .accept_first_mouse(true)
            .always_on_top(true)
            .visible(false)
            .build();
        if let Some(win) = app.get_webview_window("chat") {
            let w = win.clone();
            let _ = win.run_on_main_thread(move || {
                crate::native_mac::set_transparent_background(&w);
            });
        }
    }

    // Close overlay
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.close();
    }

    // Wait for chat frontend to be ready, then show
    let app_pin = app.clone();
    std::thread::spawn(move || {
        // Poll for chat_ready (set by chat_ready command) instead of blind sleep
        CHAT_READY.store(false, Ordering::SeqCst);
        for _ in 0..80 {
            if CHAT_READY.load(Ordering::SeqCst) { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        if let Some(w) = app_pin.get_webview_window("chat") {
            if let Some(data) = thread_data {
                let _ = w.emit("load-thread-data", &data);
            }
            let _ = w.emit("pin-state", true);
            // Brief delay for data to be processed before showing
            std::thread::sleep(std::time::Duration::from_millis(50));
            let _ = w.show();
            let _ = w.set_focus();
        }

        // Pre-warm in background — don't block pin flow
        let app_pw = app_pin.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = prewarm_overlay(&app_pw);
        });
    });
}

#[tauri::command]
pub fn toggle_pin(app: AppHandle) {
    if let Some(w) = app.get_webview_window("chat") {
        if let Ok(is_on_top) = w.is_always_on_top() {
            let _ = w.set_always_on_top(!is_on_top);
            let _ = w.emit("pin-state", !is_on_top);
        }
    }
}

#[tauri::command]
pub fn show_toast(app: AppHandle, message: String) {
    // Use a simple approach: create a window loading a toast HTML page
    std::thread::spawn(move || {
        // Close existing toast
        if let Some(w) = app.get_webview_window("toast") {
            let _ = w.close();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // Position toast centered on the active display
        let (x, y) = {
            // Try to get cursor position for active display detection
            let cursor = core_graphics::event_source::CGEventSource::new(
                core_graphics::event_source::CGEventSourceStateID::CombinedSessionState,
            ).ok().and_then(|s| core_graphics::event::CGEvent::new(s).ok())
            .map(|e| e.location());

            if let Some(pos) = cursor {
                // Find monitor at cursor
                let monitors = app.available_monitors().unwrap_or_default();
                let monitor = monitors.iter().find(|m| {
                    let mp = m.position();
                    let ms = m.size();
                    let mx = mp.x as f64;
                    let my = mp.y as f64;
                    pos.x >= mx && pos.x < mx + ms.width as f64 && pos.y >= my && pos.y < my + ms.height as f64
                }).or_else(|| monitors.first());
                if let Some(m) = monitor {
                    let mp = m.position();
                    let ms = m.size();
                    let scale = m.scale_factor();
                    (
                        mp.x as f64 / scale + (ms.width as f64 / scale / 2.0) - 110.0,
                        mp.y as f64 / scale + (ms.height as f64 / scale * 0.18),
                    )
                } else {
                    (660.0, 200.0)
                }
            } else {
                (660.0, 200.0)
            }
        };

        eprintln!("[Toast] showing at ({}, {}): {}", x, y, message);
        // Write a temp HTML file for the toast
        let html = format!(
            "<!DOCTYPE html><html><body style='margin:0;display:flex;align-items:center;justify-content:center;height:100%;background:transparent'>\
            <div style='background:rgba(20,24,36,0.92);backdrop-filter:blur(12px);-webkit-backdrop-filter:blur(12px);\
            border:1px solid rgba(0,229,255,0.2);border-radius:10px;padding:8px 20px;\
            display:flex;align-items:center;gap:8px;\
            font-family:-apple-system,BlinkMacSystemFont,sans-serif;font-size:13px;\
            color:rgba(230,240,255,0.9)'>\
            <svg width='16' height='16' viewBox='0 0 24 24' fill='none' stroke='rgb(52,199,89)' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'>\
            <path d='M20 6L9 17l-5-5'/></svg>{}</div></body></html>",
            message
        );
        let toast_path = std::env::temp_dir().join("glimpse-toast.html");
        let _ = std::fs::write(&toast_path, &html);

        if let Ok(win) = WebviewWindowBuilder::new(
            &app,
            "toast",
            WebviewUrl::External(format!("file://{}", toast_path.to_string_lossy()).parse().unwrap()),
        )
            .title("Toast")
            .position(x, y)
            .inner_size(220.0, 44.0)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .focused(false)
            .visible(false)
            .build()
        {
            // Set transparent background + disable shadow synchronously before showing
            let w = win.clone();
            let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let done2 = done.clone();
            let _ = win.run_on_main_thread(move || {
                crate::native_mac::set_transparent_background(&w);
                // Disable window shadow — toast has its own CSS box-shadow
                if let Ok(ptr) = w.ns_window() {
                    unsafe {
                        use objc2::runtime::AnyObject;
                        use objc2::msg_send;
                        let win = ptr as *mut AnyObject;
                        let _: () = msg_send![&*win, setHasShadow: false];
                    }
                }
                done2.store(true, std::sync::atomic::Ordering::SeqCst);
            });
            for _ in 0..100 {
                if done.load(std::sync::atomic::Ordering::SeqCst) { break; }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            let _ = win.show();
            let _ = win.set_ignore_cursor_events(true);
            let _ = win.set_visible_on_all_workspaces(true);
            std::thread::sleep(std::time::Duration::from_millis(1800));
            let _ = win.close();
        }
        let _ = std::fs::remove_file(&toast_path);
    });
}

#[tauri::command]
pub fn notify_providers_changed(app: AppHandle) {
    let _ = app.emit("providers-changed", ());
}

#[tauri::command]
pub fn resize_chat_window(app: AppHandle, size: serde_json::Value) {
    if let Some(w) = app.get_webview_window("chat") {
        let new_w = size.get("width").and_then(|v| v.as_f64()).unwrap_or(420.0);
        let new_h = size.get("height").and_then(|v| v.as_f64()).unwrap_or(550.0);
        // Only grow, never shrink (matches Electron behavior)
        if let Ok(current) = w.outer_size() {
            let scale = w.current_monitor().ok().flatten().map(|m| m.scale_factor()).unwrap_or(2.0);
            let cur_w = current.width as f64 / scale;
            let cur_h = current.height as f64 / scale;
            eprintln!("[Resize] requested={}x{}, current={}x{}, scale={}", new_w, new_h, cur_w, cur_h, scale);
            let width = new_w.max(cur_w);
            let height = new_h.max(cur_h);
            let _ = w.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));
        } else {
            let _ = w.set_size(tauri::Size::Logical(tauri::LogicalSize { width: new_w, height: new_h }));
        }
    }
}

#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    // Hide settings while folder picker is open to avoid square corner artifact
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.hide();
    }

    let folder = app.dialog().file().blocking_pick_folder();

    // Restore settings
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
    }

    Ok(folder.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn copy_image(data_url: String) -> Result<(), String> {
    // Extract base64 data from data URL
    let b64 = data_url.split(',').nth(1).ok_or("Invalid data URL")?;
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    // Write to temp file, then use osascript to copy to clipboard as image
    let temp_path = std::env::temp_dir().join("glimpse-copy.png");
    std::fs::write(&temp_path, &bytes).map_err(|e| e.to_string())?;

    let script = format!(
        r#"set the clipboard to (read POSIX file "{}" as «class PNGf»)"#,
        temp_path.to_string_lossy()
    );
    std::process::Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| e.to_string())?;

    let _ = std::fs::remove_file(&temp_path);
    Ok(())
}

#[tauri::command]
pub async fn save_image(app: AppHandle, data_url: String) -> Result<serde_json::Value, String> {
    use tauri_plugin_dialog::DialogExt;

    let b64 = data_url.split(',').nth(1).ok_or("Invalid data URL")?;
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    // Check save preference
    let prefs = crate::commands::settings::get_preferences().unwrap_or(serde_json::json!({}));
    let save_location = prefs.get("saveLocation").and_then(|v| v.as_str()).unwrap_or("ask");
    let save_path = prefs.get("savePath").and_then(|v| v.as_str()).unwrap_or("");

    let timestamp = chrono_now();
    let filename = format!("Glimpse {}.png", timestamp);

    let dest = if save_location == "folder" && !save_path.is_empty() {
        std::path::PathBuf::from(save_path).join(&filename)
    } else {
        // Lower overlay so save dialog isn't covered by it
        lower_overlay(app.clone());
        let file = app.dialog().file()
            .set_file_name(&filename)
            .add_filter("PNG Image", &["png"])
            .blocking_save_file();
        restore_overlay(app.clone());
        match file {
            Some(p) => std::path::PathBuf::from(p.to_string()),
            None => return Ok(serde_json::json!({ "success": false })),
        }
    };

    std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "success": true,
        "filePath": dest.to_string_lossy()
    }))
}

fn chrono_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .to_string()
}
