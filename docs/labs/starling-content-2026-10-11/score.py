"""Score starling rung 3, content gate (#2032 6112273931, FIT 6112290954, blocker fix 6112460879)."""
import re, math, statistics as st, json
from pathlib import Path
D = Path(__file__).resolve().parent                       # SC7/SC7w logs: cloud-store claude starling-content-20261011
A = D.parent / "safetch" / "starling-anneal-20261011"     # rung 2 logs: cloud-store claude starling-anneal-20261011
L = [0.669, 0.373, 0.809]                                  # rung 1 L2 read
def se(x): return st.stdev(x) / math.sqrt(len(x))
def read(root, arm):
    acc, last, hits = [], [], {}
    for s in (1, 2, 3):
        t = (root / f"{arm}-s{s}.log").read_text()
        acc.append(float(re.search(r"final in-class fresh acc ([0-9.]+)", t).group(1)))
        last.append(max(int(x) for x in re.findall(r"^step (\d+) lr", t, re.M)))
        for step, h in re.findall(r"^step (\d+) starling hit rate ([0-9.]+)", t, re.M):
            hits.setdefault(int(step), []).append(float(h))
    curve = {k: st.mean(v) for k, v in sorted(hits.items()) if len(v) == 3}
    return acc, last, curve
out = {}
for arm, base in (("SC7", "SA7"), ("SC7w", "SA7w")):
    acc, last, curve = read(D, arm); sa, _, _ = read(A, base)
    m = st.mean(acc); h0 = curve[0]; H = max(0.15, 2 * h0)
    peak = max(v for k, v in curve.items() if k >= 100); final = curve[max(curve)]
    se_diff = math.sqrt(se(acc) ** 2 + se(L) ** 2)
    noise = max(st.stdev(acc), st.stdev(sa), math.sqrt(max(m, 1e-9) * (1 - m) / 2048))
    if m >= st.mean(L) - 2 * se_diff: row = 1
    elif m >= st.mean(sa) + max(0.05, 2 * noise): row = 2
    elif final >= H: row = 3
    else: row = 4
    out[arm] = dict(acc=acc, mean=round(m, 4), base_SA=sa, base_SA_mean=round(st.mean(sa), 4), last_step=last,
                    h0=round(h0, 4), H=round(H, 4), peak=round(peak, 4), final=round(final, 4),
                    row1_bar=round(st.mean(L) - 2 * se_diff, 4), row2_bar=round(st.mean(sa) + max(0.05, 2 * noise), 4),
                    row=row, curve={k: round(v, 4) for k, v in curve.items()})
out["decision_row"] = min(out[a]["row"] for a in ("SC7", "SC7w"))
print(json.dumps({a: {k: v for k, v in out[a].items() if k != "curve"} if isinstance(out[a], dict) else out[a] for a in out}, indent=1))
(D / "result.json").write_text(json.dumps(out, indent=1))
