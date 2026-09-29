#!/usr/bin/env python3
"""Measure joules per token for a local serving command on Apple Silicon.

WHY A SCRIPT AND NOT A ONE-LINER
--------------------------------
`macmon` (or `powermetrics`) reports power, so a naive reading during a workload charges the model for
the machine's idle floor as well. On an M1 laptop that floor is a few watts, the same order as the
workload, so a raw number is not a J/token figure -- it is a wrong one. This script:

  1. samples an IDLE baseline first, with no workload running;
  2. samples during the workload you supply, aligned to its own start and end;
  3. reports both, and the difference, so the subtraction is visible rather than implied;
  4. divides by the token count, which it AUTO-DETECTS from the command's own output when it can.

SAMPLER MODES
-------------
  --sampler macmon (default):
    Uses `/opt/homebrew/bin/macmon` in pipe mode without requiring root/sudo.
    Measures `sys_power` (whole-system power in Watts), which integrates all SoC and system draw.
    Recommended on macOS versions where per-component CPU/RAM power reporting is unavailable.

  --sampler powermetrics:
    Uses `powermetrics` (requires root/sudo). Measures SoC Combined/Package/CPU power.

  --sampler dual:
    Runs both `macmon` (`sys_power`) and `powermetrics` simultaneously for cross-validation.
    Reports marginal agreement between whole-system and SoC-internal counters.

THE COMMAND GOES AFTER A BARE `--`
---------------------------------
    python3 scripts/energy_per_token.py --label "integer-w256" --idle-seconds 8 -- \
        ./target/release/examples/same-input-step --model-type integer ...
"""

import argparse
import json
import math
import os
import re
import signal
import statistics
import subprocess
import sys
import time

import threading
from datetime import datetime, timezone

MACMON_BIN = os.environ.get("MACMON_BIN", "/opt/homebrew/bin/macmon")

POWER_FIELDS = [
    (
        "Combined Power (CPU + GPU + ANE)",
        re.compile(r"Combined Power \(CPU \+ GPU \+ ANE\)\s*:\s*([0-9.]+)\s*mW"),
    ),
    ("Package Power", re.compile(r"Package Power\s*:\s*([0-9.]+)\s*mW")),
    ("CPU Power", re.compile(r"CPU Power\s*:\s*([0-9.]+)\s*mW")),
]

TOKEN_PATTERNS = [
    re.compile(r"\[(\d+)\s+model tokens"),
    re.compile(r"eval count:\s*(\d+)"),
    re.compile(r"\beval_count[\"'\s:]+(\d+)"),
    re.compile(r"Generated\s+(\d+)\s+tokens"),
    re.compile(r"Stepped\s+(\d+)\s+tokens"),
]


def drain_pipe(stream, line_accumulator):
    """Continuously drain lines from stream until EOF to prevent OS pipe buffer stall."""
    try:
        for line in iter(stream.readline, ""):
            line_accumulator.append(line)
    except Exception:
        pass
    finally:
        try:
            stream.close()
        except Exception:
            pass


def parse_powermetrics_text(text):
    """Pick ONE power field for the whole output, in priority order."""
    for name, pat in POWER_FIELDS:
        vals = [float(m) for m in pat.findall(text)]
        if vals:
            return vals, name
    return [], None


def parse_macmon_jsonl(text, reject_malformed=True):
    """Parse macmon newline-delimited JSON stream and extract sys_power in mW and timestamps.
    
    If reject_malformed is True, incomplete or corrupted JSON lines raise ValueError.
    Note: A trailing incomplete fragment on the very last line caused by SIGINT termination
    is discarded if preceding lines successfully parsed valid samples.
    """
    readings_mw = []
    timestamps = []
    lines = text.strip().splitlines()
    num_lines = len(lines)
    for line_idx, line in enumerate(lines, 1):
        line_s = line.strip()
        if not line_s or line_s.startswith("==="):
            continue
        try:
            data = json.loads(line_s)
        except json.JSONDecodeError as e:
            if line_idx == num_lines and len(readings_mw) > 0:
                # Discard trailing fragment from SIGINT process interruption
                continue
            if reject_malformed:
                raise ValueError(
                    f"Malformed or truncated JSON on line {line_idx} of macmon output: "
                    f"{line_s[:100]!r}... ({e})"
                )
            continue
        if "sys_power" in data and data["sys_power"] is not None:
            # sys_power is in Watts; convert to mW for unit parity
            readings_mw.append(float(data["sys_power"]) * 1000.0)
            if "timestamp" in data and data["timestamp"]:
                timestamps.append(data["timestamp"])
    return readings_mw, timestamps, "sys_power (macmon whole-system)"


