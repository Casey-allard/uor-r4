#!/usr/bin/env python3
"""Aggregate multi-window mode-admissibility evidence for the K1/T2 gate.

Inputs are tau-scan and cofinal-curve-fit JSON outputs produced by the
fixed_error_psi_* scripts. This does not prove admissibility; it summarizes
cross-window stability of the finite-window indicators.
"""

from __future__ import annotations

import argparse
import json
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List, Tuple


def _tau_key(tau: float) -> str:
    return f"{tau:.9f}"


@dataclass
class TauWindowSummary:
    tau: float
    scan_window_count: int
    fit_window_count: int
    delta_tri_all_positive: bool
    rr_all_below_a0: bool
    qinf_all_below_a0: bool
    rr_cofinal_min: float
    rr_cofinal_max: float
    delta_tri_min: float
    delta_tri_max: float
    qinf_min: float
    qinf_max: float
    windows_scan: Dict[str, Dict[str, float]]
    windows_fit: Dict[str, Dict[str, float]]


def _load_json(path: Path) -> Dict:
    return json.loads(path.read_text(encoding="utf-8"))


def _collect_scan_rows(path: Path) -> Tuple[str, float, Dict[str, Dict[str, float]]]:
    payload = _load_json(path)
    meta = payload.get("meta", {})
    label = f"x{int(meta.get('xmax', 0))}"
    a0 = float(meta.get("abs_cos_min", 0.98))
    rows: Dict[str, Dict[str, float]] = {}
    for row in payload.get("rows", []):
        tau = float(row["tau"])
        rows[_tau_key(tau)] = {
            "tau": tau,
            "rr": float(row["rratio_cofinal_grid"]),
            "delta_tri": float(row["delta_tri_cofinal_grid"]),
            "amp": float(row["amplitude"]),
            "aligned_n": float(row["aligned_count_tail"]),
        }
    return label, a0, rows


def _collect_fit_rows(path: Path) -> Tuple[str, float, Dict[str, Dict[str, float]]]:
    payload = _load_json(path)
    meta = payload.get("meta", {})
    label = f"x{int(meta.get('xmax', 0))}"
    a0 = float(meta.get("abs_cos_min", 0.98))
    rows: Dict[str, Dict[str, float]] = {}
    for row in payload.get("rows", []):
        tau = float(row["tau"])
        rows[_tau_key(tau)] = {
            "tau": tau,
            "qinf": float(row["q_inf_est"]),
            "rmse": float(row["rmse_fit"]),
            "aligned_n": float(row["aligned_count_tail"]),
        }
    return label, a0, rows


