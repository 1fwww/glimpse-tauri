# Glimpse Design Specification

> Hand this document + the `mockups/` directory to a Claude Code session to implement the redesign.
> Source app: Tauri v2 + React 19, single CSS file (`src/app.css`, ~3100 lines).
>
> **This is the source of truth.** Tokens are exact values — implement them literally. Component descriptions are directional — use the mockups as visual reference and derive the CSS from the tokens.

---

## 1. Design Principles

1. **Disappear when not needed.** Glimpse is a utility, not a destination.
2. **Calm over clever.** No pulsing glows, no aggressive animations. Motion should feel physical and natural.
3. **Warm, not clinical.** Soft radii, gentle contrast, muted-but-warm tones.
4. **Respect the user's focus.** Minimize cognitive load. Clear hierarchy, obvious actions.
5. **Accessible by default.** WCAG AA minimum. Color blindness safe. Reduced motion support.

---

## 2. Typography

### Fonts

| Token | Value | Usage |
|---|---|---|
| `--font-display` | `'Outfit', -apple-system, system-ui, sans-serif` | All UI text |
| `--font-mono` | `'JetBrains Mono', ui-monospace, monospace` | Code blocks, API keys, `kbd` elements, model names, dimension badge, window hover label — nothing else |

### Type Scale

| Token | Size | Usage |
|---|---|---|
| `--text-xs` | `11px` | Minimum. Timestamps, model tags, hints |
| `--text-sm` | `12px` | Secondary body, thinking indicator, button text |
| `--text-base` | `13px` | Primary body, chat messages, input text |
| `--text-md` | `14px` | Descriptions, taglines, CTA button text |
| `--text-lg` | `16px` | Section headings |
| `--text-xl` | `18px` | Page titles |
| `--text-2xl` | `22px` | Display |
| `--text-3xl` | `26px` | Large display (brand name) |

### Weights

400 (body), 500 (buttons/labels), 600 (titles), 700 (markdown bold only).

### Migration

Every element currently using `font-family: var(--font-mono)` for non-code UI text (section labels, timestamps, buttons, hints, tooltips, thinking text, footer hints, "RESET", skip links, privacy notes, model dropdown provider labels, loading text) must switch to `var(--font-display)`. Minimum size 11px. Drop all `text-transform: uppercase` and `letter-spacing` on non-code text. Change "RESET" to "Reset".

---

## 3. Color System

### Brand

| Token | Light value | Dark value | Usage |
|---|---|---|---|
| `--brand` | `#6C63FF` | `#6C63FF` | Logo, decorative indicators |
| `--brand-text` | `#5B53E0` | `#8580FF` | Links, labels, brand-colored text |
| `--brand-btn` | `#635AEE` | `#6C63FF` | Button backgrounds (white text) |
| `--brand-btn-hover` | `#5B53E0` | `#5B53E0` | Button hover |
| `--brand-muted` | `rgba(108,99,255,0.10)` | `rgba(108,99,255,0.12)` | User message bubbles, hover |
| `--brand-dim` | `rgba(108,99,255,0.06)` | `rgba(108,99,255,0.07)` | Subtle tints |
| `--brand-border` | `rgba(108,99,255,0.22)` | `rgba(108,99,255,0.25)` | Active borders, focus rings |

### Semantic

| Token | Light value | Dark value | Notes |
|---|---|---|---|
| `--success` | `#238542` | `#2DA44E` | Checks, saved states. Dark variant slightly lighter for contrast. |
| `--success-muted` | `rgba(45, 164, 78, 0.12)` | `rgba(45, 164, 78, 0.12)` | |
| `--error` | `#CF222E` | `#EF4444` | Errors, delete. **Dark needs lighter red** — `#CF222E` fails contrast on dark surfaces. |
| `--error-muted` | `rgba(207, 34, 46, 0.08)` | `rgba(239, 68, 68, 0.08)` | |

### Removed — delete entirely

