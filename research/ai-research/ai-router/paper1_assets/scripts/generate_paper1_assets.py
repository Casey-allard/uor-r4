#!/usr/bin/env python3
"""Generate canonical Paper 1 assets from analysis JSON files.

Outputs:
  - paper1_assets/data/scaling_points.csv
  - paper1_assets/data/scaling_fit_params.json
  - paper1_assets/tables/ptb_results.csv
  - paper1_assets/tables/wt2_results.csv
  - paper1_assets/tables/cross_dataset_summary.csv
  - paper1_assets/tables/key_control_null_result.csv
  - paper1_assets/figures/paper1_scaling_main.pdf
  - paper1_assets/figures/paper1_scaling_main.png
"""

from __future__ import annotations

import csv
import json
import math
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import matplotlib.ticker as mticker


ROOT = Path(__file__).resolve().parents[2]
ANALYSIS = ROOT / "router-research" / "results" / "analysis"
OUT = ROOT / "paper1_assets"
DATA = OUT / "data"
FIGURES = OUT / "figures"
TABLES = OUT / "tables"


def load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def ensure_dirs() -> None:
    for path in (DATA, FIGURES, TABLES):
        path.mkdir(parents=True, exist_ok=True)


def write_csv(path: Path, rows: list[dict], fieldnames: list[str]) -> None:
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def round_or_blank(value, digits=4):
    if value is None:
        return ""
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        return value
    return round(float(value), digits)


def load_scaling_points() -> tuple[list[dict], dict]:
    inc0168 = load_json(ANALYSIS / "inc0168_norm_geometry.json")
    inc0170 = load_json(ANALYSIS / "inc0170_large_k.json")

    anchor_rows = []
    for row in inc0168["ExpA_L2"]["routing_results"]:
        if row["mode"] != "TRANS":
            continue
        if row["variant"] not in {"ORIG", "PERM"}:
            continue
        if row["K"] < 25:
            continue
        anchor_rows.append(
            {
                "source_analysis": "inc0168_norm_geometry.json",
                "source_experiment": "ExpA_L2",
                "K": row["K"],
                "variant": row["variant"],
                "effective_bucket_count": row["effective_bucket_count"],
                "routing_gini": row["routing_gini"],
                "sector_entropy": row["sector_entropy"],
            }
        )

    large_k_rows = []
    for source_key, variant in (("raw_orig", "ORIG"), ("raw_perm", "PERM")):
        for row in inc0170[source_key]:
            if row["K"] <= 400:
                continue
            large_k_rows.append(
                {
                    "source_analysis": "inc0170_large_k.json",
                    "source_experiment": source_key,
                    "K": row["K"],
                    "variant": variant,
                    "effective_bucket_count": row["effective_bucket_count"],
                    "routing_gini": row["routing_gini"],
                    "sector_entropy": row["sector_entropy"],
                }
            )

    scaling_rows = sorted(anchor_rows + large_k_rows, key=lambda r: (r["variant"], r["K"]))
    fit_params = {
        "source_analysis": "inc0170_large_k.json",
        "anchor_fit_reference": inc0170["inc0169_reference"],
        "large_k_fit_reference": inc0170["scaling_exponents"],
        "dense_reference": {"alpha": 1.0, "c": 1.0, "r2": 1.0},
    }
    return scaling_rows, fit_params


def export_scaling_data() -> tuple[list[dict], dict]:
    scaling_rows, fit_params = load_scaling_points()
    write_csv(
        DATA / "scaling_points.csv",
        scaling_rows,
        [
            "source_analysis",
            "source_experiment",
            "K",
            "variant",
            "effective_bucket_count",
            "routing_gini",
            "sector_entropy",
        ],
    )
    with (DATA / "scaling_fit_params.json").open("w", encoding="utf-8") as f:
        json.dump(fit_params, f, indent=2)
    return scaling_rows, fit_params


