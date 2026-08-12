#!/usr/bin/env python3
"""Append structured recovery notes to the session ledger."""

from __future__ import annotations

import argparse
import datetime as dt
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / "docs/research/SESSION_LEDGER.md"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Append a session-ledger entry.")
    parser.add_argument(
        "--kind",
        required=True,
        choices=["read", "run", "edit", "decision", "next", "checkpoint", "result"],
        help="Entry category.",
    )
    parser.add_argument("--note", required=True, help="Short plain-text note.")
    parser.add_argument(
        "--files",
        default="",
        help="Comma-separated file paths relevant to the entry.",
    )
    parser.add_argument(
        "--cmd",
        default="",
        help="Shell command associated with the entry, if any.",
    )
    return parser.parse_args()


def append_entry(kind: str, note: str, files: str, cmd: str) -> None:
    now = dt.datetime.now().astimezone()
    day = now.strftime("%Y-%m-%d")
    timestamp = now.strftime("%H:%M:%S %Z")

    lines = []
    if LEDGER.exists():
        existing = LEDGER.read_text(encoding="utf-8")
        if f"## {day}\n" not in existing:
            lines.append(f"\n## {day}\n")
    else:
        lines.append(f"# Session Ledger\n\n## {day}\n")

    entry = f"- {timestamp} [{kind}] {note}\n"
    lines.append(entry)

    cleaned_files = [part.strip() for part in files.split(",") if part.strip()]
    if cleaned_files:
        lines.append(f"  Files: {', '.join(cleaned_files)}\n")
    if cmd.strip():
        lines.append(f"  Cmd: `{cmd.strip()}`\n")

    with LEDGER.open("a", encoding="utf-8") as fh:
        fh.writelines(lines)


def main() -> int:
    args = parse_args()
    append_entry(args.kind, args.note, args.files, args.cmd)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