`--cyan-primary`, `--cyan-dim`, `--cyan-glow`, `--cyan-subtle`, `--magenta-accent`, `--magenta-glow`. Replace all references to these with the brand tokens above.

### Light Theme — Warm Gray (default)

```css
--surface-base:      #F2F2F4;
--surface-panel:     #FAFAFC;
--surface-elevated:  #EEEEF2;
--surface-input:     #F4F4F6;
--surface-hover:     rgba(0, 0, 0, 0.035);
--border-dim:        rgba(0, 0, 0, 0.065);
--border-active:     rgba(0, 0, 0, 0.11);
--border-bright:     rgba(108, 99, 255, 0.35);
--text-primary:      rgba(18, 18, 25, 0.90);       /* 13.1:1 ✓ */
--text-secondary:    rgba(45, 45, 62, 0.72);       /*  5.2:1 ✓ */
--text-dim:          rgba(70, 70, 95, 0.65);       /*  3.4:1 ✓ AA-large */
--shadow-sm:         0 1px 2px rgba(0,0,0,0.04);
--shadow-md:         0 4px 16px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.03);
--shadow-lg:         0 8px 28px rgba(0,0,0,0.08), 0 2px 4px rgba(0,0,0,0.03);
--shadow-xl:         0 16px 40px rgba(0,0,0,0.10);
```

### Light Theme — Soft Lavender (pinned state only)

When pinned, surfaces shift to these values (see Pinned State section):

```css
--surface-base:      rgba(248, 247, 252, 0.92);    /* translucent for blur */
--surface-panel:     #FCFBFF;
--surface-elevated:  rgba(244, 242, 252, 0.95);
--surface-input:     rgba(246, 244, 253, 0.95);
--surface-hover:     rgba(108, 99, 255, 0.05);
--border-dim:        rgba(108, 99, 255, 0.08);
--border-active:     rgba(108, 99, 255, 0.12);
--border-bright:     rgba(108, 99, 255, 0.35);     /* inherited from base, same value */
--text-primary:      rgba(22, 18, 40, 0.90);       /* 13.2:1 */
--text-secondary:    rgba(50, 45, 75, 0.72);       /*  5.2:1 */
--text-dim:          rgba(75, 70, 110, 0.65);      /*  3.4:1 */
```

### Dark Theme — Brand-tinted Dark

```css
--surface-base:      #16151E;
--surface-panel:     rgba(22, 21, 30, 0.96);
--surface-elevated:  rgba(31, 29, 42, 0.92);
--surface-input:     rgba(26, 25, 36, 0.96);
--surface-hover:     rgba(108, 99, 255, 0.05);
--border-dim:        rgba(108, 99, 255, 0.08);
--border-active:     rgba(108, 99, 255, 0.13);
--border-bright:     rgba(108, 99, 255, 0.40);
--text-primary:      rgba(242, 240, 255, 0.93);    /* 14.1:1 ✓ */
--text-secondary:    rgba(175, 172, 205, 0.75);    /*  5.3:1 ✓ */
--text-dim:          rgba(140, 137, 175, 0.70);    /*  3.4:1 ✓ AA-large */
--shadow-sm:         0 1px 2px rgba(0,0,0,0.2);
--shadow-md:         0 4px 20px rgba(0,0,0,0.30), 0 1px 3px rgba(0,0,0,0.15);
--shadow-lg:         0 8px 32px rgba(0,0,0,0.35), 0 2px 6px rgba(0,0,0,0.15);
--shadow-xl:         0 16px 48px rgba(0,0,0,0.40);
```

---

## 4. Spacing & Radii

**Spacing:** `--space-1` (4px), `--space-2` (8), `--space-3` (12), `--space-4` (16), `--space-5` (20), `--space-6` (24), `--space-8` (32), `--space-10` (40)

**Radii:** `--radius-sm` (8px), `--radius-md` (12), `--radius-lg` (16), `--radius-xl` (20), `--radius-full` (9999)

---

## 5. Logo

### Polished SVG

