"""Score the trained-in icosian rung (#2032 6106445338, amended 6106573972, FIT 6106609174,
device amendment 6110389805: arms trained on CUDA; base Q256c = Q256 re-run on CUDA)."""
import json, math, re, statistics as st
from pathlib import Path
G = Path(__file__).resolve().parent  # GPU-trained runs, laptop-evaluated: cloud-store claude icosian-qat-20261011
C = Path("/Users/casey.allard/uor-r4/.worktrees/claude-quat-exp/local/q")  # CPU-trained study runs
LN2, BASIS = math.log(2), 2.837427
def load(root, arm):
    out = {}
    for ev in sorted(root.glob(f"{arm}-s*-eval/evaluation.json")):
        s = int(re.search(r"-s(\d+)-eval", str(ev)).group(1))
        d = json.loads(ev.read_text())
        b = d["block_nll"][64:320]; assert len(b) == 256
        out[s] = {"bpb": d["comparison"]["nll"] / LN2 / BASIS, "blocks": b}
    return out
runs = {"Q256c": load(G, "Q256c"), "QSe": load(G, "QSe"), "QSf": load(G, "QSf"),
        "Q256cpu": load(C, "Q256"), "D152": load(C, "D152"), "D256": load(C, "D256")}
mean = {a: st.mean(r["bpb"] for r in v.values()) for a, v in runs.items()}
num = sum((r["bpb"] - mean[a]) ** 2 for a in ("Q256c", "QSe") for r in runs[a].values())
pooled = math.sqrt(num / sum(len(runs[a]) - 1 for a in ("Q256c", "QSe")))
def bm(a): return [st.mean(r["blocks"][i] for r in runs[a].values()) / 256 for i in range(256)]  # per target (ctx 256), as the study
diff = [x - y for x, y in zip(bm("QSe"), bm("Q256c"))]
paired = st.stdev(diff) / math.sqrt(256) / LN2 / BASIS
noise = max(pooled, paired)
QW = mean["Q256cpu"] + 0.141            # the probe's post-hoc snap, reused (row 3 of the probe)
qse = mean["QSe"]
if qse <= mean["Q256c"] + noise: row = 1
elif qse <= mean["D152"] - 2 * noise: row = 2
elif qse < QW - 2 * noise: row = 3
else: row = 4
res = {"bpb_per_seed": {a: {s: round(r["bpb"], 4) for s, r in v.items()} for a, v in runs.items()},
       "mean": {a: round(m, 4) for a, m in mean.items()}, "pooled_sd": round(pooled, 4),
       "paired_se": round(paired, 4), "noise": round(noise, 4), "Q256_W": round(QW, 4),
       "QSe_minus_Q256c": round(qse - mean["Q256c"], 4), "QSf_minus_Q256c": round(mean["QSf"] - mean["Q256c"], 4),
       "QSe_minus_D152": round(qse - mean["D152"], 4), "row": row}
print(json.dumps(res, indent=1)); (G / "result.json").write_text(json.dumps(res, indent=1))