def validate_timestamp_coverage(
    timestamps,
    expected_duration_s,
    label="workload",
    min_coverage_ratio=0.85,
    max_gap_seconds=2.0,
    start_wall=None,
    end_wall=None,
    max_boundary_offset_s=2.5,
):
    """Validate that sample timestamps cover at least min_coverage_ratio of expected duration,
    consecutive samples do not have stalls exceeding max_gap_seconds,
    and sample timestamps align with the execution interval [start_wall, end_wall].
    
    Rejects incomplete coverage, excessive stalls, and traces outside the workload interval.
    """
    if not timestamps:
        raise ValueError(f"No timestamps found in {label} samples.")
    if len(timestamps) < 2:
        if expected_duration_s >= 2.0:
            raise ValueError(
                f"Only {len(timestamps)} timestamp recorded for a {expected_duration_s:.2f}s {label}. "
                "Sampler output is incomplete."
            )
        return 0.0

    parsed = []
    for ts_str in timestamps:
        try:
            dt = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
            if dt.tzinfo is None:
                dt = dt.replace(tzinfo=timezone.utc)
            parsed.append(dt)
        except Exception as e:
            raise ValueError(f"Cannot parse timestamp {ts_str!r}: {e}")

    t_first = parsed[0]
    t_last = parsed[-1]
    span_s = (t_last - t_first).total_seconds()
    if span_s < 0:
        raise ValueError(f"Negative timestamp span in {label}: {span_s}s")

    # Verify temporal alignment against actual wall-clock execution interval
    if start_wall is not None and end_wall is not None:
        if start_wall.tzinfo is None:
            start_wall = start_wall.replace(tzinfo=timezone.utc)
        if end_wall.tzinfo is None:
            end_wall = end_wall.replace(tzinfo=timezone.utc)

        if t_last < start_wall:
            raise ValueError(
                f"Timestamp misalignment in {label}: samples fall completely before execution window "
                f"(last sample at {t_last.isoformat()} was before start at {start_wall.isoformat()})."
            )
        if t_first > end_wall:
            raise ValueError(
                f"Timestamp misalignment in {label}: samples fall completely after execution window "
                f"(first sample at {t_first.isoformat()} was after end at {end_wall.isoformat()})."
            )
        startup_offset = (t_first - start_wall).total_seconds()
        if startup_offset > max_boundary_offset_s:
            raise ValueError(
                f"Timestamp misalignment in {label}: sampler started {startup_offset:.2f}s after "
                f"workload began (threshold: {max_boundary_offset_s:.1f}s)."
            )
        shutdown_offset = (end_wall - t_last).total_seconds()
        if shutdown_offset > max_boundary_offset_s:
            raise ValueError(
                f"Timestamp misalignment in {label}: sampler ended {shutdown_offset:.2f}s before "
                f"workload finished (threshold: {max_boundary_offset_s:.1f}s)."
            )

    coverage_ratio = span_s / expected_duration_s if expected_duration_s > 0 else 1.0

    # Check for stalls/gaps between consecutive samples
    max_gap = 0.0
    for i in range(len(parsed) - 1):
        gap = (parsed[i + 1] - parsed[i]).total_seconds()
        if gap > max_gap:
            max_gap = gap
    if max_gap > max_gap_seconds:
        raise ValueError(
            f"Sampler stalled during {label}: maximum gap between consecutive samples was {max_gap:.2f}s "
            f"(threshold: {max_gap_seconds:.1f}s)."
        )

    # Reject incomplete coverage:
    # 1. On substantial runs (>= 4.0s), coverage ratio must meet min_coverage_ratio (85%).
    # 2. On short runs (>= 2.0s), allow up to 30% startup/shutdown margin (coverage >= 70%).
    if expected_duration_s >= 4.0:
        if coverage_ratio < min_coverage_ratio:
            raise ValueError(
                f"Incomplete timestamp coverage in {label}: sampled span is {span_s:.2f}s "
                f"for a {expected_duration_s:.2f}s run ({coverage_ratio * 100.0:.1f}% coverage, "
                f"required >= {min_coverage_ratio * 100.0:.1f}%). "
                f"First timestamp: {timestamps[0]}, last: {timestamps[-1]}. "
                "Sampler output was truncated or terminated prematurely."
            )
    elif expected_duration_s >= 2.0:
        if coverage_ratio < 0.70:
            raise ValueError(
                f"Incomplete timestamp coverage in {label}: sampled span is {span_s:.2f}s "
                f"for a {expected_duration_s:.2f}s run ({coverage_ratio * 100.0:.1f}% coverage, required >= 70.0%). "
                f"First timestamp: {timestamps[0]}, last: {timestamps[-1]}."
            )
    return span_s