Drop-in replacement for all logo instances. Same `viewBox`, same stroke widths. Centered eyebrow, smoother bezier curves, tighter (but not cramped) eyebrow gap.

```svg
<svg viewBox="60 140 420 280">
  <path d="M98 212C152 174 365 158 420 248"
        fill="none" stroke="#6C63FF" stroke-width="20" stroke-linecap="round"/>
  <path d="M262 374C228 373 176 360 128 321C176 276
        314 200 390 270C462 336 350 379 322 374C248
        361 262 276 322 279C378 282 363 346 322 332"
        fill="none" stroke="#6C63FF" stroke-width="22" stroke-linecap="round"/>
</svg>
```

### Logo Animations — Three Contexts

All animations target `svg path:first-child` (the eyebrow). The eye path stays static.

#### 1. Draw On — Home page, Welcome screen

Plays once on load. The logo draws itself: eyebrow first, then the eye spiral traces in. Uses `stroke-dasharray` / `stroke-dashoffset`.

```css
/* Both paths start hidden */
.logo-draw svg path {
  stroke-dasharray: 800;
  stroke-dashoffset: 800;
}

/* Eyebrow draws first */
.logo-draw.playing svg path:first-child {
  stroke-dasharray: 400;
  stroke-dashoffset: 400;
  animation: drawBrow 0.6s cubic-bezier(0.22, 1, 0.36, 1) forwards;
}

/* Eye follows 0.15s later */
.logo-draw.playing svg path:last-child {
  stroke-dasharray: 800;
  stroke-dashoffset: 800;
  animation: drawEye 1.0s cubic-bezier(0.22, 1, 0.36, 1) 0.15s forwards;
}

@keyframes drawBrow {
  to { stroke-dashoffset: 0; }
}
@keyframes drawEye {
  to { stroke-dashoffset: 0; }
}
```

#### 2. Squint Loop — Thinking / loading state

Continuous smooth loop. Squints down (thinking), holds, opens up (understanding), rests, repeats. 2.4s per cycle. End state matches start for seamless looping.

```css
.logo-think svg path:first-child {
  transform-origin: 260px 212px;
  animation: thinkLoop 2.4s cubic-bezier(0.4, 0, 0.2, 1) infinite;
}

@keyframes thinkLoop {
  0%   { transform: translateY(0) rotate(0deg); }
  18%  { transform: translateY(22px) rotate(3deg); }     /* squint down */
  32%  { transform: translateY(22px) rotate(3deg); }     /* hold */
  52%  { transform: translateY(-10px) rotate(-4deg); }   /* open up */
  68%  { transform: translateY(0) rotate(0.5deg); }      /* settle */
  100% { transform: translateY(0) rotate(0deg); }        /* rest */
}
```

Used alongside the thinking dots:
```html
<div class="thinking">
  <div class="logo-think">
    <svg viewBox="60 140 420 280" width="28" height="19">
      <path d="M98 212C152 174 365 158 420 248" fill="none" stroke="#6C63FF" stroke-width="20" stroke-linecap="round"/>
      <path d="M262 374C228 373 176 360 128 321C176 276 314 200 390 270C462 336 350 379 322 374C248 361 262 276 322 279C378 282 363 346 322 332" fill="none" stroke="#6C63FF" stroke-width="22" stroke-linecap="round"/>
    </svg>
  </div>
  <div class="thinking-dots"><span></span><span></span><span></span></div>
  Thinking
</div>
```

#### 3. Double Blink — "Key added" / success moments

Plays once. Two quick blinks — a surprised, delighted reaction.

```css
.logo-blink svg path:first-child {
  transform-origin: 260px 230px;
}
.logo-blink.playing svg path:first-child {
  animation: doubleBlink 0.9s cubic-bezier(0.22, 1, 0.36, 1);
}

@keyframes doubleBlink {
  0%   { transform: translateY(0); }
  15%  { transform: translateY(26px); }   /* first blink */
  30%  { transform: translateY(0); }
  45%  { transform: translateY(22px); }   /* second blink */
  60%  { transform: translateY(0); }
  100% { transform: translateY(0); }      /* rest */
}
```

