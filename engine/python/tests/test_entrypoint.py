# engine/python/tests/test_entrypoint.py
import os
import sys
import tempfile
import unittest
from unittest import mock

# Add parent dir to path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from prismsplit_backends import get_backend
from prismsplit_backends.model_request import resolve_demucs_model_name
from prismsplit_engine import handle_doctor, handle_list_models, parse_request


class TestEntrypoint(unittest.TestCase):
    def test_parse_request_reads_json_command(self):
        raw = '{"command":"doctor","payload":{"ping":true}}'
        request = parse_request(raw)
        self.assertEqual(request["command"], "doctor")
        self.assertTrue(request["payload"]["ping"])

    def test_doctor_reports_dependency_flags(self):
        result = handle_doctor({})
        self.assertEqual(result["event"], "result")
        self.assertIn("payload", result)
        self.assertIn("python_version", result["payload"])
        self.assertIn("backend_imports", result["payload"])
        self.assertIn("ready", result["payload"])

    def test_get_backend_rejects_unknown_backend(self):
        with self.assertRaises(ValueError):
            get_backend("unknown")

    def test_list_models_marks_installed_entries(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            catalog_path = os.path.join(tmp_dir, "catalog.json")
            models_dir = os.path.join(tmp_dir, "models")
            os.makedirs(models_dir, exist_ok=True)
            model_path = os.path.join(models_dir, "karaoke.onnx")
            with open(model_path, "wb") as handle:
                handle.write(b"dummy")

            with open(catalog_path, "w", encoding="utf-8") as handle:
                handle.write(
                    """
                    [
                      {
                        "id": "mdx_karaoke",
                        "name": "MDX Karaoke",
                        "backend": "mdx",
                        "outputKind": "vocals_instrumental",
                        "url": "https://example.com/karaoke.onnx",
                        "sha256": "replace-with-real-sha256",
                        "sizeBytes": 10,
                        "filename": "karaoke.onnx",
                        "version": "1.0.0"
                      }
                    ]
                    """
                )

            result = handle_list_models(
                {
                    "catalog_path": catalog_path,
                    "models_dir": models_dir,
                }
            )

        self.assertEqual(result["event"], "result")
        self.assertEqual(len(result["payload"]["models"]), 1)
        self.assertTrue(result["payload"]["models"][0]["is_installed"])

    def test_list_models_prefers_local_path_for_installed_state(self):
        with tempfile.TemporaryDirectory() as tmp_dir:
            local_model = os.path.join(tmp_dir, "local-model.onnx")
            with open(local_model, "wb") as handle:
                handle.write(b"dummy")

            catalog_path = os.path.join(tmp_dir, "catalog.json")
            local_model_path = str(local_model).replace("\\", "\\\\")
            with open(catalog_path, "w", encoding="utf-8") as handle:
                handle.write(
                    f"""
                    [
                      {{
                        "id": "local_model",
                        "name": "Local Model",
                        "backend": "mdx",
                        "outputKind": "vocals_instrumental",
                        "url": "",
                        "sha256": "",
                        "sizeBytes": 10,
                        "filename": "ignored.onnx",
                        "version": "local",
                        "localPath": "{local_model_path}"
                      }}
                    ]
                    """
                )

            result = handle_list_models({"catalog_path": catalog_path})

        self.assertTrue(result["payload"]["models"][0]["is_installed"])

    def test_demucs_model_resolution_prefers_explicit_name(self):
        request = {
            "model_name": "htdemucs_ft",
            "model_path": os.path.join("models", "htdemucs.th"),
        }
        self.assertEqual(resolve_demucs_model_name(request), "htdemucs_ft")

    def test_demucs_model_resolution_falls_back_to_model_path_name(self):
        request = {"model_path": os.path.join("models", "htdemucs_6s.th")}
        self.assertEqual(resolve_demucs_model_name(request), "htdemucs_6s")

    def test_demucs_model_resolution_defaults_to_htdemucs(self):
        self.assertEqual(resolve_demucs_model_name({}), "htdemucs")


if __name__ == "__main__":
    unittest.main()
