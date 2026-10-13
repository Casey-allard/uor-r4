#!/usr/bin/env python3
"""Explicit eventual-threshold check for tau1-anchor rounding preservation.

For anchor x*_k solving tau1*log(x*)+phi1=2*pi*k and n_k=round(x*_k), use
  |log n_k - log x*_k| <= 0.5 / (x*_k - 0.5)
to derive phase-error controls.

For c0>0, if cos2(anchor) >= c0 and
  tau2 * 0.5/(x*_k-0.5) <= c0 * margin_factor
then cos2(integer) >= 0 by |cos(u+v)-cos u| <= |v|.

Also require
  tau1 * 0.5/(x*_k-0.5) <= arccos(a1)
to force |cos1(integer)| >= a1.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List

import numpy as np

from fixed_error_psi_tau14_tau21_constructive_gate_probe import _phase_anchors


def parse_float_list(text: str) -> List[float]:
    vals: List[float] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(float(tok))
    if not vals:
        raise ValueError("empty float list")
    return vals


def parse_path_list(text: str) -> List[Path]:
    vals: List[Path] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(Path(tok))
    if not vals:
        raise ValueError("empty path list")
    return vals


@dataclass
class WindowResult:
    window_label: str
    x_max: int
    anchors_total: int
    anchors_buffered: int
    anchors_checked_ge_x0: int
    fail_sign_count: int
    fail_cos1_count: int
    fail_either_count: int
    max_abs_d1_checked: float
    max_abs_d2_checked: float
    min_cos2_anchor_checked: float
    min_cos2_integer_checked: float
    min_abs_cos1_integer_checked: float


@dataclass
class C0Result:
    c0: float
    x0_threshold: float | None
    delta1_allow: float
    delta2_allow: float | None
    windows: List[WindowResult]
    total_checked: int
    total_fail_either: int
    all_windows_zero_failures: bool


def _anchor_center(k: int, tau1: float, phi1: float) -> float:
    return math.exp((2.0 * math.pi * float(k) - phi1) / tau1)


def _threshold_x(
    tau1: float,
    tau2: float,
    a1: float,
    c0: float,
    margin_factor: float,
) -> tuple[float | None, float, float | None]:
    # |delta1| <= arccos(a1) ensures cos(delta1) >= a1.
    d1_allow = float(math.acos(a1))
    if d1_allow <= 0.0:
        return None, d1_allow, None
    x1 = 0.5 + 0.5 * tau1 / d1_allow

    # Need c0>0 to force sign via Lipschitz.
    if c0 <= 0.0:
        return None, d1_allow, None
    d2_allow = float(c0 * margin_factor)
    if d2_allow <= 0.0:
        return None, d1_allow, d2_allow
    x2 = 0.5 + 0.5 * tau2 / d2_allow
    return float(max(x1, x2)), d1_allow, d2_allow


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau12 rounding eventual-threshold checker")
    ap.add_argument(
        "--fit-files",
        type=str,
        required=True,
        help="Comma-separated constructive-gate JSON files with fit {phi1,phi2}",
    )
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--a1", type=float, default=0.98)
    ap.add_argument("--c0-list", type=str, default="0.1,0.2,0.3,0.4,0.5")
    ap.add_argument(
        "--margin-factor",
        type=float,
        default=1.0,
        help="require |delta2| <= margin_factor*c0 for sign preservation",
    )
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_tau12_rounding_eventual_threshold_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.tau1 <= 0.0 or args.tau2 <= 0.0:
        raise ValueError("taus must be positive")
    if not (0.0 < args.a1 < 1.0):
        raise ValueError("a1 must be in (0,1)")
    if args.margin_factor <= 0.0:
        raise ValueError("margin-factor must be positive")

    fit_files = parse_path_list(args.fit_files)
    c0_list = sorted(set(parse_float_list(args.c0_list)))

    fit_payloads: List[Dict] = []
    for p in fit_files:
        d = json.loads(p.read_text(encoding="utf-8"))
        if "fit" not in d or "meta" not in d:
            raise ValueError(f"missing fit/meta in {p}")
        fit_payloads.append({"path": str(p), "data": d})

    c0_results: List[C0Result] = []
    for c0 in c0_list:
        x0, d1_allow, d2_allow = _threshold_x(
            tau1=float(args.tau1),
            tau2=float(args.tau2),
            a1=float(args.a1),
            c0=float(c0),
            margin_factor=float(args.margin_factor),
        )

        window_rows: List[WindowResult] = []
        total_checked = 0
        total_fail = 0

        for it in fit_payloads:
            d = it["data"]
            meta = d["meta"]
            fit = d["fit"]
            xmin = int(meta["xmin"])
            xmax = int(meta["xmax"])
            phi1 = float(fit["phi1"])
            phi2 = float(fit["phi2"])
            anchors = _phase_anchors(float(args.tau1), phi1, xmin, xmax)

            buffered = 0
            checked = 0
            fail_sign = 0
            fail_cos1 = 0
            fail_either = 0

            d1_vals: List[float] = []
            d2_vals: List[float] = []
            cos2_anchor_vals: List[float] = []
            cos2_int_vals: List[float] = []
            abs_cos1_int_vals: List[float] = []

            for k in anchors:
                x_star = _anchor_center(k, float(args.tau1), phi1)
                theta_anchor = float(args.tau2) * math.log(x_star) + phi2
                cos2_anchor = math.cos(theta_anchor)
                if cos2_anchor < float(c0):
                    continue
                buffered += 1
                if x0 is None or x_star < x0:
                    continue

                n = int(round(x_star))
                n = max(xmin, min(xmax, n))
                nf = float(max(2, n))
                dlog = math.log(nf) - math.log(x_star)
                d1 = float(args.tau1) * dlog
                d2 = float(args.tau2) * dlog
                cos2_int = math.cos(float(args.tau2) * math.log(nf) + phi2)
                abs_cos1_int = abs(math.cos(float(args.tau1) * math.log(nf) + phi1))
                sign_ok = cos2_int >= 0.0
                cos1_ok = abs_cos1_int >= float(args.a1)

                checked += 1
                if not sign_ok:
                    fail_sign += 1
                if not cos1_ok:
                    fail_cos1 += 1
                if (not sign_ok) or (not cos1_ok):
                    fail_either += 1

                d1_vals.append(abs(d1))
                d2_vals.append(abs(d2))
                cos2_anchor_vals.append(float(cos2_anchor))
                cos2_int_vals.append(float(cos2_int))
                abs_cos1_int_vals.append(float(abs_cos1_int))

            total_checked += checked
            total_fail += fail_either

            window_rows.append(
                WindowResult(
                    window_label=f"x{xmax}",
                    x_max=xmax,
                    anchors_total=len(anchors),
                    anchors_buffered=buffered,
                    anchors_checked_ge_x0=checked,
                    fail_sign_count=fail_sign,
                    fail_cos1_count=fail_cos1,
                    fail_either_count=fail_either,
                    max_abs_d1_checked=max(d1_vals) if d1_vals else 0.0,
                    max_abs_d2_checked=max(d2_vals) if d2_vals else 0.0,
                    min_cos2_anchor_checked=min(cos2_anchor_vals) if cos2_anchor_vals else 0.0,
                    min_cos2_integer_checked=min(cos2_int_vals) if cos2_int_vals else 0.0,
                    min_abs_cos1_integer_checked=min(abs_cos1_int_vals) if abs_cos1_int_vals else 0.0,
                )
            )

        c0_results.append(
            C0Result(
                c0=float(c0),
                x0_threshold=x0,
                delta1_allow=d1_allow,
                delta2_allow=d2_allow,
                windows=window_rows,
                total_checked=total_checked,
                total_fail_either=total_fail,
                all_windows_zero_failures=bool(total_checked > 0 and total_fail == 0),
            )
        )

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "fit_files": [it["path"] for it in fit_payloads],
            "tau1": float(args.tau1),
            "tau2": float(args.tau2),
            "a1": float(args.a1),
            "c0_list": c0_list,
            "margin_factor": float(args.margin_factor),
        },
        "results": [asdict(r) for r in c0_results],
        "interpretation": {
            "note": (
                "Finite-window threshold verification only. "
                "Intended to guide theorem formulation for eventual rounding-preservation."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau12 Rounding Eventual-Threshold Check ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- `tau1={args.tau1}`, `tau2={args.tau2}`, `a1={args.a1}`")
    lines.append(f"- `margin_factor={args.margin_factor}`")
    lines.append("")
    lines.append("| c0 | X0 | checked_total | fail_total | zero_failures |")
    lines.append("|---:|---:|---:|---:|---:|")
    for r in c0_results:
        x0s = "NA" if r.x0_threshold is None else f"{r.x0_threshold:.6f}"
        lines.append(
            f"| {r.c0:.3f} | {x0s} | {r.total_checked} | {r.total_fail_either} | {str(r.all_windows_zero_failures).lower()} |"
        )
    lines.append("")
    lines.append("Finite-window report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
