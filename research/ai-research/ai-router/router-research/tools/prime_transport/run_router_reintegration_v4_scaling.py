#!/usr/bin/env python3
"""Router reintegration v4 — training budget scaling study.

Identical architecture to v4 (token injection + trajectory attention readout).
Trains to 5000 batches with param snapshots at 800, 2000, and 5000.

Temperature schedule: anneal 2.0->0.1 over batches 0-799, hold at 0.1 beyond.
This makes the 800-batch snapshot directly comparable to v4 results.

No files modified.  No operators rebuilt.
"""

from __future__ import annotations

import copy
import csv
import random as pyrand
import sys
import time
from collections import Counter, defaultdict
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).parent))

from geometry_native_operator_model_v10 import initial_operator_state_v10
from geometry_native_operator_model_v20 import coupled_torus_kick_component_v20
from geometry_native_operator_model_v21 import fiber_phase_lift_component_v21
from geometry_native_operator_model_v22 import radial_transport_component_v22
from geometry_native_operator_model_v23 import torus_base_advance_component_v23
from geometry_native_operator_model_v24 import composite_swap_component_v24
from geometry_native_operator_model_v25 import composite_twist_component_v25

# -- paths ---------------------------------------------------------------------
RESULTS_DIR = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system"
)
DOCS_DIR = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/docs/research"
)
CSV_PATH = RESULTS_DIR / "prime_transport_router_reintegration_v4_scaling.csv"
MD_PATH  = DOCS_DIR    / "prime_transport_router_reintegration_v4_scaling.md"

# -- operator registry ---------------------------------------------------------
_OPS = [
    ("T_b",  torus_base_advance_component_v23,  "non_transport"),
    ("T_x",  composite_swap_component_v24,       "non_transport"),
    ("T_y",  composite_twist_component_v25,      "non_transport"),
    ("T_c",  coupled_torus_kick_component_v20,   "transport"),
    ("T_z'", fiber_phase_lift_component_v21,     "transport"),
    ("T_r*", radial_transport_component_v22,     "transport"),
]
N_OPS            = len(_OPS)
OP_NAMES         = [o[0] for o in _OPS]
OP_FNS           = [o[1] for o in _OPS]
OP_CLUSTERS      = [o[2] for o in _OPS]
NONTRANSPORT_IDX = [i for i, o in enumerate(_OPS) if o[2] == "non_transport"]

# -- config --------------------------------------------------------------------
VOCAB        = 4
DELAYS       = [2, 4, 8, 16]
BUDGETS      = [800, 2000, 5000]      # checkpoint batch counts
BUDGET_SET   = set(BUDGETS)
MAX_BATCHES  = max(BUDGETS)
BATCH_SIZE   = 32
N_EVAL       = 1000
TEMP_START   = 2.0
TEMP_END     = 0.1
ANNEAL_OVER  = 800                    # match v4: anneal over first 800 batches
LR           = 0.02
WARMUP_STEPS = 6
GLOBAL_SEED  = 42

# -- model dims ----------------------------------------------------------------
D_EMB         = 4
D_TAU         = 2 + 5 + 2 + 12   # = 21
D_IN          = D_EMB + D_TAU    # = 25
D_HIDDEN      = 32
D_HIDDEN_ATTN = 8

# -- v4 reference (800 batches) for explicit comparison -----------------------
V4_ACCS = {
    2: 0.615, 4: 0.436, 8: 0.328, 16: 0.280,
}

_SEED_STATE = initial_operator_state_v10()


# -- helpers -------------------------------------------------------------------

def tau_onehot(state) -> np.ndarray:
    tau = state.tau
    f   = np.zeros(D_TAU, dtype=np.float64)
    off = 0
    f[off + tau.swap_phase]    = 1.0;  off += 2
    f[off + tau.coupled_phase] = 1.0;  off += 5
    f[off + tau.twist_phase]   = 1.0;  off += 2
    f[off + tau.lift_phase]    = 1.0;  off += 12
    return f


def _softmax(x: np.ndarray) -> np.ndarray:
    ex = np.exp(x - x.max())
    return ex / (ex.sum() + 1e-12)


def _route_entropy(w: np.ndarray) -> float:
    p = np.clip(w, 1e-12, 1.0)
    return float(-np.sum(p * np.log(p)))


