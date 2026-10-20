#!/usr/bin/env python3
"""Prime-label permutation probe on spinning-top R^4 manifold channels."""

from __future__ import annotations

import argparse
import json
import math
import random
from datetime import datetime, timezone
from typing import Dict, List, Sequence

import spinning_top_r4_candidate_a as m
from prime_geometry_loop import load_zeta_zeros_file, sieve_primes


def label_perm_control(series: Sequence[float], labels: Sequence[int], trials: int, seed: int) -> Dict[str, float]:
    pvals = [x for x, y in zip(series, labels) if y == 1]
    cvals = [x for x, y in zip(series, labels) if y == 0]
    if len(pvals) < 8 or len(cvals) < 8:
        return {"p": 1.0, "z": 0.0, "eff": 0.0}
    obs = abs((sum(pvals) / len(pvals)) - (sum(cvals) / len(cvals)))
    rng = random.Random(seed)
    y = list(labels)
    null = []
    for _ in range(trials):
        rng.shuffle(y)
        pp = [x for x, t in zip(series, y) if t == 1]
        cc = [x for x, t in zip(series, y) if t == 0]
        val = abs((sum(pp) / len(pp)) - (sum(cc) / len(cc)))
        null.append(val)
    mean = sum(null) / len(null)
    var = sum((v - mean) ** 2 for v in null) / len(null)
    std = math.sqrt(var)
    ge = sum(1 for v in null if v >= obs)
    p = (ge + 1.0) / (len(null) + 1.0)
    z = (obs - mean) / max(1e-12, std)
    eff = 1.0 - 2.0 * p
    return {"p": p, "z": z, "eff": eff}


def gate(rows: Sequence[Dict[str, object]], subset: Sequence[str]) -> Dict[str, float]:
    p_raw = [sum(r["channels"][c]["p"] for c in subset) / len(subset) for r in rows]
    z_raw = [sum(r["channels"][c]["z"] for c in subset) / len(subset) for r in rows]
    e_raw = [sum(r["channels"][c]["eff"] for c in subset) / len(subset) for r in rows]
    pw = m.winsorize(p_raw)
    zw = m.winsorize(z_raw)
    ew = m.winsorize(e_raw)
    mp = m.median(pw)
    mz = m.median(zw)
    me = m.median(ew)
    zv = sum((x - mz) ** 2 for x in zw) / max(1, len(zw))
    ev = sum((x - me) ** 2 for x in ew) / max(1, len(ew))
    signs = [1 if x > 0 else (-1 if x < 0 else 0) for x in zw]
    sc = sum(1 for s in signs if s == signs[0]) / max(1, len(signs))
    pg = sum(1 for p in pw if p <= 0.2) / max(1, len(pw))
    ok = sc >= 0.66 and zv <= 25.0 and pg >= 0.5 and ev <= 0.25
    robust = (1.0 - mp) * max(0.0, mz) / (1.0 + math.sqrt(zv))
    rank = (1.0 - mp) * max(0.0, me) / (1.0 + math.sqrt(ev))
    return {"gate": ok, "robust": robust, "rank": rank, "p_good": pg, "z_var": zv}


def main() -> None:
    ap = argparse.ArgumentParser(description="R4 manifold prime-label probe")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="60000,100000,150000")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--perm-trials", type=int, default=20)
    ap.add_argument("--subset", type=str, default="Y11c,curvature,su2_comm")
    ap.add_argument("--output", type=str, default="research/output/spinning_top_r4_prime_label_probe.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = [int(x.strip()) for x in args.n_values.split(",") if x.strip()]
    subset = [x.strip() for x in args.subset.split(",") if x.strip()]
    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros

    max_n = max(n_values)
    pts = m.manifold_points(max_n, zeros, args.u_mode)
    is_prime = sieve_primes(max_n)
    base_idx = {b: [n for n in range(2, max_n + 1) if m.gcd(n, b) == 1] for b in bases}

    per_base = []
    for b in bases:
        rows = []
        for n in n_values:
            idx = [k for k in base_idx[b] if k <= n]
            ch = m.candidate_series_for_base(pts[: n - 1], idx)
            L = min(len(ch[c]) for c in ch)
            # Align labels to channel length.
            aligned_idx = idx[:L]
            labels = [1 if is_prime[k] else 0 for k in aligned_idx]
            chs = {}
            for i, c in enumerate(ch):
                chs[c] = label_perm_control(ch[c][:L], labels, args.perm_trials, 20260901 + b + n + i * 13)
            rows.append({"n_max": n, "channels": chs})
        met = gate(rows, subset)
        per_base.append({"wheel_base": b, "rows": rows, **met})

    gpass = all(r["gate"] for r in per_base)
    transfer = min(r["rank"] for r in per_base) + 0.2 * min(r["robust"] for r in per_base) + (0.5 if gpass else 0.0)
    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": n_values,
            "u_mode": args.u_mode,
            "perm_trials": args.perm_trials,
            "subset": subset,
        },
        "per_base": per_base,
        "global": {"pass": gpass, "transfer_score": transfer},
    }
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Spinning-Top R4 Prime-Label Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"Subset: {','.join(subset)}\n\n")
        f.write(f"Global pass: {gpass}\n")
        f.write(f"Transfer score: {transfer:.6f}\n\n")
        for r in per_base:
            f.write(
                f"- base={r['wheel_base']} pass={r['gate']} robust={r['robust']:.6f} "
                f"rank={r['rank']:.6f} p_good={r['p_good']:.3f} z_var={r['z_var']:.6f}\n"
            )
    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    print("global_pass:", gpass)
    print("transfer_score:", round(transfer, 6))


if __name__ == "__main__":
    main()
