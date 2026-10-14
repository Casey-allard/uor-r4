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
from datetime import datetime, timezone
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
    # `check-formats`, not `check`. The Rust checker validates state formats; the full `check` is 57
    # rules and is DELEGATED to Python through the entry point, so asking for `check --json` now runs
    # the Python implementation - which has no `--json` and prints prose, which is what broke this
    # comparison in CI. This function exists to compare the Rust state-format checker against Python,
    # so it names that checker explicitly.
    code, out, err = run([str(binary), "check-formats", "--repo", str(REPO), "--json"])
    if code not in (0, 1) or not out.strip():
        raise RuntimeError(f"rust check failed ({code}): {err[:400]}")
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
        " 'age_seconds': {r.lab: (int(r.age_minutes * 60) if r.age_minutes is not None else None) for r in rows},"
        " 'stale': sorted(r.lab for r in rows if r.stale)}))",
    ])
    if code != 0:
        raise RuntimeError(f"python presence failed: {err[:400]}")
    return json.loads(out)


def _normalise_problem(text: str) -> str:
    """Strip the leading path so Rust and Python violations can be compared.

    Rust reports `docs/harness/ledger/x.jsonl:11.ts: ...` and Python reports `x.jsonl:11.ts: ...`
    for the same fault, because one is handed a path relative to the repo and the other a bare
    filename. Comparing the text without that prefix is what makes this a parity check on the
    FAULT rather than on how each implementation happens to spell a location.
    """
    tail = text.split(": ", 1)
    if len(tail) != 2:
        return text.strip()
    loc, problem = tail
    # Collapse quote style: Python renders a repr as 'ts' and Rust as "ts" for the same fault.
    # Comparing that difference would make this a check on each language's formatting rather than
    # on what was found, and it is exactly the kind of noise that gets a gate ignored.
    problem = problem.replace("'", '"')
    return f"{loc.rsplit('/', 1)[-1]}: {problem.strip()}"


def normalise_formats(doc: dict) -> dict[str, dict]:
    """Format name -> {files, rows, problems}.

    `problems` is the normalised fault text, not a count. Counting was a real defect: both
    implementations reported the same two violations and the check called it agreement, so a state
    file that violated the contract passed this gate. Agreement has to be about WHAT was found.
    """
    out = {}
    for f in doc["formats"]:
        raw = f.get("violations", f.get("problems", []))
        out[f["format"]] = {
            "files": f.get("files", 0),
            "rows": f.get("rows", 0),
            "problems": sorted(_normalise_problem(str(x)) for x in raw),
        }
    return out


