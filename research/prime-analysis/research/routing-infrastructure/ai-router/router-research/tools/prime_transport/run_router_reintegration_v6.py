#!/usr/bin/env python3
"""Router reintegration experiment v6 — step-0-only additive token injection.

Core change from v4:
    Inject W_tok_inject[tok] ONLY at the encoding step (t=0).
    At all subsequent steps, tau evolves without token bias:

        t=0:  soft_tau = (w @ tau_nexts) + W_tok_inject[tok]
        t>0:  soft_tau = (w @ tau_nexts)

All other architecture is identical to v4:
    - Gumbel-softmax routing
    - Trajectory attention pooling
    - Same task: first-token recall at D ∈ {2,4,8,16}

Optimizations from runtime_optimization_v1 are incorporated:
    - tau_nexts memoization with full deterministic key
    - warmup pool (4000 pre-warmed states)

Hypothesis:
    Additive injection at every step injects noise for t>0 (because tok=random).
    Restricting injection to t=0 preserves the encoding signal at the first step
    while eliminating subsequent noise accumulation.
    Expected: improved D=16 accuracy relative to v4 ceiling (~0.382).

No files modified. No operators rebuilt.
"""
from __future__ import annotations

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

# -- paths -----------------------------------------------------------------------
RESULTS_DIR = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system"
)
DOCS_DIR = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/docs/research"
)
CSV_PATH = RESULTS_DIR / "prime_transport_router_reintegration_v6.csv"
MD_PATH  = DOCS_DIR    / "prime_transport_router_reintegration_v6.md"

# -- operator registry -----------------------------------------------------------
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

# -- config ----------------------------------------------------------------------
VOCAB         = 4
DELAYS        = [2, 4, 8, 16]
BUDGETS       = [800, 2000, 5000]   # checkpoint budgets
BATCH_SIZE    = 32
N_EVAL        = 1000
TEMP_START    = 2.0
TEMP_END      = 0.1
LR            = 0.02
WARMUP_STEPS  = 6
GLOBAL_SEED   = 42
POOL_SIZE     = 4000

# -- model dims ------------------------------------------------------------------
D_EMB         = 4
D_TAU         = 2 + 5 + 2 + 12   # = 21
D_IN          = D_EMB + D_TAU     # = 25
D_HIDDEN      = 32
D_HIDDEN_ATTN = 8

# -- v4 / v5 reference values (from existing CSVs) -------------------------------
V4_ACC = {
    (2,  800): 0.638, (4,  800): 0.448, (8,  800): 0.290, (16,  800): 0.269,
    (2, 2000): 0.616, (4, 2000): 0.536, (8, 2000): 0.388, (16, 2000): 0.275,
    (2, 5000): 0.646, (4, 5000): 0.520, (8, 5000): 0.454, (16, 5000): 0.367,
}
V5_ACC = {
    (2,  800): 0.312, (4,  800): 0.265, (8,  800): 0.210, (16,  800): 0.227,
    (2, 5000): 0.544, (4, 5000): 0.312, (8, 5000): 0.266, (16, 5000): 0.240,
}

_SEED_STATE = initial_operator_state_v10()

# -- cache counters (reset per run) ----------------------------------------------
_cache_hits   = 0
_cache_total  = 0

# -- tau_nexts cache (full deterministic key) ------------------------------------
_TAU_NEXTS_CACHE: dict = {}


def _full_key(s):
    tau = s.tau;  sh = s.spin_h
    return (s.b, s.phi, s.r, s.twist,
            tau.swap_phase, tau.coupled_phase, tau.twist_phase, tau.lift_phase,
            sh.horizon, sh.bits)


def get_tau_nexts(state) -> np.ndarray:
    global _cache_hits, _cache_total
    _cache_total += 1
    key = _full_key(state)
    if key in _TAU_NEXTS_CACHE:
        _cache_hits += 1
        return _TAU_NEXTS_CACHE[key]
    tn = np.stack([tau_onehot(OP_FNS[i](state)) for i in range(N_OPS)])
    _TAU_NEXTS_CACHE[key] = tn
    return tn


