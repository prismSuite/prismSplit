# engine/python/prismsplit_engine.py
import json
import os
import sys
from pathlib import Path

# Dynamically add the 'uvr' directory to sys.path so vendored packages (like demucs) can be resolved
PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
UVR_DIR = PROJECT_ROOT / "uvr"
if str(UVR_DIR) not in sys.path:
    sys.path.insert(0, str(UVR_DIR))

from prismsplit_backends import get_backend
from prismsplit_protocol import progress_event


def parse_request(raw: str) -> dict:
    return json.loads(raw)


def handle_doctor(payload: dict) -> dict:
    return {
        "event": "result",
        "message": "doctor_ok",
        "payload": {"ping": payload.get("ping", False)},
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
