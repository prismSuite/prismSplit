# engine/python/prismsplit_backends/mdx.py
import json
import os

# Add uvr to path for STFT
import sys
from pathlib import Path
from typing import Tuple

import librosa
import numpy as np
import onnxruntime as ort
import soundfile as sf
import torch
from prismsplit_protocol import progress_event

from prismsplit_backends.base import BackendBase

# Try to use the shared helper to find/insert the vendored `uvr` directory
try:
    from uvr_utils import ensure_uvr_in_sys_path
except Exception:
    ensure_uvr_in_sys_path = None  # type: ignore

UVR_DIR = None
if ensure_uvr_in_sys_path:
    try:
        UVR_DIR = ensure_uvr_in_sys_path()
    except Exception:
        UVR_DIR = None

if UVR_DIR is None:
    # Fallback to the repo-relative guess (preserve previous behavior)
    PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent.parent
    UVR_DIR = PROJECT_ROOT / "uvr"
    if str(UVR_DIR) not in sys.path:
        sys.path.insert(0, str(UVR_DIR))

from lib_v5.tfc_tdf_v3 import STFT


class MdxBackend(BackendBase):
    name = "mdx"

    def run_inference(
        self, audio: np.ndarray, model_path: str, job_id: str
    ) -> Tuple[np.ndarray, np.ndarray]:
        """
        Separate audio into vocals and instrumental using MDX-Net ONNX model.
        """
        device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

        # Ensure stereo
        if audio.ndim == 1:
            audio = np.stack([audio, audio])
        elif audio.shape[0] != 2:
            audio = audio.T

        # MDX-Net Parameters
        # Most MDX-Net models use these defaults
        n_fft = 4096
        hop_length = 1024
        dim_f = 2048
        dim_t = 256
        overlap = 0.25

        # Initialize STFT
        stft = STFT(n_fft, hop_length, dim_f, device)

        # Load ONNX session
        providers = ["CUDAExecutionProvider", "CPUExecutionProvider"]
        if device.type == "cpu":
            providers = ["CPUExecutionProvider"]

        session = ort.InferenceSession(model_path, providers=providers)
        input_name = session.get_inputs()[0].name

        # Audio processing
        # MDX-Net expects chunks of size hop * (dim_t - 1)
        chunk_size = hop_length * (dim_t - 1)
        margin = n_fft // 2
        step = int(chunk_size * (1 - overlap))

        # Padding
        total_samples = audio.shape[1]
        pad_size = chunk_size - (total_samples % chunk_size)
        mixture = np.concatenate(
            [np.zeros((2, margin)), audio, np.zeros((2, pad_size + margin))], axis=1
        )

        result = np.zeros_like(mixture)
        divider = np.zeros_like(mixture)

        total_chunks = (mixture.shape[1] - chunk_size) // step + 1

        for i in range(0, mixture.shape[1] - chunk_size + 1, step):
            start = i
            end = i + chunk_size

            chunk = mixture[:, start:end]
            chunk_tensor = torch.tensor([chunk], dtype=torch.float32).to(device)

            # STFT
            spek = stft(chunk_tensor)

            # Inference
            # spek shape: [1, 4, 2048, 256]
            spec_pred = session.run(None, {input_name: spek.cpu().numpy()})[0]

            # Inverse STFT
            out_chunk = (
                stft.inverse(torch.tensor(spec_pred).to(device))
                .cpu()
                .detach()
                .numpy()[0]
            )

            # Windowing for overlap-add
            window = np.hanning(chunk_size)
            window = np.tile(window, (2, 1))

            result[:, start:end] += out_chunk * window
            divider[:, start:end] += window

            # Progress (30% to 80% range)
            progress = 30.0 + (50.0 * (i / (mixture.shape[1] - chunk_size)))
            if i % (step * 5) == 0:
                print(json.dumps(progress_event(job_id, "Processing chunks", progress)))

        # Finalize vocals
        vocals = (result / (divider + 1e-10))[:, margin : margin + total_samples]

        # Instrumental is mix - vocals
        instrumental = audio - vocals

        return vocals, instrumental

    def separate(self, request: dict) -> dict:
        job_id = str(request.get("job_id", "unknown"))
        input_path = str(request.get("input_path", ""))
        model_path = str(request.get("model_path", ""))
        output_dir = str(request.get("output_dir", ""))

        if not input_path or not os.path.exists(input_path):
            raise ValueError(f"Input file not found: {input_path}")

        if not model_path or not os.path.exists(model_path):
            raise ValueError(f"Model file not found: {model_path}")

        # 1. Load Audio
        print(json.dumps(progress_event(job_id, "Loading audio", 20.0)))
        audio, sr = librosa.load(input_path, sr=44100, mono=False)

        # 2. Process
        print(json.dumps(progress_event(job_id, "Running MDX-Net inference", 30.0)))
        vocals, instrumental = self.run_inference(audio, model_path, job_id)

        # 3. Save Outputs
        print(json.dumps(progress_event(job_id, "Saving stems", 90.0)))
        base_name = os.path.splitext(os.path.basename(input_path))[0]

        vocals_path = os.path.join(output_dir, f"{base_name}_(Vocals).wav")
        instrumental_path = os.path.join(output_dir, f"{base_name}_(Instrumental).wav")

        sf.write(vocals_path, vocals.T, sr)
        sf.write(instrumental_path, instrumental.T, sr)

        print(json.dumps(progress_event(job_id, "Complete", 100.0)))

        return {
            "vocals_path": vocals_path,
            "instrumental_path": instrumental_path,
        }
