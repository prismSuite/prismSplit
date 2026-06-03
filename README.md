<div align="center">
  <img src="src/assets/logo.svg" width="600" alt="prismSplit Logo" />
  
  <p><strong>Audio Separation and Stem Isolation Platform</strong></p>
  
  <p>
    <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/Tauri-v2.0-24C8DB?style=flat-square&logo=tauri&logoColor=white" alt="Tauri" /></a>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-Stable-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust" /></a>
    <a href="https://www.python.org/"><img src="https://img.shields.io/badge/Engine-Python%203.10-blue?style=flat-square&logo=python&logoColor=white" alt="Python" /></a>
    <a href="https://onnxruntime.ai/"><img src="https://img.shields.io/badge/Inference-ONNX%20%2F%20PyTorch-475569?style=flat-square&logo=onnx&logoColor=white" alt="Inference" /></a>
  </p>
</div>

---

## Overview

prismSplit separates audio into specialized stems, including vocals, instrumentals, drums, and bass.

As part of the **prismSuite**, the application uses the **MonolithUI** design system. It integrates an isolated Python inference environment with algorithms from the **Ultimate Vocal Remover (UVR)** project.

---

## System Architecture

prismSplit uses two specialized layers:

1.  **Orchestration (Rust + Tauri v2):** Manages the application lifecycle, parallel process supervision, model downloads, and the frontend chasis.
2.  **Inference Engine (Python Runner):** A decoupled subprocess executing separation models via PyTorch and ONNX Runtime. Communication occurs via structured JSON over `stdio`.

```text
  [ Vite / React Frontend ] 
             │
      ( Tauri IPC Commands )
             │
   [ Rust Orchestration Core ]  ◄── (Mutual detection with prismConsole)
             │
   ( Stdio JSON Protocol )
             │
  [ Embedded Python Inference ] ──► [ ONNX Runtime / CUDA GPU ]
```

---

## Features

*   **Multi-Engine Support:** Includes native support for **MDX-Net**, **VR Architecture**, **Demucs (v1-v4)**, and **Roformer** models.
*   **Catalog Synchronization:** Downloads models with SHA-256 verification from UVR servers.
- **Local Model Scanning:** Detects pre-existing models via MD5 hashes to prevent data duplication.
*   **Integrated Environment:** Automates Python engine initialization within a user sandbox.
*   **Hardware Acceleration:** Supports NVIDIA (CUDA), AMD/Intel (DirectML), and multi-core CPUs.
*   **Suite Linking:** Automatically detects **prismConsole** to enable cross-platform navigation.

---

## Installation and Build

### Prerequisites
*   Windows 10 / 11 (Primary development environment)
*   Node.js (v18+)
*   Rust (Stable)

### Steps
1.  **Clone the Repository:**
    ```bash
    git clone https://github.com/julesklord/prismsplit.git
    cd prismsplit
    ```
2.  **Install UI Dependencies:**
    ```bash
    npm install
    ```
3.  **Start Development Environment:**
    ```bash
    npm run dev
    ```
4.  **Build Production Distribution:**
    ```bash
    npm run build
    ```

---
<div align="center">
  <sub>prismSuite — Designed for precision and performance.</sub>
</div>
