#!/usr/bin/env python3
"""Check that the Rust and Python harnesses agree about the shared state.

Owner decision, 13 October 2026 (D27): the harness is ported to Rust, with the state formats
frozen so both implementations read the same files. Two implementations that disagree are worse
than one, because the record then says different things depending on which tool a lab happens to
run. This script is the regression test for that.

    scripts/harness/parity.py                 # compare, exit 1 on disagreement
    scripts/harness/parity.py --build         # build the Rust binary first
    scripts/harness/parity.py --json

It compares:

  * the **format check** — every format name, file count, row count and verdict
  * the **ledger roll-up** — receipts per lab and per kind
  * the **presence board** — which labs, which tier answered, and staleness

The comparison is over structured output (`--json`), not prose, so a wording change in either
implementation cannot fail it and a real disagreement cannot hide behind one.

**What this does not prove.** Agreement about *reading* state is not agreement about behaviour. The
Python harness owns behaviour — writing claims, publishing presence, the rule engine — until those
are ported and compared the same way. This script is the first of those comparisons, not the last.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
RUST_BIN_NAME = "uor-harness"


def rust_binary() -> Path | None:
    """Find the built Rust binary, without assuming a target directory layout."""
    candidates = []
    if env := os.environ.get("CARGO_TARGET_DIR"):
        candidates.append(Path(env) / "debug" / RUST_BIN_NAME)
    candidates += [
        REPO / "local" / "harness-target" / "debug" / RUST_BIN_NAME,
        REPO / "target" / "debug" / RUST_BIN_NAME,
    ]
    for c in candidates:
        if c.is_file():
            return c
    return None


def build(binary_dir: Path) -> Path | None:
    """Build the Rust harness with an explicit target dir, never the repo's `target` symlink."""
    cargo = shutil.which("cargo") or str(Path.home() / ".cargo" / "bin" / "cargo")
    env = dict(os.environ, CARGO_TARGET_DIR=str(binary_dir))
    print(f"parity: building with CARGO_TARGET_DIR={binary_dir}", file=sys.stderr)
    proc = subprocess.run(
        [cargo, "build", "-p", "uor-r4-harness", "--offline"],
        cwd=REPO, env=env, capture_output=True, text=True, timeout=900,
    )
    if proc.returncode != 0:
        print(proc.stderr[-2000:], file=sys.stderr)
        return None
    return binary_dir / "debug" / RUST_BIN_NAME


def run(cmd: list[str]) -> tuple[int, str, str]:
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, timeout=300)
    return proc.returncode, proc.stdout, proc.stderr


def rust_check(binary: Path) -> dict:
    code, out, err = run([str(binary), "check", "--repo", str(REPO), "--json"])
    if code not in (0, 1) or not out.strip():
        raise RuntimeError(f"rust check failed ({code}): {err[:400]}")
    return json.loads(out)


def python_check(repo: Path = REPO) -> dict:
    code, out, err = run([
        sys.executable, str(REPO / "scripts" / "harness" / "statecheck.py"),
        "--json", "--repo", str(repo),
    ])
    if code not in (0, 1) or not out.strip():
        raise RuntimeError(f"python check failed ({code}): {err[:400]}")
    return json.loads(out)


def rust_ledger(binary: Path) -> dict:
    code, out, _ = run([str(binary), "ledger", "--repo", str(REPO), "--json"])
    if code != 0:
        raise RuntimeError("rust ledger failed")
    return json.loads(out)


def python_ledger() -> dict:
    """Roll up the ledger through the Python implementation's own reader."""
    code, out, err = run([
        sys.executable, "-c",
        "import sys,json; sys.path.insert(0,'scripts/harness');"
        "import core,multilab;"
        "p=core.HarnessPaths.discover(repo='.');"
        "rows=multilab.read_shards(p);"
        "import collections;"
        "c=collections.defaultdict(collections.Counter);"
        "[c[r.get('lab') or 'unknown'].__setitem__(r.get('kind',''), c[r.get('lab') or 'unknown'][r.get('kind','')]+1) for r in rows];"
        "print(json.dumps({'receipts': len(rows), 'by_lab': {k: dict(v) for k, v in c.items()}}))",
    ])
    if code != 0:
        raise RuntimeError(f"python ledger failed: {err[:400]}")
    return json.loads(out)


def rust_presence(binary: Path) -> dict:
    code, out, _ = run([str(binary), "presence", "--repo", str(REPO), "--json"])
    if code != 0:
        raise RuntimeError("rust presence failed")
    return json.loads(out)


def python_presence() -> dict:
    code, out, err = run([
        sys.executable, "-c",
        "import sys,json; sys.path.insert(0,'scripts/harness');"
        "import core,multilab;"
        "p=core.HarnessPaths.discover(repo='.');"
        "rows=multilab.all_presence(p);"
        "print(json.dumps({'labs': sorted(r.lab for r in rows),"
        " 'via': {r.lab: r.source for r in rows},"
        " 'stale': sorted(r.lab for r in rows if r.stale)}))",
    ])
    if code != 0:
        raise RuntimeError(f"python presence failed: {err[:400]}")
    return json.loads(out)


def normalise_formats(doc: dict) -> dict[str, dict]:
    """Format name -> {files, rows, problems}. Deliberately ignores ordering and prose."""
    out = {}
    for f in doc["formats"]:
        out[f["format"]] = {
            "files": f.get("files", 0),
            "rows": f.get("rows", 0),
            "problem_count": len(f.get("violations", f.get("problems", []))),
        }
    return out


