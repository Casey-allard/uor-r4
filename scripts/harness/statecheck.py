#!/usr/bin/env python3
"""Check the live harness state against the frozen format contract.

Owner decision, 13 October 2026 (D27): the harness is being ported to Rust, and the state
formats are frozen as the interface between the two implementations. This script is what makes
that freeze real rather than aspirational: it fails when a state file stops matching the shape
the contract records, so neither implementation can drift without CI saying so.

    scripts/harness/statecheck.py            # check every format
    scripts/harness/statecheck.py --json      # machine-readable
    scripts/harness/statecheck.py --regen     # re-freeze the contract from the files on disk

`--regen` is deliberate and narrow: re-freezing is how an intentional format change lands, and it
must be a separate, reviewable commit. Never regenerate to silence a failure you have not
understood — a shape change that no one intended is exactly the drift this exists to catch.
"""

from __future__ import annotations

import argparse
import glob
import json
import sys
from pathlib import Path

#: Default repository root. Overridable with --repo so this checker can be pointed at any
#: checkout -- which is what lets the Rust port and this implementation be compared on the same
#: controlled input rather than only on whatever state happens to be in the working tree.
REPO = Path(__file__).resolve().parents[2]


def value_kind(value: object) -> str:
    if isinstance(value, bool):
        return "boolean"
    if isinstance(value, (int, float)):
        return "number"
    if isinstance(value, str):
        return "string"
    if value is None:
        return "nullable"
    if isinstance(value, dict):
        return "object"
    if isinstance(value, list):
        return "array"
    return "unknown"


def shape(value: object, depth: int = 0) -> dict:
    if isinstance(value, dict):
        if depth >= 3:
            return {"type": "object"}
        return {"type": "object", "keys": {k: shape(v, depth + 1) for k, v in sorted(value.items())}}
    if isinstance(value, list):
        return {"type": "array", "items": shape(value[0], depth + 1) if value else {}}
    return {"type": value_kind(value)}


def compare(actual: dict, expected: dict, where: str, problems: list[str]) -> None:
    """Every key in the contract must be present with a compatible kind.

    Extra keys are reported as a note, not a failure: a format may gain a field without breaking
    a reader, and forbidding that would make the contract unmaintainable. A missing key or a
    changed kind breaks readers, so those fail.
    """
    atype, etype = actual.get("type"), expected.get("type")
    if etype == "nullable":
        # 'nullable' genuinely accepts any value.
        return
    if etype == "mixed":
        # 'mixed' means the contract observed more than one type for this key. It does NOT mean
        # "anything": the contract lists the types in `types`, and a value outside that list is
        # drift. Accepting everything here silently passed an object for `count`, which a real
        # reader would then have to handle and cannot.
        allowed = expected.get("types")
        if allowed and atype not in allowed:
            problems.append(f"{where}: kind {atype} not in the recorded mixed types {allowed}")
        return
    if atype != etype:
        problems.append(f"{where}: kind changed {etype} -> {atype}")
        return
    if etype == "object":
        exp_keys, act_keys = expected.get("keys", {}), actual.get("keys", {})
        for key, sub in exp_keys.items():
            if key not in act_keys:
                # A key the contract does not mark optional must appear in every document; one it
                # marks optional may legitimately be absent from this record.
                if not sub.get("optional"):
                    problems.append(f"{where}.{key}: key missing (was {sub.get('type')})")
            else:
                compare(act_keys[key], sub, f"{where}.{key}", problems)
    elif etype == "array":
        sub = expected.get("items")
        # An empty array observed in the data matches any item shape: there is nothing to
        # contradict the contract, and a legitimately empty list is not drift.
        if sub and actual.get("items"):
            compare(actual["items"], sub, f"{where}[]", problems)


def contract_path(repo: Path | None = None) -> Path:
    return (repo or REPO) / "docs" / "harness" / "state-formats.json"


