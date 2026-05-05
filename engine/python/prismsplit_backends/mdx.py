# engine/python/prismsplit_backends/mdx.py
from prismsplit_backends.base import BackendBase


class MdxBackend(BackendBase):
    name = "mdx"

    def separate(self, request: dict) -> dict:
        return {"vocals_path": "vocals.wav", "instrumental_path": "instrumental.wav"}
