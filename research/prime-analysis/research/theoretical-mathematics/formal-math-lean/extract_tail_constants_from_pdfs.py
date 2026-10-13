#!/usr/bin/env python3
"""Extract theorem and constant snippets from local RH-related PDFs."""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from datetime import date
from pathlib import Path
from typing import Iterable

from pypdf import PdfReader


DOC_TAGS = {
    "1912.00853.pdf": "Schlage-Puchta 2019",
    "2204.02588.pdf": "Fiori-Kadiri-Swidinsky 2022/2023",
    "2405.12545.pdf": "Bellotti 2024",
    "2411.13791.pdf": "Johnston 2024/2025",
    "2508.02041.pdf": "Bellotti 2025",
}

KEYWORDS = [
    "theorem",
    "corollary",
    "lemma",
    "n(",
    "n(\sigma",
    "zero",
    "density",
    "psi(x)",
    "psi(x) - x",
    "\psi(x)",
    "x)",
    "log",
    "oscillat",
    "right-half",
    "rho",
    "beta",
    "sigma",
]

FORMULA_HINTS = [
    "<=",
    ">=",
    "=",
    "<",
    ">",
    "^",
    "exp",
    "sqrt",
    "theta",
    "omega",
]

THEOREM_RE = re.compile(r"^(theorem|corollary|lemma)\b", re.IGNORECASE)


@dataclass
class Snippet:
    score: int
    page: int
    line: int
    text: str
    context_before: list[str]
    context_after: list[str]


def normalize_line(text: str) -> str:
    return re.sub(r"\s+", " ", text.strip())


def score_line(line: str) -> int:
    s = line.lower()
    score = 0
    for kw in KEYWORDS:
        if kw in s:
            score += 2
    if THEOREM_RE.match(s):
        score += 6
    if any(tok in s for tok in FORMULA_HINTS):
        score += 1
    if re.search(r"n\s*\(\s*\\?sigma\s*,\s*t\s*\)", s):
        score += 6
    if "psi" in s and "x" in s:
        score += 3
    return score


def dedupe(snippets: Iterable[Snippet], max_items: int = 80) -> list[Snippet]:
    out: list[Snippet] = []
    seen: set[str] = set()
    for sn in sorted(snippets, key=lambda z: (-z.score, z.page, z.line)):
        key = sn.text.lower()
        if key in seen:
            continue
        seen.add(key)
        out.append(sn)
        if len(out) >= max_items:
            break
    return out


def extract_doc(pdf_path: Path, context: int = 2) -> dict:
    reader = PdfReader(str(pdf_path))
    all_snippets: list[Snippet] = []

    for p_idx, page in enumerate(reader.pages, start=1):
        text = page.extract_text() or ""
        lines = [normalize_line(x) for x in text.splitlines()]
        lines = [x for x in lines if x]
        for l_idx, line in enumerate(lines, start=1):
            score = score_line(line)
            if score < 4:
                continue
            i0 = max(0, l_idx - 1 - context)
            i1 = min(len(lines), l_idx + context)
            before = lines[i0 : l_idx - 1]
            after = lines[l_idx:i1]
            all_snippets.append(
                Snippet(
                    score=score,
                    page=p_idx,
                    line=l_idx,
                    text=line,
                    context_before=before,
                    context_after=after,
                )
            )

    best = dedupe(all_snippets)
    return {
        "pdf": str(pdf_path),
        "paper": DOC_TAGS.get(pdf_path.name, pdf_path.name),
        "snippets": [
            {
                "score": sn.score,
                "page": sn.page,
                "line": sn.line,
                "text": sn.text,
                "context_before": sn.context_before,
                "context_after": sn.context_after,
            }
            for sn in best
        ],
    }


def make_markdown(data: list[dict], out_path: Path) -> None:
    lines: list[str] = []
    lines.append(f"# K1 L2 External Constants Extract ({date.today().isoformat()})")
    lines.append("")
    lines.append("Target: collect theorem-level snippets for explicit zero-density and oscillation inputs.")
    lines.append("")
    for doc in data:
        lines.append(f"## {doc['paper']}")
        lines.append(f"Source: `{doc['pdf']}`")
        lines.append("")
        snippets = doc["snippets"][:25]
        if not snippets:
            lines.append("No high-score snippets extracted.")
            lines.append("")
            continue
        for sn in snippets:
            lines.append(
                f"- p.{sn['page']} l.{sn['line']} (score {sn['score']}): {sn['text']}"
            )
            if sn["context_before"]:
                lines.append(
                    "  context_before: "
                    + " | ".join(sn["context_before"][-2:])
                )
            if sn["context_after"]:
                lines.append(
                    "  context_after: "
                    + " | ".join(sn["context_after"][:2])
                )
        lines.append("")

    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--pdf-dir",
        default="research/external/papers",
        help="Directory containing PDF sources.",
    )
    parser.add_argument(
        "--out-prefix",
        default=f"research/output/k1_l2_external_constants_extract_{date.today().isoformat()}",
        help="Output path prefix for json/md artifacts (without extension).",
    )
    args = parser.parse_args()

    pdf_dir = Path(args.pdf_dir)
    pdfs = sorted(pdf_dir.glob("*.pdf"))
    if not pdfs:
        raise SystemExit(f"No PDFs found in {pdf_dir}")

    records = [extract_doc(pdf) for pdf in pdfs]

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)

    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")

    json_path.write_text(json.dumps(records, indent=2), encoding="utf-8")
    make_markdown(records, md_path)

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
