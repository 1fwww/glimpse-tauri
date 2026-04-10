#[cfg(target_os = "macos")]
unsafe fn set_subviews_transparent(
    view: *mut objc2::runtime::AnyObject,
    clear: *mut objc2::runtime::AnyObject,
    draw_sel: objc2::runtime::Sel,
) {
    use objc2::runtime::AnyObject;
    use objc2::msg_send;

    let subviews: *mut AnyObject = msg_send![&*view, subviews];
    let count: usize = msg_send![&*subviews, count];
    for i in 0..count {
        let sub: *mut AnyObject = msg_send![&*subviews, objectAtIndex: i];
        // Try _setDrawsBackground:
        let responds: bool = msg_send![&*sub, respondsToSelector: draw_sel];
        if responds {
            let _: () = msg_send![&*sub, _setDrawsBackground: false];
        }
        // Try setBackgroundColor: with clearColor
        let bg_sel = objc2::sel!(setBackgroundColor:);
        let responds_bg: bool = msg_send![&*sub, respondsToSelector: bg_sel];
        if responds_bg {
            let _: () = msg_send![&*sub, setBackgroundColor: clear];
        }
        // Recurse
        set_subviews_transparent(sub, clear, draw_sel);
    }
}

/// Move window to the active Space when shown (for chat window).
/// Uses MoveToActiveSpace + FullScreenAuxiliary (NOT CanJoinAllSpaces — they're incompatible).
pub fn set_move_to_active_space(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::AnyObject;
        use objc2::msg_send;

        let ns_window = match window.ns_window() {
            Ok(ptr) => ptr,
            Err(e) => {
                eprintln!("[native_mac] ns_window() failed: {}", e);
                return;
            }
        };

        unsafe {
            let win = ns_window as *mut AnyObject;
            // MoveToActiveSpace (1 << 1) | FullScreenAuxiliary (1 << 8)
            let behavior: u64 = (1 << 1) | (1 << 8);
            let _: () = msg_send![&*win, setCollectionBehavior: behavior];
            let actual: u64 = msg_send![&*win, collectionBehavior];
            eprintln!("[native_mac] set_move_to_active_space: requested={}, actual={}", behavior, actual);
        }
    }
}


/// Check if the current Space is a fullscreen Space using private CGS APIs.
pub fn is_fullscreen_space() -> bool {
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            fn CGSMainConnectionID() -> u32;
            fn CGSGetActiveSpace(cid: u32) -> u64;
            fn CGSSpaceGetType(cid: u32, space: u64) -> u32;
        }
        unsafe {
            let cid = CGSMainConnectionID();
            let space = CGSGetActiveSpace(cid);
            let space_type = CGSSpaceGetType(cid, space);
            // Type 0 = normal desktop, Type 4 = fullscreen
            eprintln!("[native_mac] space check: id={}, type={}", space, space_type);
            space_type == 4
        }
    }
    #[cfg(not(target_os = "macos"))]
    { false }
}

/// Lock window to current Space only (FullScreenAuxiliary, no CanJoinAllSpaces).
pub fn set_single_space(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::AnyObject;
        use objc2::msg_send;

        let ns_window = match window.ns_window() {
            Ok(ptr) => ptr,
            Err(e) => {
                eprintln!("[native_mac] ns_window() failed: {}", e);
                return;
            }
        };

        unsafe {
            let win = ns_window as *mut AnyObject;
            // FullScreenAuxiliary only (1 << 8) — can float on fullscreen but stays on one Space
            let behavior: u64 = 1 << 8;
            let _: () = msg_send![&*win, setCollectionBehavior: behavior];
            let actual: u64 = msg_send![&*win, collectionBehavior];
            eprintln!("[native_mac] set_single_space: requested={}, actual={}", behavior, actual);
        }
    }
}

