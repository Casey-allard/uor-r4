#!/usr/bin/env python3
"""Channel ablation for R4 zeta-guardrail conjecture transfer stability."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import random
from datetime import datetime, timezone
from typing import Dict, List, Sequence

from prime_geometry_loop import dft_powers, embed_4d, load_zeta_zeros_file, parse_moduli, zeta_permutation_control


CHANNELS = [
    "precession",
    "nutation",
    "wobble",
    "wobble_banded",
    "rho_drift",
    "rho_curvature",
    "square_local_wobble",
    "phase_rho_coupling",
]
CACHE_VERSION = "channel_ablation_v2"


def sieve_primes(n_max: int) -> List[int]:
    is_prime = [False, False] + [True] * (n_max - 1)
    for p in range(2, int(n_max**0.5) + 1):
        if is_prime[p]:
            is_prime[p * p : n_max + 1 : p] = [False] * (((n_max - p * p) // p) + 1)
    return [i for i in range(2, n_max + 1) if is_prime[i]]


def unwrap_delta(a: float, b: float) -> float:
    d = b - a
    while d > math.pi:
        d -= 2 * math.pi
    while d < -math.pi:
        d += 2 * math.pi
    return d


def phase_series(primes: Sequence[int], moduli: Sequence[int], radius_power: float) -> Dict[str, List[float]]:
    phi1 = []
    phi2 = []
    rho = []
    for p in primes:
        x, y, z, t = embed_4d(p, moduli, radius_power)
        phi1.append(math.atan2(y, x))
        phi2.append(math.atan2(t, z))
        rho.append(math.sqrt(x * x + y * y + z * z + t * t))

    pre = []
    nut = []
    for i in range(len(primes) - 1):
        d1 = unwrap_delta(phi1[i], phi1[i + 1])
        d2 = unwrap_delta(phi2[i], phi2[i + 1])
        pre.append(0.5 * (d1 + d2))
        nut.append(0.5 * (d1 - d2))

    wob = [nut[i + 1] - nut[i] for i in range(len(nut) - 1)]
    rho_d = [rho[i + 1] - rho[i] for i in range(len(rho) - 1)]
    rho_c = [rho_d[i + 1] - rho_d[i] for i in range(len(rho_d) - 1)]
    sqw = []
    for i in range(len(wob)):
        n_ref = primes[i + 1]
        s = int(math.isqrt(n_ref))
        lower = s * s
        upper = (s + 1) * (s + 1)
        span = max(1, upper - lower)
        u = (n_ref - lower) / span
        boundary = min(u, 1.0 - u)
        gate = math.exp(-((boundary / 0.2) ** 2))
        sqw.append(wob[i] * gate)
    prc = [pre[i] * rho_d[i] for i in range(min(len(pre), len(rho_d)))]

    return {
        "precession": pre,
        "nutation": nut,
        "wobble": wob,
        "rho_drift": rho_d,
        "rho_curvature": rho_c,
        "square_local_wobble": sqw,
        "phase_rho_coupling": prc,
    }


def phase_series_wheel_base(primes: Sequence[int], wheel_base: int, radius_power: float) -> Dict[str, List[float]]:
    phi1 = []
    phi2 = []
    rho = []
    for p in primes:
        r = p**radius_power
        theta = (2.0 * math.pi * (p % wheel_base)) / wheel_base
        x = r * math.cos(theta)
        y = r * math.sin(theta)
        z = r * math.cos(3.0 * theta)
        t = r * math.sin(3.0 * theta)
        phi1.append(math.atan2(y, x))
        phi2.append(math.atan2(t, z))
        rho.append(math.sqrt(x * x + y * y + z * z + t * t))

    pre = []
    nut = []
    for i in range(len(primes) - 1):
        d1 = unwrap_delta(phi1[i], phi1[i + 1])
        d2 = unwrap_delta(phi2[i], phi2[i + 1])
        pre.append(0.5 * (d1 + d2))
        nut.append(0.5 * (d1 - d2))

    wob = [nut[i + 1] - nut[i] for i in range(len(nut) - 1)]
    rho_d = [rho[i + 1] - rho[i] for i in range(len(rho) - 1)]
    rho_c = [rho_d[i + 1] - rho_d[i] for i in range(len(rho_d) - 1)]
    sqw = []
    for i in range(len(wob)):
        n_ref = primes[i + 1]
        s = int(math.isqrt(n_ref))
        lower = s * s
        upper = (s + 1) * (s + 1)
        span = max(1, upper - lower)
        u = (n_ref - lower) / span
        boundary = min(u, 1.0 - u)
        gate = math.exp(-((boundary / 0.2) ** 2))
        sqw.append(wob[i] * gate)
    prc = [pre[i] * rho_d[i] for i in range(min(len(pre), len(rho_d)))]

    return {
        "precession": pre,
        "nutation": nut,
        "wobble": wob,
        "rho_drift": rho_d,
        "rho_curvature": rho_c,
        "square_local_wobble": sqw,
        "phase_rho_coupling": prc,
    }


def zeta_spectral_score_banded(series: Sequence[float], zeros: Sequence[float], frac_min: float, frac_max: float) -> float:
    if len(series) < 64 or not zeros:
        return 0.0
    n = len(series)
    m = max(zeros)
    raw_targets = [0.5 + 0.45 * (z / m) * n for z in zeros]
    lo = max(0.0, frac_min) * n
    hi = min(0.5, max(frac_min, frac_max)) * n
    targets = [f for f in raw_targets if lo <= f <= hi]
    if not targets:
        return 0.0
    tp = dft_powers(series, targets)
    base = [1 + i for i in range(min(64, n // 2))]
    base = [f for f in base if lo <= f <= hi] or [1 + i for i in range(min(64, n // 2))]
    bp = dft_powers(series, base)
    b = sum(bp) / max(1, len(bp))
    return (sum(tp) / max(1, len(tp))) / max(1e-12, b)


def banded_perm_control(series: Sequence[float], zeros: Sequence[float], trials: int, seed: int, frac_min: float, frac_max: float):
    if trials <= 0:
        return {"p_value_ge": 1.0, "z_score": 0.0, "signed_effect": 0.0}
    rng = random.Random(seed)
    obs = zeta_spectral_score_banded(series, zeros, frac_min, frac_max)
    vals = []
    s = list(series)
    for _ in range(trials):
        rng.shuffle(s)
        vals.append(zeta_spectral_score_banded(s, zeros, frac_min, frac_max))
    mean = sum(vals) / len(vals)
    var = sum((v - mean) ** 2 for v in vals) / len(vals)
    std = math.sqrt(var)
    ge = sum(1 for v in vals if v >= obs)
    p = (ge + 1.0) / (len(vals) + 1.0)
    z = (obs - mean) / max(1e-12, std)
    le = sum(1 for v in vals if v <= obs)
    percentile = le / max(1, len(vals))
    signed = 2.0 * percentile - 1.0
    return {"p_value_ge": p, "z_score": z, "signed_effect": signed}


def clip(x: float, lo: float = -25.0, hi: float = 25.0) -> float:
    return min(hi, max(lo, x))


def median(values: Sequence[float]) -> float:
    if not values:
        return 0.0
    v = sorted(values)
    n = len(v)
    m = n // 2
    return v[m] if n % 2 == 1 else 0.5 * (v[m - 1] + v[m])


def quantile(values: Sequence[float], q: float) -> float:
    if not values:
        return 0.0
    v = sorted(values)
    i = int(max(0, min(len(v) - 1, round(q * (len(v) - 1)))))
    return v[i]


def winsorize(values: Sequence[float], q_low: float = 0.1, q_high: float = 0.9) -> List[float]:
    if not values:
        return []
    lo = quantile(values, q_low)
    hi = quantile(values, q_high)
    return [min(hi, max(lo, x)) for x in values]


def load_cache(path: str) -> Dict[str, object]:
    if not path or not os.path.exists(path):
        return {}
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else {}
    except Exception:
        return {}


def save_cache(path: str, cache: Dict[str, object]) -> None:
    if not path:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(cache, f)


def zeros_signature(zeros: Sequence[float]) -> str:
    text = ",".join(f"{z:.8f}" for z in zeros)
    return hashlib.sha1(text.encode("utf-8")).hexdigest()[:16]


def cache_key(payload: Dict[str, object]) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


def compute_family_metrics(
    moduli: Sequence[int],
    radius_power: float,
    primes_by_n: Dict[int, Sequence[int]],
    zeros: Sequence[float],
    perm_trials: int,
    frac_min: float,
    frac_max: float,
    seed_base: int,
    wheel_base: int = 0,
    cache: Dict[str, object] | None = None,
    zsig: str = "",
) -> Dict[str, object]:
    per_n = []
    cache_store = cache if cache is not None else {}
    n_values = sorted(primes_by_n.keys())
    for i, n in enumerate(n_values):
        key = cache_key(
            {
                "v": CACHE_VERSION,
                "n": n,
                "moduli": list(moduli),
                "wheel_base": wheel_base,
                "radius_power": radius_power,
                "perm_trials": perm_trials,
                "frac_min": frac_min,
                "frac_max": frac_max,
                "seed_base": seed_base,
                "zeros_sig": zsig,
            }
        )
        if key in cache_store:
            per_n.append(cache_store[key])
            continue

        primes = primes_by_n[n]
        s = phase_series_wheel_base(primes, wheel_base, radius_power) if wheel_base > 1 else phase_series(primes, moduli, radius_power)

        ch = {}
        for k in [
            "precession",
            "nutation",
            "wobble",
            "rho_drift",
            "rho_curvature",
            "square_local_wobble",
            "phase_rho_coupling",
        ]:
            ctrl = zeta_permutation_control(s[k], perm_trials, seed_base + i * 10 + len(k), zeros_imag=zeros)
            ch[k] = {
                "p": ctrl.get("p_value_ge", 1.0),
                "z": clip(ctrl.get("z_score", 0.0)),
                "eff": 1.0 - 2.0 * ctrl.get("p_value_ge", 1.0),
            }

        wb = banded_perm_control(s["wobble"], zeros, perm_trials, seed_base + i * 10 + 777, frac_min, frac_max)
        ch["wobble_banded"] = {"p": wb["p_value_ge"], "z": clip(wb["z_score"]), "eff": wb["signed_effect"]}
        row = {"n_max": n, "channels": ch}
        cache_store[key] = row
        per_n.append(row)

    return {
        "moduli": list(moduli),
        "wheel_base": wheel_base,
        "moduli_label": "-".join(str(x) for x in moduli),
        "radius_power": radius_power,
        "per_n": per_n,
    }


def score_subset(per_n: Sequence[Dict[str, object]], subset: Sequence[str]) -> Dict[str, float]:
    agg = []
    for row in per_n:
        ch = row["channels"]
        p_mean = sum(ch[c]["p"] for c in subset) / len(subset)
        z_mean = sum(ch[c]["z"] for c in subset) / len(subset)
        e_mean = sum(ch[c]["eff"] for c in subset) / len(subset)
        agg.append({"p_mean": p_mean, "z_mean": z_mean, "e_mean": e_mean})

    p_w = winsorize([a["p_mean"] for a in agg])
    z_w = winsorize([a["z_mean"] for a in agg])
    e_w = winsorize([a["e_mean"] for a in agg])

    med_p = median(p_w)
    med_z = median(z_w)
    med_e = median(e_w)
    z_var = sum((z - med_z) ** 2 for z in z_w) / max(1, len(z_w))
    e_var = sum((e - med_e) ** 2 for e in e_w) / max(1, len(e_w))
    signs = [1 if z > 0 else (-1 if z < 0 else 0) for z in z_w]
    sign_consistency = sum(1 for s in signs if s == signs[0]) / max(1, len(signs))
    p_good = sum(1 for p in p_w if p <= 0.2) / max(1, len(p_w))
    gate = sign_consistency >= 0.66 and z_var <= 25.0 and p_good >= 0.5 and e_var <= 0.25

    return {
        "support_rank": (1.0 - med_p) * max(0.0, med_e) / (1.0 + math.sqrt(e_var)),
        "support_robust": (1.0 - med_p) * max(0.0, med_z) / (1.0 + math.sqrt(z_var)),
        "med_p": med_p,
        "med_e": med_e,
        "med_z": med_z,
        "z_var": z_var,
        "e_var": e_var,
        "p_good_frac": p_good,
        "sign_consistency": sign_consistency,
        "gate": gate,
    }


def main():
    ap = argparse.ArgumentParser(description="Channel ablation probe for conjecture transfer")
    ap.add_argument("--n-values", type=str, default="80000,100000,120000,150000,200000,250000")
    ap.add_argument("--family-a", type=str, default="5,7,11,13,19")
    ap.add_argument("--family-b", type=str, default="5,7,11,13,17")
    ap.add_argument("--family-a-base", type=int, default=0)
    ap.add_argument("--family-b-base", type=int, default=0)
    ap.add_argument("--radius-power", type=float, default=1.1)
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--zeta-perm-trials", type=int, default=8)
    ap.add_argument("--freq-frac-min", type=float, default=0.05)
    ap.add_argument("--freq-frac-max", type=float, default=0.30)
    ap.add_argument("--cache-file", type=str, default="research/output/cache/channel_ablation_cache.json")
    ap.add_argument("--output", type=str, default="research/output/channel_ablation_probe.json")
    args = ap.parse_args()

    n_values = [int(x) for x in args.n_values.split(",") if x.strip()]
    fam_a = parse_moduli(args.family_a)
    fam_b = parse_moduli(args.family_b)
    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros
    zsig = zeros_signature(zeros)
    primes_by_n = {n: sieve_primes(n) for n in n_values}
    cache = load_cache(args.cache_file)

    fa = compute_family_metrics(
        fam_a,
        args.radius_power,
        primes_by_n,
        zeros,
        args.zeta_perm_trials,
        args.freq_frac_min,
        args.freq_frac_max,
        20260216,
        args.family_a_base,
        cache,
        zsig,
    )
    fb = compute_family_metrics(
        fam_b,
        args.radius_power,
        primes_by_n,
        zeros,
        args.zeta_perm_trials,
        args.freq_frac_min,
        args.freq_frac_max,
        20261216,
        args.family_b_base,
        cache,
        zsig,
    )
    save_cache(args.cache_file, cache)

    subsets = {
        "full": CHANNELS,
        "phase_only": ["precession", "nutation", "wobble", "wobble_banded"],
        "phase_plus_square_local": ["precession", "nutation", "wobble", "wobble_banded", "square_local_wobble"],
        "no_wobble_banded": [c for c in CHANNELS if c != "wobble_banded"],
        "no_square_local_wobble": [c for c in CHANNELS if c != "square_local_wobble"],
        "no_rho_drift": [c for c in CHANNELS if c != "rho_drift"],
        "no_rho_curvature": [c for c in CHANNELS if c != "rho_curvature"],
        "no_phase_rho_coupling": [c for c in CHANNELS if c != "phase_rho_coupling"],
        "no_phase_rho_plus_square": [
            "precession",
            "nutation",
            "wobble",
            "wobble_banded",
            "rho_drift",
            "rho_curvature",
            "square_local_wobble",
        ],
        "radial_only": ["rho_drift", "rho_curvature", "square_local_wobble", "phase_rho_coupling"],
    }

    rows = []
    for name, subset in subsets.items():
        sa = score_subset(fa["per_n"], subset)
        sb = score_subset(fb["per_n"], subset)
        transfer_rank = min(sa["support_rank"], sb["support_rank"])
        transfer_robust = min(sa["support_robust"], sb["support_robust"])
        transfer_gate = sa["gate"] and sb["gate"]
        rows.append(
            {
                "subset_name": name,
                "channels": subset,
                "family_a": sa,
                "family_b": sb,
                "transfer_rank_min": transfer_rank,
                "transfer_robust_min": transfer_robust,
                "transfer_gate": transfer_gate,
                "rank_score": transfer_rank + 0.2 * transfer_robust + (0.5 if transfer_gate else 0.0),
            }
        )

    rows.sort(key=lambda r: r["rank_score"], reverse=True)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_values": n_values,
            "family_a": fam_a,
            "family_b": fam_b,
            "family_a_base": args.family_a_base,
            "family_b_base": args.family_b_base,
            "radius_power": args.radius_power,
            "zeta_perm_trials": args.zeta_perm_trials,
            "freq_frac_min": args.freq_frac_min,
            "freq_frac_max": args.freq_frac_max,
        },
        "family_metrics": [fa, fb],
        "ablations": rows,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Channel Ablation Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        for i, r in enumerate(rows[:8], 1):
            f.write(
                f"{i}. {r['subset_name']} | transfer_rank_min={r['transfer_rank_min']:.6f} "
                f"transfer_robust_min={r['transfer_robust_min']:.6f} gate={r['transfer_gate']}\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    if rows:
        b = rows[0]
        print("best_subset", b["subset_name"], "transfer_rank_min", round(b["transfer_rank_min"], 6), "gate", b["transfer_gate"])


if __name__ == "__main__":
    main()
