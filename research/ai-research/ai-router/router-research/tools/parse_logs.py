#!/usr/bin/env python3
import datetime
import json
import os
import re
import sys
from typing import Dict, List, Tuple

JSON_RE = re.compile(r"^__JSON_SUMMARY__\s*(\{.*\})\s*$")

REQUIRED_TOP_LEVEL = [
    "schema_version",
    "parsed",
    "args",
    "metrics",
    "timings_sec",
    "artifacts",
    "git",
    "notes",
]

REQUIRED_METRICS = [
    "test_mse_before",
    "test_mse_after",
    "train_label_sse_per",
    "test_label_sse_per",
    "buckets",
    "slots_used",
    "test_unseen_rate",
]

REQUIRED_TIMINGS = [
    "dataset",
    "chart_opt",
    "routing_eval",
    "training_ema",
    "growth",
    "total",
]


def validate_summary(summary: Dict) -> Tuple[bool, List[str]]:
    errors: List[str] = []
    if not isinstance(summary, dict):
        return False, ["summary is not an object"]

    for k in REQUIRED_TOP_LEVEL:
        if k not in summary:
            errors.append(f"missing top-level key: {k}")

    if "metrics" in summary and isinstance(summary.get("metrics"), dict):
        for k in REQUIRED_METRICS:
            if k not in summary["metrics"]:
                errors.append(f"missing metrics.{k}")
    elif "metrics" in summary:
        errors.append("metrics is not an object")

    if "timings_sec" in summary and isinstance(summary.get("timings_sec"), dict):
        for k in REQUIRED_TIMINGS:
            if k not in summary["timings_sec"]:
                errors.append(f"missing timings_sec.{k}")
    elif "timings_sec" in summary:
        errors.append("timings_sec is not an object")

    return len(errors) == 0, errors


def parse_one_log(path: str, fn: str) -> Dict:
    summary_line = None
    json_err = None

    with open(path, "r", encoding="utf-8", errors="ignore") as f:
        for line in f:
            m = JSON_RE.match(line.strip())
            if m:
                summary_line = m.group(1)

    if summary_line is None:
        return {
            "parsed": False,
            "parse_error": "summary line missing",
            "log_file": fn,
        }

    try:
        summary = json.loads(summary_line)
    except Exception as e:
        json_err = str(e)
        summary = {
            "parsed": False,
            "parse_error": f"invalid json summary: {json_err}",
            "log_file": fn,
        }
        return summary

    ok, errors = validate_summary(summary)
    if not ok:
        summary = dict(summary)
        summary["parsed"] = False
        summary["parse_error"] = "; ".join(errors)

    summary["log_file"] = fn
    summary["parsed_at"] = datetime.datetime.now().isoformat(timespec="seconds")
    return summary


def main():
    if len(sys.argv) != 3:
        print("Usage: parse_logs.py <raw_log_dir> <parsed_out_dir>")
        sys.exit(2)

    raw_dir, out_dir = sys.argv[1], sys.argv[2]
    os.makedirs(out_dir, exist_ok=True)

    n = 0
    for fn in sorted(os.listdir(raw_dir)):
        if not fn.endswith(".log"):
            continue
        path = os.path.join(raw_dir, fn)
        summary = parse_one_log(path, fn)

        out_path = os.path.join(out_dir, fn.replace(".log", ".json"))
        with open(out_path, "w", encoding="utf-8") as g:
            json.dump(summary, g, indent=2, sort_keys=True)
        n += 1

    print(f"Parsed {n} log(s) into {out_dir}")


if __name__ == "__main__":
    main()