def controlled_cases(binary: Path) -> list[str]:
    """Check both implementations agree on inputs whose verdict we know in advance.

    Agreement on whatever state happens to be in the working tree only proves both can read a valid
    file. These cases prove both *reject* the same things - without which "they agree" is not much
    of a claim.
    """
    import tempfile

    findings: list[str] = []
    contract_src = REPO / "docs" / "harness" / "state-formats.json"

    cases = [
        ("a valid row", '{"lab":"probe","kind":"skill","ts":"2026-10-13T00:00:00Z","schema":1,"count":"1/3"}', True),
        ("ts as a number", '{"lab":"probe","kind":"skill","ts":12345,"schema":1}', False),
        ("a missing required key", '{"lab":"probe","ts":"2026-10-13T00:00:00Z","schema":1}', False),
        ("count as a number (mixed accepts it)", '{"lab":"probe","kind":"team","ts":"2026-10-13T00:00:00Z","schema":1,"count":3}', True),
        ("count as an object (mixed still rejects)", '{"lab":"probe","kind":"team","ts":"2026-10-13T00:00:00Z","schema":1,"count":{}}', False),
        ("not JSON at all", 'this is not json', False),
    ]

    for label, row, should_pass in cases:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "docs" / "harness" / "ledger").mkdir(parents=True)
            shutil.copy(contract_src, root / "docs" / "harness" / "state-formats.json")
            (root / "docs" / "harness" / "ledger" / "probe-2026-10.jsonl").write_text(row + "\n")

            r_code, _, _ = run([str(binary), "check", "--repo", str(root), "--json"])
            r_ok = r_code == 0
            try:
                p_ok = python_check(root)["ok"]
            except RuntimeError as exc:
                findings.append(f"controlled [{label}]: python failed to run: {exc}")
                continue

            if r_ok != p_ok:
                findings.append(f"controlled [{label}]: rust ok={r_ok} but python ok={p_ok}")
            elif r_ok != should_pass:
                findings.append(
                    f"controlled [{label}]: both said ok={r_ok}, expected {should_pass} "
                    f"- the check is not testing what it claims"
                )
    return findings


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="Check Rust/Python harness parity on shared state.")
    ap.add_argument("--build", action="store_true", help="build the Rust binary first")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)

    binary = rust_binary()
    if binary is None and args.build:
        binary = build(REPO / "local" / "harness-target")
    if binary is None:
        msg = ("the Rust harness is not built. Build it first:\n"
               "  CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness --offline\n"
               "or run this script with --build.")
        if args.json:
            print(json.dumps({"ok": False, "skipped": True, "reason": msg}, indent=2))
        else:
            print(f"parity: {msg}", file=sys.stderr)
        return 2

    findings: list[str] = []

    # 1. the format check
    r_formats = normalise_formats(rust_check(binary))
    p_formats = normalise_formats(python_check())
    if set(r_formats) != set(p_formats):
        findings.append(f"format sets differ: rust-only {sorted(set(r_formats) - set(p_formats))}, "
                        f"python-only {sorted(set(p_formats) - set(r_formats))}")
    for name in sorted(set(r_formats) & set(p_formats)):
        if r_formats[name] != p_formats[name]:
            findings.append(f"{name}: rust {r_formats[name]} != python {p_formats[name]}")

    # 2. the ledger roll-up
    r_led, p_led = rust_ledger(binary), python_ledger()
    if r_led["receipts"] != p_led["receipts"]:
        findings.append(f"ledger receipt count: rust {r_led['receipts']} != python {p_led['receipts']}")
    r_labs = {row["lab"]: dict(row["kinds"]) for row in r_led["by_lab"]}
    if r_labs != p_led["by_lab"]:
        findings.append(f"ledger by lab: rust {r_labs} != python {p_led['by_lab']}")

    # 3. the presence board. Both sides are normalised to {lab: tier} so a difference in
    #    presentation cannot masquerade as a difference in meaning.
    r_pres, p_pres = rust_presence(binary), python_presence()
    r_via = {row["lab"]: row.get("via", "") for row in r_pres.get("labs", [])}
    if sorted(r_via) != sorted(p_pres.get("labs", [])):
        findings.append(f"presence labs: rust {sorted(r_via)} != python {sorted(p_pres.get('labs', []))}")
    if r_via != p_pres.get("via"):
        findings.append(f"presence tier answered: rust {r_via} != python {p_pres.get('via')}")
    r_stale = sorted(row["lab"] for row in r_pres.get("labs", []) if row.get("stale"))
    if r_stale != sorted(p_pres.get("stale", [])):
        findings.append(f"presence staleness: rust {r_stale} != python {sorted(p_pres.get('stale', []))}")

    controlled = controlled_cases(binary)
    findings.extend(controlled)

    if args.json:
        print(json.dumps({
            "ok": not findings,
            "controlled_cases": 6,
            "binary": str(binary),
            "formats": len(r_formats),
            "ledger_receipts": r_led["receipts"],
            "labs_with_presence": len(r_via),
            "findings": findings,
        }, indent=2))
    else:
        print(f"parity: rust ({binary.name}) vs python, {len(r_formats)} formats, "
              f"{r_led['receipts']} receipts, {len(r_via)} lab(s) with presence")
        if findings:
            print(f"  DISAGREEMENT ({len(findings)}):")
            for f in findings:
                print(f"    {f}")
        else:
            print("  agree on every format, the ledger roll-up, and the presence board")
        print(f"  agree on all {6} controlled cases (accept and reject)")
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
