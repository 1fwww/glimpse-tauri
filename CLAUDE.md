# Glimpse for Tauri — Developer Guide

## Project Overview
macOS screenshot + AI chat tool. Tauri v2 + React + Vite frontend, Rust backend with native macOS APIs (objc2).

## Build & Run

```bash
# Dev mode (frontend hot reload, but input events don't work — see Dev Limitations)
npx tauri dev

# Debug .app bundle (for testing ESC, hover, input events)
npx tauri build --debug
open src-tauri/target/debug/bundle/macos/Glimpse.app

# Release build (must set API key env vars)
GLIMPSE_ANTHROPIC_KEY="..." GLIMPSE_OPENAI_KEY="..." GLIMPSE_GEMINI_KEY="..." \
  npx tauri build --target universal-apple-darwin
```

## Restart Protocol
Always kill all processes before restarting:
```bash
pkill -f "[Gg]limpse"; lsof -ti:5173 | xargs kill -9
```

## Dev Mode Limitations
`npx tauri dev` runs a bare binary (not .app bundle). macOS does NOT route input events (mouse, keyboard) to bare binaries the same way as .app bundles. Symptoms:
- Overlay doesn't receive hover/click events before user clicks on it
- ESC before selection doesn't work
- CGEventTap may not fire

**Do NOT debug input event issues in dev mode.** Use `npx tauri build --debug` and test with the .app bundle.

## Architecture

### Key Files
- `src-tauri/src/lib.rs` — App entry, global shortcuts (Cmd+Shift+Z screenshot, Cmd+Shift+X chat), screenshot/chat handler flows
- `src-tauri/src/windows.rs` — Window creation/management (home, overlay, chat, settings, toast, welcome)
- `src-tauri/src/native_mac.rs` — Native macOS APIs: activate_app, CGEventTap ESC, window level, collection behavior
- `src-tauri/src/commands/` — Tauri commands (ai, settings, threads, permissions)
- `src/tauri-shim.js` — Bridges `window.electronAPI` to Tauri `invoke`/`listen`
- `src/App.jsx` — Screenshot overlay (selection, annotations, chat panel)
- `src/ChatOnlyApp.jsx` — Standalone chat window (Cmd+Shift+X)

### Window Lifecycle
- **Overlay**: Pre-warmed at startup. On close: hide (for reuse) → 300ms later destroy + prewarm fresh (for fullscreen Space association).
- **Home**: Created on app launch, closed during screenshot/chat. `create_home_window` restores Regular activation policy.
- **Chat**: Created on demand. `CHAT_READY` atomic flag signals frontend loaded.
- **Toast**: Temp HTML file, auto-closes after 1.8s.

## Corner Cases & Hard-Won Lessons

### Fullscreen Space Support
**User experience**: User is in a fullscreen app (e.g., Safari), presses Cmd+Shift+Z. Overlay must appear ON TOP of the fullscreen app, not on another desktop.

**How it works**: Switch to Accessory activation policy before capture. This lets the overlay appear on fullscreen Spaces.

**What breaks it**:
- Accessory switch must be SYNCHRONOUS (with spin-wait) before capture. Fire-and-forget causes overlay to appear on wrong Space.
- `set_activation_policy(Regular)` in close_overlay causes macOS to switch the user to another desktop. Never restore Regular directly — use `hide_app()` and let `create_home_window` handle it.
- `MoveToActiveSpace` collection behavior is INCOMPATIBLE with `CanJoinAllSpaces` — combining them crashes the app with "Rust cannot catch foreign exceptions".
- `w.hide()` on overlay preserves the window but loses its Space association. That's why we delayed-close + prewarm fresh after 300ms.

### ESC Key Detection
**User experience**: User triggers screenshot, decides not to select, presses ESC to cancel. Must work instantly.

**How it works**: CGEventTap (`CGEventTapCreate`) intercepts keyDown events at the system level before they reach any app.

**What breaks it**:
- `NSEvent addGlobalMonitorForEventsMatchingMask` does NOT work when app is in Accessory mode.
- `NSEvent addLocalMonitorForEventsMatchingMask` also doesn't work (app isn't "active").
- `CGEventSourceKeyState` polling doesn't reliably detect ESC in dev mode.
- `activateIgnoringOtherApps` does NOT make Accessory apps receive input events.
- CGEventTap requires Accessibility permission. If `CGEventTapCreate` returns null, permission is missing.
- If the system disables the tap (timeout), the callback receives `event_type == 0xFFFFFFFE` — must re-enable with `CGEventTapEnable`.

### Overlay Reuse & "Capturing Screen" Flash
**User experience**: User takes a screenshot, closes it, immediately takes another. Should be instant with no loading screen.

