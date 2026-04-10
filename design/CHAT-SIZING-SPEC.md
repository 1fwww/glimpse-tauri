# Chat Panel Sizing & Expansion Spec

## Principle

**"先安静，后展开"** — Chat panel starts as small as visually comfortable, then expands smoothly when content justifies it. Minimize disruption to the user's flow. No unexpected jumps, flashes, or position shifts.

---

## Rules

1. **Bottom-anchored expansion** — Input box position NEVER moves during expansion. Panel grows upward only.
2. **No empty space** — Initial height is determined by content, not a fixed number. No large blank areas.
3. **Content justifies size** — Panel is only as big as its content requires + comfortable padding.
4. **Smooth, single-direction** — All size changes animate upward with ease-out. Never shrink during a conversation.
5. **Instant for existing content** — When opening a chat that already has messages, skip animation — show at correct size immediately.

---

## Scenarios

### A. New Chat — Empty (no attachment, no quote)

**Initial height: ~140px**
- Header: 48px
- Input area: 52px (textarea + padding)
- Breathing room: 40px
- Total: ~140px

The panel is barely more than a search bar. Maximum lightness.

**Expansion trigger:** AI response arrives. Currently there is NO streaming — `chatWithAI` returns the full response in one shot (`ChatPanel.jsx:303`). Expansion happens when `result.success` is true (`ChatPanel.jsx:317-318`). Keep this trigger but:
- Start expansion BEFORE setting messages (so panel grows while content fades in)
- If streaming is added later, trigger on first chunk instead

**Expansion target:** 480px (or less if screen height is limited).

**Animation:** 350ms, `cubic-bezier(0.22, 1, 0.36, 1)` (ease-out with slight overshoot). Panel grows upward, input box stays put.

**After expansion:** If content exceeds panel height, it scrolls. Current `scrollToLastAssistant()` at 550ms delay (`ChatPanel.jsx:319`) handles this.

### B. New Chat — With Text Quote (Cmd+Shift+X)

**Initial height: ~200px**
- Same as empty + snippet preview (~60px)
- Total: ~200px

**CRITICAL — the "突然变长" bug:**

Current flow (`lib.rs:471-487` + `ChatPanel.jsx:129-132`):
1. `handle_chat_shortcut` grabs selected text
2. Calls `create_chat_window` which sets size to 432×320 (`windows.rs:197`)
3. Window shows at 320px
4. Frontend receives `text-context` event
5. `useEffect` fires `resizeChatWindow({ width: 420, height: 480 })` (`ChatPanel.jsx:131`)
6. Window animates 320→480 — **user sees the jump**

**Fix — two parts:**

**Part 1 (Rust):** `handle_chat_shortcut` knows it has text context before showing the window. Pass a height hint:

```rust
// lib.rs line 483 — change:
let _ = windows::create_chat_window(&app_clone);
// to:
let has_context = !selected_text.is_empty();
let _ = windows::create_chat_window(&app_clone, has_context);
```

**Part 2 (Rust):** `create_chat_window` uses the hint:

```rust
// windows.rs line 179 — add parameter:
pub fn create_chat_window(app: &AppHandle, has_context: bool) -> Result<(), ...> {
    // ...
    // line 197 — change:
    let h = if has_context { 480.0 } else { 320.0 };  // TODO: use CHAT_SIZES constants
    let _ = w.set_size(tauri::Size::Logical(tauri::LogicalSize { width: 432.0, height: h }));
```

**Part 3 (JS):** Remove the reactive resize in `ChatPanel.jsx:129-132` — it's no longer needed since the window starts at the right size. Or keep it as a fallback but add a check:

```javascript
// ChatPanel.jsx line 129-132 — change to:
useEffect(() => {
  if (textContext && messages.length === 0) {
    // Only resize if window is too small (fallback for cases where Rust didn't get the hint)
    // The window should already be the right size from create_chat_window
    window.electronAPI?.resizeChatWindow?.({ width: 420, height: 480 })
  }
}, [textContext])
```

**Note:** All callers of `create_chat_window` need updating:
- `lib.rs:189` (tray menu "New Chat") — `create_chat_window(&app_tray, false)`
- `lib.rs:230` (startup) — `create_chat_window(app, false)`
- `lib.rs:483` (Cmd+Shift+X) — `create_chat_window(&app_clone, has_context)`

And `prewarm_chat` (`windows.rs:146`) stays at the compact size since it's just pre-warming.

### C. New Chat — With Screenshot (overlay mode)

**Initial height: ~160px**
- Same as empty + "Screenshot attached" bar (~20px)
- Total: ~160px

In overlay mode this is CSS-driven (not native window resize). The chat panel is positioned via `getChatPosition()` in `App.jsx:537-572`. Change:

