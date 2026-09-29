#!/usr/bin/env python3
"""Synthetic workload test harness for scripts/energy_per_token.py.

Verifies:
  1. Pipe draining with high-volume output (>64 KiB, exceeding Darwin OS pipe buffer).
  2. Background drain thread avoids deadlocks/stalls.
  3. JSONL parsing and graceful handling of trailing SIGINT fragments.
  4. Monotonic timestamp verification and negative-span rejection.
  5. Out-of-interval timestamp and insufficient coverage ratio (<85%) rejection.
  6. Excessive inter-sample gap (>2.0s) stall detection.
  7. Collector failure and malformed stream error propagation.
  8. End-to-end execution of energy_per_token.py with a mock sampler (>64 KiB output)
     and dummy workload (no physical model fit required).
"""

import json
import os
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from datetime import datetime, timedelta, timezone

# Ensure scripts directory is on sys.path
SCRIPTS_DIR = os.path.dirname(os.path.abspath(__file__))
if SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SCRIPTS_DIR)

import energy_per_token


class TestEnergyRecorderUnit(unittest.TestCase):
    """Unit tests for core functions in energy_per_token.py."""

    def test_drain_pipe_exceeding_64k(self):
        """Test that drain_pipe successfully drains >64 KiB of data without pipe stall."""
        # On Darwin, default pipe capacity is 65,536 bytes. A writer emitting >64 KiB
        # will block indefinitely if the reader does not continuously consume it.
        total_lines = 1000
        line_payload = "X" * 120 + "\n"  # ~121 bytes per line -> ~121 KiB total

        proc = subprocess.Popen(
            [sys.executable, "-c", f"import sys; [sys.stdout.write({line_payload!r}) for _ in range({total_lines})]; sys.stdout.flush()"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )

        lines = []
        drain_thread = threading.Thread(target=energy_per_token.drain_pipe, args=(proc.stdout, lines), daemon=True)
        drain_thread.start()

        proc.wait(timeout=5)
        drain_thread.join(timeout=2)
        proc.stdout.close()
        proc.stderr.close()

        self.assertEqual(proc.returncode, 0)
        total_bytes = sum(len(line) for line in lines)
        self.assertGreater(total_bytes, 65536, f"Expected >64 KiB drained, got {total_bytes} bytes")
        self.assertEqual(len(lines), total_lines)

    def test_parse_macmon_jsonl_valid(self):
        """Test parsing valid macmon JSONL stream."""
        sample_jsonl = (
            '{"timestamp": "2026-09-28T18:00:00.000Z", "sys_power": 12.34}\n'
            '{"timestamp": "2026-09-28T18:00:00.100Z", "sys_power": 14.56}\n'
            '{"timestamp": "2026-09-28T18:00:00.200Z", "sys_power": 13.90}\n'
        )
        readings, timestamps, field = energy_per_token.parse_macmon_jsonl(sample_jsonl)
        self.assertEqual(len(readings), 3)
        self.assertEqual(len(timestamps), 3)
        self.assertAlmostEqual(readings[0], 12340.0)  # converted to mW
        self.assertAlmostEqual(readings[1], 14560.0)
        self.assertEqual(field, "sys_power (macmon whole-system)")

    def test_parse_macmon_jsonl_trailing_sigint_fragment(self):
        """Test that a trailing partial JSON line caused by SIGINT interruption is gracefully discarded."""
        sample_jsonl = (
            '{"timestamp": "2026-09-28T18:00:00.000Z", "sys_power": 12.0}\n'
            '{"timestamp": "2026-09-28T18:00:00.100Z", "sys_power": 13.0}\n'
            '{"timestamp": "2026-09-28T18:00:00.200Z", "sys_pow'  # truncated fragment
        )
        readings, timestamps, _ = energy_per_token.parse_macmon_jsonl(sample_jsonl)
        self.assertEqual(len(readings), 2)
        self.assertEqual(len(timestamps), 2)

    def test_parse_macmon_jsonl_intermediate_corruption_rejected(self):
        """Test that malformed JSON on intermediate lines is rejected."""
        corrupted_jsonl = (
            '{"timestamp": "2026-09-28T18:00:00.000Z", "sys_power": 12.0}\n'
            'BAD_LINE_NOT_JSON\n'
            '{"timestamp": "2026-09-28T18:00:00.200Z", "sys_power": 13.0}\n'
        )
        with self.assertRaises(ValueError) as ctx:
            energy_per_token.parse_macmon_jsonl(corrupted_jsonl, reject_malformed=True)
        self.assertIn("Malformed or truncated JSON", str(ctx.exception))

    def test_validate_timestamp_coverage_monotonic_pass(self):
        """Test that valid monotonic timestamps covering >=85% pass."""
        base_t = datetime(2026, 9, 28, 18, 0, 0, tzinfo=timezone.utc)
        timestamps = [(base_t + timedelta(milliseconds=100 * i)).isoformat() for i in range(50)]
        # Total span: 4.9s for a 5.0s run -> 98% coverage
        span = energy_per_token.validate_timestamp_coverage(timestamps, expected_duration_s=5.0)
        self.assertAlmostEqual(span, 4.9, places=2)

    def test_validate_timestamp_coverage_negative_span_rejected(self):
        """Test that non-monotonic / reverse timestamps are rejected."""
        timestamps = [
            "2026-09-28T18:00:05.000Z",
            "2026-09-28T18:00:01.000Z",
        ]
        with self.assertRaises(ValueError) as ctx:
            energy_per_token.validate_timestamp_coverage(timestamps, expected_duration_s=5.0)
        self.assertIn("Negative timestamp span", str(ctx.exception))

    def test_validate_timestamp_coverage_insufficient_span_rejected(self):
        """Test that truncated timestamps covering <85% of duration are rejected."""
        base_t = datetime(2026, 9, 28, 18, 0, 0, tzinfo=timezone.utc)
        # Span of only 2.0s for a 10.0s run (20% coverage)
        timestamps = [(base_t + timedelta(milliseconds=100 * i)).isoformat() for i in range(21)]
        with self.assertRaises(ValueError) as ctx:
            energy_per_token.validate_timestamp_coverage(timestamps, expected_duration_s=10.0)
        self.assertIn("Incomplete timestamp coverage", str(ctx.exception))
        self.assertIn("Sampler output was truncated", str(ctx.exception))

    def test_validate_timestamp_coverage_excessive_gap_rejected(self):
        """Test that excessive gaps between consecutive samples (>2.0s) trigger stall error."""
        timestamps = [
            "2026-09-28T18:00:00.000Z",
            "2026-09-28T18:00:00.500Z",
            "2026-09-28T18:00:03.000Z",  # 2.5s gap (> 2.0s threshold)
            "2026-09-28T18:00:05.000Z",
        ]
        with self.assertRaises(ValueError) as ctx:
            energy_per_token.validate_timestamp_coverage(timestamps, expected_duration_s=5.0)
        self.assertIn("Sampler stalled", str(ctx.exception))
        self.assertIn("maximum gap between consecutive samples was 2.50s", str(ctx.exception))

    def test_validate_timestamp_coverage_out_of_interval_rejected(self):
        """Test that traces with right duration but timestamps outside the workload interval are rejected.
        
        Duration alone cannot establish alignment.
        """
        # Workload ran from 18:10:00 to 18:10:05 (5.0s duration)
        workload_start = datetime(2026, 9, 28, 18, 10, 0, tzinfo=timezone.utc)
        workload_end = datetime(2026, 9, 28, 18, 10, 5, tzinfo=timezone.utc)

        # Case A: Timestamps have right duration (4.9s for a 5.0s run), but ran 10 minutes earlier (18:00:00)
        stale_base = datetime(2026, 9, 28, 18, 0, 0, tzinfo=timezone.utc)
        stale_timestamps = [(stale_base + timedelta(milliseconds=100 * i)).isoformat() for i in range(50)]
        with self.assertRaises(ValueError) as ctx:
            energy_per_token.validate_timestamp_coverage(
                stale_timestamps,
                expected_duration_s=5.0,
                start_wall=workload_start,
                end_wall=workload_end,
            )
        self.assertIn("Timestamp misalignment", str(ctx.exception))
        self.assertIn("samples fall completely before execution window", str(ctx.exception))

        # Case B: Timestamps have right duration, but ran after workload finished
        future_base = datetime(2026, 9, 28, 18, 20, 0, tzinfo=timezone.utc)
        future_timestamps = [(future_base + timedelta(milliseconds=100 * i)).isoformat() for i in range(50)]
        with self.assertRaises(ValueError) as ctx:
            energy_per_token.validate_timestamp_coverage(
                future_timestamps,
                expected_duration_s=5.0,
                start_wall=workload_start,
                end_wall=workload_end,
            )
        self.assertIn("Timestamp misalignment", str(ctx.exception))
        self.assertIn("samples fall completely after execution window", str(ctx.exception))

    def test_detect_tokens(self):
        """Test auto-detection of stepped / generated token counts."""
        self.assertEqual(energy_per_token.detect_tokens("Stepped 2048 tokens in 11.2s"), 2048)
        self.assertEqual(energy_per_token.detect_tokens("Generated 128 tokens"), 128)
        self.assertEqual(energy_per_token.detect_tokens('{"eval_count": 512}'), 512)
        self.assertIsNone(energy_per_token.detect_tokens("No tokens mentioned here"))


class TestEnergyRecorderEndToEnd(unittest.TestCase):
    """End-to-end integration tests using mock samplers and dummy workloads."""

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.mock_bin = os.path.join(self.temp_dir.name, "mock_macmon")
        self._create_mock_macmon_script()

    def tearDown(self):
        self.temp_dir.cleanup()

    def _create_mock_macmon_script(self):
        """Create a universal executable mock macmon script controlled via env vars."""
        script_content = r"""#!/usr/bin/env python3
import json
import os
import signal
import sys
import time
from datetime import datetime, timedelta, timezone

def sigint_handler(sig, frame):
    sys.exit(0)

signal.signal(signal.SIGINT, sigint_handler)

mode = os.environ.get("MOCK_MODE", "normal")
payload_extra = os.environ.get("MOCK_PAYLOAD", "")
time_offset_s = float(os.environ.get("MOCK_TIME_OFFSET_S", "0.0"))

interval_s = 0.05
count = None
args = sys.argv[1:]
for i, a in enumerate(args):
    if a == "-i" and i + 1 < len(args):
        interval_s = float(args[i + 1]) / 1000.0
    elif a == "-s" and i + 1 < len(args):
        count = int(args[i + 1])

is_workload = (count is None)

if is_workload and mode == "crash":
    sys.stderr.write("Fatal mock sampler error\n")
    sys.exit(1)

samples_emitted = 0
t_start = time.time()

while True:
    now_dt = datetime.now(timezone.utc)
    if is_workload and time_offset_s != 0.0:
        now_dt = now_dt + timedelta(seconds=time_offset_s)
    now = now_dt.isoformat()
    record = {"timestamp": now, "sys_power": 15.25}
    if payload_extra:
        record["extra"] = payload_extra
    sys.stdout.write(json.dumps(record) + "\n")
    sys.stdout.flush()
    samples_emitted += 1

    if count is not None and samples_emitted >= count:
        break

    if is_workload:
        if mode == "truncate" and (time.time() - t_start) >= 1.0:
            break
        if mode == "stall" and samples_emitted == 10:
            time.sleep(2.5)  # 2.5s stall exceeding 2.0s limit

    time.sleep(interval_s)
"""
        with open(self.mock_bin, "w", encoding="utf-8") as f:
            f.write(script_content)
        os.chmod(self.mock_bin, stat.S_IRWXU)

    def test_e2e_high_volume_pipe_draining_pass(self):
        """End-to-end test with mock macmon emitting >64 KiB of JSONL data during workload."""
        # Extra payload per sample: ~260 characters so line is ~300 bytes.
        # At 20 Hz (interval 50ms) over 4.2s workload = ~84 samples * 300 = ~25 KiB.
        # With 400 chars padding, 100 samples = ~45 KiB workload + idle 30 samples * 450 B = ~14 KiB -> ~65+ KiB.
        raw_out_path = os.path.join(self.temp_dir.name, "raw_capture.txt")
        env = os.environ.copy()
        env["MACMON_BIN"] = self.mock_bin
        env["MOCK_MODE"] = "normal"
        env["MOCK_PAYLOAD"] = "A" * 600  # ~650 bytes per line * 100 lines = ~65 KiB+

        # Run energy_per_token.py with a 4.2s dummy workload that outputs token count
        cmd = [
            sys.executable,
            os.path.join(SCRIPTS_DIR, "energy_per_token.py"),
            "--label", "synthetic-test",
            "--sampler", "macmon",
            "--idle-seconds", "1.5",
            "--interval-ms", "50",
            "--raw-out", raw_out_path,
            "--",
            sys.executable, "-c",
            "import time; time.sleep(4.2); print('Stepped 512 tokens')",
        ]

        result = subprocess.run(cmd, capture_output=True, text=True, env=env)
        self.assertEqual(result.returncode, 0, f"energy_per_token failed:\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}")

        # Verify output summary
        self.assertIn("tokens                    : 512", result.stdout)
        self.assertIn("GROSS energy", result.stdout)
        self.assertIn("NET energy (idle removed)", result.stdout)

        # Verify raw capture exists and exceeded 64 KiB
        self.assertTrue(os.path.exists(raw_out_path))
        file_size = os.path.getsize(raw_out_path)
        self.assertGreater(file_size, 65536, f"Expected raw capture >64 KiB, got {file_size} bytes")

        # Verify raw capture contains both sections
        with open(raw_out_path, "r", encoding="utf-8") as f:
            content = f.read()
        self.assertIn("=== MACMON IDLE SAMPLES ===", content)
        self.assertIn("=== MACMON WORKLOAD SAMPLES ===", content)

    def test_e2e_excessive_gap_stall_detected(self):
        """End-to-end test verifying that an inter-sample stall > 2.0s triggers error."""
        env = os.environ.copy()
        env["MACMON_BIN"] = self.mock_bin
        env["MOCK_MODE"] = "stall"

        cmd = [
            sys.executable,
            os.path.join(SCRIPTS_DIR, "energy_per_token.py"),
            "--label", "synthetic-stall-test",
            "--sampler", "macmon",
            "--idle-seconds", "1.0",
            "--interval-ms", "50",
            "--",
            sys.executable, "-c",
            "import time; time.sleep(4.2); print('Stepped 128 tokens')",
        ]

        result = subprocess.run(cmd, capture_output=True, text=True, env=env)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Sampler stalled", result.stderr + result.stdout)

    def test_e2e_truncated_timestamp_coverage_detected(self):
        """End-to-end test verifying that premature sampler termination (<85% coverage) triggers error."""
        env = os.environ.copy()
        env["MACMON_BIN"] = self.mock_bin
        env["MOCK_MODE"] = "truncate"

        cmd = [
            sys.executable,
            os.path.join(SCRIPTS_DIR, "energy_per_token.py"),
            "--label", "synthetic-truncation-test",
            "--sampler", "macmon",
            "--idle-seconds", "1.0",
            "--interval-ms", "50",
            "--",
            sys.executable, "-c",
            "import time; time.sleep(4.2); print('Stepped 128 tokens')",
        ]

        result = subprocess.run(cmd, capture_output=True, text=True, env=env)
        output = result.stderr + result.stdout
        self.assertTrue(
            "Incomplete timestamp coverage" in output or "Timestamp misalignment" in output,
            f"Expected truncation or misalignment error, got:\n{output}",
        )

    def test_e2e_collector_crash_detected(self):
        """End-to-end test verifying that sampler crash propagates cleanly."""
        env = os.environ.copy()
        env["MACMON_BIN"] = self.mock_bin
        env["MOCK_MODE"] = "crash"

        cmd = [
            sys.executable,
            os.path.join(SCRIPTS_DIR, "energy_per_token.py"),
            "--label", "synthetic-crash-test",
            "--sampler", "macmon",
            "--idle-seconds", "1.0",
            "--interval-ms", "50",
            "--",
            sys.executable, "-c",
            "print('Stepped 128 tokens')",
        ]

        result = subprocess.run(cmd, capture_output=True, text=True, env=env)
        self.assertNotEqual(result.returncode, 0)

    def test_e2e_out_of_interval_timestamps_detected(self):
        """End-to-end test verifying that timestamps with right duration but outside workload interval are rejected."""
        env = os.environ.copy()
        env["MACMON_BIN"] = self.mock_bin
        env["MOCK_MODE"] = "normal"
        env["MOCK_TIME_OFFSET_S"] = "-60.0"  # Samples are 1 minute in the past

        cmd = [
            sys.executable,
            os.path.join(SCRIPTS_DIR, "energy_per_token.py"),
            "--label", "synthetic-offset-test",
            "--sampler", "macmon",
            "--idle-seconds", "1.0",
            "--interval-ms", "50",
            "--",
            sys.executable, "-c",
            "import time; time.sleep(4.2); print('Stepped 128 tokens')",
        ]

        result = subprocess.run(cmd, capture_output=True, text=True, env=env)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Timestamp misalignment in workload", result.stderr + result.stdout)


if __name__ == "__main__":
    unittest.main()