def check(json_out: bool = False, repo: Path | None = None) -> int:
    root = repo or REPO
    CONTRACT = contract_path(root)
    if not CONTRACT.exists():
        print(f"statecheck: {CONTRACT} is missing", file=sys.stderr)
        return 2
    contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
    results = []
    for name, entry in contract["formats"].items():
        paths = sorted(glob.glob(str(root / entry["pattern"])))
        if not paths:
            results.append({"format": name, "files": 0, "problems": [], "note": "no file present"})
            continue
        problems: list[str] = []
        if "row" in entry:
            rows = 0
            for p in paths:
                for i, line in enumerate(Path(p).read_text(encoding="utf-8").splitlines(), 1):
                    if not line.strip():
                        continue
                    rows += 1
                    try:
                        rec = json.loads(line)
                    except json.JSONDecodeError as exc:
                        problems.append(f"{Path(p).name}:{i}: not JSON ({exc.msg})")
                        continue
                    compare(shape(rec), entry["row"], f"{Path(p).name}:{i}", problems)
                    for key in entry.get("required_always", []):
                        if key not in rec:
                            problems.append(f"{Path(p).name}:{i}: required key {key!r} missing")
            results.append({"format": name, "files": len(paths), "rows": rows, "problems": problems})
        else:
            for p in paths:
                if not p.endswith(".json"):
                    continue
                try:
                    doc = json.loads(Path(p).read_text(encoding="utf-8"))
                except json.JSONDecodeError as exc:
                    problems.append(f"{Path(p).name}: not JSON ({exc.msg})")
                    continue
                # panels hold many panels per file; compare each entry against the contract shape
                if name == "panels" and isinstance(doc.get("panels"), dict):
                    for pid, panel in doc["panels"].items():
                        compare(shape(panel), entry["panel_shape"], f"{Path(p).name}:{pid}", problems)
                else:
                    compare(shape(doc), entry["shape"], Path(p).name, problems)
            results.append({"format": name, "files": len(paths), "problems": problems})
    failed = [r for r in results if r["problems"]]
    if json_out:
        try:
            label = str(CONTRACT.relative_to(root))
        except ValueError:
            label = str(CONTRACT)
        print(json.dumps({"contract": label, "root": str(root), "formats": results,
                          "ok": not failed}, indent=2))
    else:
        print(f"state formats: {len(results)} checked against {contract_path(root)}")
        for r in results:
            state = "ok" if not r["problems"] else f"{len(r['problems'])} PROBLEM(S)"
            extra = f" ({r.get('rows', r['files'])} rows)" if "rows" in r else f" ({r['files']} file(s))"
            print(f"  {r['format']:<16} {state}{extra}")
            for p in r["problems"][:5]:
                print(f"      {p}")
            if len(r["problems"]) > 5:
                print(f"      ... and {len(r['problems']) - 5} more")
    return 1 if failed else 0


def regen() -> int:
    """Re-freeze the contract from the files on disk. A separate, reviewable act."""
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    script = Path(__file__).resolve()
    print("statecheck: re-freezing. Review the diff before committing; this is an interface change.")
    print(f"  (regenerate with the same generator the contract was created from, then commit both)")
    print(f"  contract: {CONTRACT.relative_to(REPO)}")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="Check harness state against the frozen format contract.")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--regen", action="store_true", help="re-freeze the contract (review the diff)")
    ap.add_argument("--repo", default=None, help="repository root (default: this checkout)")
    args = ap.parse_args(argv)
    if args.regen:
        return regen()
    return check(json_out=args.json, repo=Path(args.repo).resolve() if args.repo else None)


# --- rule-engine integration ---------------------------------------------------------------
# Registered like every other harness rule, so `cli.py check` enforces the frozen formats
# everywhere the harness runs (including CI) rather than only when someone remembers to call
# this script.


def register_rules():  # noqa: ANN201 - returns list[checkrules.Rule]
    from checkrules import Rule

    def _check(paths, reg=None):  # noqa: ANN001 - the engine passes (paths, registry)
        """Rules return a list of RuleResult; this one reports a single finding either way."""
        from checkrules import RuleResult

        import io
        from contextlib import redirect_stdout

        buf = io.StringIO()
        with redirect_stdout(buf):
            code = check(json_out=False)
        if code == 0:
            return [
                RuleResult(
                    rule_id="harness.state_formats",
                    severity="error",
                    ok=True,
                    message="every state file matches docs/harness/state-formats.json",
                )
            ]
        bad = [ln.strip() for ln in buf.getvalue().splitlines() if "PROBLEM" in ln]
        return [
            RuleResult(
                rule_id="harness.state_formats",
                severity="error",
                ok=False,
                message="state format drift: " + ("; ".join(bad[:3]) if bad else "see statecheck.py"),
                path="docs/harness/state-formats.json",
                fix="make the file match the contract, or re-freeze it deliberately and commit both",
            )
        ]

    return [
        Rule(
            id="harness.state_formats",
            description="persisted state matches the frozen format contract",
            severity="error",
            check=_check,
            rationale=(
                "the state formats are the interface between the harness and the Rust port "
                "(D27); drift means the two implementations can disagree about what the shared "
                "record says"
            ),
        )
    ]


if __name__ == "__main__":
    raise SystemExit(main())
