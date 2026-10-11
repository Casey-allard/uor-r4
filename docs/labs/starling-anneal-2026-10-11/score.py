"""Score starling rung 2 (#2032 6107021590 + amendments 6107070727, 6107090319; FIT 6107098703; device amendment 6110389805)."""
import re, math, statistics as st, json
from pathlib import Path
D = Path(__file__).resolve().parent  # run logs: cloud-store claude starling-anneal-20261011
L = [0.669, 0.373, 0.809]; S7 = [0.013, 0.008, 0.006]
def se(x): return st.stdev(x) / math.sqrt(len(x))
out = {}
for arm in ("SA7", "SA7w"):
    acc, held, hits, last = [], [], {}, []
    for s in (1, 2, 3):
        t = (D / f"{arm}-s{s}.log").read_text()
        m = re.search(r"final in-class fresh acc ([0-9.]+).*held-out-class acc ([0-9.]+)", t)
        acc.append(float(m.group(1))); held.append(float(m.group(2)))
        last.append(max(int(x) for x in re.findall(r"^step (\d+) lr", t, re.M)))
        for step, h in re.findall(r"^step (\d+) starling hit rate ([0-9.]+)", t, re.M):
            hits.setdefault(int(step), []).append(float(h))
    curve = {k: st.mean(v) for k, v in sorted(hits.items()) if len(v) == 3}
    h0 = curve[0]; H = max(0.15, 2 * h0)
    peak = max(v for k, v in curve.items() if k >= 100); final = curve[max(curve)]
    m = st.mean(acc)
    se_diff = math.sqrt(se(acc) ** 2 + se(L) ** 2)
    noise = max(st.stdev(acc), st.stdev(S7), math.sqrt(m * (1 - m) / 2048))
    if m >= st.mean(L) - 2 * se_diff: row = 1
    elif m >= st.mean(S7) + max(0.05, 2 * noise): row = 2
    elif final >= H: row = 3
    elif peak >= H: row = 4
    else: row = 5
    out[arm] = dict(acc=acc, mean=m, held=held, last_step=last, h0=h0, H=H, peak=peak, final=final,
                    se_diff=se_diff, row2_bar=st.mean(S7) + max(0.05, 2 * noise), row=row,
                    curve={k: round(v, 4) for k, v in curve.items()})
out["decision_row"] = min(out[a]["row"] for a in ("SA7", "SA7w"))
print(json.dumps(out, indent=1))
(D / "result.json").write_text(json.dumps(out, indent=1))
