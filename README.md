<div align="center">
  <img src="assets/icons/prismsplit-logo.svg" width="600" alt="prismSplit Logo" />
  
  <p><strong>Audio Separation and Stem Isolation Platform</strong></p>
  
  <p>
    <a href="https://github.com/emilk/egui"><img src="https://img.shields.io/badge/UI-egui--eframe-blueviolet?style=flat-square" alt="egui" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-Stable-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust" /></a>
    <a href="https://www.python.org/"><img src="https://img.shields.io/badge/Engine-Python%203.9--3.11-blue?style=flat-square&logo=python&logoColor=white" alt="Python" /></a>
    <a href="https://onnxruntime.ai/"><img src="https://img.shields.io/badge/Inference-ONNX%20%2F%20PyTorch-475569?style=flat-square&logo=onnx&logoColor=white" alt="Inference" /></a>
  </p>
</div>

---

## Overview

PrismSplit separates audio into specialized stems, focusing on high-quality karaoke separation (vocals and instrumentals).

Part of the **prismSuite**, the application provides a native, GPU-rendered interface using `egui` and `eframe`. It uses the **MonolithUI** (Industrial Audio Skeuomorphism and Dark Brutalism) design language. The app runs an isolated, self-repairing Python engine with inference logic from the **Ultimate Vocal Remover (UVR)** project.

---

## System Architecture

PrismSplit divides operations into two distinct layers:

1.  **Chassis & Orchestration (Rust + egui / eframe):** Renders the interface directly using OS graphics drivers. It coordinates application state, downloads models, and supervises the Python subprocess.
2.  **Inference Engine (Python Subprocess):** Runs the audio separation models. Rust and Python communicate via newline-delimited JSON messages over standard input and output.

```text
   [ egui Native UI Chassis ] 
              ▲
              │ (Rust channels / AppMsg)
              ▼
   [ Rust Orchestration Core ]
              │
              │ (Stdio JSON Protocol)
              ▼
  [ Embedded Python Inference ] ──► [ ONNX Runtime / PyTorch ]
                                    ├── CUDA GPU (Windows/Linux)
                                    └── CoreML / MPS (macOS)
```

---

## Features

*   **Lightweight:** Builds to a native binary of 5 to 8 megabytes, bypassing WebView engine requirements.
*   **Skeuomorphic GUI:** Employs a Tahoma font, beveled panels, and blocky status meters.
*   **Separation engines:** Executes ONNX models via MDX-Net and PyTorch models via Demucs.
*   **Registry sync:** Downloads models from UVR servers and verifies integrity using SHA-256.
*   **Local scans:** Recognizes local model files by their MD5 hashes to prevent duplication.
*   **Self-repairing runtime:** Sets up a Python virtual environment and repairs broken modules (like numpy or onnxruntime) in-place.
*   **Hardware acceleration:** Supports NVIDIA CUDA on Windows and Linux, and Apple Silicon MPS/CoreML on macOS.

---

## Installation and Build

### Prerequisites
*   **Rust:** Stable toolchain (edition 2021).
*   **Python:** 3.9, 3.10, or 3.11 (needed locally for development).

### Steps
1.  **Clone the Repository:**
    ```bash
    git clone https://github.com/julesklord/prismsplit.git
    cd prismsplit
    ```
2.  **Set Up Local Dev Python Path (Linux/macOS):**
    If your system's default `python3` points to an unsupported version (e.g. 3.14), configure a compatible version (like 3.10) in your `.env` file:
    ```env
    PRISMSPLIT_DEV_PYTHON=/usr/bin/python3.10
    ```
3.  **Start App (Development):**
    ```bash
    cargo run
    ```
4.  **Run Tests:**
    ```bash
    cargo test
    ```
5.  **Compile Production Release Binary:**
    ```bash
    cargo build --release
    ```

---

## Packaging and CI/CD

GitHub Actions compiles and packages releases automatically on tag pushes (e.g. `v0.1.0`):
*   **Windows:** Output includes `prismsplit_installer_windows_[version].exe` (NSIS) and `prismsplit_portable_windows_[version].zip`.
*   **Linux:** Output includes `prismsplit_installer_linux_[version].deb` (Debian package) and `prismsplit_portable_linux_[version].tar.gz`.

---
<div align="center">
  <sub>prismSuite: Designed for precision and performance.</sub>
</div>
