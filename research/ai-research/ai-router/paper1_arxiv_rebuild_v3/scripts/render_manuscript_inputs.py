#!/usr/bin/env python3
"""Generate manuscript inputs and publication figures for the v3 arXiv rebuild."""

from __future__ import annotations

import csv
import json
import math
from collections import Counter
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
from matplotlib import gridspec
from matplotlib.patches import Circle, FancyArrowPatch, FancyBboxPatch, Rectangle


ROOT = Path(__file__).resolve().parents[2]
ANALYSIS = ROOT / "router-research" / "results" / "analysis"
ASSETS = ROOT / "paper1_assets"
DATA = ASSETS / "data"
TABLES = ASSETS / "tables"
PROXY = ROOT / "router-research" / "data" / "wikitext2_proxy" / "ppmi_proxy.npz"

DRAFT = ROOT / "paper1_arxiv_rebuild_v3"
GENERATED = DRAFT / "generated"
FIGURES = DRAFT / "figures"

PHASE4_DIMS = (3, 65, 2, 21)
PERM_SEED = 42
K_OCC = 100

BLUE = "#2f59c6"
ORANGE = "#c96a04"
TEAL = "#0f766e"
SLATE = "#475569"
LIGHT_GRID = "#d9e1ea"


def read_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def read_csv(path: Path) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content.strip() + "\n", encoding="utf-8")


def fmt(value: float | str, digits: int = 2, signed: bool = False) -> str:
    number = float(value)
    if signed:
        return f"{number:+.{digits}f}"
    return f"{number:.{digits}f}"


def latex_escape(text: str) -> str:
    replacements = {
        "\\": r"\textbackslash{}",
        "&": r"\&",
        "%": r"\%",
        "$": r"\$",
        "#": r"\#",
        "_": r"\_",
    }
    for source, target in replacements.items():
        text = text.replace(source, target)
    return text


def route_stats_by_id(doc: dict) -> dict[str, dict]:
    return {row["route_id"]: row for row in doc["route_stats"]}


def wrap_to_pi(x: np.ndarray) -> np.ndarray:
    return (x + np.pi) % (2.0 * np.pi) - np.pi


def allocate_bins(K: int, min_b: int = 2) -> tuple[int, int, int]:
    K = max(1, int(K))
    best = (1, K, 1)
    best_score = None
    for k1 in range(min_b, K + 1):
        for k2 in range(min_b, K + 1):
            max_k3 = K // max(k1 * k2, 1)
            if max_k3 < min_b:
                break
            for k3 in range(min_b, max_k3 + 1):
                prod = k1 * k2 * k3
                favor = 1 if k2 >= k3 else 0
                spread = abs(k1 - k2) + abs(k2 - k3) + abs(k1 - k3)
                score = (prod, favor, -spread, k2, -k3)
                if best_score is None or score > best_score:
                    best_score = score
                    best = (k1, k2, k3)
    return best


def hopf_coordinates(Z: np.ndarray, dims: tuple[int, int, int, int] = PHASE4_DIMS) -> dict[str, np.ndarray]:
    a = Z[:, dims[0]]
    b = Z[:, dims[1]]
    c = Z[:, dims[2]]
    d = Z[:, dims[3]]

    rho1 = np.sqrt(a * a + b * b)
    rho2 = np.sqrt(c * c + d * d)
    denom = np.maximum(np.sqrt(rho1**2 + rho2**2), 1e-12)

    eta = np.arcsin(np.clip(rho2 / denom, 0.0, 1.0))
    theta1 = np.arctan2(b, a)
    theta2 = np.arctan2(d, c)
    delta = wrap_to_pi(theta1 - theta2)
    alpha = wrap_to_pi(0.5 * (theta1 + theta2))
    alpha_t = wrap_to_pi(alpha + 0.5 * np.cos(2.0 * eta) * delta)

    x = np.sin(2.0 * eta) * np.cos(delta)
    y = np.sin(2.0 * eta) * np.sin(delta)
    z = np.cos(2.0 * eta)

    return {
        "a": a,
        "b": b,
        "c": c,
        "d": d,
        "rho1": rho1,
        "rho2": rho2,
        "eta": eta,
        "delta": delta,
        "alpha": alpha,
        "alpha_t": alpha_t,
        "hopf_x": x,
        "hopf_y": y,
        "hopf_z": z,
    }


def hopf_sector_ids(Z: np.ndarray, K: int, dims: tuple[int, int, int, int] = PHASE4_DIMS) -> tuple[np.ndarray, tuple[int, int, int]]:
    coords = hopf_coordinates(Z, dims=dims)
    k_eta, k_delta, k_alpha = allocate_bins(K)

    eta_u = np.clip((np.sin(coords["eta"])) ** 2, 0.0, 1.0 - 1e-12)
    b_eta = np.minimum((eta_u * k_eta).astype(np.int64), k_eta - 1)
    b_delta = np.minimum(((coords["delta"] + np.pi) / (2.0 * np.pi) * k_delta).astype(np.int64), k_delta - 1)
    b_alpha = np.minimum(((coords["alpha_t"] + np.pi) / (2.0 * np.pi) * k_alpha).astype(np.int64), k_alpha - 1)
    sectors = (b_eta * k_delta * k_alpha + b_delta * k_alpha + b_alpha).astype(np.int64)
    return sectors, (k_eta, k_delta, k_alpha)