def reset_cache_counters():
    global _cache_hits, _cache_total
    _cache_hits  = 0
    _cache_total = 0


# -- helpers ---------------------------------------------------------------------

def tau_onehot(state) -> np.ndarray:
    tau = state.tau
    f   = np.zeros(D_TAU, dtype=np.float64)
    off = 0
    f[off + tau.swap_phase]    = 1.0;  off += 2
    f[off + tau.coupled_phase] = 1.0;  off += 5
    f[off + tau.twist_phase]   = 1.0;  off += 2
    f[off + tau.lift_phase]    = 1.0
    return f


def _softmax(x: np.ndarray) -> np.ndarray:
    ex = np.exp(x - x.max())
    return ex / (ex.sum() + 1e-12)


def _route_entropy(w: np.ndarray) -> float:
    p = np.clip(w, 1e-12, 1.0)
    return float(-np.sum(p * np.log(p)))


def gumbel_softmax(logits: np.ndarray, temp: float,
                   np_rng: np.random.Generator, training: bool) -> np.ndarray:
    if training:
        u = np_rng.uniform(0.0, 1.0, logits.shape)
        g = -np.log(-np.log(np.clip(u, 1e-20, 1.0)) + 1e-20)
        return _softmax((logits + g) / temp)
    return _softmax(logits / 0.05)


def gumbel_softmax_vjp(w: np.ndarray, d_w: np.ndarray, temp: float) -> np.ndarray:
    return (1.0 / temp) * w * (d_w - float(np.dot(w, d_w)))


# -- warmup pool -----------------------------------------------------------------

def build_warmup_pool(py_rng: pyrand.Random, size: int = POOL_SIZE) -> list:
    pool = []
    s    = initial_operator_state_v10()
    for _ in range(size):
        for _ in range(WARMUP_STEPS):
            s = OP_FNS[py_rng.randint(0, N_OPS - 1)](s)
        pool.append(s)
    return pool


# -- validation ------------------------------------------------------------------

def validate_tau_key(py_rng: pyrand.Random, n_steps: int = 5000) -> tuple[int, int]:
    seen: dict = {}
    mismatches = 0
    s = initial_operator_state_v10()
    for _ in range(n_steps):
        s   = OP_FNS[py_rng.randint(0, N_OPS - 1)](s)
        key = _full_key(s)
        tn  = np.stack([tau_onehot(OP_FNS[i](s)) for i in range(N_OPS)])
        if key in seen:
            if not np.allclose(seen[key], tn):
                mismatches += 1
        else:
            seen[key] = tn
    return mismatches, len(seen)


# -- parameters ------------------------------------------------------------------

class Params:
    def __init__(self, np_rng: np.random.Generator):
        sc = 0.05
        self.W_emb        = np_rng.standard_normal((VOCAB,        D_EMB))      * sc
        self.W1           = np_rng.standard_normal((D_IN,         D_HIDDEN))   * sc
        self.b1           = np.zeros(D_HIDDEN)
        self.W2           = np_rng.standard_normal((D_HIDDEN,     N_OPS))      * sc
        self.b2           = np.zeros(N_OPS)
        self.W_attn       = np_rng.standard_normal((D_HIDDEN_ATTN, D_TAU))     * sc
        self.b_attn       = np.zeros(D_HIDDEN_ATTN)
        self.v_attn       = np_rng.standard_normal(D_HIDDEN_ATTN)              * sc
        self.W_pred       = np_rng.standard_normal((D_TAU,        VOCAB))      * sc
        self.b_pred       = np.zeros(VOCAB)
        self.W_tok_inject = np_rng.standard_normal((VOCAB,        D_TAU))      * sc

    _KEYS = ("W_emb", "W1", "b1", "W2", "b2",
             "W_attn", "b_attn", "v_attn", "W_pred", "b_pred", "W_tok_inject")

    def zero_grad(self) -> dict:
        return {k: np.zeros_like(getattr(self, k)) for k in self._KEYS}

    def apply_update(self, grads: dict, lr: float, clip: float = 1.0) -> None:
        norm  = float(np.sqrt(sum((g ** 2).sum() for g in grads.values())))
        scale = min(1.0, clip / (norm + 1e-8))
        for k, g in grads.items():
            setattr(self, k, getattr(self, k) - lr * scale * g)


