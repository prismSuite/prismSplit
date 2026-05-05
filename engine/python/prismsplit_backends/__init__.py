# engine/python/prismsplit_backends/__init__.py
from prismsplit_backends.mdx import MdxBackend


def get_backend(name: str):
    if name == "mdx":
        return MdxBackend()
    raise ValueError(f"Unknown backend: {name}")
