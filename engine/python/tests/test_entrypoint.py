# engine/python/tests/test_entrypoint.py
import os
import sys
import unittest

# Add parent dir to path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from prismsplit_backends import get_backend
from prismsplit_engine import parse_request


class TestEntrypoint(unittest.TestCase):
    def test_parse_request_reads_json_command(self):
        raw = '{"command":"doctor","payload":{"ping":true}}'
        request = parse_request(raw)
        self.assertEqual(request["command"], "doctor")
        self.assertTrue(request["payload"]["ping"])

    def test_get_backend_returns_mdx_backend(self):
        backend = get_backend("mdx")
        self.assertEqual(backend.name, "mdx")


if __name__ == "__main__":
    unittest.main()
