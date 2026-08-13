#!/usr/bin/env python3
"""A3 off-diagonal energy majorant via S^2 = E2 + OffDiag."""

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
    ap = argparse.ArgumentParser(description="A3 off-diagonal energy majorant")
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
    ap.add_argument("--a-max", type=float, default=6.0)
    ap.add_argument("--a-step", type=float, default=0.1)
    ap.add_argument("--safety-factor", type=float, default=1.1)
    ap.add_argument("--offdiag-safety", type=float, default=1.05)
    ap.add_argument("--cache-dir", type=str, default="research/cache/a3_offdiag_energy_majorant")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a3_offdiag_energy_majorant.json")
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
                h_abs = abs(s) / math.sqrt(max(1.0, float(x)))
                s2 = s * s
                offdiag = s2 - e2
                offdiag_ratio_pos = max(0.0, offdiag / max(1e-30, e2))
                rows.append(
                    {
                        "base": b,
                        "n_max": n,
                        "x": x,
                        "k": k,
                        "h_abs": h_abs,
                        "e2": e2,
                        "s2": s2,
                        "offdiag": offdiag,
                        "offdiag_ratio_pos": offdiag_ratio_pos,
                    }
                )

    train_rows = [r for r in rows if int(r["n_max"]) in set(train_n)]
    valid_rows = [r for r in rows if int(r["n_max"]) in set(valid_n)]
    eta_train = max((float(r["offdiag_ratio_pos"]) for r in train_rows), default=0.0)
    eta_guard = eta_train * max(1.0, args.offdiag_safety)
    eta_valid_max = max((float(r["offdiag_ratio_pos"]) for r in valid_rows), default=0.0)
    eta_valid_holds = eta_valid_max <= eta_guard + 1e-12

    for r in rows:
        h_upper_eta = math.sqrt(max(0.0, (1.0 + eta_guard) * float(r["e2"]) / max(1.0, float(r["x"]))))
        r["h_upper_eta"] = h_upper_eta
        r["det_gap_h_minus_h_upper_eta"] = float(r["h_abs"]) - h_upper_eta

    model = fit_log_envelope(
        rows=rows,
        train_n=train_n,
        valid_n=valid_n,
        target_key="h_upper_eta",
        a_min=args.a_min,
        a_max=args.a_max,
        a_step=args.a_step,
        safety_factor=args.safety_factor,
    )
    a_h = float(model["A"])
    c_h = float(model["C_uplifted"])

    def check_split(ns: set):
        v = 0
        gap = -1e18
        ratios = []
        det_v = 0
        det_gap = -1e18
        for r in rows:
            if int(r["n_max"]) not in ns:
                continue
            det_gap = max(det_gap, float(r["det_gap_h_minus_h_upper_eta"]))
            if float(r["det_gap_h_minus_h_upper_eta"]) > 1e-12:
                det_v += 1
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
            "deterministic_h_eta_holds": det_v == 0 and det_gap <= 1e-12,
            "deterministic_h_eta_violations": int(det_v),
            "deterministic_h_eta_max_gap": float(det_gap),
        }

    train_chk = check_split(set(train_n))
    valid_chk = check_split(set(valid_n))

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
            "safety_factor": max(1.0, args.safety_factor),
            "offdiag_safety": max(1.0, args.offdiag_safety),
            "cache_dir": args.cache_dir,
            "cache_enabled": use_cache,
            "cache_hits": cache_hits,
            "cache_misses": max(0, len(bases) - cache_hits),
        },
        "offdiag_guardrail": {
            "eta_train_max_pos_ratio": float(eta_train),
            "eta_guard": float(eta_guard),
            "eta_valid_max_pos_ratio": float(eta_valid_max),
            "eta_valid_holds": bool(eta_valid_holds),
        },
        "h_upper_envelope": {
            "A_H": a_h,
            "C_H_uplifted": c_h,
            "ratio_max_train": float(model["ratio_max_train"]),
            "ratio_max_valid": float(model["ratio_max_valid"]),
            "target": "|H| <= sqrt((1+eta_guard)*E2/x)",
        },
        "h_transfer_envelope": {
            "A_H": a_h,
            "C_H_from_density_transfer": c_h,
            "transfer_identity": "|H| <= sqrt((1+eta_guard)*E2/x) <= C_H(log x)^A_H",
        },
        "checks": {"train": train_chk, "valid": valid_chk},
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# A3 Offdiag-Energy Majorant\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        og = report["offdiag_guardrail"]
        he = report["h_upper_envelope"]
        f.write(f"- eta_train_max_pos_ratio: {og['eta_train_max_pos_ratio']:.12g}\n")
        f.write(f"- eta_guard: {og['eta_guard']:.12g}\n")
        f.write(f"- eta_valid_max_pos_ratio: {og['eta_valid_max_pos_ratio']:.12g}\n")
        f.write(f"- eta_valid_holds: {og['eta_valid_holds']}\n")
        f.write(f"- A_H: {he['A_H']:.6f}\n")
        f.write(f"- C_H_uplifted: {he['C_H_uplifted']:.12e}\n")
        f.write(f"- train holds: {report['checks']['train']['holds']}\n")
        f.write(f"- valid holds: {report['checks']['valid']['holds']}\n")
        f.write(f"- deterministic H<=sqrt((1+eta)E2/x) holds(train): {report['checks']['train']['deterministic_h_eta_holds']}\n")
        f.write(f"- deterministic H<=sqrt((1+eta)E2/x) holds(valid): {report['checks']['valid']['deterministic_h_eta_holds']}\n")
    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
