# engine/python/prismsplit_backends/__init__.py
def get_backend(name: str):
    if name == "mdx":
        from prismsplit_backends.mdx import MdxBackend

        return MdxBackend()
    if name == "demucs":
        from prismsplit_backends.demucs import DemucsBackend

        return DemucsBackend()
    raise ValueError(f"Unknown backend: {name}")
