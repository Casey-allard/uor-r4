#!/usr/bin/env python3
"""Bridge probe: compare H(x) from manifold candidate to E(x)=psi(x)-x."""

from __future__ import annotations

import argparse
import bisect
import json
import math
import os
import sys
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

from prime_geometry_loop import load_zeta_zeros_file
import spinning_top_r4_candidate_a as s4

VENDOR_PATH = os.path.join(os.path.dirname(__file__), ".vendor")
if os.path.isdir(VENDOR_PATH) and VENDOR_PATH not in sys.path:
    sys.path.insert(0, VENDOR_PATH)

try:
    import pyprimesieve as pps
except Exception:
    pps = None

try:
    from sympy import primerange
except Exception:
    primerange = None


def prepare_zero_terms(
    zeros: Sequence[float],
    zero_kernel: str,
    kernel_scale: float,
):
    if s4.np is None:
        raise RuntimeError("numpy is required for zero-term preparation")
    np = s4.np
    g_all = np.asarray(zeros, dtype=np.float64)
    w_all = np.asarray([s4.zero_weight(float(g), zero_kernel, kernel_scale) for g in g_all], dtype=np.float64)
    dinv_all = 1.0 / (0.25 + g_all * g_all)
    return g_all, w_all, dinv_all


def psi_exact_table(n_max: int) -> List[float]:
    is_prime = [False, False] + [True] * (n_max - 1)
    for p in range(2, int(n_max**0.5) + 1):
        if is_prime[p]:
            is_prime[p * p : n_max + 1 : p] = [False] * (((n_max - p * p) // p) + 1)
    primes = [i for i in range(2, n_max + 1) if is_prime[i]]

    psi = [0.0] * (n_max + 1)
    for p in primes:
        lp = math.log(p)
        pk = p
        while pk <= n_max:
            psi[pk] += lp
            if pk > n_max // p:
                break
            pk *= p
    for x in range(1, n_max + 1):
        psi[x] += psi[x - 1]
    return psi


def _iter_primes(n_max: int):
    if pps is not None:
        yield from pps.primes(n_max)
        return
    if primerange is not None:
        yield from primerange(2, n_max + 1)
        return
    is_prime = [False, False] + [True] * (n_max - 1)
    for p in range(2, int(n_max**0.5) + 1):
        if is_prime[p]:
            is_prime[p * p : n_max + 1 : p] = [False] * (((n_max - p * p) // p) + 1)
    for p in range(2, n_max + 1):
        if is_prime[p]:
            yield p


def psi_exact_samples(xs: Sequence[int]) -> List[float]:
    if not xs:
        return []
    xs = sorted(xs)
    x_max = xs[-1]
    delta = [0.0] * len(xs)
    for p in _iter_primes(x_max):
        lp = math.log(p)
        pk = p
        while pk <= x_max:
            i = bisect.bisect_left(xs, pk)
            if i < len(xs):
                delta[i] += lp
            if pk > x_max // p:
                break
            pk *= p
    out = [0.0] * len(xs)
    acc = 0.0
    for i, v in enumerate(delta):
        acc += v
        out[i] = acc
    return out


def corr(a: Sequence[float], b: Sequence[float]) -> float:
    if len(a) != len(b) or not a:
        return 0.0
    ma = sum(a) / len(a)
    mb = sum(b) / len(b)
    va = sum((x - ma) ** 2 for x in a)
    vb = sum((y - mb) ** 2 for y in b)
    if va < 1e-12 or vb < 1e-12:
        return 0.0
    cov = sum((x - ma) * (y - mb) for x, y in zip(a, b))
    return cov / math.sqrt(va * vb)


def fit_linear(x: Sequence[float], y: Sequence[float]) -> Tuple[float, float, float]:
    # y ~= a*x + b
    if len(x) != len(y) or len(x) < 2:
        return 0.0, 0.0, 0.0
    n = len(x)
    sx = sum(x)
    sy = sum(y)
    sxx = sum(v * v for v in x)
    sxy = sum(a * b for a, b in zip(x, y))
    det = n * sxx - sx * sx
    if abs(det) < 1e-12:
        return 0.0, 0.0, 0.0
    a = (n * sxy - sx * sy) / det
    b = (sy * sxx - sx * sxy) / det
    resid = [yy - (a * xx + b) for xx, yy in zip(x, y)]
    rmse = math.sqrt(sum(r * r for r in resid) / len(resid))
    return a, b, rmse


def h_scaled_from_events(seq_n: Sequence[int], g_vals: Sequence[float], xs: Sequence[int]) -> List[float]:
    h_scaled = []
    i = 0
    acc = 0.0
    for x in xs:
        while i < len(seq_n) and seq_n[i] <= x:
            acc += g_vals[i]
            i += 1
        h_scaled.append(acc / math.sqrt(x))
    return h_scaled


def stream_weighted_events(
    n_max: int,
    base: int,
    zeros: Sequence[float],
    u_mode: str,
    zero_kernel: str,
    kernel_scale: float,
    weights: Sequence[float],
    chunk_n: int,
    zero_terms=None,
    event_stride: int = 1,
    event_scale: bool = False,
    zero_block: int | None = None,
) -> Tuple[List[int], List[float]]:
    if s4.np is None:
        raise RuntimeError("streaming mode requires numpy")
    np = s4.np

    if zero_terms is None:
        g_all, w_all, dinv_all = prepare_zero_terms(
            zeros=zeros,
            zero_kernel=zero_kernel,
            kernel_scale=kernel_scale,
        )
    else:
        g_all, w_all, dinv_all = zero_terms
    if zero_block is None:
        # Auto-tune for cache/memory pressure: larger blocks help low-zero runs,
        # while moderate blocks are more stable for heavy (e.g. m_ref=512) runs.
        z_chunk = 32 if len(g_all) >= 384 else 64
    else:
        z_chunk = max(4, int(zero_block))

    sq_3_4pi = math.sqrt(3.0 / (4.0 * math.pi))
    sq_5_16pi = math.sqrt(5.0 / (16.0 * math.pi))
    w0, w1, w2, w3 = (float(weights[0]), float(weights[1]), float(weights[2]), float(weights[3]))
    use_su2_trace = abs(w2) > 1e-15

    prev_theta = None
    prev_chi = None
    prev_n = None
    pending_n = None
    pending_g = 0.0
    seq_n = []
    g_vals = []

    for lo in range(2, n_max + 1, chunk_n):
        hi = min(n_max, lo + chunk_n - 1)
        n_arr = np.arange(lo, hi + 1, dtype=np.int64)
        mask = np.gcd(n_arr, base) == 1
        if not mask.any():
            continue
        n_sel = n_arr[mask]
        if event_stride > 1:
            n_sel = n_sel[::event_stride]
        if len(n_sel) == 0:
            continue
        u = np.log(n_sel.astype(np.float64)) if u_mode == "log" else n_sel.astype(np.float64)

        fr = np.zeros_like(u)
        fi = np.zeros_like(u)
        gr = np.zeros_like(u)
        gi = np.zeros_like(u)
        for z_lo in range(0, len(g_all), z_chunk):
            z_hi = min(len(g_all), z_lo + z_chunk)
            g = g_all[z_lo:z_hi]
            w = w_all[z_lo:z_hi]
            dinv = dinv_all[z_lo:z_hi]
            phase = np.outer(u, g)
            c = np.cos(phase)
            s = np.sin(phase)
            wd = w * dinv
            ar = (0.5 * c + s * g) * wd
            ai = (0.5 * s - c * g) * wd
            fr += ar.sum(axis=1)
            fi += ai.sum(axis=1)
            gr += (-(g * ai)).sum(axis=1)
            gi += ((g * ar)).sum(axis=1)

        theta_arr = np.arctan2(fi, fr)
        chi_arr = 0.5 * (np.arctan2(gi, gr) + math.pi)
        if len(theta_arr) == 0:
            continue

        if prev_theta is None:
            if len(theta_arr) >= 2:
                src_theta = theta_arr[:-1]
                src_chi = chi_arr[:-1]
                src_n = n_sel[:-1]
                dst_theta = theta_arr[1:]
            else:
                src_theta = np.empty((0,), dtype=np.float64)
                src_chi = np.empty((0,), dtype=np.float64)
                src_n = np.empty((0,), dtype=np.int64)
                dst_theta = np.empty((0,), dtype=np.float64)
        else:
            src_theta = np.concatenate((np.asarray([prev_theta], dtype=np.float64), theta_arr[:-1]))
            src_chi = np.concatenate((np.asarray([prev_chi], dtype=np.float64), chi_arr[:-1]))
            src_n = np.concatenate((np.asarray([prev_n], dtype=np.int64), n_sel[:-1]))
            dst_theta = theta_arr

        if len(dst_theta) > 0:
            transport = (dst_theta - src_theta + math.pi) % (2.0 * math.pi) - math.pi
            g_tr = np.zeros_like(transport)
            if abs(w0) > 1e-15:
                g_tr += w0 * (sq_3_4pi * np.sin(src_chi) * np.sin(src_theta))
            if abs(w1) > 1e-15:
                g_tr += w1 * (sq_5_16pi * (3.0 * (np.cos(src_chi) ** 2) - 1.0))
            if use_su2_trace:
                # trace(su2_step) = 2*cos(alpha/2), independent of (theta, chi)
                g_tr += w2 * (2.0 * np.cos(0.5 * transport))
            if abs(w3) > 1e-15:
                g_tr += w3 * transport
            if event_scale and event_stride > 1:
                g_tr = g_tr * float(event_stride)

            if pending_n is not None:
                seq_n.append(int(pending_n))
                g_vals.append(float(pending_g))
            if len(g_tr) > 1:
                seq_n.extend(src_n[:-1].astype(np.int64).tolist())
                g_vals.extend(g_tr[:-1].astype(np.float64).tolist())
            pending_n = int(src_n[-1])
            pending_g = float(g_tr[-1])

        prev_theta = float(theta_arr[-1])
        prev_chi = float(chi_arr[-1])
        prev_n = int(n_sel[-1])

    return seq_n, g_vals


def _stream_h_scaled(
    n_max: int,
    x_step: int,
    base: int,
    zeros: Sequence[float],
    u_mode: str,
    zero_kernel: str,
    kernel_scale: float,
    weights: Sequence[float],
    chunk_n: int,
) -> Tuple[List[int], List[float]]:
    xs = list(range(max(2, x_step), n_max + 1, x_step))
    if not xs:
        return [], []
    seq_n, g_vals = stream_weighted_events(
        n_max=n_max,
        base=base,
        zeros=zeros,
        u_mode=u_mode,
        zero_kernel=zero_kernel,
        kernel_scale=kernel_scale,
        weights=weights,
        chunk_n=chunk_n,
    )
    return xs, h_scaled_from_events(seq_n, g_vals, xs)


def write_svg(xs: Sequence[int], ys1: Sequence[float], ys2: Sequence[float], out_path: str, label1: str, label2: str) -> None:
    if not xs:
        return
    w, h, pad = 980, 520, 55
    x0, x1 = xs[0], xs[-1]
    y0 = min(min(ys1), min(ys2))
    y1 = max(max(ys1), max(ys2))
    sx = max(1e-12, x1 - x0)
    sy = max(1e-12, y1 - y0)

    def px(x: float) -> float:
        return pad + (x - x0) / sx * (w - 2 * pad)

    def py(y: float) -> float:
        return h - pad - (y - y0) / sy * (h - 2 * pad)

    p1 = " ".join(f"{px(x):.2f},{py(y):.2f}" for x, y in zip(xs, ys1))
    p2 = " ".join(f"{px(x):.2f},{py(y):.2f}" for x, y in zip(xs, ys2))
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">\n')
        f.write(f'<rect x="0" y="0" width="{w}" height="{h}" fill="#f8f8f6"/>\n')
        f.write('<text x="20" y="28" font-size="17" fill="#111827">Bridge: H(x) vs E(x)/sqrt(x)</text>\n')
        f.write(f'<polyline fill="none" stroke="#1f77b4" stroke-width="1.8" points="{p1}"/>\n')
        f.write(f'<polyline fill="none" stroke="#d97706" stroke-width="1.8" points="{p2}"/>\n')
        f.write(f'<text x="20" y="50" font-size="12" fill="#1f77b4">{label1}</text>\n')
        f.write(f'<text x="20" y="66" font-size="12" fill="#d97706">{label2}</text>\n')
        f.write("</svg>\n")


def main() -> None:
    ap = argparse.ArgumentParser(description="H(x) bridge probe from Candidate B")
    ap.add_argument("--n-max", type=int, default=150000)
    ap.add_argument("--x-step", type=int, default=500)
    ap.add_argument("--base", type=int, default=210)
    ap.add_argument("--weights", type=str, default="1,1,1,1", help="weights for Y11s,Y20,su2_trace,phase_vel")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--streaming", action="store_true", help="stream H(x) in chunks; avoids full point arrays")
    ap.add_argument("--chunk-n", type=int, default=20000, help="n-chunk for streaming mode")
    ap.add_argument("--output", type=str, default="research/output/hx_bridge_probe.json")
    args = ap.parse_args()

    w = [float(t.strip()) for t in args.weights.split(",") if t.strip()]
    if len(w) != 4:
        raise ValueError("weights must have 4 values")

    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros

    if args.streaming:
        xs, h_scaled = _stream_h_scaled(
            args.n_max,
            args.x_step,
            args.base,
            zeros,
            args.u_mode,
            args.zero_kernel,
            args.kernel_scale,
            w,
            args.chunk_n,
        )
    else:
        points = s4.manifold_points(args.n_max, zeros, args.u_mode, args.zero_kernel, args.kernel_scale)
        idx = [n for n in range(2, args.n_max + 1) if s4.gcd(n, args.base) == 1]
        ch = s4.candidate_series_for_base(points, idx)
        subset = ["Y11s", "Y20", "su2_trace", "phase_vel"]
        mlen = min(len(ch[k]) for k in subset)
        idx = idx[:mlen]

        g_series = [w[0] * ch["Y11s"][i] + w[1] * ch["Y20"][i] + w[2] * ch["su2_trace"][i] + w[3] * ch["phase_vel"][i] for i in range(mlen)]
        g_by_n = [0.0] * (args.n_max + 1)
        for n, gv in zip(idx, g_series):
            g_by_n[n] = gv

        h = [0.0] * (args.n_max + 1)
        for n in range(2, args.n_max + 1):
            h[n] = h[n - 1] + g_by_n[n]
        xs = list(range(max(2, args.x_step), args.n_max + 1, args.x_step))
        h_scaled = [h[x] / math.sqrt(x) for x in xs]

    psi_x = psi_exact_samples(xs)
    e_scaled = [(v - x) / math.sqrt(x) for x, v in zip(xs, psi_x)]
    c = corr(h_scaled, e_scaled)
    a, b, rmse = fit_linear(h_scaled, e_scaled)
    fit = [a * v + b for v in h_scaled]
    c_fit = corr(fit, e_scaled)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_max": args.n_max,
            "x_step": args.x_step,
            "base": args.base,
            "u_mode": args.u_mode,
            "weights": w,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "max_zeta_zeros": len(zeros),
            "streaming": bool(args.streaming),
            "chunk_n": args.chunk_n if args.streaming else 0,
        },
        "bridge_metrics": {
            "corr_hscaled_vs_escaled": c,
            "linear_fit_slope": a,
            "linear_fit_intercept": b,
            "linear_fit_rmse": rmse,
            "corr_fitted_vs_escaled": c_fit,
        },
        "series": {
            "x": xs,
            "h_scaled": h_scaled,
            "e_scaled": e_scaled,
            "fit_scaled": fit,
        },
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    svg1 = args.output.replace(".json", ".svg")
    write_svg(xs, h_scaled, e_scaled, svg1, "H(x)/sqrt(x)", "E(x)/sqrt(x)")
    svg2 = args.output.replace(".json", "_fit.svg")
    write_svg(xs, fit, e_scaled, svg2, "a*H(x)/sqrt(x)+b", "E(x)/sqrt(x)")

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        m = report["bridge_metrics"]
        f.write("# H(x) Bridge Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"- base: {args.base}\n")
        f.write(f"- u_mode: {args.u_mode}\n")
        f.write(f"- weights(Y11s,Y20,su2_trace,phase_vel): {w}\n")
        f.write(f"- corr(H/sqrt(x), E/sqrt(x)): {m['corr_hscaled_vs_escaled']:.6f}\n")
        f.write(f"- linear slope: {m['linear_fit_slope']:.6f}\n")
        f.write(f"- linear intercept: {m['linear_fit_intercept']:.6f}\n")
        f.write(f"- linear RMSE: {m['linear_fit_rmse']:.6f}\n")
        f.write(f"- corr(fit, E/sqrt(x)): {m['corr_fitted_vs_escaled']:.6f}\n")
        f.write(f"- svg: {svg1}\n")
        f.write(f"- fit svg: {svg2}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    print("corr_hscaled_vs_escaled:", round(c, 6))
    print("corr_fitted_vs_escaled:", round(c_fit, 6))


if __name__ == "__main__":
    main()