def generate_scaling_figure(scaling_rows: list[dict], fit_params: dict) -> None:
    orig_points = sorted([r for r in scaling_rows if r["variant"] == "ORIG"], key=lambda r: r["K"])
    perm_points = sorted([r for r in scaling_rows if r["variant"] == "PERM"], key=lambda r: r["K"])

    k_values = [r["K"] for r in scaling_rows]
    k_min, k_max = min(k_values), max(k_values)
    k_fit = [10 ** x for x in frange(math.log10(k_min), math.log10(k_max), 300)]

    hopf_anchor = fit_params["anchor_fit_reference"]["TRANS_ORIG"]
    perm_anchor = fit_params["anchor_fit_reference"]["TRANS_PERM"]

    hopf_fit = [hopf_anchor["c"] * (k ** hopf_anchor["alpha"]) for k in k_fit]
    perm_fit = [perm_anchor["c"] * (k ** perm_anchor["alpha"]) for k in k_fit]
    dense_fit = k_fit

    compression_by_k = load_json(ANALYSIS / "inc0170_large_k.json")["compression_by_K"]
    k_400_ratio = compression_by_k["400"]["eff_ratio_PERM_over_ORIG"]
    k_5000_ratio = compression_by_k["5000"]["eff_ratio_PERM_over_ORIG"]

    fig, ax = plt.subplots(figsize=(6.6, 4.6))

    ax.plot(k_fit, dense_fit, color="#8a8a8a", lw=1.4, linestyle="--", label=r"Dense ($\propto K^{1.0}$)")
    ax.plot(k_fit, perm_fit, color="#d97706", lw=1.6, linestyle="-.", label=rf"Permuted ($\propto K^{{{perm_anchor['alpha']:.3f}}}$)")
    ax.plot(k_fit, hopf_fit, color="#1d4ed8", lw=2.1, linestyle="-", label=rf"Hopf ($\propto K^{{{hopf_anchor['alpha']:.3f}}}$)")

    ax.scatter(
        [r["K"] for r in perm_points],
        [r["effective_bucket_count"] for r in perm_points],
        color="#d97706",
        edgecolors="#92400e",
        s=32,
        marker="s",
        zorder=4,
    )
    ax.scatter(
        [r["K"] for r in orig_points],
        [r["effective_bucket_count"] for r in orig_points],
        color="#1d4ed8",
        edgecolors="#1e3a8a",
        s=34,
        marker="o",
        zorder=5,
    )

    x_400 = next(r["K"] for r in orig_points if r["K"] == 400)
    y_400 = next(r["effective_bucket_count"] for r in orig_points if r["K"] == 400)
    ax.annotate(
        f"{k_400_ratio:.2f}x at K=400",
        xy=(x_400, y_400),
        xytext=(470, 160),
        fontsize=8,
        color="#1d4ed8",
        arrowprops={"arrowstyle": "->", "color": "#1d4ed8", "lw": 0.8},
    )
    ax.text(
        2250,
        720,
        f"Large-K persistence:\n{k_5000_ratio:.2f}x at K=5000",
        fontsize=8,
        color="#374151",
        ha="left",
        va="center",
        bbox={"boxstyle": "round,pad=0.25", "fc": "white", "ec": "#d1d5db", "alpha": 0.95},
    )

    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlabel("Routing budget K")
    ax.set_ylabel("Effective routing footprint")
    ax.set_title("Fixed Hopf routing remains sublinear across routing budgets")
    ax.grid(which="both", linestyle=":", linewidth=0.5, alpha=0.6)
    ax.legend(loc="upper left", framealpha=0.95, fontsize=8.5)
    ax.set_xticks([25, 50, 75, 100, 150, 200, 400, 600, 1000, 2000, 3000, 5000])
    ax.get_xaxis().set_major_formatter(mticker.ScalarFormatter())
    plt.tight_layout()

    fig.savefig(FIGURES / "paper1_scaling_main.pdf", bbox_inches="tight")
    fig.savefig(FIGURES / "paper1_scaling_main.png", bbox_inches="tight", dpi=180)
    plt.close(fig)


def param_count_by_condition(results: list[dict], condition: str):
    for row in results:
        if row["condition"] == condition:
            return row.get("n_params")
    return None


def export_ptb_table() -> None:
    inc0171 = load_json(ANALYSIS / "inc0171_lm_integration.json")
    rows = []
    for condition in ("BASELINE", "HOPF", "PERMUTED"):
        summary = inc0171["summary"][condition]
        rows.append(
            {
                "condition": condition,
                "mean_val_ppl": round_or_blank(summary["mean_val_ppl"]),
                "mean_eff_buckets": round_or_blank(summary["mean_eff_buckets"]),
                "eff_ratio_vs_dense": round_or_blank(64.0 / summary["mean_eff_buckets"]),
                "n_seeds": summary["n_seeds"],
                "n_params": param_count_by_condition(inc0171["results"], condition),
            }
        )
    write_csv(
        TABLES / "ptb_results.csv",
        rows,
        ["condition", "mean_val_ppl", "mean_eff_buckets", "eff_ratio_vs_dense", "n_seeds", "n_params"],
    )


