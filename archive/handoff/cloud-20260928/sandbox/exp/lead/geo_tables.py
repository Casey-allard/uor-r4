"""Build markdown tables from the geometric-attention results (lead scratch)."""
import json, os

G = "geo"


def load(path):
    try:
        return json.load(open(path))
    except Exception:
        return None


def pct(x):
    return "—" if x is None else f"{100 * x:.1f}%"


def recall_table():
    rows = []
    names = {"dot": "Dot-product softmax (float)", "cos": "Spherical geodesic (cosine, float)",
             "hyp": "Hyperbolic geodesic (arcosh, float)", "ham": "Hamming: 64 sign bits (XOR+popcount)",
             "e8": "E8 block codes (8 blocks of 240 roots ≈ 63 bits)"}
    src = {"dot": "smoke2_dot.json", "ham": "smoke2_ham.json", "e8": "smoke2_e8.json",
           "cos": f"{G}/cos_dk64.json", "hyp": f"{G}/hyp_dk64.json"}
    out = ["| Scoring (64 dims or ≈64 bits per key) | Recall, 128 pairs | 256 pairs | 512 pairs (1,024 candidates) | Hard top-1 at 512 pairs |",
           "|---|---:|---:|---:|---:|"]
    for k in ("dot", "cos", "hyp", "ham", "e8"):
        r = load(src[k])
        if r is None:
            out.append(f"| {names[k]} | pending | | | |"); continue
        out.append(f"| {names[k]} | {pct(r.get('acc_D128'))} | {pct(r.get('acc_D256'))} | {pct(r.get('acc_D512'))} | {pct(r.get('hard_acc_D512'))} |")
    b = load("smoke2_bind.json")
    if b:
        out.append(f"| Superposed binding memory, 64-wide state (no attention) | {pct(b.get('acc_D128'))} | {pct(b.get('acc_D256'))} | {pct(b.get('acc_D512'))} | — |")
    return "\n".join(out)


def precision_table():
    out = ["| Width per key | Dot (float) | Cosine (float) | Hyperbolic (float) | Hamming (bits) | E8 blocks (≈bits) |",
           "|---:|---:|---:|---:|---:|---:|"]
    for dk in (8, 16, 32):
        cells = []
        for k in ("dot", "cos", "hyp", "ham", "e8"):
            r = load(f"{G}/{k}_dk{dk}.json")
            cells.append("pending" if r is None else f"{pct(r.get('acc_D512'))} / {pct(r.get('hard_acc_D512'))}")
        out.append(f"| {dk} | " + " | ".join(cells) + " |")
    return "\n".join(out)


def routing_rows(r, label):
    rows = []
    for x in r.get("routing", []):
        cfg = f"c={x['c']}, keep {x['r']}" + (f"; super-chunks of {x['g']}, keep {x['r1']}" if "g" in x else "")
        rows.append((label, cfg, x["comparisons_per_query"], x["acc"]))
    return rows


def routing_table():
    cols = [("c=16, keep 1", 16, 1, None), ("c=16, keep 4", 16, 4, None), ("c=8, keep 4", 8, 4, None),
            ("two-level", 16, 4, 8)]
    out = ["| Advertisement (codes, bundle) | 16-token chunks, keep 1 (80 comparisons) | keep 4 (128) | 8-token chunks, keep 4 (160) | two-level: super-chunks of 8, keep 2, then 4 chunks (88) |",
           "|---|---:|---:|---:|---:|"]
    specs = [("smoke2_dot.json", "Dot, 64 dims, mean bundle"), (f"{G}/dot_dk256.json", "Dot, 256 dims, mean bundle"),
             (f"{G}/dot_dk1024.json", "Dot, 1,024 dims, mean bundle"),
             ("smoke2_ham.json", "Hamming, 64 bits, majority bundle"), (f"{G}/ham_dk256.json", "Hamming, 256 bits, majority bundle"),
             (f"{G}/ham_dk1024.json", "Hamming, 1,024 bits, majority bundle"),
             ("smoke2_e8.json", "E8, 64 dims, mean bundle"), (f"{G}/e8_dk256.json", "E8, 256 dims, mean bundle"),
             (f"{G}/e8_dk1024.json", "E8, 1,024 dims, mean bundle"),
             (f"{G}/hyp_dk64_route.json", "Hyperbolic, 64 dims, Lorentzian-centroid bundle"),
             (f"{G}/rt_dot_dk64.json", "Dot, 64 dims, **advertisements trained**"),
             (f"{G}/rt_hyp_dk64.json", "Hyperbolic, 64 dims, **advertisements trained**"),
             (f"{G}/rt_e8_dk64.json", "E8, 64 dims, **advertisements trained**"),
             (f"{G}/rt_ham_dk64.json", "Hamming, 64 bits, **advertisements trained**"),
             (f"{G}/rt_ham_dk256.json", "Hamming, 256 bits, **advertisements trained**")]
    for path, label in specs:
        r = load(path)
        if r is None:
            out.append(f"| {label} | pending | | | |"); continue
        cells = []
        for _, c, rr, g in cols:
            hit = [x for x in r.get("routing", []) if x["c"] == c and x["r"] == rr and x.get("g") == g]
            cells.append(pct(hit[0]["acc"]) if hit else "—")
        out.append(f"| {label} | " + " | ".join(cells) + " |")
    return "\n".join(out)


