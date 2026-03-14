# claude-view-tray — Design Spec

**Date:** 2026-03-14
**Status:** Approved
**Author:** hoklims + Claude Code

## Overview

A Tauri v2 desktop app for Windows that runs claude-view via Docker in the background and provides a system tray icon with an embedded WebView2 window. The app auto-starts at Windows boot and requires zero user interaction to keep claude-view running.

## Architecture

```
┌─────────────────────────────────┐
│  Windows Boot                   │
│  └─ Startup: claude-view-tray   │
│     ├─ Spawn: docker compose up │
│     ├─ System Tray Icon         │
│     └─ WebView2 (on demand)     │
│        └─ localhost:47892       │
└─────────────────────────────────┘
```

Three components:

1. **Tray icon** — always visible when app is running, context menu on right-click
2. **Docker manager** — starts/stops the claude-view container via `docker compose`
3. **WebView window** — opens/closes on demand, loads `http://localhost:47892`

## Behavior

| Event | Action |
|-------|--------|
| Windows boot | App launches silently, starts Docker, tray icon appears |
| Left-click tray | Opens/reopens WebView window |
| Right-click tray | Context menu: "Open" / "Restart Docker" / "Quit" |
| Window close (X) | Closes window only, Docker keeps running, tray stays |
| "Quit" from menu | `docker compose down` → exit app → tray disappears |
| Docker crash | Windows notification "claude-view stopped" + retry in tray menu |

## Tech Stack

- **Tauri v2** — desktop framework (Rust + WebView2)
- **Rust backend** — manages Docker via `std::process::Command`
- **Frontend** — minimal, just WebView loading `http://localhost:47892`
- **Auto-start** — Windows registry `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

## Tray Menu

```
claude-view ● Running
─────────────────
  Open                (left-click shortcut)
  Restart Docker
─────────────────
  Quit
```

Status indicator changes color: green (running) / red (stopped).

## Docker Integration

- **Start:** `docker compose -f <path>/docker-compose.yml up -d`
- **Stop:** `docker compose -f <path>/docker-compose.yml down`
- **Health check:** Poll `http://localhost:47892/api/health` every 5s until ready
- **Path:** docker-compose.yml path resolved relative to the tray app binary or configurable via env var `CLAUDE_VIEW_COMPOSE_PATH`

## Auto-Start

Registry key: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
Value name: `claude-view-tray`
Value data: `"<install_path>\claude-view-tray.exe"`

Set on first launch, removable via tray menu if needed later.

## Out of Scope (YAGNI)

- No settings UI — configuration lives in docker-compose.yml
- No log viewer — Docker Desktop handles logs
- No auto-update mechanism — iterate later if needed
- No multi-instance support — single claude-view container
- No macOS/Linux builds — Windows only

## Project Structure

```
claude-view-tray/
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs          # Tauri app entry, tray setup
│   │   ├── docker.rs        # Docker compose start/stop/health
│   │   ├── autostart.rs     # Windows registry auto-start
│   │   └── tray.rs          # Tray icon, menu, status updates
│   ├── icons/               # Tray icons (running/stopped)
│   └── tauri.conf.json      # Tauri configuration
├── src/
│   └── index.html           # Minimal HTML (just loads WebView)
├── package.json
└── README.md
```

## Success Criteria

1. App starts at Windows boot without user interaction
2. Docker container starts automatically
3. Tray icon visible with green/red status
4. Left-click opens WebView to claude-view dashboard
5. Close (X) only hides window, Docker keeps running
6. "Quit" stops Docker and exits cleanly
7. Binary size < 10MB