def sector_mass_curve(sectors: np.ndarray, K: int) -> np.ndarray:
    counts = Counter(sectors.tolist())
    masses = np.array(sorted((v / len(sectors) for v in counts.values()), reverse=True), dtype=float)
    if len(masses) < K:
        masses = np.concatenate([masses, np.zeros(K - len(masses), dtype=float)])
    return masses


def mean_curve(results: list[dict], condition: str) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    rows = [row for row in results if row["condition"] == condition]
    steps = np.array([entry["step"] for entry in rows[0]["log"]], dtype=float)
    ppl = np.mean([[entry["val_ppl"] for entry in row["log"]] for row in rows], axis=0)
    eff = np.mean([[entry["eff_buckets"] for entry in row["log"]] for row in rows], axis=0)
    return steps, np.asarray(ppl, dtype=float), np.asarray(eff, dtype=float)


def render_metrics() -> None:
    fit = read_json(DATA / "scaling_fit_params.json")
    cross = {row["metric"]: row for row in read_csv(TABLES / "cross_dataset_summary.csv")}
    controls = read_csv(TABLES / "key_control_null_result.csv")
    aux_row = next(row for row in controls if row["control_case"] == "Baseline_rationale_aux_loss_control")
    ptb = {row["condition"]: row for row in read_csv(TABLES / "ptb_results.csv")}
    wt2 = {row["condition"]: row for row in read_csv(TABLES / "wt2_results.csv")}

    inc0170 = read_json(ANALYSIS / "inc0170_large_k.json")
    inc0143 = route_stats_by_id(read_json(ANALYSIS / "inc0143_ppmi_semantic_finalize.json"))
    inc0144 = route_stats_by_id(read_json(ANALYSIS / "inc0144_hopf_vs_kmeans_stage3_confirm.json"))
    inc0168 = read_json(ANALYSIS / "inc0168_norm_geometry.json")

    proxy = np.load(PROXY)
    x_train = proxy["x_train"]
    corr = np.corrcoef(x_train[:, [3, 65, 2, 21]].T)
    pair1 = abs(corr[0, 1])
    pair2 = abs(corr[2, 3])

    invariance_span = max(
        inc0168[exp]["scaling_exponents"]["TRANS_ORIG"]["alpha"]
        for exp in ["ExpA_L2", "ExpB1_L1_canonicalDeltaR", "ExpC1_L3", "ExpC2_L4"]
    ) - min(
        inc0168[exp]["scaling_exponents"]["TRANS_ORIG"]["alpha"]
        for exp in ["ExpA_L2", "ExpB1_L1_canonicalDeltaR", "ExpC1_L3", "ExpC2_L4"]
    )

    lines = [
        "% Auto-generated by render_manuscript_inputs.py",
        rf"\newcommand{{\PaperHopfScalingA}}{{{fit['anchor_fit_reference']['TRANS_ORIG']['c']:.3f}}}",
        rf"\newcommand{{\PaperHopfScalingAlpha}}{{{fit['anchor_fit_reference']['TRANS_ORIG']['alpha']:.3f}}}",
        rf"\newcommand{{\PaperPermScalingAlpha}}{{{fit['anchor_fit_reference']['TRANS_PERM']['alpha']:.3f}}}",
        rf"\newcommand{{\PaperHopfLargeKAlpha}}{{{fit['large_k_fit_reference']['TRANS_ORIG']['alpha']:.3f}}}",
        rf"\newcommand{{\PaperRatioAtFourHundred}}{{{inc0170['compression_by_K']['400']['eff_ratio_PERM_over_ORIG']:.2f}}}",
        rf"\newcommand{{\PaperRatioAtFiveThousand}}{{{inc0170['compression_by_K']['5000']['eff_ratio_PERM_over_ORIG']:.2f}}}",
        rf"\newcommand{{\PaperPTBHopfBaselineRatio}}{{{fmt(cross['hopf_vs_baseline_ppl_ratio']['PTB'], 3)}}}",
        rf"\newcommand{{\PaperWTTwoHopfBaselineRatio}}{{{fmt(cross['hopf_vs_baseline_ppl_ratio']['WT2'], 3)}}}",
        rf"\newcommand{{\PaperPTBHopfPermDelta}}{{{fmt(cross['hopf_vs_permuted_delta_ppl']['PTB'], 2, signed=True)}}}",
        rf"\newcommand{{\PaperWTTwoHopfPermDelta}}{{{fmt(cross['hopf_vs_permuted_delta_ppl']['WT2'], 2, signed=True)}}}",
        rf"\newcommand{{\PaperPTBHopfEff}}{{{fmt(ptb['HOPF']['mean_eff_buckets'], 1)}}}",
        rf"\newcommand{{\PaperWTTwoHopfEff}}{{{fmt(wt2['HOPF']['mean_eff_buckets'], 1)}}}",
        rf"\newcommand{{\PaperAuxSparseEff}}{{{fmt(aux_row['learned_sparse_mean_eff_b'], 1)}}}",
        rf"\newcommand{{\PaperStructuredPmax}}{{{fmt(inc0143['SEM_ORIG']['mean_sector_pmax'], 3)}}}",
        rf"\newcommand{{\PaperColPermPmax}}{{{fmt(inc0143['SEM_COL_PERM']['mean_sector_pmax'], 3)}}}",
        rf"\newcommand{{\PaperHopfControlPmax}}{{{fmt(inc0144['HOPF_ORIG']['mean_sector_pmax'], 3)}}}",
        rf"\newcommand{{\PaperKMeansPmax}}{{{fmt(inc0144['KMEANS_ORIG']['mean_sector_pmax'], 3)}}}",
        rf"\newcommand{{\PaperNormInvariantSpan}}{{{invariance_span:.3f}}}",
        rf"\newcommand{{\PaperDimPairOneCorr}}{{{pair1:.3f}}}",
        rf"\newcommand{{\PaperDimPairTwoCorr}}{{{pair2:.3f}}}",
    ]
    write(GENERATED / "metrics.tex", "\n".join(lines))


