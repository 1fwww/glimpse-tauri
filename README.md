# Glimpse

**Snap it. Ask it. Never lose your flow.**

You're in the middle of something. You see a chart, an error, a message you want to rewrite. The old way: copy it, switch to your AI, paste, explain context, wait, switch back — and you've already lost your train of thought.

Glimpse lives on your screen. Hit a shortcut, capture what you're looking at or select any text, and ask AI right there. Chat stays open, pins to your screen, and goes away when you're done.

Your work keeps flowing — just capture, ask, and move on.

## What it does

- **Screenshot + Ask** — Capture any region, annotate it, and chat with AI about what you see
- **Quick Chat** — Select text anywhere, press a shortcut, and ask about it instantly
- **Pin to Screen** — Keep the chat floating on top while you work
- **Annotate** — Draw, highlight, arrow, text, blur — all built in

## Download

[Download the latest release](https://github.com/1fwww/glimpse-tauri/releases/latest) for macOS.

> First launch: right-click the app and select Open to bypass Gatekeeper.

## Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+Z` | Screenshot — capture, annotate, ask |
| `Cmd+Shift+X` | Quick chat — grabs selected text |
| `Esc` | Close / dismiss |

## Setup

Glimpse works as a standalone screenshot tool out of the box. To enable AI chat, add an API key when prompted — or use an invite code if you have one.

Supported providers: [Anthropic Claude](https://console.anthropic.com/), [Google Gemini](https://aistudio.google.com/), and [OpenAI](https://platform.openai.com/api-keys).

## Development

```bash
git clone https://github.com/1fwww/glimpse-tauri.git
cd glimpse-tauri
npm install
npm run tauri:dev
```

### Build

```bash
npm run tauri:build
```

The DMG will be in `src-tauri/target/release/bundle/dmg/`.

## Built with

Tauri v2, React, Vite, Rust, Anthropic Claude API, Google Gemini API, OpenAI API, Canvas-based annotation.

## License

MIT