Trigger with JS by toggling a class:
```js
// Add 'playing' class, remove after animation completes
el.classList.remove('playing');
void el.offsetWidth; // force reflow
el.classList.add('playing');
```

### App Icon

Current app icon uses `rx="3"` (nearly square corners). Modern iOS uses a superellipse (squircle) with ~23.4% corner radius. The icon background should use `#141414` (near-black) with the polished logo centered. **The OS applies the mask automatically** — just provide a square icon with no rounding, and iOS/macOS will clip it to the squircle shape.

### Full logo SVG file (for app icon)

```svg
<svg width="543" height="543" viewBox="0 0 543 543" fill="none" xmlns="http://www.w3.org/2000/svg">
  <path d="M40 120C40 75.8172 75.8172 40 120 40H423C467.183 40 503 75.8172 503 120V423C503 467.183 467.183 503 423 503H120C75.8172 503 40 467.183 40 423V120Z" fill="#141414"/>
  <path d="M98 212C152 174 365 158 420 248" stroke="#6C63FF" stroke-width="20" stroke-linecap="round" fill="none" transform="translate(32, 50)"/>
  <path d="M262 374C228 373 176 360 128 321C176 276 314 200 390 270C462 336 350 379 322 374C248 361 262 276 322 279C378 282 363 346 322 332" stroke="#6C63FF" stroke-width="22" stroke-linecap="round" fill="none" transform="translate(32, 50)"/>
</svg>
```

Note: The `transform="translate(32, 50)"` centers the logo (viewBox 60-480 range) within the 543×543 icon canvas. Adjust if needed after visual verification.

---

## 6. Motion

### Remove
- `selectionPulse` — infinite glow loop
- `shortcutPulse` — infinite border glow
- `overlayExit` — TV shutdown effect → replace with 150ms opacity fade
- Old `eyebrowWiggle` — replaced by context-specific animations above

### Keep (with refinements)
- `panelSlideIn` (200ms), `msgFadeIn` (250ms), `menuPopIn` (150ms), `tooltipFadeIn` (100ms), `setupSlideUp` (300ms), `welcomeFadeIn` (250ms)

### Thinking indicator
Replace current opacity-pulsing text with: Glimpse eye (squint loop animation) + three animated dots + "Thinking" in Outfit 12px. The eye becomes visually active during loading — it's "working on it."

### Reduced motion
```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

### Default transitions
`0.15s ease` baseline. `ease-out` for entrances. Never bounce or elastic.

---

## 7. Accessibility

### Focus states
All interactive elements: `:focus-visible { outline: 2px solid var(--brand); outline-offset: 2px; }`

### ARIA labels
Add `aria-label` to every icon-only button: Close, Pin/Unpin, Settings, New thread, Send message, Copy code, Copy text, Save, Undo, Reset annotations, Remove screenshot, Open thread menu, Change model.

### Annotation Color Palette — Colorblind-safe

The original 8-color palette had 5 confusable pairs across CVD types (red↔green, orange↔yellow, blue↔purple). Replaced with a 7-color palette (5 chromatic + black + white) that has **zero confusable pairs** across protanopia, deuteranopia, and tritanopia.

| Color | Hex | Replaces |
|---|---|---|
| Red | `#DC3545` | `#e5243a` (similar, slightly adjusted) |
| Amber | `#E08700` | `#e07800` (orange, kept) |
| Cyan | `#06B6D4` | `#1aab42` (green — dropped, replaced by cyan) |
| Blue | `#1E40AF` | `#0062e0` (similar, deeper) |
| Pink | `#EC4899` | `#9333cf` (purple — dropped, replaced by pink) |
| Black | `#000000` | `#000000` (kept) |
| White | `#ffffff` | `#ffffff` (kept) |

**Dropped:** Green (`#1aab42`) — indistinguishable from red for 8% of males. Yellow (`#d4a800`) — confuses with orange. Purple (`#9333cf`) — confuses with blue.

