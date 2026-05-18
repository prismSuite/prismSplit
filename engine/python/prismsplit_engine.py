# engine/python/prismsplit_engine.py
import json
import os
import sys
from pathlib import Path
from typing import Any, Optional

# Robustly add the vendored 'uvr' directory (which contains `lib_v5`) to sys.path
# so imports like `import lib_v5` work both in-source and in bundled installers.
try:
    from uvr_utils import ensure_uvr_in_sys_path
except Exception:
    ensure_uvr_in_sys_path = None  # type: ignore

UVR_DIR: Optional[Path] = None
if ensure_uvr_in_sys_path:
    try:
        UVR_DIR = ensure_uvr_in_sys_path()
    except Exception:
        UVR_DIR = None

if UVR_DIR is None:
    PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
    UVR_DIR = PROJECT_ROOT / "uvr"
    if str(UVR_DIR) not in sys.path:
        sys.path.insert(0, str(UVR_DIR))

from prismsplit_backends import get_backend
from prismsplit_protocol import progress_event


def parse_request(raw: str) -> dict:
    return json.loads(raw)


def _import_status(module_name: str) -> dict[str, Any]:
    try:
        __import__(module_name)
        return {"ok": True}
    except Exception as exc:  # pragma: no cover - exercised through doctor payload
        return {"ok": False, "error": f"{type(exc).__name__}: {exc}"}


def _detect_backend_imports() -> dict[str, dict[str, Any]]:
    return {
        "numpy": _import_status("numpy"),
        "soundfile": _import_status("soundfile"),
        "librosa": _import_status("librosa"),
        "onnxruntime": _import_status("onnxruntime"),
        "torch": _import_status("torch"),
    }


def handle_doctor(payload: dict) -> dict:
    backend_imports = _detect_backend_imports()
    ready = all(result["ok"] for result in backend_imports.values())
    return {
        "event": "result",
        "message": "doctor_ok",
        "payload": {
            "ping": payload.get("ping", False),
            "python_version": sys.version.split()[0],
            "uvr_path": str(UVR_DIR),
            "backend_imports": backend_imports,
            "ready": ready,
        },
    }


def _read_catalog(catalog_path: str) -> list[dict[str, Any]]:
    with open(catalog_path, "r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, list):
        raise ValueError("Catalog JSON must be a list of model entries")
    return data


def _is_installed(entry: dict[str, Any], models_dir: str | None) -> bool:
    local_path = entry.get("localPath") or entry.get("local_path")
    if isinstance(local_path, str) and local_path:
        return os.path.isfile(local_path)

    filename = entry.get("filename")
    if not models_dir or not isinstance(filename, str) or not filename:
        return False

    return os.path.isfile(os.path.join(models_dir, filename))


def handle_list_models(payload: dict) -> dict:
    catalog_path = payload.get("catalog_path")
    if not isinstance(catalog_path, str) or not catalog_path:
        raise ValueError("catalog_path is required")

    models_dir = payload.get("models_dir")
    models = _read_catalog(catalog_path)
    normalized = []
    for entry in models:
        item = dict(entry)
        item["is_installed"] = _is_installed(item, models_dir)
        normalized.append(item)

    return {
        "event": "result",
        "message": "models_loaded",
        "payload": {"models": normalized},
    }


def handle_separate(payload: dict) -> dict:
    job_id = payload.get("job_id", "unknown")
    backend_name = payload.get("backend", "mdx")

    print(json.dumps(progress_event(job_id, "Initializing backend", 10.0)))

    try:
        backend = get_backend(backend_name)
        result = backend.separate(payload)
        return {
            "event": "result",
            "job_id": job_id,
            "message": "success",
            "payload": result,
        }
    except Exception as e:
        return {
            "event": "error",
            "job_id": job_id,
            "message": str(e),
        }


def main():
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            request = parse_request(line)
            command = request.get("command")
            payload = request.get("payload", {})

            if command == "doctor":
                response = handle_doctor(payload)
            elif command == "list_models":
                response = handle_list_models(payload)
            elif command == "separate":
                response = handle_separate(payload)
            else:
                response = {"event": "error", "message": f"Unknown command: {command}"}

            print(json.dumps(response))
            sys.stdout.flush()
        except Exception as e:
            print(json.dumps({"event": "error", "message": f"Parse error: {str(e)}"}))
            sys.stdout.flush()


if __name__ == "__main__":
    main()
