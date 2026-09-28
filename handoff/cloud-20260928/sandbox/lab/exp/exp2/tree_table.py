"""exp2: compact tree table with actual scored fractions (best config per family at a target budget, <=1.5x)."""
import sys
import make_tables as m

FAM = [("core + direction", ["radir"]), ("core + k-means", ["kmeans+core[exact]", "kmeans+core[l2]"]),
       ("k-means (no core)", ["kmeans[exact]", "kmeans[l2]"]), ("fixed codebook", ["codebook[exact]", "codebook[l2]"]),
       ("flat sign-code scan", ["hamscan"]), ("storage-order chunks", ["chunk[exact]", "chunk[l2]"]), ("ceiling", ["oracle"])]


def row(nm, N, f=0.02, label=None):
    d = m.load(nm)
    if d is None:
        return None
    blk = [b for b in d["per_N"] if b["N"] == N]
    if not blk:
        return None
    blk = blk[0]; rows = m.rows_of(blk, 0.0); dg = blk["diag"]
    cells = []
    for g, fams in FAM:
        extra = (lambda r: not r["cfg"].startswith("core=0.000")) if g == "core + direction" else None
        b = m.best_at(rows, fams, f, 1.5, extra)
        if b is None:
            cells.append("—")
        else:
            x = b[2]
            cells.append(f"{100 * x['recall']:.1f}" + (f" / {100 * x['final']:.1f}" if g == "core + direction" else "")
                         + f" ({100 * x['frac']:.1f}%)")
    lab = label or nm
    return (f"| {lab} | {N:,} | {100 * dg['full_acc']:.1f} ({100 * (dg.get('full_acc_nonroot') or 0):.1f}) | "
            + " | ".join(cells) + " |")


if __name__ == "__main__":
    f = float(sys.argv[1]) if len(sys.argv) > 1 else 0.02
    print("| keys (training), order | N | full scan (non-root) | " + " | ".join(g for g, _ in FAM) + " |")
    print("|---|---:|---:|" + "---:|" * len(FAM))
    for nm, lab in [("ev_ct512_hyp", "code tree, hyp (512), random"), ("ev_ct512_hyp_dfs", "code tree, hyp (512), source order"),
                    ("ev_ct_hyp", "code tree, hyp (48), random"), ("ev_ct_dot", "code tree, dot (48), random"),
                    ("ev_st8_hyp", "synthetic, hyp (256, anc-biased), random"), ("ev_st8_hyp_dfs", "synthetic, hyp, DFS order"),
                    ("ev_st8_dot", "synthetic, dot (256, anc-biased), random")]:
        for N in (1024, 2048, 4096, 16384):
            r = row(nm, N, f, lab)
            if r:
                print(r)
