mod autostart;
mod docker;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};

static DOCKER_HEALTHY: AtomicBool = AtomicBool::new(false);

const TRAY_ID: &str = "cv-tray";

/// Create a 32x32 RGBA tray icon: rounded square with pulse/heartbeat line.
/// Anti-aliased edges, subtle gradient, white activity indicator.
fn create_tray_icon(r: u8, g: u8, b: u8, active: bool) -> Image<'static> {
    let size: u32 = 32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let corner_radius = 7.0_f32;

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let fx = x as f32;
            let fy = y as f32;

            // Signed distance to rounded rectangle (1px inset for anti-alias room)
            let half = (size as f32 / 2.0) - 1.0;
            let dx = (fx - half - 1.0).abs() - (half - corner_radius);
            let dy = (fy - half - 1.0).abs() - (half - corner_radius);
            let dist = if dx > 0.0 && dy > 0.0 {
                (dx * dx + dy * dy).sqrt() - corner_radius
            } else {
                dx.max(dy) - corner_radius
            };

            if dist < 1.0 {
                // Anti-aliased alpha
                let alpha = if dist < 0.0 { 1.0 } else { 1.0 - dist };

                // Subtle vertical gradient (lighter top, darker bottom)
                let gradient = 1.0 - (fy / size as f32) * 0.25;
                let gr = (r as f32 * gradient).min(255.0) as u8;
                let gg = (g as f32 * gradient).min(255.0) as u8;
                let gb = (b as f32 * gradient).min(255.0) as u8;

                rgba[idx] = gr;
                rgba[idx + 1] = gg;
                rgba[idx + 2] = gb;
                rgba[idx + 3] = (alpha * 255.0) as u8;
            }
        }
    }

    // Draw white pulse/heartbeat line across the middle
    if active {
        // Pulse shape: flat — spike up — spike down — flat
        let pulse_y: Vec<(f32, f32)> = vec![
            // (x_fraction, y_offset from center)
            (0.15, 0.0),
            (0.30, 0.0),
            (0.38, -5.0),
            (0.46, 5.0),
            (0.54, -3.0),
            (0.60, 0.0),
            (0.85, 0.0),
        ];

        let center_y = size as f32 / 2.0;
        // Draw line segments between pulse points
        for i in 0..pulse_y.len() - 1 {
            let (x0f, y0) = pulse_y[i];
            let (x1f, y1) = pulse_y[i + 1];
            let x0 = (x0f * size as f32) as i32;
            let x1 = (x1f * size as f32) as i32;

            for px in x0..=x1 {
                let t = if x1 != x0 {
                    (px - x0) as f32 / (x1 - x0) as f32
                } else {
                    0.0
                };
                let py = center_y + y0 + (y1 - y0) * t;

                // Draw 2px thick line with anti-aliasing
                for dy in -1..=1 {
                    let target_y = (py as i32 + dy).clamp(0, size as i32 - 1) as u32;
                    let target_x = px.clamp(0, size as i32 - 1) as u32;
                    let idx = ((target_y * size + target_x) * 4) as usize;

                    let line_dist = (py - target_y as f32).abs();
                    let line_alpha = (1.0 - line_dist / 1.5).clamp(0.0, 1.0);

                    if line_alpha > 0.0 && rgba[idx + 3] > 0 {
                        // Blend white over existing color
                        let bg_a = rgba[idx + 3] as f32 / 255.0;
                        let blend = line_alpha * 0.95;
                        rgba[idx] = (rgba[idx] as f32 * (1.0 - blend) + 255.0 * blend) as u8;
                        rgba[idx + 1] =
                            (rgba[idx + 1] as f32 * (1.0 - blend) + 255.0 * blend) as u8;
                        rgba[idx + 2] =
                            (rgba[idx + 2] as f32 * (1.0 - blend) + 255.0 * blend) as u8;
                        rgba[idx + 3] = (bg_a * 255.0) as u8;
                    }
                }
            }
        }
    } else {
        // Stopped: draw a small square "stop" icon in center
        let sq = 4_u32;
        let offset = (size - sq) / 2;
        for y in offset..offset + sq {
            for x in offset..offset + sq {
                let idx = ((y * size + x) * 4) as usize;
                if rgba[idx + 3] > 0 {
                    let blend = 0.9_f32;
                    rgba[idx] = (rgba[idx] as f32 * (1.0 - blend) + 255.0 * blend) as u8;
                    rgba[idx + 1] =
                        (rgba[idx + 1] as f32 * (1.0 - blend) + 255.0 * blend) as u8;
                    rgba[idx + 2] =
                        (rgba[idx + 2] as f32 * (1.0 - blend) + 255.0 * blend) as u8;
                }
            }
        }
    }

    Image::new_owned(rgba, size, size)
}

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
            let restart_item =
                MenuItemBuilder::with_id("restart", "Restart Docker").build(app)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[&open_item, &restart_item, &separator, &quit_item])
                .build()?;

            // --- Create tray icon with raw RGBA (no ICO/PNG parsing) ---
            let stopped_icon = create_tray_icon(239, 68, 68, false); // red, stop symbol

            let _tray = TrayIconBuilder::with_id(TRAY_ID)
                .icon(stopped_icon)
                .tooltip("claude-view — Starting...")
                .menu(&menu)
                .show_menu_on_left_click(false)
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
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        if let Some(w) = tray.app_handle().get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
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

                    if healthy != was_healthy {
                        if let Some(tray) = app_handle.tray_by_id(TRAY_ID) {
                            let icon = if healthy {
                                create_tray_icon(34, 197, 94, true) // green, pulse
                            } else {
                                create_tray_icon(239, 68, 68, false) // red, stop
                            };
                            let _ = tray.set_icon(Some(icon));
                            let tooltip = if healthy {
                                "claude-view — Running"
                            } else {
                                "claude-view — Stopped"
                            };
                            let _ = tray.set_tooltip(Some(tooltip));
                        }

                        if healthy {
                            if let Some(w) = app_handle.get_webview_window("main") {
                                let _ =
                                    w.navigate("http://localhost:47892".parse().unwrap());
                            }
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