def validate_powermetrics_coverage(readings, expected_duration_s, interval_ms, label="workload", min_coverage_ratio=0.85):
    """Validate sample count for powermetrics against expected duration."""
    if not readings:
        raise ValueError(f"No power readings found in {label}.")
    sampled_s = len(readings) * (interval_ms / 1000.0)
    if expected_duration_s >= 2.0:
        coverage_ratio = sampled_s / expected_duration_s
        if coverage_ratio < min_coverage_ratio:
            raise ValueError(
                f"Incomplete sample coverage in {label}: {len(readings)} samples (~{sampled_s:.2f}s) "
                f"for a {expected_duration_s:.2f}s run ({coverage_ratio * 100.0:.1f}% coverage, "
                f"required >= {min_coverage_ratio * 100.0:.1f}%)."
            )
    return sampled_s


def get_powermetrics_cmd(interval_ms, count):
    base = ["powermetrics", "--samplers", "cpu_power", "-i", str(interval_ms), "-n", str(count)]
    if os.geteuid() != 0:
        return ["sudo", "-n"] + base
    return base


def run_powermetrics_idle(seconds, interval_ms):
    count = max(1, math.ceil(seconds * 1000 / interval_ms))
    cmd = get_powermetrics_cmd(interval_ms, count)
    out = subprocess.run(cmd, capture_output=True, text=True)
    readings, field = parse_powermetrics_text(out.stdout)
    validate_powermetrics_coverage(readings, seconds, interval_ms, label="idle baseline")
    return readings, field, out.stdout, out.stderr


def run_macmon_idle(seconds, interval_ms):
    count = max(1, math.ceil(seconds * 1000 / interval_ms))
    cmd = [MACMON_BIN, "pipe", "-s", str(count), "-i", str(interval_ms)]
    t0_wall = datetime.now(timezone.utc)
    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )
    stdout_lines = []
    stderr_lines = []
    t_out = threading.Thread(target=drain_pipe, args=(proc.stdout, stdout_lines), daemon=True)
    t_err = threading.Thread(target=drain_pipe, args=(proc.stderr, stderr_lines), daemon=True)
    t_out.start()
    t_err.start()
    proc.wait(timeout=max(30, int(seconds * 2)))
    t_out.join(timeout=2)
    t_err.join(timeout=2)
    t1_wall = datetime.now(timezone.utc)
    raw_stdout = "".join(stdout_lines)
    raw_stderr = "".join(stderr_lines)
    readings, timestamps, field = parse_macmon_jsonl(raw_stdout)
    validate_timestamp_coverage(timestamps, seconds, label="idle baseline", start_wall=t0_wall, end_wall=t1_wall)
    return readings, timestamps, field, raw_stdout, raw_stderr


