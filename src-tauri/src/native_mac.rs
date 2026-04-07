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
                // CanJoinAllSpaces (1) | FullScreenAuxiliary (256)
                let behavior: u64 = 1 | 256;
                let _: () = msg_send![&*win, setCollectionBehavior: behavior];
                eprintln!("[native_mac] set collection behavior");
            } else {
                let behavior: u64 = 0;
                let _: () = msg_send![&*win, setCollectionBehavior: behavior];
            }
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

/// Set window level to screen-saver level (above everything)
pub fn set_window_level_screen_saver(window: &tauri::WebviewWindow) {
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
            let level: i64 = 1000; // kCGScreenSaverWindowLevel
            let _: () = msg_send![&*win, setLevel: level];
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