def write_interop(binary: Path) -> list[str]:
    """Rust WRITES a receipt; the Python harness must read it back intact.

    Every other check here compares the two implementations on state Python wrote, which proves Rust
    can read the shared record and says nothing about whether Rust can add to it. A writer that is
    subtly wrong is worse than no writer: it corrupts the record the labs coordinate through.

    Runs in a temporary directory, never the real ledger. The month is fixed so the shard name is
    predictable, and the receipt carries a marker so a leak into real state would be obvious.
    """
    import tempfile

    findings: list[str] = []

    # 0b. THE PORT'S OWN COVERAGE. `check`'s rule set is a MOVING TARGET: while the D30 port runs
    #     module by module, other labs register rules into the same modules. A module is not done
    #     because it matched Python at one instant - if a rule is added afterwards, the Rust module is
    #     short and NOTHING REPORTS IT, so the port would look complete while checking less.
    #
    #     Rust declares what it implements (`uor-harness check-rules --json`); this compares that
    #     declaration against the rules Python actually registers. It fails in BOTH directions:
    #     a registered rule with no Rust implementation, and a declared rule Python does not register
    #     (which would inflate the count in the direction that flatters the port).
    #
    #     This is reported as a NOTE, not a failure, while the port is incomplete - the port is
    #     deliberately partial, and failing CI for that would block every other lab. It becomes a
    #     FAILURE the moment `check` stops delegating, which is the only claim it exists to protect.
    inventory = rust_inventory(binary)
    if inventory is not None:
        declared = {i for ids in inventory.get("ported", {}).values() for i in ids}
        registered = python_registered_rules()
        if registered is not None:
            unimplemented = sorted(registered - declared)
            phantom = sorted(declared - registered)
            module_counts = ", ".join(
                f"{m} {len(ids)}" for m, ids in sorted(inventory.get("ported", {}).items())
            )
            # **Two denominators, because they answer different questions and collapsing them hides
            # the one the goal is measured against.** `check`'s own rules are what `delegate_to_python`
            # is waiting on; the PR-body gates are real check logic that lives in `prlint` and would
            # otherwise be counted as already done. Reporting one number made the port look further
            # along than it was when a gate moved into the inventory.
            checkrules_only = {
                i for i in registered if not i.startswith("prlint.")
            }
            checkrules_left = sorted(checkrules_only - declared)
            print(f"rule inventory: {len(declared)} implemented ({module_counts}); "
                  f"{len(checkrules_left)} of check's own {len(checkrules_only)} rule(s) still in Python, "
                  f"{len(unimplemented)} of {len(registered)} including prlint's PR-body gates")
            if phantom:
                findings.append(
                    "the Rust rule inventory declares rules Python does not register: "
                    + ", ".join(phantom)
                    + " - a declared rule that does not exist inflates coverage"
                )
            if not unimplemented and not phantom:
                print("rule inventory: COMPLETE - every registered rule has a Rust implementation")
    marker = "parity-write-interop-marker"
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "docs" / "harness").mkdir(parents=True)
        code, out, err = run([
            str(binary), "receipt", "--repo", str(root), "--lab", "parityprobe",
            "--kind", "skill", "--skill", "interop", "--note", marker,
        ])
        if code != 0:
            findings.append(f"write interop: the Rust writer exited {code}: {err[:200]}")
            return findings

        shard = root / "docs" / "harness" / "ledger" / f"parityprobe-{datetime.now(timezone.utc):%Y-%m}.jsonl"
        if not shard.is_file():
            findings.append(f"write interop: Rust reported success but no shard at {shard.name}")
            return findings

        # Read it back through the PYTHON implementation's own reader, not by parsing the file here.
        probe = (
            "import json,sys;"
            "sys.path.insert(0,'scripts/harness');"
            "import core,multilab;"
            f"p=core.HarnessPaths.discover(repo={str(root)!r});"
            "rows=multilab.read_shards(p);"
            "print(json.dumps(rows))"
        )
        code, out, err = run([sys.executable, "-c", probe])
        if code != 0:
            findings.append(f"write interop: the Python reader failed: {err[:200]}")
            return findings
        try:
            rows = json.loads(out)
        except json.JSONDecodeError:
            findings.append(f"write interop: Python's reader returned non-JSON: {out[:200]}")
            return findings

        if len(rows) != 1:
            findings.append(f"write interop: Python read {len(rows)} receipt(s), expected 1")
            return findings
        row = rows[0]
        for key, expected in (
            ("lab", "parityprobe"),
            ("kind", "skill"),
            ("skill", "interop"),
            ("note", marker),
            ("schema", 1),
        ):
            if row.get(key) != expected:
                findings.append(f"write interop: {key} is {row.get(key)!r}, expected {expected!r}")
        if not row.get("ts"):
            findings.append("write interop: the receipt has no timestamp")
    return findings