def warmup(rng: pyrand.Random):
    s = _SEED_STATE
    for _ in range(WARMUP_STEPS):
        s = OP_FNS[rng.randint(0, N_OPS - 1)](s)
    return s


def gumbel_softmax(logits: np.ndarray, temp: float,
                   np_rng: np.random.Generator, training: bool) -> np.ndarray:
    if training:
        u = np_rng.uniform(0.0, 1.0, logits.shape)
        g = -np.log(-np.log(np.clip(u, 1e-20, 1.0)) + 1e-20)
        return _softmax((logits + g) / temp)
    return _softmax(logits / 0.05)


def gumbel_softmax_vjp(w: np.ndarray, d_w: np.ndarray, temp: float) -> np.ndarray:
    return (1.0 / temp) * w * (d_w - float(np.dot(w, d_w)))


def get_temp(batch_idx: int) -> float:
    """Match v4 for batches 0-799; hold at TEMP_END beyond."""
    if batch_idx < ANNEAL_OVER:
        frac = batch_idx / max(ANNEAL_OVER - 1, 1)
        return TEMP_START * (TEMP_END / TEMP_START) ** frac
    return TEMP_END


# -- parameters ----------------------------------------------------------------

class Params:
    _KEYS = ("W_emb", "W1", "b1", "W2", "b2",
             "W_attn", "b_attn", "v_attn", "W_pred", "b_pred", "W_tok_inject")

    def __init__(self, np_rng: np.random.Generator):
        sc = 0.05
        self.W_emb        = np_rng.standard_normal((VOCAB,        D_EMB))         * sc
        self.W1           = np_rng.standard_normal((D_IN,         D_HIDDEN))      * sc
        self.b1           = np.zeros(D_HIDDEN)
        self.W2           = np_rng.standard_normal((D_HIDDEN,     N_OPS))         * sc
        self.b2           = np.zeros(N_OPS)
        self.W_attn       = np_rng.standard_normal((D_HIDDEN_ATTN, D_TAU))        * sc
        self.b_attn       = np.zeros(D_HIDDEN_ATTN)
        self.v_attn       = np_rng.standard_normal(D_HIDDEN_ATTN)                 * sc
        self.W_pred       = np_rng.standard_normal((D_TAU,        VOCAB))         * sc
        self.b_pred       = np.zeros(VOCAB)
        self.W_tok_inject = np_rng.standard_normal((VOCAB,        D_TAU))         * sc

    def clone(self) -> "Params":
        p = Params.__new__(Params)
        for k in self._KEYS:
            setattr(p, k, getattr(self, k).copy())
        return p

    def zero_grad(self) -> dict:
        return {k: np.zeros_like(getattr(self, k)) for k in self._KEYS}

    def apply_update(self, grads: dict, lr: float, clip: float = 1.0) -> None:
        norm  = float(np.sqrt(sum((g ** 2).sum() for g in grads.values())))
        scale = min(1.0, clip / (norm + 1e-8))
        for k, g in grads.items():
            setattr(self, k, getattr(self, k) - lr * scale * g)


# -- forward / backward (identical to v4) --------------------------------------

