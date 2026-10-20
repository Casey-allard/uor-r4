#!/usr/bin/env python3
"""Lemma B program: empirical truncation envelopes B_M(X) for H_W."""

from __future__ import annotations

import argparse
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
        if "h" not in d:
            return None
        return d["h"].astype(np.float64).tolist()
    except Exception:
        return None


def save_cache(path: str, h: Sequence[float]) -> None:
    np = s4.np
    if np is None:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    np.savez_compressed(path, h=np.asarray(h, dtype=np.float64))


def analytic_tail_sums(zeros: Sequence[float], m: int, kernel: str, scale: float) -> Dict[str, float]:
    t0 = 0.0
    t1 = 0.0
    for g in zeros[m:]:
        w = s4.zero_weight(g, kernel, scale)
        den = math.sqrt(0.25 + g * g)
        t0 += w / den
        t1 += w * (g / den)
    return {"tail_F_bound": t0, "tail_Fp_bound": t1}


def l2_diff(a: Sequence[float], b: Sequence[float]) -> float:
    if not a or len(a) != len(b):
        return 0.0
    return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)) / len(a))


def max_abs_diff(a: Sequence[float], b: Sequence[float]) -> float:
    if not a or len(a) != len(b):
        return 0.0
    return max(abs(x - y) for x, y in zip(a, b))


def _stream_worker(payload):
    base, m, n_max, zeros, u_mode, zero_kernel, kernel_scale, chunk_n, event_stride, event_scale = payload
    seq_n, g_vals = hx.stream_weighted_events(
        n_max=n_max,
        base=base,
        zeros=zeros,
        u_mode=u_mode,
        zero_kernel=zero_kernel,
        kernel_scale=kernel_scale,
        weights=WEIGHTS,
        chunk_n=chunk_n,
        event_stride=event_stride,
        event_scale=event_scale,
    )
    return base, m, seq_n, g_vals


def compute_series(
    bases: Sequence[int],
    n_max: int,
    xs: Sequence[int],
    zeros_all: Sequence[float],
    limits: Sequence[int],
    u_mode: str,
    zero_kernel: str,
    kernel_scale: float,
    chunk_n: int,
    jobs: int,
    event_stride: int = 1,
    event_scale: bool = False,
    cache_dir: str = "",
    use_cache: bool = False,
) -> Dict[int, Dict[int, List[float]]]:
    out: Dict[int, Dict[int, List[float]]] = {b: {} for b in bases}
    zsig = zeros_sig(zeros_all[: max(limits)]) if limits else ""
    payloads = []
    cache_hits = 0
    for b in bases:
        for m in limits:
            key = cache_key(
                {
                    "version": 1,
                    "base": b,
                    "m": m,
                    "n_max": n_max,
                    "xs_len": len(xs),
                    "x_last": xs[-1] if xs else 0,
                    "u_mode": u_mode,
                    "zero_kernel": zero_kernel,
                    "kernel_scale": kernel_scale,
                    "chunk_n": chunk_n,
                    "event_stride": event_stride,
                    "event_scale": bool(event_scale),
                    "weights": WEIGHTS,
                    "zsig": zsig,
                }
            )
            cp = os.path.join(cache_dir, f"{key}.npz") if cache_dir else ""
            if use_cache and cp:
                h = load_cache(cp)
                if h is not None and len(h) == len(xs):
                    out[b][m] = h
                    cache_hits += 1
                    continue
            payloads.append((b, m, n_max, zeros_all[:m], u_mode, zero_kernel, kernel_scale, chunk_n, event_stride, event_scale, cp))

    if jobs > 1 and len(payloads) > 1:
        try:
            with concurrent.futures.ProcessPoolExecutor(max_workers=jobs) as ex:
                mapped = ((b, m, nm, z, um, zk, ks, cn, es, ec) for (b, m, nm, z, um, zk, ks, cn, es, ec, _) in payloads)
                for (b, m, seq_n, g_vals), (_, _, _, _, _, _, _, _, _, _, cp) in zip(ex.map(_stream_worker, mapped), payloads):
                    out[b][m] = hx.h_scaled_from_events(seq_n, g_vals, xs)
                    if use_cache and cp:
                        save_cache(cp, out[b][m])
        except Exception:
            # Safe fallback to sequential (sandbox often blocks process pools).
            for b, m, nm, z, um, zk, ks, cn, es, ec, cp in payloads:
                _, _, seq_n, g_vals = _stream_worker((b, m, nm, z, um, zk, ks, cn, es, ec))
                out[b][m] = hx.h_scaled_from_events(seq_n, g_vals, xs)
                if use_cache and cp:
                    save_cache(cp, out[b][m])
            return out

    # Sequential path (or after fallback).
    if not any(out[b] for b in out):
        for b, m, nm, z, um, zk, ks, cn, es, ec, cp in payloads:
            _, _, seq_n, g_vals = _stream_worker((b, m, nm, z, um, zk, ks, cn, es, ec))
            out[b][m] = hx.h_scaled_from_events(seq_n, g_vals, xs)
            if use_cache and cp:
                save_cache(cp, out[b][m])
    out["_cache_stats"] = {"hits": cache_hits, "misses": len(payloads)}  # type: ignore[index]
    return out


