# Chat Sizing Hotfix — Implementation Bugs

These are bugs in the current implementation of CHAT-SIZING-SPEC.md. Fix these before continuing with other changes.

---

## Bug 1: Existing chat sizing uses undefined constants

**File:** `ChatPanel.jsx` ~line 225
**Problem:** 
```javascript
const chromeH = CHAT_SIZES.HEADER_HEIGHT + CHAT_SIZES.INPUT_HEIGHT + CHAT_SIZES.PADDING
```
These three fields (`HEADER_HEIGHT`, `INPUT_HEIGHT`, `PADDING`) do NOT exist in `CHAT_SIZES`. The result is `NaN`, making the resize call a no-op. Existing chats never resize.

**Fix:**
```javascript
const chromeH = CHAT_SIZES.CHROME_FIXED  // = 183, already includes header + input + actions + window padding
```

Full corrected block:
```javascript
useEffect(() => {
  if (messages.length > 0 && !initialSizeSet.current && messagesContainerRef.current) {
    initialSizeSet.current = true
    const contentH = messagesContainerRef.current.scrollHeight
    const chromeH = CHAT_SIZES.CHROME_FIXED
    const targetH = Math.min(contentH + chromeH, CHAT_SIZES.EXPANDED_MAX)
    window.electronAPI?.resizeChatWindow?.({ width: CHAT_SIZES.EXPANDED_WIDTH, height: targetH, force: true })
    setChatFullSize(true)
  }
  if (messages.length === 0) {
    initialSizeSet.current = false
  }
}, [messages.length > 0])
```

---

## Bug 2: Quote compact height (300px) too small for thread-actions bar

**Problem:** `COMPACT_QUOTE: 300` was calculated as `280 + 37 (snippet) - 17 (breathing)`. But in practice, when a long quote text is present, the snippet box can be taller than 37px (it has `max-height: 120px` with scroll). Also the model selector / thread-actions bar at the bottom may get clipped.

**Fix:** Increase `COMPACT_QUOTE` to 320:
```javascript
COMPACT_QUOTE: 320,       // 183 chrome + 80 messages min + 37 snippet + 20 safety
```

Also update the Rust height hint in `lib.rs:482`:
```rust
let height_hint = if has_context { Some(320.0) } else { None };
```

---

## Bug 3: Fresh window creation uses 140px (not 280px)

**File:** `windows.rs` ~line 262
**Problem:**
```rust
let fresh_h = height_hint.unwrap_or(140.0);
```
When creating a fresh window (non pre-warmed, e.g. on fullscreen Space), the fallback height is 140px — way too small. This was a leftover from the old spec values.

**Fix:**
```rust
let fresh_h = height_hint.unwrap_or(280.0);
```

---

## Bug 4: User message sending causes visual "expand"

**Problem:** When user sends a message, `scrollToBottom()` is called. In compact mode (280px), the user's message appears in the 80px messages area. If the message is longer than 80px, it overflows and the scroll triggers — this looks like the panel "grew" even though it didn't actually resize.

This is not a resize bug — it's a visual illusion from content overflow in a small area. The real fix is already in the spec: expand should happen on AI first response, not on user message send.

**Additional safeguard:** Don't call `scrollToBottom()` on user message send if panel is still compact. Let the messages area scroll naturally — the user just typed it, they know what's there. Only auto-scroll when AI responds and panel expands.

---

## Bug 5: `chatFullSize` state not reset on new chat

**Problem:** If user had an expanded chat, then starts a new thread, `chatFullSize` may still be `true`. The expansion logic (`if (!chatFullSize)`) then never triggers for the new chat.

**Fix:** When starting a new thread / clearing chat, reset:
```javascript
// In the new thread handler:
setChatFullSize(false)
window.electronAPI?.resizeChatWindow?.({ 
  width: CHAT_SIZES.COMPACT_WIDTH, 
  height: CHAT_SIZES.COMPACT_EMPTY, 
  force: true 
})
```

---

## Priority

1. **Bug 1** (NaN) — This breaks all existing chat sizing. Fix first.
2. **Bug 3** (140px fallback) — Causes tiny window on fullscreen Spaces.
3. **Bug 2** (quote height) — Causes clipping with text quotes.
4. **Bug 5** (chatFullSize not reset) — Prevents compact→expand cycle on new chats.
5. **Bug 4** (scroll visual) — Lower priority, cosmetic.