**CVD verification:** All 10 chromatic pairs pass >55 distance in protanopia, deuteranopia, and tritanopia simulations.

See `mockups/annotation-colors.html` for visual comparison.

---

## 8. Pinned State Transition

The pin transition communicates "floating companion on top of everything" through **structural depth changes** — not just color. This matters because the creator is color weak, and many users will be too.

**Duration:** 450ms, `cubic-bezier(0.22, 1, 0.36, 1)`

### What changes (both themes):

| Property | Unpinned | Pinned |
|---|---|---|
| Box shadow | Flat, subtle | Deep, layered, faint brand tint |
| Transform | `scale(1) translateY(0)` | `scale(1.015) translateY(-4px)` |
| Backdrop filter | none | `blur(20px) saturate(1.3)` |
| Border | 1px, dim | 1.5px, brand-tinted |
| Background | Opaque surface | Translucent (enables blur) |
| Pin button | dim | brand-filled |

### Light: Warm Gray → Soft Lavender surface shift
### Dark: opaque `rgba(22,21,30,0.97)` → translucent `rgba(22,21,30,0.88)`

Typography, layout, spacing do NOT change. See `mockups/pinned-v2.html` and `mockups/dark-theme.html` for interactive demos.

---

## 9. Key Layout Decisions

### Chat panel structure

```
┌─────────────────────────────────────┐
│ [👁] [title ▾] ........ [+new] | [📌] │  header (drag handle)
│─────────────────────────────────────│
│              Messages               │
│─────────────────────────────────────│
│ ┌─ input box ─────────────────────┐ │
│ │ [📷 Screenshot attached      ×] │ │  attachment (conditional)
│ │ ["Referenced text..."         ×] │ │  text snippet (conditional)
│ │ [Ask about this screenshot... ➤] │ │  textarea + send
│ └─────────────────────────────────┘ │
│ ⚙  Esc to close         model ▾   │  footer bar
└─────────────────────────────────────┘
```

**Header:** Eye (brand, wiggle on click) → Title + chevron (thread menu) → flex space → [+ new] | [pin]. No close button. Eye is part of drag handle, not a click target for navigation.

**Footer bar:** Settings gear (left) → "Esc to close" with kbd styling (left) → spacer → model switcher in mono (right).

**Attachments live inside the input box**, stacked above the textarea — they're part of the composition, not floating context.

### Selection border
Static `1.5px` border in `--brand` color. Round resize handles (circles, not squares). No corner brackets. No pulsing. No glow.

### Edit toolbar
Same layout as production (settings | 7 tools | undo + reset | copy + save | cancel + AI toggle). Changes: brand color for active states instead of cyan. "Reset" in Outfit 11px. Cancel button dim by default, red only on hover. No glow shadows. See `mockups/edit-toolbar.html`.

### Code blocks
Header bar with language label + dim copy icon (icon-only, no "Copy" text). Icon always visible at very low opacity, brightens on hover, green checkmark on copy.

### Blockquotes
Left border accent + subtle brand tint background. Same dim copy icon as code blocks, positioned top-right.

### Inline code
Click-to-copy, no visible button. Brand-tinted hover state. Green flash on copy.

---

## 10. Welcome Flow

4-step onboarding wizard. Structure is the same as production — changes are styling, copy, and micro-interactions.

### Step 0 — Intro
- Logo plays **draw-on animation** on load (not the old wiggle)
- After draw-on completes (~1.2s), a **0.3s delay**, then the eye does a **single blink** (animation G / doubleBlink but just one blink). The companion has "arrived" and is looking at the user.
- Title: "Glimpse" (not "Welcome to Glimpse")
- Tagline: "Snap it. Ask it. Never lose your flow."
- CTA: solid brand button "Get Started"

