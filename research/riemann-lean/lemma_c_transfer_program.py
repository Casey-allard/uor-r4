#!/usr/bin/env python3
"""Lemma C program: bridge transfer via explicit-formula surrogate S_M(x)."""

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
from prime_geometry_loop import load_zeta_zeros_file
import spinning_top_r4_candidate_a as s4


WEIGHTS = (-1.0, -1.0, 0.0, -1.0)


def explicit_surrogate_s(xs: Sequence[int], zeros: Sequence[float]) -> List[float]:
    if not xs or not zeros:
        return [0.0 for _ in xs]
    np = __import__("spinning_top_r4_candidate_a").np
    if np is not None:
        u = np.log(np.asarray(xs, dtype=np.float64))
        g = np.asarray(zeros, dtype=np.float64)
        den = 0.25 + g * g
        phase = np.outer(u, g)
        c = np.cos(phase)
        s = np.sin(phase)
        re_term = (0.5 * c + s * g) / den
        # E/sqrt(x) leading oscillatory surrogate from explicit formula.
        return list((-2.0 * re_term.sum(axis=1)).astype(float))

    out = []
    for x in xs:
        u = math.log(x)
        acc = 0.0
        for g in zeros:
            den = 0.25 + g * g
            acc += (0.5 * math.cos(g * u) + g * math.sin(g * u)) / den
        out.append(-2.0 * acc)
    return out


def row_metrics(hs: Sequence[float], s: Sequence[float], e: Sequence[float]) -> Dict[str, float]:
    # H -> S
    a_hs, b_hs, rmse_hs = hx.fit_linear(hs, s)
    fit_hs = [a_hs * v + b_hs for v in hs]
    c_hs = hx.corr(hs, s)
    c_hs_fit = hx.corr(fit_hs, s)

    # S -> E
    a_se, b_se, rmse_se = hx.fit_linear(s, e)
    fit_se = [a_se * v + b_se for v in s]
    c_se = hx.corr(s, e)
    c_se_fit = hx.corr(fit_se, e)

    # Direct H -> E
    a_he, b_he, rmse_he = hx.fit_linear(hs, e)
    fit_he = [a_he * v + b_he for v in hs]
    c_he = hx.corr(hs, e)
    c_he_fit = hx.corr(fit_he, e)

    # Composed map H -> (S-fit) -> E-fit
    a_comp = a_se * a_hs
    b_comp = a_se * b_hs + b_se
    fit_comp = [a_comp * v + b_comp for v in hs]
    c_comp_fit = hx.corr(fit_comp, e)
    rmse_comp = math.sqrt(sum((yy - ff) ** 2 for yy, ff in zip(e, fit_comp)) / len(e)) if e else 0.0
    max_abs_comp = max(abs(yy - ff) for yy, ff in zip(e, fit_comp)) if e else 0.0

    return {
        "corr_h_vs_s": c_hs,
        "corr_fit_h_to_s_vs_s": c_hs_fit,
        "a_h_to_s": a_hs,
        "b_h_to_s": b_hs,
        "rmse_h_to_s": rmse_hs,
        "corr_s_vs_e": c_se,
        "corr_fit_s_to_e_vs_e": c_se_fit,
        "a_s_to_e": a_se,
        "b_s_to_e": b_se,
        "rmse_s_to_e": rmse_se,
        "corr_h_vs_e": c_he,
        "corr_fit_h_to_e_vs_e": c_he_fit,
        "a_h_to_e": a_he,
        "b_h_to_e": b_he,
        "rmse_h_to_e": rmse_he,
        "a_composed_h_to_e": a_comp,
        "b_composed_h_to_e": b_comp,
        "corr_fit_composed_h_to_e_vs_e": c_comp_fit,
        "rmse_composed_h_to_e": rmse_comp,
        "max_abs_composed_h_to_e": max_abs_comp,
    }


def cache_key(payload):
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


