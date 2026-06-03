# Development Guide

## Prerequisites
- **Rust:** Stable toolchain.
- **Python:** 3.9 - 3.11 (App handles embedded distribution, but a local compatible installation is useful for dev/testing).
- **Windows:** Primary development and target platform.

## Setup
1. Clone the repository.
2. Set up the development Python path environment variable `PRISMSPLIT_DEV_PYTHON` pointing to your local python executable if needed (e.g. in `.env`).

## Commands
- **Run App (Dev):** `cargo run` (Builds and runs the egui window in debug mode).
- **Build App:** `cargo build --release` (Compiles the production-ready standalone binary).
- **Clean Build:** `cargo clean`.
- **Rust Tests:** `cargo test` (Runs all Rust backend and UI unit/integration tests).
- **Python Engine Tests:** `pytest engine/python/tests/` (Requires a configured Python environment).
- **Rust Linting:** `cargo clippy`.

## Engine Preparation
On first run, the app will show the setup panel. Click "Prepare Engine" to:
1. Verify directories.
2. Unpack the embedded Python runtime.
3. Create the virtual environment.
4. Install dependencies from `engine/pyproject.toml`.

## Repository Structure
- `src/`: Rust source code, containing the GUI entry point (`main.rs`), application layout (`app.rs`), state (`state.rs`), panels (`panels/`), widgets (`widgets.rs`), custom design system (`theme.rs`), and core backend services.
- `engine/`: Python inference runner.
- `uvr/`: Vendored source of truth for inference logic (used by the engine).
- `docs/`: Jules Dev Standard documentation.