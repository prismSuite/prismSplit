
 # PrismSplit Code Audit

 > **Date:** 2026-06-09
 > **Scope:** `download_manager.rs`, `backend.rs`, `app.rs`, `app_paths.rs`, `preview.rs`, `state.rs`, `model_registry.rs`, `theme.rs`, `widgets.rs`
 > **Status:** Open — awaiting implementation

 ---

 ## 1. Download & Model Management (`src/download_manager.rs`)

 ### 1.1 Inconsistent Hashing APIs
 **Severity:** Medium
 **File:** `src/download_manager.rs`

 `md5_file` and `sha256_file` use different update styles (`hasher.update()` vs `sha2::Digest::update()`). This is inconsistent and error-prone for
 future maintainers.

 **Fix:** Extract a generic `hash_file` helper that accepts any `Digest` implementation, or at least unify the calling style.

 ```rust
 fn hash_file<D: Digest>(path: &Path) -> Result<String>
 ```

 ### 1.2 Missing Atomic Temp File Handling
 **Severity:** Critical
 **File:** `src/download_manager.rs`

 `download_file_with_progress` writes directly to `destination`. If the process is interrupted mid-download, a partial/corrupt file remains. The
 backend (`backend.rs`) does create a `.download` temp file, but then does `std::fs::copy` + `remove_file` instead of an atomic `rename`.

 **Fix:** Download to a unique temp file (e.g., `destination.incomplete.<random>`) and `std::fs::rename()` on success. Remove temp file on failure.

 ### 1.3 Hardcoded User-Agent
 **Severity:** Low
 **File:** `src/download_manager.rs`

 The `User-Agent` header is hardcoded to `"PrismSplit/0.1.0"`. Should be defined at a single source of truth (e.g., a crate-level constant or pulled
 from `Cargo.toml` at compile time via `env!("CARGO_PKG_VERSION")`).

 ### 1.4 Silent Checksum Skip is Dangerous
 **Severity:** Medium
 **File:** `src/download_manager.rs`

 `verify_sha256` silently accepts `"replace-with-real-sha256"` as a valid skip signal. A user downloading from a synced UVR catalog will never know
 their model was not integrity-verified.

 **Fix:** Return a typed result:
 ```rust
 pub_inside enum VerificationResult {
     Verified,
     SkippedPlaceholder,
     Failed { expected: String, actual: String },
 }
 ```
 The UI should display a warning (orange chip) when a model has no trusted hash.

 ### 1.5 No Retry / Resumable Download
 **Severity:** High
 **File:** `src/download_manager.rs`

 Network failures are terminal. There is no retry logic, no exponential backoff, and no resumable download via HTTP `Range` headers.

 **Fix:** Wrap the HTTP download in a retry loop (3 attempts, exponential backoff). For large model files (>50MB), support `Range` requests to resume
 interrupted downloads.

 ---

 ## 2. Backend `download_model` (`src/backend.rs`)

 ### 2.1 Race Condition on `models_dir` Mutex
 **Severity:** High
 **File:** `src/backend.rs` → `download_model()`

 The `models_dir` mutex is locked twice in sequence:
 ```rust
 let dir = self.model_registry.models_dir.lock()?; // lock 1
 std::fs::create_dir_all(&*dir)?;
 // ... later ...
 let destination = self.model_registry.installed_model_path(&entry)?; // lock 2
 ```
 The directory could change between the two locks, leading to writing a file to a different path than the one checked for existence.

 **Fix:** Lock once, hold the guard across both operations:
 ```rust
 let models_dir = self.model_registry.models_dir.lock().map_err(...)?;
 let destination = models_dir.join(&entry.filename);
 std::fs::create_dir_all(&*models_dir)?;
 // ... use destination ...
 ```

 ### 2.2 Non-Atomic Finalisation
 **Severity:** Critical
 **File:** `src/backend.rs` → `download_model()`

 ```rust
 std::fs::copy(&temp_destination, &destination)?; // ← copy, not rename
 let _ = std::fs::remove_file(&temp_destination);
 ```
 `std::fs::copy` is slow (duplicates bytes) and leaves the destination in a partially-written state if interrupted. `remove_file` return value is
 ignored.

 **Fix:** Use `std::fs::rename(&temp_destination, &destination)` which is atomic on the same filesystem. Always `remove_file` on the `Err` branch of a
 `try` block, never ignore it.

 ### 2.3 Progress Callback Floods the Channel
 **Severity:** High
 **File:** `src/backend.rs` → `download_model()`

 The `on_progress` closure is called for every network chunk (often 8KB). If the UI channel is bounded or the UI is repaint-bound, this floods the
 message queue.

 **Fix:** Throttle the callback. Only emit a message if progress changed by at least 1% or 500ms elapsed since the last update.

 ---

 ## 3. User Data & Configuration (`src/app_paths.rs`, `src/backend.rs`)

 ### 3.1 AppPaths Hardcodes Relative Root
 **Severity:** Critical
 **File:** `src/app_paths.rs`

 ```rust
 pub runtime_dir: root.join("runtime"),
 pub models_dir: root.join("models"),
 ```

 All user data (models, configs, cache, logs) is dumped into a single `root` directory, likely next to the binary. This violates platform conventions:
 - Linux: `~/.local/share/prismsplit/`
 - macOS: `~/Library/Application Support/prismsplit/`
 - Windows: `%APPDATA%\PrismSplit\`

 **Fix:** Use the `dirs` crate (already in `Cargo.toml`) to resolve proper platform directories:
 ```rust
 use dirs::{data_dir, cache_dir, config_dir};
 ```
 Models go to `data_dir()`, config to `config_dir()`, cache to `cache_dir()`, etc. The binary directory should be read-only.

 ### 3.2 `load_config` Silently Loses Corrupt Data
 **Severity:** Medium
 **File:** `src/backend.rs`

 ```rust
 fn load_config(path: &Path) -> AppConfig {
     if let Ok(content) = std::fs::read_to_string(path) {
         serde_json::from_str(&content).unwrap_or_default()
     } else {
         AppConfig::default()
     }
 }
 ```

 If `config.json` exists but contains invalid JSON, it is silently discarded and overwritten with defaults. The user loses all settings without
 warning.

 **Fix:** Distinguish between "file not found" (ok, use defaults) and "parse error" (log warning, backup the corrupt file, then use defaults).

 ### 3.3 `save_config` is Not Atomic
 **Severity:** Medium
 **File:** `src/backend.rs`

 ```rust
 let content = serde_json::to_string_pretty(config)?;
 std::fs::write(path, content)?;
 ```

 If the process is killed during `write`, the config file ends up truncated.

 **Fix:** Write to a temp file (e.g., `config.json.tmp`) then `rename` to `config.json`.

 ### 3.4 Config Schema Has No Versioning
 **Severity:** Low
 **File:** `src/models.rs` (AppConfig)

 `AppConfig` has no `version` field. If the schema changes in a future release, the app will deserialize old configs into the new shape (likely
 silently dropping fields or failing).

 **Fix:** Add a `version: u32` field. On load, if version < current, run a migration function.

 ---

 ## 4. EGUI / UI (`src/app.rs`)

 ### 4.1 `auto_save_config` Spawns a Task Every Frame
 **Severity:** Critical
 **File:** `src/app.rs` → `auto_save_config()`

 ```rust
 if changed {
     self.state.config = config.clone();
     let backend = Arc::clone(&self.backend);
     self.runtime.spawn(async move {
         let _ = backend.update_config(config);
     });
 }
 ```

 This runs at 60 FPS while any slider, text field, or toggle changes. It spawns 60 tokio tasks per second, all writing to disk.

 **Fix:** Implement debouncing. Track `last_change_time: Instant`. Only save after 1–2 seconds of idle. Use a single
 `Option<tokio::task::JoinHandle<()>>` and abort the previous one if a new change arrives.

 ### 4.2 Preview Analysis is Fire-and-Forget with No Timeout
 **Severity:** High
 **File:** `src/app.rs` → `AppMsg::ProcessFinished` handler

 After separation, a `runtime.spawn` calls `analyze_audio_peaks` for each stem. If `rodio::Decoder` hangs on a malformed file, the task never
 completes. The UI shows "Analyzing spectral data..." forever.

 **Fix:** Add a `tokio::time::timeout(Duration::from_secs(30), ...)` around the analysis. If it expires, send an `AppMsg::Log("WARNING: Preview
 analysis timed out")`.

 ### 4.3 No Cancellation for Long-Running Jobs
 **Severity:** High
 **File:** `src/app.rs` → `process_audio()`

 Once `process_audio` is called, the only way to stop it is killing the entire app (or waiting for the engine subprocess to finish). There is no "Stop"
 or "Cancel" button.

 **Fix:** Store the `JoinHandle` or child PID in state. Render a "CANCEL" button during processing that calls `child.kill()` or aborts the async task.

 ### 4.4 Drag & Drop Accepts Invalid Files
 **Severity:** Medium
 **File:** `src/app.rs` → `handle_dropped_files()`

 ```rust
 if let Some(path) = dropped.first()...
 ```

 No validation of file extension, MIME type, or whether it is a file vs directory. Dropping a folder or a `.txt` will set it as `input_file` and fail
 later.

 **Fix:** Filter by extension (`wav`, `mp3`, `flac`, `m4a`, `ogg`, `aac`). Show an error log if the dropped item is invalid.

 ### 4.5 Waveform Rendering Recomputes Every Frame
 **Severity:** Medium
 **File:** `src/app.rs` → `render_preview_window()`

 Inside the `Window::show` closure, the waveform is painted fresh every frame by iterating all `stem.peaks`. The peaks are already computed, but the
 painting logic (grid lines, bar calculations) runs 60 times per second.

 **Fix:** Cache the waveform into an `egui::TextureHandle` (or an `egui::Shape` list) when `PreviewStemsLoaded` arrives. In `render_preview_window`,
 just draw the cached mesh/texture.

 ### 4.6 eframe Persistence Feature is Unused
 **Severity:** Low
 **File:** `Cargo.toml`, `src/app.rs`

 `eframe` is compiled with `features = ["persistence"]` but `PrismSplitApp` does not implement any `save` / `load` for window size, position, or last
 active tab.

 **Fix:** Implement `eframe::App::save` to persist at least:
 - Window size and position
 - Last active `Tab`
 - Whether the preview window was open

 This provides a much better UX on restart.

 ---

 ## 5. Playback Controller (`src/preview.rs`)

 ### 5.1 `OutputStream` Created on Every `play()` Call
 **Severity:** Medium
 **File:** `src/preview.rs`

 ```rust
 let (stream, stream_handle) = OutputStream::try_default()?;
 let sink = Sink::try_new(&stream_handle)?;
 ```

 A new `OutputStream` is allocated every time the user hits play. This is slow and can fail if the default audio device is temporarily unavailable. The
 stream is dropped on `stop()`.

 **Fix:** Create the `OutputStream` once in `PlaybackController::new()` and reuse it. Only recreate if `play()` fails with a device error.

 ### 5.2 `PlaybackController` Does Not Report Errors on `stop()`
 **Severity:** Low
 **File:** `src/preview.rs`

 If `sink.stop()` fails (dangling pointer, device removed), the error is silently dropped.

 **Fix:** At minimum, log the error via `eprintln!` or a callback.

 ---

 ## 6. Model Registry (`src/model_registry.rs`)

 ### 6.1 `Mutex<PathBuf>` for `models_dir` is Unnecessary
 **Severity:** Low
 **File:** `src/model_registry.rs`

 `models_dir` is a `Mutex<PathBuf>` but it is only written in `set_models_dir` and read elsewhere. There is no concurrent mutation. This adds lock
 overhead and complexity.

 **Fix:** Use `std::sync::RwLock` or, better, wrap `ModelRegistry` in an `Arc<RwLock<_>>` at the `Backend` level and keep `models_dir` as a plain
 `PathBuf` inside the registry.

 ---

 ## 7. Summary Table

 | # | Severity | File | Issue | Key Action |
 |---|----------|------|-------|------------|
 | 1 | Critical | `download_manager.rs` | Non-atomic temp file handling | Use `rename`, not `copy` + `remove` |
 | 2 | Critical | `app_paths.rs` | Hardcoded relative root for data | Use `dirs` crate for platform dirs |
 | 3 | Critical | `app.rs` | `auto_save_config` spawns task every frame | Debounce saves (1–2s idle) |
 | 4 | High | `backend.rs` | Race condition on `models_dir` mutex | Lock once across operations |
 | 5 | High | `backend.rs` | No retry on download failure | Add retry loop + backoff |
 | 6 | High | `app.rs` | Preview analysis has no timeout | Wrap in `tokio::time::timeout` |
 | 7 | High | `app.rs` | No cancellation for running jobs | Add stop/cancel button + kill PID |
 | 8 | Medium | `download_manager.rs` | Silent hash skip | Return typed result + UI warning |
 | 9 | Medium | `backend.rs` | Progress callback floods channel | Throttle to 1% or 500ms |
 | 10 | Medium | `backend.rs` | `save_config` not atomic | Write to temp + `rename` |
 | 11 | Medium | `app.rs` | D&D accepts invalid files | Validate extension |
 | 12 | Medium | `app.rs` | Waveform repaints every frame | Cache as texture/shape |
 | 13 | Medium | `preview.rs` | New `OutputStream` per play | Reuse stream, recreate on failure |
 | 14 | Low | `backend.rs` | Config has no schema version | Add `version: u32` field |
 | 15 | Low | `app.rs` | eframe persistence unused | Save window state + last tab |
 | 16 | Low | `model_registry.rs` | Unnecessary `Mutex<PathBuf>` | Use `RwLock` or plain field |
 | 17 | Low | `download_manager.rs` | Hardcoded User-Agent | Use crate version constant |

 ---

 ## 8. Recommended Execution Order

 1. **Fix app paths** (Item 2) — touches almost every other file.
 2. **Fix `auto_save_config`Transaction** (Item 3) — high user impact, simple fix.
 3. **Fix `download_model` atomicity** (Items 1, 4, 5, 9) — core data integrity.
 4. **Add job cancellation** (Item 7) — major UX improvement.
 5. **Fix preview timeout** (Item 6) — prevents UI lockup.
 6. **Add debounced progress / D&D validation** (Items 8, 11) — polish.
 7. **Waveform caching + eframe persistence** (Items 12, 15) — performance.