def render_tables() -> None:
    ptb = read_csv(TABLES / "ptb_results.csv")
    wt2 = read_csv(TABLES / "wt2_results.csv")
    cross = {row["metric"]: row for row in read_csv(TABLES / "cross_dataset_summary.csv")}
    controls = read_csv(TABLES / "key_control_null_result.csv")
    inc0143 = route_stats_by_id(read_json(ANALYSIS / "inc0143_ppmi_semantic_finalize.json"))
    inc0144 = route_stats_by_id(read_json(ANALYSIS / "inc0144_hopf_vs_kmeans_stage3_confirm.json"))
    inc0168 = read_json(ANALYSIS / "inc0168_norm_geometry.json")
    inc0172 = read_json(ANALYSIS / "inc0172_moe_substitution.json")

    lm_lines = [
        r"\begin{tabular}{llcccc}",
        r"\toprule",
        r"Dataset & Router & Val.\ PPL & H/B & Eff.\ paths & Params \\",
        r"\midrule",
    ]
    for dataset, rows in [("PTB", ptb), ("WT2", wt2)]:
        baseline_ppl = float(next(row["mean_val_ppl"] for row in rows if row["condition"] == "BASELINE"))
        for row in rows:
            ratio = float(row["mean_val_ppl"]) / baseline_ppl
            params_m = float(row["n_params"]) / 1_000_000.0
            lm_lines.append(
                f"{dataset} & {row['condition']} & {fmt(row['mean_val_ppl'])} & "
                f"{ratio:.3f} & {fmt(row['mean_eff_buckets'], 1)} & {params_m:.2f}M \\\\"
            )
        if dataset == "PTB":
            lm_lines.append(r"\midrule")
    lm_lines.extend([r"\bottomrule", r"\end{tabular}"])
    write(GENERATED / "table_lm_combined.tex", "\n".join(lm_lines))

    setup_lines = [
        r"\begin{tabular}{ll}",
        r"\toprule",
        r"Item & Value \\",
        r"\midrule",
        r"Static proxy & PPMI-SVD embeddings (100 dimensions), context length 32 \\",
        r"Proxy source text & PTB co-occurrence statistics applied to WT2 token windows \\",
        r"Proxy sample counts & 50{,}000 train / 50{,}000 val / 50{,}000 test \\",
        r"4D routing chart & dims $(3,65,2,21)$ from proxy correlation pre-screen \\",
        r"LM architecture & 2 layers, hidden 128, 4 heads, sequence length 32 \\",
        r"Experts & 64 experts, expert dimension 128 \\",
        r"Datasets & PTB and WikiText-2 \\",
        r"Training horizon & 4000 steps, 2 seeds per dataset \\",
        r"Main LM conditions & BASELINE, HOPF, PERMUTED \\",
        r"Primary routing metric & $e_{\mathrm{eff}} = \exp(-\sum_r p_r \log p_r)$ \\",
        r"\bottomrule",
        r"\end{tabular}",
    ]
    write(GENERATED / "table_setup_appendix.tex", "\n".join(setup_lines))

    lines = [
        r"\begin{tabular}{lcccc}",
        r"\toprule",
        r"Proxy condition & Runs & Sector $p_{\max}$ & Sector entropy & Test MSE \\",
        r"\midrule",
    ]
    for key, label in [("SEM_ORIG", "Structured"), ("SEM_COL_PERM", "Column-permuted")]:
        row = inc0143[key]
        lines.append(
            f"{label} & {row['n_runs']} & {fmt(row['mean_sector_pmax'], 3)} & "
            f"{fmt(row['mean_sector_entropy'], 3)} & {fmt(row['mean_test_mse_after'], 4)} \\\\"
        )
    lines.extend([r"\bottomrule", r"\end{tabular}"])
    write(GENERATED / "table_proxy_structured_control.tex", "\n".join(lines))

    lines = [
        r"\begin{tabular}{lcccc}",
        r"\toprule",
        r"Routing family & Runs & Sector $p_{\max}$ & Sector entropy & Test MSE \\",
        r"\midrule",
    ]
    for key, label in [("HOPF_ORIG", "Hopf"), ("KMEANS_ORIG", "$K$-means"), ("HOPF_PERM", "Hopf permuted"), ("KMEANS_PERM", "$K$-means permuted")]:
        row = inc0144[key]
        lines.append(
            f"{label} & {row['n_runs']} & {fmt(row['mean_sector_pmax'], 3)} & "
            f"{fmt(row['mean_sector_entropy'], 3)} & {fmt(row['mean_test_mse_after'], 4)} \\\\"
        )
    lines.extend([r"\bottomrule", r"\end{tabular}"])
    write(GENERATED / "table_proxy_hopf_kmeans_control.tex", "\n".join(lines))

    labels = {
        "ExpA_L2": "L2",
        "ExpB1_L1_canonicalDeltaR": "L1",
        "ExpC1_L3": "L3",
        "ExpC2_L4": "L4",
    }
    lines = [
        r"\begin{tabular}{lccc}",
        r"\toprule",
        r"Normalization & Hopf exponent & Permuted exponent & Base exponent \\",
        r"\midrule",
    ]
    for exp in ["ExpB1_L1_canonicalDeltaR", "ExpA_L2", "ExpC1_L3", "ExpC2_L4"]:
        se = inc0168[exp]["scaling_exponents"]
        lines.append(
            f"{labels[exp]} & {fmt(se['TRANS_ORIG']['alpha'], 3)} & "
            f"{fmt(se['TRANS_PERM']['alpha'], 3)} & {fmt(se['BASE_ORIG']['alpha'], 3)} \\\\"
        )
    lines.extend([r"\bottomrule", r"\end{tabular}"])
    write(GENERATED / "table_norm_invariance.tex", "\n".join(lines))

    lines = [
        r"\begin{tabular}{lccc}",
        r"\toprule",
        r"Condition & Val.\ PPL & Eff.\ paths & Params \\",
        r"\midrule",
    ]
    for key in ["BASELINE", "LEARNED_SPARSE", "HOPF", "PERMUTED"]:
        row = inc0172["summary"][key]
        params_m = float(row["n_params"]) / 1_000_000.0
        eff_text = "--" if row["mean_eff_buckets"] is None else fmt(row["mean_eff_buckets"], 1)
        lines.append(
            f"{latex_escape(key)} & {fmt(row['mean_val_ppl'])} & {eff_text} & {params_m:.2f}M \\\\"
        )
    lines.extend([r"\bottomrule", r"\end{tabular}"])
    write(GENERATED / "table_auxiliary_control.tex", "\n".join(lines))

    lines = [r"\begin{tabular}{lcc}", r"\toprule", r"Metric & PTB & WT2 \\", r"\midrule"]
    rows = [
        ("HOPF / BASELINE PPL ratio", "hopf_vs_baseline_ppl_ratio", 3, False, True),
        ("HOPF - PERMUTED PPL", "hopf_vs_permuted_delta_ppl", 2, True, False),
        ("HOPF effective-path ratio", "hopf_eff_ratio_vs_dense", 2, False, True),
        ("BASELINE eff. paths", "baseline_mean_eff_buckets", 1, False, False),
        ("HOPF eff. paths", "hopf_mean_eff_buckets", 1, False, False),
        ("PERMUTED eff. paths", "permuted_mean_eff_buckets", 1, False, False),
    ]
    for label, key, digits, signed, mult in rows:
        suffix = r"$\times$" if mult else ""
        lines.append(
            f"{label} & {fmt(cross[key]['PTB'], digits, signed)}{suffix} & "
            f"{fmt(cross[key]['WT2'], digits, signed)}{suffix} \\\\"
        )
    lines.extend([r"\bottomrule", r"\end{tabular}"])
    write(GENERATED / "table_cross_dataset_summary.tex", "\n".join(lines))

    null_rows = [row for row in controls if "null" in row["control_case"].lower()]
    lines = [r"\begin{tabular}{lcccc}", r"\toprule", r"Dataset & HOPF / BASE & HOPF - PERM & HOPF eff. paths & PERM eff. paths \\", r"\midrule"]
    for row in null_rows:
        lines.append(
            f"{row['dataset_or_setting']} & {fmt(row['hopf_vs_baseline_ratio'], 3)} & "
            f"{fmt(row['hopf_minus_permuted_ppl'], 2, True)} & {fmt(row['hopf_mean_eff_b'], 1)} & "
            f"{fmt(row['permuted_mean_eff_b'], 1)} \\\\"
        )
    lines.extend([r"\bottomrule", r"\end{tabular}"])
    write(GENERATED / "table_key_null_main.tex", "\n".join(lines))

    lines = [
        r"\begin{tabular}{p{0.20\linewidth}p{0.10\linewidth}p{0.10\linewidth}p{0.10\linewidth}p{0.34\linewidth}}",
        r"\toprule",
        r"Case & HOPF / BASE & HOPF - PERM & Aux eff. paths & Interpretation \\",
        r"\midrule",
    ]
    for row in controls:
        aux_text = "--" if not row["learned_sparse_mean_eff_b"] else fmt(row["learned_sparse_mean_eff_b"], 1)
        lines.append(
            f"{latex_escape(row['dataset_or_setting'])} & {fmt(row['hopf_vs_baseline_ratio'], 3)} & "
            f"{fmt(row['hopf_minus_permuted_ppl'], 2, True)} & {aux_text} & "
            f"{latex_escape(row['interpretation'])} \\\\"
        )
    lines.extend([r"\bottomrule", r"\end{tabular}"])
    write(GENERATED / "table_key_control_null_result.tex", "\n".join(lines))

    def detail(rows: list[dict[str, str]], include_std: bool, out_name: str) -> None:
        if include_std:
            lines = [r"\begin{tabular}{lcccc}", r"\toprule", r"Condition & Val.\ PPL & Std.\ PPL & Eff.\ paths & $64/\mathrm{eff}_b$ \\", r"\midrule"]
            for row in rows:
                lines.append(
                    f"{row['condition']} & {fmt(row['mean_val_ppl'])} & {fmt(row['std_val_ppl'])} & "
                    f"{fmt(row['mean_eff_buckets'])} & {fmt(row['eff_ratio_vs_dense'])}$\\times$ \\\\"
                )
        else:
            lines = [r"\begin{tabular}{lcccc}", r"\toprule", r"Condition & Val.\ PPL & Eff.\ paths & $64/\mathrm{eff}_b$ & Params \\", r"\midrule"]
            for row in rows:
                params_m = float(row["n_params"]) / 1_000_000.0
                lines.append(
                    f"{row['condition']} & {fmt(row['mean_val_ppl'])} & {fmt(row['mean_eff_buckets'])} & "
                    f"{fmt(row['eff_ratio_vs_dense'])}$\\times$ & {params_m:.2f}M \\\\"
                )
        lines.extend([r"\bottomrule", r"\end{tabular}"])
        write(GENERATED / out_name, "\n".join(lines))

    detail(ptb, include_std=False, out_name="table_ptb_results.tex")
    detail(wt2, include_std=True, out_name="table_wt2_results.tex")