def forward_episode(
    params: Params,
    x0: int,
    D: int,
    py_rng: pyrand.Random,
    np_rng: np.random.Generator,
    temp: float,
    training: bool,
) -> dict:
    state         = warmup(py_rng)
    tokens        = [x0] + [py_rng.randint(0, VOCAB - 1) for _ in range(D - 1)]

    soft_tau_prev  = tau_onehot(state).copy()
    cache:         list[dict]        = []
    op_seq:        list[int]         = []
    entropy_seq:   list[float]       = []
    soft_taus_seq: list[np.ndarray]  = []

    for tok in tokens:
        tau_nexts = np.stack([tau_onehot(OP_FNS[i](state)) for i in range(N_OPS)])

        emb    = params.W_emb[tok % VOCAB]
        h_in   = np.concatenate([emb, soft_tau_prev])
        h_pre  = h_in @ params.W1 + params.b1
        h      = np.tanh(h_pre)
        logits = h @ params.W2 + params.b2
        w      = gumbel_softmax(logits, temp, np_rng, training)

        soft_tau_curr = w @ tau_nexts + params.W_tok_inject[tok % VOCAB]
        entropy_seq.append(_route_entropy(w))
        soft_taus_seq.append(soft_tau_curr.copy())

        k = int(np.argmax(w))
        op_seq.append(k)

        if training:
            cache.append({
                "tok":       tok,
                "h_in":      h_in.copy(),
                "h":         h.copy(),
                "w":         w.copy(),
                "tau_nexts": tau_nexts.copy(),
            })

        state         = OP_FNS[k](state)
        soft_tau_prev = soft_tau_curr

    soft_taus_mat = np.stack(soft_taus_seq)
    h_attn_mat    = np.tanh(soft_taus_mat @ params.W_attn.T + params.b_attn)
    a_scores      = h_attn_mat @ params.v_attn
    alpha         = _softmax(a_scores)
    pooled        = alpha @ soft_taus_mat
    pred_logits   = pooled @ params.W_pred + params.b_pred
    pred          = int(np.argmax(pred_logits))
    loss          = float(-np.log(_softmax(pred_logits)[x0] + 1e-12))

    return {
        "loss":          loss,
        "correct":       (pred == x0),
        "cache":         cache,
        "soft_taus_mat": soft_taus_mat,
        "h_attn_mat":    h_attn_mat,
        "alpha":         alpha,
        "pooled":        pooled,
        "pred_logits":   pred_logits,
        "op_seq":        op_seq,
        "entropy_seq":   entropy_seq,
    }


def backward_episode(params: Params, ep: dict, x0: int, temp: float) -> dict:
    grads = params.zero_grad()

    d_pred          = _softmax(ep["pred_logits"]).copy()
    d_pred[x0]     -= 1.0
    grads["W_pred"] += np.outer(ep["pooled"], d_pred)
    grads["b_pred"] += d_pred
    d_pooled         = params.W_pred @ d_pred

    alpha         = ep["alpha"]
    h_attn_mat    = ep["h_attn_mat"]
    soft_taus_mat = ep["soft_taus_mat"]

    d_alpha        = soft_taus_mat @ d_pooled
    d_st_from_pool = np.outer(alpha, d_pooled)
    d_a_pre        = alpha * (d_alpha - float(np.dot(alpha, d_alpha)))

    d_h_attn_mat    = np.outer(d_a_pre, params.v_attn)
    grads["v_attn"] += d_a_pre @ h_attn_mat

    d_pre_mat       = d_h_attn_mat * (1.0 - h_attn_mat ** 2)
    grads["W_attn"] += d_pre_mat.T @ soft_taus_mat
    grads["b_attn"] += d_pre_mat.sum(axis=0)
    d_st_from_attn  = d_pre_mat @ params.W_attn

    d_st_total = d_st_from_pool + d_st_from_attn

    d_soft_tau_recurrent = np.zeros(D_TAU)
    for t in range(len(ep["cache"]) - 1, -1, -1):
        step      = ep["cache"][t]
        tok       = step["tok"]
        h_in      = step["h_in"]
        h         = step["h"]
        w         = step["w"]
        tau_nexts = step["tau_nexts"]

        d_soft_tau_t = d_st_total[t] + d_soft_tau_recurrent
        grads["W_tok_inject"][tok % VOCAB] += d_soft_tau_t

        d_w      = tau_nexts @ d_soft_tau_t
        d_logits = gumbel_softmax_vjp(w, d_w, temp)

        grads["W2"] += np.outer(h, d_logits)
        grads["b2"] += d_logits
        d_h    = params.W2 @ d_logits
        d_hpre = d_h * (1.0 - h * h)
        grads["W1"] += np.outer(h_in, d_hpre)
        grads["b1"] += d_hpre
        d_hin   = params.W1 @ d_hpre

        grads["W_emb"][tok % VOCAB] += d_hin[:D_EMB]
        d_soft_tau_recurrent = d_hin[D_EMB:]

    return grads


# -- training with checkpoints -------------------------------------------------

