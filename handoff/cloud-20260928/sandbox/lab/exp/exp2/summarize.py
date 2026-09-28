"""exp2: compact tables from index_eval.py JSON outputs.

For each index family and target budget f, pick the configuration with the highest admission recall whose actual
admitted fraction is at most tol*f (whole cells overshoot the budget), and print recall / final top-1 / actual
fraction / comparisons per query (advertisements + exactly scored items).
"""
import json, sys

FAMILIES = ["chunk[exact]", "chunk[l2]", "lsh", "codebook[exact]", "codebook[l2]", "codebook[code64]", "codebook[code256]",
            "kmeans[exact]", "kmeans[l2]", "kmeans[code64]", "kmeans[code256]", "kmeans+ball",
            "kmeans2[exact]", "kmeans2[l2]", "kmeans2[code64]", "kmeans2[code256]", "radir", "hamscan", "oracle"]


def best(rows, fam, j, f, tol):
    cands = [r for r in rows if r["index"] == fam and r["res"][j]["frac"] <= tol * f + 1e-9]
    if not cands:
        return None
    r = max(cands, key=lambda r: (r["res"][j]["recall"], r["res"][j]["final"], -r["res"][j]["comps"]))
    return r


def table(path, noise=0.0, tol=1.5, fams=FAMILIES, show_cfg=False):
    d = json.load(open(path))
    meta = d["meta"]
    lines = [f"### {path.split('/')[-1]}  kind={meta['kind']} dk={meta['dk']} task={meta['task']} "
             f"D_train={meta['D_train']} noise={noise}"]
    for blk in d["per_N"]:
        N = blk["N"]
        rows = [r for r in blk["summary"] if abs(r["noise"] - noise) < 1e-9]
        if not rows:
            continue
        budgets = [x["f"] for x in rows[0]["res"]]
        dg = blk["diag"]
        lines.append(f"N={N}  diag: " + ", ".join(f"{k}={v:.3f}" for k, v in dg.items() if v is not None))
        lines.append("| index | " + " | ".join(f"≈{int(f * 100)}% scored" for f in budgets) + " |")
        lines.append("|---|" + "---:|" * len(budgets))
        for fam in fams:
            cells = []
            for j, f in enumerate(budgets):
                r = best(rows, fam, j, f, tol)
                if r is None:
                    cells.append("—")
                    continue
                x = r["res"][j]
                s = f"{100 * x['recall']:.1f} / {100 * x['final']:.1f} ({100 * x['frac']:.1f}%, {x['comps']:.0f})"
                if show_cfg:
                    s += f" [{r['cfg']}]"
                cells.append(s)
            if all(c == "—" for c in cells):
                continue
            lines.append(f"| {fam} | " + " | ".join(cells) + " |")
    return "\n".join(lines)


def _main():
    args = sys.argv[1:]
    noise = 0.0; tol = 1.5; show = False
    paths = []
    for a in args:
        if a.startswith("noise="):
            noise = float(a[6:])
        elif a.startswith("tol="):
            tol = float(a[4:])
        elif a == "cfg":
            show = True
        else:
            paths.append(a)
    for p in paths:
        if "min99" in args:
            continue
        print(table(p, noise, tol, show_cfg=show))
        print()
    if "min99" in args:
        for p in [x for x in paths if x != "min99"]:
            print(min_cost(p, noise))
            print()


def min_cost(path, noise=0.0, thr=0.99, fams=None):
    """Cheapest configuration (comparisons per query, advertisements included) reaching admission recall >= thr."""
    d = json.load(open(path))
    out = [f"### {path.split('/')[-1]} kind={d['meta']['kind']} noise={noise} threshold={thr}"]
    for blk in d["per_N"]:
        N = blk["N"]
        rows = [r for r in blk["summary"] if abs(r["noise"] - noise) < 1e-9]
        fam_names = fams or sorted({r["index"] for r in rows})
        out.append(f"N={N}: full-scan acc={blk['diag'].get('full_acc', float('nan')):.3f}")
        out.append("| index | min comparisons/query | as % of N | scored | recall | final top-1 | config |")
        out.append("|---|---:|---:|---:|---:|---:|---|")
        for fam in fam_names:
            best_ = None
            for r in rows:
                if r["index"] != fam:
                    continue
                for x in r["res"]:
                    if x["recall"] >= thr and (best_ is None or x["comps"] < best_[1]["comps"]):
                        best_ = (r, x)
            if best_ is None:
                rr = [x["recall"] for r in rows if r["index"] == fam for x in r["res"]]
                out.append(f"| {fam} | not reached (max recall {100 * max(rr):.1f}%) | | | | | |" if rr else f"| {fam} | — | | | | | |")
                continue
            r, x = best_
            extra = " + N cheap code comparisons" if fam == "hamscan" else ""
            out.append(f"| {fam} | {x['comps']:.0f}{extra} | {100 * x['comps'] / N:.1f}% | {100 * x['frac']:.1f}% | "
                       f"{100 * x['recall']:.1f}% | {100 * x['final']:.1f}% | {r['cfg']} |")
    return "\n".join(out)


if __name__ == "__main__":
    _main()
