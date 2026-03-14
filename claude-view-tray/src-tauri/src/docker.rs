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
///
/// Priority:
/// 1. CLAUDE_VIEW_COMPOSE_PATH env var
/// 2. Bundled next to executable (Tauri resource)
/// 3. Fallback: %USERPROFILE%/.claude-view/docker-compose.yml
fn compose_file() -> PathBuf {
    if let Ok(p) = std::env::var("CLAUDE_VIEW_COMPOSE_PATH") {
        return PathBuf::from(p);
    }

    // Check bundled resource next to executable
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default();
    let bundled = exe_dir.join("docker-compose.yml");
    if bundled.exists() {
        return bundled;
    }

    // Fallback
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