```javascript
// App.jsx line 543 — change:
const compactHeight = 320
// to:
const compactHeight = screenshotAttached ? 160 : 140
```

The `maxHeight` style on `.chat-panel` starts at this compact value, then transitions to expanded on AI reply via the existing CSS `max-height` transition.

### D. Existing Chat — Has Messages

**Initial height: `min(contentHeight + chrome, expandedMax)`**

No animation. Panel appears at the correct size immediately. User opened an existing workspace — it should feel like reopening a document, not launching a new tool.

- Short conversation (fits in <400px): show at content height
- Long conversation: show at expandedMax (480-550px), scrolled to bottom (last message visible)

**Implementation:** When opening an existing thread (user clicks a thread in tray menu, or the chat window has `messages.length > 0` on mount), measure content and resize immediately:

```javascript
// In ChatPanel.jsx, add after messages load:
useEffect(() => {
  if (messages.length > 0 && messagesContainerRef.current) {
    const contentH = messagesContainerRef.current.scrollHeight
    const chromeH = CHAT_SIZES.HEADER_HEIGHT + CHAT_SIZES.INPUT_HEIGHT + CHAT_SIZES.PADDING
    const targetH = Math.min(contentH + chromeH, CHAT_SIZES.EXPANDED_MAX)
    window.electronAPI?.resizeChatWindow?.({ width: 420, height: targetH, force: true })
    setChatFullSize(true)
  }
}, [messages.length > 0])  // only on first load, not every message
```

---

## Height Constants

Replace scattered magic numbers with a single source of truth:

```javascript
const CHAT_SIZES = {
  // Initial compact heights (by content type)
  COMPACT_EMPTY: 140,       // header + input + padding
  COMPACT_QUOTE: 200,       // + snippet preview
  COMPACT_SCREENSHOT: 160,  // + attachment bar
  
  // Expanded
  EXPANDED_DEFAULT: 480,    // standard reading height
  EXPANDED_MAX: 550,        // absolute max before scrolling
  
  // Constraints
  MIN_HEIGHT: 120,          // never smaller than this
  MAX_HEIGHT_RATIO: 0.7,    // never taller than 70% of screen
  
  // Width
  COMPACT_WIDTH: 380,
  EXPANDED_WIDTH: 420,
  
  // Chrome (non-content areas)
  HEADER_HEIGHT: 48,
  INPUT_HEIGHT: 52,
  PADDING: 40,
  SNIPPET_HEIGHT: 60,
  ATTACHMENT_HEIGHT: 20,
}
```

---

## Expansion Animation

### Overlay mode (CSS-driven)

```css
.chat-panel {
  transition: max-height 350ms cubic-bezier(0.22, 1, 0.36, 1);
}
```

Panel uses `max-height` transition. Initial `max-height` = compact size. On expansion, `max-height` transitions to expanded size. Content fills naturally.

**Do NOT transition `height` directly** — use `max-height` so the panel can be smaller than max when content is short.

### Standalone mode (native window)

In `windows.rs` `resize_chat_window` (line 786-825), adjust the y-coordinate to expand upward:

```rust
// Around line 800-813, after calculating width/height, before animate_frame:
if let Ok(pos) = w.outer_position() {
    let mut x = pos.x as f64 / scale;
    let mut y = pos.y as f64 / scale;
    
    // *** ADD THIS: expand upward (bottom stays fixed) ***
    let grow = height - cur_h;
    if grow > 0.0 {
        y -= grow;
    }
    
    // Clamp to monitor bounds (existing code)
    if let Some(m) = w.current_monitor().ok().flatten() {
        // ... existing clamp logic ...
    }
    // ... animate_frame call ...
}
```

This keeps the bottom edge (input box) at the same screen position. The `y -= grow` must happen BEFORE the monitor bounds clamping so the window doesn't go above the screen top.

---

## Expansion Trigger

**Current state:** No streaming. `chatWithAI` (`ChatPanel.jsx:303`) returns the full response in one `invoke` call. Expansion currently happens at `ChatPanel.jsx:317-318` after the response is received.

**Change:** Move the expansion to BEFORE setting messages, so the panel grows while the response fades in:

```javascript
// ChatPanel.jsx, around line 312-318. Change from:
setMessages(prev => [...prev, { role: 'assistant', text: assistantText, model: currentModelName }])
if (!chatFullSize) setChatFullSize(true)
window.electronAPI?.resizeChatWindow?.({ width: 420, height: 550 })

// To:
if (!chatFullSize) {
  setChatFullSize(true)
  const screenH = window.innerHeight
  const targetH = Math.min(CHAT_SIZES.EXPANDED_DEFAULT, screenH * CHAT_SIZES.MAX_HEIGHT_RATIO)
  window.electronAPI?.resizeChatWindow?.({ width: CHAT_SIZES.EXPANDED_WIDTH, height: targetH })
}
// Small delay so expansion starts before content appears
setTimeout(() => {
  setMessages(prev => [...prev, { role: 'assistant', text: assistantText, model: currentModelName }])
  setTimeout(() => scrollToLastAssistant(), 350)
}, 50)
```

