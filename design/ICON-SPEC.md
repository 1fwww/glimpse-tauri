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

**For now: use the full logo as a template image.** We'll design a simplified tray-specific variant later.

### Requirements
- Must be a **template image** — provide only black shapes on transparent background
- macOS automatically renders it white on dark menu bar, black on light menu bar
- Size: 22×22 pt (44×44 px @2x)
- No color — the `#6C63FF` will be ignored by macOS template rendering

### Source SVG for tray (current — full logo)

```svg
<svg viewBox="0 4 525 525" width="44" height="44" xmlns="http://www.w3.org/2000/svg">
  <path d="M98 212C152 174 365 158 420 248" fill="none" stroke="black" stroke-width="28" stroke-linecap="round"/>
  <path d="M262 374C228 373 176 360 128 321C176 276 314 200 390 270C462 336 350 379 322 374C248 361 262 276 322 279C378 282 363 346 322 332" fill="none" stroke="black" stroke-width="30" stroke-linecap="round"/>
</svg>
```

Note: strokes are thicker (28/30 vs 20/22) to be readable at tray size. Background must be **transparent** (no rect fill). Export as PNG with alpha channel.

### Future TODO
Design a simplified tray icon variant (eyebrow + outer eye arc only, no spiral) for better readability at 22px. Not blocking for current release.

---

## In-app Logo (header, welcome, home)

No changes — uses the existing polished SVG paths with `viewBox="60 140 420 280"`. Already working correctly at 24px (header), 48px (home), 64px (welcome).
