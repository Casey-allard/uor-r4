"""Score the mantissa-norm rung (#2032 6114450465, FIT 6114483483)."""
import json, math, re, statistics as st
from pathlib import Path
R = Path(__file__).resolve().parent; B = R / "base"  # runs + evaluations: cloud-store claude icosian-mantissa-20261011 (base/ holds the reused bases)
LN2, BASIS, CTX = math.log(2), 2.837427, 256
def load(root, arm):
    out = {}
    for ev in sorted(root.glob(f"{arm}-s*-eval/evaluation.json")):
        s = int(re.search(r"-s(\d+)-eval", str(ev)).group(1)); d = json.loads(ev.read_text())
        b = d["block_nll"][64:320]; assert len(b) == 256
        out[s] = {"bpb": d["comparison"]["nll"] / LN2 / BASIS, "blocks": b}
    return out
runs = {"QM2": load(R, "QM2"), "QM1": load(R, "QM1"), "Q256c": load(B, "Q256c"), "QPf": load(B, "QPf"),
        "QP": load(B, "QP"), "D152": load(B, "D152")}
mean = {a: st.mean(r["bpb"] for r in v.values()) for a, v in runs.items()}
def sd(a): return st.stdev([r["bpb"] for r in runs[a].values()])
def bm(a): return [st.mean(r["blocks"][i] for r in runs[a].values()) / CTX for i in range(256)]
def paired(a, b): d = [(x - y) / LN2 / BASIS for x, y in zip(bm(a), bm(b))]; return st.stdev(d) / math.sqrt(256)
def noise(a):
    pooled = math.sqrt(sum((r["bpb"] - mean[x]) ** 2 for x in (a, "Q256c") for r in runs[x].values())
                       / sum(len(runs[x]) - 1 for x in (a, "Q256c")))
    return max(pooled, paired(a, "Q256c"), sd(a))
def row(a):
    n = noise(a); m = mean[a]
    if m <= mean["Q256c"] + n: return 1
    if m <= mean["D152"] - 2 * n: return 2
    if m <= mean["QPf"] + max(0.01, 2 * n): return 3
    if m < mean["QP"] - 2 * n: return 4
    return 5
res = {"bpb_per_seed": {a: {s: round(r["bpb"], 4) for s, r in v.items()} for a, v in runs.items()},
       "mean": {a: round(m, 4) for a, m in mean.items()},
       "noise": {a: round(noise(a), 4) for a in ("QM2", "QM1")},
       "vs_Q256c": {a: round(mean[a] - mean["Q256c"], 4) for a in ("QM2", "QM1", "QPf", "QP")},
       "row": {a: row(a) for a in ("QM2", "QM1")}}
print(json.dumps(res, indent=1)); (R / "result.json").write_text(json.dumps(res, indent=1))
