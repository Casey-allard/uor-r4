"""Tabulate joint-read-geometry report roots (final 512-window metrics + development curve)."""
import json, math, sys, glob, os
print(f"{'run':20s} {'steps':>5s} {'nll':>7s} {'bpb':>6s} {'noread':>7s} {'effect':>6s} {'nr_mass':>7s} {'beta':>6s} {'delta':>6s} {'s/step':>6s}  development nll curve")
for d in sorted(glob.glob(sys.argv[1] + "/*/report.json")):
    r = json.load(open(d)); f = r["final"]; l = r.get("lorentz", {})
    beta = math.exp(l["read.lorentz_log_beta"][0]) if "read.lorentz_log_beta" in l else None
    off = l.get("read.lorentz_offset", [None])[0]
    curve = " ".join(f"{c['development']['nll_read']:.3f}" for c in r["curve"])
    b = "" if beta is None else f"{beta:.3f}"; o = "" if off is None else f"{off:.3f}"
    sps = r["seconds"]["training_updates"] / max(r["steps_completed"], 1)
    print(f"{os.path.basename(os.path.dirname(d)):20s} {r['steps_completed']:5d} {f['nll_read']:7.4f} {f['bits_per_byte_read']:6.4f} "
          f"{f['nll_no_read']:7.4f} {f['read_effect_nats']:6.3f} {f['no_read_mass']:7.3f} {b:>6s} {o:>6s} {sps:6.2f}  {curve}")