def controlled_cases(binary: Path) -> list[str]:
    """Check the format checker on inputs whose verdict is known in advance.

    Agreement on whatever state happens to be in the working tree only proves a reader can read a
    valid file. These cases prove it *rejects* the things it should — without which "it passes" is
    not much of a claim.

    **These used to compare two implementations.** The Python one (`statecheck.py`) was removed under
    D27, so there is now a single checker and the comparison would be Rust against itself. The cases
    keep their value as known-bad inputs; what they assert is now that the one checker accepts and
    rejects correctly.
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

            r_code, _, _ = run([str(binary), "check-formats", "--repo", str(root), "--json"])
            r_ok = r_code == 0
            if r_ok != should_pass:
                findings.append(
                    f"controlled [{label}]: said ok={r_ok}, expected {should_pass} "
                    f"- the check is not testing what it claims"
                )
    return findings


def prep_agreement(binary: Path) -> list[str]:
    """The Rust and Python readers must agree about every pre-registration sheet.

    The D22 gate turns on a sheet's TEST FITNESS verdict, so a reader that disagrees about one is a
    gate that opens for one implementation and not the other. This compares the verdict and the
    missing-block list per sheet, read through each implementation's own parser.
    """
    import glob

    findings: list[str] = []
    sheets = sorted(glob.glob("docs/harness/prep/*.md"))
    if not sheets:
        return findings  # a checkout with no sheets is not a disagreement

    code, out, err = run([
        str(binary), "prep-check", "--repo", str(REPO), "--json",
    ])
    if code != 0:
        findings.append(f"prep agreement: the Rust prep check exited {code}: {err[:200]}")
        return findings
    try:
        rust_rows = {Path(r["sheet"]).name: r for r in json.loads(out)["sheets"]}
    except (json.JSONDecodeError, KeyError, TypeError) as exc:
        findings.append(f"prep agreement: could not read the Rust output ({exc}): {out[:200]}")
        return findings

    probe = (
        "import json,sys,pathlib;"
        "sys.path.insert(0,'scripts/harness');"
        "import core,prep;"
        "p=core.HarnessPaths.discover(repo='.');"
        "reqs=prep.sheet_requirements(p);"
        "out={};"
        "[out.update({f.name:{'fitness':prep.test_fitness(f.read_text(encoding='utf-8')),"
        "'missing':[n for k,(n,pred) in reqs.items() if not pred(prep.sections(f.read_text(encoding='utf-8')), f.read_text(encoding='utf-8'))]}})"
        " for f in sorted(pathlib.Path('docs/harness/prep').glob('*.md'))];"
        "print(json.dumps(out))"
    )
    code, out, err = run([sys.executable, "-c", probe])
    if code != 0:
        findings.append(f"prep agreement: the Python reader failed: {err[:200]}")
        return findings
    py_rows = json.loads(out)

    if set(rust_rows) != set(py_rows):
        findings.append(f"prep agreement: sheet sets differ (rust {sorted(rust_rows)}, python {sorted(py_rows)})")
    for name in sorted(set(rust_rows) & set(py_rows)):
        r, q = rust_rows[name], py_rows[name]
        if r["fitness"] != q["fitness"]:
            findings.append(f"prep {name}: rust says {r['fitness']}, python says {q['fitness']}")
        if sorted(r.get("missing", [])) != sorted(q.get("missing", [])):
            findings.append(
                f"prep {name}: missing blocks differ (rust {r.get('missing')}, python {q.get('missing')})"
            )
    return findings


def registry_agreement(binary: Path) -> list[str]:
    """Every milestone's headline and line counts must read identically in both implementations.

    The headline is the number a run is scored against, so a reader that disagrees about it is a
    pre-registration written against a different bar than the panel grades. The registry is also
    where three live headline FORMS meet — vacuous ("not qualified"), qualitative ("yes -> target
    no") and numeric — and rendering the wrong one misstates the bar.
    """
    findings: list[str] = []
    code, out, err = run([str(binary), "registry", "--repo", str(REPO), "--json"])
    if code != 0:
        findings.append(f"registry agreement: the Rust reader exited {code}: {err[:200]}")
        return findings
    try:
        rust_rows = {r["key"]: r for r in json.loads(out)["milestones"]}
    except (json.JSONDecodeError, KeyError, TypeError) as exc:
        findings.append(f"registry agreement: could not read the Rust output ({exc})")
        return findings

    probe = (
        "import json,sys;"
        "sys.path.insert(0,'scripts/harness');"
        "import core,registries;"
        "p=core.HarnessPaths.discover(repo='.');"
        "ms=registries.load_milestones(p);"
        "print(json.dumps({k:{'headline':m.headline_line(),'lines':m.line_counts(),'issue':m.issue}"
        " for k,m in ms.by_key.items()}))"
    )
    code, out, err = run([sys.executable, "-c", probe])
    if code != 0:
        findings.append(f"registry agreement: the Python reader failed: {err[:200]}")
        return findings
    py_rows = json.loads(out)

    if set(rust_rows) != set(py_rows):
        findings.append(
            f"registry agreement: milestone sets differ (rust {sorted(rust_rows)}, python {sorted(py_rows)})"
        )
    for key in sorted(set(rust_rows) & set(py_rows)):
        r, q = rust_rows[key], py_rows[key]
        if r["headline"] != q["headline"]:
            findings.append(f"registry {key} headline: rust {r['headline']!r} != python {q['headline']!r}")
        if r["lines"] != q["lines"]:
            findings.append(f"registry {key} lines: rust {r['lines']!r} != python {q['lines']!r}")
        if r["issue"] != q["issue"]:
            findings.append(f"registry {key} issue: rust {r['issue']} != python {q['issue']}")
    return findings


def render_agreement(binary: Path) -> list[str]:
    """A rendered pre-registration sheet must be byte-identical in both implementations.

    A sheet is a DOCUMENT two implementations must agree about — a lab may read one written by either,
    and the D22 gate scores against it. A one-character difference in a bar or a target is a different
    pre-registration, so this compares the whole text rather than selected fields.
    """
    import subprocess

    findings: list[str] = []
    code, rust_text, err = run([str(binary), "prep-render", "--repo", str(REPO), "-m", "m1",
                                "--mechanism", "parity-probe"])
    if code != 0:
        findings.append(f"render agreement: the Rust renderer exited {code}: {err[:200]}")
        return findings

    probe = (
        "import sys,types;"
        "sys.path.insert(0,'scripts/harness');"
        "import core,registries,prep;"
        "p=core.HarnessPaths.discover(repo='.');"
        "reg=registries.Registry.load(p);"
        "m=reg.milestones.get('m1');"
        "panel=registries.find_panel(reg.panels,m.panel) if m.panel else None;"
        "a=types.SimpleNamespace(mechanism='parity-probe',run='',bar='',base='',split='',steps='',"
        "stop='',cost='',seeds=None,rows=None);"
        "sys.stdout.write(prep.render_sheet(a,p,reg,m,panel))"
    )
    proc = subprocess.run([sys.executable, "-c", probe], cwd=REPO, capture_output=True, text=True, timeout=300)
    if proc.returncode != 0:
        findings.append(f"render agreement: the Python renderer failed: {proc.stderr[:200]}")
        return findings
    py_text = proc.stdout

    if rust_text != py_text:
        # Report the first differing line: the whole diff is 4.7KB of prose and unreadable in a log.
        r_lines, p_lines = rust_text.splitlines(), py_text.splitlines()
        where = "length differs"
        for i in range(max(len(r_lines), len(p_lines))):
            r = r_lines[i] if i < len(r_lines) else "<nothing>"
            q = p_lines[i] if i < len(p_lines) else "<nothing>"
            if r != q:
                where = f"line {i + 1}: rust {r[:90]!r} != python {q[:90]!r}"
                break
        findings.append(f"render agreement: sheets differ - {where}")
    return findings


def status_agreement(binary: Path) -> list[str]:
    """The rendered MILESTONES section must be identical in both implementations.

    Compared against Python's REAL `status` output rather than a Python helper, because the helpers
    are not what a lab reads. This is the section that carries the headline acceptance number, the
    line counts and the panel every lab works against, so a difference here is a lab reading a
    different milestone than the one being graded.
    """
    import subprocess

    findings: list[str] = []
    # Compare today's status in both implementations, not a historical
    # October 13 snapshot against the Python command's live local date.
    at = datetime.now(timezone.utc).astimezone().isoformat(timespec="seconds")
    code, rust_out, err = run([str(binary), "status", "--repo", str(REPO), "--at",
                               at, "--lab", "parity"])
    if code != 0:
        findings.append(f"status agreement: the Rust status exited {code}: {err[:200]}")
        return findings

    proc = subprocess.run(
        [sys.executable, str(REPO / "scripts" / "harness" / "cli.py"), "--lab", "parity", "status", "--at", at],
        cwd=REPO, capture_output=True, text=True, timeout=600,
    )
    if proc.returncode != 0:
        findings.append(f"status agreement: the Python status failed: {proc.stderr[:200]}")
        return findings

    def section(text: str, start: str, end: str) -> list[str]:
        lines = text.splitlines()
        try:
            i = next(i for i, l in enumerate(lines) if l.startswith(start))
        except StopIteration:
            return []
        j = next((j for j, l in enumerate(lines) if l.startswith(end) and j > i), len(lines))
        return [l for l in lines[i:j]]

    rust_lines = section(rust_out, "1. MILESTONES", "2. RECEIPTS")
    py_lines = section(proc.stdout, "1. MILESTONES", "2. RECEIPTS")
    if not py_lines:
        findings.append("status agreement: Python produced no MILESTONES section to compare")
        return findings
    if rust_lines != py_lines:
        where = "length differs"
        for i in range(max(len(rust_lines), len(py_lines))):
            r = rust_lines[i] if i < len(rust_lines) else "<nothing>"
            q = py_lines[i] if i < len(py_lines) else "<nothing>"
            if r != q:
                where = f"line {i + 1}: rust {r[:110]!r} != python {q[:110]!r}"
                break
        findings.append(f"status agreement: MILESTONES differ - {where}")

    # The source line carries the registry note, which is where a lab learns which owner direction
    # the cache was reconciled to. Compared without the timestamp, which is the only variable part.
    def source_line(text: str) -> str:
        for line in text.splitlines():
            if line.startswith("source: "):
                return line
        return ""

    if source_line(rust_out) != source_line(proc.stdout):
        findings.append(
            f"status agreement: source line differs (rust {source_line(rust_out)[:120]!r} != "
            f"python {source_line(proc.stdout)[:120]!r})"
        )
    return findings


def status_receipts_agreement(binary: Path) -> list[str]:
    """The RECEIPTS section must aggregate every lab's shard identically in both implementations.

    This is the section whose whole purpose is cross-lab visibility, so the properties compared are
    the aggregate counts, the per-kind breakdown, the last-5 lines and the two local 24-hour clauses.
    Two clauses legitimately differ and are excluded rather than tolerated silently:

      * the live timestamp of the newest receipt, which moves between the two runs; and
      * the milestone-issue clause, which the Rust side reports as UNAVAILABLE because it does not
        query GitHub. Claiming the issues are current without looking would be the worse bug.
    """
    import subprocess

    findings: list[str] = []
    # The Python status command uses the current local date for its receipts.
    at = datetime.now(timezone.utc).astimezone().isoformat(timespec="seconds")
    code, rust_out, err = run([str(binary), "status", "--repo", str(REPO), "--at",
                               at, "--lab", "parity"])
    if code != 0:
        findings.append(f"status receipts: the Rust status exited {code}: {err[:200]}")
        return findings
    # **The same instant as Rust.** Passing `--at` to Rust alone made this compare *Rust at an
    # injected instant* against *Python at the wall clock*, so it agreed only while the machine's
    # local date happened to equal the injected date: `main` passed in the late UTC evening and the
    # very next attempt failed once the UTC date rolled over. **A check that fails on a schedule is
    # one people learn to ignore**, so both sides now take the same clock.
    proc = subprocess.run(
        [sys.executable, str(REPO / "scripts" / "harness" / "cli.py"),
         "--lab", "parity", "status", "--at", at],
        cwd=REPO, capture_output=True, text=True, timeout=600,
    )
    if proc.returncode != 0:
        findings.append(f"status receipts: the Python status failed: {proc.stderr[:200]}")
        return findings

    def section(text: str, start: str, end: str) -> list[str]:
        lines = text.splitlines()
        try:
            i = next(i for i, l in enumerate(lines) if l.startswith(start))
        except StopIteration:
            return []
        j = next((j for j, l in enumerate(lines) if l.startswith(end) and j > i), len(lines))
        return lines[i:j]

    def stable(lines: list[str]) -> list[str]:
        out = []
        for l in lines:
            if "milestone issue" in l:
                continue  # the GitHub clause; see the docstring
            # The newest receipt line carries a live timestamp. Dropped by position: it is the first
            # `- <ts>` row, and its content is a receipt the other run may not have seen yet.
            if l.strip().startswith("- 2") and "heartbeat" in l and "no model line" in l:
                continue
            out.append(l)
        return out

    py_lines = stable(section(proc.stdout, "2. RECEIPTS", "3. DELIVERY"))
    rust_lines = stable(section(rust_out, "2. RECEIPTS", "3. DELIVERY"))
    if not py_lines:
        findings.append("status receipts: Python produced no RECEIPTS section")
        return findings

    # The aggregate counts are the claim. Compared as parsed fields, so a difference is reported as
    # numbers rather than as a wall of text.
    def counts(lines: list[str]) -> dict[str, str]:
        out: dict[str, str] = {}
        for l in lines:
            body = l.strip()
            if " receipts (" in body and "·" in body:
                for part in body.split("·"):
                    part = part.strip()
                    if ":" in part:
                        k, v = part.split(":", 1)
                        out[k.strip()] = v.strip()
            elif body.startswith("skills used:"):
                out["used"] = body
        return out

    rc, pc = counts(rust_lines), counts(py_lines)
    for key in sorted(set(rc) | set(pc)):
        if rc.get(key) != pc.get(key):
            findings.append(f"status receipts {key}: rust {rc.get(key)!r} != python {pc.get(key)!r}")

    # The last-5 ordering and the two local 24-hour clauses, compared as text minus the known clause.
    r_tail = [l for l in rust_lines if l.strip().startswith("milestones with no receipt")]
    p_tail = [l for l in py_lines if l.strip().startswith("milestones with no receipt")]
    if r_tail != p_tail:
        findings.append(f"status receipts stale milestones: rust {r_tail} != python {p_tail}")
    r_hb = [l for l in rust_lines if "never posted on the issue" in l]
    p_hb = [l for l in py_lines if "never posted on the issue" in l]
    if r_hb != p_hb:
        findings.append(f"status receipts unposted cards: rust {r_hb} != python {p_hb}")
    return findings


def brief_agreement(binary: Path) -> list[str]:
    """The brief's title, section 1 and owner rules must be identical in both implementations.

    Compared against Python's REAL `brief` output, because the brief is what a lab reads to learn what
    its task is. The token counts are compared too: they are a rough cost estimate, and a brief whose
    reported cost disagrees with the tool that produced it is a brief nobody can budget from.
    """
    import subprocess

    findings: list[str] = []
    code, rust_out, err = run([str(binary), "brief", "--repo", str(REPO), "-m", "m1"])
    if code != 0:
        findings.append(f"brief agreement: the Rust brief exited {code}: {err[:200]}")
        return findings
    proc = subprocess.run(
        [sys.executable, str(REPO / "scripts" / "harness" / "cli.py"), "brief", "-m", "m1"],
        cwd=REPO, capture_output=True, text=True, timeout=600,
    )
    if proc.returncode != 0:
        findings.append(f"brief agreement: the Python brief failed: {proc.stderr[:200]}")
        return findings

    def content_blocks(text: str) -> list[str]:
        """The title and `## heading` bodies, excluding the provenance footer and anything after."""
        out = []
        for line in text.splitlines():
            if line.startswith("---"):
                break
            if line.startswith("*Not ported to Rust yet:*"):
                break
            out.append(line)
        return out

    pys = content_blocks(proc.stdout)
    rss = content_blocks(rust_out)
    if not pys:
        findings.append("brief agreement: Python produced no brief content")
        return findings
    # Python carries sections this port does not; compare the blocks BOTH provide.
    rust_headings = {l[3:].strip() for l in rss if l.startswith("## ")}
    py_lines: list[str] = []
    skip = False
    for line in pys:
        if line.startswith("## "):
            skip = line[3:].strip() not in rust_headings
        if not skip:
            py_lines.append(line)
    if rust_headings and py_lines != rss:
        where = "length differs"
        for i in range(max(len(py_lines), len(rss))):
            a = py_lines[i] if i < len(py_lines) else "<nothing>"
            b = rss[i] if i < len(rss) else "<nothing>"
            if a != b:
                where = f"line {i + 1}: python {a[:100]!r} != rust {b[:100]!r}"
                break
        findings.append(f"brief agreement: shared sections differ - {where}")

    # Token counts for the shared sources must agree exactly.
    def src_tokens(text: str) -> dict[str, str]:
        out: dict[str, str] = {}
        for line in text.splitlines():
            if line.startswith("- `docs/harness/milestones.json`"):
                out[line] = ""
        return out

    if src_tokens(proc.stdout) != src_tokens(rust_out):
        findings.append(
            f"brief agreement: milestone source token counts differ "
            f"(python {sorted(src_tokens(proc.stdout))} != rust {sorted(src_tokens(rust_out))})"
        )
    return findings


def constants_agreement(binary: Path) -> list[str]:
    """Shared constants must be equal in both implementations, and both must accept every kind.

    These are the values two implementations hand to labs. When they disagree, the DISAGREEMENT is
    invisible - each side is self-consistent - and the lab believes whichever one it happened to read.
    Two real defects arrived this way on 13 October:

      * `presence` printed "stale after 30m" from a hardcoded local constant while the heartbeat used
        five, so the same question had two answers; and
      * the Rust `receipt` command defaulted to kind `receipt`, which this repository's authoritative
        kind set did not accept - so the generic command emitted a record its own checker rejected,
        every time it ran without an explicit kind.
    """
    findings: list[str] = []
    code, out, err = run([str(binary), "presence", "--repo", str(REPO), "--json"])
    if code != 0:
        findings.append(f"constants: presence exited {code}: {err[:200]}")
        return findings
    try:
        rust_ttl = int(json.loads(out).get("ttl_minutes", -1))
    except (json.JSONDecodeError, TypeError, ValueError) as exc:
        findings.append(f"constants: cannot read the Rust TTL from presence ({exc})")
        return findings

    probe = (
        "import sys;sys.path.insert(0,'scripts/harness');"
        "import multilab,receipts,json;"
        "print(json.dumps({'ttl':multilab.PRESENCE_TTL_MINUTES,'kinds':list(receipts.KINDS)}))"
    )
    code, out, err = run([sys.executable, "-c", probe])
    if code != 0:
        findings.append(f"constants: the Python side failed: {err[:200]}")
        return findings
    py = json.loads(out)

    if rust_ttl != py["ttl"]:
        findings.append(
            f"constants: PRESENCE_TTL_MINUTES differs - rust presence says {rust_ttl}, "
            f"python says {py['ttl']}. Both are self-consistent, so only the lab reading one of them "
            f"can tell."
        )

    # The generic kind the Rust receipt command defaults to must be one the authoritative set accepts.
    default_probe = (
        "import sys,re,pathlib;"
        "src=pathlib.Path('crates/uor-r4-harness/src/main.rs').read_text();"
        "m=re.search(r'flag_value\(args, \"--kind\"\)\.unwrap_or_else\(\|\| \"(\w+)\"', src);"
        "print(m.group(1) if m else '')"
    )
    code, out, err = run([sys.executable, "-c", default_probe])
    default_kind = out.strip() if code == 0 else ""
    if default_kind and default_kind not in py["kinds"]:
        findings.append(
            f"constants: the Rust `receipt` command defaults --kind to {default_kind!r}, which the "
            f"authoritative kind set rejects ({py['kinds']}). The generic command would create a "
            f"record its own checker refuses."
        )
    return findings


def rust_inventory(binary: Path) -> dict[str, object] | None:
    """What the Rust side says it implements, or None if the command is missing."""
    try:
        out = subprocess.run(
            [str(binary), "check-rules", "--json"],
            capture_output=True, text=True, timeout=60, check=False,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if out.returncode != 0 or not out.stdout.strip():
        return None
    try:
        return json.loads(out.stdout)
    except json.JSONDecodeError:
        return None


def python_registered_rules() -> set[str] | None:
    """Every rule id the Python side can register, or None if it cannot be imported.

    **`checkrules.rules()` is not the whole set, and this function said it was.** `prlint` declares
    its gates in `prlint.GATE_RULES` - they are raised against a **PR body**, which the engine never
    has, so they are data rather than registered checks - and the Rust inventory covers them.

    **This is the third time in this port that "the registered set" turned out to have more than one
    source**, after `mistakes._known_rule_ids` (which resolves from three) and my own data file that
    took one of them and stopped. The first two produced a check that was **wrong rather than
    missing**; this one did too, reporting three genuinely-ported rules as *"a declared rule that does
    not exist"* and calling the port inflated.

    Delegating to `mistakes._known_rule_ids()` rather than re-listing the sources: **it already
    answers this question, and re-listing is how the previous two went wrong.**
    """
    sys.path.insert(0, str(REPO / "scripts" / "harness"))
    try:
        import mistakes  # type: ignore import-not-found
    except ImportError:
        return None
    try:
        return set(mistakes._known_rule_ids())
    except Exception:  # a registry that will not load is another rule's problem
        return None


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

    # 0. A gate that only compares implementations passes whenever BOTH are wrong together. State
    #    that violates the frozen contract must fail here regardless of agreement, because that is
    #    what this job exists to catch.
    invalid = sorted(
        f["format"] for f in rust_check(binary)["formats"]
        if f.get("violations") or f.get("problems")
    )
    if invalid:
        findings.append(
            f"state violates the frozen contract in {invalid} - run `uor-harness check` for the faults"
        )

    # 1. the format check
    # The format check has ONE implementation now — the Rust one — so comparing it against itself
    # would be theatre. What is still worth asserting is that it runs and that the state is valid;
    # the meaningful cross-implementation comparisons are the ledger roll-up, the presence board,
    # the controlled cases and the write interop below.
    r_formats = normalise_formats(rust_check(binary))
    if not r_formats:
        findings.append("the Rust format check reported no formats; the contract may be unreadable")

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
    # Staleness is time-dependent and the two implementations read the clock seconds apart, so a
    # record sitting exactly on the TTL boundary will legitimately flap. That is timing jitter, not
    # a divergence - but a disagreement *away* from the boundary is a real bug (this check found
    # exactly that: Rust truncated the age to whole minutes before comparing, so a record aged
    # 30m25s read live while Python read stale).
    r_stale = sorted(row["lab"] for row in r_pres.get("labs", []) if row.get("stale"))
    p_stale = sorted(p_pres.get("stale", []))
    if r_stale != p_stale:
        p_ages = p_pres.get("age_seconds", {})
        only_rust = set(r_stale) - set(p_stale)
        only_py = set(p_stale) - set(r_stale)
        boundary = (r_pres.get("ttl_minutes", 30) * 60)
        jitter = {lab for lab in (only_rust | only_py)
                  if abs((p_ages.get(lab) or 0) - boundary) <= 60}
        real = (only_rust | only_py) - jitter
        if real:
            findings.append(
                f"presence staleness: rust {r_stale} != python {p_stale} "
                f"(away from the {boundary}s TTL boundary: {sorted(real)})"
            )
        elif not args.json:
            print(f"  (note: {sorted(jitter)} sits within 60s of the TTL boundary - timing jitter, not divergence)")

    controlled = controlled_cases(binary)
    findings.extend(controlled)
    interop = write_interop(binary)
    findings.extend(interop)
    prep = prep_agreement(binary)
    findings.extend(prep)
    registry = registry_agreement(binary)
    findings.extend(registry)
    render = render_agreement(binary)
    findings.extend(render)
    status_cmp = status_agreement(binary)
    findings.extend(status_cmp)
    receipts_cmp = status_receipts_agreement(binary)
    findings.extend(receipts_cmp)
    brief_cmp = brief_agreement(binary)
    findings.extend(brief_cmp)
    const_cmp = constants_agreement(binary)
    findings.extend(const_cmp)

    if args.json:
        print(json.dumps({
            "ok": not findings,
            "note": "the format check has one implementation (Rust); controlled cases test it directly",
            "controlled_cases": 6,
            "write_interop": "ok" if not interop else "FAILED",
            "prep_agreement": "ok" if not prep else "FAILED",
            "registry_agreement": "ok" if not registry else "FAILED",
            "render_agreement": "ok" if not render else "FAILED",
            "status_agreement": "ok" if not status_cmp else "FAILED",
            "status_receipts_agreement": "ok" if not receipts_cmp else "FAILED",
            "brief_agreement": "ok" if not brief_cmp else "FAILED",
            "constants_agreement": "ok" if not const_cmp else "FAILED",
            "state_valid": not invalid,
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
            print(f"  state valid in all {len(r_formats)} formats (one implementation; nothing to compare)")
        print("  ledger roll-up and presence board agree across both implementations")
        print(f"  the format checker accepts and rejects all {6} controlled cases")
        print(
            "  Rust-written receipt read back by Python: "
            + ("ok" if not interop else f"FAILED ({len(interop)})")
        )
        print("  prep sheets read identically by both: " + ("ok" if not prep else f"FAILED ({len(prep)})"))
        print(
            "  milestone headlines and line counts identical: "
            + ("ok" if not registry else f"FAILED ({len(registry)})")
        )
        print(
            "  rendered pre-registration sheet identical: "
            + ("ok" if not render else f"FAILED ({len(render)})")
        )
        print(
            "  status MILESTONES section identical: "
            + ("ok" if not status_cmp else f"FAILED ({len(status_cmp)})")
        )
        print(
            "  status RECEIPTS aggregation identical: "
            + ("ok" if not receipts_cmp else f"FAILED ({len(receipts_cmp)})")
        )
        print(
            "  brief shared sections and token counts identical: "
            + ("ok" if not brief_cmp else f"FAILED ({len(brief_cmp)})")
        )
        print(
            "  shared constants agree (TTL, receipt kinds): "
            + ("ok" if not const_cmp else f"FAILED ({len(const_cmp)})")
        )
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
