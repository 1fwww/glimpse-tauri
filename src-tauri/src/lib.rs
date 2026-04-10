mod capture;
mod commands;
mod native_mac;
mod windows;

use std::fs;
use tauri::{Emitter, Manager, AppHandle};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconId};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let screenshot_i = MenuItem::with_id(app, "screenshot", "Screenshot    ⌘⇧Z", true, None::<&str>)?;
    let chat_i = MenuItem::with_id(app, "chat", "Text Chat    ⌘⇧X", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit Glimpse", true, None::<&str>)?;

    let threads = commands::threads::get_threads().unwrap_or_default();
    let mut menu_items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();
    menu_items.push(Box::new(screenshot_i));
    menu_items.push(Box::new(chat_i));

    if !threads.is_empty() {
        menu_items.push(Box::new(MenuItem::with_id(app, "sep_chats", "", false, None::<&str>)?));
        menu_items.push(Box::new(MenuItem::with_id(app, "recent_header", "Recent Chats", false, None::<&str>)?));
        for (i, t) in threads.iter().take(5).enumerate() {
            let title = t.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
            let truncated = if title.len() > 28 { format!("💬 {}...", &title[..25]) } else { format!("💬 {}", title) };
            menu_items.push(Box::new(MenuItem::with_id(app, &format!("thread_{}", i), &truncated, true, None::<&str>)?));
        }
    } else {
        menu_items.push(Box::new(MenuItem::with_id(app, "sep_chats", "", false, None::<&str>)?));
        menu_items.push(Box::new(MenuItem::with_id(app, "no_chats", "No recent chats yet", false, None::<&str>)?));
    }

    menu_items.push(Box::new(MenuItem::with_id(app, "sep_settings", "", false, None::<&str>)?));
    menu_items.push(Box::new(settings_i));
    menu_items.push(Box::new(MenuItem::with_id(app, "sep_quit", "", false, None::<&str>)?));
    menu_items.push(Box::new(quit_i));

    let refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = menu_items.iter().map(|b| b.as_ref()).collect();
    Ok(Menu::with_items(app, &refs)?)
}