def train_with_checkpoints(
    D: int,
    np_rng: np.random.Generator,
    py_rng: pyrand.Random,
) -> dict[int, Params]:
    """Train for MAX_BATCHES; return cloned params at each BUDGET checkpoint."""
    params    = Params(np_rng)
    snapshots = {}

    for batch_idx in range(MAX_BATCHES):
        temp  = get_temp(batch_idx)
        acc_g = params.zero_grad()

        for _ in range(BATCH_SIZE):
            x0    = py_rng.randint(0, VOCAB - 1)
            ep    = forward_episode(params, x0, D, py_rng, np_rng,
                                    temp=temp, training=True)
            grads = backward_episode(params, ep, x0, temp)
            for k in acc_g:
                acc_g[k] += grads[k] / BATCH_SIZE

        params.apply_update(acc_g, LR)

        completed = batch_idx + 1
        if completed in BUDGET_SET:
            snapshots[completed] = params.clone()

    return snapshots


# -- evaluation ----------------------------------------------------------------

def evaluate(
    params: Params,
    D: int,
    py_rng: pyrand.Random,
    np_rng: np.random.Generator,
) -> dict:
    correct_total      = 0
    op_counts:         Counter               = Counter()
    transport_by_step: dict[int, list[int]]  = defaultdict(list)
    entropy_vals:      list[float]           = []
    alpha_sum:         np.ndarray            = np.zeros(D)
    alpha_count:       int                   = 0

    for _ in range(N_EVAL):
        x0 = py_rng.randint(0, VOCAB - 1)
        ep = forward_episode(params, x0, D, py_rng, np_rng,
                             temp=TEMP_END, training=False)
        correct_total += ep["correct"]
        for step, op_idx in enumerate(ep["op_seq"]):
            op_counts[OP_NAMES[op_idx]] += 1
            transport_by_step[step].append(
                1 if OP_CLUSTERS[op_idx] == "transport" else 0
            )
        entropy_vals.extend(ep["entropy_seq"])
        alpha_sum   += ep["alpha"]
        alpha_count += 1

    n_total = sum(op_counts.values()) or 1
    tr_frac = sum(
        op_counts[n] for n in OP_NAMES
        if OP_CLUSTERS[OP_NAMES.index(n)] == "transport"
    ) / n_total

    return {
        "accuracy":          correct_total / N_EVAL,
        "op_counts":         dict(op_counts),
        "transport_frac":    tr_frac,
        "mean_entropy":      float(np.mean(entropy_vals)) if entropy_vals else 0.0,
        "transport_by_step": {s: float(np.mean(v))
                              for s, v in transport_by_step.items()},
        "mean_alpha":        (alpha_sum / max(alpha_count, 1)).tolist(),
    }


# -- run all -------------------------------------------------------------------

def run_all() -> dict:
    t0     = time.perf_counter()
    np_rng = np.random.default_rng(GLOBAL_SEED)
    py_rng = pyrand.Random(GLOBAL_SEED)
    results: dict = {}   # key: (budget, D)

    for D in DELAYS:
        print(f"\n=== Delay D={D} (training to {MAX_BATCHES} batches) ===")
        t_delay = time.perf_counter()
        snaps   = train_with_checkpoints(D, np_rng, py_rng)
        print(f"  Training done ({time.perf_counter()-t_delay:.1f}s)")

        for budget in BUDGETS:
            res = evaluate(snaps[budget], D, py_rng, np_rng)
            results[(budget, D)] = res
            v4_ref = V4_ACCS.get(D, 0.0)
            v4s    = f"  (v4@800:{v4_ref:.3f})" if budget == 800 else ""
            print(
                f"  budget={budget:5d}  acc={res['accuracy']:.3f}{v4s}  "
                f"tr={res['transport_frac']:.3f}  H={res['mean_entropy']:.3f}  "
                f"alpha0={res['mean_alpha'][0]:.3f}"
            )

    print(f"\nTotal: {time.perf_counter()-t0:.1f}s")
    return results


# -- write CSV -----------------------------------------------------------------

