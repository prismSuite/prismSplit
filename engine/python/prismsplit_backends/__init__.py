# engine/python/prismsplit_backends/__init__.py
from prismsplit_backends.demucs import DemucsBackend
from prismsplit_backends.mdx import MdxBackend


def get_backend(name: str):
    if name == "mdx":
        return MdxBackend()
    elif name == "demucs":
        return DemucsBackend()
    raise ValueError(f"Unknown backend: {name}")