def export_wt2_table() -> None:
    inc0173 = load_json(ANALYSIS / "inc0173_wt2_confirm.json")
    rows = []
    for condition in ("BASELINE", "HOPF", "PERMUTED"):
        summary = inc0173["summary"][condition]
        rows.append(
            {
                "condition": condition,
                "mean_val_ppl": round_or_blank(summary["mean_val_ppl"]),
                "std_val_ppl": round_or_blank(summary.get("std_val_ppl")),
                "mean_eff_buckets": round_or_blank(summary["mean_eff_buckets"]),
                "std_eff_buckets": round_or_blank(summary.get("std_eff_buckets")),
                "eff_ratio_vs_dense": round_or_blank(64.0 / summary["mean_eff_buckets"]),
                "n_seeds": summary["n_seeds"],
                "n_params": param_count_by_condition(inc0173["results"], condition),
            }
        )
    write_csv(
        TABLES / "wt2_results.csv",
        rows,
        [
            "condition",
            "mean_val_ppl",
            "std_val_ppl",
            "mean_eff_buckets",
            "std_eff_buckets",
            "eff_ratio_vs_dense",
            "n_seeds",
            "n_params",
        ],
    )


def export_cross_dataset_table() -> None:
    inc0171 = load_json(ANALYSIS / "inc0171_lm_integration.json")
    inc0173 = load_json(ANALYSIS / "inc0173_wt2_confirm.json")

    ptb_hopf = inc0171["summary"]["HOPF"]
    ptb_perm = inc0171["summary"]["PERMUTED"]
    ptb_ratio = inc0171["ppl_ratio_hopf_vs_baseline"]
    ptb_null = inc0171["summary"]["HOPF"]["mean_val_ppl"] - inc0171["summary"]["PERMUTED"]["mean_val_ppl"]

    wt2_hopf = inc0173["summary"]["HOPF"]
    wt2_ratio = inc0173["ppl_ratio_hopf_vs_baseline"]
    wt2_null = inc0173["hopf_vs_permuted_delta_ppl"]

    rows = [
        {
            "metric": "hopf_vs_baseline_ppl_ratio",
            "PTB": round_or_blank(ptb_ratio),
            "WT2": round_or_blank(wt2_ratio),
        },
        {
            "metric": "hopf_vs_permuted_delta_ppl",
            "PTB": round_or_blank(ptb_null),
            "WT2": round_or_blank(wt2_null),
        },
        {
            "metric": "hopf_eff_ratio_vs_dense",
            "PTB": round_or_blank(64.0 / ptb_hopf["mean_eff_buckets"]),
            "WT2": round_or_blank(64.0 / wt2_hopf["mean_eff_buckets"]),
        },
        {
            "metric": "baseline_mean_eff_buckets",
            "PTB": round_or_blank(inc0171["summary"]["BASELINE"]["mean_eff_buckets"]),
            "WT2": round_or_blank(inc0173["summary"]["BASELINE"]["mean_eff_buckets"]),
        },
        {
            "metric": "hopf_mean_eff_buckets",
            "PTB": round_or_blank(ptb_hopf["mean_eff_buckets"]),
            "WT2": round_or_blank(wt2_hopf["mean_eff_buckets"]),
        },
        {
            "metric": "permuted_mean_eff_buckets",
            "PTB": round_or_blank(ptb_perm["mean_eff_buckets"]),
            "WT2": round_or_blank(inc0173["summary"]["PERMUTED"]["mean_eff_buckets"]),
        },
    ]
    write_csv(TABLES / "cross_dataset_summary.csv", rows, ["metric", "PTB", "WT2"])


