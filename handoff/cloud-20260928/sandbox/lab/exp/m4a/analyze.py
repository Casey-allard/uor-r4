"""M4a: tabulate arms and apply the work card's fixed decision rules."""
import json, sys
r = json.load(open(sys.argv[1]))
arms = {(a["geometry"], a["seed"]): a for a in r["arms"]}
seeds = sorted({s for _, s in arms})
geoms = ["dot", "euclid", "lorentz"]
print("| geometry | seed | NLL change | gate | beta | mean |k|^2 | cache has target | read share for 99% | top-8 mass |")
print("|---|---:|---:|---:|---:|---:|---:|---:|---:|")
for g in geoms:
    for s in seeds:
        f = arms[(g, s)]["final"]
        print(f"| {g} | {s} | {f['delta_nats']:+.4f} | {f['mean_gate']:.3f} | {f['beta']:.2f} | {f['mean_key_norm_sq']:.1f} | "
              f"{f['cache_covers_target']:.3f} | {f['read_share_for_99_percent']:.4f} | {f['top8_mass']:.3f} |")
d = {g: [arms[(g, s)]["final"]["delta_nats"] for s in seeds] for g in geoms}
backbone = arms[("dot", seeds[0])]["final"]["backbone_nll"]
print(f"\nbackbone NLL at query positions: {backbone:.4f}")
def beats(a, b):
    diffs = [x - y for x, y in zip(d[a], d[b])]
    same_sign = all(x < 0 for x in diffs)
    spread = max(max(d[a]) - min(d[a]), max(d[b]) - min(d[b]))
    mean = sum(diffs) / len(diffs)
    return same_sign and -mean > spread, mean, diffs, spread
def beats_backbone(g):
    same_sign = all(x < 0 for x in d[g]); spread = max(d[g]) - min(d[g]); mean = sum(d[g]) / len(d[g])
    return same_sign and -mean > spread, mean, spread
for g in geoms:
    ok, mean, spread = beats_backbone(g)
    print(f"{g} vs backbone: mean {mean:+.4f}, spread {spread:.4f}, beats: {ok}")
for a, b in [("lorentz", "euclid"), ("lorentz", "dot"), ("euclid", "dot")]:
    ok, mean, diffs, spread = beats(a, b)
    print(f"{a} vs {b}: mean {mean:+.4f}, per seed {[round(x, 4) for x in diffs]}, spread {spread:.4f}, beats: {ok}")
any_beats = any(beats_backbone(g)[0] for g in geoms)
if not any_beats:
    print("DECISION 1: no arm beats the backbone; M4a stops (negative).")
else:
    lor_e, lor_d, euc_d = beats("lorentz", "euclid")[0], beats("lorentz", "dot")[0], beats("euclid", "dot")[0]
    if lor_e and lor_d:
        print("DECISION 2: Lorentz beats euclid and dot -> Lorentz keys for M4b (integer arcosh).")
    elif (not beats("lorentz", "euclid")[0] and not beats("euclid", "lorentz")[0]) and lor_d and euc_d:
        print("DECISION 3: distance matters, curvature does not -> adopt euclid.")
    elif not beats("euclid", "dot")[0] and not beats("lorentz", "dot")[0] and (beats("dot", "euclid")[0] or beats("dot", "lorentz")[0] or True):
        print("DECISION 4 (or unresolved): dot at least as good as both -> adopt dot if it beats the backbone.")
    else:
        print("UNRESOLVED between geometries: report; adopt the arm that beats the backbone with the smallest read share only if NLL within noise.")