def set_plot_style() -> None:
    plt.rcParams.update(
        {
            "font.family": "serif",
            "font.size": 9.5,
            "axes.titlesize": 10,
            "axes.labelsize": 10,
            "xtick.labelsize": 8.5,
            "ytick.labelsize": 8.5,
            "legend.fontsize": 8.5,
            "axes.spines.top": False,
            "axes.spines.right": False,
        }
    )


def render_method_figure() -> None:
    set_plot_style()
    proxy = np.load(PROXY)
    x_train = proxy["x_train"]
    corr = np.corrcoef(x_train[:, [3, 65, 2, 21]].T)
    pair1 = abs(corr[0, 1])
    pair2 = abs(corr[2, 3])
    fig = plt.figure(figsize=(7.4, 5.8))
    gs = gridspec.GridSpec(2, 2, height_ratios=[1.0, 1.08], width_ratios=[1.0, 1.0], hspace=0.34, wspace=0.24)

    # Panel A: normalized representation and selected chart dims.
    ax = fig.add_subplot(gs[0, 0])
    ax.set_title("A. Fixed 4D chart", loc="left", fontweight="bold")
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 1)
    ax.axis("off")
    sphere = Circle((0.28, 0.48), 0.23, facecolor="#f2f5fb", edgecolor=SLATE, linewidth=1.2)
    ax.add_patch(sphere)
    rng = np.random.default_rng(2)
    pts = rng.normal(size=(140, 2))
    pts /= np.linalg.norm(pts, axis=1, keepdims=True)
    pts *= rng.uniform(0.02, 0.21, size=(140, 1))
    ax.scatter(0.28 + pts[:, 0], 0.48 + pts[:, 1], s=7, color=BLUE, alpha=0.45, linewidths=0)
    ax.text(0.28, 0.09, r"normalized state $x \in \mathbb{R}^{100}$", ha="center")
    for idx, xpos in zip([3, 65, 2, 21], [0.60, 0.70, 0.80, 0.90]):
        rect = FancyBboxPatch((xpos - 0.035, 0.29), 0.07, 0.40, boxstyle="round,pad=0.01,rounding_size=0.01",
                              facecolor="#eef3ff", edgecolor=BLUE, linewidth=1.0)
        ax.add_patch(rect)
        ax.text(xpos, 0.52, f"{idx}", ha="center", va="center", fontsize=10, fontweight="bold")
    ax.text(0.75, 0.14, r"chart dims $(3,65,2,21)$", ha="center")
    ax.add_patch(FancyArrowPatch((0.49, 0.48), (0.56, 0.48), arrowstyle="->", mutation_scale=11, linewidth=1.2, color=SLATE))
    ax.text(0.73, 0.83, f"proxy pre-screen\n($|\\rho| \\approx {pair1:.3f}, {pair2:.3f}$)", ha="center", va="top", fontsize=9)

    # Panel B: paired complex planes.
    ax = fig.add_subplot(gs[0, 1])
    ax.set_title("B. Two complex coordinates", loc="left", fontweight="bold")
    ax.set_xlim(-1.1, 2.3)
    ax.set_ylim(-1.1, 1.1)
    ax.axis("off")
    for cx, label, rho, theta, color in [(0.1, r"$q_1 = a + ib$", 0.82, 0.7, BLUE), (1.55, r"$q_2 = c + id$", 0.66, -0.95, TEAL)]:
        ax.plot([cx - 0.75, cx + 0.75], [0, 0], color="#64748b", linewidth=0.8)
        ax.plot([cx, cx], [-0.75, 0.75], color="#64748b", linewidth=0.8)
        ax.add_patch(FancyArrowPatch((cx, 0), (cx + rho * math.cos(theta), rho * math.sin(theta)),
                                     arrowstyle="->", mutation_scale=12, linewidth=2.0, color=color))
        ax.text(cx, -0.96, label, ha="center")
        ax.text(cx + 0.34, 0.22 if theta > 0 else -0.22, r"$\rho,\theta$", color=color)
    ax.text(0.88, 0.88, r"$\eta = \arctan2(\rho_2,\rho_1)$" "\n" r"$\delta = \theta_1 - \theta_2$" "\n" r"$\alpha = \frac{1}{2}(\theta_1+\theta_2)$",
            ha="center", va="top", fontsize=9)

    # Panel C: Hopf split.
    ax = fig.add_subplot(gs[1, 0])
    ax.set_title("C. Hopf base and fiber phase", loc="left", fontweight="bold")
    ax.set_xlim(-1.25, 1.25)
    ax.set_ylim(-1.15, 1.25)
    ax.axis("off")
    base = Circle((0, 0), 0.75, facecolor="#f8fafc", edgecolor=SLATE, linewidth=1.1)
    ax.add_patch(base)
    ax.scatter([0.35], [0.42], color=ORANGE, s=28, zorder=4)
    ax.add_patch(FancyArrowPatch((0, 0), (0.35, 0.42), arrowstyle="->", mutation_scale=10, linewidth=1.1, color=ORANGE))
    ax.text(0.0, -0.98, r"base point uses $(\eta,\delta)$", ha="center")
    ax.text(0.47, 0.60, r"base point", color=ORANGE)
    ring = Circle((0.35, 0.42), 0.18, fill=False, edgecolor=BLUE, linewidth=1.1, linestyle="--")
    ax.add_patch(ring)
    ax.add_patch(FancyArrowPatch((0.47, 0.42), (0.35, 0.58), connectionstyle="arc3,rad=0.35",
                                 arrowstyle="->", mutation_scale=10, linewidth=1.0, color=BLUE))
    ax.text(0.72, 0.28, r"fiber phase $\alpha$", color=BLUE)
    ax.text(-1.00, 0.95, r"$\alpha$ stays on the fiber;" "\n" r"it refines routing" "\n" r"without moving the base point.", ha="left", va="top", fontsize=9)

    # Panel D: sectorization and concentration.
    ax = fig.add_subplot(gs[1, 1])
    ax.set_title("D. Sectorization", loc="left", fontweight="bold")
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 1)
    ax.axis("off")
    left, bottom, width, height = 0.08, 0.15, 0.82, 0.62
    ax.add_patch(Rectangle((left, bottom), width, height, facecolor="#f8fafc", edgecolor=SLATE, linewidth=1.0))
    cols, rows = 6, 4
    for i in range(1, cols):
        ax.plot([left + i * width / cols] * 2, [bottom, bottom + height], color="#cbd5e1", linewidth=0.7)
    for j in range(1, rows):
        ax.plot([left, left + width], [bottom + j * height / rows] * 2, color="#cbd5e1", linewidth=0.7)
    hot_cells = [(0, 0), (1, 0), (1, 1), (2, 1), (2, 2), (4, 1)]
    for cx, cy in hot_cells:
        ax.add_patch(Rectangle((left + cx * width / cols, bottom + cy * height / rows),
                               width / cols, height / rows, facecolor=BLUE, alpha=0.70, edgecolor="none"))
    ax.text(0.49, 0.83, r"balanced bins $(k_\eta,k_\delta,k_\alpha)$ with $k_\eta k_\delta k_\alpha \leq K$", ha="center", fontsize=9)
    ax.text(0.49, 0.06, r"structured states occupy only a subset of cells, so $e_{\mathrm{eff}} \ll K$", ha="center", fontsize=9)

    fig.savefig(FIGURES / "paper1_method_figure.pdf", bbox_inches="tight")
    plt.close(fig)


