#!/usr/bin/env python3
"""Dedicated evaluator for Lie-Harmonic Candidate A.

Candidate A channels: Y11c, curvature, su2_comm
Outputs compact pass/fail and confidence per base plus global summary.
"""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence

import lie_phase_harmonic_probe as probe
from prime_geometry_loop import load_zeta_zeros_file


CANDIDATE_A = ("Y11c", "curvature", "su2_comm")


def gate_metrics(rows: Sequence[Dict[str, object]], channels: Sequence[str]) -> Dict[str, float]:
    p_raw = [sum(r["channels"][c]["p"] for c in channels) / len(channels) for r in rows]
    z_raw = [sum(r["channels"][c]["z"] for c in channels) / len(channels) for r in rows]
    e_raw = [sum(r["channels"][c]["eff"] for c in channels) / len(channels) for r in rows]

    p_w = probe.winsorize(p_raw)
    z_w = probe.winsorize(z_raw)
    e_w = probe.winsorize(e_raw)
    med_p = probe.median(p_w)
    med_z = probe.median(z_w)
    med_e = probe.median(e_w)
    z_var = sum((z - med_z) ** 2 for z in z_w) / max(1, len(z_w))
    e_var = sum((e - med_e) ** 2 for e in e_w) / max(1, len(e_w))
    signs = [1 if z > 0 else (-1 if z < 0 else 0) for z in z_w]
    sign_consistency = sum(1 for s in signs if s == signs[0]) / max(1, len(signs))
    p_good = sum(1 for p in p_w if p <= 0.2) / max(1, len(p_w))
    gate = sign_consistency >= 0.66 and z_var <= 25.0 and p_good >= 0.5 and e_var <= 0.25
    robust = (1.0 - med_p) * max(0.0, med_z) / (1.0 + math.sqrt(z_var))
    rank = (1.0 - med_p) * max(0.0, med_e) / (1.0 + math.sqrt(e_var))
    return {
        "gate": gate,
        "robust": robust,
        "rank": rank,
        "p_good": p_good,
        "z_var": z_var,
        "e_var": e_var,
        "sign_consistency": sign_consistency,
    }


def confidence_score(base_rows: Sequence[Dict[str, object]]) -> float:
    gate_frac = sum(1.0 for r in base_rows if r["gate"]) / max(1, len(base_rows))
    min_rank = min(r["rank"] for r in base_rows) if base_rows else 0.0
    min_robust = min(r["robust"] for r in base_rows) if base_rows else 0.0
    raw = 0.45 * gate_frac + 0.45 * min(1.0, min_rank) + 0.10 * min(1.0, min_robust / 10.0)
    return max(0.0, min(1.0, raw))


def main() -> None:
    ap = argparse.ArgumentParser(description="Candidate A compact evaluator")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="100000,150000,200000")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--zeta-perm-trials", type=int, default=20)
    ap.add_argument("--gauge-beta", type=float, default=0.75)
    ap.add_argument("--gauge-base", type=int, default=210)
    ap.add_argument("--regime-gamma", type=float, default=0.5)
    ap.add_argument("--regime-whiten", action="store_true")
    ap.add_argument("--regime-min-count", type=int, default=8)
    ap.add_argument("--max-control-points", type=int, default=2048)
    ap.add_argument("--control-max-zeros", type=int, default=64)
    ap.add_argument("--cache-file", type=str, default="research/output/cache/lie_harmonic_cache.json")
    ap.add_argument("--output", type=str, default="research/output/lie_candidate_a_compact_report.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = [int(x.strip()) for x in args.n_values.split(",") if x.strip()]
    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros
    zsig = probe.zeros_sig(zeros)
    primes_by_n = {n: probe.sieve_primes(n) for n in n_values}
    cache = probe.load_cache(args.cache_file)

    by_base = []
    for base in bases:
        scored = probe.score_base(
            base=base,
            n_values=n_values,
            primes_by_n=primes_by_n,
            zeros=zeros,
            perm_trials=args.zeta_perm_trials,
            cache=cache,
            zsig=zsig,
            gauge_beta=args.gauge_beta,
            gauge_base=args.gauge_base,
            regime_gamma=args.regime_gamma,
            regime_whiten_flag=args.regime_whiten,
            regime_min_count=args.regime_min_count,
            max_control_points=args.max_control_points,
            control_max_zeros=args.control_max_zeros,
        )
        m = gate_metrics(scored["per_n"], CANDIDATE_A)
        by_base.append({"wheel_base": base, **m})

    probe.save_cache(args.cache_file, cache)
    by_base.sort(key=lambda r: r["wheel_base"])
    global_gate = all(r["gate"] for r in by_base)
    conf = confidence_score(by_base)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "candidate": "Lie-Harmonic Candidate A",
        "channels": list(CANDIDATE_A),
        "config": {
            "bases": bases,
            "n_values": n_values,
            "zeta_perm_trials": args.zeta_perm_trials,
            "gauge_beta": args.gauge_beta,
            "gauge_base": args.gauge_base,
            "regime_gamma": args.regime_gamma,
            "regime_whiten": args.regime_whiten,
            "regime_min_count": args.regime_min_count,
            "max_control_points": args.max_control_points,
            "control_max_zeros": args.control_max_zeros,
            "max_zeta_zeros": len(zeros),
        },
        "per_base": by_base,
        "global": {
            "pass": global_gate,
            "confidence": conf,
            "message": "PASS" if global_gate else "FAIL",
        },
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Candidate A Compact Report\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"Global: `{report['global']['message']}` | confidence={report['global']['confidence']:.3f}\n\n")
        for row in by_base:
            f.write(
                f"- base={row['wheel_base']} pass={row['gate']} robust={row['robust']:.6f} "
                f"rank={row['rank']:.6f} p_good={row['p_good']:.3f} z_var={row['z_var']:.6f}\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    print("global_pass:", global_gate)
    print("confidence:", round(conf, 6))


if __name__ == "__main__":
    main()