### Step 1 — Permissions
- Title: "Quick setup"
- Description: "Glimpse needs two permissions to capture your screen and respond to shortcuts."
- Two permission cards: Screen Recording + Accessibility
- When a permission is granted, the **green checkmark draws its stroke** (stroke-dashoffset animation, 0.3s) instead of appearing instantly
- Disabled button text: "Grant both to continue" (not "Waiting for permissions...")
- Skip link appears **after 5 seconds** (not immediately): "I've granted permissions, continue"
- Green check color: `#238542` (passes 3.7:1 on dark cards, 4.5:1 on light)

### Step 2 — Shortcuts
- Title: "Two ways to start"
- Two shortcut cards: Cmd+Shift+Z (screenshot) and Cmd+Shift+X (text chat)
- Active card (waiting for user to try): **static brand-tinted background** — no pulsing animation
- Done card: green check (same draw-stroke animation as step 1)
- When both shortcuts complete, the CTA button enables with a **subtle scale pop** (`scale(0.97) → scale(1.0)` over 200ms, ease-out)
- Disabled button text: "Try both to continue"
- Skip link appears **after 5 seconds**: "Shortcuts not working? Skip"

### Step 3 — Pin feature + API hint
- Title: "Pin to screen"
- Interactive pin card: clicking it toggles a pinned visual state (card lifts, brand border, shadow). "Try it — click this card."
- Active dot in the footer navigation does a **single scale pulse** (1.0 → 1.3 → 1.0 over 300ms) when step 3 loads
- **API key hint** below the pin card: "To chat with AI, add an API key in **Settings**. You can also use an invite code." — sets expectations so the in-chat setup screen doesn't feel like a surprise
- CTA: "Start Using Glimpse"
- Easter egg text when pinned: "You got it. That's all you need." (not "Now go build something" — inclusive for non-developers)

### Changes from production
- Draw-on logo replaces old wiggle on step 0
- Blink after draw-on (new delight moment)
- Green checks draw their stroke instead of instant appear
- No `shortcutPulse` animation — static brand tint for active state
- CTA scale pop on step 2 completion (new delight moment)
- Dot pulse on step load (new delight moment)
- Skip delays shortened from 10s to 5s
- API key hint added to step 3
- Pin icon color: `#6C63FF` (was `#00E5FF`)
- All text in Outfit (was mixed mono), minimum 11px
- Button text softened ("Grant both to continue" instead of "Waiting for permissions...")

See `mockups/welcome-flow.html` for all 4 steps in both dark and light themes.

---

## 11. Settings Screen

Three sections: API keys, Preferences, Shortcuts. Compact window, auto-closes on blur.

### Layout

```
┌──────────────────────────────────────┐
│  Settings                         ✕  │  header (drag bar)
│──────────────────────────────────────│
│                                      │
│  API keys                            │
│  ┌──────────────────────────────────┐│
│  │ Anthropic   sk-ant-•••  [Update] ││  configured
│  │ Gemini      Not configured [Add] ││  unconfigured
│  │ OpenAI      sk-•••  [Update][Del]││  configured
│  └──────────────────────────────────┘│
│                                      │
│  Preferences                         │
│  ┌──────────────────────────────────┐│
│  │ Appearance    [Light][Dark][Sys] ││  NEW — theme toggle
│  │ Launch at login        [toggle]  ││
│  │ Save screenshots to  [Ask][Dir]  ││
│  └──────────────────────────────────┘│
│                                      │
│  Shortcuts                           │
│  ┌──────────────────────────────────┐│
│  │ Screenshot    Cmd Shift Z        ││
│  │ Text chat     Cmd Shift X        ││
│  └──────────────────────────────────┘│
│  Customizable shortcuts coming soon  │
│                                      │
│  Glimpse v1.0           Esc to close │  footer
└──────────────────────────────────────┘
```

### API Key States

Each provider row has these states:

**Unconfigured:** Label + "Not configured" in dim text + [Add] button.

**Configured:** Label + masked key in mono (`sk-ant-•••7f2a`) + [Update] [Delete] buttons.

**Editing (inline):** Label + password input field (mono, placeholder shows key format) + [Save] [Cancel]. Row border subtly highlights.

