#!/usr/bin/env python3
"""A3 quadratic-energy majorant via B^2 <= sum g_i^2."""

from __future__ import annotations

import argparse
import bisect
import concurrent.futures
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence

import hx_bridge_probe as hx
import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


WEIGHTS = (-1.0, -1.0, 0.0, -1.0)


def parse_ints(raw: str) -> List[int]:
    return [int(x.strip()) for x in raw.split(",") if x.strip()]


def cache_key(payload: Dict[str, object]) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


def zeros_sig(zeros: Sequence[float]) -> str:
    txt = ",".join(f"{z:.8f}" for z in zeros)
    return hashlib.sha1(txt.encode("utf-8")).hexdigest()[:16]


def load_cache(path: str):
    np = s4.np
    if np is None or not os.path.exists(path):
        return None
    try:
        d = np.load(path)
        if "seq_n" not in d or "g_vals" not in d:
            return None
        return d["seq_n"].astype(np.int64).tolist(), d["g_vals"].astype(np.float64).tolist()
    except Exception:
        return None


def save_cache(path: str, seq_n: Sequence[int], g_vals: Sequence[float]) -> None:
    np = s4.np
    if np is None:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    np.savez_compressed(path, seq_n=np.asarray(seq_n, dtype=np.int64), g_vals=np.asarray(g_vals, dtype=np.float64))


def prime_factors(n: int) -> List[int]:
    x = n
    out: List[int] = []
    p = 2
    while p * p <= x:
        if x % p == 0:
            out.append(p)
            while x % p == 0:
                x //= p
        p += 1 if p == 2 else 2
    if x > 1:
        out.append(x)
    return out


