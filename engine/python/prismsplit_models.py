# engine/python/prismsplit_models.py
import os


def validate_model_path(path: str) -> bool:
    return os.path.exists(path) and (path.endswith(".onnx") or path.endswith(".pth"))
