"""Print text tables from results_<dataset>.json and serving_<dataset>.json."""
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TARGETS = ["1.5", "2.0", "2.5", "3.0", "4.0"]
ORDER = ["SC-pow2", "SC-opt", "SCchan", "TQ-qr", "TQ-rht", "TQprod-qr", "P2-none", "P2-rht",
         "PQ-none", "PQ-rht", "GS4-none-vec", "GS4-rht-vec", "GS4-none-abs", "GS4-rht-abs",
         "GS4rnd-rht-vec", "GS4skm-rht-vec", "VQ4km-rht", "D4-none", "D4-rht", "E8-none", "E8-rht"]


def load(ds):
    import glob
    files = sorted(glob.glob(os.path.join(HERE, f"results_{ds}_*.json")))
    if not files:
        return None
    r = dict(dataset=ds, stage1={}, stage2={})
    for fn in files:
        x = json.load(open(fn))
        r["stage1"].update(x["stage1"])
        r["stage2"].update(x["stage2"])
    return r


def db(x, ref):
    return 10 * math.log10(ref / x) if x > 0 and ref > 0 else float("nan")


def mse_table(r, ref="TQ-qr"):
    s2 = r["stage2"]
    lines = [f"relMSE = mean ||x-xhat||^2/||x||^2 at matched total bits/dim (actual bits in brackets); "
             f"dB = gain vs {ref} (positive = better)"]
    head = f"{'family':16s}" + "".join(f"{t:>22s}" for t in TARGETS)
    lines.append(head)
    for fam in ORDER:
        if fam not in s2:
            continue
        row = f"{fam:16s}"
        for t in TARGETS:
            m = s2[fam].get(t)
            if m is None:
                row += f"{'-':>22s}"
                continue
            refm = s2.get(ref, {}).get(t)
            g = db(m["relmse"], refm["relmse"]) if refm else float("nan")
            row += f"{m['relmse']:9.4f}[{m['bits_per_dim']:.2f}]{g:+6.2f}dB"
        lines.append(row)
    return "\n".join(lines)


def metric_table(r, t, keys=("recall10", "r1at10", "ip_abs", "ip_slope", "cos")):
    s2 = r["stage2"]
    lines = [f"at {t} bits/dim: " + ", ".join(keys) + ", config"]
    for fam in ORDER:
        m = s2.get(fam, {}).get(t)
        if m is None:
            continue
        lines.append(f"{fam:16s} " + " ".join(f"{m[k]:8.4f}" for k in keys) + f"  [{m['bits_per_dim']:.2f}] {m['config']}")
    return "\n".join(lines)


def envelope(r, fam):
    pts = sorted(r["stage1"][fam], key=lambda s: s["bits_per_dim"])
    env = []
    best = float("inf")
    for p in pts:
        if p["relmse"] < best - 1e-12:
            env.append(p)
            best = p["relmse"]
    return env


def main(dsets):
    for ds in dsets:
        r = load(ds)
        if r is None:
            continue
        print("=" * 100)
        print(f"DATASET {ds}")
        print(mse_table(r))
        for t in ("2.0", "3.0"):
            print(metric_table(r, t))
        print()
    for ds in dsets:
        p = os.path.join(HERE, f"serving_{ds}.json")
        if not os.path.exists(p):
            continue
        s = json.load(open(p))
        print("=" * 100)
        print(f"SERVING {ds}: 2I Gram values {s['gram_2I_values']}")
        for row in s["rows"]:
            print(f"{row['keys']:28s} {row['key_bits_per_dim']:5.2f}  {row['query']:38s} rec10={row['recall10']:.3f} "
                  f"r1@10={row['r1at10']:.2f} ip_abs={row['ip_abs']:.3f} slope={row['ip_slope']:.3f} tv={row['attn_tv']:.3f}")


if __name__ == "__main__":
    main(sys.argv[1:] or ["gauss", "t3", "mvt3", "outlier", "aniso"])


KEY = ["SC-pow2", "SCchan", "TQ-qr", "TQ-rht", "P2-rht", "PQ-rht", "PQ-none", "GS4-rht-vec", "GS4-none-vec",
       "GS4-rht-abs", "GS4-none-abs", "GS4rnd-rht-vec", "VQ4km-rht", "D4-rht", "D4-none", "E8-rht", "E8-none"]


def cross(metric="relmse", targets=("2.0", "3.0"), dsets=("gauss", "t3", "mvt3", "outlier", "aniso"), fmt="{:.4f}"):
    rs = {d: load(d) for d in dsets}
    head = f"{'family':15s}" + "".join(f"{d + '@' + t:>13s}" for d in dsets for t in targets)
    print(f"[{metric}]")
    print(head)
    for fam in KEY:
        row = f"{fam:15s}"
        for d in dsets:
            for t in targets:
                m = (rs[d] or {}).get("stage2", {}).get(fam, {}).get(t) if rs[d] else None
                row += f"{(fmt.format(m[metric]) if m else '-'):>13s}"
        print(row)
    print()