def render_proxy_geometry_figure() -> None:
    set_plot_style()
    proxy = np.load(PROXY)
    x_train = proxy["x_train"][:5000].astype(np.float64)
    rng = np.random.default_rng(PERM_SEED)
    perm = rng.permutation(x_train.shape[1])
    x_perm = x_train[:, perm]

    structured = hopf_coordinates(x_train)
    permuted = hopf_coordinates(x_perm)
    struct_sectors, _ = hopf_sector_ids(x_train, K_OCC)
    perm_sectors, _ = hopf_sector_ids(x_perm, K_OCC)
    struct_mass = sector_mass_curve(struct_sectors, K_OCC)
    perm_mass = sector_mass_curve(perm_sectors, K_OCC)

    fig = plt.figure(figsize=(7.2, 3.2))
    gs = gridspec.GridSpec(1, 2, width_ratios=[1.1, 1.0], wspace=0.28)

    ax = fig.add_subplot(gs[0, 0])
    hb = ax.hexbin(
        structured["hopf_x"],
        structured["hopf_y"],
        gridsize=28,
        cmap="Blues",
        mincnt=1,
        linewidths=0.0,
    )
    ax.contour(
        np.histogram2d(permuted["hopf_x"], permuted["hopf_y"], bins=24, range=[[-1, 1], [-1, 1]])[0].T,
        levels=5,
        colors=ORANGE,
        linewidths=0.8,
        origin="lower",
        extent=[-1, 1, -1, 1],
    )
    ax.set_xlim(-1.05, 1.05)
    ax.set_ylim(-1.05, 1.05)
    ax.set_aspect("equal")
    ax.set_xlabel(r"Hopf base $x$")
    ax.set_ylabel(r"Hopf base $y$")
    ax.set_title("A. Hopf-base occupancy", loc="left", fontweight="bold")
    cbar = fig.colorbar(hb, ax=ax, fraction=0.047, pad=0.02)
    cbar.set_label("structured token count")

    ax = fig.add_subplot(gs[0, 1])
    ranks = np.arange(1, K_OCC + 1)
    ax.plot(ranks, struct_mass, color=BLUE, linewidth=2.0, label="Structured")
    ax.plot(ranks, perm_mass, color=ORANGE, linewidth=2.0, label="Column-permuted")
    ax.axhline(1.0 / K_OCC, color="#7c8795", linestyle="--", linewidth=1.0, label="Uniform")
    ax.set_xlim(1, K_OCC)
    ax.set_xlabel(f"Sector rank at $K={K_OCC}$")
    ax.set_ylabel("Traffic share")
    ax.set_title("B. Rank-ordered sector mass", loc="left", fontweight="bold")
    ax.grid(color=LIGHT_GRID, linewidth=0.6)
    ax.legend(frameon=False, loc="upper right")

    fig.savefig(FIGURES / "paper1_proxy_geometry.pdf", bbox_inches="tight")
    plt.close(fig)