**How it works**: `close_overlay` hides the window (doesn't destroy it). Next screenshot reuses the hidden overlay instantly. After 300ms, if the overlay wasn't reused, it's destroyed and a fresh one is pre-warmed.

**What breaks it**:
- If `close_overlay` emits `reset-overlay` (clears frontend state), the reused overlay shows blank "Capturing screen" because `screen-captured` event hasn't been processed yet. Don't reset state on close.
- If `close_overlay` destroys the overlay (`w.close()`), the next screenshot falls into the "no pre-warm" path which creates a fresh webview (slow, 200-800ms).
- The frontend renders `null` when `screenImage` is null (no "Capturing screen" spinner) to avoid any flash.

### Screenshot Speed
**User experience**: Pressing Cmd+Shift+Z should feel instant. Any delay >150ms is noticeable.

**Critical path**: hide windows (50ms) → capture (30-60ms) → sync activate+orderFront (max 60ms) → emit + show.

**What slows it down**:
- `run_on_main_thread` sync spin-waits. Keep iterations low (30×2ms = 60ms max). The original code had 100×5ms = 500ms.
- `capture_screen_to_memory` and `get_window_bounds` should run in parallel (separate threads).
- The "no pre-warm" path has a 200ms delay for webview init. Always try to have a pre-warmed overlay available.
- Don't set window level / collection behavior in the show path if already set during prewarm.

### Toast Background Border
**User experience**: Toast notification shows with visible border/outline artifact around the dark pill shape.

**Fix**: Disable NSWindow shadow (`setHasShadow: false`) + remove CSS box-shadow + set transparent background synchronously before show (`visible(false)` → `set_transparent_background` → `show()`).

### Welcome Flow Permissions Skip
**User experience**: New user installs app, but welcome flow skips the permissions step and goes straight to shortcuts.

**Cause**: `welcome-step` is stored in localStorage. WebView localStorage persists in `~/Library/WebKit/` even when `~/Library/Application Support/glimpse/` is deleted. Stale value skips steps.

**Fix**: On mount, check actual permission state via `checkPermissions()`. If permissions not granted and step > 1, force back to step 1.

### Screen Recording Permission Grant
**User experience**: User clicks "Grant" button, dismisses the system dialog, clicks "Grant" again — nothing happens.

**Cause**: `CGRequestScreenCaptureAccess()` only shows the system prompt on the FIRST call. Subsequent calls return false silently.

**Fix**: Track with `SCREEN_REQUESTED` atomic flag. First click: show system prompt. Second click: open System Settings directly.

### Text Quote Re-trigger
**User experience**: User quotes text via Cmd+Shift+X, dismisses it, selects the same text again, presses Cmd+Shift+X — quoted text doesn't appear.

**Cause**: `initialContext` was a plain string. React's `useEffect` doesn't re-trigger when the same string is set again.

**Fix**: Use `{ text, seq }` object with incrementing counter. `useEffect` depends on `seq`, not `text`.

### Save Dialog White Overlay
**User experience**: User clicks save, the save dialog appears but the background turns bright white.

**Cause**: The overlay window (at CGShieldingWindowLevel) covers the save dialog. The dialog's background shows through the overlay.

**Fix**: `lower_overlay()` before `blocking_save_file()`, `restore_overlay()` after.

### Pin from Fullscreen → Space Switch
**User experience**: User screenshots in fullscreen, pins the chat, presses ESC — gets teleported to another desktop.

**Cause**: `close_overlay` restored Regular policy, which triggers macOS Space switch when there's a pinned chat window.

**Fix**: Call `hide_app()` BEFORE `set_activation_policy(Regular)`. Only restore Regular when there ARE other visible windows (pinned chat).

## macOS Native API Patterns

1. **`run_on_main_thread` is async** — Use AtomicBool spin-wait when ordering matters (activate before show).
2. **`accept_first_mouse(true)`** — Required for windows that need click-through when app is unfocused.
3. **`activateIgnoringOtherApps`** — Does NOT work for Accessory apps. Does NOT work for bare binaries in dev mode.
4. **`w.close()` is async** — `get_webview_window()` may still return Some. Prefer hiding over close+recreate.
5. **Window levels**: CGShieldingWindowLevel (2147483628) for overlay. Level 3 (floating) for chat.

## TODO

### Screenshot Selection Performance (P0)
Three-step approach, do in order, stop when "good enough":
1. **rAF throttle** — Wrap mousemove handler in `requestAnimationFrame`. 5 min, zero risk. Prevents >60fps event flooding.
2. **useRef + DOM direct** — Store selection in `useRef`, update DOM elements directly (`.style.transform`), only `setState` on mouseup. Medium risk: selection bounds are read by chat panel positioning, toolbar positioning, cropped image calc, window hover detection — all need to read from ref instead of state.
3. **Native NSView selection** — Replace WebView selection with Rust/objc2 Core Graphics overlay for the selection phase only. Mouseup passes bounds to WebView for chat+annotation. Does NOT require changing activation policy, Space logic, or ESC detection — only replaces the rendering layer. 3-5 days. All major screenshot tools (CleanShot, Shottr, Lark) use native selection. This is the only way to match their smoothness.

### Screenshot → Pin Transition (P1)
~1s delay. Root cause: creates new webview + React init on every pin. Fix: pre-warm chat window at startup (hidden, offscreen) so pin only needs reposition + show. Prior attempt caused screenshot shortcut bug — needs careful window lifecycle management. Do NOT change `close_chat_window` to hide (tried, caused issues). Instead: create chat window offscreen during startup prewarm phase, after overlay prewarm.

### Tray + Home Window (P1)
- Home window removed, tray is primary interface
- After onboarding completes, dock icon disappears (Accessory mode) — user has no visible entry if tray permission not granted
- Options: (a) open chat window after onboarding, (b) guide user to enable Menu Bar permission, (c) keep Home window as fallback
- macOS Tahoe requires "Allow in Menu Bar" in System Settings → Menu Bar for tray to appear
- Tray menu has dynamic recent chats (rebuilds on thread save/delete)

### Fullscreen Space (P2)
Overlay may still appear on wrong Space after hide+delayed-close prewarm.