/// Set a window to be visible on all workspaces including fullscreen Spaces.
pub fn set_visible_on_fullscreen(window: &tauri::WebviewWindow, visible: bool) {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::AnyObject;
        use objc2::msg_send;

        let ns_window = match window.ns_window() {
            Ok(ptr) => ptr,
            Err(e) => {
                eprintln!("[native_mac] ns_window() failed: {}", e);
                return;
            }
        };

        unsafe {
            let win = ns_window as *mut AnyObject;
            if visible {
                // CanJoinAllSpaces (1 << 0) | FullScreenAuxiliary (1 << 8)
                let behavior: u64 = (1 << 0) | (1 << 8);
                let _: () = msg_send![&*win, setCollectionBehavior: behavior];
                // Verify it was set
                let actual: u64 = msg_send![&*win, collectionBehavior];
                eprintln!("[native_mac] set collection behavior: requested={}, actual={}", behavior, actual);
                // Also check current level
                let level: i64 = msg_send![&*win, level];
                eprintln!("[native_mac] current window level: {}", level);
            } else {
                let behavior: u64 = 0;
                let _: () = msg_send![&*win, setCollectionBehavior: behavior];
            }
        }
    }
}


/// Enable or disable NSWindow shadow
pub fn set_window_shadow(window: &tauri::WebviewWindow, shadow: bool) {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::AnyObject;
        use objc2::msg_send;

        let ns_window = match window.ns_window() {
            Ok(ptr) => ptr,
            Err(_) => return,
        };

        unsafe {
            let win = ns_window as *mut AnyObject;
            let _: () = msg_send![&*win, setHasShadow: shadow];
        }
    }
}

/// Animate window frame change using NSWindow setFrame:display:animate:
pub fn animate_frame(window: &tauri::WebviewWindow, x: f64, y: f64, w: f64, h: f64) {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::AnyObject;
        use objc2::msg_send;

        // NSRect-compatible struct for objc2 msg_send
        #[repr(C)]
        #[derive(Copy, Clone)]
        struct CGPoint { x: f64, y: f64 }
        #[repr(C)]
        #[derive(Copy, Clone)]
        struct CGSize { width: f64, height: f64 }
        #[repr(C)]
        #[derive(Copy, Clone)]
        struct CGRect { origin: CGPoint, size: CGSize }

        unsafe impl objc2::Encode for CGPoint {
            const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CGPoint", &[objc2::Encoding::Double, objc2::Encoding::Double]);
        }
        unsafe impl objc2::Encode for CGSize {
            const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CGSize", &[objc2::Encoding::Double, objc2::Encoding::Double]);
        }
        unsafe impl objc2::Encode for CGRect {
            const ENCODING: objc2::Encoding = objc2::Encoding::Struct("CGRect", &[CGPoint::ENCODING, CGSize::ENCODING]);
        }

        let ns_window = match window.ns_window() {
            Ok(ptr) => ptr,
            Err(_) => return,
        };

        unsafe {
            let win = ns_window as *mut AnyObject;
            // Get screen height for coordinate flip (macOS uses bottom-left origin)
            let screen: *mut AnyObject = msg_send![&*win, screen];
            if screen.is_null() {
                eprintln!("[native_mac] animate_frame: window has no screen, falling back to set_size");
                return;
            }
            let screen_frame: CGRect = msg_send![&*screen, frame];
            let screen_h = screen_frame.size.height;
            // Flip y: macOS origin is bottom-left
            let flipped_y = screen_h - y - h;
            let frame = CGRect {
                origin: CGPoint { x, y: flipped_y },
                size: CGSize { width: w, height: h },
            };
            let _: () = msg_send![&*win, setFrame: frame display: true animate: true];
        }
    }
}

/// Force window to front regardless of Space
pub fn order_front(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::AnyObject;
        use objc2::msg_send;

        let ns_window = match window.ns_window() {
            Ok(ptr) => ptr,
            Err(_) => return,
        };

        unsafe {
            let win = ns_window as *mut AnyObject;
            let _: () = msg_send![&*win, orderFrontRegardless];
        }
    }
}