def render_scaling_figure() -> None:
    set_plot_style()
    rows = read_csv(DATA / "scaling_points.csv")
    fit = read_json(DATA / "scaling_fit_params.json")
    inc0170 = read_json(ANALYSIS / "inc0170_large_k.json")

    orig = sorted((int(r["K"]), float(r["effective_bucket_count"])) for r in rows if r["variant"] == "ORIG")
    perm = sorted((int(r["K"]), float(r["effective_bucket_count"])) for r in rows if r["variant"] == "PERM")

    fig = plt.figure(figsize=(7.2, 3.4))
    gs = gridspec.GridSpec(1, 2, width_ratios=[1.15, 0.95], wspace=0.28)

    ax = fig.add_subplot(gs[0, 0])
    ax.set_xscale("log")
    ax.set_yscale("log")
    ks = np.array([k for k, _ in orig], dtype=float)
    orig_y = np.array([v for _, v in orig], dtype=float)
    perm_x = np.array([k for k, _ in perm], dtype=float)
    perm_y = np.array([v for _, v in perm], dtype=float)
    ax.plot(ks, ks, linestyle="--", color="#7c8795", linewidth=1.3, label="Dense")
    ax.scatter(ks, orig_y, color=BLUE, s=22, zorder=4)
    ax.scatter(perm_x, perm_y, color=ORANGE, s=22, zorder=4)
    kline = np.array([25, 50, 75, 100, 150, 200, 400, 600, 1000, 2000, 3000, 5000], dtype=float)
    hopf_fit = fit["anchor_fit_reference"]["TRANS_ORIG"]["c"] * (kline ** fit["anchor_fit_reference"]["TRANS_ORIG"]["alpha"])
    perm_fit = fit["large_k_fit_reference"]["TRANS_PERM"]["c"] * (kline ** fit["large_k_fit_reference"]["TRANS_PERM"]["alpha"])
    ax.plot(kline, hopf_fit, color=BLUE, linewidth=1.8, label=r"Fixed angular routing")
    ax.plot(kline, perm_fit, color=ORANGE, linewidth=1.8, label=r"Permuted fixed control")
    ax.set_xlabel(r"Routing budget $K$")
    ax.set_ylabel(r"Effective routing footprint $e_{\mathrm{eff}}$")
    ax.set_title("A. Effective footprint vs budget", loc="left", fontweight="bold")
    ax.grid(which="both", color=LIGHT_GRID, linewidth=0.6)
    ax.legend(frameon=False, loc="upper left")
    ax.annotate(r"$e_{\mathrm{eff}} \propto K^{0.572}$", xy=(130, 52), xytext=(92, 28), color=BLUE,
                arrowprops=dict(arrowstyle="-", color=BLUE, lw=1.0))
    ax.annotate(r"$e_{\mathrm{eff}} \propto K^{0.816}$", xy=(650, 365), xytext=(460, 760), color=ORANGE,
                arrowprops=dict(arrowstyle="-", color=ORANGE, lw=1.0))

    ax = fig.add_subplot(gs[0, 1])
    k_vals = sorted(int(k) for k in inc0170["compression_by_K"].keys())
    ratio = [inc0170["compression_by_K"][str(k)]["eff_ratio_PERM_over_ORIG"] for k in k_vals]
    orig_lookup = {int(row["K"]): float(row["effective_bucket_count"]) for row in inc0170["raw_orig"]}
    perm_lookup = {int(row["K"]): float(row["effective_bucket_count"]) for row in inc0170["raw_perm"]}
    active_frac_orig = [orig_lookup[k] / k for k in k_vals]
    active_frac_perm = [perm_lookup[k] / k for k in k_vals]
    ax.plot(k_vals, ratio, color=SLATE, linewidth=1.8, marker="o", markersize=4, label="Permuted / structured")
    ax2 = ax.twinx()
    ax2.plot(k_vals, active_frac_orig, color=BLUE, linewidth=1.4, linestyle="-", label=r"Structured $e_{\mathrm{eff}}/K$")
    ax2.plot(k_vals, active_frac_perm, color=ORANGE, linewidth=1.4, linestyle="-", label=r"Permuted $e_{\mathrm{eff}}/K$")
    ax.set_xscale("log")
    ax.set_xlabel(r"Routing budget $K$")
    ax.set_ylabel("Gap ratio", color=SLATE)
    ax2.set_ylabel(r"Used fraction $e_{\mathrm{eff}}/K$")
    ax.set_title("B. Growing occupancy gap", loc="left", fontweight="bold")
    ax.grid(color=LIGHT_GRID, linewidth=0.6)
    ax.tick_params(axis="y", labelcolor=SLATE)
    lines1, labels1 = ax.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax.legend(lines1 + lines2, labels1 + labels2, frameon=False, loc="upper left")

    fig.savefig(FIGURES / "paper1_scaling_v2.pdf", bbox_inches="tight")
    plt.close(fig)


