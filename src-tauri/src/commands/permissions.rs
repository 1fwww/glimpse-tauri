use serde_json::{json, Value};

#[tauri::command]
pub fn check_permissions() -> Result<Value, String> {
    let screen = check_screen_recording();
    let accessibility = check_accessibility();
    Ok(json!({
        "screen": screen,
        "accessibility": accessibility
    }))
}

#[tauri::command]
pub fn request_screen_permission() -> Result<Value, String> {
    let granted = request_screen_recording();
    Ok(json!({ "granted": granted }))
}

#[tauri::command]
pub fn open_permission_settings(r#type: String) {
    let url = match r#type.as_str() {
        "accessibility" => "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        "screen" => "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture",
        _ => return,
    };
    let _ = std::process::Command::new("open").arg(url).spawn();
}

#[cfg(target_os = "macos")]
fn check_screen_recording() -> bool {
    // CGPreflightScreenCaptureAccess() - available since macOS 10.15
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
    }
    unsafe { CGPreflightScreenCaptureAccess() }
}

#[cfg(not(target_os = "macos"))]
fn check_screen_recording() -> bool {
    true
}

#[cfg(target_os = "macos")]
fn check_accessibility() -> bool {
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
fn check_accessibility() -> bool {
    true
}

#[cfg(target_os = "macos")]
fn request_screen_recording() -> bool {
    extern "C" {
        fn CGRequestScreenCaptureAccess() -> bool;
    }
    unsafe { CGRequestScreenCaptureAccess() }
}

#[cfg(not(target_os = "macos"))]
fn request_screen_recording() -> bool {
    true
}
