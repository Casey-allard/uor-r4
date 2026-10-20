"""exp2: report tables from index_eval.py outputs (runs/ev_*.json)."""
import json, os, sys

R = os.path.join(os.path.dirname(os.path.abspath(__file__)), "runs")
GROUPS = [
    ("storage-order chunks", ["chunk[exact]", "chunk[l2]"]),
    ("sign buckets (LSH)", ["lsh"]),
    ("fixed codebook", ["codebook[exact]", "codebook[l2]"]),
    ("k-means cells", ["kmeans[exact]", "kmeans[l2]"]),
    ("two-level cells", ["kmeans2[exact]", "kmeans2[l2]"]),
    ("cells, 256-bit code routing", ["kmeans[code256]", "codebook[code256]", "kmeans2[code256]"]),
    ("core + k-means", ["kmeans+core[exact]", "kmeans+core[l2]"]),
    ("core + direction cells", ["radir"]),
    ("flat Hamming scan", ["hamscan"]),
    ("exact top-f (ceiling)", ["oracle"]),
]


def load(name):
    p = os.path.join(R, name + ".json")
    return json.load(open(p)) if os.path.exists(p) else None


def rows_of(blk, noise):
    return [r for r in blk["summary"] if abs(r["noise"] - noise) < 1e-9]


def best_at(rows, fams, f, tol=1.5, extra=None):
    best = None
    for r in rows:
        if r["index"] not in fams:
            continue
        if extra and not extra(r):
            continue
        for x in r["res"]:
            if abs(x["f"] - f) > 1e-9 or x["frac"] > tol * f + 1e-9:
                continue
            key = (x["recall"], x["final"], -x["comps"])
            if best is None or key > best[0]:
                best = (key, r, x)
    return best


def min_at(rows, fams, thr=0.99, extra=None):
    best = None
    for r in rows:
        if r["index"] not in fams:
            continue
        if extra and not extra(r):
            continue
        for x in r["res"]:
            if x["recall"] >= thr and (best is None or x["comps"] < best[1]["comps"]):
                best = (r, x)
    return best


def max_recall(rows, fams, maxfrac=0.11):
    rr = [x["recall"] for r in rows if r["index"] in fams for x in r["res"] if x["frac"] <= maxfrac]
    return max(rr) if rr else None


def t_min99(names, noise=0.0, groups=None):
    groups = groups or [g for g in GROUPS if g[0] not in ("core + k-means", "core + direction cells")]
    out = ["| geometry | N | full scan | " + " | ".join(g for g, _ in groups if g != "exact top-f (ceiling)") + " |",
           "|---|---:|---:|" + "---|" * (len(groups) - 1)]
    for nm in names:
        d = load(nm)
        if d is None:
            continue
        for blk in d["per_N"]:
            N = blk["N"]; rows = rows_of(blk, noise)
            cells = []
            for g, fams in groups:
                if g == "exact top-f (ceiling)":
                    continue
                b = min_at(rows, fams)
                if b is None:
                    mr = max_recall(rows, fams)
                    cells.append("—" if mr is None else f"max {100 * mr:.1f}%")
                else:
                    r, x = b
                    s = f"**{x['comps']:.0f}** ({100 * x['comps'] / N:.1f}%; {100 * x['frac']:.1f}% scored)"
                    if g == "flat Hamming scan":
                        s = f"{x['comps']:.0f} exact + {N} code ({100 * x['frac']:.1f}% scored)"
                    cells.append(s)
            fa = blk["diag"].get("full_acc")
            if noise > 0:
                fa = blk["diag"].get(f"full_acc_noise{noise}")
            out.append(f"| {d['meta']['kind']} | {N} | {100 * fa:.1f}% | " + " | ".join(cells) + " |")
    return "\n".join(out)


def t_recall_at(name, N, f, noises, groups=None, tol=1.5):
    d = load(name)
    if d is None:
        return ""
    blk = [b for b in d["per_N"] if b["N"] == N][0]
    groups = groups or GROUPS
    hdr = "| index (" + d["meta"]["kind"] + f", N={N}, ≈{int(100 * f)}% scored) | " + " | ".join(
        f"noise {nz} (full scan {100 * (blk['diag'].get('full_acc') if nz == 0 else blk['diag'].get(f'full_acc_noise{nz}')):.1f}%)"
        for nz in noises) + " |"
    out = [hdr, "|---|" + "---:|" * len(noises)]
    for g, fams in groups:
        cells = []
        for nz in noises:
            b = best_at(rows_of(blk, nz), fams, f, tol)
            if b is None:
                cells.append("—")
            else:
                _, r, x = b
                cells.append(f"{100 * x['recall']:.1f} / {100 * x['final']:.1f} ({100 * x['frac']:.1f}%, {x['comps']:.0f})")
        if all(c == "—" for c in cells):
            continue
        out.append(f"| {g} | " + " | ".join(cells) + " |")
    return "\n".join(out)


def t_tree(names, fs=(0.01, 0.02, 0.03, 0.05, 0.10), groups=None, tol=1.5):
    groups = groups or GROUPS
    out = []
    for nm in names:
        d = load(nm)
        if d is None:
            continue
        for blk in d["per_N"]:
            N = blk["N"]; rows = rows_of(blk, 0.0); dg = blk["diag"]
            out.append(f"\n**{nm}** (kind={d['meta']['kind']}, order={d['args']['order']}, N={N}): full-scan acc "
                       f"{100 * dg['full_acc']:.1f}%, non-root {100 * (dg.get('full_acc_nonroot') or 0):.1f}%, "
                       f"non-root share {100 * dg['nonroot_frac']:.0f}%"
                       + (f", radius-depth corr {dg['radius_depth_corr']:.2f}" if dg.get("radius_depth_corr") is not None else ""))
            out.append("| index | " + " | ".join(f"≈{int(100 * f)}%" for f in fs) + " |")
            out.append("|---|" + "---:|" * len(fs))
            for g, fams in groups:
                cells = []
                for f in fs:
                    b = best_at(rows, fams, f, tol)
                    cells.append("—" if b is None else
                                 f"{100 * b[2]['recall']:.1f} / {100 * b[2]['final']:.1f} ({b[2]['comps']:.0f})")
                if all(c == "—" for c in cells):
                    continue
                out.append(f"| {g} | " + " | ".join(cells) + " |")
    return "\n".join(out)


if __name__ == "__main__":
    what = sys.argv[1] if len(sys.argv) > 1 else "all"
    if what in ("all", "min99"):
        print("## min comparisons for >=99% admission, near-copy queries")
        print(t_min99(["ev_mq_dot", "ev_mq_hyp", "ev_mq_ham"]))
        for nz in (0.5, 1.0):
            print(f"\n## min comparisons for >=99% admission, query noise {nz}")
            print(t_min99(["ev_mq_dot", "ev_mq_hyp", "ev_mq_ham"], noise=nz))
    if what in ("all", "noise"):
        for nm in ("ev_mq_dot", "ev_mq_hyp", "ev_mq_ham"):
            print()
            print(t_recall_at(nm, 16384, 0.02, (0.0, 0.5, 1.0, 1.5)))
    if what in ("all", "tree"):
        print(t_tree(["ev_ct512_hyp", "ev_ct512_hyp_dfs", "ev_ct_dot", "ev_ct_hyp", "ev_st8_hyp", "ev_st8_hyp_dfs",
                      "ev_st8_dot"]))
    if what in ("all", "route"):
        print(t_tree(["ev_mq_dotroute"]) if load("ev_mq_dotroute") else "")
