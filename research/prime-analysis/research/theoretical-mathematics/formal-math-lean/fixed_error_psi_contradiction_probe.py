#!/usr/bin/env python3
"""Empirical contradiction-route probe on E*(x)=psi(x)-x.

Math target (empirical finite-window form):
  lower:  |E*(x)| >= c_beta * x^beta      (on many tail peak points)
  upper:  |E*(x)| <= C_vk * x^(1/2) log(x)^2

For beta>1/2, compare growth rates and estimate crossover scale X where
  c_beta * X^beta = C_vk * X^(1/2) log(X)^2.

This is numeric research evidence only (not theorem-grade closure).
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Iterable, List, Sequence, Tuple

import numpy as np
import sympy as sp


@dataclass
class BetaContradictionRow:
    beta: float
    local_peak_count_tail: int
    c_beta_q10_tail_peaks: float
    c_beta_q25_tail_peaks: float
    c_beta_median_tail_peaks: float
    tail_max_ratio: float
    observed_tail_fraction_ge_q10: float
    crossover_x_q10_vs_endpoint: float | None
    crossover_x_q25_vs_endpoint: float | None
    crossover_x_median_vs_endpoint: float | None


def parse_float_list(text: str) -> List[float]:
    vals: List[float] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(float(tok))
    if not vals:
        raise ValueError("empty float list")
    return vals


def logspace_int(xmin: int, xmax: int, n: int) -> np.ndarray:
    vals = np.unique(np.floor(np.logspace(math.log10(xmin), math.log10(xmax), n)).astype(np.int64))
    vals = vals[(vals >= xmin) & (vals <= xmax)]
    if vals.size < 16:
        raise ValueError("not enough sample points")
    return vals


def _events_cache_path(cache_dir: Path, xmax: int) -> Path:
    return cache_dir / f"psi_events_xmax_{xmax}.json"


def _find_reusable_superset_cache(cache_dir: Path, xmax: int) -> Path | None:
    """Return the smallest cached xmax file with xmax' > xmax, if available."""
    cands: List[Tuple[int, Path]] = []
    for p in cache_dir.glob("psi_events_xmax_*.json"):
        stem = p.stem
        parts = stem.split("_")
        if len(parts) < 4:
            continue
        try:
            val = int(parts[-1])
        except ValueError:
            continue
        if val > xmax:
            cands.append((val, p))
    if not cands:
        return None
    cands.sort(key=lambda t: t[0])
    return cands[0][1]


def _materialize_from_superset_cache(cache_dir: Path, xmax: int, out_path: Path) -> List[Tuple[int, float]] | None:
    """Build exact-xmax cache by truncating a cached superset event list."""
    superset_path = _find_reusable_superset_cache(cache_dir, xmax)
    if superset_path is None:
        return None

    raw = json.loads(superset_path.read_text(encoding="utf-8"))
    src_events = raw.get("events", [])
    events: List[Tuple[int, float]] = []
    for t in src_events:
        n = int(t[0])
        if n > xmax:
            break
        events.append((n, float(t[1])))
    if not events:
        return None

    payload = {
        "xmax": int(xmax),
        "event_count": len(events),
        "derived_from_superset_cache": str(superset_path),
        "events": [[int(a), float(b)] for (a, b) in events],
    }
    out_path.write_text(json.dumps(payload), encoding="utf-8")
    return events


def psi_events(xmax: int, cache_dir: Path) -> List[Tuple[int, float]]:
    cache_dir.mkdir(parents=True, exist_ok=True)
    path = _events_cache_path(cache_dir, xmax)
    if path.exists():
        raw = json.loads(path.read_text(encoding="utf-8"))
        events = [(int(t[0]), float(t[1])) for t in raw.get("events", [])]
        return events

    reused = _materialize_from_superset_cache(cache_dir, xmax, path)
    if reused is not None:
        return reused

    events: List[Tuple[int, float]] = []
    for p in sp.primerange(2, xmax + 1):
        lp = math.log(float(p))
        pk = int(p)
        while pk <= xmax:
            events.append((pk, lp))
            if pk > xmax // p:
                break
            pk *= p
    events.sort(key=lambda t: t[0])
    payload = {
        "xmax": int(xmax),
        "event_count": len(events),
        "events": [[int(a), float(b)] for (a, b) in events],
    }
    path.write_text(json.dumps(payload), encoding="utf-8")
    return events