**Error:** Same as editing but input has red border + error message: "Invalid key — get a new one" (link to provider console). Same condensed pattern as the in-chat API key setup. Error text uses `--error` color.

**Just saved:** Label + "✓ Saved" in `--success` color (reverts to masked key after ~1.5s). Row border briefly tints green.

**Delete confirmation (inline):** Row shows "Delete this key?" with [Delete] (red) and [Keep] buttons. No modal — confirmation is right in the row.

**Invite code:** Entire API keys section collapses to one row: "Using invite code" badge + [Use my own keys] button.

### New: Theme Toggle

Added to Preferences. Three-way button group: [Light] [Dark] [System]. Same button-group pattern as save location toggle. Persists to localStorage. "System" follows `prefers-color-scheme`.

### New: Version Info

Footer shows "Glimpse v1.0" (left) and "Esc to close" with kbd styling (right).

### New: Shortcuts Note

Below the shortcut rows: "Customizable shortcuts coming soon" in `--text-dim`. This sets expectations that shortcuts are currently fixed but will be configurable.

### Cross-feature Consistency

- Card padding: `12px 14px` (matches welcome flow cards)
- Card border-radius: `12px` / `--radius-md` (matches welcome flow)
- Section titles: Outfit 11px medium (matches welcome flow section labels)
- Footer Esc hint: same pattern as chat panel footer
- Minimum text size: 11px everywhere (kbd elements use 10px mono — acceptable as they're key references)

### Dark-specific: Error Color

`#CF222E` (the light-theme error red) fails contrast on dark surfaces. Dark theme uses `#EF4444` (lighter red, 4.6:1 on dark cards). This is documented in the Semantic Colors section.

See `mockups/settings.html` for all states in both themes.

---

## 12. API Key Setup (In-chat)

Appears inside the ChatPanel when no API key is configured. Replaces the message area.

### Layout

```
┌─────────────────────────────────────┐
│ [👁] [New Thread ▾]  ........ [📌]  │  chat header (normal)
│─────────────────────────────────────│
│                                     │
│  Connect to AI                      │  title (16px, semibold)
│  Keys stay on your device.          │  desc (13px, secondary)
│                                     │
│  ┌─ Anthropic (Claude)          → ┐│
│  ┌─ OpenAI                      → ┐│  provider cards
│  ┌─ Google Gemini               → ┐│
│                                     │
│       Use invite code · Skip for now│  footer links
└─────────────────────────────────────┘
```

### States

**Provider selection:** Title + "Keys stay on your device." description + three provider cards + footer links. No Glimpse eye in the setup area — the header eye provides brand presence.

**Key input (drill-in):** Back button with provider name + password input (mono) + "Get key from [Provider]" link + "Connect" CTA (disabled until input). Title stays, description drops.

**Verifying:** Input disabled, CTA shows "Verifying...". **Header eye does squint-loop animation** — the companion is checking your key.

**Error:** One-line error with inline action: "Invalid key — [get a new one]" where the link opens the provider's console. Same error color as the system (`#EF4444` dark, `#CF222E` light). No separate "Get key" line — condense to one.

**Key saved:** Provider card shows green check, becomes disabled. Other providers remain available. "Continue" button appears (secondary style, not solid CTA). Description changes to "Add more keys or start chatting." **Header eye does double-blink.**

**Success transition:** After clicking Continue (or after saving when coming from the chat), shows centered: Glimpse eye (double-blink) + "Key added. Happy chatting!" — displays for ~2 seconds, then fades into the chat interface.

**Invite code mode:** Toggled via footer link. Same layout — title + text input (display font, not mono) + "Connect" CTA. No separate description line — input placeholder is enough.

### Copy

| Element | Text |
|---|---|
| Title | "Connect to AI" |
| Description (provider list) | "Keys stay on your device." |
| Description (key saved) | "Add more keys or start chatting." |
| CTA (key input) | "Connect" |
| CTA (verifying) | "Verifying..." |
| Error | "Invalid key — get a new one" (link to provider console) |
| Footer links | "Use invite code · Skip for now" / "Use API keys · Skip for now" |
| Success message | "Key added. Happy chatting!" |
| Back button | "← [Provider name]" |
| Key hint | "Get key from [Provider name]" |

### Delight

- **Verifying:** header eye squint-loop (companion is "checking")
- **Key saved:** header eye double-blink (companion is "delighted")
- **Success transition:** eye double-blink + "Key added. Happy chatting!" centered, fades after ~2s

### Cross-feature consistency

- Provider cards: same padding (12px 14px) and radius (12px) as welcome flow and settings
- CTA button: same solid brand style as welcome flow
- Error color: uses theme-specific `--error` (`#EF4444` dark, `#CF222E` light)
- Footer links: same style and opacity as used elsewhere (0.60 alpha)
- Description text: same `--text-secondary` treatment as welcome flow descriptions

See `mockups/api-key-setup.html` for all states in both themes.

---

## 13. Home Dashboard

**Deferred.** The current home dashboard may be removed or replaced with a menu bar popover in a future update. The onboarding flow and chat panel handle all the functions the home screen currently provides (shortcuts education, recent threads, settings access). Do not redesign — keep as-is for now and revisit after the core redesign ships.

---

## 14. Migration Summary

1. **Delete** all cyan/magenta tokens and references
2. **Add** the new token system (brand, light theme, dark theme, pinned state variants)
3. **Switch** all non-code UI text from `--font-mono` to `--font-display`, minimum 11px, drop uppercase
4. **Replace** selection border (remove brackets, glow, pulse; add round handles, static brand border)
5. **Replace** all `#00e5ff` / cyan references with `--brand` / `--brand-text`
6. **Remove** infinite animations (`selectionPulse`, `shortcutPulse`), replace `overlayExit` with fade
7. **Add** `prefers-reduced-motion` media query
8. **Add** `:focus-visible` states and `aria-label` to all icon buttons
9. **Replace** "Copy" text buttons with dim icon-only copy buttons
10. **Implement** pinned state transition (shadow lift + scale + blur + border + surface shift)
11. **Add** theme toggle support (light default, dark optional, user preference persisted)
12. **Replace** logo SVG paths with polished version (all instances in JSX components)
13. **Replace** old `eyebrowWiggle` with three context-specific animations (draw-on, squint loop, double blink)
14. **Update** app icon: use polished logo on `#141414` background, provide as square (OS applies squircle mask)
15. **Replace** annotation color palette: 8 colors → 7 colorblind-safe colors (drop green, yellow, purple; add cyan, pink)

---

## Appendix: Visual References

HTML mockups in `mockups/` — open in browser for visual reference.

| File | Shows |
|---|---|
| `mockups/logo-compare.html` | Logo original vs polished, both themes, all sizes |
| `mockups/logo-final-animations.html` | Three logo animations: draw-on, squint loop, double blink |
| `mockups/polish.html` | Welcome + Chat + Pinned, both themes (theme toggle) |
| `mockups/light-surfaces.html` | Warm Gray vs Soft Lavender with a11y scores |
| `mockups/dark-theme.html` | Three dark surface options + dark pinned transition |
| `mockups/pinned-v2.html` | Light pinned transition (interactive) |
| `mockups/chat-header.html` | Header layout options |
| `mockups/copy-patterns.html` | Copy button pattern comparison |
| `mockups/edit-toolbar.html` | Edit toolbar before/after |
| `mockups/toolbar.html` | Selection border before/after |
| `mockups/welcome-flow.html` | All 4 welcome steps, both themes, with delight moments |
| `mockups/settings.html` | Settings — all API key states, themes, theme toggle, delete confirm |
| `mockups/api-key-setup.html` | API key setup — all states, both themes |
| `mockups/annotation-colors.html` | Colorblind-safe annotation palette comparison |
| `mockups/welcome-compare.html` | Welcome + Home, dark vs light (earlier exploration) |