def monotone_nonincreasing(vals: Sequence[float], tol: float = 1e-15) -> bool:
    return all(vals[i + 1] <= vals[i] + tol for i in range(len(vals) - 1))


def main() -> None:
    ap = argparse.ArgumentParser(description="Lemma B truncation envelope program")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="300000,1000000")
    ap.add_argument("--x-step", type=int, default=2000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zero-kernel", type=str, default="gaussian", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=100.0)
    ap.add_argument("--zero-limits", type=str, default="64,128,192,256")
    ap.add_argument("--ref-zero-limit", type=int, default=512)
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--cache-dir", type=str, default="research/cache/lemma_b_truncation_program")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/lemma_b_truncation_program.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = sorted(int(x.strip()) for x in args.n_values.split(",") if x.strip())
    limits = sorted(set(int(x.strip()) for x in args.zero_limits.split(",") if x.strip()))
    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.ref_zero_limit > len(zeros_all):
        raise ValueError("ref-zero-limit exceeds available zeros")
    limits = [m for m in limits if 0 < m < args.ref_zero_limit]
    if not limits:
        raise ValueError("need at least one zero limit below ref-zero-limit")
    limits_ref = limits + [args.ref_zero_limit]

    per_n = []
    global_worst = {str(m): 0.0 for m in limits}
    monotone_summary = []
    for n_max in n_values:
        xs = list(range(max(2, args.x_step), n_max + 1, args.x_step))
        hs = compute_series(
            bases=bases,
            n_max=n_max,
            xs=xs,
            zeros_all=zeros_all,
            limits=limits_ref,
            u_mode=args.u_mode,
            zero_kernel=args.zero_kernel,
            kernel_scale=args.kernel_scale,
            chunk_n=args.chunk_n,
            jobs=args.jobs,
            event_stride=max(1, args.event_stride),
            event_scale=args.event_scale,
            cache_dir=args.cache_dir,
            use_cache=(not args.no_cache),
        )
        base_rows = []
        for b in bases:
            ref = hs[b][args.ref_zero_limit]
            rows = []
            diffs_for_mono = []
            for m in limits:
                cur = hs[b][m]
                mad = max_abs_diff(cur, ref)
                ld = l2_diff(cur, ref)
                diffs_for_mono.append(mad)
                global_worst[str(m)] = max(global_worst[str(m)], mad)
                rows.append(
                    {
                        "M": m,
                        "max_abs_diff_vs_ref": mad,
                        "l2_diff_vs_ref": ld,
                        **analytic_tail_sums(zeros_all[: args.ref_zero_limit], m, args.zero_kernel, args.kernel_scale),
                    }
                )
            mono = monotone_nonincreasing(diffs_for_mono)
            monotone_summary.append({"n_max": n_max, "base": b, "monotone_nonincreasing": mono})
            base_rows.append({"base": b, "rows": rows, "monotone_nonincreasing": mono})
        per_n.append({"n_max": n_max, "per_base": base_rows})

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": n_values,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "zero_limits": limits,
            "ref_zero_limit": args.ref_zero_limit,
            "weights": WEIGHTS,
            "chunk_n": args.chunk_n,
            "jobs": args.jobs,
            "event_stride": max(1, args.event_stride),
            "event_scale": bool(args.event_scale),
            "cache_dir": args.cache_dir,
            "cache_enabled": not args.no_cache,
        },
        "per_n": per_n,
        "global_worst_max_abs_diff_by_M": global_worst,
        "all_monotone_nonincreasing": all(x["monotone_nonincreasing"] for x in monotone_summary),
        "monotone_checks": monotone_summary,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Lemma B Truncation Program\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"- all monotone nonincreasing: {report['all_monotone_nonincreasing']}\n")
        f.write("- global worst max_abs_diff by M:\n")
        for m in limits:
            f.write(f"  - M={m}: {report['global_worst_max_abs_diff_by_M'][str(m)]:.6e}\n")
        f.write("\n")
        for rn in per_n:
            f.write(f"## n_max={rn['n_max']}\n\n")
            for br in rn["per_base"]:
                f.write(f"- base={br['base']} monotone={br['monotone_nonincreasing']}\n")
                for row in br["rows"]:
                    f.write(
                        f"  - M={row['M']} max_abs={row['max_abs_diff_vs_ref']:.6e} "
                        f"l2={row['l2_diff_vs_ref']:.6e} "
                        f"tail_F={row['tail_F_bound']:.6e} tail_Fp={row['tail_Fp_bound']:.6e}\n"
                    )
            f.write("\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    print("all_monotone_nonincreasing:", report["all_monotone_nonincreasing"])


if __name__ == "__main__":
    main()
