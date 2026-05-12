# engine/python/prismsplit_backends/demucs.py
import json
import os
from typing import Tuple

import librosa
import numpy as np
import soundfile as sf
import torch
from demucs.pretrained import get_model
from prismsplit_protocol import progress_event

from prismsplit_backends.base import BackendBase
from prismsplit_backends.model_request import resolve_demucs_model_name


class DemucsBackend(BackendBase):
    name = "demucs"

    def run_inference(
        self, audio: np.ndarray, model_name: str
    ) -> Tuple[np.ndarray, np.ndarray]:
        """
        Separate audio using Demucs.
        Returns (vocals, instrumental).
        """
        device = "cuda" if torch.cuda.is_available() else "cpu"
        # In UVR context, we might load a specific model path, but standard Demucs
        # often uses name-based fetching. If model_path is a directory or weights file,
        # get_model might need adaptation, but we stick to the plan's specification.
        model = get_model(model_name).to(device)
        model.eval()

        # Ensure stereo tensor
        if audio.ndim == 1:
            audio = np.stack([audio, audio])
        elif audio.shape[0] != 2:
            audio = audio.T

        audio_tensor = torch.from_numpy(audio).float().to(device)

        with torch.no_grad():
            # Add batch dimension
            sources = model(audio_tensor.unsqueeze(0))

        # Demucs htdemucs returns [drums, bass, other, vocals]
        # Combine drums + bass + other = instrumental
        vocals = sources[0, 3].cpu().numpy()
        instrumental = (sources[0, 0] + sources[0, 1] + sources[0, 2]).cpu().numpy()

        return vocals, instrumental

    def separate(self, request: dict) -> dict:
        job_id = str(request.get("job_id", "unknown"))
        input_path = str(request.get("input_path", ""))
        model_path = str(request.get("model_path", ""))
        output_dir = str(request.get("output_dir", ""))

        if not input_path or not os.path.exists(input_path):
            raise ValueError(f"Input file not found: {input_path}")

        model_name = resolve_demucs_model_name(request)

        # 1. Load Audio
        print(json.dumps(progress_event(job_id, "Loading audio", 20.0)))
        audio, sr = librosa.load(input_path, sr=44100, mono=False)

        # 2. Process
        print(json.dumps(progress_event(job_id, "Running Demucs inference", 50.0)))
        vocals, instrumental = self.run_inference(audio, model_name)

        # 3. Save Outputs
        print(json.dumps(progress_event(job_id, "Saving stems", 80.0)))
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
