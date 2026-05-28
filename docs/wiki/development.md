# Development Guide

## Prerequisites
- **Node.js:** v18+
- **Rust:** Stable toolchain.
- **Python:** 3.10 (App handles embedded distribution, but local install is useful for testing).
- **Windows:** Primary development and target platform.

## Setup
1. Clone the repository.
2. Install frontend dependencies: `npm install`.

## Commands
- **Run App (Dev):** `npm run dev` (Vite + Tauri window).
- **Build App:** `npm run build` (Production installer).
- **Clean Dist:** `npm run clean`.
- **Rust Tests:** `cd src-tauri && cargo test`.
- **Python Engine Tests:** `pytest engine/python/tests/`.
- **Linting:** `npm run lint`.

## Engine Preparation
On first run, use the **Setup** panel to "Prepare Engine". This will:
1. Unpack the embedded Python runtime.
2. Create the virtual environment.
3. Install dependencies from `engine/pyproject.toml`.

## Repository Structure
- `src/`: React frontend.
- `src-tauri/`: Rust orchestration logic.
- `engine/`: Python inference runner.
- `docs/`: Jules Dev Standard documentation.