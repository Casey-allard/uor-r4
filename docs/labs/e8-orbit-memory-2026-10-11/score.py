"""Score E8 orbit memory rung 1 (#2032 6105777437, amended 6105824579; FIT recorded)."""
import json, math, re, statistics as st
from pathlib import Path
E = Path(__file__).resolve().parent            # evaluations: cloud-store claude e8-rung1-done-20261011, e8a-rung1-20261011
B = E.parent / "base"                          # reused D256 evaluations (quaternion study)
LN2, BASIS, CTX = math.log(2), 2.837427, 256
def load(root, arm):
    out = {}
    for ev in sorted(root.glob(f"{arm}-s*-eval/evaluation.json")):
        s = int(re.search(r"-s(\d+)-eval", str(ev)).group(1)); d = json.loads(ev.read_text())
        b = d["block_nll"][64:320]; assert len(b) == 256
        out[s] = {"bpb": d["comparison"]["nll"] / LN2 / BASIS, "blocks": b, "usage": d.get("e8_usage")}
    return out
runs = {"D256": load(B, "D256"), "E8h": load(E, "E8h"), "E8a": load(E, "E8a"), "P": load(E, "P")}
mean = {a: st.mean(r["bpb"] for r in v.values()) for a, v in runs.items()}
num = sum((r["bpb"] - mean[a]) ** 2 for a in runs for r in runs[a].values())
pooled = math.sqrt(num / sum(len(v) - 1 for v in runs.values()))
def bm(a): return [st.mean(r["blocks"][i] for r in runs[a].values()) / CTX for i in range(256)]
def paired(a, b):
    d = [(x - y) / LN2 / BASIS for x, y in zip(bm(a), bm(b))]; return st.stdev(d) / math.sqrt(256)
def noise(a, b): return max(pooled, paired(a, b))
d = mean["D256"]
if mean["E8a"] <= d + noise("E8a", "D256"): row = 1
elif mean["E8h"] <= d + noise("E8h", "D256"): row = 2
elif mean["E8h"] <= d + max(0.05, 2 * noise("E8h", "D256")): row = 3
else: row = 4
geometry_credit = mean["E8h"] <= mean["P"] + noise("E8h", "P")
usage = {}
for arm in ("E8h", "E8a"):
    for s, r in runs[arm].items():
        u = r["usage"] or {}
        heads = u.get("heads", [])
        usage[f"{arm}-s{s}"] = {
            "orbit_entropy_bits_mean": round(st.mean([(h["orbit_entropy_bits_a"] + h["orbit_entropy_bits_b"]) / 2 for h in heads]), 3) if heads else None,
            "slot_share_mean": round(st.mean([h["slot_share"] for h in heads]), 4) if heads else None,
            "max_entropy_bits": u.get("max_orbit_entropy_bits")}
res = {"bpb_per_seed": {a: {s: round(r["bpb"], 4) for s, r in v.items()} for a, v in runs.items()},
       "mean": {a: round(m, 4) for a, m in mean.items()}, "pooled_sd": round(pooled, 4),
       "noise_E8h_D256": round(noise("E8h", "D256"), 4), "noise_E8a_D256": round(noise("E8a", "D256"), 4),
       "noise_E8h_P": round(noise("E8h", "P"), 4),
       "E8h_minus_D256": round(mean["E8h"] - d, 4), "E8a_minus_D256": round(mean["E8a"] - d, 4),
       "E8h_minus_P": round(mean["E8h"] - mean["P"], 4), "P_minus_D256": round(mean["P"] - d, 4),
       "row": row, "geometry_credit": geometry_credit, "usage": usage}
print(json.dumps(res, indent=1)); (E / "result.json").write_text(json.dumps(res, indent=1))
