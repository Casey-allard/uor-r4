#!/usr/bin/env python3
"""Build conservative A1-A4 constants from train grid and validate on held-out grid."""

from __future__ import annotations

import argparse
import bisect
import concurrent.futures
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

import hx_bridge_probe as hx
import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


WEIGHTS = (-1.0, -1.0, 0.0, -1.0)


def parse_int_list(raw: str) -> List[int]:
    return [int(x.strip()) for x in raw.split(",") if x.strip()]


def zeros_sig(zeros: Sequence[float]) -> str:
    txt = ",".join(f"{z:.8f}" for z in zeros)
    return hashlib.sha1(txt.encode("utf-8")).hexdigest()[:16]


def cache_key(payload: Dict[str, object]) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


def load_cache(path: str):
    np = s4.np
    if np is None or not os.path.exists(path):
        return None
    try:
        d = np.load(path)
        if "h_max" not in d:
            return None
        return d["h_max"].astype(np.float64).tolist()
    except Exception:
        return None


def save_cache(path: str, h_max: Sequence[float]) -> None:
    np = s4.np
    if np is None:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    np.savez_compressed(path, h_max=np.asarray(h_max, dtype=np.float64))


def tau_tail_fp(zeros_ref: Sequence[float], m: int, kernel: str, scale: float) -> float:
    t = 0.0
    for g in zeros_ref[m:]:
        w = s4.zero_weight(g, kernel, scale)
        den = math.sqrt(0.25 + g * g)
        t += w * (g / den)
    return t


def theorem_rhs(
    x: int,
    a_ref: float,
    b_ref: float,
    c0: float,
    c_delta: float,
    tau_m: float,
    beta: float,
    c_h: float,
    a_h: float,
) -> float:
    lx = math.log(max(3, x))
    return (
        abs(a_ref) * c_h * (lx**a_h)
        + abs(b_ref)
        + c0
        + abs(a_ref) * c_delta * (lx**beta) * tau_m
    )


