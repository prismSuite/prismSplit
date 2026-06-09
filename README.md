<div align="center">
  <img src="assets/icons/prismsplit-logo.svg" width="250" alt="prismSplit Logo" />

  <p><strong>High-quality AI audio source separation</strong></p>

  <p>
    <a href="https://github.com/emilk/egui"><img src="https://img.shields.io/badge/UI-egui--eframe-blueviolet?style=flat-square" alt="egui" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-Stable-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust" /></a>
    <a href="https://www.python.org/"><img src="https://img.shields.io/badge/Engine-Python%203.9--3.11-blue?style=flat-square&logo=python&logoColor=white" alt="Python" /></a>
    <a href="https://onnxruntime.ai/"><img src="https://img.shields.io/badge/Inference-ONNX%20%2F%20PyTorch-475569?style=flat-square&logo=onnx&logoColor=white" alt="Inference" /></a>
  </p>
</div>

---

## What is PrismSplit?

PrismSplit is a desktop application for separating audio tracks into individual stems (vocals, drums, bass, etc.). It is part of the **prismSuite**.

The app provides a native, GPU-rendered interface using `egui` and `eframe`, following the **MonolithUI** design language (industrial audio skeuomorphism and dark brutalism). It runs an isolated Python inference engine based on the **Ultimate Vocal Remover (UVR)** project.

---

## Architecture

PrismSplit splits its work between two layers:

1.  **UI & Orchestration (Rust + egui / eframe):** Renders the interface using the OS graphics stack, manages application state, downloads models, and manages the Python subprocess.
2.  **Inference Engine (Python subprocess):** Runs the audio separation models. Rust and Python communicate via newline-delimited JSON messages over standard input and output.

```text
   [ egui Native UI ]
              ▲
              │ (Rust channels / AppMsg)
              ▼
   [ Rust Orchestration Core ]
              │
              │ (Stdio JSON Protocol)
              ▼
  [ Python Inference Engine ] ──► [ ONNX Runtime / PyTorch ]
                                    ├── CUDA GPU (Windows/Linux)
                                    └── CoreML / MPS (macOS)
```

---

## Features

*   **Lightweight native binary** (~5–8 MB) — no WebView or browser engine needed.
*   **Skeuomorphic GUI** — MonolithUI with beveled panels, Tahoma typeface, and hardware-style status meters.
*   **Multiple separation engines** — ONNX models via MDX-Net and PyTorch models via Demucs.
*   **Model registry sync** — Downloads models from UVR servers with SHA-256 integrity checks.
*   **Local model scan** — Detects local model files by MD5 to avoid duplicates.
*   **Self-repairing runtime** — Automatically sets up a Python virtual environment and fixes broken packages (e.g. `numpy`, `onnxruntime`).
*   **Hardware acceleration** — NVIDIA CUDA on Windows/Linux; Apple Silicon MPS / CoreML on macOS.
*   **Format support** — WAV, MP3, FLAC, M4A, and more via `ffmpeg`.

---

## Installation and Build

### Prerequisites

**To compile:**
*   **Rust** — stable toolchain (edition 2021).

**To run the inference engine:**
*   **Python** — 3.9 – 3.11 (verified; minimum `>=3.9`).
*   **Hardware Acceleration (GPU):**
    *   **NVIDIA CUDA** (Windows / Linux) — CUDA Toolkit 11.8 or 12.x with compatible drivers.
    *   **Apple Silicon** (macOS) — MPS and CoreML are supported out of the box.
    *   **CPU fallback** — always available if no GPU is detected.
*   **ffmpeg** — must be on your `PATH` to decode and encode non-WAV formats.

### Build from source

1.  **Clone the repo**
    ```bash
    git clone https://github.com/julesklord/prismsplit.git
    cd prismsplit
    ```

2.  **Configure Python version (Linux / macOS)**
    If your system `python3` is newer than 3.11, point to a compatible version via `.env`:
    ```env
    PRISMSPLIT_DEV_PYTHON=/usr/bin/python3.10
    ```

3.  **Run in development mode**
    ```bash
    cargo run
    ```

4.  **Run tests**
    ```bash
    cargo test
    ```

5.  **Build release binary**
    ```bash
    cargo build --release
    ```

---

## Packaging and CI/CD

GitHub Actions automatically builds and packages releases on each version tag (e.g. `v0.1.0`):

*   **Windows** — `prismsplit_installer_windows_[version].exe` (NSIS) and `prismsplit_portable_windows_[version].zip`
*   **Linux** — `prismsplit_installer_linux_[version].deb` (Debian package) and `prismsplit_portable_linux_[version].tar.gz`

---

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

Third-party components are listed in [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md).

---
<div align="center">
  <sub>prismSuite — Designed for precision and performance.</sub>
</div>
