use base64::Engine;

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

// FFI declarations for CGWindowList + ImageIO
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGMainDisplayID() -> u32;
    fn CGDisplayBounds(display: u32) -> core_graphics::geometry::CGRect;
    fn CGWindowListCreateImage(
        rect: core_graphics::geometry::CGRect,
        list_option: u32,
        window_id: u32,
        image_option: u32,
    ) -> *mut std::ffi::c_void; // CGImageRef
}

#[link(name = "ImageIO", kind = "framework")]
extern "C" {
    fn CGImageDestinationCreateWithData(
        data: *mut std::ffi::c_void,   // CFMutableDataRef
        type_: *const std::ffi::c_void, // CFStringRef
        count: usize,
        options: *const std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    fn CGImageDestinationAddImage(
        dest: *mut std::ffi::c_void,
        image: *mut std::ffi::c_void,
        properties: *const std::ffi::c_void,
    );
    fn CGImageDestinationFinalize(dest: *mut std::ffi::c_void) -> bool;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDataCreateMutable(allocator: *const std::ffi::c_void, capacity: i64) -> *mut std::ffi::c_void;
    fn CFDataGetLength(data: *const std::ffi::c_void) -> i64;
    fn CFDataGetBytePtr(data: *const std::ffi::c_void) -> *const u8;
    fn CFRelease(cf: *const std::ffi::c_void);
    fn CFStringCreateWithCString(
        allocator: *const std::ffi::c_void,
        cstr: *const i8,
        encoding: u32,
    ) -> *const std::ffi::c_void;
}

const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;
const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1;

/// Fast screen capture using CGWindowListCreateImage (no subprocess, no disk I/O).
pub fn capture_screen_to_memory() -> Result<(String, serde_json::Value), String> {
    let display_info = get_display_info()?;
    let start = std::time::Instant::now();

    unsafe {
        let display_id = CGMainDisplayID();
        let bounds = CGDisplayBounds(display_id);

        // Capture entire screen
        let image = CGWindowListCreateImage(
            bounds,
            K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY,
            0, // kCGNullWindowID
            0, // kCGWindowImageDefault
        );
        if image.is_null() {
            return Err("CGWindowListCreateImage failed".to_string());
        }

        eprintln!("[Screenshot] CGWindowListCreateImage took {:?}", start.elapsed());

        // Create mutable data buffer
        let data = CFDataCreateMutable(std::ptr::null(), 0);
        if data.is_null() {
            CFRelease(image);
            return Err("CFDataCreateMutable failed".to_string());
        }

        // Create JPEG destination
        let jpeg_type = CFStringCreateWithCString(
            std::ptr::null(),
            b"public.jpeg\0".as_ptr() as *const i8,
            K_CF_STRING_ENCODING_UTF8,
        );
        let dest = CGImageDestinationCreateWithData(data, jpeg_type, 1, std::ptr::null());
        if dest.is_null() {
            CFRelease(data);
            CFRelease(image);
            CFRelease(jpeg_type);
            return Err("CGImageDestinationCreateWithData failed".to_string());
        }

        // Add image and finalize
        CGImageDestinationAddImage(dest, image, std::ptr::null());
        if !CGImageDestinationFinalize(dest) {
            CFRelease(dest);
            CFRelease(data);
            CFRelease(image);
            CFRelease(jpeg_type);
            return Err("CGImageDestinationFinalize failed".to_string());
        }

        // Extract bytes
        let len = CFDataGetLength(data) as usize;
        let ptr = CFDataGetBytePtr(data);
        let bytes = std::slice::from_raw_parts(ptr, len).to_vec();

        CFRelease(dest);
        CFRelease(data);
        CFRelease(image);
        CFRelease(jpeg_type);

        eprintln!("[Screenshot] total capture + encode took {:?}, size={}KB", start.elapsed(), bytes.len() / 1024);

        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok((format!("data:image/jpeg;base64,{}", b64), display_info))
    }
}

/// Fallback: capture via screencapture command
pub fn capture_screen_to_file() -> Result<(String, serde_json::Value), String> {
    let display_info = get_display_info()?;
    let temp_path = std::env::temp_dir().join("glimpse-capture.jpg");
    let path_str = temp_path.to_string_lossy().to_string();

    let output = std::process::Command::new("screencapture")
        .args(["-x", "-t", "jpg", "-C", &path_str])
        .output()
        .map_err(|e| format!("screencapture failed: {}", e))?;

    if !output.status.success() {
        return Err("screencapture returned non-zero".to_string());
    }

    Ok((path_str, display_info))
}