def psi_at_samples(xs: np.ndarray, events: Sequence[Tuple[int, float]]) -> np.ndarray:
    out = np.zeros(len(xs), dtype=np.float64)
    acc = 0.0
    j = 0
    m = len(events)
    for i, x in enumerate(xs):
        xi = int(x)
        while j < m and events[j][0] <= xi:
            acc += events[j][1]
            j += 1
        out[i] = acc
    return out


def local_peak_values(vals: np.ndarray) -> List[float]:
    if vals.size < 3:
        return []
    out: List[float] = []
    for i in range(1, vals.size - 1):
        if vals[i] >= vals[i - 1] and vals[i] >= vals[i + 1]:
            out.append(float(vals[i]))
    return out


def _upper_gap(x: float, c_beta: float, beta: float, c_endpoint: float) -> float:
    return c_beta * (x ** beta) - c_endpoint * (x ** 0.5) * (math.log(x) ** 2)


def crossover_x(
    c_beta: float,
    beta: float,
    c_endpoint: float,
    x_lo: float = 1.0e4,
    x_hi_cap: float = 1.0e300,
) -> float | None:
    if c_beta <= 0.0 or beta <= 0.5 or c_endpoint <= 0.0:
        return None

    f_lo = _upper_gap(x_lo, c_beta, beta, c_endpoint)
    if f_lo >= 0.0:
        return float(x_lo)

    hi = max(x_lo * 2.0, 1.0e6)
    while hi < x_hi_cap:
        f_hi = _upper_gap(hi, c_beta, beta, c_endpoint)
        if f_hi >= 0.0:
            break
        hi *= 2.0
    else:
        return None

    lo = x_lo
    for _ in range(160):
        mid = math.sqrt(lo * hi)
        f_mid = _upper_gap(mid, c_beta, beta, c_endpoint)
        if f_mid >= 0.0:
            hi = mid
        else:
            lo = mid
    return float(hi)


def quantiles(arr: np.ndarray, probs: Iterable[float]) -> List[float]:
    if arr.size == 0:
        return [0.0 for _ in probs]
    return [float(np.quantile(arr, p)) for p in probs]


