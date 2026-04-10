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

**Expansion trigger:** AI starts streaming first token (NOT after full response).

**Expansion target:** 480px (or less if screen height is limited).

**Animation:** 350ms, `cubic-bezier(0.22, 1, 0.36, 1)` (ease-out with slight overshoot). Panel grows upward, input box stays put.

**After expansion:** If AI response continues to fill space, panel can grow further up to `maxHeight` (screen height × 70%), then content scrolls.

### B. New Chat — With Text Quote (Cmd+Shift+X)

**Initial height: ~200px**
- Same as empty + snippet preview (~60px)
- Total: ~200px

**CRITICAL:** Window must be created at this height from the start. Currently the window opens at 320px then jumps to 480px — this is the "突然变长" problem. The Rust side (`create_chat_window`) should accept an initial height parameter based on whether context is attached.

**Expansion:** Same as empty — on AI first token, grow to 480px.

### C. New Chat — With Screenshot (overlay mode)

**Initial height: ~160px**
- Same as empty + "Screenshot attached" bar (~20px)
- Total: ~160px

In overlay mode this is CSS-driven (not native window resize). The `.chat-panel` height should be set to `auto` with a `max-height` that starts at 160px and transitions to expanded on AI reply.

**Expansion:** Same trigger and animation.

### D. Existing Chat — Has Messages

**Initial height: `min(contentHeight + chrome, expandedMax)`**

No animation. Panel appears at the correct size immediately. User opened an existing workspace — it should feel like reopening a document, not launching a new tool.

- Short conversation (fits in <400px): show at content height
- Long conversation: show at expandedMax (480-550px), scrolled to bottom (last message visible)

**Implementation:** After mounting, measure `scrollHeight` of messages area, compute total needed height, set window size in one frame before showing the window.

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

Use `animate_frame` but **adjust y-coordinate** to expand upward:

```rust
// In resize_chat_window:
let grow = new_h - cur_h;
if grow > 0.0 {
    y -= grow;  // Move window up by the growth amount = bottom stays fixed
}
```

This keeps the bottom edge (input box) at the same screen position.

---

## Expansion Trigger — Streaming First Token

Currently expansion happens AFTER the full AI response (`ChatPanel.jsx:312`). Change to trigger on first streaming chunk:

```javascript
// In the streaming handler (or first message chunk received):
if (!chatFullSize && messages.length === 0) {
  setChatFullSize(true)
  // Expand window
  const targetH = Math.min(CHAT_SIZES.EXPANDED_DEFAULT, screenH * CHAT_SIZES.MAX_HEIGHT_RATIO)
  window.electronAPI?.resizeChatWindow?.({ 
    width: CHAT_SIZES.EXPANDED_WIDTH, 
    height: targetH 
  })
}
```

The expansion and content appearance happen simultaneously — the panel "grows to make room" for the incoming text.

---

## "Only Grow" Policy — Revised

Current behavior: window never shrinks (`windows.rs:790`).

**New behavior:**
- **During a conversation:** Never shrink (current behavior, keep it).
- **On new chat:** Allow shrinking back to compact size. When user starts a new thread, reset to compact.
- **On open existing chat:** Set to content-appropriate size (may be smaller or larger than current).

```rust
// Add a `force` parameter to resize_chat_window:
pub fn resize_chat_window(app: AppHandle, size: serde_json::Value) {
    let force = size.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    // ... existing logic ...
    if !force {
        // Only grow (current behavior)
        let width = new_w.max(cur_w);
        let height = new_h.max(cur_h);
    } else {
        // Force exact size (for new chat / open existing)
        let width = new_w;
        let height = new_h;
    }
}
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

1. **P0: Fix quote jump** — Pass height hint to `create_chat_window`. Eliminates the most visible jank.
2. **P1: Reduce initial compact height** — From 320px to content-appropriate (~140-200px). Extract `CHAT_SIZES` constants.
3. **P1: Bottom-anchored expansion in standalone mode** — Adjust y-coordinate in `animate_frame` call.
4. **P2: Stream-trigger expansion** — Move `setChatFullSize(true)` from response-complete to first-token.
5. **P2: Existing chat sizing** — Measure content height on mount, set window size before show.
6. **P3: Gradual growth** — ResizeObserver on messages area, grow panel as content fills, up to max.
