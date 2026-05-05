# engine/python/prismsplit_engine.py
import json


def parse_request(raw: str) -> dict:
    return json.loads(raw)


def handle_doctor(payload: dict) -> dict:
    return {
        "event": "result",
        "message": "doctor_ok",
        "payload": {"ping": payload.get("ping", False)},
    }
