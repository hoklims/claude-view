# claude-view-tray Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri v2 Windows tray app that auto-starts Docker claude-view and provides a WebView2 window to the dashboard.

**Architecture:** Tauri v2 app with Rust backend managing Docker via `std::process::Command`. System tray icon shows container status. WebView2 window loads `http://localhost:47892`. Auto-start via Windows registry.

**Tech Stack:** Tauri v2, Rust, WebView2 (built into Windows 11), Docker Compose CLI

---

## File Structure

```
claude-view-tray/
├── src-tauri/
│   ├── Cargo.toml              # Tauri deps + features
│   ├── build.rs                # Tauri build script
│   ├── tauri.conf.json         # Window config, bundle settings
│   ├── capabilities/
│   │   └── default.json        # Permissions for tray, shell, etc.
│   ├── icons/
│   │   ├── icon.ico            # App icon
│   │   ├── tray-running.png    # Green tray icon (32x32)
│   │   └── tray-stopped.png    # Red tray icon (32x32)
│   └── src/
│       ├── lib.rs              # Tauri app setup, tray, window events
│       ├── docker.rs           # Docker compose start/stop/health
│       └── autostart.rs        # Windows registry auto-start
├── src/
│   └── index.html              # Minimal loading page (shown while Docker starts)
├── package.json                # npm scripts for tauri dev/build
└── README.md
```

---

## Chunk 1: Project Scaffold + Docker Manager

### Task 1: Scaffold Tauri v2 project

**Files:**
- Create: `claude-view-tray/package.json`
- Create: `claude-view-tray/src/index.html`
- Create: `claude-view-tray/src-tauri/Cargo.toml`
- Create: `claude-view-tray/src-tauri/build.rs`
- Create: `claude-view-tray/src-tauri/tauri.conf.json`
- Create: `claude-view-tray/src-tauri/capabilities/default.json`

- [ ] **Step 1: Create project directory**

```bash
mkdir -p claude-view-tray/src claude-view-tray/src-tauri/src claude-view-tray/src-tauri/icons claude-view-tray/src-tauri/capabilities
```

- [ ] **Step 2: Create package.json**

```json
{
  "name": "claude-view-tray",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "tauri": "tauri",
    "dev": "tauri dev",
    "build": "tauri build"
  },
  "devDependencies": {
    "@anthropic-ai/tauri-cli": "^2"
  }
}
```

Write to `claude-view-tray/package.json`.

- [ ] **Step 3: Create minimal index.html**