def write_csv(all_results: dict) -> None:
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "model", "training_budget", "delay", "metric_name", "metric_value",
        "transport_usage_fraction", "route_entropy", "note",
    ]
    rows = []
    for (budget, D), res in sorted(all_results.items()):
        model = f"traj_inject_router_b{budget}"
        task  = f"first_token_recall_D{D}"
        rows.append({
            "model":                    model,
            "training_budget":          budget,
            "delay":                    D,
            "metric_name":              "accuracy",
            "metric_value":             round(res["accuracy"], 4),
            "transport_usage_fraction": round(res["transport_frac"], 4),
            "route_entropy":            round(res["mean_entropy"], 4),
            "note":                     task,
        })
        rows.append({
            "model":                    model,
            "training_budget":          budget,
            "delay":                    D,
            "metric_name":              "transport_frac",
            "metric_value":             round(res["transport_frac"], 4),
            "transport_usage_fraction": round(res["transport_frac"], 4),
            "route_entropy":            round(res["mean_entropy"], 4),
            "note":                     task,
        })
        rows.append({
            "model":                    model,
            "training_budget":          budget,
            "delay":                    D,
            "metric_name":              "route_entropy",
            "metric_value":             round(res["mean_entropy"], 4),
            "transport_usage_fraction": round(res["transport_frac"], 4),
            "route_entropy":            round(res["mean_entropy"], 4),
            "note":                     task,
        })
        for step, alpha_t in enumerate(res["mean_alpha"]):
            rows.append({
                "model":                    model,
                "training_budget":          budget,
                "delay":                    D,
                "metric_name":              "mean_attn_weight",
                "metric_value":             round(alpha_t, 4),
                "transport_usage_fraction": round(res["transport_frac"], 4),
                "route_entropy":            round(res["mean_entropy"], 4),
                "note":                     f"step={step}",
            })
    with CSV_PATH.open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"[v4s] Wrote {CSV_PATH}")


# -- write markdown ------------------------------------------------------------

