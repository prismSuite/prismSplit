# engine/python/prismsplit_protocol.py
def progress_event(job_id: str, message: str, percent: float) -> dict:
    return {
        "event": "progress",
        "job_id": job_id,
        "message": message,
        "percent": percent,
    }
