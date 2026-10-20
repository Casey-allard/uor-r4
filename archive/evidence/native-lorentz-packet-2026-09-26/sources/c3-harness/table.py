"""Tabulate c3 run JSONs: final (256-window) metrics and the step curve."""
import json, math, sys, glob, os
rows = []
for path in sorted(glob.glob(sys.argv[1] + "/*.json")):
    d = json.load(open(path))
    f = d["final"]; l = d.get("lorentz", {})
    beta = math.exp(l["read.lorentz_log_beta"][0]) if "read.lorentz_log_beta" in l else None
    off = l.get("read.lorentz_offset", [None])[0]
    curve = " ".join(f"{c['eval']['nll_read']:.3f}" for c in d["curve"])
    rows.append((os.path.basename(path)[:-5], d["steps_completed"], f["nll_read"], f["bpb_read"], f["nll_no_read"],
                 f["read_effect_nats"], f["no_read_mass"], beta, off, d["train_seconds"] / max(d["steps_completed"], 1), curve))
print(f"{'run':18s} {'steps':>5s} {'nll':>7s} {'bpb':>6s} {'noread':>7s} {'effect':>6s} {'nr_mass':>7s} {'beta':>6s} {'delta':>6s} {'s/step':>6s}  curve(every eval)")
for r in rows:
    b = "" if r[7] is None else f"{r[7]:.3f}"; o = "" if r[8] is None else f"{r[8]:.3f}"
    print(f"{r[0]:18s} {r[1]:5d} {r[2]:7.4f} {r[3]:6.4f} {r[4]:7.4f} {r[5]:6.3f} {r[6]:7.3f} {b:>6s} {o:>6s} {r[9]:6.2f}  {r[10]}")
