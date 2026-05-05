# engine/python/tests/test_entrypoint.py
import os
import sys
import unittest

# Add parent dir to path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from prismsplit_backends import get_backend
from prismsplit_engine import parse_request


class TestEntrypoint(unittest.TestCase):
    def test_parse_request_reads_json_command(self):
        raw = '{"command":"doctor","payload":{"ping":true}}'
        request = parse_request(raw)
        self.assertEqual(request["command"], "doctor")
        self.assertTrue(request["payload"]["ping"])

    def test_get_backend_returns_mdx_backend(self):
        backend = get_backend("mdx")
        self.assertEqual(backend.name, "mdx")

    def test_mdx_backend_returns_two_stem_paths(self):
        # We'll mock the internal inference for this test
        # but the backend should still return the expected structure
        import tempfile

        import numpy as np
        import soundfile as sf
        from prismsplit_backends.mdx import MdxBackend

        backend = MdxBackend()
        with tempfile.TemporaryDirectory() as tmp_dir:
            input_wav = os.path.join(tmp_dir, "input.wav")
            # Create a dummy 1s stereo wav
            dummy_data = np.zeros((44100, 2))
            sf.write(input_wav, dummy_data, 44100)

            result = backend.separate(
                {
                    "job_id": "job-1",
                    "input_path": input_wav,
                    "model_path": "model.onnx",
                    "output_dir": tmp_dir,
                }
            )
            self.assertIn("vocals_path", result)
            self.assertIn("instrumental_path", result)
            self.assertTrue(os.path.exists(result["vocals_path"]))
            self.assertTrue(os.path.exists(result["instrumental_path"]))


if __name__ == "__main__":
    unittest.main()
