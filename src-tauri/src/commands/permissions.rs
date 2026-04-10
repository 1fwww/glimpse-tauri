use serde_json::{json, Value};

#[tauri::command]
pub fn check_permissions() -> Result<Value, String> {
    let screen = check_screen_recording();
    let accessibility = check_accessibility();
    eprintln!("[Permissions] screen={}, accessibility={}", screen, accessibility);
    Ok(json!({
        "screen": screen,
        "accessibility": accessibility
    }))
}

static SCREEN_REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[tauri::command]
pub fn request_screen_permission() -> Result<Value, String> {
    if check_screen_recording() {
        return Ok(json!({ "granted": true }));
    }
    if SCREEN_REQUESTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        // Already prompted once — system won't show again, open Settings instead
        open_permission_settings("screen".to_string());
    } else {
        // First time — show the system prompt
        request_screen_recording();
    }
    Ok(json!({ "granted": false }))
}

#[tauri::command]
pub fn request_accessibility_permission() -> Result<Value, String> {
    if check_accessibility() {
        return Ok(json!({ "granted": true }));
    }
    // AXIsProcessTrustedWithOptions with prompt — registers correct process identity in TCC
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            fn CFStringCreateWithCString(alloc: *const std::ffi::c_void, c_str: *const i8, encoding: u32) -> *const std::ffi::c_void;
            fn CFDictionaryCreate(
                allocator: *const std::ffi::c_void,
                keys: *const *const std::ffi::c_void,
                values: *const *const std::ffi::c_void,
                num_values: isize,
                key_callbacks: *const std::ffi::c_void,
                value_callbacks: *const std::ffi::c_void,
            ) -> *const std::ffi::c_void;
            fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
            static kCFBooleanTrue: *const std::ffi::c_void;
            static kCFTypeDictionaryKeyCallBacks: std::ffi::c_void;
            static kCFTypeDictionaryValueCallBacks: std::ffi::c_void;
        }
        unsafe {
            let key_str = b"AXTrustedCheckOptionPrompt\0";
            let key = CFStringCreateWithCString(
                std::ptr::null(),
                key_str.as_ptr() as *const i8,
                0x08000100, // kCFStringEncodingUTF8
            );
            let keys = [key];
            let values = [kCFBooleanTrue];
            let options = CFDictionaryCreate(
                std::ptr::null(),
                keys.as_ptr(),
                values.as_ptr(),
                1,
                &kCFTypeDictionaryKeyCallBacks as *const _ as *const std::ffi::c_void,
                &kCFTypeDictionaryValueCallBacks as *const _ as *const std::ffi::c_void,
            );
            let result = AXIsProcessTrustedWithOptions(options);
            eprintln!("[Permissions] AXIsProcessTrustedWithOptions(prompt=true) = {}", result);
        }
    }
    Ok(json!({ "granted": false }))
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
    check_accessibility_trusted()
}

/// Public accessor for AXIsProcessTrusted (used by grab_selected_text)
#[cfg(target_os = "macos")]
pub fn check_accessibility_trusted() -> bool {
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_trusted() -> bool {
    true
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
