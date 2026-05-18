# Project Memory: PrismSplit

## Status: v0.1.0 (Development)
- **Current Goal:** Finalize the "Prepare Engine" automated workflow and stabilize the MDX-Net inference bridge.
- **Last Milestone:** Integrated the UVR model registry sync and local directory MD5 scanner.

## Persistent Context
- **Stack:** Tauri (Rust/React), Embedded Python 3.10, ONNX Runtime, PyTorch.
- **Core Architecture:** Orchestrator (Rust) -> Bridge (JSON/stdio) -> Engine (Python).

## Active Tasks
- [ ] Implement the "Prepare Engine" UI stepper for first-run setup.
- [ ] Optimize the SHA-256 verification for large model files.
- [ ] Refine the "Dark Brutalism" UI components (Beveled panels, Progress bars).

## Technical Debt
- Some Python engine backends (e.g., Demucs) require additional VRAM optimizations.
- The `src-tauri` directory structure could be more modular for better service separation.

## Notes
- *2026-05-18:* Jules Dev Standard v1.0 applied. Root documents consolidated in `docs/wiki/`.
- Project is focused on specialized Karaoke separation (vocals/instrumentals).
- Design language: Industrial Audio Skeuomorphism. Avoid modern padding/radius.