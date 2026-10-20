#!/usr/bin/env python3
"""Claim-wording gate (issue #123, docs/formal_vocabulary.md section 2.1).
RETIRED 2026-10-18: use `cargo run -p uor-r4-claim-wording` (Rust port, D27). Kept as dated evidence; no caller remains.

Scans normative Markdown (docs/**, crate and root READMEs) for prohibited claim
phrases: "machine-verified", "machine verified", "exact equivalence",
"exact teacher equivalence", "provably equivalent".

A line containing a prohibited phrase passes only when the same line also:
  - links a proof artifact or certificate (markdown link whose target mentions
    proof/cert, or an inline proof_matrix / PROOF.md / ProofStatus reference), or
  - explicitly disavows the claim (negation markers such as "no", "without",
    "prohibited", ...), or
  - carries the escape hatch "claim-wording: allow".

Exit 0 when clean, 1 with one diagnostic per violation otherwise.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

PROHIBITED = re.compile(
    r"machine[- ]verified"
    r"|exact(?:ly)?[- ](?:teacher[- ])?equivalen(?:ce|t)"
    r"|provably[- ]equivalent",
    re.IGNORECASE,
)

DISAVOWAL = re.compile(
    r"\bno\b|\bnot\b|\bnever\b|\bwithout\b|\bavoid\w*\b|\bprevent\w*\b"
    r"|\bprohibit\w*\b|\bforbid\w*\b|\bban(?:ned|s)?\b|\bdisavow\w*\b"
    r"|\bdo(?:es)?\s+not\b|\bdon't\b|\binformal\b",
    re.IGNORECASE,
)

PROOF_LINK = re.compile(
    r"\]\([^)]*(?:proof|cert)[^)]*\)"  # markdown link to a proof/cert artifact
    r"|proof_matrix|PROOF\.md|ProofStatus::Verified|certificate",
    re.IGNORECASE,
)

ALLOW_MARKER = "claim-wording: allow"


def candidate_files():
    files = sorted(ROOT.glob("docs/**/*.md"))
    files += sorted(ROOT.glob("crates/*/README.md"))
    files += sorted(ROOT.glob("README.md"))
    # Verbatim evidence archives (archive/history/github-archive-*) preserve
    # historical comments byte-identically; editing them to satisfy current
    # claim wording would falsify the record. The gate governs normative
    # documents, not preserved evidence.
    return [
        path
        for path in files
        if "archive/history/github-archive-" not in path.as_posix()
    ]


def main():
    violations = []
    for path in candidate_files():
        rel = path.relative_to(ROOT)
        try:
            lines = path.read_text(encoding="utf-8").splitlines()
        except UnicodeDecodeError:
            continue
        for lineno, line in enumerate(lines, 1):
            if not PROHIBITED.search(line):
                continue
            if (
                ALLOW_MARKER in line
                or DISAVOWAL.search(line)
                or PROOF_LINK.search(line)
            ):
                continue
            violations.append(f"{rel}:{lineno}: {line.strip()}")

    if violations:
        print("Claim-wording gate FAILED (docs/formal_vocabulary.md section 2.1):")
        print("Prohibited claim phrase without a linked proof artifact/certificate,")
        print("a same-line disavowal, or a 'claim-wording: allow' marker:\n")
        for v in violations:
            print(f"  {v}")
        sys.exit(1)

    print("Claim-wording gate passed: no unlinked machine-verified/exact-equivalence wording.")


if __name__ == "__main__":
    main()
