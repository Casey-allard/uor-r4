#!/usr/bin/env python3
"""Measure joules per token for a local serving command on Apple Silicon.

WHY A SCRIPT AND NOT A ONE-LINER
--------------------------------
`powermetrics` reports SoC-wide power, so a naive reading during a workload charges the model for
the machine's idle floor as well. On an M1 laptop that floor is a few watts, the same order as the
workload, so a raw number is not a J/token figure -- it is a wrong one. This script:

  1. samples an IDLE baseline first, with no workload running;
  2. samples during the workload you supply, aligned to its own start and end;
  3. reports both, and the difference, so the subtraction is visible rather than implied;
  4. divides by the token count, which it AUTO-DETECTS from the command's own output when it can.

THE COMMAND GOES AFTER A BARE `--`
---------------------------------
    sudo python3 scripts/energy_per_token.py --label "native" --idle-seconds 8 \
        -- ./target/release/r4-native-chat --model <artifact.rgm> --temperature 0.0 --top-k 1 \
           -- "Once upon a time there was a little girl who"

Everything after the first `--` is the command, verbatim. (An earlier version of this script
declared an argument named `--command` while documenting `--`; that mismatch is fixed.)

TOKEN COUNT
-----------
`--tokens` is optional. If omitted, the script looks for the count in the command's stdout:
`[N model tokens]` (the native CLI) or `eval count: N` (ollama). If neither is found it refuses to
report J/token rather than dividing by a guess -- an energy figure with a wrong denominator is
worse than no figure. Pass `--tokens N` to override.

REQUIREMENTS
------------
  * root, because `powermetrics` requires it.
  * a constant machine state across both phases: AC power, lid open, nobody using the machine.
    Battery-discharge power is not comparable with AC power, and a hot SoC is not comparable with
    a cool one.

WHAT IT CANNOT DO
-----------------
  * No per-process attribution. That is `powermetrics --show-process-energy`, a different
    measurement (per-process energy impact, not SoC power).
  * No control of SoC power state. Keep runs short and repeat them.
  * A single repeat is not a result. Run each configuration at least three times and report the
    range, not just the mean.
"""

import argparse
import math
import re
import statistics
import subprocess
import sys
import time

# Power fields in PRIORITY order. One is chosen for the whole run and used consistently; see
# `parse_power`. `Combined Power` is the Apple Silicon SoC total and is what an energy figure
# should use. `CPU Power` is the CPU cluster alone and appears in the SAME sample, so a per-line
# alternation averages two different quantities together -- which is what an earlier version did,
# reporting an idle baseline of 1.4 mW that is not physically plausible for an M1.
POWER_FIELDS = [
    (
        "Combined Power (CPU + GPU + ANE)",
        re.compile(r"Combined Power \(CPU \+ GPU \+ ANE\)\s*:\s*([0-9.]+)\s*mW"),
    ),
    ("Package Power", re.compile(r"Package Power\s*:\s*([0-9.]+)\s*mW")),
    ("CPU Power", re.compile(r"CPU Power\s*:\s*([0-9.]+)\s*mW")),
]


def parse_power(text):
    """Pick ONE power field for the whole output, in priority order.

    Returns (readings_mW, field_name). Mixing fields corrupts the mean because several power
    fields appear in every sample, so the field used is reported to the user.
    """
    for name, pat in POWER_FIELDS:
        vals = [float(m) for m in pat.findall(text)]
        if vals:
            return vals, name
    return [], None

# Token-count patterns read from the command's own output.
TOKEN_PATTERNS = [
    # r4-native-chat emits `[128 model tokens; length; 617.3 tok/s, 1619.9 us/token]`, so the
    # closing bracket does NOT follow the word "tokens". Verified against real output.
    re.compile(r"\[(\d+)\s+model tokens"),
    re.compile(r"eval count:\s*(\d+)"),  # ollama --verbose
    re.compile(r"\beval_count[\"'\s:]+(\d+)"),  # ollama API JSON
]


def split_on_double_dash(argv):
    """Return (options, command). Everything after the first `--` is the command verbatim."""
    if "--" in argv:
        i = argv.index("--")
        return argv[:i], argv[i + 1 :]
    return argv, []


def run_powermetrics(seconds, interval_ms):
    """Sample for `seconds` and return (readings_mW, field_name, raw_text, stderr)."""
    count = max(1, math.ceil(seconds * 1000 / interval_ms))
    cmd = ["powermetrics", "--samplers", "cpu_power", "-i", str(interval_ms), "-n", str(count)]
    out = subprocess.run(cmd, capture_output=True, text=True)
    readings, field = parse_power(out.stdout)
    return readings, field, out.stdout, out.stderr


def detect_tokens(text):
    """Total tokens generated, summed across every match of the FIRST pattern that matches.

    Summing matters for a looped command: five runs of the native CLI print five telemetry lines,
    and returning only the first would under-count the denominator by 5x. Summing within one
    pattern (rather than across patterns) avoids double-counting when a tool reports the same
    quantity twice.
    """
    for pat in TOKEN_PATTERNS:
        hits = [int(m) for m in pat.findall(text)]
        if hits:
            return sum(hits)
    return None


