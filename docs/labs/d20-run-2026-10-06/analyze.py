#!/usr/bin/env python3
"""D20 section 2 comparison analysis: read the sealed report roots, apply the
pre-registered thresholds, emit tables. Read-only."""
import json
import os
import statistics as st
import sys

REPORTS = os.path.expanduser("~/uor-r4-worktrees/reports")
BATTERY = os.path.expanduser(
    "~/uor-r4-local/mqar-bench/d20-control-20261006-0130-seq"
)
ROOTS = {
    # Parent's sequential battery (0130-seq) + this session's fill-in of
    # C6-matched-s1, which neither parent script schedules and whose original
    # root was killed unsealed at step 1300/1800.
    ("geometric", 1): f"{BATTERY}/R2-G-rrarra-s1",
    ("geometric", 2): f"{BATTERY}/R2-G-rrarra-s2",
    ("geometric", 3): f"{BATTERY}/R2-G-rrarra-s3",
    ("control", 1): f"{REPORTS}/d20run-20261006-0136-ctl6-rrarra-s1",
    ("control", 2): f"{REPORTS}/d20run-20261006-0152-ctl6-rrarra-s2",
    ("control", 3): f"{REPORTS}/d20run-20261006-0152-ctl6-rrarra-s3",
}
VOID_ROOTS = {
    "G-s1-original": "d20-control-20261006-0110-battery/G-rrarra-s1 (SIGTERM 9 s)",
    "G-s1r": "d20-control-20261006-0110-battery/_poisoned-20261006/G-rrarra-s1r (SIGTERM 565 s)",
    "C6-s1-original": "d20-control-20261006-0110-battery/_poisoned-20261006/C6-matched-s1 (SIGTERM step 1300)",
}
EXTRA_ROOTS = {
    # The depth-matched (layers=2) control: the probe for the scored-position
    # asymmetry, since it has the same 2 context-reading layers as rrarra.
    ("control2", 1): f"{BATTERY}/R2-C2-matched-s1",
    ("control2", 2): f"{BATTERY}/R2-C2-matched-s2",
    ("control2", 3): f"{BATTERY}/R2-C2-matched-s3",
    ("f2", 1): f"{BATTERY}/R2-F2-keyshift-s1",
    ("f2", 2): f"{BATTERY}/R2-F2-keyshift-s2",
    ("f2", 3): f"{BATTERY}/R2-F2-keyshift-s3",
}
BASELINE_ROOT = f"{BATTERY}/baselines"
THRESHOLD = 0.05


def dig(d, *keys, default=None):
    cur = d
    for k in keys:
        if not isinstance(cur, dict) or k not in cur:
            return default
        cur = cur[k]
    return cur


def load(arm, seed, root):
    path = os.path.join(root, "report.json")
    if not os.path.exists(path):
        return {"arm": arm, "seed": seed, "root": root, "missing": True}
    r = json.load(open(path))
    res = r.get("results", {})
    a = r.get("arm", {})
    ic = res.get("final_in_class_fresh_pairings", {})
    ho = res.get("final_held_out_class_pairings", {})
    firing = res.get("read_firing", {})
    reach = res.get("context_reachability", {})
    acc = res.get("context_access", {})
    pm = a.get("parameter_match") or {}
    curve = res.get("curve", [])
    last_nll = curve[-1].get("train_query_nll") if curve else None
    return {
        "arm": arm, "seed": seed, "root": root, "missing": False,
        "status": r.get("status"),
        "label": a.get("label"), "kind": a.get("kind"),
        "parameters": a.get("parameters"),
        "pm_reference": dig(pm, "reference_geometric", "parameters"),
        "pm_reference_pattern": dig(pm, "reference_geometric", "pattern"),
        "pm_reference_mlp": dig(pm, "reference_geometric", "mlp_hidden"),
        "pm_control_mlp": dig(pm, "control", "mlp_hidden"),
        "pm_residual": pm.get("residual_parameters"),
        "pm_residual_fraction": pm.get("residual_fraction"),
        "steps_completed": res.get("steps_completed"),
        "stopped_early": res.get("stopped_early_at_max_seconds"),
        "supervised_queries_seen": res.get("supervised_queries_seen"),
        "train_seconds": res.get("train_seconds"),
        "median_step_seconds": res.get("median_step_seconds"),
        "wall_seconds": res.get("wall_seconds"),
        "final_eval_seconds": res.get("final_eval_seconds"),
        "reachability_seconds": res.get("reachability_seconds"),
        "acc_in_class": ic.get("accuracy"),
        "acc_held_out": ho.get("accuracy"),
        "by_bucket": {k: v.get("accuracy") for k, v in (ic.get("by_bucket") or {}).items()},
        "nll_step1800": last_nll,
        "firing_status": firing.get("status"),
        "firing_rows": firing.get("rows_evaluated"),
        "firing_nonzero": firing.get("rows_nonzero_mass_on_key"),
        "firing_over_half": firing.get("rows_over_half_on_key"),
        "firing_mean_mass": firing.get("mean_mass_on_key"),
        "reach_rows": reach.get("rows"),
        "reach_changed": reach.get("rows_with_any_logit_change"),
        "reach_argmax_changed": reach.get("rows_whose_argmax_changed"),
        "reach_max_delta": reach.get("max_abs_logit_delta"),
        "pos_scored_query_token_mean": acc.get("positions_scored_per_query_token_mean"),
        "pos_scored_last": acc.get("positions_scored_at_last_position"),
        "config": r.get("argv", []),
    }