def bind_table():
    out = ["| State width | Recall, 64 pairs | 128 pairs | 256 pairs | 512 pairs |", "|---:|---:|---:|---:|---:|"]
    for dk, path in ((64, "smoke2_bind.json"), (256, f"{G}/bind_dk256.json"), (1024, f"{G}/bind_dk1024.json"), (4096, f"{G}/bind_dk4096.json")):
        r = load(path)
        if r is None:
            out.append(f"| {dk} | pending | | | |"); continue
        out.append(f"| {dk} | {pct(r.get('acc_D64'))} | {pct(r.get('acc_D128'))} | {pct(r.get('acc_D256'))} | {pct(r.get('acc_D512'))} |")
    return "\n".join(out)


def text_table():
    out = ["| Layers 1–2 → layer 3 | Parameters | Bits/byte, 512-byte windows | At the 128-byte training length |", "|---|---:|---:|---:|"]
    base = [("full_diag_s0.json", "diagonal → diagonal (no attention), seed 0"), ("full_diag_s1.json", "diagonal → diagonal (no attention), seed 1")]
    for path, label in base:
        r = load(path)
        if r:
            out.append(f"| {label} | {r['params']:,} | {r['valid_bpb']:.3f} | {r.get('valid_bpb_at_train_T', float('nan')):.3f} |" if r.get('valid_bpb_at_train_T') else f"| {label} | {r['params']:,} | {r['valid_bpb']:.3f} | — |")
    for k, label in (("dot", "dot-product softmax, no positions"), ("cos", "spherical (cosine)"), ("hyp", "hyperbolic geodesic (arcosh)"), ("ham", "Hamming sign codes"), ("e8", "E8 block codes")):
        r = load(f"full_hyb_{k}_s0.json")
        if r is None:
            out.append(f"| diagonal → {label} | pending | | |"); continue
        out.append(f"| diagonal → {label} | {r['params']:,} | {r['valid_bpb']:.3f} | {r['valid_bpb_at_train_T']:.3f} |")
    return "\n".join(out)


def tree_table():
    out = ["| Width per key | Dot (float) | Cosine (float) | Hyperbolic (arcosh, float) | Hamming (bits) | E8 blocks |",
           "|---:|---:|---:|---:|---:|---:|"]
    for dk in (2, 4, 8, 16):
        cells = []
        for k in ("dot", "cos", "hyp", "ham", "e8"):
            if k in ("ham", "e8") and dk < 8:
                cells.append("—"); continue
            r = load(f"tree/{k}_dk{dk}.json")
            cells.append("pending" if r is None else
                         f"{pct(r.get('acc_D48'))} · {pct(r.get('acc_D96'))} · {pct(r.get('acc_D192'))}")
        out.append(f"| {dk} | " + " | ".join(cells) + " |")
    return "\n".join(out)


if __name__ == "__main__":
    for name, fn in (("RECALL", recall_table), ("PRECISION (acc soft / hard at 512 pairs)", precision_table),
                     ("ROUTING", routing_table), ("BIND", bind_table), ("TEXT", text_table), ("TREE", tree_table)):
        print(f"### {name}\n{fn()}\n")
