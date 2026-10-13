#!/usr/bin/env python3
"""Ratchet gate for D25: no dense matrix product may be added to executable source.

D25 (owner, 2026-10-12) prohibits dense matrix multiplication in training and in
serving, and the owner restated it on 2026-10-13: neither softmax nor matmul may
appear anywhere, and neither may be dressed up behind a helper with a geometric
name. D23 has its interlock (`scripts/check_no_softmax.py`); this is D25's, built
to the same mechanics so one reviewer reads both the same way. It cannot prove the
tree is matmul-free; it stops the count from growing so removal can proceed
without new violations landing underneath it.

What it counts, per scanned file, in **code** (comments and plain string literals
are removed first, so prose about the ban cannot trip it):

  `.matmul(`                  a call to the dense product
  `MatMul`                    Candle's dense-product op, applied by name
  `cublas`  (any case)        a cuBLAS handle, binding or entry point
  `gemm`    (word, any case)  a BLAS dense product by its own name

Raw strings (`r"..."`, `r#"..."#`) are kept: the CUDA kernels live in one, and
that text is executable. Tensor-core fragment intrinsics (`wmma`, `mma_sync`) are
counted **informationally** and never fail the gate: the owner anticipates tensor
cores for *structured geometric* kernels, while a dense GEMM on tensor cores
stays banned. `--list` prints every site of both kinds, which is the audit a
reviewer reads to find wrapper dressing (a helper with a geometric name whose
body calls one of the patterns above).

Baseline: scripts/no-matmul-baseline.json maps repo-relative path -> number of
densely-multiplying sites when the ratchet was set. A file above its baseline, or
a file absent from the baseline with any hit, fails.

  python3 scripts/check_no_matmul.py                   # check (exit 1 on growth)
  python3 scripts/check_no_matmul.py --list            # every site, for audit
  python3 scripts/check_no_matmul.py --update-baseline # after REMOVING sites

--update-baseline refuses to raise any count, so it cannot be used to bless a new
dense product. Dated records under docs/ are not scanned: they are evidence, not
plans.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BASELINE = ROOT / "scripts" / "no-matmul-baseline.json"
SCAN_DIRS = ("crates",)
SUFFIXES = (".rs", ".cu", ".cuh", ".wgsl")

# The gated call-site patterns, as D25's interlock defines them.
SITE_PATTERNS = (
    ("matmul-method", re.compile(r"\.matmul\(")),
    ("candle-op", re.compile(r"MatMul")),
    ("cublas", re.compile(r"cublas", re.IGNORECASE)),
    ("gemm", re.compile(r"\bgemm\b", re.IGNORECASE)),
)
# Tensor-core fragment intrinsics: reported, never gated.
TENSOR_CORE_PATTERN = re.compile(r"wmma|mma_sync", re.IGNORECASE)


def strip_comments_and_strings(text: str) -> str:
    """`text` with comments and plain string literals replaced by spaces.

    Length and line breaks are preserved, so a site's offset and line number in
    the result are the site's own. Raw strings are kept: they carry the CUDA
    kernel source, which is executable. Nested block comments are Rust's.
    """
    out = list(text)
    i = 0
    n = len(text)
    while i < n:
        c = text[i]
        if c == "/" and i + 1 < n and text[i + 1] == "/":
            j = text.find("\n", i)
            j = n if j < 0 else j
            for k in range(i, j):
                out[k] = " "
            i = j
        elif c == "/" and i + 1 < n and text[i + 1] == "*":
            depth = 0
            j = i
            while j < n:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                    if depth == 0:
                        break
                else:
                    j += 1
            for k in range(i, min(j, n)):
                if out[k] != "\n":
                    out[k] = " "
            i = min(j, n)
        elif c == "r" and _raw_string_at(text, i) is not None:
            i = _raw_string_at(text, i)
        elif c == '"':
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == '"':
                    break
                j += 1
            for k in range(i, min(j + 1, n)):
                if out[k] != "\n":
                    out[k] = " "
            i = min(j + 1, n)
        else:
            i += 1
    return "".join(out)


def _raw_string_at(text: str, i: int) -> int | None:
    """The offset just past a raw string starting at `i`, or None."""
    if text[i] != "r":
        return None
    j = i + 1
    hashes = 0
    while j < len(text) and text[j] == "#":
        hashes += 1
        j += 1
    if j >= len(text) or text[j] != '"':
        return None
    closing = '"' + "#" * hashes
    end = text.find(closing, j + 1)
    return len(text) if end < 0 else end + len(closing)


def scan() -> tuple[dict[str, int], dict[str, list[tuple[int, str, str]]], int, int]:
    """(sites per file, sites per file in detail, tensor-core count, its files)."""
    found: dict[str, int] = {}
    detail: dict[str, list[tuple[int, str, str]]] = {}
    tensor_core = 0
    tensor_files = 0
    for d in SCAN_DIRS:
        base = ROOT / d
        if not base.is_dir():
            continue
        for path in sorted(base.rglob("*")):
            if not path.is_file() or path.suffix not in SUFFIXES:
                continue
            if "target" in path.parts:
                continue
            try:
                text = path.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            rel = str(path.relative_to(ROOT))
            code = strip_comments_and_strings(text)
            hits: list[tuple[int, str, str]] = []
            for line_number, line in enumerate(code.split("\n"), 1):
                for kind, pattern in SITE_PATTERNS:
                    for match in pattern.finditer(line):
                        hits.append((line_number, kind, match.group(0)))
            if hits:
                found[rel] = len(hits)
                detail[rel] = hits
            if TENSOR_CORE_PATTERN.search(text):
                tensor_files += 1
                tensor_core += len(TENSOR_CORE_PATTERN.findall(text))
    return found, detail, tensor_core, tensor_files


def load_baseline() -> dict[str, int]:
    if not BASELINE.is_file():
        return {}
    return json.loads(BASELINE.read_text(encoding="utf-8"))


def print_sites(detail: dict[str, list[tuple[int, str, str]]]) -> None:
    print("dense matrix product call sites in code (D25):")
    for path, hits in sorted(detail.items()):
        for line_number, kind, text in hits:
            print(f"  {path}:{line_number}: [{kind}] {text}")
    print(f"  {sum(len(v) for v in detail.values())} sites in {len(detail)} files")


def main() -> int:
    ap = argparse.ArgumentParser(description="D25 no-matmul ratchet")
    ap.add_argument("--update-baseline", action="store_true")
    ap.add_argument("--list", action="store_true", help="print every site, for audit")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()

    current, detail, tensor_core, tensor_files = scan()
    total = sum(current.values())

    # The audit list is always available, before any verdict.
    if args.list:
        print_sites(detail)

    if args.update_baseline:
        old = load_baseline()
        # First run sets the ratchet; afterwards it may only be lowered.
        raised = (
            {p: (old.get(p, 0), n) for p, n in current.items() if n > old.get(p, 0)}
            if old
            else {}
        )
        if not old:
            print("first run: setting the ratchet at the current counts")
        if raised:
            print("REFUSED: --update-baseline may only lower counts. Raised:", file=sys.stderr)
            for p, (o, n) in sorted(raised.items()):
                print(f"  {p}: {o} -> {n}", file=sys.stderr)
            print(
                "Remove the new dense product, or justify it as an owner decision first.",
                file=sys.stderr,
            )
            return 1
        BASELINE.write_text(json.dumps(current, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"baseline written: {len(current)} files, {total} sites")
        return 0

    if not BASELINE.is_file():
        print("FAIL: no baseline. Run --update-baseline once to set the ratchet.", file=sys.stderr)
        return 1
    # An empty baseline is the ratchet at zero: the end state. D23's script reads
    # `{}` as "no baseline"; here it means "no site may be added", which is the
    # same verdict for every site that appears.
    baseline = load_baseline()

    growth: list[str] = []
    for path, n in sorted(current.items()):
        base = baseline.get(path, 0)
        if n > base:
            growth.append(f"  {path}: {base} -> {n} (+{n - base})")

    removed = sum(max(0, baseline.get(p, 0) - current.get(p, 0)) for p in baseline)
    base_total = sum(baseline.values())

    if growth:
        print(
            "FAIL: dense matrix product call sites added to executable source (D25 prohibits "
            "them, and a geometric wrapper that calls one is the same violation):",
            file=sys.stderr,
        )
        print("\n".join(growth), file=sys.stderr)
        print(f"\ntotal {base_total} -> {total}. Replace it with a structured geometric", file=sys.stderr)
        print("product, or get an owner decision; do not raise the baseline.", file=sys.stderr)
        return 1

    if not args.quiet:
        print(
            f"OK: no dense matrix product call sites added. {total} remaining across "
            f"{len(current)} files (baseline {base_total})."
        )
        if removed:
            print(f"    {removed} removed since the baseline: run --update-baseline to lock it in.")
        print(
            f"    informational: {tensor_core} tensor-core intrinsic sites (wmma/mma_sync) across "
            f"{tensor_files} files — anticipated for structured geometric kernels, not gated."
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