def main() -> None:
    ap = argparse.ArgumentParser(description="Multi-window mode-admissibility summary")
    ap.add_argument(
        "--scan-files",
        type=str,
        required=True,
        help="Comma-separated tau-scan JSON paths",
    )
    ap.add_argument(
        "--fit-files",
        type=str,
        required=True,
        help="Comma-separated cofinal-curve-fit JSON paths",
    )
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_mode_admissibility_three_window_summary_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    scan_paths = [Path(p.strip()) for p in args.scan_files.split(",") if p.strip()]
    fit_paths = [Path(p.strip()) for p in args.fit_files.split(",") if p.strip()]
    if not scan_paths or not fit_paths:
        raise ValueError("scan-files and fit-files must be non-empty")

    scan_by_window: Dict[str, Dict[str, Dict[str, float]]] = {}
    fit_by_window: Dict[str, Dict[str, Dict[str, float]]] = {}
    a0_scan = 0.98
    a0_fit = 0.98
    for p in scan_paths:
        label, a0, rows = _collect_scan_rows(p)
        scan_by_window[label] = rows
        a0_scan = a0
    for p in fit_paths:
        label, a0, rows = _collect_fit_rows(p)
        fit_by_window[label] = rows
        a0_fit = a0

    tau_keys = set()
    for rows in scan_by_window.values():
        tau_keys.update(rows.keys())
    for rows in fit_by_window.values():
        tau_keys.update(rows.keys())

    scan_labels = sorted(scan_by_window.keys())
    fit_labels = sorted(fit_by_window.keys())

    summaries: List[TauWindowSummary] = []
    for tk in sorted(tau_keys):
        scan_windows: Dict[str, Dict[str, float]] = {}
        fit_windows: Dict[str, Dict[str, float]] = {}
        tau_val = None

        rr_vals: List[float] = []
        delta_tri_vals: List[float] = []
        qinf_vals: List[float] = []

        for w in scan_labels:
            row = scan_by_window[w].get(tk)
            if row is None:
                continue
            tau_val = float(row["tau"])
            scan_windows[w] = row
            rr_vals.append(float(row["rr"]))
            delta_tri_vals.append(float(row["delta_tri"]))
        for w in fit_labels:
            row = fit_by_window[w].get(tk)
            if row is None:
                continue
            tau_val = float(row["tau"])
            fit_windows[w] = row
            qinf_vals.append(float(row["qinf"]))

        if tau_val is None:
            continue

        delta_tri_all_positive = bool(delta_tri_vals) and all(v > 0.0 for v in delta_tri_vals)
        rr_all_below_a0 = bool(rr_vals) and all(v < a0_scan for v in rr_vals)
        qinf_all_below_a0 = bool(qinf_vals) and all(v < a0_fit for v in qinf_vals)

        summaries.append(
            TauWindowSummary(
                tau=float(tau_val),
                scan_window_count=len(scan_windows),
                fit_window_count=len(fit_windows),
                delta_tri_all_positive=delta_tri_all_positive,
                rr_all_below_a0=rr_all_below_a0,
                qinf_all_below_a0=qinf_all_below_a0,
                rr_cofinal_min=min(rr_vals) if rr_vals else 0.0,
                rr_cofinal_max=max(rr_vals) if rr_vals else 0.0,
                delta_tri_min=min(delta_tri_vals) if delta_tri_vals else 0.0,
                delta_tri_max=max(delta_tri_vals) if delta_tri_vals else 0.0,
                qinf_min=min(qinf_vals) if qinf_vals else 0.0,
                qinf_max=max(qinf_vals) if qinf_vals else 0.0,
                windows_scan=scan_windows,
                windows_fit=fit_windows,
            )
        )

    robust_scan = [
        s.tau
        for s in summaries
        if s.scan_window_count == len(scan_labels)
        and s.delta_tri_all_positive
        and s.rr_all_below_a0
    ]
    robust_fit = [
        s.tau
        for s in summaries
        if s.fit_window_count == len(fit_labels) and s.qinf_all_below_a0
    ]
    robust_joint = sorted(set(robust_scan).intersection(robust_fit))

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "scan_files": [str(p) for p in scan_paths],
            "fit_files": [str(p) for p in fit_paths],
            "scan_windows": scan_labels,
            "fit_windows": fit_labels,
            "a0_scan": float(a0_scan),
            "a0_fit": float(a0_fit),
        },
        "summary_rows": [asdict(s) for s in summaries],
        "robust_sets": {
            "robust_scan_delta_tri_and_rr": robust_scan,
            "robust_fit_qinf": robust_fit,
            "robust_joint": robust_joint,
        },
        "interpretation": {
            "note": (
                "Finite-window robustness only. "
                "Joint set narrows candidate taus for theorem-grade admissibility work."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# K1 Mode-Admissibility Multi-Window Summary ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Scan windows: `{', '.join(scan_labels)}`")
    lines.append(f"- Fit windows: `{', '.join(fit_labels)}`")
    lines.append(f"- a0 (scan): `{a0_scan}`")
    lines.append(f"- a0 (fit): `{a0_fit}`")
    lines.append("")
    lines.append("## Robust Sets")
    lines.append("")
    lines.append(
        "- Robust scan set (`delta_tri>0` and `rr<a0` in all scan windows): "
        + (", ".join(f"{v:.9f}" for v in robust_scan) if robust_scan else "none")
    )
    lines.append(
        "- Robust fit set (`q_inf<a0` in all fit windows): "
        + (", ".join(f"{v:.9f}" for v in robust_fit) if robust_fit else "none")
    )
    lines.append(
        "- Robust joint set: "
        + (", ".join(f"{v:.9f}" for v in robust_joint) if robust_joint else "none")
    )
    lines.append("")
    lines.append("## Tau Table")
    lines.append("")
    lines.append("| tau | scan_n | fit_n | rr_min | rr_max | delta_tri_min | qinf_min | qinf_max | scan_robust | fit_robust |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for s in sorted(summaries, key=lambda z: z.tau):
        scan_ok = s.scan_window_count == len(scan_labels) and s.delta_tri_all_positive and s.rr_all_below_a0
        fit_ok = s.fit_window_count == len(fit_labels) and s.qinf_all_below_a0
        lines.append(
            f"| {s.tau:.9f} | {s.scan_window_count} | {s.fit_window_count} | "
            f"{s.rr_cofinal_min:.6f} | {s.rr_cofinal_max:.6f} | {s.delta_tri_min:.6f} | "
            f"{s.qinf_min:.6f} | {s.qinf_max:.6f} | {str(scan_ok).lower()} | {str(fit_ok).lower()} |"
        )

    lines.append("")
    lines.append("Finite-window robustness report only; not a proof of mode admissibility.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
