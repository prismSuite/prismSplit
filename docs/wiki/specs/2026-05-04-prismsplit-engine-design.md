# PrismSplit Engine Design

## Summary

PrismSplit will become a Windows-first `Rust + Tauri` desktop application with an embedded, app-managed Python runtime used only as an internal separation engine. The first release will support a single job type: `karaoke separation` (`vocals` + `instrumental`).

The existing Ultimate Vocal Remover GUI repository inside this workspace will be treated as a source of engine logic, not as the application itself. PrismSplit must not launch the legacy UVR GUI, depend on Tkinter UX flows, or expose Python setup work directly to end users.

## Goals

- Deliver real model downloads and real audio separation in the Tauri app.
- Keep the user-facing app lightweight and visually modern.
- Avoid requiring a system Python installation.
- Make engine setup explicit, recoverable, and observable inside the app.
- Keep the architecture modular so additional backends, ensembles, and stem combinations can be added later without rewriting the UI.

## Non-Goals For Release 1

- No legacy UVR GUI embedding.
- No training mode.
- No 4-stem separation in the first release.
- No ensemble chaining in the first release.
- No macOS or Linux support in the first release.
- No full rewrite of UVR inference to Rust in this phase.

## Product Scope

Release 1 covers:

- Embedded Python runtime managed by PrismSplit.
- Guided setup flow inside the app.
- Real model registry with downloadable manifests.
- Real model download with checksum verification.
- Real separation jobs for one audio input into `vocals.wav` and `instrumental.wav`.
- Real progress/log streaming back into the Tauri UI.
- Local cache management for runtime, dependencies, models, logs, temp files, and job outputs.

## High-Level Architecture

PrismSplit is split into two layers:

1. `Desktop shell and orchestration` in Rust/Tauri.
2. `Inference engine` in Python, launched and supervised by Rust.

Rust owns:

- setup state
- directories
- downloads
- checksums
- job lifecycle
- UI-facing status
- structured logs
- cancellation
- future backend selection

Python owns:

- audio loading
- model loading
- inference
- output writing
- backend-specific progress events

The Python process is an internal worker, not an independent product surface.

## Runtime Layout

PrismSplit will maintain an app-private runtime tree under an application data root, e.g.:

`%APPDATA%/PrismSplit/`

Suggested layout:

- `runtime/python/` - embedded Python distribution
- `runtime/venv/` - private environment built from embedded Python
- `runtime/wheels/` - cached wheel downloads or offline wheel bundle
- `engine/` - PrismSplit runner scripts copied from the app bundle
- `models/` - downloaded model files
- `manifests/` - model catalog snapshots
- `jobs/` - per-job temp/output metadata
- `logs/` - setup logs and job logs
- `cache/` - transient files

This layout keeps runtime, assets, and outputs isolated and easy to inspect or repair.

## Setup Flow

The first meaningful user interaction is a setup health check.

If the engine is not ready, PrismSplit shows a `Prepare Engine` screen with explicit stages:

1. verify app directories
2. unpack embedded Python runtime
3. create virtual environment
4. install or sync required dependencies
5. validate engine runner files
6. validate FFmpeg availability strategy
7. write setup manifest
8. run final doctor check

Each stage must emit:

- machine-readable status
- human-readable message
- elapsed time
- actionable recovery text on failure

Setup is resumable. If a stage already completed and its artifacts are valid, PrismSplit skips it on retry.

## Separation Model Strategy

Release 1 targets only the `vocals/instrumental` workflow, but the system must be built around generic model descriptors rather than hard-coded UI strings.

Each model descriptor should include:

- stable `id`
- display `name`
- backend type
- supported output mode
- download URL
- checksum
- file size
- local filename
- version
- optional notes about quality or hardware fit

Rust stores and serves the registry to the UI. The UI should not know whether a model is MDX, Demucs, or another backend.

## Backend Modularity

The engine must expose a stable contract like:

- `doctor`
- `setup-validate`
- `list-installed-models`
- `separate`

Inside Python, backend-specific logic is hidden behind a common interface. Release 1 can ship with one real backend path, but the shape must support later additions:

- `mdx`
- `demucs`
- `ensemble`
- `de-reverb`

That means the job request should describe desired behavior, not internal implementation details.

## Rust Responsibilities

Rust modules should cover:

- app paths
- setup/runtime manager
- model registry manager
- download manager
- engine bridge
- job manager
- event streaming

Tauri commands should move from mock commands to typed operations. The frontend should consume typed payloads rather than string-only success messages.

## Python Responsibilities

The Python runner should be minimal and detached from UVR GUI assumptions.

It should:

- parse JSON commands
- resolve local model files
- open the input audio
- run inference with the selected backend
- emit JSON progress events
- write output files
- emit structured success or error payloads

The Python runner should reuse as much existing UVR inference code as practical, but only after isolating the parts needed for release 1.

## IPC Contract

Rust and Python should communicate with newline-delimited JSON messages over stdio.

Required event kinds:

- `stage`
- `progress`
- `log`
- `result`
- `error`

Each message should include:

- `event`
- `job_id` where relevant
- `message`
- optional `percent`
- optional payload object

This makes the bridge testable and keeps UI updates deterministic.

## Error Handling

Setup and job execution must fail loudly and recoverably.

Important failure classes:

- corrupted embedded runtime
- venv creation failure
- dependency install failure
- missing or invalid model file
- checksum mismatch
- unsupported audio input
- inference crash
- disk space exhaustion
- cancellation

The UI should never be left with an ambiguous “hung” state. Every job or setup stage ends in:

- `pending`
- `running`
- `completed`
- `failed`
- `cancelled`

## Performance And Weight Strategy

To reduce product weight:

- do not ship the UVR GUI
- do not ship all models in the installer
- do not install optional backends for release 1 unless required
- split engine responsibilities so future Rust ports can replace Python modules incrementally

To reduce runtime overhead:

- launch Python only when needed
- reuse cached runtime and downloaded models
- avoid loading models before a job starts
- keep one worker process per job in release 1

## Testing Strategy

Rust tests should cover:

- path resolution
- setup state transitions
- model manifest parsing
- download state logic
- engine event parsing
- job state transitions

Python tests should cover:

- command parsing
- manifest and path validation
- runner output contract
- backend adapter behavior for release 1

Integration coverage should include:

1. fresh setup path
2. setup retry after partial failure
3. model download plus checksum verification
4. separation of a short fixture into two output stems
5. cancellation path
6. corrupted model recovery

## Packaging Notes

The repository should eventually include:

- embedded Python payload or unpack recipe
- runner scripts
- dependency lock inputs
- model manifest source

Release 1 should prefer reproducibility over cleverness. A slightly heavier but deterministic setup path is better than a magic flow that is hard to repair.

## Migration Path Beyond Release 1

After release 1, we can incrementally port functionality away from Python:

1. Rust-side registry, downloader, and setup remain authoritative.
2. Python remains only for inference and audio-specific transforms.
3. ONNX-first paths can be ported toward native Rust + ONNX Runtime.
4. Additional job types can be added without breaking the UI contract.

This keeps today’s implementation realistic while preserving a path toward a leaner engine later.