def phi(n: int) -> int:
    r = n
    for p in prime_factors(n):
        r = (r // p) * (p - 1)
    return r


def fit_log_envelope(
    rows: Sequence[Dict[str, float]],
    train_n: Sequence[int],
    valid_n: Sequence[int],
    target_key: str,
    a_min: float,
    a_max: float,
    a_step: float,
    safety_factor: float,
):
    tset = set(int(v) for v in train_n)
    vset = set(int(v) for v in valid_n)
    train_rows = [r for r in rows if int(r["n_max"]) in tset]
    valid_rows = [r for r in rows if int(r["n_max"]) in vset]
    if not train_rows or not valid_rows:
        raise ValueError("empty split for fit")

    max_x_all = max(float(r["x"]) for r in rows) if rows else 3.0
    best = None
    a = a_min
    while a <= a_max + 1e-12:
        train_scaled = []
        for r in train_rows:
            den = max(1e-30, math.log(max(3.0, float(r["x"]))) ** a)
            train_scaled.append(float(r[target_key]) / den)
        c_train = max(train_scaled) if train_scaled else 0.0
        c_u = c_train * max(1.0, safety_factor)

        r_train = 0.0
        r_valid = 0.0
        for r in train_rows:
            rhs = c_u * (math.log(max(3.0, float(r["x"]))) ** a)
            r_train = max(r_train, float(r[target_key]) / max(1e-30, rhs))
        for r in valid_rows:
            rhs = c_u * (math.log(max(3.0, float(r["x"]))) ** a)
            r_valid = max(r_valid, float(r[target_key]) / max(1e-30, rhs))

        holds = r_train <= 1.0 + 1e-12 and r_valid <= 1.0 + 1e-12
        rhs_at_max_x = c_u * (math.log(max(3.0, max_x_all)) ** a)
        cand = {
            "A": float(a),
            "C_train_max": float(c_train),
            "C_uplifted": float(c_u),
            "ratio_max_train": float(r_train),
            "ratio_max_valid": float(r_valid),
            "rhs_at_max_x": float(rhs_at_max_x),
            "holds": bool(holds),
        }
        if holds:
            if best is None:
                best = cand
            elif (cand["rhs_at_max_x"] < best["rhs_at_max_x"]) or (
                cand["rhs_at_max_x"] == best["rhs_at_max_x"] and cand["A"] < best["A"]
            ):
                best = cand
        a += a_step
    if best is None:
        raise ValueError("no held-out-valid envelope candidate found")
    return best


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 quadratic-energy majorant")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--train-n-values", type=str, default="300000,1000000")
    ap.add_argument("--valid-n-values", type=str, default="2000000,5000000")
    ap.add_argument("--x-step", type=int, default=5000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--m-zero", type=int, default=128)
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--a-min", type=float, default=0.0)
    ap.add_argument("--a-max", type=float, default=10.0)
    ap.add_argument("--a-step", type=float, default=0.1)
    ap.add_argument("--safety-factor-e", type=float, default=1.1)
    ap.add_argument(
        "--fit-target",
        type=str,
        default="direct_h_energy",
        choices=["energy_then_transfer", "direct_h_energy"],
    )
    ap.add_argument("--density-mode", type=str, default="analytic", choices=["analytic", "sampled"])
    ap.add_argument("--density-safety", type=float, default=1.0)
    ap.add_argument("--cache-dir", type=str, default="research/cache/a3_quadratic_energy_majorant")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a3_quadratic_energy_majorant.json")
    args = ap.parse_args()

    bases = sorted(set(parse_ints(args.bases)))
    train_n = sorted(set(parse_ints(args.train_n_values)))
    valid_n = sorted(set(parse_ints(args.valid_n_values)))
    all_n = sorted(set(train_n + valid_n))
    if not all_n:
        raise ValueError("need non-empty n grid")

    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.m_zero > len(zeros_all):
        raise ValueError("m-zero exceeds available zero data")
    zeros = zeros_all[: args.m_zero]
    zsig = zeros_sig(zeros)

    x0 = max(2, args.x_step)
    max_n = max(all_n)
    xs_max = list(range(x0, max_n + 1, args.x_step))
    x_map = {}
    for n in all_n:
        k = ((n - x0) // args.x_step) + 1 if n >= x0 else 0
        k = max(0, min(k, len(xs_max)))
        x_map[n] = xs_max[:k]

    zero_terms = hx.prepare_zero_terms(zeros, args.zero_kernel, args.kernel_scale)
    use_cache = not args.no_cache
    event_stride = max(1, args.event_stride)
    cache_hits = 0

    def build_base_series(b: int):
        nonlocal cache_hits
        key = cache_key(
            {
                "version": 1,
                "base": b,
                "max_n": max_n,
                "u_mode": args.u_mode,
                "zero_kernel": args.zero_kernel,
                "kernel_scale": args.kernel_scale,
                "chunk_n": args.chunk_n,
                "event_stride": event_stride,
                "event_scale": bool(args.event_scale),
                "weights": WEIGHTS,
                "zsig": zsig,
            }
        )
        cp = os.path.join(args.cache_dir, f"{key}.npz")
        cached = load_cache(cp) if use_cache else None
        if cached is not None:
            cache_hits += 1
            return b, cached[0], cached[1]
        seq_n, g_vals = hx.stream_weighted_events(
            n_max=max_n,
            base=b,
            zeros=zeros,
            u_mode=args.u_mode,
            zero_kernel=args.zero_kernel,
            kernel_scale=args.kernel_scale,
            weights=WEIGHTS,
            chunk_n=args.chunk_n,
            zero_terms=zero_terms,
            event_stride=event_stride,
            event_scale=args.event_scale,
        )
        if use_cache:
            save_cache(cp, seq_n, g_vals)
        return b, seq_n, g_vals

    series_by_base = {}
    if args.jobs > 1 and len(bases) > 1:
        with concurrent.futures.ThreadPoolExecutor(max_workers=min(args.jobs, len(bases))) as ex:
            futs = [ex.submit(build_base_series, b) for b in bases]
            for fut in concurrent.futures.as_completed(futs):
                b, seq_n, g_vals = fut.result()
                series_by_base[b] = (seq_n, g_vals)
    else:
        for b in bases:
            bb, seq_n, g_vals = build_base_series(b)
            series_by_base[bb] = (seq_n, g_vals)

    rows = []
    rho_values = []
    for b in bases:
        seq_n, g_vals = series_by_base[b]
        pref_s = []
        pref_e2 = []
        a1 = 0.0
        a2 = 0.0
        for g in g_vals:
            a1 += g
            a2 += g * g
            pref_s.append(a1)
            pref_e2.append(a2)
        for n in all_n:
            for x in x_map[n]:
                i = bisect.bisect_right(seq_n, x) - 1
                if i < 0:
                    continue
                k = i + 1
                s = pref_s[i]
                e2 = pref_e2[i]
                b_norm = abs(s) / math.sqrt(max(1.0, float(k)))
                b_energy = math.sqrt(max(0.0, e2))
                h_abs = abs(s) / math.sqrt(max(1.0, float(x)))
                dens = k / max(1.0, float(x))
                h_energy_upper = b_energy * math.sqrt(max(0.0, dens))
                rows.append(
                    {
                        "base": b,
                        "n_max": n,
                        "x": x,
                        "k": k,
                        "b_norm": b_norm,
                        "b_energy": b_energy,
                        "h_abs": h_abs,
                        "dens": dens,
                        "h_energy_upper": h_energy_upper,
                        "det_gap_b_minus_energy": b_norm - b_energy,
                        "det_gap_h_minus_h_energy": h_abs - h_energy_upper,
                    }
                )
                rho_values.append(dens)

    fit_target_key = "h_energy_upper" if args.fit_target == "direct_h_energy" else "b_energy"
    e_model = fit_log_envelope(
        rows=rows,
        train_n=train_n,
        valid_n=valid_n,
        target_key=fit_target_key,
        a_min=args.a_min,
        a_max=args.a_max,
        a_step=args.a_step,
        safety_factor=args.safety_factor_e,
    )
    a_h = float(e_model["A"])
    c_e = float(e_model["C_uplifted"])

    rho_sup = max(rho_values) if rho_values else 1.0
    if args.density_mode == "analytic":
        x0_bound = float(min(all_n))
        per_base_analytic = []
        rho_guard = 0.0
        for b in bases:
            ph = float(phi(int(b)))
            rho_b = min(1.0, (ph / float(b)) + (ph / max(1.0, x0_bound)))
            per_base_analytic.append(
                {
                    "base": int(b),
                    "phi_W": int(ph),
                    "phi_over_W": float(ph / float(b)),
                    "rho_upper_x_ge_x0": float(rho_b),
                }
            )
            rho_guard = max(rho_guard, rho_b)
    else:
        per_base_analytic = []
        rho_guard = min(1.0, max(0.0, rho_sup * max(1.0, args.density_safety)))
    density_factor = math.sqrt(rho_guard)
    if args.fit_target == "direct_h_energy":
        c_h = c_e
        transfer_identity = "|H| <= sqrt((sum g_i^2)*(k/x)) <= C_H(log x)^A_H"
    else:
        c_h = c_e * density_factor
        transfer_identity = "|H| <= sqrt(sum g_i^2)*sqrt(k/x), sqrt(sum g_i^2)<=C_E(log x)^A_E"

    tset = set(train_n)
    vset = set(valid_n)

    def check_split(ns: set):
        v = 0
        gap = -1e18
        ratios = []
        det_v = 0
        det_gap = -1e18
        det_h_v = 0
        det_h_gap = -1e18
        for r in rows:
            if int(r["n_max"]) not in ns:
                continue
            det_gap = max(det_gap, float(r["det_gap_b_minus_energy"]))
            if float(r["det_gap_b_minus_energy"]) > 1e-12:
                det_v += 1
            det_h_gap = max(det_h_gap, float(r["det_gap_h_minus_h_energy"]))
            if float(r["det_gap_h_minus_h_energy"]) > 1e-12:
                det_h_v += 1
            rhs = c_h * (math.log(max(3.0, float(r["x"]))) ** a_h)
            g = float(r["h_abs"]) - rhs
            if g > 1e-12:
                v += 1
            gap = max(gap, g)
            ratios.append(float(r["h_abs"]) / max(1e-30, rhs))
        return {
            "holds": v == 0 and gap <= 1e-12,
            "violations": int(v),
            "max_gap_h_minus_rhs": float(gap),
            "ratio_min": min(ratios) if ratios else 0.0,
            "ratio_max": max(ratios) if ratios else 0.0,
            "deterministic_b_energy_holds": det_v == 0 and det_gap <= 1e-12,
            "deterministic_b_energy_violations": int(det_v),
            "deterministic_b_energy_max_gap": float(det_gap),
            "deterministic_h_energy_holds": det_h_v == 0 and det_h_gap <= 1e-12,
            "deterministic_h_energy_violations": int(det_h_v),
            "deterministic_h_energy_max_gap": float(det_h_gap),
        }

    train_chk = check_split(tset)
    valid_chk = check_split(vset)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "train_n_values": train_n,
            "valid_n_values": valid_n,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "weights": WEIGHTS,
            "m_zero": args.m_zero,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "chunk_n": args.chunk_n,
            "jobs": args.jobs,
            "event_stride": event_stride,
            "event_scale": bool(args.event_scale),
            "a_min": args.a_min,
            "a_max": args.a_max,
            "a_step": args.a_step,
            "safety_factor_e": max(1.0, args.safety_factor_e),
            "fit_target": args.fit_target,
            "density_mode": args.density_mode,
            "density_safety": max(1.0, args.density_safety),
            "cache_dir": args.cache_dir,
            "cache_enabled": use_cache,
            "cache_hits": cache_hits,
            "cache_misses": max(0, len(bases) - cache_hits),
        },
        "density_guardrail": {
            "rho_sup_sampled": float(rho_sup),
            "rho_guard": float(rho_guard),
            "sqrt_rho_guard": float(density_factor),
            "analytic_per_base": per_base_analytic,
        },
        "energy_envelope": {
            "A_E": a_h,
            "C_E_train_max": float(e_model["C_train_max"]),
            "C_E_uplifted": c_e,
            "ratio_max_train": float(e_model["ratio_max_train"]),
            "ratio_max_valid": float(e_model["ratio_max_valid"]),
        },
        "h_transfer_envelope": {
            "A_H": a_h,
            "C_H_from_density_transfer": c_h,
            "transfer_identity": transfer_identity,
        },
        "checks": {"train": train_chk, "valid": valid_chk},
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# A3 Quadratic-Energy Majorant\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        dg = report["density_guardrail"]
        ee = report["energy_envelope"]
        he = report["h_transfer_envelope"]
        f.write(f"- rho_guard: {dg['rho_guard']:.12g}\n")
        f.write(f"- sqrt_rho_guard: {dg['sqrt_rho_guard']:.12g}\n")
        f.write(f"- A_E: {ee['A_E']:.6f}\n")
        f.write(f"- C_E_uplifted: {ee['C_E_uplifted']:.12e}\n")
        f.write(f"- A_H (transfer): {he['A_H']:.6f}\n")
        f.write(f"- C_H (transfer): {he['C_H_from_density_transfer']:.12e}\n")
        f.write(f"- train holds: {report['checks']['train']['holds']}\n")
        f.write(f"- valid holds: {report['checks']['valid']['holds']}\n")
        f.write(f"- deterministic B<=sqrt(E2) holds(train): {report['checks']['train']['deterministic_b_energy_holds']}\n")
        f.write(f"- deterministic B<=sqrt(E2) holds(valid): {report['checks']['valid']['deterministic_b_energy_holds']}\n")
        f.write(f"- deterministic H<=sqrt(E2*k/x) holds(train): {report['checks']['train']['deterministic_h_energy_holds']}\n")
        f.write(f"- deterministic H<=sqrt(E2*k/x) holds(valid): {report['checks']['valid']['deterministic_h_energy_holds']}\n")
    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
