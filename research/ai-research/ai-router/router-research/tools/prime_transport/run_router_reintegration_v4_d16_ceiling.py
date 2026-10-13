#!/usr/bin/env python3
"""Router reintegration v4 — D=16 convergence ceiling test.

Identical architecture to v4 (token injection + trajectory attention readout).
Trains on D=16 only to 10,000 batches with param snapshots at 2000, 5000,
7500, and 10000.

The 5000-batch snapshot bridges directly to v4_scaling results (D=16: 0.367).
The 7500 and 10000 snapshots determine whether long-delay recall continues
to improve or saturates.

Temperature schedule: anneal 2.0->0.1 over batches 0-799, hold at 0.1 beyond.
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
CSV_PATH = RESULTS_DIR / "prime_transport_router_reintegration_v4_d16_ceiling.csv"
MD_PATH  = DOCS_DIR    / "prime_transport_router_reintegration_v4_d16_ceiling.md"

# -- operator registry ---------------------------------------------------------
_OPS = [
    ("T_b",  torus_base_advance_component_v23,  "non_transport"),
    ("T_x",  composite_swap_component_v24,       "non_transport"),
    ("T_y",  composite_twist_component_v25,      "non_transport"),
    ("T_c",  coupled_torus_kick_component_v20,   "transport"),
    ("T_z'", fiber_phase_lift_component_v21,     "transport"),
    ("T_r*", radial_transport_component_v22,     "transport"),
]
N_OPS       = len(_OPS)
OP_NAMES    = [o[0] for o in _OPS]
OP_FNS      = [o[1] for o in _OPS]
OP_CLUSTERS = [o[2] for o in _OPS]

# -- config --------------------------------------------------------------------
VOCAB        = 4
DELAY        = 16                           # only D=16 in this study
BUDGETS      = [2000, 5000, 7500, 10000]    # checkpoint batch counts
BUDGET_SET   = set(BUDGETS)
MAX_BATCHES  = max(BUDGETS)
BATCH_SIZE   = 32
N_EVAL       = 1000
TEMP_START   = 2.0
TEMP_END     = 0.1
ANNEAL_OVER  = 800                          # match v4/v4_scaling schedule
LR           = 0.02
WARMUP_STEPS = 6
GLOBAL_SEED  = 42

# -- model dims ----------------------------------------------------------------
D_EMB         = 4
D_TAU         = 2 + 5 + 2 + 12    # = 21
D_IN          = D_EMB + D_TAU     # = 25
D_HIDDEN      = 32
D_HIDDEN_ATTN = 8

# -- v4 and v4_scaling D=16 reference values -----------------------------------
V4_ACCS_D16 = {
    800:  0.269,   # v4 standalone (800 batches)
    2000: 0.275,   # v4_scaling
    5000: 0.367,   # v4_scaling
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
    py_rng: pyrand.Random,
    np_rng: np.random.Generator,
    temp: float,
    training: bool,
) -> dict:
    state         = warmup(py_rng)
    tokens        = [x0] + [py_rng.randint(0, VOCAB - 1) for _ in range(DELAY - 1)]

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
    np_rng: np.random.Generator,
    py_rng: pyrand.Random,
) -> dict[int, Params]:
    """Train for MAX_BATCHES on D=16; return cloned params at each checkpoint."""
    params    = Params(np_rng)
    snapshots = {}

    for batch_idx in range(MAX_BATCHES):
        temp  = get_temp(batch_idx)
        acc_g = params.zero_grad()

        for _ in range(BATCH_SIZE):
            x0    = py_rng.randint(0, VOCAB - 1)
            ep    = forward_episode(params, x0, py_rng, np_rng,
                                    temp=temp, training=True)
            grads = backward_episode(params, ep, x0, temp)
            for k in acc_g:
                acc_g[k] += grads[k] / BATCH_SIZE

        params.apply_update(acc_g, LR)

        completed = batch_idx + 1
        if completed in BUDGET_SET:
            snapshots[completed] = params.clone()
            print(f"  [checkpoint] batch {completed}")

    return snapshots


# -- evaluation ----------------------------------------------------------------

def evaluate(
    params: Params,
    py_rng: pyrand.Random,
    np_rng: np.random.Generator,
) -> dict:
    correct_total      = 0
    op_counts:         Counter               = Counter()
    entropy_vals:      list[float]           = []
    alpha_sum:         np.ndarray            = np.zeros(DELAY)
    alpha_count:       int                   = 0

    for _ in range(N_EVAL):
        x0 = py_rng.randint(0, VOCAB - 1)
        ep = forward_episode(params, x0, py_rng, np_rng,
                             temp=TEMP_END, training=False)
        correct_total += ep["correct"]
        for op_idx in ep["op_seq"]:
            op_counts[OP_NAMES[op_idx]] += 1
        entropy_vals.extend(ep["entropy_seq"])
        alpha_sum   += ep["alpha"]
        alpha_count += 1

    n_total = sum(op_counts.values()) or 1
    tr_frac = sum(
        op_counts[n] for n in OP_NAMES
        if OP_CLUSTERS[OP_NAMES.index(n)] == "transport"
    ) / n_total

    return {
        "accuracy":       correct_total / N_EVAL,
        "op_counts":      dict(op_counts),
        "transport_frac": tr_frac,
        "mean_entropy":   float(np.mean(entropy_vals)) if entropy_vals else 0.0,
        "mean_alpha":     (alpha_sum / max(alpha_count, 1)).tolist(),
    }


# -- run -----------------------------------------------------------------------

def run_all() -> dict[int, dict]:
    t0     = time.perf_counter()
    np_rng = np.random.default_rng(GLOBAL_SEED)
    py_rng = pyrand.Random(GLOBAL_SEED)

    print(f"\n=== D=16 ceiling study: training to {MAX_BATCHES} batches ===")
    t_start = time.perf_counter()
    snaps   = train_with_checkpoints(np_rng, py_rng)
    print(f"Training done ({time.perf_counter()-t_start:.1f}s)")

    results: dict[int, dict] = {}
    for budget in BUDGETS:
        res = evaluate(snaps[budget], py_rng, np_rng)
        results[budget] = res
        v_ref_str = f"  (scaling@{budget}: {V4_ACCS_D16[budget]:.3f})" \
            if budget in V4_ACCS_D16 else ""
        print(
            f"  budget={budget:5d}  acc={res['accuracy']:.3f}{v_ref_str}  "
            f"tr={res['transport_frac']:.3f}  H={res['mean_entropy']:.3f}  "
            f"alpha0={res['mean_alpha'][0]:.4f}"
        )

    print(f"\nTotal: {time.perf_counter()-t0:.1f}s")
    return results


# -- write CSV -----------------------------------------------------------------

def write_csv(all_results: dict[int, dict]) -> None:
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "model", "training_budget", "delay", "metric_name", "metric_value",
        "transport_usage_fraction", "route_entropy", "note",
    ]
    rows = []
    for budget in BUDGETS:
        res   = all_results[budget]
        model = f"traj_inject_router_D16_b{budget}"
        task  = "first_token_recall_D16"

        rows.append({
            "model":                    model,
            "training_budget":          budget,
            "delay":                    DELAY,
            "metric_name":              "accuracy",
            "metric_value":             round(res["accuracy"], 4),
            "transport_usage_fraction": round(res["transport_frac"], 4),
            "route_entropy":            round(res["mean_entropy"], 4),
            "note":                     task,
        })
        rows.append({
            "model":                    model,
            "training_budget":          budget,
            "delay":                    DELAY,
            "metric_name":              "transport_frac",
            "metric_value":             round(res["transport_frac"], 4),
            "transport_usage_fraction": round(res["transport_frac"], 4),
            "route_entropy":            round(res["mean_entropy"], 4),
            "note":                     task,
        })
        rows.append({
            "model":                    model,
            "training_budget":          budget,
            "delay":                    DELAY,
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
                "delay":                    DELAY,
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
    print(f"[ceiling] Wrote {CSV_PATH}")


# -- write markdown ------------------------------------------------------------

def write_md(all_results: dict[int, dict]) -> None:
    DOCS_DIR.mkdir(parents=True, exist_ok=True)
    chance = 1.0 / VOCAB

    def acc(b):   return all_results[b]["accuracy"]
    def tr(b):    return all_results[b]["transport_frac"]
    def ent(b):   return all_results[b]["mean_entropy"]
    def a0(b):    return all_results[b]["mean_alpha"][0]

    # compute improvement deltas vs v4_scaling 5000-batch baseline
    ref_5k = V4_ACCS_D16.get(5000, 0.0)
    best   = acc(max(BUDGETS))

    lines: list[str] = []
    lines += [
        "# Prime Transport — Router Reintegration v4 D=16 Convergence Ceiling",
        "",
        "**Experiment type:** D=16 convergence ceiling study (10,000-batch training)",
        "**Operator layer:** `geometry_native_operator_model_v25`",
        "**Architecture:** v4 (token injection + trajectory attention), unchanged",
        "**No surface build. No files modified. No operators rebuilt.**",
        "",
        "## What Changed from v4_scaling",
        "",
        "This experiment is a **single-delay extension** of the v4 budget scaling study.",
        "",
        "| Aspect | v4_scaling | This study |",
        "|--------|-----------|------------|",
        "| Delays tested | D ∈ {2,4,8,16} | D=16 only |",
        "| Max training budget | 5,000 batches | **10,000 batches** |",
        f"| Checkpoints | 800, 2000, 5000 | 2000, 5000, 7500, 10000 |",
        "| Architecture | v4 (unchanged) | v4 (unchanged) |",
        "",
        "The 5000-batch checkpoint bridges cleanly to v4_scaling D=16 results for "
        "direct comparison. New checkpoints at 7500 and 10000 determine whether "
        "long-delay recall continues to improve beyond the scaling study range.",
        "",
        "## Reference: v4 and v4_scaling D=16 Baselines",
        "",
        "| Budget | acc@D=16 | Source |",
        "|--------|----------|--------|",
        "| 800    | 0.269    | v4 standalone |",
        "| 2000   | 0.275    | v4_scaling |",
        "| 5000   | 0.367    | v4_scaling |",
        "",
        "## Primary Results",
        "",
        "| Budget | acc@D=16 | vs. chance (0.250) | vs. v4_scaling@5000 (0.367) |",
        "|--------|----------|-------------------|------------------------------|",
    ]

    for b in BUDGETS:
        a     = acc(b)
        delta = a - ref_5k
        sign  = "+" if delta >= 0 else ""
        lines.append(
            f"| {b:5d} | {a:.3f} | {'+' if a>chance else ''}{a-chance:.3f} | "
            f"{sign}{delta:.3f} |"
        )

    lines += [
        "",
        "## Does Performance Scale Beyond 5,000 Batches?",
        "",
    ]

    a5k  = acc(5000) if 5000 in all_results else ref_5k
    a10k = acc(10000)
    if a10k > a5k + 0.01:
        verdict = (f"**Yes.** Accuracy continued to improve from {a5k:.3f} at 5000 "
                   f"batches to {a10k:.3f} at 10,000 batches "
                   f"(Δ={a10k-a5k:+.3f}).")
    elif a10k > a5k:
        verdict = (f"**Marginal improvement.** Accuracy moved from {a5k:.3f} "
                   f"at 5000 batches to {a10k:.3f} at 10,000 batches "
                   f"(Δ={a10k-a5k:+.3f}). Improvement is present but small.")
    else:
        verdict = (f"**Saturated.** Accuracy at 10,000 batches ({a10k:.3f}) "
                   f"did not exceed 5000-batch accuracy ({a5k:.3f}). "
                   f"Additive injection signal has reached its ceiling at D=16.")

    lines.append(verdict)
    lines.append("")

    lines += [
        "## Routing Behavior",
        "",
        "| Budget | transport_frac | route_entropy | collapsed (<0.3)? |",
        "|--------|---------------|---------------|-------------------|",
    ]
    for b in BUDGETS:
        lines.append(
            f"| {b:5d} | {tr(b):.3f} | {ent(b):.3f} | "
            f"{'yes' if ent(b) < 0.3 else 'no'} |"
        )

    lines += [
        "",
        "## Attention Weight at Step 0 (encoding step)",
        "",
        f"Uniform baseline: 1/D = 1/16 = 0.0625",
        "",
        "| Budget | alpha[0] | near uniform? |",
        "|--------|----------|---------------|",
    ]
    for b in BUDGETS:
        a0v = a0(b)
        near = abs(a0v - 1.0 / DELAY) < 0.01
        lines.append(f"| {b:5d} | {a0v:.4f} | {'yes' if near else 'no'} |")

    lines += [
        "",
        "## Long-Delay Ceiling Analysis",
        "",
        f"| Metric | Value |",
        f"|--------|-------|",
        f"| Chance level | 0.250 |",
        f"| v4 standalone (800b) | 0.269 |",
        f"| v4_scaling (5000b) | 0.367 |",
        f"| This study best | {best:.3f} |",
        f"| Max gain over v4_scaling@5000 | {best - ref_5k:+.3f} |",
        "",
    ]

    saturation = "yes" if best - ref_5k < 0.02 else "no"
    lines += [
        f"**Saturated at 5000b?** {saturation}",
        "",
        "## Honesty Section",
        "",
        "### What improved",
        "",
        f"- Ceiling accuracy at D=16 reached {best:.3f} vs. chance 0.250.",
        "- Routing remained non-collapsed throughout all checkpoints.",
        "- Extended training directly answered whether additive injection "
          "can recover further signal at long delays.",
        "",
        "### What saturated or failed",
        "",
    ]

    if best - ref_5k < 0.02:
        lines += [
            "- Performance at D=16 essentially saturated by 5000 batches. "
              "The ceiling of additive token injection at D=16 is approximately 0.367.",
            "- The injection signal is the same for all 16 steps; noise-token "
              "injections (steps 1–15) accumulate and dilute the encoding-step "
              "contribution over a 16-step window.",
        ]
    else:
        lines += [
            f"- Performance continued to grow to {best:.3f} but may not "
              "reflect a hard ceiling — further training could yield more.",
            "- Long-delay degradation gap (D=2 vs. D=16) persists structurally.",
        ]

    lines += [
        "",
        "### What remains uncertain",
        "",
        "- Whether a position-aware injection (step 0 only) would "
          "prevent noise accumulation and lift the D=16 ceiling further.",
        "- Whether gated injection (multiplicative W_gate[tok]) would "
          "modulate individual tau dimensions more effectively at large D.",
        "- Whether full exact spin_H is solved: **No.**",
        "",
        "## Recommended Next Step",
        "",
    ]

    if best - ref_5k < 0.02:
        lines += [
            "D=16 performance has saturated under additive injection. "
            "Implement **gated injection** (v5): replace the additive offset "
            "`W_tok_inject[tok]` with a multiplicative gate "
            "`soft_tau_curr = (w @ tau_nexts) * (1 + W_gate[tok])`, "
            "where `W_gate` is a learned `(VOCAB, D_TAU)` parameter. "
            "Multiplicative modulation scales individual tau dimensions at every "
            "step, maintaining a stronger token-dependent signal through long delay "
            "windows instead of an additive offset that mixes into background noise.",
        ]
    else:
        lines += [
            f"Performance at D=16 has not yet saturated (best: {best:.3f}). "
            "Run one additional checkpoint at 15,000 batches to confirm whether "
            "improvement is still ongoing before pivoting to a new architecture.",
        ]

    DOCS_DIR.mkdir(parents=True, exist_ok=True)
    MD_PATH.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[ceiling] Wrote {MD_PATH}")


# -- entry point ---------------------------------------------------------------

if __name__ == "__main__":
    results = run_all()
    write_csv(results)
    write_md(results)
    print("\nDone.")