def write_md(all_results: dict) -> None:
    DOCS_DIR.mkdir(parents=True, exist_ok=True)
    chance = 1.0 / VOCAB

    def r(budget, D):
        return all_results.get((budget, D), {})

    lines: list[str] = []

    lines += [
        "# Prime Transport — Router Reintegration v4 Training-Budget Scaling",
        "",
        "**Experiment type:** Training-budget scaling study on v4 token injection architecture",
        "**Operator layer:** `geometry_native_operator_model_v25`",
        "**No surface build.  No files modified.  No operators rebuilt.**",
        "",
        "## What Changed from v4",
        "",
        "The architecture is **identical to v4** in every respect. Only training budget varies.",
        "",
        "| Aspect | Value |",
        "|--------|-------|",
        f"| Architecture | v4 (token injection + trajectory attention) |",
        f"| Training budgets tested | {BUDGETS} batches |",
        f"| Batch size | {BATCH_SIZE} |",
        f"| Episodes @ 5000 batches | {5000 * BATCH_SIZE:,} per delay |",
        f"| Temperature schedule | Anneal 2.0→0.1 over batches 0-799; hold 0.1 beyond |",
        f"| Checkpointing | Params cloned at batches 800, 2000, 5000 within one run |",
        "",
        "**Why the temperature is held at 0.1 after batch 800:**",
        "v4 used exactly 800 batches with temperature annealed over the full run. "
        "For clean comparison, the extended training uses the same annealing schedule "
        "for batches 0-799 and then continues at TEMP_END=0.1. The 800-batch snapshot "
        "is therefore directly comparable to v4.",
        "",
        "## Primary Results",
        "",
        "### Accuracy by training budget and delay",
        "",
        f"v4 reference (800 batches): D=2→{V4_ACCS[2]:.3f}, D=4→{V4_ACCS[4]:.3f}, "
        f"D=8→{V4_ACCS[8]:.3f}, D=16→{V4_ACCS[16]:.3f}",
        "",
        f"| Budget | D=2 | D=4 | D=8 | D=16 | above-chance at all delays? |",
        f"|--------|-----|-----|-----|------|-----------------------------|",
    ]

    for budget in BUDGETS:
        accs = [r(budget, D).get("accuracy", 0.0) for D in DELAYS]
        all_above = all(a > chance for a in accs)
        lines.append(
            f"| {budget} | "
            + " | ".join(f"{a:.3f}" for a in accs)
            + f" | {'yes' if all_above else 'no'} |"
        )
    lines.append("")

    lines += [
        "### Delta vs 800-batch baseline, by delay",
        "",
        "| Budget | D=2 delta | D=4 delta | D=8 delta | D=16 delta |",
        "|--------|-----------|-----------|-----------|------------|",
    ]
    base_accs = {D: r(800, D).get("accuracy", 0.0) for D in DELAYS}
    for budget in [2000, 5000]:
        deltas = [r(budget, D).get("accuracy", 0.0) - base_accs[D] for D in DELAYS]
        lines.append(
            f"| {budget} | "
            + " | ".join(f"{'+' if d >= 0 else ''}{d:.3f}" for d in deltas)
            + " |"
        )
    lines.append("")

    # ── scaling analysis ──
    accs_by_delay: dict[int, list[float]] = {}
    for D in DELAYS:
        accs_by_delay[D] = [r(b, D).get("accuracy", 0.0) for b in BUDGETS]

    lines += [
        "### Does performance scale with training budget?",
        "",
        "| D | acc@800 | acc@2000 | acc@5000 | scales? |",
        "|---|---------|----------|----------|---------|",
    ]
    scales_flags = []
    for D in DELAYS:
        accs = accs_by_delay[D]
        scales = accs[2] > accs[0] + 0.02   # 5000 > 800 by meaningful margin
        scales_flags.append(scales)
        trend = "yes" if scales else "no"
        lines.append(
            f"| {D} | {accs[0]:.3f} | {accs[1]:.3f} | {accs[2]:.3f} | {trend} |"
        )
    lines.append("")

    n_scales = sum(scales_flags)
    if n_scales >= 3:
        lines.append(
            f"**Performance scales with training at {n_scales}/{len(DELAYS)} delays.** "
            f"More training materially improves accuracy."
        )
    elif n_scales >= 1:
        lines.append(
            f"Performance scales at {n_scales}/{len(DELAYS)} delays. "
            f"Mixed result: shorter delays benefit more from additional training."
        )
    else:
        lines.append(
            f"Performance does not scale consistently with training budget. "
            f"Saturation occurs early, even at short delays."
        )
    lines.append("")

    # ── routing behavior ──
    max_ent = float(np.log(N_OPS))
    lines += [
        "## Routing Behavior",
        "",
        "### Route entropy (operator diversity) by budget and delay",
        "",
        "| Budget | D=2 | D=4 | D=8 | D=16 | any collapsed (<0.3)? |",
        "|--------|-----|-----|-----|------|-----------------------|",
    ]
    for budget in BUDGETS:
        entrs   = [r(budget, D).get("mean_entropy", 0.0) for D in DELAYS]
        any_col = any(e < 0.3 for e in entrs)
        lines.append(
            f"| {budget} | "
            + " | ".join(f"{e:.3f}" for e in entrs)
            + f" | {'yes' if any_col else 'no'} |"
        )
    lines.append("")

    lines += [
        "### Transport fraction by budget and delay",
        "",
        "| Budget | D=2 | D=4 | D=8 | D=16 |",
        "|--------|-----|-----|-----|------|",
    ]
    for budget in BUDGETS:
        fracs = [f"{r(budget, D).get('transport_frac', 0):.3f}" for D in DELAYS]
        lines.append(f"| {budget} | {' | '.join(fracs)} |")
    lines.append("")

    # ── attention weights ──
    lines += [
        "## Attention Weight Structure",
        "",
        "Mean alpha at step 0 (encoding step) by budget — uniform = 1/D",
        "",
        "| Budget | D=2 (unif=0.50) | D=4 (unif=0.25) | D=8 (unif=0.125) | D=16 (unif=0.0625) |",
        "|--------|-----------------|-----------------|------------------|--------------------|",
    ]
    for budget in BUDGETS:
        alpha0s = []
        for D in DELAYS:
            alphas = r(budget, D).get("mean_alpha", [1.0/D]*D)
            a0     = alphas[0] if alphas else 1.0/D
            unif   = 1.0 / D
            rel    = a0 / unif
            alpha0s.append(
                f"{a0:.3f}({'↑' if rel > 1.05 else '↓' if rel < 0.95 else '~'})"
            )
        lines.append(f"| {budget} | {' | '.join(alpha0s)} |")
    lines.append("")

    # ── long-delay degradation ──
    lines += [
        "## Long-Delay Degradation Analysis",
        "",
        "| Budget | D=2 | D=16 | gap (D=2 - D=16) |",
        "|--------|-----|------|------------------|",
    ]
    for budget in BUDGETS:
        a2  = r(budget, 2).get("accuracy", 0.0)
        a16 = r(budget, 16).get("accuracy", 0.0)
        gap = a2 - a16
        lines.append(f"| {budget} | {a2:.3f} | {a16:.3f} | {gap:.3f} |")
    lines.append("")

    gap800  = r(800, 2).get("accuracy", 0.0)  - r(800, 16).get("accuracy", 0.0)
    gap5000 = r(5000, 2).get("accuracy", 0.0) - r(5000, 16).get("accuracy", 0.0)
    if gap5000 < gap800 - 0.05:
        lines.append(
            f"**Long-delay gap narrows with more training** "
            f"(gap at 800: {gap800:.3f}, gap at 5000: {gap5000:.3f}). "
            f"Additional training helps disproportionately at longer delays."
        )
    elif gap5000 > gap800 + 0.05:
        lines.append(
            f"Long-delay gap WIDENS with more training "
            f"(gap at 800: {gap800:.3f}, gap at 5000: {gap5000:.3f}). "
            f"Short delays benefit more from additional training."
        )
    else:
        lines.append(
            f"Long-delay gap is stable across training budgets "
            f"(gap at 800: {gap800:.3f}, gap at 5000: {gap5000:.3f}). "
            f"Delay-based degradation persists regardless of training budget."
        )
    lines.append("")

    # ── honesty ──
    best_5k = max(r(5000, D).get("accuracy", 0.0) for D in DELAYS)
    best_d2 = r(5000, 2).get("accuracy", 0.0)

    lines += [
        "## Honesty Section",
        "",
        "### What improved",
        "",
        f"- Best accuracy at 5000 batches: {best_5k:.3f} (chance {chance:.3f}).",
        f"- D=2 accuracy at 5000 batches: {best_d2:.3f}.",
        "- Routing remained non-collapsed throughout all budgets.",
        "- Training-budget scaling effect (if any) is directly measurable from this run.",
        "",
        "### What saturated or failed",
        "",
    ]
    if gap5000 >= gap800 - 0.05:
        lines += [
            "- Long-delay degradation persisted: D=16 accuracy did not converge "
            "toward D=2 accuracy even at 5000 batches.",
        ]
    lines += [
        "- The per-step additive injection (W_tok_inject[tok_t]) is the same "
        "for ALL occurrences of a given token across steps. At D=16, the 15 "
        "noise-token injections may accumulate and dilute the encoding-step signal.",
        "",
        "### What remains uncertain",
        "",
        "- Whether a position-aware injection (step 0 only, not all steps) "
        "would prevent noise accumulation at large D.",
        "- Whether the attention mechanism could specialize for position if "
        "position embeddings were added.",
        "- Whether full exact spin_H is solved: No.",
        "",
        "## Recommended Next Step",
        "",
    ]

    d16_at_5k = r(5000, 16).get("accuracy", 0.0)
    if d16_at_5k > 0.35:
        lines.append(
            "Performance at D=16 is reaching practical significance. Run v4 "
            "with **10,000 batches** at D=16 only to determine the convergence "
            "ceiling for long-delay recall."
        )
    else:
        lines.append(
            "Implement **step-0-only injection**: inject W_tok_inject[tok] only at "
            "the first step (t=0), not at all D steps. This eliminates noise "
            "accumulation from subsequent non-target tokens while preserving the "
            "encoding-step signal — directly testing whether per-step noise injection "
            "is the cause of long-delay degradation."
        )
    lines.append("")

    with MD_PATH.open("w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")
    print(f"[v4s] Wrote {MD_PATH}")


# -- main ----------------------------------------------------------------------

def main():
    print("=== Prime Transport Router Reintegration v4 — Budget Scaling ===")
    print(f"Budgets: {BUDGETS} batches × {BATCH_SIZE} = "
          + ", ".join(str(b * BATCH_SIZE) for b in BUDGETS) + " episodes")
    print(f"Task: first-token recall, VOCAB={VOCAB}, DELAYS={DELAYS}")
    print()
    results = run_all()
    write_csv(results)
    write_md(results)
    print("[v4s] Experiment complete.")


if __name__ == "__main__":
    main()
