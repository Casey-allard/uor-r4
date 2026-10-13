#!/usr/bin/env python3
"""Numerical certificate helper for rho = tau2/tau1 non-integer separation."""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path


@dataclass
class RatioCertificate:
    tau1: float
    tau2: float
    rho: float
    floor_rho: int
    ceil_rho: int
    dist_to_floor: float
    dist_to_ceil: float
    nearest_integer: int
    dist_to_nearest_integer: float
    in_open_interval_1_2: bool


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau ratio non-integer certificate")
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_tau_ratio_noninteger_certificate_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.tau1 <= 0.0 or args.tau2 <= 0.0:
        raise ValueError("taus must be positive")

    rho = float(args.tau2 / args.tau1)
    fl = math.floor(rho)
    ce = math.ceil(rho)
    dist_fl = float(rho - fl)
    dist_ce = float(ce - rho)
    nearest = int(round(rho))
    dist_nearest = float(abs(rho - nearest))
    cert = RatioCertificate(
        tau1=float(args.tau1),
        tau2=float(args.tau2),
        rho=rho,
        floor_rho=int(fl),
        ceil_rho=int(ce),
        dist_to_floor=dist_fl,
        dist_to_ceil=dist_ce,
        nearest_integer=nearest,
        dist_to_nearest_integer=dist_nearest,
        in_open_interval_1_2=bool(1.0 < rho < 2.0),
    )

    payload = {
        "meta": {"date": date.today().isoformat()},
        "certificate": asdict(cert),
        "interpretation": {
            "note": (
                "Numeric separation only. Useful as a practical guard that rho is far from any integer."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines = [
        f"# Tau Ratio Non-Integer Certificate ({date.today().isoformat()})",
        "",
        f"- `tau1 = {cert.tau1:.9f}`",
        f"- `tau2 = {cert.tau2:.9f}`",
        f"- `rho = tau2/tau1 = {cert.rho:.12f}`",
        f"- `rho in (1,2): {str(cert.in_open_interval_1_2).lower()}`",
        f"- `nearest integer = {cert.nearest_integer}`",
        f"- `dist_to_nearest_integer = {cert.dist_to_nearest_integer:.12f}`",
        "",
        "Numeric separation only; not a formal proof by itself.",
    ]
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
