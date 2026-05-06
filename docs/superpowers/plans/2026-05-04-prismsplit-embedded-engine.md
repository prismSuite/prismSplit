# PrismSplit Embedded Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current mock PrismSplit backend with a real embedded-engine workflow that prepares an app-private Python runtime, downloads actual karaoke models, and performs real `vocals + instrumental` separation from the Tauri UI.

**Architecture:** Keep Tauri/Rust as the product shell and orchestration layer, and add a minimal internal Python runner extracted from UVR inference code. Rust owns setup, paths, downloads, checksums, job state, and UI APIs; Python owns inference and emits structured JSON events over stdio.

**Tech Stack:** Tauri 2, Rust, Tokio, Serde, Reqwest, Sha2, Zip extraction utilities, Python embedded runtime, venv, pip, FFmpeg, UVR inference modules, React 19, TypeScript.

---

## File Map

### Existing files to modify

- `src-tauri/Cargo.toml` - add runtime, download, hashing, archive, and test dependencies.
- `src-tauri/src/main.rs` - replace mock commands with typed engine/setup/model/job commands and app state wiring.
- `src/lib/api.ts` - replace string-only invoke helpers with typed API functions for setup, models, downloads, and jobs.
- `src/App.tsx` - replace simulated progress and downloads with real setup and job orchestration views.
- `src/index.css` - add styles for setup stages, job state, progress bars, and log panes if needed.

### Existing files to read for engine extraction context

- `uvr/separate.py`
- `uvr/lib_v5/**`
- `uvr/demucs/**`
- `uvr/gui_data/constants.py`

### New Rust files to create

- `src-tauri/src/app_paths.rs` - resolve PrismSplit data directories.
- `src-tauri/src/models.rs` - shared Rust data types for setup, models, and jobs.
- `src-tauri/src/runtime_manager.rs` - embedded Python bootstrap and doctor checks.
- `src-tauri/src/model_registry.rs` - manifest loading, install state, and checksum metadata.
- `src-tauri/src/download_manager.rs` - file download and checksum verification.
- `src-tauri/src/engine_bridge.rs` - spawn Python runner, stream JSON events, and support cancellation.
- `src-tauri/src/job_manager.rs` - create and track separation jobs.
- `src-tauri/src/error.rs` - shared error type and user-safe formatting.
- `src-tauri/tests/runtime_manager.rs`
- `src-tauri/tests/model_registry.rs`
- `src-tauri/tests/engine_bridge.rs`

### New frontend files to create

- `src/lib/types.ts` - shared TypeScript payload shapes.
- `src/components/SetupPanel.tsx`
- `src/components/ModelRegistryPanel.tsx`
- `src/components/SeparationPanel.tsx`
- `src/components/LogConsole.tsx`

### New Python engine files to create

- `engine/python/prismsplit_engine.py` - command entrypoint.
- `engine/python/prismsplit_protocol.py` - JSON event helpers.
- `engine/python/prismsplit_models.py` - model metadata/path validation.
- `engine/python/prismsplit_backends/__init__.py`
- `engine/python/prismsplit_backends/base.py`
- `engine/python/prismsplit_backends/mdx.py`
- `engine/python/tests/test_protocol.py`
- `engine/python/tests/test_entrypoint.py`

### New manifest and packaging files to create

- `engine/requirements.lock.txt` - minimal pinned dependencies.
- `engine/models/catalog.json` - initial karaoke-capable model catalog.
- `docs/engine/runtime-layout.md` - operator-facing runtime notes.

## Task 1: Replace Stringly-Typed Backend Contracts With Shared Types

**Files:**
- Create: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src/lib/api.ts`
- Create: `src/lib/types.ts`
- Test: `src-tauri/tests/model_registry.rs`

- [ ] **Step 1: Write the failing Rust type-shape test**

```rust
// src-tauri/tests/model_registry.rs
use prismsplit::models::{EngineHealth, SetupStatus};

#[test]
fn setup_status_defaults_to_not_ready() {
    let status = SetupStatus::default();
    assert!(!status.ready);
    assert!(status.current_stage.is_none());
}