This is the loading page shown while Docker starts up. Once healthy, we redirect to localhost:47892.

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>claude-view</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      background: #0a0a0a;
      color: #e5e5e5;
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100vh;
    }
    .container { text-align: center; }
    .spinner {
      width: 40px; height: 40px;
      border: 3px solid #333;
      border-top-color: #22c55e;
      border-radius: 50%;
      animation: spin 0.8s linear infinite;
      margin: 0 auto 16px;
    }
    @keyframes spin { to { transform: rotate(360deg); } }
    h1 { font-size: 20px; margin-bottom: 8px; }
    p { font-size: 14px; color: #888; }
  </style>
</head>
<body>
  <div class="container">
    <div class="spinner"></div>
    <h1>claude-view</h1>
    <p>Starting Docker container...</p>
  </div>
</body>
</html>
```

Write to `claude-view-tray/src/index.html`.

- [ ] **Step 4: Create Cargo.toml**

```toml
[package]
name = "claude-view-tray"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-png", "image-ico"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }
tokio = { version = "1", features = ["time"] }
winreg = "0.55"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

Write to `claude-view-tray/src-tauri/Cargo.toml`.

- [ ] **Step 5: Create build.rs**

```rust
fn main() {
    tauri_build::build()
}
```

Write to `claude-view-tray/src-tauri/build.rs`.

- [ ] **Step 6: Create tauri.conf.json**

```json
{
  "$schema": "https://raw.githubusercontent.com/nicedouble/tauri-settings-schema/main/src/v2/tauri.conf.json",
  "productName": "claude-view-tray",
  "version": "0.1.0",
  "identifier": "com.claudeview.tray",
  "build": {
    "frontendDist": "../src"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "claude-view",
        "url": "index.html",
        "width": 1280,
        "height": 800,
        "visible": false,
        "decorations": true,
        "resizable": true
      }
    ],
    "security": {
      "dangerousRemoteUrlAccess": [
        { "url": "http://localhost:47892/**", "enableTauri": false }
      ]
    }
  },
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "icon": [
      "icons/icon.ico"
    ],
    "windows": {
      "allowDowngrades": true
    }
  }
}
```

Write to `claude-view-tray/src-tauri/tauri.conf.json`.

- [ ] **Step 7: Create capabilities/default.json**

```json
{
  "identifier": "default",
  "description": "Default permissions",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "shell:allow-execute"
  ]
}
```

Write to `claude-view-tray/src-tauri/capabilities/default.json`.

- [ ] **Step 8: Commit scaffold**

```bash
git add claude-view-tray/
git commit -m "feat(tray): scaffold Tauri v2 project"
```

---

### Task 2: Docker manager module

**Files:**
- Create: `claude-view-tray/src-tauri/src/docker.rs`

- [ ] **Step 1: Write docker.rs**

```rust
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum DockerStatus {
    Starting,
    Running,
    Stopped,
    Error(String),
}

/// Resolve the path to docker-compose.yml.
/// Checks: CLAUDE_VIEW_COMPOSE_PATH env, then next to the executable.
fn compose_file() -> PathBuf {
    if let Ok(p) = std::env::var("CLAUDE_VIEW_COMPOSE_PATH") {
        return PathBuf::from(p);
    }
    let mut dir = std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    dir.push("docker-compose.yml");
    if dir.exists() {
        return dir;
    }
    // Fallback: claude-view repo location
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    PathBuf::from(home)
        .join(".claude-view")
        .join("docker-compose.yml")
}

pub fn start() -> Result<(), String> {
    let file = compose_file();
    let output = Command::new("docker")
        .args(["compose", "-f", &file.to_string_lossy(), "up", "-d"])
        .output()
        .map_err(|e| format!("Failed to run docker compose: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("docker compose up failed: {stderr}"))
    }
}

pub fn stop() -> Result<(), String> {
    let file = compose_file();
    let output = Command::new("docker")
        .args(["compose", "-f", &file.to_string_lossy(), "down"])
        .output()
        .map_err(|e| format!("Failed to run docker compose: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("docker compose down failed: {stderr}"))
    }
}

/// Check if claude-view is healthy by hitting /api/health.
pub async fn health_check() -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();
    match client.get("http://localhost:47892/api/health").send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}
```

Write to `claude-view-tray/src-tauri/src/docker.rs`.

- [ ] **Step 2: Commit**

```bash
git add claude-view-tray/src-tauri/src/docker.rs
git commit -m "feat(tray): add Docker compose manager module"
```

---

### Task 3: Auto-start module

**Files:**
- Create: `claude-view-tray/src-tauri/src/autostart.rs`

- [ ] **Step 1: Write autostart.rs**

```rust
use winreg::enums::*;
use winreg::RegKey;

const REG_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "claude-view-tray";

/// Register this executable to auto-start at Windows login.
pub fn enable() -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Cannot get exe path: {e}"))?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(REG_KEY)
        .map_err(|e| format!("Cannot open registry: {e}"))?;
    key.set_value(APP_NAME, &exe.to_string_lossy().as_ref())
        .map_err(|e| format!("Cannot set registry value: {e}"))?;
    Ok(())
}

/// Remove auto-start entry.
pub fn disable() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(REG_KEY, KEY_WRITE)
        .map_err(|e| format!("Cannot open registry: {e}"))?;
    // Ignore error if value doesn't exist
    let _ = key.delete_value(APP_NAME);
    Ok(())
}

/// Check if auto-start is enabled.
pub fn is_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = match hkcu.open_subkey(REG_KEY) {
        Ok(k) => k,
        Err(_) => return false,
    };
    key.get_value::<String, _>(APP_NAME).is_ok()
}
```

Write to `claude-view-tray/src-tauri/src/autostart.rs`.

- [ ] **Step 2: Commit**

```bash
git add claude-view-tray/src-tauri/src/autostart.rs
git commit -m "feat(tray): add Windows auto-start registry module"
```

---

## Chunk 2: Main App + Tray Integration

### Task 4: Tray icons

**Files:**
- Create: `claude-view-tray/src-tauri/icons/tray-running.png`
- Create: `claude-view-tray/src-tauri/icons/tray-stopped.png`
- Create: `claude-view-tray/src-tauri/icons/icon.ico`

- [ ] **Step 1: Generate tray icons**

Create two 32x32 PNG icons:
- `tray-running.png`: green circle (●) on transparent background
- `tray-stopped.png`: red circle (●) on transparent background
- `icon.ico`: app icon for the window/installer (use a simple "CV" monogram or the claude-view logo)

Use any image tool or generate programmatically. These can be placeholder icons refined later.

- [ ] **Step 2: Commit icons**

```bash
git add claude-view-tray/src-tauri/icons/
git commit -m "feat(tray): add tray status icons"
```

---

### Task 5: Main app with tray, Docker lifecycle, and WebView

**Files:**
- Create: `claude-view-tray/src-tauri/src/lib.rs`

This is the core of the app. It wires together:
- System tray with status icon and context menu
- Docker start on launch, stop on quit
- WebView window that opens/closes independently
- Health check polling to update tray status and redirect WebView

- [ ] **Step 1: Write lib.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod autostart;
mod docker;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, RunEvent, WindowEvent,
};

static DOCKER_HEALTHY: AtomicBool = AtomicBool::new(false);

