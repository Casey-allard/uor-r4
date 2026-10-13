import json
import os
import subprocess
import sys
import tempfile
import unittest


class TestParseLogs(unittest.TestCase):
    def test_good_and_bad_logs(self):
        with tempfile.TemporaryDirectory() as td:
            raw = os.path.join(td, "raw")
            parsed = os.path.join(td, "parsed")
            os.makedirs(raw, exist_ok=True)

            good = {
                "schema_version": "1.0",
                "parsed": True,
                "args": {"seed": 0},
                "metrics": {
                    "test_mse_before": 1.0,
                    "test_mse_after": 0.9,
                    "train_label_sse_per": 1.0,
                    "test_label_sse_per": 1.0,
                    "buckets": 5,
                    "slots_used": 8,
                    "test_unseen_rate": 0.0,
                },
                "timings_sec": {
                    "dataset": 0.1,
                    "chart_opt": 0.2,
                    "routing_eval": 0.1,
                    "training_ema": 0.1,
                    "growth": 0.1,
                    "total": 0.6,
                },
                "artifacts": {},
                "git": {},
                "notes": [],
            }

            with open(os.path.join(raw, "good.log"), "w", encoding="utf-8") as f:
                f.write("hello\n")
                f.write("__JSON_SUMMARY__ " + json.dumps(good) + "\n")

            with open(os.path.join(raw, "bad.log"), "w", encoding="utf-8") as f:
                f.write("no summary\n")

            subprocess.run(
                [sys.executable, "tools/parse_logs.py", raw, parsed],
                check=True,
            )

            with open(os.path.join(parsed, "good.json"), "r", encoding="utf-8") as f:
                gj = json.load(f)
            with open(os.path.join(parsed, "bad.json"), "r", encoding="utf-8") as f:
                bj = json.load(f)

            self.assertTrue(gj["parsed"])
            self.assertFalse(bj["parsed"])
            self.assertIn("parse_error", bj)


if __name__ == "__main__":
    unittest.main()