def arm_stats(rows, key):
    vals = [r[key] for r in rows if isinstance(r.get(key), (int, float))]
    if not vals:
        return None
    return {
        "n": len(vals), "values": vals,
        "mean": st.mean(vals),
        "sd": st.stdev(vals) if len(vals) > 1 else 0.0,
        "min": min(vals), "max": max(vals),
        "range": max(vals) - min(vals),
        "variance": st.variance(vals) if len(vals) > 1 else 0.0,
    }


def main():
    runs = [load(a, s, p) for (a, s), p in sorted(ROOTS.items())]
    geo = [r for r in runs if r["arm"] == "geometric" and not r["missing"]]
    ctl = [r for r in runs if r["arm"] == "control" and not r["missing"]]
    out = {"runs": runs, "baseline_root": BASELINE_ROOT}

    print("=" * 100)
    print("PER-RUN")
    print("=" * 100)
    hdr = f"{'arm':10} {'seed':>4} {'params':>9} {'mlp':>5} {'steps':>5} {'early':>5} {'train_s':>8} {'wall_s':>8} {'in_class':>9} {'held_out':>9}"
    print(hdr)
    for r in runs:
        if r["missing"]:
            print(f"{r['arm']:10} {r['seed']:>4}  MISSING {r['root']}")
            continue
        mlp = r["pm_control_mlp"] if r["arm"] == "control" else r["pm_reference_mlp"]
        print(f"{r['arm']:10} {r['seed']:>4} {r['parameters']:>9} {str(mlp):>5} "
              f"{r['steps_completed']:>5} {str(r['stopped_early']):>5} "
              f"{r['train_seconds']:>8.1f} {r['wall_seconds']:>8.1f} "
              f"{r['acc_in_class']:>9.4f} {r['acc_held_out']:>9.4f}")

    print()
    print("=" * 100)
    print("V1 equal training (need steps_completed==1800, stopped_early False, equal supervised_queries_seen)")
    for r in runs:
        if r["missing"]:
            continue
        ok = (r["steps_completed"] == 1800 and r["stopped_early"] is False)
        print(f"  {r['arm']:10} s{r['seed']}: steps={r['steps_completed']} early={r['stopped_early']} "
              f"queries={r['supervised_queries_seen']} -> {'OK' if ok else 'VOID'}")

    print()
    print("=" * 100)
    print("V2 firing counter on scored positions")
    for r in runs:
        if r["missing"]:
            continue
        if r["arm"] == "geometric":
            ok = r["firing_status"] == "MEASURED" and (r["firing_nonzero"] or 0) > 0
            print(f"  geo s{r['seed']}: read_firing status={r['firing_status']} rows={r['firing_rows']} "
                  f"nonzero={r['firing_nonzero']} over_half={r['firing_over_half']} -> {'OK' if ok else 'VOID'}")
        else:
            ok = (r["reach_changed"] or 0) > 0
            print(f"  ctl s{r['seed']}: read_firing={r['firing_status']!r} (UNAVAILABLE by construction); "
                  f"reachability changed={r['reach_changed']}/{r['reach_rows']} argmax_changed={r['reach_argmax_changed']} "
                  f"-> {'OK' if ok else 'VOID'}")

    print()
    print("=" * 100)
    print("V3 scored positions (asymmetry, not a voiding condition)")
    for r in runs:
        if r["missing"]:
            continue
        print(f"  {r['arm']:10} s{r['seed']}: mean positions scored per query token="
              f"{r['pos_scored_query_token_mean']}, at last position={r['pos_scored_last']}")

    print()
    print("=" * 100)
    print("PARAMETER MATCH")
    for r in ctl:
        print(f"  ctl s{r['seed']}: reference({r['pm_reference_pattern']}, mlp {r['pm_reference_mlp']})="
              f"{r['pm_reference']}  control(mlp {r['pm_control_mlp']})={r['parameters']}  "
              f"residual={r['pm_residual']} ({r['pm_residual_fraction']:+.6%})")
    for r in geo:
        print(f"  geo s{r['seed']}: parameters={r['parameters']}  parameter_match={r['pm_reference']}")

    print()
    print("=" * 100)
    print("PRIMARY METRIC: final in-class fresh-pairing accuracy")
    g = arm_stats(geo, "acc_in_class")
    c = arm_stats(ctl, "acc_in_class")
    for name, s in (("geometric", g), ("control", c)):
        if s:
            print(f"  {name:10} n={s['n']} values={['%.4f' % v for v in s['values']]} "
                  f"mean={s['mean']:.4f} sd={s['sd']:.4f} var={s['variance']:.6f} "
                  f"min={s['min']:.4f} max={s['max']:.4f} range={s['range']:.4f}")
    verdict = "INSUFFICIENT DATA"
    if g and c:
        delta = c["mean"] - g["mean"]
        separated_c = c["min"] > g["max"]
        separated_g = g["min"] > c["max"]
        if g["n"] < 3 or c["n"] < 3:
            # Pre-registration requires >=3 seeds per arm (D20 section 2
            # condition 2). A 1-seed-per-arm delta is NOT a verdict, and with
            # unequal seeds it is not even a matched pair.
            verdict = (f"NOT YET DECIDABLE: n(geometric)={g['n']}, n(control)={c['n']}; "
                       f"pre-registration requires 3 seeds per arm. Raw delta "
                       f"(control - geometric) = {delta:+.4f} on unmatched seeds is NOT the verdict.")
        elif abs(delta) <= THRESHOLD:
            verdict = f"TIE (|mean diff| {abs(delta):.4f} <= {THRESHOLD})"
        elif delta > THRESHOLD and separated_c:
            verdict = f"CONTROL WINS (mean diff {delta:+.4f} > {THRESHOLD}, per-seed separated)"
        elif delta < -THRESHOLD and separated_g:
            verdict = f"GEOMETRIC ARM WINS (mean diff {delta:+.4f} < -{THRESHOLD}, per-seed separated)"
        else:
            verdict = f"INCONCLUSIVE (mean diff {delta:+.4f} but per-seed ranges overlap)"
        out["delta_mean_control_minus_geometric"] = delta
    print(f"  VERDICT (pre-registered): {verdict}")
    out["verdict"] = verdict

    print()
    print("SECONDARY: per-bucket in-class accuracy")
    for r in runs:
        if not r["missing"]:
            print(f"  {r['arm']:10} s{r['seed']}: " +
                  " ".join(f"{k}={v:.4f}" for k, v in sorted(r["by_bucket"].items())))

    print()
    print("SECONDARY: held-out-class accuracy, step-1800 train NLL")
    for r in runs:
        if not r["missing"]:
            print(f"  {r['arm']:10} s{r['seed']}: held_out={r['acc_held_out']:.4f} nll={r['nll_step1800']:.4f}")

    for key, tag in (("acc_held_out", "held_out"), ("nll_step1800", "nll")):
        gs, cs = arm_stats(geo, key), arm_stats(ctl, key)
        if gs and cs:
            print(f"  mean {tag}: geometric={gs['mean']:.4f} (sd {gs['sd']:.4f}) "
                  f"control={cs['mean']:.4f} (sd {cs['sd']:.4f})")
            out[f"mean_{tag}"] = {"geometric": gs, "control": cs}

    json.dump(out, open(os.path.expanduser("~/uor-r4-worktrees/d20-run/analysis.json"), "w"), indent=2)
    print("\nwrote ~/uor-r4-worktrees/d20-run/analysis.json")


if __name__ == "__main__":
    sys.exit(main())
