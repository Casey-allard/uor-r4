"""Score the residual icosian pair rung (#2032 6111364163, amended 6111530535, FIT 6111540766;
owner cap approval 6111589309). QP/QPf trained on CUDA, evaluated on the laptop."""
import json, math, re, statistics as st
from pathlib import Path
R = Path(__file__).resolve().parent          # QP, QPf evaluations: cloud-store claude icosian-pair-20261011
B = R.parent / "base"                        # Q256c, QSe, QSf: cloud-store icosian-qat-20261011; D152: quaternion-study-20261011
LN2, BASIS, CTX = math.log(2), 2.837427, 256
def load(root, arm):
    out = {}
    for ev in sorted(root.glob(f"{arm}-s*-eval/evaluation.json")):
        s = int(re.search(r"-s(\d+)-eval", str(ev)).group(1)); d = json.loads(ev.read_text())
        b = d["block_nll"][64:320]; assert len(b) == 256
        out[s] = {"bpb": d["comparison"]["nll"] / LN2 / BASIS, "blocks": b}
    return out
runs = {"QP": load(R, "QP"), "QPf": load(R, "QPf"), "Q256c": load(B, "Q256c"),
        "QSe": load(B, "QSe"), "QSf": load(B, "QSf"), "D152": load(B, "D152")}
mean = {a: st.mean(r["bpb"] for r in v.values()) for a, v in runs.items()}
def sd(a): return st.stdev([r["bpb"] for r in runs[a].values()])
pooled = math.sqrt(sum((r["bpb"] - mean[a]) ** 2 for a in ("QP", "Q256c") for r in runs[a].values())
                   / sum(len(runs[a]) - 1 for a in ("QP", "Q256c")))
def bm(a): return [st.mean(r["blocks"][i] for r in runs[a].values()) / CTX for i in range(256)]
def paired(a, b):
    d = [(x - y) / LN2 / BASIS for x, y in zip(bm(a), bm(b))]; return st.stdev(d) / math.sqrt(256)
noise = max(pooled, paired("QP", "Q256c"))
noise_f = max(noise, sd("QPf"), paired("QPf", "Q256c"))      # FIT nit applied: QPf's own spread
qp, qpf, base = mean["QP"], mean["QPf"], mean["Q256c"]
bar_f = base + max(0.02, 2 * noise_f)
if qp <= base + noise: row = 1
elif qp <= mean["D152"] - 2 * noise: row = 2
elif qp < mean["QSe"] - 2 * noise and qpf <= bar_f: row = 3
elif qp < mean["QSe"] - 2 * noise: row = 4
else: row = 5
res = {"bpb_per_seed": {a: {s: round(r["bpb"], 4) for s, r in v.items()} for a, v in runs.items()},
       "mean": {a: round(m, 4) for a, m in mean.items()}, "noise": round(noise, 4), "noise_QPf": round(noise_f, 4),
       "QP_minus_Q256c": round(qp - base, 4), "QPf_minus_Q256c": round(qpf - base, 4),
       "QP_minus_QSe": round(qp - mean["QSe"], 4), "QP_minus_D152": round(qp - mean["D152"], 4),
       "row3_QPf_bar": round(bar_f, 4), "row": row}
print(json.dumps(res, indent=1)); (R / "result.json").write_text(json.dumps(res, indent=1))