def render_lm_dynamics_figure() -> None:
    set_plot_style()
    inc0171 = read_json(ANALYSIS / "inc0171_lm_integration.json")
    fig = plt.figure(figsize=(7.2, 3.25))
    gs = gridspec.GridSpec(1, 2, width_ratios=[1.0, 1.0], wspace=0.28)

    ax = fig.add_subplot(gs[0, 0])
    for condition, color in [("BASELINE", SLATE), ("HOPF", BLUE), ("PERMUTED", ORANGE)]:
        steps, ppl, _ = mean_curve(inc0171["results"], condition)
        ax.plot(steps, ppl, color=color, linewidth=2.0, label=condition.title())
    ax.set_xlabel("Training step")
    ax.set_ylabel("Validation perplexity")
    ax.set_title("A. PTB validation perplexity", loc="left", fontweight="bold")
    ax.grid(color=LIGHT_GRID, linewidth=0.6)
    ax.legend(frameon=False, loc="upper right")

    ax = fig.add_subplot(gs[0, 1])
    for condition, color in [("BASELINE", SLATE), ("HOPF", BLUE), ("PERMUTED", ORANGE)]:
        steps, _, eff = mean_curve(inc0171["results"], condition)
        ax.plot(steps, eff, color=color, linewidth=2.0, label=condition.title())
    ax.axhline(64, color="#94a3b8", linestyle="--", linewidth=1.0)
    ax.set_xlabel("Training step")
    ax.set_ylabel(r"Effective expert paths $e_{\mathrm{eff}}$")
    ax.set_title("B. PTB effective expert paths", loc="left", fontweight="bold")
    ax.grid(color=LIGHT_GRID, linewidth=0.6)
    ax.text(2130, 60.4, "dense = 64", color="#64748b")

    fig.savefig(FIGURES / "paper1_lm_dynamics.pdf", bbox_inches="tight")
    plt.close(fig)


def main() -> None:
    GENERATED.mkdir(parents=True, exist_ok=True)
    FIGURES.mkdir(parents=True, exist_ok=True)
    render_metrics()
    render_tables()
    render_method_figure()
    render_proxy_geometry_figure()
    render_scaling_figure()
    render_lm_dynamics_figure()
    print(f"Rendered manuscript inputs and figures in {DRAFT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
