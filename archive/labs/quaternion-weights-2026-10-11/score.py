"""Score the quaternion-weights study exactly as pre-registered on #2032
(6103994742 + amendments 6104011708, 6104031189, 6104037862). Reads only the
sealed run roots under local/q; prints a JSON summary and the decision."""
import json, math, re, statistics as st, sys
from pathlib import Path

Q = Path(__file__).resolve().parent
BASIS = 2.837427            # bytes per token, held-out stream e5f400b0
LN2 = math.log(2)
CTX = 256
ARMS = {"D256": (256, 4, 352, False), "Q256": (256, 4, 352, True),
        "D128": (128, 2, 176, False), "D152": (152, 2, 208, False)}
V = 4096

def bpb(nll): return nll / LN2 / BASIS

def cost(width, mlp, quat):
    w = width
    maps = 6 * (2 * w * w + (w // 4 + w) * w + w * w) + 2 * 4 * w * w + 8 * 3 * w * mlp
    stored = maps // 4 if quat else maps
    emb = V * w
    # float multiply-adds per token: a Hamilton block is 16 MACs, same as dense
    macs = maps + V * w + 2 * 2 * (CTX / 2) * w
    return {"map_weights": stored, "total_params": stored + emb + 0,
            "macs_per_token": int(macs), "weight_bytes_read_f32": 4 * (stored + emb)}

runs = {}
for arm in ARMS:
    for ev in sorted(Q.glob(f"{arm}-s*-eval/evaluation.json")):
        seed = int(re.search(r"-s(\d+)-eval", str(ev)).group(1))
        d = json.loads(ev.read_text())
        blocks = d["block_nll"][64:320]
        assert len(blocks) == 256, (ev, len(blocks))
        log = (Q / f"{arm}-s{seed}.train.log").read_text()
        dev = {int(m.group(2)): float(m.group(1)) for m in
               re.finditer(r'"dev":\{[^}]*"nll":([0-9.]+)[^}]*\}.*?"step":(\d+)', log)}
        runs.setdefault(arm, {})[seed] = {"bpb": bpb(d["comparison"]["nll"]),
                                          "blocks": blocks, "dev": dev}

missing = [a for a in ARMS if a not in runs]
if missing:
    sys.exit(f"missing arms: {missing}")

mean = {a: st.mean(r["bpb"] for r in runs[a].values()) for a in runs}
# pooled within-arm seed SD (BPB)
num = sum(sum((r["bpb"] - mean[a]) ** 2 for r in runs[a].values()) for a in runs)
dof = sum(len(runs[a]) - 1 for a in runs)
pooled = math.sqrt(num / dof)

def block_mean(a):  # per-block NLL per target, averaged over seeds
    rs = list(runs[a].values())
    return [st.mean(r["blocks"][i] for r in rs) / CTX for i in range(256)]

def paired_se(a, b):
    x, y = block_mean(a), block_mean(b)
    diff = [(x[i] - y[i]) / LN2 / BASIS for i in range(256)]
    return st.stdev(diff) / math.sqrt(256)

def noise(a, b): return max(pooled, paired_se(a, b))

pre_noise = noise("D128", "D256")
precondition = (mean["D128"] - mean["D256"]) >= 2 * pre_noise

n_q_d256, n_q_d152 = noise("Q256", "D256"), noise("Q256", "D152")
q, d256, d152 = mean["Q256"], mean["D256"], mean["D152"]
if not precondition:
    decision = "INCONCLUSIVE: the panel does not separate sizes (D128 - D256 < 2x noise); extend steps"
elif q <= d256 + n_q_d256:
    decision = "KEEP: Q256 <= D256 + noise (4x fewer map weights at no measurable loss)"
elif q <= d152 - 2 * n_q_d152:
    decision = "KEEP: Q256 <= D152 - 2x noise (structure beats an equal-total-parameter dense model)"
elif q <= d152 - n_q_d152:
    decision = "KEEP AS BASE, ITERATE: D152 - 2x noise < Q256 <= D152 - noise"
elif q <= d152 + n_q_d152:
    decision = "CONFIGURATION CLOSED: Q256 within noise of D152 (structure adds nothing over sizing here)"
else:
    decision = "CONFIGURATION CLOSED: Q256 > D152 + noise"

# curve settlement (dev panel, BPB; dev noise = pooled seed SD of final dev BPB)
def dev_mean(a, step): return st.mean(bpb(r["dev"][step]) for r in runs[a].values() if step in r["dev"])
dev_final = {a: [bpb(r["dev"][2000]) for r in runs[a].values() if 2000 in r["dev"]] for a in runs}
dnum = sum(sum((v - st.mean(vs)) ** 2 for v in vs) for vs in dev_final.values())
ddof = sum(len(vs) - 1 for vs in dev_final.values())
dev_noise = math.sqrt(dnum / ddof) if ddof else float("nan")
gaps = {s: dev_mean("Q256", s) - dev_mean("D152", s) for s in (1500, 1750, 2000)}
settled = (max(gaps.values()) - min(gaps.values())) <= dev_noise

out = {
    "bpb_by_seed": {a: {s: round(r["bpb"], 6) for s, r in sorted(runs[a].items())} for a in runs},
    "bpb_mean": {a: round(m, 6) for a, m in mean.items()},
    "pooled_seed_sd": round(pooled, 6),
    "paired_se": {k: round(paired_se(*k.split("-")), 6) for k in ("D128-D256", "Q256-D256", "Q256-D152")},
    "noise": {"D128-D256": round(pre_noise, 6), "Q256-D256": round(n_q_d256, 6), "Q256-D152": round(n_q_d152, 6)},
    "precondition_D128_minus_D256": round(mean["D128"] - mean["D256"], 6),
    "precondition_met": precondition,
    "deltas": {"Q256-D256": round(q - d256, 6), "Q256-D152": round(q - d152, 6), "Q256-D128": round(q - mean["D128"], 6)},
    "curve": {"dev_noise": round(dev_noise, 6), "gap_Q256_minus_D152": {k: round(v, 6) for k, v in gaps.items()}, "settled": settled},
    "cost": {a: cost(ARMS[a][0], ARMS[a][2], ARMS[a][3]) for a in ARMS},
    "decision": decision if settled else "NOT SETTLED: extend steps (curve still moving) — provisional: " + decision,
}
print(json.dumps(out, indent=1))
