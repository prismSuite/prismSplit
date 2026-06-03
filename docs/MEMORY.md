# Project Memory: PrismSplit

## Status: v0.1.0 (Development)
- **Current Goal:** Stabilize the native egui interface and finalize integration of UVR inference engine.
- **Last Milestone:** Successfully migrated the codebase from Tauri/React to pure Rust + egui/eframe. Validated compilation and backend tests.

## Persistent Context
- **Stack:** Rust (egui/eframe), Embedded Python 3.9 - 3.11, ONNX Runtime, PyTorch.
- **Core Architecture:** GUI App & Orchestrator (Rust) -> Bridge (JSON/stdio) -> Engine (Python).

## Active Tasks
- [x] Migrate frontend to native immediate-mode egui GUI.
- [x] Implement live waveform previsualizer with `rodio` playback controls.
- [ ] Update obsolete project documentation references (Tauri/React references in wiki).
- [ ] Rename `analyze_wav_peaks` to reflect general audio support.

## Technical Debt
- Some Python engine backends (e.g., Demucs) require additional VRAM optimizations.
- Outdated documentation files containing Tauri/React references need ongoing tracking/maintenance.

## Notes
- *2026-06-03:* Completed full migration to egui. `cargo test` passes 15 tests successfully.
- *2026-05-18:* Jules Dev Standard v1.0 applied. Root documents consolidated in `docs/wiki/`.
- Project is focused on specialized Karaoke separation (vocals/instrumentals).
- Design language: Industrial Audio Skeuomorphism. Avoid modern padding/radius.