const ICON_RUNNING: &[u8] = include_bytes!("../icons/tray-running.png");
const ICON_STOPPED: &[u8] = include_bytes!("../icons/tray-stopped.png");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // --- Auto-start registration (first launch) ---
            if !autostart::is_enabled() {
                let _ = autostart::enable();
            }

            // --- Start Docker ---
            if let Err(e) = docker::start() {
                eprintln!("Docker start failed: {e}");
            }

            // --- Build tray menu ---
            let open_item = MenuItemBuilder::with_id("open", "Open claude-view").build(app)?;
            let restart_item = MenuItemBuilder::with_id("restart", "Restart Docker").build(app)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[&open_item, &restart_item, &separator, &quit_item])
                .build()?;

            // --- Create tray icon ---
            let _tray = TrayIconBuilder::new()
                .icon(Image::from_bytes(ICON_STOPPED)?)
                .tooltip("claude-view — Starting...")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "restart" => {
                        let _ = docker::stop();
                        DOCKER_HEALTHY.store(false, Ordering::SeqCst);
                        if let Err(e) = docker::start() {
                            eprintln!("Docker restart failed: {e}");
                        }
                    }
                    "quit" => {
                        let _ = docker::stop();
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            if let Some(w) = tray.app_handle().get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // --- Health check polling (background task) ---
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let healthy = docker::health_check().await;
                    let was_healthy = DOCKER_HEALTHY.swap(healthy, Ordering::SeqCst);

                    // Update tray icon and tooltip on status change
                    if healthy != was_healthy {
                        if let Some(tray) = app_handle.tray_by_id("main") {
                            if healthy {
                                let _ = tray.set_icon(Some(Image::from_bytes(ICON_RUNNING).unwrap()));
                                let _ = tray.set_tooltip(Some("claude-view — Running"));
                            } else {
                                let _ = tray.set_icon(Some(Image::from_bytes(ICON_STOPPED).unwrap()));
                                let _ = tray.set_tooltip(Some("claude-view — Stopped"));
                            }
                        }

                        // Redirect WebView to dashboard when Docker becomes healthy
                        if healthy {
                            if let Some(w) = app_handle.get_webview_window("main") {
                                let _ = w.navigate("http://localhost:47892".parse().unwrap());
                            }
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide window on close instead of quitting
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Write to `claude-view-tray/src-tauri/src/lib.rs`.

- [ ] **Step 2: Create main.rs entry point**

```rust
fn main() {
    claude_view_tray::run();
}
```

Write to `claude-view-tray/src-tauri/src/main.rs`.

- [ ] **Step 3: Commit**

```bash
git add claude-view-tray/src-tauri/src/
git commit -m "feat(tray): main app with tray, Docker lifecycle, and WebView"
```

---

## Chunk 3: Build, Test, Polish

### Task 6: Copy docker-compose.yml for distribution

The tray app needs access to docker-compose.yml to manage the container. We bundle it next to the binary.

**Files:**
- Modify: `claude-view-tray/src-tauri/tauri.conf.json`

- [ ] **Step 1: Add resource to tauri.conf.json**

Add to the `"app"` section in `tauri.conf.json`:

```json
"resources": [
  "../../docker-compose.yml"
]
```

This bundles the docker-compose.yml from the repo root into the app package.

- [ ] **Step 2: Update docker.rs compose_file() fallback**

Update the `compose_file()` function to also check the Tauri resource directory:

Add as first fallback after env var check:
```rust
// Check bundled resource next to executable
let exe_dir = std::env::current_exe()
    .ok()
    .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    .unwrap_or_default();
let bundled = exe_dir.join("docker-compose.yml");
if bundled.exists() {
    return bundled;
}
```

- [ ] **Step 3: Commit**

```bash
git add claude-view-tray/
git commit -m "feat(tray): bundle docker-compose.yml with app"
```

---

### Task 7: Build and test on Windows

- [ ] **Step 1: Install Tauri CLI**

```bash
cd claude-view-tray
npm install
```

Note: If `@anthropic-ai/tauri-cli` doesn't exist, use `@tauri-apps/cli` instead in package.json.

- [ ] **Step 2: Build the app**

```bash
cd claude-view-tray
npx tauri build
```

Expected: produces `src-tauri/target/release/claude-view-tray.exe` and an NSIS installer in `src-tauri/target/release/bundle/nsis/`.

- [ ] **Step 3: Test manually**

1. Run `src-tauri/target/release/claude-view-tray.exe`
2. Verify: tray icon appears (red — Docker starting)
3. Wait ~10s: tray icon turns green, tooltip says "Running"
4. Left-click tray: WebView opens to claude-view dashboard
5. Close window (X): window hides, tray stays
6. Left-click tray again: window reopens
7. Right-click tray → "Restart Docker": icon goes red, then green
8. Right-click tray → "Quit": Docker stops, app exits

- [ ] **Step 4: Verify auto-start**

1. Open regedit → `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
2. Verify `claude-view-tray` entry exists with correct exe path
3. Restart Windows (or log out/in)
4. Verify app starts automatically with tray icon

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "feat(tray): working Windows tray app with Docker integration"
```

---

## Summary

| Task | Description | Estimated Time |
|------|-------------|---------------|
| 1 | Scaffold Tauri v2 project | 5 min |
| 2 | Docker manager module | 5 min |
| 3 | Auto-start module | 3 min |
| 4 | Tray icons | 3 min |
| 5 | Main app (tray + Docker + WebView) | 10 min |
| 6 | Bundle docker-compose.yml | 3 min |
| 7 | Build and test | 10 min |
