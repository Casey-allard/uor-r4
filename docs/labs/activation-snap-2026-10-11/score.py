"""Score exact activations trained in (#2032 6115101545, FIT 6115118695). All on CUDA."""
import json, math, re, statistics as st
from pathlib import Path
R = Path(__file__).resolve().parent  # evaluations: cloud-store claude activation-snap-20261011 (models on the pod volume)
LN2, BASIS, CTX = math.log(2), 2.837427, 256
def load(pattern):
    out = {}
    for ev in sorted(R.glob(pattern)):
        s = int(re.search(r"-s(\d+)-", str(ev)).group(1)); d = json.loads(ev.read_text())
        b = d["block_nll"][64:320]; assert len(b) == 256
        out[s] = {"bpb": d["comparison"]["nll"] / LN2 / BASIS, "blocks": b, "device": d.get("device"),
                  "snap": d.get("activation_snap")}
    return out
runs = {"QA": load("QA-s*-eval/evaluation.json"), "QM2g": load("QM2g-s*-eval/evaluation.json"),
        "QM2g-post": load("QM2g-s*-post/evaluation.json")}
mean = {a: st.mean(r["bpb"] for r in v.values()) for a, v in runs.items()}
def bm(a): return [st.mean(r["blocks"][i] for r in runs[a].values()) / CTX for i in range(256)]
pooled = math.sqrt(sum((r["bpb"] - mean[a]) ** 2 for a in ("QA", "QM2g") for r in runs[a].values())
                   / sum(len(runs[a]) - 1 for a in ("QA", "QM2g")))
paired = st.stdev([(x - y) / LN2 / BASIS for x, y in zip(bm("QA"), bm("QM2g"))]) / math.sqrt(256)
noise = max(pooled, paired, st.stdev([r["bpb"] for r in runs["QA"].values()]))
qa, base, post = mean["QA"], mean["QM2g"], mean["QM2g-post"]
row = 1 if qa <= base + noise else 2 if qa <= post - 2 * noise else 3
res = {"bpb_per_seed": {a: {s: round(r["bpb"], 4) for s, r in v.items()} for a, v in runs.items()},
       "labels": {a: sorted({(r["device"], r["snap"]) for r in v.values()}) for a, v in runs.items()},
       "mean": {a: round(m, 4) for a, m in mean.items()}, "noise": round(noise, 4),
       "QA_minus_QM2g": round(qa - base, 4), "post_minus_QM2g": round(post - base, 4),
       "QA_minus_post": round(qa - post, 4), "row": row}
print(json.dumps(res, indent=1)); (R / "result.json").write_text(json.dumps(res, indent=1))