def export_key_control_table() -> None:
    inc0171 = load_json(ANALYSIS / "inc0171_lm_integration.json")
    inc0172 = load_json(ANALYSIS / "inc0172_moe_substitution.json")
    inc0173 = load_json(ANALYSIS / "inc0173_wt2_confirm.json")

    rows = [
        {
            "control_case": "PTB_null_HOPF_vs_PERMUTED",
            "dataset_or_setting": "PTB",
            "baseline_mean_val_ppl": round_or_blank(inc0171["summary"]["BASELINE"]["mean_val_ppl"]),
            "hopf_mean_val_ppl": round_or_blank(inc0171["summary"]["HOPF"]["mean_val_ppl"]),
            "permuted_mean_val_ppl": round_or_blank(inc0171["summary"]["PERMUTED"]["mean_val_ppl"]),
            "hopf_vs_baseline_ratio": round_or_blank(inc0171["ppl_ratio_hopf_vs_baseline"]),
            "hopf_minus_permuted_ppl": round_or_blank(
                inc0171["summary"]["HOPF"]["mean_val_ppl"] - inc0171["summary"]["PERMUTED"]["mean_val_ppl"]
            ),
            "baseline_mean_eff_b": round_or_blank(inc0171["summary"]["BASELINE"]["mean_eff_buckets"]),
            "hopf_mean_eff_b": round_or_blank(inc0171["summary"]["HOPF"]["mean_eff_buckets"]),
            "permuted_mean_eff_b": round_or_blank(inc0171["summary"]["PERMUTED"]["mean_eff_buckets"]),
            "learned_sparse_mean_eff_b": "",
            "concentration_guard_limit": "",
            "concentration_guard_passed": "",
            "interpretation": "Hopf and permuted are effectively indistinguishable on PTB quality.",
        },
        {
            "control_case": "WT2_null_HOPF_vs_PERMUTED",
            "dataset_or_setting": "WT2",
            "baseline_mean_val_ppl": round_or_blank(inc0173["summary"]["BASELINE"]["mean_val_ppl"]),
            "hopf_mean_val_ppl": round_or_blank(inc0173["summary"]["HOPF"]["mean_val_ppl"]),
            "permuted_mean_val_ppl": round_or_blank(inc0173["summary"]["PERMUTED"]["mean_val_ppl"]),
            "hopf_vs_baseline_ratio": round_or_blank(inc0173["ppl_ratio_hopf_vs_baseline"]),
            "hopf_minus_permuted_ppl": round_or_blank(inc0173["hopf_vs_permuted_delta_ppl"]),
            "baseline_mean_eff_b": round_or_blank(inc0173["summary"]["BASELINE"]["mean_eff_buckets"]),
            "hopf_mean_eff_b": round_or_blank(inc0173["summary"]["HOPF"]["mean_eff_buckets"]),
            "permuted_mean_eff_b": round_or_blank(inc0173["summary"]["PERMUTED"]["mean_eff_buckets"]),
            "learned_sparse_mean_eff_b": "",
            "concentration_guard_limit": "",
            "concentration_guard_passed": "",
            "interpretation": "The same null result replicates on WT2.",
        },
        {
            "control_case": "Baseline_rationale_aux_loss_control",
            "dataset_or_setting": "PTB_aux_control_screen",
            "baseline_mean_val_ppl": round_or_blank(inc0172["summary"]["BASELINE"]["mean_val_ppl"]),
            "hopf_mean_val_ppl": round_or_blank(inc0172["summary"]["HOPF"]["mean_val_ppl"]),
            "permuted_mean_val_ppl": round_or_blank(inc0172["summary"]["PERMUTED"]["mean_val_ppl"]),
            "hopf_vs_baseline_ratio": round_or_blank(inc0172["key_metrics"]["ppl_ratio_hopf_vs_baseline"]),
            "hopf_minus_permuted_ppl": round_or_blank(
                inc0172["summary"]["HOPF"]["mean_val_ppl"] - inc0172["summary"]["PERMUTED"]["mean_val_ppl"]
            ),
            "baseline_mean_eff_b": round_or_blank(inc0172["summary"]["BASELINE"]["mean_eff_buckets"]),
            "hopf_mean_eff_b": round_or_blank(inc0172["summary"]["HOPF"]["mean_eff_buckets"]),
            "permuted_mean_eff_b": round_or_blank(inc0172["summary"]["PERMUTED"]["mean_eff_buckets"]),
            "learned_sparse_mean_eff_b": round_or_blank(inc0172["summary"]["LEARNED_SPARSE"]["mean_eff_buckets"]),
            "concentration_guard_limit": round_or_blank(inc0172["key_metrics"]["concentration_guard_limit"]),
            "concentration_guard_passed": inc0172["key_metrics"]["concentration_guard_passed"],
            "interpretation": "Load-balancing aux loss drives routing toward near-uniform usage, so no-aux top-1 is the correct paper comparator.",
        },
    ]
    write_csv(
        TABLES / "key_control_null_result.csv",
        rows,
        [
            "control_case",
            "dataset_or_setting",
            "baseline_mean_val_ppl",
            "hopf_mean_val_ppl",
            "permuted_mean_val_ppl",
            "hopf_vs_baseline_ratio",
            "hopf_minus_permuted_ppl",
            "baseline_mean_eff_b",
            "hopf_mean_eff_b",
            "permuted_mean_eff_b",
            "learned_sparse_mean_eff_b",
            "concentration_guard_limit",
            "concentration_guard_passed",
            "interpretation",
        ],
    )


def frange(start: float, stop: float, count: int) -> list[float]:
    if count <= 1:
        return [start]
    step = (stop - start) / (count - 1)
    return [start + i * step for i in range(count)]


def main() -> None:
    ensure_dirs()
    scaling_rows, fit_params = export_scaling_data()
    generate_scaling_figure(scaling_rows, fit_params)
    export_ptb_table()
    export_wt2_table()
    export_cross_dataset_table()
    export_key_control_table()
    print("Generated Paper 1 assets in paper1_assets/")


if __name__ == "__main__":
    main()