def main() -> None:
    ap = argparse.ArgumentParser(description="Lemma C transfer program")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="300000,600000,1000000")
    ap.add_argument("--x-step", type=int, default=2000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=64)
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--chunk-n", type=int, default=50000)
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--event-stride", type=int, default=1)
    ap.add_argument("--event-scale", action="store_true")
    ap.add_argument("--cache-dir", type=str, default="research/cache/lemma_c_transfer_program")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/lemma_c_transfer_program.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = sorted(int(x.strip()) for x in args.n_values.split(",") if x.strip())
    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros

    rows = []
    x0 = max(2, args.x_step)
    max_n = max(n_values)
    xs_max = list(range(x0, max_n + 1, args.x_step))
    psi_max = hx.psi_exact_samples(xs_max)
    e_max = [(v - x) / math.sqrt(x) for x, v in zip(xs_max, psi_max)]
    x_map = {}
    e_map = {}
    count = {}
    for n in n_values:
        k = ((n - x0) // args.x_step) + 1 if n >= x0 else 0
        k = max(0, min(k, len(xs_max)))
        x_map[n] = xs_max[:k]
        e_map[n] = e_max[:k]
        count[n] = k
    zero_terms = hx.prepare_zero_terms(zeros, args.zero_kernel, args.kernel_scale)
    zsig = zeros_sig(zeros)
    event_stride = max(1, args.event_stride)
    use_cache = not args.no_cache
    cache_hits = 0

    def prefix(vals):
        out = []
        a = 0.0
        for v in vals:
            a += v
            out.append(a)
        return out

    def h_from_prefix(seq_n, pref, xs):
        out = []
        for x in xs:
            i = bisect.bisect_right(seq_n, x) - 1
            a = pref[i] if i >= 0 else 0.0
            out.append(a / math.sqrt(x))
        return out

    def base_rows(b):
        nonlocal cache_hits
        k = cache_key({"version": 1, "base": b, "max_n": max_n, "x_step": args.x_step, "u_mode": args.u_mode, "zero_kernel": args.zero_kernel, "kernel_scale": args.kernel_scale, "chunk_n": args.chunk_n, "event_stride": event_stride, "event_scale": bool(args.event_scale), "weights": WEIGHTS, "zsig": zsig})
        cp = os.path.join(args.cache_dir, f"{k}.npz")
        h_max = load_cache(cp) if use_cache else None
        if h_max is not None and len(h_max) == len(xs_max):
            cache_hits += 1
        else:
            seq_n, g_vals = hx.stream_weighted_events(max_n, b, zeros, args.u_mode, args.zero_kernel, args.kernel_scale, WEIGHTS, args.chunk_n, zero_terms=zero_terms, event_stride=event_stride, event_scale=args.event_scale)
            h_max = h_from_prefix(seq_n, prefix(g_vals), xs_max)
            if use_cache:
                save_cache(cp, h_max)
        out = []
        for n in n_values:
            kk = count[n]
            xs = x_map[n]
            e_scaled = e_map[n]
            s_scaled = explicit_surrogate_s(xs, zeros)
            met = row_metrics(h_max[:kk], s_scaled, e_scaled)
            out.append({"n_max": n, "base": b, **met})
        return out

    if args.jobs > 1 and len(bases) > 1:
        with concurrent.futures.ThreadPoolExecutor(max_workers=min(args.jobs, len(bases))) as ex:
            futs = [ex.submit(base_rows, b) for b in bases]
            for fut in concurrent.futures.as_completed(futs):
                rows.extend(fut.result())
    else:
        for b in bases:
            rows.extend(base_rows(b))
    rows.sort(key=lambda z: (z["n_max"], z["base"]))

    # global composed envelope
    max_abs_comp = max(r["max_abs_composed_h_to_e"] for r in rows) if rows else 0.0
    mean_abs_corr_hs = sum(abs(r["corr_h_vs_s"]) for r in rows) / max(1, len(rows))
    mean_abs_corr_se = sum(abs(r["corr_s_vs_e"]) for r in rows) / max(1, len(rows))
    mean_abs_corr_he = sum(abs(r["corr_h_vs_e"]) for r in rows) / max(1, len(rows))
    mean_abs_corr_comp = sum(abs(r["corr_fit_composed_h_to_e_vs_e"]) for r in rows) / max(1, len(rows))

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": n_values,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "weights": WEIGHTS,
            "max_zeta_zeros": len(zeros),
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "chunk_n": args.chunk_n,
            "jobs": args.jobs,
            "event_stride": event_stride,
            "event_scale": bool(args.event_scale),
            "cache_dir": args.cache_dir,
            "cache_enabled": use_cache,
            "cache_hits": cache_hits,
            "cache_misses": max(0, len(bases) - cache_hits),
        },
        "summary": {
            "mean_abs_corr_h_vs_s": mean_abs_corr_hs,
            "mean_abs_corr_s_vs_e": mean_abs_corr_se,
            "mean_abs_corr_h_vs_e": mean_abs_corr_he,
            "mean_abs_corr_fit_composed_h_to_e_vs_e": mean_abs_corr_comp,
            "global_max_abs_composed_h_to_e": max_abs_comp,
        },
        "rows": rows,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Lemma C Transfer Program\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        s = report["summary"]
        f.write(f"- mean |corr(H,S)|: {s['mean_abs_corr_h_vs_s']:.6f}\n")
        f.write(f"- mean |corr(S,E)|: {s['mean_abs_corr_s_vs_e']:.6f}\n")
        f.write(f"- mean |corr(H,E)|: {s['mean_abs_corr_h_vs_e']:.6f}\n")
        f.write(f"- mean |corr(compose(H->S->E),E)|: {s['mean_abs_corr_fit_composed_h_to_e_vs_e']:.6f}\n")
        f.write(f"- global max_abs composed residual: {s['global_max_abs_composed_h_to_e']:.6f}\n\n")
        for r in sorted(report["rows"], key=lambda z: (z["n_max"], z["base"])):
            f.write(
                f"- n_max={r['n_max']} base={r['base']} corr(H,S)={r['corr_h_vs_s']:.6f} "
                f"corr(S,E)={r['corr_s_vs_e']:.6f} corr(H,E)={r['corr_h_vs_e']:.6f} "
                f"corr(comp,E)={r['corr_fit_composed_h_to_e_vs_e']:.6f} "
                f"max_abs_comp={r['max_abs_composed_h_to_e']:.6f}\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
