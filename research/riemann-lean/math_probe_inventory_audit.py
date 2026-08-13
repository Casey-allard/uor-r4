#!/usr/bin/env python3
"""Build a math-probe inventory to keep research context stable.

This script scans local research probes and classifies them by topic and
dependency signals (prime/zeta/explicit-formula), then emits a compact
inventory report.
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List


KEYWORD_TOPICS: Dict[str, str] = {
    "spinning_top": "r4_spinning_top",
    "r6_": "r6_tail_or_majorant",
    "tau12": "tau12_anchor_rotation",
    "tau14": "tau14_tau21_phase_gate",
    "lie_phase": "lie_harmonic",
    "harmonic": "lie_harmonic",
    "phase_square": "phase_square",
    "divisor_wheel": "wheel_geometry",
    "pair_correlation": "pair_correlation",
    "explicit_formula": "explicit_formula",
    "fixed_error_psi": "psi_normalized_error",
    "k1_": "k1_final_component",
    "hx_": "hx_bridge_or_tail",
    "lemma_": "lemma_probe",
}


@dataclass
class ProbeRow:
    path: str
    topic: str
    uses_prime_signal: bool
    uses_zeta_signal: bool
    uses_explicit_formula_signal: bool
    uses_zeros_file_signal: bool
    dependency_class: str
    matching_outputs_count: int
    recent_outputs: List[str]


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description="Research probe inventory audit")
    ap.add_argument("--research-dir", type=str, default="research")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/math_probe_inventory_audit_{date.today().isoformat()}",
    )
    ap.add_argument("--recent-limit", type=int, default=3)
    return ap.parse_args()


def guess_topic(name: str) -> str:
    lower = name.lower()
    for key, topic in KEYWORD_TOPICS.items():
        if key in lower:
            return topic
    return "general"


def classify_dependency(
    uses_prime: bool,
    uses_zeta: bool,
    uses_explicit_formula: bool,
) -> str:
    if uses_prime and uses_zeta:
        return "prime_and_zeta"
    if uses_prime:
        return "prime_only"
    if uses_zeta:
        return "zeta_or_zero_only"
    if uses_explicit_formula:
        return "explicit_formula_only"
    return "independent_or_generic"


def explicit_dependency_intent(text: str) -> str | None:
    m = re.search(r'dependency_intent\s*=\s*"([^"]+)"', text)
    if not m:
        return None
    intent = m.group(1).strip()
    if intent in {"prime_and_zeta", "prime_only", "zeta_or_zero_only", "explicit_formula_only", "independent_or_generic"}:
        return intent
    return None


def stem_candidates(path: Path) -> List[str]:
    stem = path.stem.lower()
    out = [stem]
    if stem.endswith("_probe"):
        out.append(stem[:-6])
    if stem.endswith("_checker"):
        out.append(stem[:-8])
    if stem.endswith("_summary"):
        out.append(stem[:-8])
    return sorted(set(out))


def find_outputs(output_dir: Path, script_path: Path, recent_limit: int) -> List[Path]:
    cands = stem_candidates(script_path)
    matches: List[Path] = []
    for p in output_dir.iterdir():
        if not p.is_file():
            continue
        if p.suffix.lower() not in {".json", ".md"}:
            continue
        name = p.name.lower()
        if any(name.startswith(c) or f"_{c}_" in name for c in cands):
            matches.append(p)
    matches.sort(key=lambda p: p.stat().st_mtime, reverse=True)
    return matches[: max(0, recent_limit)]


def main() -> None:
    args = parse_args()
    research_dir = Path(args.research_dir)
    output_dir = research_dir / "output"
    if not research_dir.exists():
        raise FileNotFoundError(f"research dir not found: {research_dir}")
    if not output_dir.exists():
        raise FileNotFoundError(f"output dir not found: {output_dir}")

    probe_files = sorted(p for p in research_dir.glob("*.py") if "probe" in p.stem.lower())
    rows: List[ProbeRow] = []

    for p in probe_files:
        text = p.read_text(encoding="utf-8", errors="ignore").lower()
        prime_hits = len(re.findall(r"\bprime(s)?\b", text))
        prime_neg_hits = len(re.findall(r"\b(no|without)\s+prime(s)?\b", text))
        uses_prime = prime_hits > prime_neg_hits

        zeta_hits = len(re.findall(r"\bzeta\b|\bzero(s)?\b", text))
        zeta_neg_hits = len(re.findall(r"\b(no|without)\s+zeta\b|\b(no|without)\s+zeta[- ]?zero(s)?\b", text))
        uses_zeta = zeta_hits > zeta_neg_hits
        uses_explicit_formula = bool(re.search(r"explicit[_ -]?formula|von mangoldt|chebyshev", text))
        uses_zeros_file = "zeta_zeros" in text or "odlyzko" in text or "lmfdb" in text

        recent = find_outputs(output_dir=output_dir, script_path=p, recent_limit=int(args.recent_limit))
        dep_cls = classify_dependency(
            uses_prime=uses_prime,
            uses_zeta=uses_zeta or uses_zeros_file,
            uses_explicit_formula=uses_explicit_formula,
        )
        dep_cls = explicit_dependency_intent(text) or dep_cls

        rows.append(
            ProbeRow(
                path=str(p),
                topic=guess_topic(p.name),
                uses_prime_signal=uses_prime,
                uses_zeta_signal=uses_zeta,
                uses_explicit_formula_signal=uses_explicit_formula,
                uses_zeros_file_signal=uses_zeros_file,
                dependency_class=dep_cls,
                matching_outputs_count=len(recent),
                recent_outputs=[str(x) for x in recent],
            )
        )

    topic_counts: Dict[str, int] = {}
    dep_counts: Dict[str, int] = {}
    for r in rows:
        topic_counts[r.topic] = topic_counts.get(r.topic, 0) + 1
        dep_counts[r.dependency_class] = dep_counts.get(r.dependency_class, 0) + 1

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "research_dir": str(research_dir),
            "probe_count": len(rows),
            "recent_limit": int(args.recent_limit),
        },
        "topic_counts": topic_counts,
        "dependency_class_counts": dep_counts,
        "rows": [asdict(r) for r in rows],
        "interpretation": {
            "note": (
                "Inventory classification is lexical and intended for navigation/context control. "
                "Use script-specific outputs for theorem-grade claims."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Math Probe Inventory Audit ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- `probe_count={len(rows)}`")
    lines.append("")
    lines.append("## Dependency Classes")
    for k in sorted(dep_counts):
        lines.append(f"- `{k}`: {dep_counts[k]}")
    lines.append("")
    lines.append("## Topics")
    for k in sorted(topic_counts):
        lines.append(f"- `{k}`: {topic_counts[k]}")
    lines.append("")
    lines.append("## Probe Rows")
    lines.append("| probe | topic | dependency | recent outputs |")
    lines.append("|---|---|---|---:|")
    for r in rows:
        lines.append(
            f"| `{Path(r.path).name}` | `{r.topic}` | `{r.dependency_class}` | {r.matching_outputs_count} |"
        )
    lines.append("")
    lines.append("Lexical inventory only; verify critical claims directly from source outputs.")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
