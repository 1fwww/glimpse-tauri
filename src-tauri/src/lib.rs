mod capture;
mod commands;
mod native_mac;
mod windows;

use std::fs;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            // Thread management
            commands::threads::get_threads,
            commands::threads::save_thread,
            commands::threads::delete_thread,
            // API keys & providers
            commands::settings::get_api_keys,
            commands::settings::save_api_keys,
            commands::settings::delete_api_key,
            commands::settings::get_available_providers,
            commands::settings::validate_invite_code,
            // Preferences
            commands::settings::get_preferences,
            commands::settings::set_preference,
            // AI
            commands::ai::chat_with_ai,
            commands::ai::generate_title,
            // Permissions
            commands::permissions::check_permissions,
            commands::permissions::request_screen_permission,
            commands::permissions::open_permission_settings,
            // Window management
            windows::close_home,
            windows::close_welcome,
            windows::input_focus,
            windows::lower_overlay,
            windows::restore_overlay,
            windows::close_settings,
            windows::close_chat_window,
            windows::open_settings,
            windows::toggle_settings,
            windows::open_thread_in_chat,
            windows::welcome_done,
            windows::chat_ready,
            windows::pin_chat,
            windows::toggle_pin,
            windows::show_toast,
            windows::notify_providers_changed,
            windows::resize_chat_window,
            windows::select_folder,
            windows::copy_image,
            windows::save_image,
            // Screenshot
            trigger_screenshot,
            overlay_pong,
            windows::close_overlay,
        ])
        .setup(|app| {
            // Ensure data directories exist
            let data_dir = get_data_dir();
            let threads_dir = data_dir.join("threads");
            fs::create_dir_all(&threads_dir).ok();

            // Check onboarding status
            let onboarding_path = data_dir.join("onboarding-done");
            if onboarding_path.exists() {
                windows::create_home_window(&app.handle())?;
            } else {
                windows::create_welcome_window(&app.handle())?;
            }

            // Register global shortcuts
            let screenshot_shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyZ);
            let chat_shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyX);

            let app_handle1 = app.handle().clone();
            let app_handle2 = app.handle().clone();
            app.global_shortcut().on_shortcut(screenshot_shortcut, move |_app, _shortcut, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    handle_screenshot_shortcut(&app_handle1);
                }
            })?;
            app.global_shortcut().on_shortcut(chat_shortcut, move |_app, _shortcut, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    handle_chat_shortcut(&app_handle2);
                }
            })?;

            // Pre-warm overlay window for instant screenshot
            let app_prewarm = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                let _ = windows::prewarm_overlay(&app_prewarm);
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            match event {
                tauri::RunEvent::ExitRequested { api, .. } => {
                    eprintln!("[App] ExitRequested — preventing");
                    api.prevent_exit();
                }
                tauri::RunEvent::Reopen { .. } => {
                    // Dock icon clicked — show home if no windows visible
                    let has_visible = app.webview_windows().values().any(|w| {
                        let label = w.label();
                        // Skip hidden pre-warmed overlay
                        label != "overlay" && w.is_visible().unwrap_or(false)
                    });
                    if !has_visible {
                        let data_dir = get_data_dir();
                        if data_dir.join("onboarding-done").exists() {
                            let _ = windows::create_home_window(app);
                        } else {
                            let _ = windows::create_welcome_window(app);
                        }
                    }
                }
                _ => {}
            }
        });
}

/// Get the app data directory, matching Electron's userData path
pub fn get_data_dir() -> std::path::PathBuf {
    let home = dirs::home_dir().expect("no home directory");
    home.join("Library")
        .join("Application Support")
        .join("glimpse")
}

