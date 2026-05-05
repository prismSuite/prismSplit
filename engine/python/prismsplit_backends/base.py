# engine/python/prismsplit_backends/base.py
class BackendBase:
    name = "base"

    def separate(self, request: dict) -> dict:
        raise NotImplementedError
