# PrismSplit

---

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/built%20with-Tauri-blue)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/powered%20by-Rust-orange)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/engine-Python%203.10-blue)](https://www.python.org/)

**PrismSplit** is a high-performance Windows desktop application designed for specialized audio separation. Primarily focused on **Karaoke Separation** (vocals/instrumentals), it combines the power of the Ultimate Vocal Remover (UVR) inference logic with a modern, lightweight orchestration layer written in Rust.

---

## ⚡ Superpowers

- **Multi-Engine Support:** Full compatibility with **MDX-Net**, **VR Architecture**, **Demucs (v1-v4)**, and **Roformer** models.
- **Smart Catalog Sync:** One-click synchronization with official UVR model servers to access hundreds of pre-trained weights.
- **Zero-Copy Local Scan:** Already have models? Scan your local directories (e.g., `D:/uvr/models`). PrismSplit identifies them via MD5 hash and uses them in-place without duplicating data.
- **Embedded Runtime:** Automatically manages its own isolated Python environment—no more "Dependency Hell".
- **Real-Time Telemetry:** Streamed inference logs and byte-accurate download progress bars.
- **Hardware Accelerated:** Support for NVIDIA (CUDA), AMD/Intel (DirectML), and optimized CPU inference.

---

## 🛠️ Architecture

PrismSplit is split into two specialized layers:

1.  **Orchestration (Rust + Tauri):** Manages the application lifecycle, process supervision, job scheduling, model registry, and high-speed file I/O.
2.  **Inference (Python Engine):** A detached, minimal runner that executes the heavy mathematical lifting using optimized backends like ONNX Runtime and PyTorch.

---

## 🚀 Getting Started (Development)

### Prerequisites
- **Node.js** (v18+)
- **Rust** (Stable)
- **Windows 10/11** (Primary target)

### Installation
1.  **Clone the Repository:**

    ```bash
    git clone https://github.com/julesklord/prismsplit.git
    cd prismsplit
    ```

2.  **Install Frontend Dependencies:**

    ```bash
    npm install
    ```

3.  **Run in Development Mode:**

    ```bash
    npm run tauri dev
    ```

4.  **First Run Setup:**
    Upon launching, PrismSplit will guide you through the **"Prepare Engine"** process, which handles the automated unpacking of the Python runtime and dependency installation.

---

## 🎛️ Usage

1.  **Initialize:** Run the "Prepare Engine" step from the Setup panel.
2.  **Import Models:**
    - Go to **Model Registry**.
    - Click **Sync with UVR Servers** to see available models.
    - Or click **Scan Local Directory** to import your existing UVR collection.
3.  **Extract:**
    - Drag & Drop your audio file into the **Extraction** tab.
    - Select your desired Architecture and Compute Profile (GPU/CPU).
    - Hit **Start Processing** and watch the real-time logs.

---

## 📜 Credits & License

PrismSplit is licensed under the **MIT License**.

This project would not be possible without the incredible research and model development by the **Ultimate Vocal Remover (UVR)** team. PrismSplit acts as a specialized shell and orchestration layer for the inference logic vendored from the UVR project.

- **UVR Project:** [GitHub Repository](https://github.com/Anjok07/uvr)
- **Model Repositories:** Provided by [TRvlvr](https://github.com/TRvlvr)

---

## 🛠️ Roadmap to Alpha
- [x] Rust Orchestration Core
- [x] Full UVR Model Registry Sync
- [x] Local Directory Scanning (MD5 Identification)
- [x] Real-time Progress Events
- [ ] Training Module Bridge
- [ ] Audio Preview Player
- [ ] Advanced Ensemble Builder

---