def detect_tokens(text):
    for pat in TOKEN_PATTERNS:
        hits = [int(m) for m in pat.findall(text)]
        if hits:
            return sum(hits)
    return None


def split_on_double_dash(argv):
    if "--" in argv:
        i = argv.index("--")
        return argv[:i], argv[i + 1 :]
    return argv, []


def main():
    options, command = split_on_double_dash(sys.argv[1:])

    ap = argparse.ArgumentParser(
        description="Joules per token on Apple Silicon with idle-baseline subtraction. "
        "The command to measure goes after a bare --."
    )
    ap.add_argument("--label", required=True, help="name of the configuration under test")
    ap.add_argument("--sampler", choices=["macmon", "powermetrics", "dual"], default="macmon",
                    help="power sampler: 'macmon' (sys_power, no sudo), 'powermetrics' (sudo), or 'dual' (both)")
    ap.add_argument("--tokens", type=int, default=None, help="override the auto-detected count")
    ap.add_argument("--idle-seconds", type=float, default=8.0)
    ap.add_argument("--interval-ms", type=int, default=100)
    ap.add_argument(
        "--max-seconds",
        type=float,
        default=300.0,
        help="safety bound on the workload sampling window",
    )
    ap.add_argument("--raw-out", type=str, default=None, help="path to write raw sampler output lines")
    args = ap.parse_args(options)

    if not command:
        sys.exit(
            "no command supplied. Put it after a bare --, e.g.\n"
            "  python3 scripts/energy_per_token.py --label X --idle-seconds 8 -- "
            "./target/release/examples/same-input-step ..."
        )

    if args.sampler in ("macmon", "dual") and not os.path.exists(MACMON_BIN):
        sys.exit(f"ERROR: macmon binary not found at {MACMON_BIN}. Please install with 'brew install macmon'.")

    print(f"config  : {args.label}")
    print(f"sampler : {args.sampler}")
    print(f"command : {' '.join(command)}")
    print()

    # Phase 1: Idle baseline
    print(f"[1/2] idle baseline, {args.idle_seconds:.1f}s, no workload ...")
    
    idle_pm = []
    idle_field_pm = None
    idle_mac = []
    idle_mac_ts = []
    idle_field_mac = None
    raw_pm = ""
    raw_mac = ""

    if args.sampler in ("powermetrics", "dual"):
        idle_pm, idle_field_pm, raw_pm, err_pm = run_powermetrics_idle(args.idle_seconds, args.interval_ms)
        if not idle_pm and args.sampler == "powermetrics":
            sys.exit(
                "powermetrics produced no readings for the idle phase.\n"
                f"stderr: {err_pm.strip()[:500]}\n"
                "Run 'sudo -v' beforehand so 'sudo -n powermetrics' succeeds without interactive prompting."
            )
        if idle_pm:
            mean_pm = statistics.mean(idle_pm)
            std_pm = statistics.stdev(idle_pm) if len(idle_pm) > 1 else 0.0
            print(f"      [powermetrics] idle n={len(idle_pm):>3}  mean {mean_pm:8.1f} mW (std ±{std_pm:6.1f} mW)  field: {idle_field_pm}")

    if args.sampler in ("macmon", "dual"):
        idle_mac, idle_mac_ts, idle_field_mac, raw_mac, err_mac = run_macmon_idle(args.idle_seconds, args.interval_ms)
        if not idle_mac:
            sys.exit(
                "macmon produced no readings for the idle phase.\n"
                f"stderr: {err_mac.strip()[:500]}"
            )
        mean_mac = statistics.mean(idle_mac)
        std_mac = statistics.stdev(idle_mac) if len(idle_mac) > 1 else 0.0
        min_mac = min(idle_mac)
        max_mac = max(idle_mac)
        print(f"      [macmon]       idle n={len(idle_mac):>3}  mean {mean_mac:8.1f} mW (std ±{std_mac:6.1f} mW, min {min_mac:7.1f} mW, max {max_mac:7.1f} mW)  field: {idle_field_mac}")

    print()

    # Phase 2: Workload
    print("[2/2] workload ...")
    
    pm_proc = None
    mac_proc = None
    pm_lines = []
    pm_err_lines = []
    pm_thread = None
    pm_err_thread = None

    mac_lines = []
    mac_err_lines = []
    mac_thread = None
    mac_err_thread = None

    if args.sampler in ("powermetrics", "dual"):
        n = max(1, math.ceil(args.max_seconds * 1000 / args.interval_ms))
        pm_cmd = get_powermetrics_cmd(args.interval_ms, n)
        pm_proc = subprocess.Popen(
            pm_cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        pm_thread = threading.Thread(target=drain_pipe, args=(pm_proc.stdout, pm_lines), daemon=True)
        pm_err_thread = threading.Thread(target=drain_pipe, args=(pm_proc.stderr, pm_err_lines), daemon=True)
        pm_thread.start()
        pm_err_thread.start()

    if args.sampler in ("macmon", "dual"):
        mac_proc = subprocess.Popen(
            [MACMON_BIN, "pipe", "-i", str(args.interval_ms)],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        mac_thread = threading.Thread(target=drain_pipe, args=(mac_proc.stdout, mac_lines), daemon=True)
        mac_err_thread = threading.Thread(target=drain_pipe, args=(mac_proc.stderr, mac_err_lines), daemon=True)
        mac_thread.start()
        mac_err_thread.start()

    t0_wall = datetime.now(timezone.utc)
    t0 = time.perf_counter()
    run = subprocess.run(command, capture_output=True, text=True)
    elapsed = time.perf_counter() - t0
    t1_wall = datetime.now(timezone.utc)

    raw_pm_work = ""
    raw_mac_work = ""

    if pm_proc:
        pm_proc.terminate()
        try:
            pm_proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            pm_proc.kill()
            pm_proc.wait()
        if pm_thread:
            pm_thread.join(timeout=2)
        if pm_err_thread:
            pm_err_thread.join(timeout=2)
        raw_pm_work = "".join(pm_lines)

    if mac_proc:
        mac_proc.send_signal(signal.SIGINT)
        try:
            mac_proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            mac_proc.kill()
            mac_proc.wait()
        if mac_thread:
            mac_thread.join(timeout=2)
        if mac_err_thread:
            mac_err_thread.join(timeout=2)
        raw_mac_work = "".join(mac_lines)

    if args.raw_out:
        os.makedirs(os.path.dirname(os.path.abspath(args.raw_out)), exist_ok=True)
        with open(args.raw_out, "w", encoding="utf-8") as rf:
            if raw_mac:
                rf.write("=== MACMON IDLE SAMPLES ===\n")
                rf.write(raw_mac)
                if not raw_mac.endswith("\n"):
                    rf.write("\n")
            if raw_mac_work:
                rf.write("=== MACMON WORKLOAD SAMPLES ===\n")
                rf.write(raw_mac_work)
                if not raw_mac_work.endswith("\n"):
                    rf.write("\n")
            if raw_pm:
                rf.write("=== POWERMETRICS IDLE SAMPLES ===\n")
                rf.write(raw_pm)
                if not raw_pm.endswith("\n"):
                    rf.write("\n")
            if raw_pm_work:
                rf.write("=== POWERMETRICS WORKLOAD SAMPLES ===\n")
                rf.write(raw_pm_work)
                if not raw_pm_work.endswith("\n"):
                    rf.write("\n")

    # Parse workload power and validate coverage
    work_mac = []
    work_mac_ts = []
    work_field_mac = None
    try:
        if raw_mac_work:
            work_mac, work_mac_ts, work_field_mac = parse_macmon_jsonl(raw_mac_work)
            validate_timestamp_coverage(
                work_mac_ts,
                elapsed,
                label="workload",
                start_wall=t0_wall,
                end_wall=t1_wall,
            )

        if raw_pm_work:
            work_pm, work_field_pm = parse_powermetrics_text(raw_pm_work)
            validate_powermetrics_coverage(work_pm, elapsed, args.interval_ms, label="workload")
    except ValueError as e:
        sys.exit(f"[energy_per_token] Sampler validation error: {e}")

    combined = (run.stdout or "") + (run.stderr or "")
    tokens = args.tokens if args.tokens else detect_tokens(combined)
    if not tokens:
        print(f"      workload exited {run.returncode} in {elapsed:.2f}s")
        sys.exit(
            "could not determine the token count from the command's output, and J/token with a "
            "guessed denominator is worse than no figure.\n"
            "Pass --tokens N using the count the tool itself reported.\n"
            "Last 400 bytes of the command's output:\n  " + combined[-400:].replace("\n", "\n  ")
        )

    print(f"      workload exited {run.returncode} in {elapsed:.2f}s")
    print(f"      tokens detected: {tokens}{' (from --tokens)' if args.tokens else ' (auto)'}")
    print()

    # Primary analysis based on chosen sampler
    primary_idle = idle_mac if args.sampler in ("macmon", "dual") else idle_pm
    primary_work = work_mac if args.sampler in ("macmon", "dual") else work_pm
    primary_field = work_field_mac if args.sampler in ("macmon", "dual") else work_field_pm

    if not primary_work:
        sys.exit(f"ERROR: {args.sampler} produced no readings for the workload phase in {elapsed:.2f}s.")

    idle_mean = statistics.mean(primary_idle)
    idle_std = statistics.stdev(primary_idle) if len(primary_idle) > 1 else 0.0
    idle_min = min(primary_idle)
    idle_max = max(primary_idle)

    work_mean = statistics.mean(primary_work)
    work_std = statistics.stdev(primary_work) if len(primary_work) > 1 else 0.0
    gross_j = (work_mean / 1000.0) * elapsed
    net_w = max(work_mean - idle_mean, 0.0)
    net_j = (net_w / 1000.0) * elapsed

    print("RESULT")
    print(f"  wall time                 : {elapsed:.3f} s")
    print(f"  tokens                    : {tokens}")
    print(f"  decode rate               : {tokens / elapsed:.2f} tok/s")
    print(f"  idle power (not the model): {idle_mean:.1f} mW")
    print(f"  idle spread (std / range) : ±{idle_std:.1f} mW [{idle_min:.1f} - {idle_max:.1f} mW]")
    print(f"  workload power            : {work_mean:.1f} mW (std ±{work_std:.1f} mW)")
    print(f"  power field used          : {primary_field}")
    print(f"  GROSS energy              : {gross_j:.3f} J   ({gross_j / tokens:.4f} J/token)")
    print(f"  NET energy (idle removed) : {net_j:.3f} J   ({net_j / tokens:.4f} J/token)")
    print()

    # Dual cross-validation report
    if args.sampler == "dual" and work_pm and idle_pm:
        pm_idle_mean = statistics.mean(idle_pm)
        pm_work_mean = statistics.mean(work_pm)
        pm_net_w = max(pm_work_mean - pm_idle_mean, 0.0)
        pm_net_j = (pm_net_w / 1000.0) * elapsed
        print("DUAL VALIDATION (macmon sys_power vs powermetrics)")
        print(f"  macmon sys_power net energy     : {net_j:.3f} J   ({net_j / tokens:.4f} J/token)")
        print(f"  powermetrics net energy         : {pm_net_j:.3f} J   ({pm_net_j / tokens:.4f} J/token)")
        if pm_net_j > 0:
            ratio = net_j / pm_net_j
            print(f"  Whole-System / SoC Internal Ratio: {ratio:.2f}x")
        else:
            print("  powermetrics reported ~0 net power (CPU power unpopulated on this OS).")
        print()

    # Plausibility check
    if work_mean < 200.0:
        print("*** STOP: THE POWER READING IS NOT PHYSICALLY PLAUSIBLE (<200 mW) ***")
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
