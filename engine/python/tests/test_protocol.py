# engine/python/tests/test_protocol.py
import os
import sys
import unittest

# Add parent dir to path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from prismsplit_protocol import progress_event


class TestProtocol(unittest.TestCase):
    def test_progress_event_contains_expected_shape(self):
        payload = progress_event(job_id="job-1", message="Loading model", percent=25.0)
        self.assertEqual(payload["event"], "progress")
        self.assertEqual(payload["job_id"], "job-1")
        self.assertEqual(payload["percent"], 25.0)


if __name__ == "__main__":
    unittest.main()
