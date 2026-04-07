use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFDictionaryRef;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_graphics::display::{
    kCGNullWindowID, kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
    CGWindowListCopyWindowInfo,
};
use serde_json::{json, Value};

/// Get bounds of all visible windows on screen.
/// Replaces the external `get-windows` binary from the Electron version.
pub fn get_window_bounds() -> Vec<Value> {
    let mut results = Vec::new();

    unsafe {
        let window_list = CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            kCGNullWindowID,
        );

        if window_list.is_null() {
            return results;
        }

        let count = core_foundation::array::CFArray::<CFType>::wrap_under_get_rule(
            window_list as core_foundation::array::CFArrayRef,
        )
        .len();

        let arr = core_foundation::array::CFArray::<CFType>::wrap_under_get_rule(
            window_list as core_foundation::array::CFArrayRef,
        );

        for i in 0..count {
            let Some(item) = arr.get(i) else { continue };
            let dict_ref = item.as_CFTypeRef() as CFDictionaryRef;

            // Get window owner (app name)
            let owner = get_dict_string(dict_ref, "kCGWindowOwnerName").unwrap_or_default();

            // Skip our own app and system UI elements
            if owner == "Glimpse" || owner == "Window Server" || owner == "SystemUIServer" {
                continue;
            }

            // Get window layer — only include layer 0 (normal windows)
            let layer = get_dict_number(dict_ref, "kCGWindowLayer").unwrap_or(-1);
            if layer != 0 {
                continue;
            }

            // Get window name
            let name = get_dict_string(dict_ref, "kCGWindowName").unwrap_or_default();

            // Get bounds
            if let Some(bounds) = get_dict_bounds(dict_ref) {
                let (x, y, w, h) = bounds;
                // Skip tiny windows
                if w < 50.0 || h < 50.0 {
                    continue;
                }
                results.push(json!({
                    "owner": owner,
                    "name": name,
                    "x": x as i32,
                    "y": y as i32,
                    "w": w as i32,
                    "h": h as i32,
                }));
            }
        }
    }

    results
}

unsafe fn get_dict_string(dict: CFDictionaryRef, key: &str) -> Option<String> {
    let cf_key = CFString::new(key);
    let mut value: *const core::ffi::c_void = std::ptr::null();
    if core_foundation::dictionary::CFDictionaryGetValueIfPresent(
        dict,
        cf_key.as_concrete_TypeRef() as *const _,
        &mut value,
    ) != 0
    {
        if !value.is_null() {
            let cf_str =
                core_foundation::string::CFString::wrap_under_get_rule(value as core_foundation::string::CFStringRef);
            return Some(cf_str.to_string());
        }
    }
    None
}

unsafe fn get_dict_number(dict: CFDictionaryRef, key: &str) -> Option<i64> {
    let cf_key = CFString::new(key);
    let mut value: *const core::ffi::c_void = std::ptr::null();
    if core_foundation::dictionary::CFDictionaryGetValueIfPresent(
        dict,
        cf_key.as_concrete_TypeRef() as *const _,
        &mut value,
    ) != 0
    {
        if !value.is_null() {
            let cf_num =
                CFNumber::wrap_under_get_rule(value as core_foundation::number::CFNumberRef);
            return cf_num.to_i64();
        }
    }
    None
}

unsafe fn get_dict_bounds(dict: CFDictionaryRef) -> Option<(f64, f64, f64, f64)> {
    let cf_key = CFString::new("kCGWindowBounds");
    let mut value: *const core::ffi::c_void = std::ptr::null();
    if core_foundation::dictionary::CFDictionaryGetValueIfPresent(
        dict,
        cf_key.as_concrete_TypeRef() as *const _,
        &mut value,
    ) != 0
    {
        if !value.is_null() {
            let bounds_dict = value as CFDictionaryRef;
            let x = get_dict_cf_number(bounds_dict, "X")?;
            let y = get_dict_cf_number(bounds_dict, "Y")?;
            let w = get_dict_cf_number(bounds_dict, "Width")?;
            let h = get_dict_cf_number(bounds_dict, "Height")?;
            return Some((x, y, w, h));
        }
    }
    None
}

unsafe fn get_dict_cf_number(dict: CFDictionaryRef, key: &str) -> Option<f64> {
    let cf_key = CFString::new(key);
    let mut value: *const core::ffi::c_void = std::ptr::null();
    if core_foundation::dictionary::CFDictionaryGetValueIfPresent(
        dict,
        cf_key.as_concrete_TypeRef() as *const _,
        &mut value,
    ) != 0
    {
        if !value.is_null() {
            let cf_num =
                CFNumber::wrap_under_get_rule(value as core_foundation::number::CFNumberRef);
            return cf_num.to_f64();
        }
    }
    None
}