**If streaming is added later:** Trigger expansion on first chunk instead of before `setMessages`. The principle is the same: panel grows WHILE content appears, not after.

---

## "Only Grow" Policy — Revised

Current behavior: window never shrinks (`windows.rs:790`).

**New behavior:**
- **During a conversation:** Never shrink (current behavior, keep it).
- **On new chat:** Allow shrinking back to compact size. When user starts a new thread, reset to compact.
- **On open existing chat:** Set to content-appropriate size (may be smaller or larger than current).

```rust
// windows.rs line 786 — add `force` parameter from JSON:
pub fn resize_chat_window(app: AppHandle, size: serde_json::Value) {
    let force = size.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    // ...
    // line 795 — change the only-grow logic:
    let width = if force { new_w } else { new_w.max(cur_w) };
    let height = if force { new_h } else { new_h.max(cur_h) };
}
```

Also update `tauri-shim.js` to pass force flag:
```javascript
resizeChatWindow: (opts) => invoke('resize_chat_window', { size: opts }),
// Usage: window.electronAPI.resizeChatWindow({ width: 420, height: 140, force: true })
```

---

## Quote Jump Fix (P0)

The most urgent fix. Current flow:

1. User presses Cmd+Shift+X with text selected
2. Chat window created at 432×320 (`windows.rs:153`)
3. Window shows at 320px
4. `useEffect` fires, calls `resizeChatWindow({ width: 420, height: 480 })`
5. Window animates from 320→480 — **user sees the jump**

**Fix:** Pass the initial context info to `create_chat_window` so it can size correctly from the start:

```rust
// In create_chat_window, accept a height hint:
pub fn create_chat_window(app: &AppHandle, height_hint: Option<f64>) {
    let h = height_hint.unwrap_or(320.0);  // was hardcoded 320
    // ... create window with h ...
}
```

The caller (in `lib.rs`) knows whether there's a text context and can pass the appropriate height. Window is created at the right size — no jump.

---

## Screen Bounds

Keep existing logic but use the new constants:

```javascript
const getChatPosition = () => {
  const screenH = window.innerHeight
  const maxH = Math.min(
    chatFullSize ? CHAT_SIZES.EXPANDED_DEFAULT : getCompactHeight(),
    screenH * CHAT_SIZES.MAX_HEIGHT_RATIO
  )
  // ... rest of positioning logic ...
}

function getCompactHeight() {
  if (textContext) return CHAT_SIZES.COMPACT_QUOTE
  if (screenshotAttached) return CHAT_SIZES.COMPACT_SCREENSHOT
  return CHAT_SIZES.COMPACT_EMPTY
}
```

---

## Implementation Priority

1. **P0: Fix quote jump** — See Scenario B above. Changes in:
   - `lib.rs:483` — pass `has_context` bool to `create_chat_window`
   - `lib.rs:189, 230` — update other callers to pass `false`
   - `windows.rs:179` — accept `has_context` param, use to set initial height
   - `windows.rs:197` — use `480.0` when has_context, else compact height
   - `ChatPanel.jsx:129-132` — keep as fallback, no longer primary resize path

2. **P1: Reduce initial compact height** — Changes in:
   - `windows.rs:153` — prewarm at compact height (140) instead of 320
   - `windows.rs:197` — reset to compact height instead of 320
   - `App.jsx:543` — `compactHeight = 140` (was 320), or content-aware via `getCompactHeight()`
   - New: `CHAT_SIZES` constants object in a shared location (e.g. `src/constants.js`)

3. **P1: Bottom-anchored expansion** — Changes in:
   - `windows.rs:800-813` — add `y -= (height - cur_h)` before monitor clamping
   - Only affects standalone mode; overlay mode already uses CSS `bottom` positioning

4. **P2: Expand before content** — Changes in:
   - `ChatPanel.jsx:312-319` — reorder: expand first, then `setMessages` with 50ms delay
   - See "Expansion Trigger" section above for exact code

5. **P2: Existing chat sizing** — Changes in:
   - `ChatPanel.jsx` — add useEffect on initial `messages.length > 0` to measure + force-resize
   - `windows.rs:786` — support `force: true` in resize_chat_window (see "Only Grow Policy" section)

6. **P3: Gradual growth** — Lower priority. Use ResizeObserver on messages container, grow panel as content fills, cap at `screenH * 0.7`.