def main() -> None:
    ap = argparse.ArgumentParser(description="Empirical contradiction probe for psi(x)-x")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=10_000_000)
    ap.add_argument("--samples", type=int, default=2200)
    ap.add_argument("--betas", type=str, default="0.55,0.58,0.60,0.62")
    ap.add_argument("--tail-frac", type=float, default=0.5)
    ap.add_argument("--peak-tail-quantile", type=float, default=0.10)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_contradiction_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    betas = parse_float_list(args.betas)
    xs = logspace_int(args.xmin, args.xmax, args.samples)
    events = psi_events(args.xmax, cache_dir=Path(args.cache_dir))
    psi_vals = psi_at_samples(xs, events)
    e_vals = psi_vals - xs.astype(np.float64)
    abs_e = np.abs(e_vals)

    x_float = xs.astype(np.float64)
    endpoint_ratio = abs_e / (np.sqrt(x_float) * np.power(np.log(x_float), 2))
    c_endpoint = float(np.max(endpoint_ratio))
    endpoint_p95 = float(np.quantile(endpoint_ratio, 0.95))

    tail_start = int((1.0 - float(args.tail_frac)) * len(xs))
    tail_start = max(1, min(tail_start, len(xs) - 2))

    rows: List[BetaContradictionRow] = []
    for beta in betas:
        if beta <= 0.5:
            continue
        ratio = abs_e / np.power(x_float, beta)
        ratio_tail = ratio[tail_start:]
        peaks_all = np.array(local_peak_values(ratio_tail), dtype=np.float64)
        if peaks_all.size == 0:
            q10 = q25 = q50 = 0.0
        else:
            q10, q25, q50 = quantiles(peaks_all, [0.10, 0.25, 0.50])

        frac_ge_q10 = float(np.mean(ratio_tail >= q10)) if q10 > 0 else 0.0
        row = BetaContradictionRow(
            beta=float(beta),
            local_peak_count_tail=int(peaks_all.size),
            c_beta_q10_tail_peaks=float(q10),
            c_beta_q25_tail_peaks=float(q25),
            c_beta_median_tail_peaks=float(q50),
            tail_max_ratio=float(np.max(ratio_tail)),
            observed_tail_fraction_ge_q10=float(frac_ge_q10),
            crossover_x_q10_vs_endpoint=crossover_x(q10, beta, c_endpoint, x_lo=float(args.xmin)),
            crossover_x_q25_vs_endpoint=crossover_x(q25, beta, c_endpoint, x_lo=float(args.xmin)),
            crossover_x_median_vs_endpoint=crossover_x(q50, beta, c_endpoint, x_lo=float(args.xmin)),
        )
        rows.append(row)

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": int(args.xmin),
            "xmax": int(args.xmax),
            "sample_count": int(len(xs)),
            "betas": betas,
            "tail_frac": float(args.tail_frac),
            "peak_tail_quantile": float(args.peak_tail_quantile),
            "event_count": int(len(events)),
            "cache_dir": str(args.cache_dir),
        },
        "series": {
            "sign_changes": int(np.sum(np.sign(e_vals[1:]) != np.sign(e_vals[:-1]))),
            "positive_count": int(np.sum(e_vals > 0)),
            "negative_count": int(np.sum(e_vals < 0)),
            "min_E": float(np.min(e_vals)),
            "max_E": float(np.max(e_vals)),
        },
        "endpoint_upper_envelope": {
            "form": "|E*(x)| <= C_endpoint * x^(1/2) * (log x)^2",
            "C_endpoint_sup_window": c_endpoint,
            "C_endpoint_p95_window": endpoint_p95,
        },
        "beta_rows": [asdict(r) for r in rows],
        "interpretation": {
            "note": (
                "Finite-window empirical contradiction-map only. "
                "To upgrade to theorem form, replace empirical c_beta and C_endpoint "
                "with theorem-grade constants and asymptotic quantifiers."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Fixed Error psi(x)-x Contradiction Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Samples: `{len(xs)}`")
    lines.append(f"- Prime-power events used for psi(x): `{len(events)}`")
    lines.append(f"- Sign changes: `{payload['series']['sign_changes']}`")
    lines.append("")
    lines.append("## Endpoint Envelope (Empirical Window)")
    lines.append("")
    lines.append(f"- `C_endpoint_sup_window = {c_endpoint:.9e}` in `|E*(x)| <= C_endpoint * x^(1/2) * (log x)^2`")
    lines.append(f"- `C_endpoint_p95_window = {endpoint_p95:.9e}`")
    lines.append("")
    lines.append("## Lower-vs-Upper Contradiction Map")
    lines.append("")
    lines.append("| beta | q10 tail-peak c_beta | q25 c_beta | median c_beta | tail max | crossover(q10) | crossover(q25) | crossover(median) |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        def fmt_x(x: float | None) -> str:
            return "NA" if x is None else f"{x:.6e}"

        lines.append(
            f"| {r.beta:.4f} | {r.c_beta_q10_tail_peaks:.6e} | "
            f"{r.c_beta_q25_tail_peaks:.6e} | {r.c_beta_median_tail_peaks:.6e} | "
            f"{r.tail_max_ratio:.6e} | {fmt_x(r.crossover_x_q10_vs_endpoint)} | "
            f"{fmt_x(r.crossover_x_q25_vs_endpoint)} | {fmt_x(r.crossover_x_median_vs_endpoint)} |"
        )
    lines.append("")
    lines.append(payload["interpretation"]["note"])
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
