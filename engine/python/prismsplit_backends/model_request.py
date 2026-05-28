import os


def resolve_demucs_model_name(request: dict) -> str:
    requested_name = str(request.get("model_name", "")).strip()
    if requested_name:
        return requested_name

    model_path = str(request.get("model_path", "")).strip()
    if model_path:
        return os.path.splitext(os.path.basename(model_path))[0]

    return "htdemucs"
