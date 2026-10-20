#!/usr/bin/env python3
"""Propagate the workspace release version to every non-Rust binding.

The single source of truth is `Cargo.toml`'s `[workspace.package].version`
field. Rust workspace members inherit it automatically via
`version.workspace = true`. The non-Rust artifacts (`bindings/npm/
package.json`, `bindings/python/pyproject.toml`) and the
`[workspace.dependencies]` entries that pin inter-crate path+version
references do not have a similar inheritance mechanism; this script
keeps them in sync.

Usage
-----
Bump the release version:

    1. Edit `Cargo.toml`'s `[workspace.package].version`.
    2. Run `python3 tools/sync-versions.py`.
    3. Commit the changes.

Verify there is no drift (called by CI):

    python3 tools/sync-versions.py --check

Exits non-zero with a diagnostic if any downstream file's pinned
version disagrees with the workspace's. Used as a guard in
`.github/workflows/ci.yml` so the gate fails before a release-time
mismatch can land on `main`.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent


def read_workspace_version() -> str:
    """Parse `[workspace.package].version` from the root Cargo.toml.

    Plain regex parse — we avoid pulling in a TOML lib so the script
    is dependency-free (matches the rest of `tools/`).
    """
    path = REPO_ROOT / "Cargo.toml"
    text = path.read_text()
    in_workspace_package = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("["):
            in_workspace_package = stripped == "[workspace.package]"
            continue
        if not in_workspace_package:
            continue
        m = re.match(r'\s*version\s*=\s*"([^"]+)"', line)
        if m:
            return m.group(1)
    raise SystemExit(
        "error: could not find [workspace.package].version in Cargo.toml"
    )


def file_update(path: Path, pattern: str, replacement: str, label: str) -> bool:
    """Apply a regex substitution to `path`. Returns True if the file
    changed. Uses `re.MULTILINE` so `^` matches line starts.
    """
    original = path.read_text()
    updated, count = re.subn(
        pattern, replacement, original, count=1, flags=re.MULTILINE
    )
    if count == 0:
        raise SystemExit(
            f"error: {label}: pattern not found in {path.relative_to(REPO_ROOT)}"
        )
    if updated == original:
        return False
    path.write_text(updated)
    return True


def diffs(version: str) -> list[tuple[Path, str, str]]:
    """Return (path, pinned-version, replacement-pattern-label) tuples
    for every downstream file that currently disagrees with `version`.
    """
    out: list[tuple[Path, str, str]] = []

    # `[workspace.dependencies].uor-addr* { ..., version = "X.Y.Z" }`
    cargo_toml = REPO_ROOT / "Cargo.toml"
    cargo_text = cargo_toml.read_text()
    for ws_dep in ("uor-addr", "uor-addr-c", "uor-addr-wasm"):
        m = re.search(
            rf'^{re.escape(ws_dep)}\s+=\s+\{{[^}}]*?version\s*=\s*"([^"]+)"',
            cargo_text,
            flags=re.MULTILINE,
        )
        if not m:
            raise SystemExit(
                f"error: could not find [workspace.dependencies].{ws_dep}.version in Cargo.toml"
            )
        if m.group(1) != version:
            out.append((cargo_toml, m.group(1), f"workspace.dependencies.{ws_dep}"))

    # `bindings/npm/package.json` — `"version"` field at the top level.
    npm = REPO_ROOT / "bindings" / "npm" / "package.json"
    npm_data = json.loads(npm.read_text())
    if npm_data.get("version") != version:
        out.append((npm, npm_data.get("version", "?"), "npm package.json"))

    # `bindings/python/pyproject.toml` — `version = "X.Y.Z"` under
    # `[project]`.
    py = REPO_ROOT / "bindings" / "python" / "pyproject.toml"
    py_text = py.read_text()
    py_match = re.search(r'^version\s*=\s*"([^"]+)"', py_text, flags=re.MULTILINE)
    if not py_match:
        raise SystemExit("error: could not find version in bindings/python/pyproject.toml")
    if py_match.group(1) != version:
        out.append((py, py_match.group(1), "python pyproject.toml"))

    return out


def write_synced(version: str) -> list[Path] :
    """Force every downstream file to match `version`. Returns the list
    of paths actually written.
    """
    changed: list[Path] = []

    # `[workspace.dependencies].uor-addr* { version = "X.Y.Z" }`
    cargo_toml = REPO_ROOT / "Cargo.toml"
    for ws_dep in ("uor-addr", "uor-addr-c", "uor-addr-wasm"):
        if file_update(
            cargo_toml,
            rf'(^{re.escape(ws_dep)}\s+=\s+\{{[^}}]*?version\s*=\s*")[^"]+(")',
            rf'\g<1>{version}\g<2>',
            f"workspace.dependencies.{ws_dep}",
        ):
            if cargo_toml not in changed:
                changed.append(cargo_toml)

    # npm package.json
    npm = REPO_ROOT / "bindings" / "npm" / "package.json"
    npm_data = json.loads(npm.read_text())
    if npm_data.get("version") != version:
        npm_data["version"] = version
        # Preserve indentation (npm canonical is 2-space).
        npm.write_text(json.dumps(npm_data, indent=2) + "\n")
        changed.append(npm)

    # Python pyproject.toml
    py = REPO_ROOT / "bindings" / "python" / "pyproject.toml"
    if file_update(
        py,
        r'(^version\s*=\s*")[^"]+(")',
        rf'\g<1>{version}\g<2>',
        "python pyproject.toml",
    ):
        changed.append(py)

    return changed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="Exit non-zero if any downstream file disagrees with the workspace version.",
    )
    args = parser.parse_args()

    version = read_workspace_version()

    if args.check:
        drifted = diffs(version)
        if not drifted:
            print(f"version sync OK — every binding pinned at {version}")
            return 0
        print(
            f"version drift detected — workspace.package.version = {version} but:",
            file=sys.stderr,
        )
        for path, pinned, label in drifted:
            rel = path.relative_to(REPO_ROOT)
            print(f"  {label}: {rel} pins {pinned}", file=sys.stderr)
        print(
            "\nrun `python3 tools/sync-versions.py` to fix.", file=sys.stderr
        )
        return 1

    written = write_synced(version)
    if not written:
        print(f"version sync OK — every binding already pinned at {version}")
    else:
        print(f"synced {len(written)} file(s) to version {version}:")
        for path in written:
            print(f"  {path.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