/// Set window level to shielding level (what Electron uses for 'screen-saver')
pub fn set_window_level_screen_saver(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::AnyObject;
        use objc2::msg_send;

        extern "C" {
            fn CGShieldingWindowLevel() -> i32;
        }

        let ns_window = match window.ns_window() {
            Ok(ptr) => ptr,
            Err(_) => return,
        };

        unsafe {
            let win = ns_window as *mut AnyObject;
            let level = CGShieldingWindowLevel() as i64;
            let _: () = msg_send![&*win, setLevel: level];
            let actual: i64 = msg_send![&*win, level];
            eprintln!("[native_mac] set window level: requested={}, actual={}", level, actual);
        }
    }
}

/// Activate the app (bring to front), even if no windows are visible
pub fn activate_app() {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::{AnyClass, AnyObject};
        use objc2::msg_send;

        unsafe {
            let cls = AnyClass::get(c"NSApplication").unwrap();
            let app: *mut AnyObject = msg_send![cls, sharedApplication];
            let _: () = msg_send![&*app, activateIgnoringOtherApps: true];
        }
    }
}

/// Make window + webview background fully transparent (for rounded corners)
pub fn set_transparent_background(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::{AnyClass, AnyObject};
        use objc2::msg_send;

        // Make NSWindow transparent
        if let Ok(ptr) = window.ns_window() {
            unsafe {
                let win = ptr as *mut AnyObject;
                let cls = AnyClass::get(c"NSColor").unwrap();
                let clear: *mut AnyObject = msg_send![cls, clearColor];
                let _: () = msg_send![&*win, setBackgroundColor: clear];
                let _: () = msg_send![&*win, setOpaque: false];
                let _: () = msg_send![&*win, setHasShadow: true];
            }
        }

        // Make WKWebView transparent
        if let Ok(ptr) = window.ns_view() {
            unsafe {
                let view = ptr as *mut AnyObject;
                let color_cls = AnyClass::get(c"NSColor").unwrap();
                let clear: *mut AnyObject = msg_send![color_cls, clearColor];

                // Try underPageBackgroundColor (macOS 12+)
                let sel = objc2::sel!(setUnderPageBackgroundColor:);
                let responds: bool = msg_send![&*view, respondsToSelector: sel];
                if responds {
                    let _: () = msg_send![&*view, setUnderPageBackgroundColor: clear];
                }

                // Also try _setDrawsBackground: on view and subviews
                let draw_sel = objc2::sel!(_setDrawsBackground:);
                let responds2: bool = msg_send![&*view, respondsToSelector: draw_sel];
                if responds2 {
                    let _: () = msg_send![&*view, _setDrawsBackground: false];
                }
                set_subviews_transparent(view, clear, draw_sel);
            }
        }
    }
}

/// Hide the app and return focus to the previous app (prevents Space switching)
pub fn hide_app() {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::{AnyClass, AnyObject};
        use objc2::msg_send;

        unsafe {
            let cls = AnyClass::get(c"NSApplication").unwrap();
            let app: *mut AnyObject = msg_send![cls, sharedApplication];
            let _: () = msg_send![&*app, hide: std::ptr::null::<AnyObject>()];
        }
    }
}

/// ESC detection via CGEventTap — system-level event interception.
/// Works regardless of activation policy, app focus, or window state.
/// Requires Accessibility permission (which Glimpse already requests).
use std::ffi::c_void;
use std::sync::OnceLock;

struct TapPtrs {
    port: *mut c_void,
    source: *mut c_void,
}
unsafe impl Send for TapPtrs {}
unsafe impl Sync for TapPtrs {}

static ESC_TAP: std::sync::Mutex<Option<TapPtrs>> = std::sync::Mutex::new(None);
static ESC_TAP_APP: OnceLock<std::sync::Mutex<Option<tauri::AppHandle>>> = OnceLock::new();