def main():
    options, command = split_on_double_dash(sys.argv[1:])

    ap = argparse.ArgumentParser(
        description="Joules per token via powermetrics, with an idle-baseline subtraction. "
        "The command to measure goes after a bare --."
    )
    ap.add_argument("--label", required=True, help="name of the configuration under test")
    ap.add_argument("--tokens", type=int, default=None, help="override the auto-detected count")
    ap.add_argument("--idle-seconds", type=float, default=8.0)
    ap.add_argument("--interval-ms", type=int, default=500)
    ap.add_argument(
        "--max-seconds",
        type=float,
        default=180.0,
        help="safety bound on the workload sampling window",
    )
    args = ap.parse_args(options)

    if not command:
        sys.exit(
            "no command supplied. Put it after a bare --, e.g.\n"
            "  sudo python3 scripts/energy_per_token.py --label X --idle-seconds 8 -- "
            "./target/release/r4-native-chat --model ... -- \"prompt\""
        )

    print(f"config : {args.label}")
    print(f"command: {' '.join(command)}")
    print()

    print(f"[1/2] idle baseline, {args.idle_seconds:.0f}s, no workload ...")
    idle, idle_field, idle_raw, idle_err = run_powermetrics(args.idle_seconds, args.interval_ms)
    if not idle:
        sys.exit(
            "powermetrics produced no readings for the idle phase, so the subtraction is "
            "impossible and any J/token would be wrong.\n"
            f"stderr: {idle_err.strip()[:500]}\n"
            "If this says 'must be run as root', run the script under sudo.\n"
            "If it printed output but nothing matched, the field names differ on this OS; the "
            "first 600 bytes of the raw sample follow:\n" + (idle_raw or "")[:600]
        )
    idle_mean = statistics.mean(idle)
    print(f"      idle  n={len(idle):>3}  mean {idle_mean:8.1f} mW   field: {idle_field}")
    print()

    print("[2/2] workload ...")
    # Run the sampler without -n so it lives until we terminate it, and bound it with -n computed
    # from --max-seconds in case a sample count is required on this OS.
    n = max(1, math.ceil(args.max_seconds * 1000 / args.interval_ms))
    sampler = subprocess.Popen(
        ["powermetrics", "--samplers", "cpu_power", "-i", str(args.interval_ms), "-n", str(n)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    t0 = time.perf_counter()
    run = subprocess.run(command, capture_output=True, text=True)
    elapsed = time.perf_counter() - t0
    sampler.terminate()
    try:
        sampler_out, sampler_err = sampler.communicate(timeout=5)
    except subprocess.TimeoutExpired:
        sampler.kill()
        sampler_out, sampler_err = sampler.communicate()

    work, work_field = parse_power(sampler_out or "")
    if not work:
        print(f"      workload exited {run.returncode} in {elapsed:.2f}s")
        sys.exit(
            "powermetrics produced no readings for the workload phase.\n"
            f"stderr: {(sampler_err or '').strip()[:500]}\n"
            "If the workload finished in far less than one sampling interval "
            f"({args.interval_ms} ms), make it longer: the sampler cannot resolve a run shorter "
            "than its own window."
        )
    work_mean = statistics.mean(work)

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
    print(f"      work  n={len(work):>3}  mean {work_mean:8.1f} mW   field: {work_field}")
    print(f"      tokens detected: {tokens}{' (from --tokens)' if args.tokens else ' (auto)'}")
    print()

    gross_j = work_mean / 1000.0 * elapsed
    net_w = max(work_mean - idle_mean, 0.0)
    net_j = net_w / 1000.0 * elapsed

    print("RESULT")
    print(f"  wall time                 : {elapsed:.3f} s")
    print(f"  tokens                    : {tokens}")
    print(f"  decode rate               : {tokens / elapsed:.2f} tok/s")
    print(f"  idle power (not the model): {idle_mean:.1f} mW")
    print(f"  workload power            : {work_mean:.1f} mW")
    print(f"  power field used          : {work_field}")
    print(f"  GROSS energy              : {gross_j:.3f} J   ({gross_j / tokens:.4f} J/token)")
    print(f"  NET energy (idle removed) : {net_j:.3f} J   ({net_j / tokens:.4f} J/token)")
    print()

    # PLAUSIBILITY GUARD. A CPU-bound workload cannot draw under 200 mW on an M1-class SoC; an
    # idle machine sits at roughly 1-3 W and a loaded one at several watts. Observed on
    # MacBookPro17,1 / macOS 26A5378n: `CPU Power` reports 0 mW while the E-cluster sits at 60-71%
    # active residency, so `Combined Power` contains only the GPU. Reporting a J/token from that
    # would understate energy by roughly three orders of magnitude.
    if work_mean < 200.0:
        print("*** STOP: THE POWER READING IS NOT PHYSICALLY PLAUSIBLE ***")
        print(
            f"  A workload drawing {work_mean:.1f} mW cannot be a CPU-bound run on an M1-class SoC,\n"
            "  which sits near 1-3 W idle and several watts under load. The most likely cause is\n"
            "  that this powermetrics build does not populate the CPU power term at all, so the\n"
            "  selected field excludes the CPU. Check the raw sample for `CPU Power: 0 mW` while\n"
            "  cluster active residency is non-trivial."
        )
        print(
            "  DO NOT quote the J/token above. Report the measurement as UNAVAILABLE and say why.\n"
            "  Alternatives: try `sudo powermetrics -n 2 -i 1000` for any SoC/package power line;\n"
            "  or `sudo powermetrics --samplers tasks --show-process-energy` for a per-process\n"
            "  energy-impact comparison, which is Apple's unitless score and not joules."
        )
        print()
        return 2

    print("Quote the NET figure: the gross one charges the model for the machine's idle floor.")
    print("A single run is not a result. Repeat at least three times and report the range.")
    print("Record: AC or battery, whether the SoC was warm, and where the token count came from.")
    if run.returncode != 0:
        print(f"\nwarning: the command exited {run.returncode}; timing may be meaningless.")
        print((run.stderr or "").strip()[:300])
    return 0


if __name__ == "__main__":
    sys.exit(main())
