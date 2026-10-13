#!/usr/bin/env python3
"""Wheel-base v2 leaderboard with residue-native geometry and zeta guardrails."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence

from prime_geometry_loop import dft_powers, load_zeta_zeros_file, zeta_permutation_control


CHANNELS = [
    "precession",
    "nutation",
    "wobble",
    "wobble_banded",
    "square_local_wobble",
    "torsion",
]


def sieve_primes(n_max: int) -> List[int]:
    is_prime = [False, False] + [True] * (n_max - 1)
    for p in range(2, int(n_max**0.5) + 1):
        if is_prime[p]:
            is_prime[p * p : n_max + 1 : p] = [False] * (((n_max - p * p) // p) + 1)
    return [i for i in range(2, n_max + 1) if is_prime[i]]


def gcd(a: int, b: int) -> int:
    while b:
        a, b = b, a % b
    return a


def totatives(base: int) -> List[int]:
    return [r for r in range(1, base) if gcd(r, base) == 1]


def unwrap_delta(a: float, b: float) -> float:
    d = b - a
    while d > math.pi:
        d -= 2.0 * math.pi
    while d < -math.pi:
        d += 2.0 * math.pi
    return d


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


def banded_perm_control(
    series: Sequence[float],
    zeros: Sequence[float],
    trials: int,
    seed: int,
    frac_min: float,
    frac_max: float,
) -> Dict[str, float]:
    if trials <= 0:
        return {"p_value_ge": 1.0, "z_score": 0.0, "signed_effect": 0.0}
    import random

    rng = random.Random(seed)
    obs = zeta_spectral_score_banded(series, zeros, frac_min, frac_max)
    vals = []
    work = list(series)
    for _ in range(trials):
        rng.shuffle(work)
        vals.append(zeta_spectral_score_banded(work, zeros, frac_min, frac_max))
    mean = sum(vals) / max(1, len(vals))
    var = sum((v - mean) ** 2 for v in vals) / max(1, len(vals))
    std = math.sqrt(var)
    ge = sum(1 for v in vals if v >= obs)
    p = (ge + 1.0) / (len(vals) + 1.0)
    z = (obs - mean) / max(1e-12, std)
    le = sum(1 for v in vals if v <= obs)
    percentile = le / max(1, len(vals))
    signed_effect = 2.0 * percentile - 1.0
    return {"p_value_ge": p, "z_score": z, "signed_effect": signed_effect}


def clip(x: float, lo: float = -25.0, hi: float = 25.0) -> float:
    return min(hi, max(lo, x))


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


def median(values: Sequence[float]) -> float:
    if not values:
        return 0.0
    v = sorted(values)
    n = len(v)
    m = n // 2
    return v[m] if n % 2 == 1 else 0.5 * (v[m - 1] + v[m])


def cache_key(payload: Dict[str, object]) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


def zeros_signature(zeros: Sequence[float]) -> str:
    text = ",".join(f"{z:.8f}" for z in zeros)
    return hashlib.sha1(text.encode("utf-8")).hexdigest()[:16]


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


def wheel_series(primes: Sequence[int], base: int, radius_power: float) -> Dict[str, List[float]]:
    rs = totatives(base)
    idx = {r: i for i, r in enumerate(rs)}
    m = len(rs)

    filt = [p for p in primes if gcd(p, base) == 1]
    if len(filt) < 8:
        return {k: [] for k in CHANNELS}

    theta = []
    psi = []
    rho = []
    for i, p in enumerate(filt):
        r = p % base
        k = idx[r]
        t = 2.0 * math.pi * (k / m)
        gap = 0 if i == 0 else (filt[i] - filt[i - 1]) % base
        g = 2.0 * math.pi * (gap / base)
        theta.append(t)
        psi.append(g)
        # Keep RH-inspired radial law around exponent 1/2-style family.
        rho.append(p**radius_power)

    phi1 = theta
    phi2 = [((3.0 * t + g + math.pi) % (2.0 * math.pi)) - math.pi for t, g in zip(theta, psi)]

    pre = []
    nut = []
    for i in range(len(phi1) - 1):
        d1 = unwrap_delta(phi1[i], phi1[i + 1])
        d2 = unwrap_delta(phi2[i], phi2[i + 1])
        pre.append(0.5 * (d1 + d2))
        nut.append(0.5 * (d1 - d2))

    wob = [nut[i + 1] - nut[i] for i in range(len(nut) - 1)]
    torsion = [unwrap_delta(psi[i], psi[i + 1]) for i in range(len(psi) - 1)]

    sqw = []
    for i in range(len(wob)):
        n_ref = filt[i + 1]
        s = int(math.isqrt(n_ref))
        lo = s * s
        hi = (s + 1) * (s + 1)
        span = max(1, hi - lo)
        u = (n_ref - lo) / span
        b = min(u, 1.0 - u)
        gate = math.exp(-((b / 0.2) ** 2))
        sqw.append(wob[i] * gate)

    return {
        "precession": pre,
        "nutation": nut,
        "wobble": wob,
        "wobble_banded": wob,
        "square_local_wobble": sqw,
        "torsion": torsion,
    }


def score_base(
    base: int,
    n_values: Sequence[int],
    primes_by_n: Dict[int, Sequence[int]],
    radius_power: float,
    zeros: Sequence[float],
    perm_trials: int,
    frac_min: float,
    frac_max: float,
    cache: Dict[str, object],
    zsig: str,
) -> Dict[str, object]:
    rows = []
    for i, n in enumerate(n_values):
        key = cache_key(
            {
                "v": "wheel_v2",
                "base": base,
                "n": n,
                "radius_power": radius_power,
                "perm_trials": perm_trials,
                "frac_min": frac_min,
                "frac_max": frac_max,
                "zsig": zsig,
            }
        )
        if key in cache:
            rows.append(cache[key])
            continue

        s = wheel_series(primes_by_n[n], base, radius_power)
        ch = {}
        for c in ["precession", "nutation", "wobble", "square_local_wobble", "torsion"]:
            ctrl = zeta_permutation_control(s[c], perm_trials, 20260601 + base + i * 10 + len(c), zeros_imag=zeros)
            ch[c] = {
                "p": ctrl.get("p_value_ge", 1.0),
                "z": clip(ctrl.get("z_score", 0.0)),
                "eff": 1.0 - 2.0 * ctrl.get("p_value_ge", 1.0),
            }
        wb = banded_perm_control(s["wobble"], zeros, perm_trials, 20269900 + base + i * 10, frac_min, frac_max)
        ch["wobble_banded"] = {"p": wb["p_value_ge"], "z": clip(wb["z_score"]), "eff": wb["signed_effect"]}
        row = {"n_max": n, "channels": ch}
        cache[key] = row
        rows.append(row)

    p_raw = [sum(r["channels"][c]["p"] for c in CHANNELS) / len(CHANNELS) for r in rows]
    z_raw = [sum(r["channels"][c]["z"] for c in CHANNELS) / len(CHANNELS) for r in rows]
    e_raw = [sum(r["channels"][c]["eff"] for c in CHANNELS) / len(CHANNELS) for r in rows]

    p_w = winsorize(p_raw)
    z_w = winsorize(z_raw)
    e_w = winsorize(e_raw)
    med_p = median(p_w)
    med_z = median(z_w)
    med_e = median(e_w)
    z_var = sum((z - med_z) ** 2 for z in z_w) / max(1, len(z_w))
    e_var = sum((e - med_e) ** 2 for e in e_w) / max(1, len(e_w))
    signs = [1 if z > 0 else (-1 if z < 0 else 0) for z in z_w]
    sign_consistency = sum(1 for s in signs if s == signs[0]) / max(1, len(signs))
    p_good = sum(1 for p in p_w if p <= 0.2) / max(1, len(p_w))
    gate = sign_consistency >= 0.66 and z_var <= 25.0 and p_good >= 0.5 and e_var <= 0.25

    mean_p = sum(p_raw) / len(p_raw)
    mean_z = sum(z_raw) / len(z_raw)
    mean_e = sum(e_raw) / len(e_raw)
    return {
        "wheel_base": base,
        "channels": CHANNELS,
        "per_n": rows,
        "conjecture": {
            "guardrail_mean_p": mean_p,
            "guardrail_mean_z": mean_z,
            "guardrail_mean_effect": mean_e,
            "guardrail_median_p_winsor": med_p,
            "guardrail_median_z_winsor": med_z,
            "guardrail_median_effect_winsor": med_e,
            "guardrail_z_var_winsor": z_var,
            "guardrail_effect_var_winsor": e_var,
            "guardrail_sign_consistency": sign_consistency,
            "guardrail_p_good_fraction": p_good,
            "stability_gate_pass": gate,
            "support_score": (1.0 - mean_p) * max(0.0, mean_z),
            "support_score_robust": (1.0 - med_p) * max(0.0, med_z) / (1.0 + math.sqrt(z_var)),
            "support_score_rank": (1.0 - med_p) * max(0.0, med_e) / (1.0 + math.sqrt(e_var)),
        },
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Wheel-base v2 leaderboard")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="100000,150000,200000")
    ap.add_argument("--radius-power", type=float, default=1.1)
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--zeta-perm-trials", type=int, default=10)
    ap.add_argument("--freq-frac-min", type=float, default=0.05)
    ap.add_argument("--freq-frac-max", type=float, default=0.30)
    ap.add_argument("--cache-file", type=str, default="research/output/cache/wheel_v2_cache.json")
    ap.add_argument("--output", type=str, default="research/output/wheel_v2_leaderboard.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = [int(x.strip()) for x in args.n_values.split(",") if x.strip()]
    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros
    zsig = zeros_signature(zeros)
    primes_by_n = {n: sieve_primes(n) for n in n_values}
    cache = load_cache(args.cache_file)

    rows = []
    for base in bases:
        rows.append(
            score_base(
                base,
                n_values,
                primes_by_n,
                args.radius_power,
                zeros,
                args.zeta_perm_trials,
                args.freq_frac_min,
                args.freq_frac_max,
                cache,
                zsig,
            )
        )

    save_cache(args.cache_file, cache)
    rows.sort(
        key=lambda r: (
            r["conjecture"]["stability_gate_pass"],
            r["conjecture"]["support_score_robust"],
        ),
        reverse=True,
    )

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": n_values,
            "radius_power": args.radius_power,
            "zeta_perm_trials": args.zeta_perm_trials,
            "freq_frac_min": args.freq_frac_min,
            "freq_frac_max": args.freq_frac_max,
            "max_zeta_zeros": len(zeros),
        },
        "leaderboard": rows,
    }
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Wheel-Base V2 Leaderboard\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        for i, r in enumerate(rows, 1):
            c = r["conjecture"]
            f.write(
                f"{i}. base={r['wheel_base']} gate={c['stability_gate_pass']} "
                f"robust={c['support_score_robust']:.6f} rank={c['support_score_rank']:.6f} "
                f"mean_p={c['guardrail_mean_p']:.6f} mean_z={c['guardrail_mean_z']:.6f}\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    if rows:
        top = rows[0]
        c = top["conjecture"]
        print(
            "top_base:",
            top["wheel_base"],
            "gate:",
            c["stability_gate_pass"],
            "robust:",
            round(c["support_score_robust"], 6),
        )


if __name__ == "__main__":
    main()