#[cfg(target_os = "macos")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32, place: u32, options: u32, events_of_interest: u64,
        callback: extern "C" fn(*mut c_void, u32, *mut c_void, *mut c_void) -> *mut c_void,
        user_info: *mut c_void,
    ) -> *mut c_void;
    fn CFMachPortCreateRunLoopSource(alloc: *const c_void, port: *mut c_void, order: i64) -> *mut c_void;
    fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);
    fn CFRunLoopRemoveSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);
    fn CFRunLoopGetMain() -> *mut c_void;
    fn CGEventTapEnable(tap: *mut c_void, enable: bool);
    fn CGEventGetIntegerValueField(event: *mut c_void, field: u32) -> i64;
    fn CFRelease(cf: *const c_void);
    static kCFRunLoopCommonModes: *const c_void;
}

#[cfg(target_os = "macos")]
extern "C" fn esc_tap_callback(
    _proxy: *mut c_void, event_type: u32, event: *mut c_void, _user_info: *mut c_void,
) -> *mut c_void {
    // Re-enable tap if system disabled it (timeout)
    // kCGEventTapDisabledByTimeout = 0xFFFFFFFE
    if event_type == 0xFFFFFFFE {
        eprintln!("[native_mac] CGEventTap was disabled by timeout, re-enabling");
        if let Some(ref ptrs) = *ESC_TAP.lock().unwrap() {
            unsafe { CGEventTapEnable(ptrs.port, true); }
        }
        return event;
    }
    // kCGEventKeyDown = 10
    if event_type == 10 {
        // kCGKeyboardEventKeycode = 9
        let keycode = unsafe { CGEventGetIntegerValueField(event, 9) };
        if keycode == 53 {
            eprintln!("[native_mac] ESC detected via CGEventTap!");
            let mutex = ESC_TAP_APP.get_or_init(|| std::sync::Mutex::new(None));
            if let Some(app) = mutex.lock().unwrap().clone() {
                crate::windows::close_overlay(app);
            }
        }
    }
    event
}

pub fn install_esc_monitor(app: tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    {
        let mutex = ESC_TAP_APP.get_or_init(|| std::sync::Mutex::new(None));
        *mutex.lock().unwrap() = Some(app);

        remove_esc_monitor();

        unsafe {
            // CGEventMaskBit(kCGEventKeyDown) = 1 << 10
            let tap = CGEventTapCreate(
                1,    // kCGSessionEventTap
                0,    // kCGHeadInsertEventTap
                1,    // kCGEventTapOptionListenOnly
                1u64 << 10,
                esc_tap_callback,
                std::ptr::null_mut(),
            );
            if tap.is_null() {
                eprintln!("[native_mac] CGEventTapCreate FAILED — grant Accessibility permission");
                return;
            }

            let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
            if source.is_null() {
                eprintln!("[native_mac] CFMachPortCreateRunLoopSource failed");
                CFRelease(tap as *const c_void);
                return;
            }

            let rl = CFRunLoopGetMain();
            CFRunLoopAddSource(rl, source, kCFRunLoopCommonModes);
            CGEventTapEnable(tap, true);

            *ESC_TAP.lock().unwrap() = Some(TapPtrs { port: tap, source });
            eprintln!("[native_mac] ESC CGEventTap installed");
        }
    }
}

pub fn remove_esc_monitor() {
    #[cfg(target_os = "macos")]
    {
        let mut guard = ESC_TAP.lock().unwrap();
        if let Some(ptrs) = guard.take() {
            unsafe {
                CGEventTapEnable(ptrs.port, false);
                let rl = CFRunLoopGetMain();
                CFRunLoopRemoveSource(rl, ptrs.source, kCFRunLoopCommonModes);
                CFRelease(ptrs.source as *const c_void);
                CFRelease(ptrs.port as *const c_void);
            }
            eprintln!("[native_mac] ESC CGEventTap removed");
        }
    }
}

/// Set window level to floating
pub fn set_window_level_floating(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use objc2::runtime::AnyObject;
        use objc2::msg_send;

        let ns_window = match window.ns_window() {
            Ok(ptr) => ptr,
            Err(_) => return,
        };

        unsafe {
            let win = ns_window as *mut AnyObject;
            let level: i64 = 3; // kCGFloatingWindowLevel
            let _: () = msg_send![&*win, setLevel: level];
        }
    }
}