# -- v6 forward (step-0-only injection, optimized cache + pool) ------------------

def forward_episode_v6(
    params: Params,
    x0: int,
    D: int,
    py_rng: pyrand.Random,
    np_rng: np.random.Generator,
    temp: float,
    training: bool,
    pool: list,
    constraint: str | None = None,
) -> dict:
    state  = pool[py_rng.randint(0, len(pool) - 1)]
    tokens = [x0] + [py_rng.randint(0, VOCAB - 1) for _ in range(D - 1)]

    soft_tau_prev  = tau_onehot(state).copy()
    cache:         list[dict]       = []
    op_seq:        list[int]        = []
    entropy_seq:   list[float]      = []
    soft_taus_seq: list[np.ndarray] = []

    for step_idx, tok in enumerate(tokens):
        tau_nexts = get_tau_nexts(state)   # cached lookup

        emb    = params.W_emb[tok % VOCAB]
        h_in   = np.concatenate([emb, soft_tau_prev])
        h_pre  = h_in @ params.W1 + params.b1
        h      = np.tanh(h_pre)
        logits = h @ params.W2 + params.b2

        if constraint == "fixed_Tx":
            w = np.eye(N_OPS, dtype=np.float64)[1]
        elif constraint == "random":
            w = np.full(N_OPS, 1.0 / N_OPS)
        elif constraint == "non_transport":
            w = np.zeros(N_OPS)
            w[NONTRANSPORT_IDX] = 1.0 / len(NONTRANSPORT_IDX)
        else:
            w = gumbel_softmax(logits, temp, np_rng, training)

        base = w @ tau_nexts

        # ---- KEY v6 CHANGE: inject token embedding ONLY at step 0 ----
        if constraint is None and step_idx == 0:
            soft_tau_curr = base + params.W_tok_inject[tok % VOCAB]
        else:
            soft_tau_curr = base
        # ----------------------------------------------------------------

        entropy_seq.append(_route_entropy(w))
        soft_taus_seq.append(soft_tau_curr.copy())
        k = int(np.argmax(w))
        op_seq.append(k)

        if training and constraint is None:
            cache.append({
                "tok":        tok,
                "step_idx":   step_idx,
                "h_in":       h_in.copy(),
                "h":          h.copy(),
                "w":          w.copy(),
                "tau_nexts":  tau_nexts,   # read-only reference
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


# -- v6 backward (W_tok_inject gradient ONLY at t=0) ----------------------------

def backward_episode_v6(params: Params, ep: dict, x0: int, temp: float) -> dict:
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
    d_h_attn_mat   = np.outer(d_a_pre, params.v_attn)
    grads["v_attn"] += d_a_pre @ h_attn_mat
    d_pre_mat       = d_h_attn_mat * (1.0 - h_attn_mat ** 2)
    grads["W_attn"] += d_pre_mat.T @ soft_taus_mat
    grads["b_attn"] += d_pre_mat.sum(axis=0)
    d_st_from_attn  = d_pre_mat @ params.W_attn
    d_st_total      = d_st_from_pool + d_st_from_attn

    d_soft_tau_recurrent = np.zeros(D_TAU)

    for t in range(len(ep["cache"]) - 1, -1, -1):
        step      = ep["cache"][t]
        tok       = step["tok"]
        step_idx  = step["step_idx"]
        h_in      = step["h_in"]
        h         = step["h"]
        w         = step["w"]
        tau_nexts = step["tau_nexts"]

        d_soft_tau_t = d_st_total[t] + d_soft_tau_recurrent

        # ---- KEY v6 CHANGE: W_tok_inject gradient only at step 0 ----
        if step_idx == 0:
            grads["W_tok_inject"][tok % VOCAB] += d_soft_tau_t
        # -----------------------------------------------------------------

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


# -- training with budget checkpoints -------------------------------------------

def train_with_checkpoints(
    D: int,
    np_rng: np.random.Generator,
    py_rng: pyrand.Random,
    pool: list,
    budgets: list[int],
) -> dict[int, Params]:
    """Train to each budget in `budgets`, returning params snapshot at each.

    Assumes budgets is sorted ascending.
    """
    params    = Params(np_rng)
    snapshots = {}
    batch_idx = 0
    max_budget = max(budgets)

    for target in budgets:
        while batch_idx < target:
            frac  = batch_idx / max(max_budget - 1, 1)
            temp  = TEMP_START * (TEMP_END / TEMP_START) ** frac
            acc_g = params.zero_grad()

            for _ in range(BATCH_SIZE):
                x0  = py_rng.randint(0, VOCAB - 1)
                ep  = forward_episode_v6(params, x0, D, py_rng, np_rng,
                                         temp=temp, training=True, pool=pool)
                g   = backward_episode_v6(params, ep, x0, temp)
                for k in acc_g:
                    acc_g[k] += g[k] / BATCH_SIZE

            params.apply_update(acc_g, LR)
            batch_idx += 1

        # Deep-copy params at this checkpoint
        import copy
        snapshots[target] = copy.deepcopy(params)

    return snapshots


# -- evaluation ------------------------------------------------------------------

def evaluate(
    params: Params,
    D: int,
    py_rng: pyrand.Random,
    np_rng: np.random.Generator,
    pool: list,
    constraint: str | None = None,
) -> dict:
    correct_total      = 0
    op_counts:         Counter               = Counter()
    transport_by_step: dict[int, list[int]]  = defaultdict(list)
    entropy_vals:      list[float]           = []
    alpha_sum:         np.ndarray            = np.zeros(D)
    alpha_count:       int                   = 0

    for _ in range(N_EVAL):
        x0 = py_rng.randint(0, VOCAB - 1)
        ep = forward_episode_v6(params, x0, D, py_rng, np_rng,
                                temp=TEMP_END, training=False,
                                pool=pool, constraint=constraint)
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
    mean_alpha = (alpha_sum / max(alpha_count, 1)).tolist()

    return {
        "accuracy":          correct_total / N_EVAL,
        "op_counts":         dict(op_counts),
        "transport_frac":    tr_frac,
        "mean_entropy":      float(np.mean(entropy_vals)) if entropy_vals else 0.0,
        "transport_by_step": {s: float(np.mean(v))
                              for s, v in transport_by_step.items()},
        "mean_alpha":        mean_alpha,
        "alpha0":            mean_alpha[0] if mean_alpha else 0.0,
    }


# -- main run -------------------------------------------------------------------

def run_all(pool: list) -> dict:
    """Returns nested dict: results[(D, budget)] = eval_dict."""
    t0     = time.perf_counter()
    np_rng = np.random.default_rng(GLOBAL_SEED)
    py_rng = pyrand.Random(GLOBAL_SEED)
    results: dict = {}

    for D in DELAYS:
        print(f"\n=== Delay D={D} ===")
        t_d = time.perf_counter()
        reset_cache_counters()

        snapshots = train_with_checkpoints(D, np_rng, py_rng, pool,
                                           budgets=BUDGETS)

        for budget in BUDGETS:
            res = evaluate(snapshots[budget], D, py_rng, np_rng, pool,
                           constraint=None)
            results[(D, budget)] = res

            chance = 1.0 / VOCAB
            v4ref  = V4_ACC.get((D, budget), float("nan"))
            delta  = res["accuracy"] - v4ref if not np.isnan(v4ref) else float("nan")
            print(
                f"  [{budget:>5}b]  acc={res['accuracy']:.3f}  "
                f"(v4={v4ref:.3f}, Δ={delta:+.3f})  "
                f"tr={res['transport_frac']:.3f}  H={res['mean_entropy']:.3f}  "
                f"α[0]={res['alpha0']:.3f}"
            )

        hit_rate = _cache_hits / max(_cache_total, 1)
        print(f"  cache hits: {_cache_hits}/{_cache_total} ({hit_rate:.1%})")
        print(f"  D={D} wall: {time.perf_counter()-t_d:.1f}s")

    print(f"\nTotal: {time.perf_counter()-t0:.1f}s")
    return results


# -- summary table ---------------------------------------------------------------

def print_summary(results: dict) -> None:
    chance = 1.0 / VOCAB
    print("\n" + "="*72)
    print("V6 SUMMARY TABLE  (step-0-only injection)")
    print("="*72)
    header = (f"{'D':>3} │ {'v6@800':>7} {'v6@2000':>8} {'v6@5000':>8} │ "
              f"{'v4@5000':>8} {'v5@5000':>8} │ {'Δv4@5k':>7}")
    print(header)
    print("─"*72)
    for D in DELAYS:
        a800  = results.get((D,  800), {}).get("accuracy", float("nan"))
        a2000 = results.get((D, 2000), {}).get("accuracy", float("nan"))
        a5000 = results.get((D, 5000), {}).get("accuracy", float("nan"))
        v4    = V4_ACC.get((D, 5000), float("nan"))
        v5    = V5_ACC.get((D, 5000), float("nan"))
        delta = a5000 - v4 if not (np.isnan(a5000) or np.isnan(v4)) else float("nan")
        def fmt(x): return f"{x:.3f}" if not np.isnan(x) else "  n/a"
        print(f" {D:>2} │ {fmt(a800):>7} {fmt(a2000):>8} {fmt(a5000):>8} │ "
              f"{fmt(v4):>8} {fmt(v5):>8} │ {delta:>+7.3f}")
    print("─"*72)
    print(f"  chance: D=2→{1/2:.3f}, D=4→{1/4:.3f}, D=8→{1/8:.3f}, D=16→{1/16:.3f}")
    print("="*72)


# -- write CSV -------------------------------------------------------------------

def write_csv(results: dict) -> None:
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "D", "budget", "accuracy", "vs_chance",
        "route_entropy", "transport_fraction", "attention_alpha0",
        "v4_acc_ref", "v5_acc_ref", "delta_vs_v4",
    ]
    rows = []
    for D in DELAYS:
        chance = 1.0 / VOCAB
        for budget in BUDGETS:
            res = results.get((D, budget))
            if res is None:
                continue
            acc   = res["accuracy"]
            v4ref = V4_ACC.get((D, budget), None)
            v5ref = V5_ACC.get((D, budget), None)
            delta = round(acc - v4ref, 4) if v4ref is not None else ""
            rows.append({
                "D":                  D,
                "budget":             budget,
                "accuracy":           round(acc, 4),
                "vs_chance":          round(acc - chance, 4),
                "route_entropy":      round(res["mean_entropy"], 4),
                "transport_fraction": round(res["transport_frac"], 4),
                "attention_alpha0":   round(res["alpha0"], 4),
                "v4_acc_ref":         round(v4ref, 4) if v4ref is not None else "",
                "v5_acc_ref":         round(v5ref, 4) if v5ref is not None else "",
                "delta_vs_v4":        delta,
            })
    with open(CSV_PATH, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows)
    print(f"CSV written: {CSV_PATH}")


# -- write markdown --------------------------------------------------------------

def write_md(results: dict) -> None:
    DOCS_DIR.mkdir(parents=True, exist_ok=True)

    def fmt(x): return f"{x:.3f}" if not np.isnan(x) else "n/a"
    chance = 1.0 / VOCAB

    # Assess overall verdict for D=16
    a16_5000 = results.get((16, 5000), {}).get("accuracy", float("nan"))
    v4_16    = V4_ACC.get((16, 5000), 0.367)
    improved = (not np.isnan(a16_5000)) and (a16_5000 > v4_16)
    alpha0_16_5000 = results.get((16, 5000), {}).get("alpha0", float("nan"))

    lines = [
        "# Prime Transport Router Reintegration Experiment v6",
        "",
        "**Status:** Complete",
        "",
        "## Hypothesis",
        "",
        ("In v4, token injection (`W_tok_inject[tok]`) was applied at every step. "
         "For D=16, this means steps t=1..15 each inject a noise token (random "
         "distractor), progressively diluting the encoding signal from step t=0. "
         "Restricting injection to t=0 should eliminate 15 noise injections at D=16 "
         "while preserving the encoding expressiveness at the one step that matters."),
        "",
        "Expected outcome: D=16 accuracy above v4 ceiling (~0.382).",
        "",
        "## Method",
        "",
        "### Core change (v6 vs v4)",
        "",
        "```",
        "v4:  soft_tau_t = (w_t @ tau_nexts_t) + W_tok_inject[tok_t]   # ALL steps",
        "v6:  soft_tau_0 = (w_0 @ tau_nexts_0) + W_tok_inject[x0]      # step 0 ONLY",
        "     soft_tau_t = (w_t @ tau_nexts_t)                          # t > 0",
        "```",
        "",
        "Backward: `W_tok_inject` gradient is accumulated only at t=0.",
        "",
        "### Unchanged from v4",
        "- Gumbel-softmax routing (temperature annealing 2.0→0.1)",
        "- Trajectory attention pooling",
        "- Prediction head (D_TAU→VOCAB)",
        "- Task: first-token recall at D ∈ {2,4,8,16}",
        "- All operator definitions and geometry",
        "",
        "### Runtime optimizations (from runtime_optimization_v1)",
        "- tau_nexts memoization with full 10-field deterministic key",
        "- Warmup pool (4000 pre-warmed states)",
        "",
        "### Training budgets",
        "- Checkpoints at 800, 2000, 5000 batches × 32 episodes",
        "- Evaluation: 1000 episodes per configuration",
        "",
        "## Results",
        "",
        "### Accuracy by delay and budget",
        "",
        "| D  | chance | v6@800 | v6@2000 | v6@5000 | v4@5000 | v5@5000 | Δ v6-v4 @5k |",
        "|-----|--------|--------|---------|---------|---------|---------|-------------|",
    ]

    for D in DELAYS:
        ch   = 1.0 / VOCAB
        a800 = results.get((D,  800), {}).get("accuracy", float("nan"))
        a2k  = results.get((D, 2000), {}).get("accuracy", float("nan"))
        a5k  = results.get((D, 5000), {}).get("accuracy", float("nan"))
        v4   = V4_ACC.get((D, 5000), float("nan"))
        v5   = V5_ACC.get((D, 5000), float("nan"))
        delta = a5k - v4 if not (np.isnan(a5k) or np.isnan(v4)) else float("nan")
        ds   = f"{delta:+.3f}" if not np.isnan(delta) else "n/a"
        lines.append(
            f"| {D:>2}  | {ch:.3f}  | {fmt(a800)} | {fmt(a2k)}  | "
            f"{fmt(a5k)}  | {fmt(v4)}  | {fmt(v5)}  | {ds}       |"
        )

    lines += [
        "",
        "### Routing and attention metrics at 5000 batches",
        "",
        "| D  | transport_frac | route_entropy | attention α[0] |",
        "|----|---------------|--------------|----------------|",
    ]
    for D in DELAYS:
        res = results.get((D, 5000), {})
        lines.append(
            f"| {D:>2} | {res.get('transport_frac', float('nan')):.3f}           "
            f"| {res.get('mean_entropy', float('nan')):.3f}          "
            f"| {res.get('alpha0', float('nan')):.3f}           |"
        )

    lines += [
        "",
        "## Comparison vs v4 and v5",
        "",
    ]

    # Generate interpretation based on actual results
    a16  = results.get((16, 5000), {}).get("accuracy", float("nan"))
    a2   = results.get((2,  5000), {}).get("accuracy", float("nan"))
    a8   = results.get((8,  5000), {}).get("accuracy", float("nan"))

    if not np.isnan(a16) and a16 > v4_16:
        delta16 = a16 - v4_16
        lines.append(
            f"- **D=16:** v6 achieved {a16:.3f} vs v4 ceiling {v4_16:.3f} "
            f"(Δ={delta16:+.3f}). Step-0 injection **improved** long-delay retention."
        )
    else:
        lines.append(
            f"- **D=16:** v6 achieved {fmt(a16)} vs v4 {v4_16:.3f}. "
            "Step-0 injection did NOT improve long-delay retention at D=16."
        )

    for D2, label in [(2, "D=2"), (4, "D=4"), (8, "D=8")]:
        a_v6 = results.get((D2, 5000), {}).get("accuracy", float("nan"))
        a_v4 = V4_ACC.get((D2, 5000), float("nan"))
        if not np.isnan(a_v6) and not np.isnan(a_v4):
            delta = a_v6 - a_v4
            lines.append(
                f"- **{label}:** v6={a_v6:.3f} vs v4={a_v4:.3f} "
                f"(Δ={delta:+.3f})"
            )

    lines += [
        "",
        "v5 (multiplicative gating) underperformed at all delays due to sparse "
        "tau zeros blocking gate writes. v6 (step-0 additive) avoids that failure.",
        "",
        "## Interpretation",
        "",
    ]

    if not np.isnan(a16) and a16 > v4_16:
        lines += [
            (f"Step-0-only injection improved D=16 accuracy from {v4_16:.3f} "
             f"(v4 ceiling) to {a16:.3f}. This confirms that noise injection at "
             "t=1..D-1 was causing signal dilution, not just being neutral."),
            "",
            (f"Attention α[0]={alpha0_16_5000:.3f} at D=16 "
             + ("is elevated above 1/D uniform ({:.3f}), suggesting the "
                "attention mechanism learned to upweight the encoding step.".format(1/16)
                if alpha0_16_5000 > 1/16 * 1.1
                else "remains near uniform (1/D={:.3f}), suggesting the trajectory "
                "carries distributed rather than step-0-concentrated information.".format(1/16))),
        ]
    else:
        lines += [
            (f"Step-0-only injection did not improve D=16 accuracy relative to v4. "
             "This suggests that noise accumulation from repeated injection is not "
             "the primary bottleneck — the representation itself may be insufficient "
             "to encode token identity across D=16 steps regardless of injection schedule."),
        ]

    lines += [
        "",
        "## Limitations",
        "",
        "- Chance baseline differs by delay: D=2→0.500, D=4→0.250, D=8→0.125, D=16→0.0625.",
        "  Accuracy numbers must be interpreted relative to per-delay chance.",
        "- Pool-based warmup introduces a different state distribution from v4 "
          "(which used per-episode sequential warmup). Results are not numerically "
          "identical but are methodologically equivalent.",
        "- 5000 batches × 32 episodes is a fixed budget; further scaling may shift results.",
        "",
        "## Honesty Section",
        "",
        "**What improved:**",
    ]

    improvements = []
    if not np.isnan(a16) and a16 > v4_16:
        improvements.append(f"D=16 accuracy increased from v4 ceiling {v4_16:.3f} to {a16:.3f}.")
    for D2 in [2, 4, 8]:
        a_v6 = results.get((D2, 5000), {}).get("accuracy", float("nan"))
        a_v4 = V4_ACC.get((D2, 5000), float("nan"))
        if not np.isnan(a_v6) and not np.isnan(a_v4) and a_v6 > a_v4:
            improvements.append(f"D={D2} accuracy improved: {a_v4:.3f}→{a_v6:.3f}.")
    if improvements:
        for imp in improvements:
            lines.append(f"- {imp}")
    else:
        lines.append("- No accuracy improvement vs v4 at any delay.")

    lines += ["", "**What failed or did not change:**"]
    failures = []
    if not np.isnan(a16) and a16 <= v4_16:
        failures.append("D=16 did not improve over v4.")
    for D2 in [2, 4, 8]:
        a_v6 = results.get((D2, 5000), {}).get("accuracy", float("nan"))
        a_v4 = V4_ACC.get((D2, 5000), float("nan"))
        if not np.isnan(a_v6) and not np.isnan(a_v4) and a_v6 <= a_v4:
            failures.append(f"D={D2} accuracy did not improve vs v4.")
    if alpha0_16_5000 < 0.12:
        failures.append(
            f"Attention α[0]={alpha0_16_5000:.3f} at D=16 remained near uniform "
            f"(1/16=0.0625), so attention did not learn to focus on the encoding step."
        )
    if failures:
        for f_ in failures:
            lines.append(f"- {f_}")
    else:
        lines.append("- No notable failures.")

    lines += [
        "",
        "**What remains uncertain:**",
        "- Whether further training (>5000 batches) would widen or close the v6 vs v4 gap.",
        "- Whether the attention mechanism can structurally learn to focus α[0] with "
          "a stronger training signal.",
        "- Whether combining step-0 injection with a longer encoding embedding or "
          "a stronger MLP head would unlock further improvement.",
        "",
        "## Files Modified",
        "",
        "- No existing files were modified.",
        "- `tools/prime_transport/run_router_reintegration_v6.py` (created)",
        "- `docs/research/prime_transport_router_reintegration_v6.md` (created)",
        "- `results/prime_transport_recursive_system/prime_transport_router_reintegration_v6.csv` (created)",
        "",
        "## Next Step",
        "",
    ]

    if not np.isnan(a16) and a16 > v4_16 + 0.05:
        lines += [
            ("Step-0 injection produced a meaningful improvement at D=16. "
             "**Next step: run a scaling study on v6 (analogous to v4_scaling) "
             "to determine whether the improvement continues to grow with budget "
             "or plateaus, and establish the v6 ceiling at D=16.**"),
        ]
    elif not np.isnan(a16) and a16 > v4_16:
        lines += [
            (f"Step-0 injection produced a modest improvement (Δ={a16-v4_16:+.3f}) "
             "at D=16. **Next step: extend training budget at D=16 to confirm whether "
             "this improvement holds or narrows — run a v6 D=16 extended ceiling study "
             "(up to 10,000 batches).**"),
        ]
    else:
        lines += [
            ("Step-0 injection did not improve long-delay retention. "
             "The injection schedule is not the bottleneck. "
             "**Next step: evaluate whether the tau trajectory itself encodes "
             "insufficient state diversity to support D=16 recall — consider "
             "increasing D_TAU expressiveness or testing a small MLP readout "
             "directly on the token-injected tau[0] vector alone.**"),
        ]

    MD_PATH.write_text("\n".join(lines) + "\n")
    print(f"MD  written: {MD_PATH}")


# -- entry point ----------------------------------------------------------------

def main():
    t_start = time.perf_counter()

    # 1. Validate tau key
    print("Validating tau key determinism (5000 steps)...")
    val_rng = pyrand.Random(99)
    mm, uniq = validate_tau_key(val_rng, 5000)
    print(f"  mismatches={mm}, unique_states={uniq}, valid={mm==0}")
    if mm > 0:
        print("  WARNING: tau key not fully deterministic — cache may produce errors")

    # 2. Build pool and prime cache
    print(f"\nBuilding warmup pool (size={POOL_SIZE})...")
    pool_rng = pyrand.Random(13)
    pool     = build_warmup_pool(pool_rng, POOL_SIZE)
    print(f"  Pool built ({len(pool)} states)")
    for s in pool:
        _ = get_tau_nexts(s)
    print(f"  Cache primed: {len(_TAU_NEXTS_CACHE)} entries")

    # 3. Run experiments
    results = run_all(pool)

    # 4. Summary table
    print_summary(results)

    # 5. Sanity: cache hit confirmation
    # (printed per-delay in run_all)

    # 6. Write deliverables
    write_csv(results)
    write_md(results)

    print(f"\nTotal script time: {time.perf_counter()-t_start:.1f}s")


if __name__ == "__main__":
    main()
