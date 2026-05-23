// src-tauri/src/companion.rs
// Companion detection module for prismSplit.
// Discovers whether prismConsole is installed (sibling workspace or production),
// and provides a detached async spawn mechanism for the [4] SUITE tab.
// Uses only std::env vars for Windows path resolution (no extra crates needed).

use serde::Serialize;
use std::path::PathBuf;

/// The mode in which the companion executable was found.
#[derive(Debug, Clone, Serialize)]
pub enum CompanionMode {
    /// Found in a local development workspace (sibling git repo).
    Dev,
    /// Found in a standard Windows production install location.
    Release,
}

/// Result of a companion detection probe.
#[derive(Debug, Clone, Serialize)]
pub struct CompanionStatus {
    /// True if prismConsole was found on disk.
    pub installed: bool,
    /// Absolute path to the discovered executable, if found.
    pub path: Option<String>,
    /// Resolution mode.
    pub mode: Option<CompanionMode>,
    /// Loose version hint derived from the build path.
    pub version_hint: Option<String>,
}

/// Resolve %LOCALAPPDATA% via env var (Windows-primary target).
fn local_app_data() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
}

/// Resolve candidate paths for `prismboard.exe` (prismConsole) in priority order.
fn candidate_paths() -> Vec<(PathBuf, CompanionMode)> {
    let mut candidates: Vec<(PathBuf, CompanionMode)> = Vec::new();
    let exe_name = "prismboard.exe";

    // ── Development workspace paths (sibling git repos) ──────────────────────
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dev_relatives = [
        "../prismboard/src-tauri/target/debug",
        "../prismboard/src-tauri/target/release",
        "../../prismboard/src-tauri/target/debug",
        "../../prismboard/src-tauri/target/release",
    ];
    for rel in &dev_relatives {
        let path = manifest_dir.join(rel).join(exe_name);
        let mode = if rel.contains("release") {
            CompanionMode::Release
        } else {
            CompanionMode::Dev
        };
        candidates.push((path, mode));
    }

    // ── Production Windows install locations ─────────────────────────────────
    if let Some(local_app_data) = local_app_data() {
        candidates.push((
            local_app_data.join("Programs").join("prismconsole").join(exe_name),
            CompanionMode::Release,
        ));
        candidates.push((
            local_app_data.join("prismconsole").join(exe_name),
            CompanionMode::Release,
        ));
    }

    // %ProgramFiles% fallbacks
    if let Ok(pf) = std::env::var("ProgramFiles") {
        candidates.push((
            PathBuf::from(&pf).join("prismconsole").join(exe_name),
            CompanionMode::Release,
        ));
    }
    if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
        candidates.push((
            PathBuf::from(&pf86).join("prismconsole").join(exe_name),
            CompanionMode::Release,
        ));
    }

    candidates
}

/// Probe all candidate paths and return the first valid one.
pub fn probe_companion() -> CompanionStatus {
    for (path, mode) in candidate_paths() {
        if path.is_file() {
            let version_hint = match &mode {
                CompanionMode::Dev => Some("dev-build".to_string()),
                CompanionMode::Release => Some("release".to_string()),
            };
            return CompanionStatus {
                installed: true,
                path: Some(path.to_string_lossy().to_string()),
                mode: Some(mode),
                version_hint,
            };
        }
    }

    CompanionStatus {
        installed: false,
        path: None,
        mode: None,
        version_hint: None,
    }
}

/// Launch prismConsole as a detached independent process.
/// Returns the PID of the spawned process.
pub fn launch_companion() -> Result<u32, String> {
    let status = probe_companion();
    let exe_path = status
        .path
        .ok_or_else(|| "prismConsole is not installed. Cannot launch companion.".to_string())?;

    let child = std::process::Command::new(&exe_path)
        .spawn()
        .map_err(|e| format!("Failed to launch prismConsole: {}", e))?;

    Ok(child.id())
}