def collect_rows(
    n_values: Sequence[int],
    bases: Sequence[int],
    x_step: int,
    m_zeros: Sequence[float],
    ref_zeros: Sequence[float],
    u_mode: str,
    zero_kernel: str,
    kernel_scale: float,
    chunk_n: int,
    beta: float,
    jobs: int,
    event_stride: int,
    event_scale: bool,
    cache_dir: str,
    use_cache: bool,
) -> Tuple[List[Dict[str, object]], Dict[str, int]]:
    rows: List[Dict[str, object]] = []
    if not n_values:
        return rows

    max_n = max(n_values)
    zero_terms_m = hx.prepare_zero_terms(
        zeros=m_zeros,
        zero_kernel=zero_kernel,
        kernel_scale=kernel_scale,
    )
    zero_terms_ref = hx.prepare_zero_terms(
        zeros=ref_zeros,
        zero_kernel=zero_kernel,
        kernel_scale=kernel_scale,
    )

    x0 = max(2, x_step)
    xs_max = list(range(x0, max_n + 1, x_step))
    psi_max = hx.psi_exact_samples(xs_max)
    e_max = [(v - x) / math.sqrt(x) for x, v in zip(xs_max, psi_max)]
    lb_max = [math.log(max(3, x)) ** beta for x in xs_max]

    x_map = {}
    e_map = {}
    lb_map = {}
    x_counts = {}
    for n_max in sorted(set(n_values)):
        k = ((n_max - x0) // x_step) + 1 if n_max >= x0 else 0
        k = max(0, min(k, len(xs_max)))
        x_map[n_max] = xs_max[:k]
        e_map[n_max] = e_max[:k]
        lb_map[n_max] = lb_max[:k]
        x_counts[n_max] = k

    def prefix_sums(vals: Sequence[float]) -> List[float]:
        out = []
        acc = 0.0
        for v in vals:
            acc += v
            out.append(acc)
        return out

    def h_from_prefix(seq_n: Sequence[int], pref: Sequence[float], xs: Sequence[int]) -> List[float]:
        out = []
        for x in xs:
            i = bisect.bisect_right(seq_n, x) - 1
            acc = pref[i] if i >= 0 else 0.0
            out.append(acc / math.sqrt(x))
        return out

    base_list = sorted(set(bases))
    zsig_m = zeros_sig(m_zeros)
    zsig_r = zeros_sig(ref_zeros)
    cache_hits = 0

    def build_base_rows(b: int) -> List[Dict[str, object]]:
        out: List[Dict[str, object]] = []
        nonlocal cache_hits

        key_m = cache_key(
            {
                "version": 1,
                "mode": "m",
                "base": b,
                "max_n": max_n,
                "x_step": x_step,
                "u_mode": u_mode,
                "zero_kernel": zero_kernel,
                "kernel_scale": kernel_scale,
                "chunk_n": chunk_n,
                "event_stride": event_stride,
                "event_scale": bool(event_scale),
                "weights": WEIGHTS,
                "zsig": zsig_m,
            }
        )
        key_r = cache_key(
            {
                "version": 1,
                "mode": "ref",
                "base": b,
                "max_n": max_n,
                "x_step": x_step,
                "u_mode": u_mode,
                "zero_kernel": zero_kernel,
                "kernel_scale": kernel_scale,
                "chunk_n": chunk_n,
                "event_stride": event_stride,
                "event_scale": bool(event_scale),
                "weights": WEIGHTS,
                "zsig": zsig_r,
            }
        )
        pm = os.path.join(cache_dir, f"{key_m}.npz")
        pr = os.path.join(cache_dir, f"{key_r}.npz")
        h_m_max = load_cache(pm) if use_cache else None
        h_r_max = load_cache(pr) if use_cache else None
        if h_m_max is not None and len(h_m_max) == len(xs_max):
            cache_hits += 1
        else:
            seq_m_n, seq_m_g = hx.stream_weighted_events(
                n_max=max_n,
                base=b,
                zeros=m_zeros,
                u_mode=u_mode,
                zero_kernel=zero_kernel,
                kernel_scale=kernel_scale,
                weights=WEIGHTS,
                chunk_n=chunk_n,
                zero_terms=zero_terms_m,
                event_stride=event_stride,
                event_scale=event_scale,
            )
            pref_m = prefix_sums(seq_m_g)
            h_m_max = h_from_prefix(seq_m_n, pref_m, xs_max)
            if use_cache:
                save_cache(pm, h_m_max)
        if h_r_max is not None and len(h_r_max) == len(xs_max):
            cache_hits += 1
        else:
            seq_r_n, seq_r_g = hx.stream_weighted_events(
                n_max=max_n,
                base=b,
                zeros=ref_zeros,
                u_mode=u_mode,
                zero_kernel=zero_kernel,
                kernel_scale=kernel_scale,
                weights=WEIGHTS,
                chunk_n=chunk_n,
                zero_terms=zero_terms_ref,
                event_stride=event_stride,
                event_scale=event_scale,
            )
            pref_r = prefix_sums(seq_r_g)
            h_r_max = h_from_prefix(seq_r_n, pref_r, xs_max)
            if use_cache:
                save_cache(pr, h_r_max)

        for n_max in sorted(set(n_values)):
            k = x_counts[n_max]
            h_m = h_m_max[:k]
            h_ref = h_r_max[:k]
            out.append(
                {
                    "split_n": int(n_max),
                    "base": int(b),
                    "x": x_map[n_max],
                    "e_scaled": e_map[n_max],
                    "h_m": h_m,
                    "h_ref": h_ref,
                    "log_beta": lb_map[n_max],
                }
            )
        return out

    if jobs > 1 and len(base_list) > 1:
        workers = min(jobs, len(base_list))
        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as ex:
            futs = [ex.submit(build_base_rows, b) for b in base_list]
            for fut in concurrent.futures.as_completed(futs):
                rows.extend(fut.result())
    else:
        for b in base_list:
            rows.extend(build_base_rows(b))
    rows.sort(key=lambda r: (r["split_n"], r["base"]))
    return rows, {"hits": cache_hits, "misses": max(0, 2 * len(base_list) - cache_hits)}


def extract_constants(
    rows: Sequence[Dict[str, object]],
    tau_m: float,
    beta: float,
    a_h: float,
) -> Dict[str, float]:
    pooled_h_ref: List[float] = []
    pooled_e: List[float] = []
    for row in rows:
        pooled_h_ref.extend(row["h_ref"])  # type: ignore[arg-type]
        pooled_e.extend(row["e_scaled"])  # type: ignore[arg-type]
    a_ref, b_ref, _ = hx.fit_linear(pooled_h_ref, pooled_e)

    c0 = 0.0
    c_delta = 0.0
    c_h = 0.0
    for row in rows:
        e = row["e_scaled"]  # type: ignore[assignment]
        h_m = row["h_m"]  # type: ignore[assignment]
        h_ref = row["h_ref"]  # type: ignore[assignment]
        log_beta = row["log_beta"]  # type: ignore[assignment]
        resid_ref = [abs(ev - (a_ref * hr + b_ref)) for ev, hr in zip(e, h_ref)]
        deltas = [abs(hm - hr) for hm, hr in zip(h_m, h_ref)]
        c0 = max(c0, max(resid_ref) if resid_ref else 0.0)
        for d, lb in zip(deltas, log_beta):
            c_delta = max(c_delta, d / max(1e-30, lb * tau_m))
        for hm, x in zip(h_m, row["x"]):  # type: ignore[index]
            c_h = max(c_h, abs(hm) / max(1e-30, math.log(max(3, x)) ** a_h))

    return {
        "a_ref": float(a_ref),
        "b_ref": float(b_ref),
        "C0_ref": float(c0),
        "C_delta": float(c_delta),
        "C_H": float(c_h),
    }


def validate_rows(
    rows: Sequence[Dict[str, object]],
    consts: Dict[str, float],
    tau_m: float,
    beta: float,
    a_h: float,
) -> Dict[str, object]:
    worst_gap = -1e18
    violations = 0
    worst = None
    ratios = []
    per_row = []
    for row in rows:
        max_lhs = 0.0
        max_rhs = 0.0
        max_ratio = 0.0
        row_worst_gap = -1e18
        for x, ev in zip(row["x"], row["e_scaled"]):  # type: ignore[index]
            lhs = abs(ev)
            rhs = theorem_rhs(
                x=x,
                a_ref=consts["a_ref"],
                b_ref=consts["b_ref"],
                c0=consts["C0_ref"],
                c_delta=consts["C_delta"],
                tau_m=tau_m,
                beta=beta,
                c_h=consts["C_H"],
                a_h=a_h,
            )
            gap = lhs - rhs
            ratio = lhs / max(1e-30, rhs)
            if gap > 1e-12:
                violations += 1
            if gap > worst_gap:
                worst_gap = gap
                worst = {"n_max": row["split_n"], "base": row["base"], "x": x, "lhs": lhs, "rhs": rhs, "gap": gap}
            row_worst_gap = max(row_worst_gap, gap)
            max_lhs = max(max_lhs, lhs)
            max_rhs = max(max_rhs, rhs)
            max_ratio = max(max_ratio, ratio)
            ratios.append(ratio)
        per_row.append(
            {
                "n_max": row["split_n"],
                "base": row["base"],
                "max_abs_e_scaled": max_lhs,
                "max_rhs": max_rhs,
                "max_ratio_lhs_over_rhs": max_ratio,
                "row_max_gap": row_worst_gap,
            }
        )
    return {
        "holds_on_rows": violations == 0 and worst_gap <= 1e-12,
        "violations": int(violations),
        "max_gap": float(worst_gap),
        "ratio_min": min(ratios) if ratios else 0.0,
        "ratio_max": max(ratios) if ratios else 0.0,
        "worst_point": worst,
        "per_row": per_row,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Conservative constant replacement pack (A1-A4)")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--train-n-values", type=str, default="300000,1000000,2000000")
    ap.add_argument("--valid-n-values", type=str, default="5000000,10000000")
    ap.add_argument("--x-step", type=int, default=5000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--m-zero", type=int, default=128)
    ap.add_argument("--m-ref", type=int, default=512)
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--beta", type=float, default=2.6)
    ap.add_argument("--a-h", type=float, default=7.2)
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--cache-dir", type=str, default="research/cache/analytic_constant_replacement_pack")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--safety-factor", type=float, default=1.2)
    ap.add_argument(
        "--output",
        type=str,
        default="research/output/analytic_constant_replacement_pack_none_z128_ref512.json",
    )
    args = ap.parse_args()

    bases = parse_int_list(args.bases)
    train_n_values = sorted(parse_int_list(args.train_n_values))
    valid_n_values = sorted(parse_int_list(args.valid_n_values))
    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.m_ref > len(zeros_all):
        raise ValueError("m-ref exceeds available zero data")
    if not (0 < args.m_zero < args.m_ref):
        raise ValueError("need 0 < m-zero < m-ref")

    zeros_m = zeros_all[: args.m_zero]
    zeros_ref = zeros_all[: args.m_ref]
    tau_m = tau_tail_fp(zeros_ref, args.m_zero, args.zero_kernel, args.kernel_scale)

    all_n_values = sorted(set(train_n_values + valid_n_values))
    all_rows, cache_stats = collect_rows(
        n_values=all_n_values,
        bases=bases,
        x_step=args.x_step,
        m_zeros=zeros_m,
        ref_zeros=zeros_ref,
        u_mode=args.u_mode,
        zero_kernel=args.zero_kernel,
        kernel_scale=args.kernel_scale,
        chunk_n=args.chunk_n,
        beta=args.beta,
        jobs=args.jobs,
        event_stride=max(1, args.event_stride),
        event_scale=args.event_scale,
        cache_dir=args.cache_dir,
        use_cache=(not args.no_cache),
    )
    train_set = set(train_n_values)
    valid_set = set(valid_n_values)
    train_rows = [r for r in all_rows if r["split_n"] in train_set]
    valid_rows = [r for r in all_rows if r["split_n"] in valid_set]
    train_points = sum(len(r["x"]) for r in train_rows)
    valid_points = sum(len(r["x"]) for r in valid_rows)
    if train_points < 10:
        raise ValueError(
            "insufficient train grid points; increase train-n-values or decrease x-step"
        )
    if valid_points == 0:
        raise ValueError(
            "validation grid has zero points; increase valid-n-values or decrease x-step"
        )

    base_consts = extract_constants(train_rows, tau_m=tau_m, beta=args.beta, a_h=args.a_h)
    sf = max(1.0, args.safety_factor)
    conservative_consts = {
        "a_ref": base_consts["a_ref"],
        "b_ref": base_consts["b_ref"],
        "C0_ref": sf * base_consts["C0_ref"],
        "C_delta": sf * base_consts["C_delta"],
        "C_H": sf * base_consts["C_H"],
    }

    train_eval = validate_rows(train_rows, consts=conservative_consts, tau_m=tau_m, beta=args.beta, a_h=args.a_h)
    valid_eval = validate_rows(valid_rows, consts=conservative_consts, tau_m=tau_m, beta=args.beta, a_h=args.a_h)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "train_n_values": train_n_values,
            "valid_n_values": valid_n_values,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "weights": WEIGHTS,
            "m_zero": args.m_zero,
            "m_ref": args.m_ref,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "beta": args.beta,
            "a_h": args.a_h,
            "chunk_n": args.chunk_n,
            "jobs": args.jobs,
            "event_stride": max(1, args.event_stride),
            "event_scale": bool(args.event_scale),
            "cache_dir": args.cache_dir,
            "cache_enabled": not args.no_cache,
            "safety_factor": sf,
            "train_points": train_points,
            "valid_points": valid_points,
        },
        "tau_m_tail_fp": tau_m,
        "fit_constants_train": base_consts,
        "conservative_constants": conservative_consts,
        "train_eval": train_eval,
        "valid_eval": valid_eval,
    }
    report["cache_stats"] = cache_stats

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Analytic Constant Replacement Pack\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write("## Conservative constants\n\n")
        c = conservative_consts
        f.write(f"- a_ref: {c['a_ref']:.12f}\n")
        f.write(f"- b_ref: {c['b_ref']:.12f}\n")
        f.write(f"- C0_ref: {c['C0_ref']:.12f}\n")
        f.write(f"- C_delta: {c['C_delta']:.12e}\n")
        f.write(f"- C_H: {c['C_H']:.12e}\n")
        f.write(f"- tau_m_tail_fp: {tau_m:.12e}\n\n")
        t = train_eval
        v = valid_eval
        f.write("## Train grid check\n\n")
        f.write(f"- holds: {t['holds_on_rows']}\n")
        f.write(f"- violations: {t['violations']}\n")
        f.write(f"- max_gap(lhs-rhs): {t['max_gap']:.6e}\n")
        f.write(f"- ratio range lhs/rhs: [{t['ratio_min']:.6f}, {t['ratio_max']:.6f}]\n\n")
        f.write("## Validation grid check\n\n")
        f.write(f"- holds: {v['holds_on_rows']}\n")
        f.write(f"- violations: {v['violations']}\n")
        f.write(f"- max_gap(lhs-rhs): {v['max_gap']:.6e}\n")
        f.write(f"- ratio range lhs/rhs: [{v['ratio_min']:.6f}, {v['ratio_max']:.6f}]\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
