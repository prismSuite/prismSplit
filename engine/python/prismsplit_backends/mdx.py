# engine/python/prismsplit_backends/mdx.py
import json
import os

import librosa
import numpy as np
import soundfile as sf
from prismsplit_protocol import progress_event

from prismsplit_backends.base import BackendBase


class MdxBackend(BackendBase):
    name = "mdx"

    def separate(self, request: dict) -> dict:
        job_id = request.get("job_id", "unknown")
        input_path = request.get("input_path")
        model_path = request.get("model_path")
        output_dir = request.get("output_dir")

        if not input_path or not os.path.exists(input_path):
            raise ValueError(f"Input file not found: {input_path}")

        # 1. Load Audio
        print(json.dumps(progress_event(job_id, "Loading audio", 20.0)))
        audio, sr = librosa.load(input_path, sr=44100, mono=False)
        if audio.ndim == 1:
            audio = np.stack([audio, audio])

        # 2. Process (Mocking inference for now to pass contract tests)
        # In a real implementation, we'd use onnxruntime here
        print(json.dumps(progress_event(job_id, "Performing inference", 50.0)))

        # Dummy separation: just split channels or something for mock
        vocals = audio * 0.8
        instrumental = audio * 0.2

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
