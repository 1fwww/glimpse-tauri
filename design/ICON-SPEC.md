# Glimpse Icon Spec

## App Icon (Dock, Finder, Applications folder, DMG)

### Source SVG

```svg
<svg viewBox="0 4 525 525" width="1024" height="1024" xmlns="http://www.w3.org/2000/svg">
  <rect x="-20" y="-20" width="600" height="600" fill="#141414"/>
  <path d="M98 212C152 174 365 158 420 248" fill="none" stroke="#6C63FF" stroke-width="20" stroke-linecap="round"/>
  <path d="M262 374C228 373 176 360 128 321C176 276 314 200 390 270C462 336 350 379 322 374C248 361 262 276 322 279C378 282 363 346 322 332" fill="none" stroke="#6C63FF" stroke-width="22" stroke-linecap="round"/>
</svg>
```

### Key details
- **Background:** `#141414` (near-black, NOT pure `#000000`)
- **Eye color:** `#6C63FF` (brand indigo)
- **ViewBox:** `0 4 525 525` — optically centered (geometric center is x=18 but visual weight of the spiral pulls right, so shifted left to compensate)
- **Fill:** ~73% of the square — balanced breathing room, won't be clipped by macOS squircle corners
- **The OS applies the squircle mask automatically** — export as a square PNG with no rounding

### Export sizes needed
From this source SVG, generate PNGs at:
- `icon.png` — 1024×1024 (source)
- `128x128.png` — 128×128
- `128x128@2x.png` — 256×256
- `64x64.png` — 64×64  
- `32x32.png` — 32×32
- `icon.icns` — macOS icon bundle (contains all sizes)

Place in `src-tauri/icons/`.

### Reference
See `design/mockups/app-icons.html` — Option A.

---

## Tray Icon (menu bar)

Right-side crop of the full logo with **rounded stroke ends** at the three left-edge cut points. The paths are split from the original curves via De Casteljau, preserving exact curve direction — only the start/end points are moved ~13px inside the viewBox so `stroke-linecap="round"` creates natural round caps instead of flat cuts.

### Requirements
- Must be a **template image** — provide only black shapes on transparent background
- macOS automatically renders it white on dark menu bar, black on light menu bar
- Size: 22×22 pt (44×44 px @2x)
- No color — macOS template rendering ignores color
- Background must be **transparent** (no rect fill). Export as PNG with alpha channel.

### Source SVG for tray (UPDATED — rounded ends)

```svg
<svg viewBox="220 168 230 232" width="44" height="44" xmlns="http://www.w3.org/2000/svg">
  <!-- Brow: trimmed to start inside viewBox -->
  <path d="M243 182C315 182 390 199 420 248" fill="none" stroke="black" stroke-width="26" stroke-linecap="round"/>
  <!-- Body lower stub: short arc going left, round cap at end -->
  <path d="M262 374C253 374 244 372 236 370" fill="none" stroke="black" stroke-width="28" stroke-linecap="round"/>
  <!-- Body main: re-enters from left + spiral -->
  <path d="M245 253C296 237 351 234 390 270C462 336 350 379 322 374C248 361 262 276 322 279C378 282 363 346 322 332" fill="none" stroke="black" stroke-width="28" stroke-linecap="round"/>
</svg>
```

### Key changes from previous version
- **viewBox:** `220 168 230 232` (was `230 180 220 220`) — expanded left and top to accommodate round caps
- **3 sub-paths** instead of 2 continuous paths — split at the two points where the body curve exits/re-enters the viewBox
- **Stroke-linecap="round"** now visible at all endpoints — no more flat cuts
- Stroke widths unchanged (26 for brow, 28 for body)

### Also update: Welcome Step 4 inline tray icon
The small inline icon in "Find [icon] in your menu bar" hint text should use the same updated paths and viewBox:
```jsx
<svg viewBox="220 168 230 232" width="14" height="10" style={{display:'inline',verticalAlign:'middle'}}>
  <path d="M243 182C315 182 390 199 420 248" fill="none" stroke="currentColor" strokeWidth="26" strokeLinecap="round"/>
  <path d="M262 374C253 374 244 372 236 370" fill="none" stroke="currentColor" strokeWidth="28" strokeLinecap="round"/>
  <path d="M245 253C296 237 351 234 390 270C462 336 350 379 322 374C248 361 262 276 322 279C378 282 363 346 322 332" fill="none" stroke="currentColor" strokeWidth="28" strokeLinecap="round"/>
</svg>
```

### Reference
`design/mockups/tray-icon-rounded-v2.html` — side-by-side comparison with previous version at both large and menu bar sizes.

---

## In-app Logo (header, welcome, home)

No changes — uses the existing polished SVG paths with `viewBox="60 140 420 280"`. Already working correctly at 24px (header), 48px (home), 64px (welcome).