fn handle_screenshot_shortcut(app: &tauri::AppHandle) {
    // If welcome window is open, notify it
    if let Some(w) = app.get_webview_window("welcome") {
        let _ = w.emit("shortcut-tried", "screenshot");
        return;
    }

    // Check which windows are actually visible on the current Space
    let is_vis = |label: &str| -> bool {
        app.get_webview_window(label).map(|w| w.is_visible().unwrap_or(false)).unwrap_or(false)
    };
    let has_visible = is_vis("home") || is_vis("chat") || is_vis("settings") || is_vis("overlay");
    eprintln!("[Screenshot] triggered, has_visible={}", has_visible);

    // Switch to Accessory policy + refresh overlay's collection behavior
    // This allows overlay to appear on fullscreen Spaces
    if !IS_ACCESSORY.load(std::sync::atomic::Ordering::SeqCst) {
        let app2 = app.clone();
        let ow_ref = app.get_webview_window("overlay");
        if let Some(ow) = ow_ref {
            let ow2 = ow.clone();
            let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let done2 = done.clone();
            let _ = ow.run_on_main_thread(move || {
                app2.set_activation_policy(tauri::ActivationPolicy::Accessory);
                // Refresh collection behavior under new policy so prewarm works on fullscreen
                native_mac::set_visible_on_fullscreen(&ow2, true);
                native_mac::set_window_level_screen_saver(&ow2);
                done2.store(true, std::sync::atomic::Ordering::SeqCst);
            });
            for _ in 0..50 {
                if done.load(std::sync::atomic::Ordering::SeqCst) { break; }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            IS_ACCESSORY.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    let app_clone = app.clone();
    std::thread::spawn(move || {
        // If overlay is already showing AND actively used, just reset its state
        if let Some(w) = app_clone.get_webview_window("overlay") {
            let visible = w.is_visible().unwrap_or(false);
            let on_top = w.is_always_on_top().unwrap_or(false);
            eprintln!("[Screenshot] overlay exists, visible={}, on_top={}", visible, on_top);
            // Only treat as "active" if both visible AND on top (not just pre-warmed)
            if visible && on_top {
                // Ping overlay to check if it's alive
                OVERLAY_ALIVE.store(false, std::sync::atomic::Ordering::SeqCst);
                let _ = w.emit("reset-overlay", ());
                // Brief wait to see if overlay responds with pong
                std::thread::sleep(std::time::Duration::from_millis(100));
                if OVERLAY_ALIVE.load(std::sync::atomic::Ordering::SeqCst) {
                    eprintln!("[Screenshot] overlay alive, reset sent");
                    if let Some(sw) = app_clone.get_webview_window("settings") {
                        let _ = sw.close();
                    }
                    return;
                }
                // Overlay is dead — close it and fall through to recapture
                eprintln!("[Screenshot] overlay dead, closing and recapturing");
                let _ = w.close();
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        }

        if has_visible {
            for label in &["home", "chat", "settings"] {
                if let Some(w) = app_clone.get_webview_window(label) {
                    let _ = w.hide();
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        // Capture screen and get window bounds in parallel
        let bounds_handle = std::thread::spawn(|| capture::windows::get_window_bounds());
        let capture_result = capture::screenshot::capture_screen_to_memory();
        let window_bounds = bounds_handle.join().unwrap_or_default();

        match capture_result {
            Ok((data_url, display_info)) => {
                let offset = serde_json::json!({"x": 0, "y": 0});

                let payload = serde_json::json!({
                    "dataUrl": data_url,
                    "windowBounds": window_bounds,
                    "displayInfo": display_info,
                    "offset": offset,
                });

                eprintln!("[Screenshot] capture done, showing overlay");
                // Use pre-warmed overlay or create new one
                if let Some(w) = app_clone.get_webview_window("overlay") {
                    eprintln!("[Screenshot] using pre-warmed overlay");
                    let dx = display_info.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let dy = display_info.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let dw = display_info.get("width").and_then(|v| v.as_f64()).unwrap_or(1920.0);
                    let dh = display_info.get("height").and_then(|v| v.as_f64()).unwrap_or(1080.0);
                    let _ = w.set_position(tauri::Position::Logical(tauri::LogicalPosition { x: dx, y: dy }));
                    let _ = w.set_size(tauri::Size::Logical(tauri::LogicalSize { width: dw, height: dh }));
                    let _ = w.set_always_on_top(true);
                    let _ = w.set_visible_on_all_workspaces(true);
                    // Activate app + set window level synchronously before showing
                    let w2 = w.clone();
                    let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                    let done2 = done.clone();
                    let _ = w.run_on_main_thread(move || {
                        native_mac::activate_app();
                        native_mac::set_visible_on_fullscreen(&w2, true);
                        native_mac::set_window_level_screen_saver(&w2);
                        native_mac::order_front(&w2);
                        done2.store(true, std::sync::atomic::Ordering::Release);
                    });
                    for _ in 0..100 {
                        if done.load(std::sync::atomic::Ordering::Acquire) { break; }
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    let _ = w.emit("screen-captured", &payload);
                    let _ = w.show();
                    let _ = w.set_focus();
                    // Install ESC global monitor so ESC works before selection
                    native_mac::install_esc_monitor(app_clone.clone());
                } else {
                    eprintln!("[Screenshot] no pre-warm, creating fresh overlay");
                    let _ = windows::create_overlay_window(&app_clone, &display_info);
                    // Wait for webview to load (fresh overlay, no prewarm)
                    std::thread::sleep(std::time::Duration::from_millis(800));
                    if let Some(w) = app_clone.get_webview_window("overlay") {
                        let _ = w.emit("screen-captured", &payload);
                        native_mac::install_esc_monitor(app_clone.clone());
                    }
                }

                // Clean up hidden windows
                for label in &["home", "chat", "settings"] {
                    if let Some(w) = app_clone.get_webview_window(label) {
                        let _ = w.close();
                    }
                }

                // Schedule next pre-warm
                let app_prewarm = app_clone.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    let _ = windows::prewarm_overlay(&app_prewarm);
                });
            }
            Err(e) => eprintln!("Screenshot capture failed: {}", e),
        }
    });
}

fn handle_chat_shortcut(app: &tauri::AppHandle) {
    // If welcome window is open, notify it
    if let Some(w) = app.get_webview_window("welcome") {
        let _ = w.emit("shortcut-tried", "chat");
        return;
    }

    // Close home window if open
    if let Some(w) = app.get_webview_window("home") {
        let _ = w.close();
    }

    // If no visible windows, switch to Accessory policy for fullscreen Space support
    let needs_policy = !app.get_webview_window("home").is_some()
        && !app.get_webview_window("chat").map(|w| w.is_visible().unwrap_or(false)).unwrap_or(false)
        && !app.get_webview_window("overlay").map(|w| w.is_visible().unwrap_or(false)).unwrap_or(false);
    if needs_policy && !IS_ACCESSORY.load(std::sync::atomic::Ordering::SeqCst) {
        if let Some(ow) = app.get_webview_window("overlay") {
            let app2 = app.clone();
            let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let done2 = done.clone();
            let _ = ow.run_on_main_thread(move || {
                app2.set_activation_policy(tauri::ActivationPolicy::Accessory);
                done2.store(true, std::sync::atomic::Ordering::SeqCst);
            });
            for _ in 0..50 {
                if done.load(std::sync::atomic::Ordering::SeqCst) { break; }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            IS_ACCESSORY.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    // Grab selected text BEFORE focusing our window
    let app_clone = app.clone();
    std::thread::spawn(move || {
        // Wait for shortcut keys to release
        std::thread::sleep(std::time::Duration::from_millis(100));
        let selected_text = grab_selected_text();

        if let Some(w) = app_clone.get_webview_window("chat") {
            // Chat already exists — send text and focus
            if !selected_text.is_empty() {
                let _ = w.emit("text-context", &selected_text);
            }
            let _ = w.set_focus();
        } else {
            // Create new chat window, wait for ready via event
            let _ = windows::create_chat_window(&app_clone);
            use std::sync::atomic::Ordering;
            windows::CHAT_READY.store(false, Ordering::SeqCst);
            for _ in 0..80 {
                if windows::CHAT_READY.load(Ordering::SeqCst) { break; }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            if let Some(w) = app_clone.get_webview_window("chat") {
                if !selected_text.is_empty() {
                    let _ = w.emit("text-context", &selected_text);
                }
                let _ = w.set_focus();
            }
        }
    });
}

/// Simulate Cmd+C to grab the currently selected text from any app.
/// Must be called BEFORE focusing our window.
fn grab_selected_text() -> String {
    use std::process::Command;

    // Save current clipboard
    let saved = Command::new("pbpaste").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    // Clear clipboard
    let _ = Command::new("pbcopy").stdin(std::process::Stdio::piped()).spawn()
        .and_then(|mut c| {
            use std::io::Write;
            if let Some(stdin) = c.stdin.as_mut() {
                stdin.write_all(b"")?;
            }
            c.wait()
        });

    // Simulate Cmd+C
    let _ = Command::new("osascript")
        .args(["-e", r#"tell application "System Events" to keystroke "c" using command down"#])
        .output();

    // Poll clipboard for the new content
    let mut selected = String::new();
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(30));
        if let Ok(output) = Command::new("pbpaste").output() {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            if !text.is_empty() {
                selected = text;
                break;
            }
        }
    }

    // Restore original clipboard
    let _ = Command::new("pbcopy").stdin(std::process::Stdio::piped()).spawn()
        .and_then(|mut c| {
            use std::io::Write;
            if let Some(stdin) = c.stdin.as_mut() {
                stdin.write_all(saved.as_bytes())?;
            }
            c.wait()
        });

    selected
}

static OVERLAY_ALIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub static IS_ACCESSORY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[tauri::command]
fn overlay_pong() {
    OVERLAY_ALIVE.store(true, std::sync::atomic::Ordering::SeqCst);
}

#[tauri::command]
fn trigger_screenshot(app: tauri::AppHandle) {
    handle_screenshot_shortcut(&app);
}
