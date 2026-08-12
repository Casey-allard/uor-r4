import json
import subprocess
import sys
import unittest


def extract_summary(stdout: str):
    for line in stdout.splitlines():
        if line.startswith("__JSON_SUMMARY__"):
            return json.loads(line[len("__JSON_SUMMARY__"):].strip())
    raise AssertionError("summary line not found")


class TestDeterminism(unittest.TestCase):
    def test_same_seed_stable_metrics(self):
        cmd = [
            sys.executable,
            "hyperbolic_router_so8.py",
            "--mode", "anis",
            "--seed", "7",
            "--N", "500",
            "--K", "6",
            "--d", "8",
            "--dy", "16",
            "--depth", "3",
            "--branching", "2",
            "--learn_so8", "0",
            "--learn_scale", "1",
            "--scale_mode", "radial",
            "--radial_bins", "6",
            "--chart_iters", "20",
            "--so8_candidates", "4",
            "--scale_candidates", "4",
            "--split_rounds", "20",
            "--extra_budget", "16",
            "--fast_dev", "1",
            "--cache_chart", "0",
            "--cache_routes", "0",
        ]
        p1 = subprocess.run(cmd, capture_output=True, text=True, check=True)
        p2 = subprocess.run(cmd, capture_output=True, text=True, check=True)
        s1 = extract_summary(p1.stdout)
        s2 = extract_summary(p2.stdout)

        self.assertAlmostEqual(
            s1["metrics"]["test_mse_after"],
            s2["metrics"]["test_mse_after"],
            places=9,
        )


if __name__ == "__main__":
    unittest.main()
