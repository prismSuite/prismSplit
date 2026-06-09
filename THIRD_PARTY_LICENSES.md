# Third-Party Software Licenses

PrismSplit incorporates and redistributes code from several third-party projects under their respective licenses. This document contains a summary of these dependencies and their licenses, including the full text of licenses for vendored libraries.

---

## 1. Vendored Libraries

The following software is vendored directly in the `uvr/` directory of the PrismSplit repository.

### 1.1. Ultimate Vocal Remover (UVR) GUI Core & `lib_v5`
* **Path:** `uvr/`
* **License:** MIT License
* **Copyright:** Copyright (c) 2022-2024 Ultimate Vocal Remover
* **Source:** https://github.com/Anjok07/ultimatevocalremovergui

```text
MIT License

Copyright (c) 2022-2024 Ultimate Vocal Remover

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### 1.2. Demucs (Facebook Research / Meta)
* **Path:** `uvr/demucs/`
* **License:** MIT License
* **Copyright:** Copyright (c) Facebook, Inc. and its affiliates.
* **Source:** https://github.com/facebookresearch/demucs

```text
MIT License

Copyright (c) Facebook, Inc. and its affiliates.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## 2. Direct Software Dependencies

The following open-source libraries are used during building or at runtime.

### 2.1. Rust Dependencies (`Cargo.toml`)

| Crate | License | Project URL |
| :--- | :--- | :--- |
| `egui` / `eframe` | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| `tokio` | MIT | https://github.com/tokio-rs/tokio |
| `rodio` | MIT OR Apache-2.0 | https://github.com/RustAudio/rodio |
| `reqwest` | MIT OR Apache-2.0 | https://github.com/seanmonstar/reqwest |
| `serde` / `serde_json` | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `rfd` (Rust File Dialog) | MIT | https://github.com/Polyfrost/rfd |
| `anyhow` | MIT OR Apache-2.0 | https://github.com/dtolnay/anyhow |
| `thiserror` | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `walkdir` | Unlicense OR MIT | https://github.com/BurntSushi/walkdir |
| `sha2` | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| `md-5` | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |

### 2.2. Python Dependencies (`engine/pyproject.toml`)

| Package | License | Project URL |
| :--- | :--- | :--- |
| `torch` (PyTorch) | BSD-3-Clause | https://pytorch.org/ |
| `torchaudio` | BSD-3-Clause | https://github.com/pytorch/audio |
| `onnxruntime` | MIT | https://onnxruntime.ai/ |
| `numpy` | BSD-3-Clause | https://numpy.org/ |
| `librosa` | ISC | https://librosa.org/ |
| `soundfile` | BSD-3-Clause | https://github.com/bastibe/python-soundfile |
| `scipy` | BSD-3-Clause | https://scipy.org/ |
| `PyYAML` | MIT | https://github.com/yaml/pyyaml |