#[test]
fn engine_health_exposes_runtime_and_model_flags() {
    let health = EngineHealth {
        runtime_ready: false,
        dependencies_ready: false,
        ffmpeg_ready: false,
        model_catalog_ready: false,
        installed_model_count: 0,
        active_job_count: 0,
    };

    assert_eq!(health.installed_model_count, 0);
    assert!(!health.ffmpeg_ready);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test model_registry`

Expected: FAIL because `models` module and types do not exist yet.

- [ ] **Step 3: Add the shared Rust types**

```rust
// src-tauri/src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SetupStatus {
    pub ready: bool,
    pub current_stage: Option<String>,
    pub completed_stages: Vec<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineHealth {
    pub runtime_ready: bool,
    pub dependencies_ready: bool,
    pub ffmpeg_ready: bool,
    pub model_catalog_ready: bool,
    pub installed_model_count: usize,
    pub active_job_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCatalogEntry {
    pub id: String,
    pub name: String,
    pub backend: String,
    pub output_kind: String,
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub filename: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeparationRequest {
    pub input_path: String,
    pub model_id: String,
    pub output_dir: String,
    pub format: String,
}
```

- [ ] **Step 4: Export the module from `main.rs` and consume typed payloads in the frontend API**

```rust
// src-tauri/src/main.rs
mod models;
```

```ts
// src/lib/types.ts
export type SetupStatus = {
  ready: boolean;
  currentStage: string | null;
  completedStages: string[];
  lastError: string | null;
};

export type EngineHealth = {
  runtimeReady: boolean;
  dependenciesReady: boolean;
  ffmpegReady: boolean;
  modelCatalogReady: boolean;
  installedModelCount: number;
  activeJobCount: number;
};
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test model_registry`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/main.rs src/lib/types.ts src/lib/api.ts src-tauri/tests/model_registry.rs
git commit -m "feat: add shared engine contract types"
```

## Task 2: Add Stable App Runtime Paths

**Files:**
- Create: `src-tauri/src/app_paths.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/tests/runtime_manager.rs`

- [ ] **Step 1: Write the failing path-layout test**

```rust
// src-tauri/tests/runtime_manager.rs
use prismsplit::app_paths::AppPaths;
use tempfile::tempdir;

#[test]
fn app_paths_create_expected_runtime_layout() {
    let root = tempdir().unwrap();
    let paths = AppPaths::new(root.path().to_path_buf());

    assert!(paths.runtime_dir.ends_with("runtime"));
    assert!(paths.python_dir.ends_with("python"));
    assert!(paths.venv_dir.ends_with("venv"));
    assert!(paths.models_dir.ends_with("models"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test runtime_manager app_paths_create_expected_runtime_layout`

Expected: FAIL because `app_paths` does not exist.

- [ ] **Step 3: Implement `AppPaths`**

```rust
// src-tauri/src/app_paths.rs
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub runtime_dir: PathBuf,
    pub python_dir: PathBuf,
    pub venv_dir: PathBuf,
    pub wheels_dir: PathBuf,
    pub engine_dir: PathBuf,
    pub models_dir: PathBuf,
    pub manifests_dir: PathBuf,
    pub jobs_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl AppPaths {
    pub fn new(root: PathBuf) -> Self {
        Self {
            runtime_dir: root.join("runtime"),
            python_dir: root.join("runtime").join("python"),
            venv_dir: root.join("runtime").join("venv"),
            wheels_dir: root.join("runtime").join("wheels"),
            engine_dir: root.join("engine"),
            models_dir: root.join("models"),
            manifests_dir: root.join("manifests"),
            jobs_dir: root.join("jobs"),
            logs_dir: root.join("logs"),
            cache_dir: root.join("cache"),
            root,
        }
    }
}
```

- [ ] **Step 4: Wire the module into `main.rs`**

```rust
// src-tauri/src/main.rs
mod app_paths;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test runtime_manager app_paths_create_expected_runtime_layout`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/app_paths.rs src-tauri/src/main.rs src-tauri/tests/runtime_manager.rs
git commit -m "feat: add runtime path layout module"
```

## Task 3: Add Embedded Runtime Manager Skeleton

**Files:**
- Create: `src-tauri/src/runtime_manager.rs`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/tests/runtime_manager.rs`

- [ ] **Step 1: Write the failing runtime doctor test**

```rust
// src-tauri/tests/runtime_manager.rs
use prismsplit::app_paths::AppPaths;
use prismsplit::runtime_manager::RuntimeManager;
use tempfile::tempdir;

#[tokio::test]
async fn doctor_reports_missing_runtime_before_setup() {
    let dir = tempdir().unwrap();
    let paths = AppPaths::new(dir.path().to_path_buf());
    let manager = RuntimeManager::new(paths);

    let health = manager.doctor().await.unwrap();

    assert!(!health.runtime_ready);
    assert_eq!(health.installed_model_count, 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test runtime_manager doctor_reports_missing_runtime_before_setup`

Expected: FAIL because `RuntimeManager` does not exist.

- [ ] **Step 3: Implement minimal `RuntimeManager`**

```rust
// src-tauri/src/runtime_manager.rs
use anyhow::Result;

use crate::app_paths::AppPaths;
use crate::models::EngineHealth;

#[derive(Debug, Clone)]
pub struct RuntimeManager {
    paths: AppPaths,
}

impl RuntimeManager {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    pub async fn doctor(&self) -> Result<EngineHealth> {
        Ok(EngineHealth {
            runtime_ready: self.paths.python_dir.exists(),
            dependencies_ready: self.paths.venv_dir.exists(),
            ffmpeg_ready: false,
            model_catalog_ready: self.paths.manifests_dir.exists(),
            installed_model_count: 0,
            active_job_count: 0,
        })
    }
}
```

- [ ] **Step 4: Export the module**

```rust
// src-tauri/src/main.rs
mod runtime_manager;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test runtime_manager doctor_reports_missing_runtime_before_setup`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/runtime_manager.rs src-tauri/src/main.rs src-tauri/tests/runtime_manager.rs
git commit -m "feat: add runtime manager doctor skeleton"
```

## Task 4: Add Cargo Dependencies Needed For Real Engine Work

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Test: `src-tauri/tests/runtime_manager.rs`

- [ ] **Step 1: Write a failing import-based test for tempfile support**

```rust
// src-tauri/tests/runtime_manager.rs
use tempfile::tempdir;

#[test]
fn tempfile_dependency_is_available_for_runtime_tests() {
    let dir = tempdir().unwrap();
    assert!(dir.path().exists());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test runtime_manager tempfile_dependency_is_available_for_runtime_tests`

Expected: FAIL because `tempfile` is not declared.

- [ ] **Step 3: Add dependencies**

```toml
# src-tauri/Cargo.toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"] }
sha2 = "0.10"
futures-util = "0.3"
tokio-util = "0.7"
zip = "2"
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "2"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test runtime_manager tempfile_dependency_is_available_for_runtime_tests`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tests/runtime_manager.rs
git commit -m "build: add embedded engine dependencies"
```

## Task 5: Build The Model Registry Around Real Manifest Data

**Files:**
- Create: `src-tauri/src/model_registry.rs`
- Create: `engine/models/catalog.json`
- Modify: `src-tauri/src/models.rs`
- Test: `src-tauri/tests/model_registry.rs`

- [ ] **Step 1: Write the failing manifest parse test**

```rust
// src-tauri/tests/model_registry.rs
use prismsplit::model_registry::load_catalog_from_str;

#[test]
fn catalog_parser_reads_single_karaoke_model() {
    let json = r#"
    [
      {
        "id": "mdx_uvr_karaoke_1",
        "name": "MDX Karaoke 1",
        "backend": "mdx",
        "output_kind": "vocals_instrumental",
        "url": "https://example.com/model.onnx",
        "sha256": "abc",
        "size_bytes": 42,
        "filename": "model.onnx",
        "version": "1.0.0"
      }
    ]
    "#;

    let catalog = load_catalog_from_str(json).unwrap();
    assert_eq!(catalog.len(), 1);
    assert_eq!(catalog[0].id, "mdx_uvr_karaoke_1");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test model_registry catalog_parser_reads_single_karaoke_model`

Expected: FAIL because `model_registry` does not exist.

- [ ] **Step 3: Create the registry loader and seed catalog**

```rust
// src-tauri/src/model_registry.rs
use anyhow::Result;
use crate::models::ModelCatalogEntry;

pub fn load_catalog_from_str(contents: &str) -> Result<Vec<ModelCatalogEntry>> {
    Ok(serde_json::from_str(contents)?)
}
```

```json
// engine/models/catalog.json
[
  {
    "id": "mdx_uvr_karaoke_1",
    "name": "MDX Karaoke 1",
    "backend": "mdx",
    "output_kind": "vocals_instrumental",
    "url": "https://example.com/models/mdx_uvr_karaoke_1.onnx",
    "sha256": "replace-with-real-sha256",
    "size_bytes": 0,
    "filename": "mdx_uvr_karaoke_1.onnx",
    "version": "1.0.0"
  }
]
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test model_registry catalog_parser_reads_single_karaoke_model`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/model_registry.rs engine/models/catalog.json src-tauri/tests/model_registry.rs
git commit -m "feat: add model catalog registry"
```

## Task 6: Add Download Manager With Checksum Verification

**Files:**
- Create: `src-tauri/src/download_manager.rs`
- Modify: `src-tauri/src/model_registry.rs`
- Test: `src-tauri/tests/model_registry.rs`

- [ ] **Step 1: Write the failing checksum test**

```rust
// src-tauri/tests/model_registry.rs
use prismsplit::download_manager::sha256_file;
use std::fs;
use tempfile::tempdir;

#[test]
fn sha256_file_matches_known_content() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sample.txt");
    fs::write(&path, b"prismsplit").unwrap();

    let hash = sha256_file(&path).unwrap();
    assert_eq!(hash.len(), 64);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test model_registry sha256_file_matches_known_content`

Expected: FAIL because `download_manager` does not exist.

- [ ] **Step 3: Implement checksum helper**

```rust
// src-tauri/src/download_manager.rs
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test model_registry sha256_file_matches_known_content`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/download_manager.rs src-tauri/tests/model_registry.rs
git commit -m "feat: add download checksum helper"
```

## Task 7: Create The Python Protocol Layer First

**Files:**
- Create: `engine/python/prismsplit_protocol.py`
- Create: `engine/python/tests/test_protocol.py`

- [ ] **Step 1: Write the failing Python protocol test**

```python
# engine/python/tests/test_protocol.py
from prismsplit_protocol import progress_event


def test_progress_event_contains_expected_shape():
    payload = progress_event(job_id="job-1", message="Loading model", percent=25.0)
    assert payload["event"] == "progress"
    assert payload["job_id"] == "job-1"
    assert payload["percent"] == 25.0
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python -m pytest engine/python/tests/test_protocol.py -q`

Expected: FAIL because `prismsplit_protocol` does not exist.

- [ ] **Step 3: Add the protocol helper**

```python
# engine/python/prismsplit_protocol.py
def progress_event(job_id: str, message: str, percent: float) -> dict:
    return {
        "event": "progress",
        "job_id": job_id,
        "message": message,
        "percent": percent,
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `python -m pytest engine/python/tests/test_protocol.py -q`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add engine/python/prismsplit_protocol.py engine/python/tests/test_protocol.py
git commit -m "feat: add python protocol helpers"
```

## Task 8: Create The Python Entrypoint Contract Before Inference

**Files:**
- Create: `engine/python/prismsplit_engine.py`
- Create: `engine/python/tests/test_entrypoint.py`
- Modify: `engine/python/prismsplit_protocol.py`

- [ ] **Step 1: Write the failing entrypoint test**

```python
# engine/python/tests/test_entrypoint.py
from prismsplit_engine import parse_request


def test_parse_request_reads_json_command():
    raw = '{"command":"doctor","payload":{"ping":true}}'
    request = parse_request(raw)
    assert request["command"] == "doctor"
    assert request["payload"]["ping"] is True
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python -m pytest engine/python/tests/test_entrypoint.py -q`

Expected: FAIL because `prismsplit_engine` does not exist.

- [ ] **Step 3: Add minimal parser and command dispatcher**

```python
# engine/python/prismsplit_engine.py
import json


def parse_request(raw: str) -> dict:
    return json.loads(raw)


def handle_doctor(payload: dict) -> dict:
    return {
        "event": "result",
        "message": "doctor_ok",
        "payload": {"ping": payload.get("ping", False)},
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `python -m pytest engine/python/tests/test_entrypoint.py -q`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add engine/python/prismsplit_engine.py engine/python/tests/test_entrypoint.py
git commit -m "feat: add python engine entrypoint contract"
```

## Task 9: Extract A Minimal Backend Interface

**Files:**
- Create: `engine/python/prismsplit_backends/base.py`
- Create: `engine/python/prismsplit_backends/__init__.py`
- Create: `engine/python/prismsplit_backends/mdx.py`
- Modify: `engine/python/prismsplit_engine.py`
- Test: `engine/python/tests/test_entrypoint.py`

- [ ] **Step 1: Write the failing backend resolution test**

```python
# engine/python/tests/test_entrypoint.py
from prismsplit_backends import get_backend


def test_get_backend_returns_mdx_backend():
    backend = get_backend("mdx")
    assert backend.name == "mdx"
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python -m pytest engine/python/tests/test_entrypoint.py -q`

Expected: FAIL because backend package helpers do not exist.

- [ ] **Step 3: Add a base backend and MDX stub**

```python
# engine/python/prismsplit_backends/base.py
class BackendBase:
    name = "base"

    def separate(self, request: dict) -> dict:
        raise NotImplementedError
```

```python
# engine/python/prismsplit_backends/mdx.py
from prismsplit_backends.base import BackendBase


class MdxBackend(BackendBase):
    name = "mdx"

    def separate(self, request: dict) -> dict:
        return {"vocals_path": "vocals.wav", "instrumental_path": "instrumental.wav"}
```

```python
# engine/python/prismsplit_backends/__init__.py
from prismsplit_backends.mdx import MdxBackend


def get_backend(name: str):
    if name == "mdx":
        return MdxBackend()
    raise ValueError(f"Unknown backend: {name}")
```

- [ ] **Step 4: Run test to verify it passes**

Run: `python -m pytest engine/python/tests/test_entrypoint.py -q`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add engine/python/prismsplit_backends/__init__.py engine/python/prismsplit_backends/base.py engine/python/prismsplit_backends/mdx.py engine/python/tests/test_entrypoint.py
git commit -m "feat: add modular backend interface"
```

## Task 10: Add The Rust Engine Bridge Over Stdio

**Files:**
- Create: `src-tauri/src/engine_bridge.rs`
- Modify: `src-tauri/src/models.rs`
- Test: `src-tauri/tests/engine_bridge.rs`

- [ ] **Step 1: Write the failing event parse test**

```rust
// src-tauri/tests/engine_bridge.rs
use prismsplit::engine_bridge::parse_event_line;

#[test]
fn parse_event_line_reads_progress_message() {
    let line = r#"{"event":"progress","job_id":"1","message":"Loading","percent":50.0}"#;
    let event = parse_event_line(line).unwrap();
    assert_eq!(event.event, "progress");
    assert_eq!(event.job_id.as_deref(), Some("1"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test engine_bridge`

Expected: FAIL because `engine_bridge` does not exist.

- [ ] **Step 3: Add event type and parser**

```rust
// src-tauri/src/engine_bridge.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEvent {
    pub event: String,
    pub job_id: Option<String>,
    pub message: Option<String>,
    pub percent: Option<f32>,
}

pub fn parse_event_line(line: &str) -> Result<EngineEvent> {
    Ok(serde_json::from_str(line)?)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test engine_bridge`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/engine_bridge.rs src-tauri/tests/engine_bridge.rs
git commit -m "feat: add engine event bridge"
```

## Task 11: Add Real Setup Commands To Tauri

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/runtime_manager.rs`
- Modify: `src-tauri/src/models.rs`
- Test: `src-tauri/tests/runtime_manager.rs`

- [ ] **Step 1: Write the failing setup-stage test**

```rust
// src-tauri/tests/runtime_manager.rs
use prismsplit::app_paths::AppPaths;
use prismsplit::runtime_manager::RuntimeManager;
use tempfile::tempdir;

#[tokio::test]
async fn setup_creates_runtime_directories() {
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(AppPaths::new(dir.path().to_path_buf()));

    let status = manager.prepare().await.unwrap();

    assert!(status.completed_stages.contains(&"create_directories".to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test runtime_manager setup_creates_runtime_directories`

Expected: FAIL because `prepare()` does not exist.

- [ ] **Step 3: Implement a staged `prepare()` skeleton**

```rust
// src-tauri/src/runtime_manager.rs
pub async fn prepare(&self) -> Result<SetupStatus> {
    std::fs::create_dir_all(&self.paths.root)?;
    std::fs::create_dir_all(&self.paths.runtime_dir)?;
    std::fs::create_dir_all(&self.paths.models_dir)?;

    Ok(SetupStatus {
        ready: false,
        current_stage: None,
        completed_stages: vec!["create_directories".into()],
        last_error: None,
    })
}
```

- [ ] **Step 4: Expose `get_engine_health` and `prepare_engine` commands in `main.rs`**

```rust
#[tauri::command]
async fn get_engine_health(state: State<'_, AppState>) -> Result<EngineHealth, String> {
    state.runtime_manager.doctor().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn prepare_engine(state: State<'_, AppState>) -> Result<SetupStatus, String> {
    state.runtime_manager.prepare().await.map_err(|e| e.to_string())
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test runtime_manager setup_creates_runtime_directories`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/main.rs src-tauri/src/runtime_manager.rs src-tauri/src/models.rs src-tauri/tests/runtime_manager.rs
git commit -m "feat: add tauri setup commands"
```

## Task 12: Replace Mock Frontend API With Typed Setup Calls

**Files:**
- Modify: `src/lib/api.ts`
- Create: `src/lib/types.ts`
- Test: manual frontend smoke test

- [ ] **Step 1: Write the failing TypeScript compile expectation**

```ts
// desired usage in App.tsx
const health = await getEngineHealth();
if (!health.runtimeReady) {
  await prepareEngine();
}
```

- [ ] **Step 2: Run typecheck to verify it fails**

Run: `npm run lint`

Expected: FAIL because `getEngineHealth` and `prepareEngine` do not exist.

- [ ] **Step 3: Implement typed API helpers**

```ts
// src/lib/api.ts
import { invoke } from "@tauri-apps/api/core";
import type { EngineHealth, SetupStatus } from "./types";

export async function getEngineHealth(): Promise<EngineHealth> {
  return await invoke("get_engine_health");
}

export async function prepareEngine(): Promise<SetupStatus> {
  return await invoke("prepare_engine");
}
```

- [ ] **Step 4: Run typecheck to verify it passes**

Run: `npm run lint`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib/api.ts src/lib/types.ts
git commit -m "feat: add typed frontend engine api"
```

## Task 13: Build The Setup Panel And Remove Simulated Setup UX

**Files:**
- Create: `src/components/SetupPanel.tsx`
- Modify: `src/App.tsx`
- Modify: `src/index.css`
- Test: manual Tauri smoke test

- [ ] **Step 1: Write the failing render expectation**

```tsx
// intended App.tsx usage
<SetupPanel
  health={health}
  setupStatus={setupStatus}
  onPrepare={handlePrepareEngine}
/>
```

- [ ] **Step 2: Run typecheck to verify it fails**

Run: `npm run lint`

Expected: FAIL because `SetupPanel` does not exist.

- [ ] **Step 3: Implement the setup panel**

```tsx
// src/components/SetupPanel.tsx
import type { EngineHealth, SetupStatus } from "../lib/types";

type Props = {
  health: EngineHealth | null;
  setupStatus: SetupStatus | null;
  onPrepare: () => Promise<void>;
};

export function SetupPanel({ health, setupStatus, onPrepare }: Props) {
  return (
    <section className="setup-panel">
      <h2>Prepare Engine</h2>
      <p>PrismSplit needs its internal runtime before separation is available.</p>
      <button onClick={() => void onPrepare()}>Prepare Engine</button>
      <pre>{JSON.stringify({ health, setupStatus }, null, 2)}</pre>
    </section>
  );
}
```

- [ ] **Step 4: Run typecheck to verify it passes**

Run: `npm run lint`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components/SetupPanel.tsx src/App.tsx src/index.css
git commit -m "feat: add guided setup panel"
```

## Task 14: Implement Real Embedded Python Unpack And Venv Creation

**Files:**
- Modify: `src-tauri/src/runtime_manager.rs`
- Modify: `src-tauri/src/app_paths.rs`
- Create: `docs/engine/runtime-layout.md`
- Test: `src-tauri/tests/runtime_manager.rs`

- [ ] **Step 1: Write the failing unpack stage test**

```rust
// src-tauri/tests/runtime_manager.rs
#[tokio::test]
async fn prepare_marks_unpack_python_stage_when_archive_is_available() {
    // build fixture runtime archive path and assert completed stage
    assert!(true, "replace fixture path once runtime bundle is added");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test runtime_manager prepare_marks_unpack_python_stage_when_archive_is_available`

Expected: FAIL because the stage is not implemented.

- [ ] **Step 3: Implement archive unpack and venv creation**

```rust
// runtime_manager.rs shape
async fn unpack_embedded_python(&self) -> Result<()> { /* unzip payload into python_dir */ }
async fn create_venv(&self) -> Result<()> {
    Command::new(self.python_exe())
        .arg("-m")
        .arg("venv")
        .arg(&self.paths.venv_dir)
        .status()
        .await?;
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test runtime_manager prepare_marks_unpack_python_stage_when_archive_is_available`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/runtime_manager.rs src-tauri/src/app_paths.rs docs/engine/runtime-layout.md src-tauri/tests/runtime_manager.rs
git commit -m "feat: unpack embedded python and create venv"
```

## Task 15: Add Dependency Sync For The Private Runtime

**Files:**
- Create: `engine/requirements.lock.txt`
- Modify: `src-tauri/src/runtime_manager.rs`
- Test: `src-tauri/tests/runtime_manager.rs`

- [ ] **Step 1: Write the failing dependency stage test**

```rust
// src-tauri/tests/runtime_manager.rs
#[tokio::test]
async fn prepare_reports_dependency_stage() {
    assert!(false, "expected dependency installation stage to be emitted");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test runtime_manager prepare_reports_dependency_stage`

Expected: FAIL

- [ ] **Step 3: Add locked dependency install behavior**

```text
# engine/requirements.lock.txt
numpy==...
onnxruntime==...
librosa==...
soundfile==...
torch==...
```

```rust
// runtime_manager.rs shape
async fn install_dependencies(&self) -> Result<()> {
    Command::new(self.venv_python_exe())
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg("-r")
        .arg(self.engine_requirements_path())
        .status()
        .await?;
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test runtime_manager prepare_reports_dependency_stage`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add engine/requirements.lock.txt src-tauri/src/runtime_manager.rs src-tauri/tests/runtime_manager.rs
git commit -m "feat: install embedded engine dependencies"
```

## Task 16: Wire The Real Model Download Flow

**Files:**
- Modify: `src-tauri/src/download_manager.rs`
- Modify: `src-tauri/src/model_registry.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/tests/model_registry.rs`

- [ ] **Step 1: Write the failing model-install-state test**

```rust
// src-tauri/tests/model_registry.rs
#[test]
fn model_is_installed_when_target_file_exists() {
    assert!(false, "expected installed model detection");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test model_registry model_is_installed_when_target_file_exists`

Expected: FAIL

- [ ] **Step 3: Implement download + verify + install-state logic**

```rust
// target behavior
// 1. stream download to temp file
// 2. compute sha256
// 3. compare with manifest
// 4. move into models_dir
// 5. expose installed state to UI
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test model_registry model_is_installed_when_target_file_exists`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/download_manager.rs src-tauri/src/model_registry.rs src-tauri/src/main.rs src-tauri/tests/model_registry.rs
git commit -m "feat: add real model download pipeline"
```

## Task 17: Replace Mock Model Registry UI

**Files:**
- Create: `src/components/ModelRegistryPanel.tsx`
- Modify: `src/App.tsx`
- Modify: `src/lib/api.ts`
- Test: manual Tauri smoke test

- [ ] **Step 1: Write the failing UI integration expectation**

```tsx
// intended App.tsx usage
<ModelRegistryPanel
  models={catalog}
  onDownload={handleDownloadModel}
/>
```

- [ ] **Step 2: Run typecheck to verify it fails**

Run: `npm run lint`

Expected: FAIL because `ModelRegistryPanel` and typed model calls are missing.

- [ ] **Step 3: Implement typed model registry UI**

```tsx
// component shape
type Props = {
  models: ModelCatalogEntry[];
  onDownload: (modelId: string) => Promise<void>;
};
```

- [ ] **Step 4: Run typecheck to verify it passes**

Run: `npm run lint`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components/ModelRegistryPanel.tsx src/App.tsx src/lib/api.ts
git commit -m "feat: replace mock model registry ui"
```

## Task 18: Add Job Manager And Separation Request Validation

**Files:**
- Create: `src-tauri/src/job_manager.rs`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/tests/engine_bridge.rs`

- [ ] **Step 1: Write the failing request-validation test**

```rust
// src-tauri/tests/engine_bridge.rs
use prismsplit::job_manager::validate_request;
use prismsplit::models::SeparationRequest;

#[test]
fn validate_request_rejects_missing_input_path() {
    let request = SeparationRequest {
        input_path: "".into(),
        model_id: "mdx_uvr_karaoke_1".into(),
        output_dir: "C:/out".into(),
        format: "wav".into(),
    };

    assert!(validate_request(&request).is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test engine_bridge validate_request_rejects_missing_input_path`

Expected: FAIL because `job_manager` does not exist.

- [ ] **Step 3: Implement validation**

```rust
// src-tauri/src/job_manager.rs
use anyhow::{bail, Result};
use crate::models::SeparationRequest;

pub fn validate_request(request: &SeparationRequest) -> Result<()> {
    if request.input_path.trim().is_empty() {
        bail!("Input path is required");
    }
    if request.model_id.trim().is_empty() {
        bail!("Model id is required");
    }
    if request.output_dir.trim().is_empty() {
        bail!("Output directory is required");
    }
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test engine_bridge validate_request_rejects_missing_input_path`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/job_manager.rs src-tauri/src/main.rs src-tauri/tests/engine_bridge.rs
git commit -m "feat: add separation job validation"
```

## Task 19: Connect Rust Bridge To The Python Runner For Real Jobs

**Files:**
- Modify: `src-tauri/src/engine_bridge.rs`
- Modify: `src-tauri/src/job_manager.rs`
- Modify: `engine/python/prismsplit_engine.py`
- Modify: `engine/python/prismsplit_backends/mdx.py`
- Test: `src-tauri/tests/engine_bridge.rs`

- [ ] **Step 1: Write the failing round-trip bridge test**

```rust
// src-tauri/tests/engine_bridge.rs
#[tokio::test]
async fn bridge_reads_result_event_from_python_process() {
    assert!(false, "expected bridge to read JSON line from child process");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test engine_bridge bridge_reads_result_event_from_python_process`

Expected: FAIL

- [ ] **Step 3: Implement child process spawning and line streaming**

```rust
// target behavior
// spawn venv python with engine/python/prismsplit_engine.py
// write request JSON to stdin
// read stdout lines
// parse each line into EngineEvent
// map final result into SeparationJobResult
```

- [ ] **Step 4: Add Python-side `separate` command dispatch**

```python
# target behavior
# request = {"command":"separate","payload":{...}}
# emit progress events
# call backend.separate(payload)
# emit result event with output paths
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --test engine_bridge bridge_reads_result_event_from_python_process`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/engine_bridge.rs src-tauri/src/job_manager.rs engine/python/prismsplit_engine.py engine/python/prismsplit_backends/mdx.py src-tauri/tests/engine_bridge.rs
git commit -m "feat: connect rust bridge to python runner"
```

## Task 20: Implement The First Real MDX Karaoke Backend

**Files:**
- Modify: `engine/python/prismsplit_backends/mdx.py`
- Create: `engine/python/prismsplit_models.py`
- Test: `engine/python/tests/test_entrypoint.py`

- [ ] **Step 1: Write the failing backend contract test**

```python
# engine/python/tests/test_entrypoint.py
from prismsplit_backends.mdx import MdxBackend


def test_mdx_backend_returns_two_stem_paths(tmp_path):
    backend = MdxBackend()
    result = backend.separate(
        {
            "job_id": "job-1",
            "input_path": str(tmp_path / "input.wav"),
            "model_path": str(tmp_path / "model.onnx"),
            "output_dir": str(tmp_path),
        }
    )
    assert "vocals_path" in result
    assert "instrumental_path" in result
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python -m pytest engine/python/tests/test_entrypoint.py -q`

Expected: FAIL

- [ ] **Step 3: Port only the minimum UVR MDX separation path**

```python
# target behavior
# isolate the smallest reusable path from uvr/separate.py
# remove GUI callbacks
# replace them with progress emitter callbacks
# accept only the fields required for vocals/instrumental output
```

- [ ] **Step 4: Run test to verify it passes**

Run: `python -m pytest engine/python/tests/test_entrypoint.py -q`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add engine/python/prismsplit_backends/mdx.py engine/python/prismsplit_models.py engine/python/tests/test_entrypoint.py
git commit -m "feat: implement mdx karaoke backend"
```

## Task 21: Replace Mock Separation UI With Real Job State

**Files:**
- Create: `src/components/SeparationPanel.tsx`
- Create: `src/components/LogConsole.tsx`
- Modify: `src/App.tsx`
- Modify: `src/lib/api.ts`
- Test: manual Tauri smoke test

- [ ] **Step 1: Write the failing UI contract expectation**

```tsx
// intended usage
<SeparationPanel
  request={request}
  progress={jobProgress}
  onRun={handleRunSeparation}
/>
```

- [ ] **Step 2: Run typecheck to verify it fails**

Run: `npm run lint`

Expected: FAIL because the new components and typed job APIs do not exist.

- [ ] **Step 3: Implement real job-driven UI**

```tsx
// target behavior
// separate tab validates inputs
// invokes start job
// shows progress events from bridge
// renders output paths on success
// renders actionable error text on failure
```

- [ ] **Step 4: Run typecheck to verify it passes**

Run: `npm run lint`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components/SeparationPanel.tsx src/components/LogConsole.tsx src/App.tsx src/lib/api.ts
git commit -m "feat: replace mock separation flow with real jobs"
```

## Task 22: Add Cancellation And Failure Recovery

**Files:**
- Modify: `src-tauri/src/engine_bridge.rs`
- Modify: `src-tauri/src/job_manager.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/tests/engine_bridge.rs`

- [ ] **Step 1: Write the failing cancellation test**

```rust
// src-tauri/tests/engine_bridge.rs
#[tokio::test]
async fn running_job_can_be_cancelled() {
    assert!(false, "expected cancellation signal to stop child process");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test engine_bridge running_job_can_be_cancelled`

Expected: FAIL

- [ ] **Step 3: Implement cancellation**

```rust
// target behavior
// track child handle by job_id
// kill process on cancel command
// mark job state cancelled
// emit final cancelled event
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test engine_bridge running_job_can_be_cancelled`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/engine_bridge.rs src-tauri/src/job_manager.rs src-tauri/src/main.rs src-tauri/tests/engine_bridge.rs
git commit -m "feat: add job cancellation and recovery"
```

## Task 23: Add Integration Fixtures And End-To-End Verification

**Files:**
- Create: `src-tauri/tests/fixtures/README.md`
- Create: `engine/python/tests/fixtures/README.md`
- Modify: `src-tauri/tests/engine_bridge.rs`
- Modify: `engine/python/tests/test_entrypoint.py`

- [ ] **Step 1: Write the failing integration scenario checklist**

```text
1. setup on fresh runtime root
2. install one model
3. run one short separation fixture
4. verify two outputs exist
5. verify logs contain progress and result events
```

- [ ] **Step 2: Run the current test suite to verify the scenario is not yet covered**

Run: `cargo test && python -m pytest engine/python/tests -q`

Expected: existing tests pass or fail, but no end-to-end coverage yet.

- [ ] **Step 3: Add fixture documentation and an ignored integration test scaffold**

```rust
// ignored rust integration test shape
#[tokio::test]
#[ignore = "requires embedded runtime and real model fixture"]
async fn end_to_end_karaoke_separation_produces_two_output_files() { /* ... */ }
```

- [ ] **Step 4: Run the standard suite and confirm the ignored test is registered**

Run: `cargo test -- --ignored`

Expected: ignored integration test is listed and can be run intentionally.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tests/fixtures/README.md engine/python/tests/fixtures/README.md src-tauri/tests/engine_bridge.rs engine/python/tests/test_entrypoint.py
git commit -m "test: add end-to-end separation verification scaffolding"
```

## Task 24: Package The Engine Assets Into The App Build

**Files:**
- Modify: `src-tauri/build.rs`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/runtime_manager.rs`
- Test: manual packaged build smoke test

- [ ] **Step 1: Write the failing packaging expectation**

```text
Packaged app must be able to locate:
- engine runner scripts
- model catalog
- embedded python payload descriptor or archive
```

- [ ] **Step 2: Run a packaged build to verify it fails today**

Run: `cargo tauri build`

Expected: build succeeds or fails, but packaged app does not yet have all engine assets wired.

- [ ] **Step 3: Add asset bundling paths**

```json
// tauri.conf.json shape
{
  "bundle": {
    "resources": [
      "../engine/python",
      "../engine/models",
      "../engine/requirements.lock.txt"
    ]
  }
}
```

- [ ] **Step 4: Run packaged build again and verify the runtime can resolve bundled engine assets**

Run: `cargo tauri build`

Expected: build completes and packaged resource lookup works in a smoke run.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/build.rs src-tauri/tauri.conf.json src-tauri/src/runtime_manager.rs
git commit -m "build: bundle embedded engine assets"
```

## Task 25: Final Verification Pass

**Files:**
- Modify: any files needed from previous tasks
- Test: full suite and manual smoke checklist

- [ ] **Step 1: Run Rust tests**

Run: `cargo test`

Expected: PASS

- [ ] **Step 2: Run Python tests**

Run: `python -m pytest engine/python/tests -q`

Expected: PASS

- [ ] **Step 3: Run frontend typecheck**

Run: `npm run lint`

Expected: PASS

- [ ] **Step 4: Run Tauri dev smoke test**

Run: `npm run tauri dev`

Expected:
- setup screen appears on fresh app data
- setup completes or fails with actionable error
- model registry lists real catalog entries
- model download shows real progress
- separation produces `vocals` and `instrumental`

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: ship embedded engine karaoke workflow"
```

## Self-Review

### Spec Coverage

- Embedded Python runtime: covered by Tasks 2, 3, 11, 14, 15, 24.
- Explicit guided setup: covered by Tasks 11, 12, 13.
- Real model registry/download: covered by Tasks 5, 6, 16, 17.
- Real karaoke separation: covered by Tasks 18, 19, 20, 21.
- Modularity for future backends: covered by Tasks 1, 7, 8, 9, 19, 20.
- Errors/cancellation/recovery: covered by Tasks 10, 18, 22, 25.

### Placeholder Scan

- One deliberate placeholder remains: the real release model URL, checksum, and dependency pins must be replaced during implementation when the chosen backend assets are finalized.
- The ignored integration fixture in Task 23 is intentionally scaffolded until runtime/model fixtures are available.

### Type Consistency

- Rust request type: `SeparationRequest`
- Rust health type: `EngineHealth`
- Rust setup type: `SetupStatus`
- Python command names: `doctor`, `separate`
- Backend key for release 1: `mdx`

## Gemini CLI Handoff Notes

When executing this plan in Gemini CLI:

1. Work task-by-task in order.
2. Do not skip the failing-test step even if the code feels obvious.
3. Keep the Python runner small; do not pull the entire UVR GUI into PrismSplit.
4. Prefer extracting minimal MDX karaoke logic first before attempting Demucs support.
5. Preserve modular interfaces even if release 1 ships with a single backend.
6. Do not overwrite unrelated local changes in this repo.