#[tauri::command]
fn refresh_tray_menu(app: AppHandle) {
    if let Ok(menu) = build_tray_menu(&app) {
        if let Some(tray) = app.tray_by_id(&TrayIconId::new("glimpse-tray")) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

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
            commands::permissions::request_accessibility_permission,
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
            refresh_tray_menu,
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
            if !onboarding_path.exists() {
                windows::create_welcome_window(&app.handle())?;
            }
            // No Home window — tray icon is the primary interface

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
            let app_chat_prewarm = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                let _ = windows::prewarm_overlay(&app_prewarm);
                // Pre-warm chat webview after overlay (separate delay, no competition)
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = windows::prewarm_chat(&app_chat_prewarm);
            });

            // Pre-warm System Events in separate thread (can be slow, don't block overlay)
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(2000));
                let _ = std::process::Command::new("osascript")
                    .args(["-e", r#"tell application "System Events" to return"#])
                    .output();
            });

            // Tray icon
            let tray_menu = build_tray_menu(app.handle())?;

            let app_tray = app.handle().clone();
            let tray_icon_bytes = include_bytes!("../icons/tray-icon.png");
            let tray_icon = tauri::image::Image::from_bytes(tray_icon_bytes).expect("failed to load tray icon");
            let _ = TrayIconBuilder::with_id("glimpse-tray")
                .icon(tray_icon)
                .icon_as_template(true)
                .menu(&tray_menu)
                .show_menu_on_left_click(true)
                .on_menu_event(move |_app, event| {
                    // If onboarding not done, redirect to welcome
                    let data_dir = get_data_dir();
                    if !data_dir.join("onboarding-done").exists() && event.id.as_ref() != "quit" {
                        let _ = windows::create_welcome_window(&app_tray);
                        return;
                    }
                    let id = event.id.as_ref().to_string();
                    match id.as_str() {
                        "screenshot" => handle_screenshot_shortcut(&app_tray),
                        "chat" => handle_chat_shortcut(&app_tray),
                        "settings" => { let _ = windows::create_settings_window(&app_tray, None); },
                        "quit" => std::process::exit(0),
                        _ if id.starts_with("thread_") => {
                            // Open thread by index
                            if let Ok(idx) = id.replace("thread_", "").parse::<usize>() {
                                let threads = commands::threads::get_threads().unwrap_or_default();
                                if let Some(thread) = threads.get(idx) {
                                    let thread_data = thread.clone();
                                    let app_thread = app_tray.clone();
                                    let _ = windows::create_chat_window(&app_tray, None);
                                    // Delay emit to ensure chat window is ready
                                    std::thread::spawn(move || {
                                        // Wait for CHAT_READY
                                        for _ in 0..50 {
                                            if windows::CHAT_READY.load(std::sync::atomic::Ordering::SeqCst) { break; }
                                            std::thread::sleep(std::time::Duration::from_millis(10));
                                        }
                                        std::thread::sleep(std::time::Duration::from_millis(50));
                                        if let Some(w) = app_thread.get_webview_window("chat") {
                                            let _ = w.emit("load-thread-data", &thread_data);
                                            let _ = w.set_focus();
                                        }
                                    });
                                }
                            }
                        },
                        _ => {}
                    }
                })
                .build(app);

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
                    // Dock icon clicked
                    let has_visible = app.webview_windows().values().any(|w| {
                        let label = w.label();
                        label != "overlay" && w.is_visible().unwrap_or(false)
                    });
                    if !has_visible {
                        let data_dir = get_data_dir();
                        if data_dir.join("onboarding-done").exists() {
                            let _ = windows::create_chat_window(app, None);
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
    let vis_home = is_vis("home"); let vis_chat = is_vis("chat"); let vis_settings = is_vis("settings"); let vis_overlay = is_vis("overlay");
    let has_visible = vis_home || vis_chat || vis_settings || vis_overlay;
    eprintln!("[Screenshot] triggered, has_visible={} (home={}, chat={}, settings={}, overlay={}), IS_ACCESSORY={}",
        has_visible, vis_home, vis_chat, vis_settings, vis_overlay,
        IS_ACCESSORY.load(std::sync::atomic::Ordering::SeqCst));

    // Switch to Accessory policy — must complete before capture for fullscreen Spaces
    if !IS_ACCESSORY.load(std::sync::atomic::Ordering::SeqCst) {
        eprintln!("[Screenshot] switching to Accessory policy...");
        if let Some(ow) = app.get_webview_window("overlay") {
            let app2 = app.clone();
            let ow2 = ow.clone();
            let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let done2 = done.clone();
            let _ = ow.run_on_main_thread(move || {
                let _ = app2.set_activation_policy(tauri::ActivationPolicy::Accessory);
                native_mac::set_visible_on_fullscreen(&ow2, true);
                native_mac::set_window_level_screen_saver(&ow2);
                done2.store(true, std::sync::atomic::Ordering::SeqCst);
            });
            for _ in 0..30 {
                if done.load(std::sync::atomic::Ordering::SeqCst) { break; }
                std::thread::sleep(std::time::Duration::from_millis(2));
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
                    // Activate app synchronously before showing (minimal wait)
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
                    for _ in 0..30 {
                        if done.load(std::sync::atomic::Ordering::Acquire) { break; }
                        std::thread::sleep(std::time::Duration::from_millis(2));
                    }
                    let _ = w.emit("screen-captured", &payload);
                    let _ = w.show();
                    let _ = w.set_focus();
                    // Install ESC monitor
                    native_mac::install_esc_monitor(app_clone.clone());
                } else {
                    eprintln!("[Screenshot] no pre-warm, creating fresh overlay");
                    let _ = windows::create_overlay_window(&app_clone, &display_info);
                    // Wait for webview to load (fresh overlay, no prewarm)
                    for _ in 0..40 {
                        if app_clone.get_webview_window("overlay").is_some() { break; }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    // Small extra delay for webview JS to initialize
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    if let Some(w) = app_clone.get_webview_window("overlay") {
                        // Same show sequence as pre-warmed path
                        let _ = w.set_always_on_top(true);
                        let _ = w.set_visible_on_all_workspaces(true);
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
                        for _ in 0..30 {
                            if done.load(std::sync::atomic::Ordering::Acquire) { break; }
                            std::thread::sleep(std::time::Duration::from_millis(2));
                        }
                        let _ = w.emit("screen-captured", &payload);
                        let _ = w.show();
                        let _ = w.set_focus();
                        native_mac::install_esc_monitor(app_clone.clone());
                    }
                }

                // Hide other windows (don't close chat — keep pre-warmed)
                for label in &["home", "settings"] {
                    if let Some(w) = app_clone.get_webview_window(label) {
                        let _ = w.close();
                    }
                }
                if let Some(w) = app_clone.get_webview_window("chat") {
                    let _ = w.hide();
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
    eprintln!("[Chat] shortcut triggered, IS_ACCESSORY={}", IS_ACCESSORY.load(std::sync::atomic::Ordering::SeqCst));
    // If welcome window is open, notify it
    if let Some(w) = app.get_webview_window("welcome") {
        let _ = w.emit("shortcut-tried", "chat");
        return;
    }

    // Close/hide all other windows first (shortcut = clear everything + execute)
    if let Some(w) = app.get_webview_window("home") {
        let _ = w.close();
    }
    // If overlay is active, hide it (lightweight — no policy change, no hide_app)
    if let Some(w) = app.get_webview_window("overlay") {
        if w.is_visible().unwrap_or(false) {
            native_mac::remove_esc_monitor();
            let _ = w.set_always_on_top(false);
            let _ = w.hide();
        }
    }

    // Switch to Accessory policy for fullscreen Space support
    // (CanJoinAllSpaces alone isn't enough — fullscreen needs Accessory)
    if !IS_ACCESSORY.load(std::sync::atomic::Ordering::SeqCst) {
        let any_window = app.get_webview_window("chat")
            .or_else(|| app.get_webview_window("overlay"));
        if let Some(w) = any_window {
            let app2 = app.clone();
            let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let done2 = done.clone();
            let _ = w.run_on_main_thread(move || {
                let _ = app2.set_activation_policy(tauri::ActivationPolicy::Accessory);
                done2.store(true, std::sync::atomic::Ordering::SeqCst);
            });
            for _ in 0..30 {
                if done.load(std::sync::atomic::Ordering::SeqCst) { break; }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            IS_ACCESSORY.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    // Grab text FIRST (need focus on source app), then show chat
    let app_clone = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        let selected_text = grab_selected_text();

        // Clear stale state BEFORE showing chat (prevents flash of old content)
        if let Some(w) = app_clone.get_webview_window("chat") {
            let _ = w.emit("clear-screenshot", ());
            let _ = w.emit("clear-text-context", ());
        }
        // Now show chat (instant if pre-warmed)
        let has_context = !selected_text.is_empty();
        let height_hint = if has_context { Some(300.0) } else { None };
        let _ = windows::create_chat_window(&app_clone, height_hint);
        if let Some(w) = app_clone.get_webview_window("chat") {
            if has_context {
                let _ = w.emit("text-context", &selected_text);
            }
        }
    });
}

/// Simulate Cmd+C to grab the currently selected text from any app.
/// Must be called BEFORE focusing our window.
fn grab_selected_text() -> String {
    use std::process::Command;

    // Check if Accessibility is granted (required for keystroke simulation)
    let ax_trusted = commands::permissions::check_accessibility_trusted();
    eprintln!("[GrabText] AXIsProcessTrusted={}", ax_trusted);
    if !ax_trusted {
        eprintln!("[GrabText] Accessibility not granted, skipping text grab");
        return String::new();
    }

    // Save current clipboard
    let saved = Command::new("pbpaste").args(["-Prefer", "txt"]).output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    // Clear clipboard with a unique sentinel to detect "no new copy"
    let sentinel = format!("__glimpse_sentinel_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
    let _ = Command::new("pbcopy").stdin(std::process::Stdio::piped()).spawn()
        .and_then(|mut c| {
            use std::io::Write;
            if let Some(stdin) = c.stdin.as_mut() {
                stdin.write_all(sentinel.as_bytes())?;
            }
            c.wait()
        });

    // Small delay to ensure clipboard is set before Cmd+C
    std::thread::sleep(std::time::Duration::from_millis(20));

    // Simulate Cmd+C
    let _ = Command::new("osascript")
        .args(["-e", r#"tell application "System Events" to keystroke "c" using command down"#])
        .output();

    // Poll clipboard — check for new content (not sentinel)
    let mut selected = String::new();
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        if let Ok(output) = Command::new("pbpaste").args(["-Prefer", "txt"]).output() {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            if !text.is_empty() && text != sentinel {
                selected = text;
                break;
            }
        }
    }

    eprintln!("[GrabText] result: {} chars, empty={}", selected.len(), selected.is_empty());

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
