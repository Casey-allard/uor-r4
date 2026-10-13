#!/usr/bin/env python3
"""Router reintegration experiment v3 — trajectory-level attention readout.

Changes from v2:
  - Terminal-state linear prediction head replaced with learned attention-weighted
    pooling over the full soft-tau trajectory (all D soft tau states)
  - New attention module:
      h_t     = tanh(W_attn @ soft_tau_t + b_attn)    shape: (D_HIDDEN_ATTN,)
      a_t     = v_attn @ h_t                           scalar
      alpha   = softmax(a_0, ..., a_{D-1})             shape: (D,)
      pooled  = sum_t alpha_t * soft_tau_t             shape: (D_TAU,)
      logits  = pooled @ W_pred + b_pred               shape: (VOCAB,)
  - Gradients flow: loss -> W_pred -> pooled -> alpha, all D soft taus
    -> W_attn, b_attn, v_attn
    -> each step's routing weights -> router MLP -> W_emb
  - All D tau states now contribute to the readout; the model can learn to
    attend to early steps (encoding step) even at large D

Hypothesis: if token-identity is encoded in the tau trajectory rather than
in the terminal tau state alone, a trajectory attention readout will be able
to recover that signal where v2's terminal readout could not.

No files modified.  No operators rebuilt.
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

# -- paths ---------------------------------------------------------------------
RESULTS_DIR = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system"
)
DOCS_DIR = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/docs/research"
)
CSV_PATH = RESULTS_DIR / "prime_transport_router_reintegration_v3.csv"
MD_PATH  = DOCS_DIR    / "prime_transport_router_reintegration_v3.md"

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
TRANSPORT_IDX    = [i for i, o in enumerate(_OPS) if o[2] == "transport"]
NONTRANSPORT_IDX = [i for i, o in enumerate(_OPS) if o[2] == "non_transport"]

# -- task / training config ----------------------------------------------------
VOCAB          = 4
DELAYS         = [2, 4, 8, 16]
N_BATCHES      = 800
BATCH_SIZE     = 32
N_EVAL         = 1000
TEMP_START     = 2.0
TEMP_END       = 0.1
LR             = 0.02
WARMUP_STEPS   = 6
GLOBAL_SEED    = 42

# -- model dims ----------------------------------------------------------------
D_EMB         = 4
D_TAU         = 2 + 5 + 2 + 12   # = 21
D_IN          = D_EMB + D_TAU    # = 25
D_HIDDEN      = 32
D_HIDDEN_ATTN = 8

# -- v2 reference numbers for explicit comparison ------------------------------
V2_ACCS = {
    ("gumbel_router", 2):  0.234, ("gumbel_router", 4):  0.233,
    ("gumbel_router", 8):  0.254, ("gumbel_router", 16): 0.247,
    ("fixed_Tx",      2):  0.240, ("fixed_Tx",      4):  0.272,
    ("fixed_Tx",      8):  0.245, ("fixed_Tx",      16): 0.252,
    ("random",        2):  0.245, ("random",        4):  0.240,
    ("random",        8):  0.247, ("random",        16): 0.259,
    ("non_transport", 2):  0.227, ("non_transport", 4):  0.242,
    ("non_transport", 8):  0.228, ("non_transport", 16): 0.264,
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
    """Standard softmax VJP scaled by 1/temp."""
    return (1.0 / temp) * w * (d_w - float(np.dot(w, d_w)))


# -- parameters ----------------------------------------------------------------

class Params:
    def __init__(self, np_rng: np.random.Generator):
        sc = 0.05
        self.W_emb   = np_rng.standard_normal((VOCAB,        D_EMB))         * sc
        self.W1      = np_rng.standard_normal((D_IN,         D_HIDDEN))      * sc
        self.b1      = np.zeros(D_HIDDEN)
        self.W2      = np_rng.standard_normal((D_HIDDEN,     N_OPS))         * sc
        self.b2      = np.zeros(N_OPS)
        # trajectory attention
        self.W_attn  = np_rng.standard_normal((D_HIDDEN_ATTN, D_TAU))        * sc
        self.b_attn  = np.zeros(D_HIDDEN_ATTN)
        self.v_attn  = np_rng.standard_normal(D_HIDDEN_ATTN)                 * sc
        # prediction from pooled trajectory
        self.W_pred  = np_rng.standard_normal((D_TAU,        VOCAB))         * sc
        self.b_pred  = np.zeros(VOCAB)

    _KEYS = ("W_emb", "W1", "b1", "W2", "b2",
             "W_attn", "b_attn", "v_attn", "W_pred", "b_pred")

    def zero_grad(self) -> dict:
        return {k: np.zeros_like(getattr(self, k)) for k in self._KEYS}

    def apply_update(self, grads: dict, lr: float, clip: float = 1.0) -> None:
        norm  = float(np.sqrt(sum((g ** 2).sum() for g in grads.values())))
        scale = min(1.0, clip / (norm + 1e-8))
        for k, g in grads.items():
            setattr(self, k, getattr(self, k) - lr * scale * g)


# -- forward -------------------------------------------------------------------

def forward_episode(
    params: Params,
    x0: int,
    D: int,
    py_rng: pyrand.Random,
    np_rng: np.random.Generator,
    temp: float,
    training: bool,
    constraint: str | None = None,
) -> dict:
    state         = warmup(py_rng)
    tokens        = [x0] + [py_rng.randint(0, VOCAB - 1) for _ in range(D - 1)]

    soft_tau_prev = tau_onehot(state).copy()
    cache:        list[dict]       = []
    op_seq:       list[int]        = []
    entropy_seq:  list[float]      = []
    soft_taus_seq: list[np.ndarray] = []   # D soft tau states (output of each step)

    for tok in tokens:
        tau_nexts = np.stack([tau_onehot(OP_FNS[i](state)) for i in range(N_OPS)])

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

        soft_tau_curr = w @ tau_nexts
        entropy_seq.append(_route_entropy(w))
        soft_taus_seq.append(soft_tau_curr.copy())

        k = int(np.argmax(w))
        op_seq.append(k)

        # only cache router internals for unconstrained (trainable) routing
        if training and constraint is None:
            cache.append({
                "tok":       tok,
                "h_in":      h_in.copy(),
                "h":         h.copy(),
                "w":         w.copy(),
                "tau_nexts": tau_nexts.copy(),
            })

        state         = OP_FNS[k](state)
        soft_tau_prev = soft_tau_curr

    # -- trajectory attention pooling ------------------------------------------
    soft_taus_mat = np.stack(soft_taus_seq)                           # (D, D_TAU)
    h_attn_mat    = np.tanh(soft_taus_mat @ params.W_attn.T
                            + params.b_attn)                          # (D, D_HIDDEN_ATTN)
    a_scores      = h_attn_mat @ params.v_attn                        # (D,)
    alpha         = _softmax(a_scores)                                # (D,)
    pooled        = alpha @ soft_taus_mat                             # (D_TAU,)

    pred_logits   = pooled @ params.W_pred + params.b_pred            # (VOCAB,)
    pred          = int(np.argmax(pred_logits))
    loss          = float(-np.log(_softmax(pred_logits)[x0] + 1e-12))

    return {
        "loss":          loss,
        "correct":       (pred == x0),
        "cache":         cache,
        "soft_taus_mat": soft_taus_mat,    # (D, D_TAU) — keep reference
        "h_attn_mat":    h_attn_mat,       # (D, D_HIDDEN_ATTN)
        "alpha":         alpha,            # (D,) — attention weights
        "pooled":        pooled,           # (D_TAU,)
        "pred_logits":   pred_logits,      # (VOCAB,)
        "op_seq":        op_seq,
        "entropy_seq":   entropy_seq,
    }


# -- backward ------------------------------------------------------------------

def backward_episode(params: Params, ep: dict, x0: int, temp: float) -> dict:
    grads = params.zero_grad()

    # 1. Prediction head  (pooled @ W_pred + b_pred = pred_logits)
    d_pred          = _softmax(ep["pred_logits"]).copy()
    d_pred[x0]     -= 1.0                                            # (VOCAB,)
    grads["W_pred"] += np.outer(ep["pooled"], d_pred)                # (D_TAU, VOCAB)
    grads["b_pred"] += d_pred                                        # (VOCAB,)
    d_pooled         = params.W_pred @ d_pred                        # (D_TAU,)

    # 2. Attention backward
    #    pooled   = alpha @ soft_taus_mat
    #    alpha    = softmax(a_scores)
    #    a_scores = h_attn_mat @ v_attn
    #    h_attn_t = tanh(W_attn @ soft_tau_t + b_attn)

    alpha         = ep["alpha"]                                       # (D,)
    h_attn_mat    = ep["h_attn_mat"]                                  # (D, D_HIDDEN_ATTN)
    soft_taus_mat = ep["soft_taus_mat"]                               # (D, D_TAU)

    # d_pooled -> d_alpha and d_soft_taus
    d_alpha        = soft_taus_mat @ d_pooled                         # (D,)
    d_st_from_pool = np.outer(alpha, d_pooled)                        # (D, D_TAU)

    # softmax VJP: d_a_pre = alpha * (d_alpha - <alpha, d_alpha>)
    d_a_pre = alpha * (d_alpha - float(np.dot(alpha, d_alpha)))       # (D,)

    # a = h_attn_mat @ v_attn  ->  d_h_attn_mat[t] = v_attn * d_a_pre[t]
    d_h_attn_mat    = np.outer(d_a_pre, params.v_attn)                # (D, D_HIDDEN_ATTN)
    grads["v_attn"] += d_a_pre @ h_attn_mat                          # (D_HIDDEN_ATTN,)

    # h_attn = tanh(soft_taus_mat @ W_attn.T + b_attn)
    d_pre_mat       = d_h_attn_mat * (1.0 - h_attn_mat ** 2)         # (D, D_HIDDEN_ATTN)
    grads["W_attn"] += d_pre_mat.T @ soft_taus_mat                   # (D_HIDDEN_ATTN, D_TAU)
    grads["b_attn"] += d_pre_mat.sum(axis=0)                         # (D_HIDDEN_ATTN,)
    d_st_from_attn  = d_pre_mat @ params.W_attn                      # (D, D_TAU)

    # Combined gradient for each soft_tau_t from attention
    d_st_total = d_st_from_pool + d_st_from_attn                     # (D, D_TAU)

    # 3. Router chain backward (only when cache is non-empty)
    #    For step t: soft_tau_t = w_t @ tau_nexts_t
    #                h_in_t     = concat(emb[tok_t], soft_tau_{t-1})
    #    Recurrent: d_soft_tau_{t-1} receives d_h_in_t[D_EMB:]
    #    Each step also receives d_st_total[t] from the attention backward.

    d_soft_tau_recurrent = np.zeros(D_TAU)

    for t in range(len(ep["cache"]) - 1, -1, -1):
        step = ep["cache"][t]
        tok       = step["tok"]
        h_in      = step["h_in"]
        h         = step["h"]
        w         = step["w"]
        tau_nexts = step["tau_nexts"]

        # combined gradient for the soft tau OUTPUT at step t
        d_soft_tau_t = d_st_total[t] + d_soft_tau_recurrent          # (D_TAU,)

        # soft_tau_t = w_t @ tau_nexts_t  ->  d_w_t = tau_nexts_t @ d_soft_tau_t
        d_w      = tau_nexts @ d_soft_tau_t                           # (N_OPS,)
        d_logits = gumbel_softmax_vjp(w, d_w, temp)                   # (N_OPS,)

        grads["W2"] += np.outer(h, d_logits)                         # (D_HIDDEN, N_OPS)
        grads["b2"] += d_logits                                       # (N_OPS,)
        d_h    = params.W2 @ d_logits                                 # (D_HIDDEN,)
        d_hpre = d_h * (1.0 - h * h)                                 # (D_HIDDEN,)
        grads["W1"] += np.outer(h_in, d_hpre)                        # (D_IN, D_HIDDEN)
        grads["b1"] += d_hpre                                         # (D_HIDDEN,)
        d_hin   = params.W1 @ d_hpre                                  # (D_IN,)

        grads["W_emb"][tok % VOCAB] += d_hin[:D_EMB]
        d_soft_tau_recurrent = d_hin[D_EMB:]                         # flows to step t-1

    return grads


# -- training ------------------------------------------------------------------

def train(D: int, np_rng: np.random.Generator, py_rng: pyrand.Random) -> Params:
    params = Params(np_rng)
    for batch_idx in range(N_BATCHES):
        frac  = batch_idx / max(N_BATCHES - 1, 1)
        temp  = TEMP_START * (TEMP_END / TEMP_START) ** frac
        acc_g = params.zero_grad()

        for _ in range(BATCH_SIZE):
            x0    = py_rng.randint(0, VOCAB - 1)
            ep    = forward_episode(params, x0, D, py_rng, np_rng,
                                    temp=temp, training=True, constraint=None)
            grads = backward_episode(params, ep, x0, temp)
            for k in acc_g:
                acc_g[k] += grads[k] / BATCH_SIZE

        params.apply_update(acc_g, LR)
    return params


# -- evaluation ----------------------------------------------------------------

def evaluate(
    params: Params,
    D: int,
    py_rng: pyrand.Random,
    np_rng: np.random.Generator,
    constraint: str | None = None,
) -> dict:
    correct_total     = 0
    op_counts:        Counter               = Counter()
    transport_by_step: dict[int, list[int]] = defaultdict(list)
    entropy_vals:     list[float]           = []
    alpha_sum:        np.ndarray            = np.zeros(D)
    alpha_count:      int                   = 0

    for _ in range(N_EVAL):
        x0 = py_rng.randint(0, VOCAB - 1)
        ep = forward_episode(params, x0, D, py_rng, np_rng,
                             temp=TEMP_END, training=False, constraint=constraint)
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

    mean_alpha = alpha_sum / max(alpha_count, 1)   # mean attention weight per step

    return {
        "accuracy":          correct_total / N_EVAL,
        "op_counts":         dict(op_counts),
        "transport_frac":    tr_frac,
        "mean_entropy":      float(np.mean(entropy_vals)) if entropy_vals else 0.0,
        "transport_by_step": {s: float(np.mean(v))
                              for s, v in transport_by_step.items()},
        "mean_alpha":        mean_alpha.tolist(),   # list of D floats
    }


# -- run all -------------------------------------------------------------------

MODEL_CONFIGS = [
    ("traj_router",  None),
    ("fixed_Tx",     "fixed_Tx"),
    ("random",       "random"),
    ("non_transport","non_transport"),
]


def run_all() -> dict:
    t0     = time.perf_counter()
    np_rng = np.random.default_rng(GLOBAL_SEED)
    py_rng = pyrand.Random(GLOBAL_SEED)
    results: dict = {}
    trained: dict[int, Params] = {}

    for D in DELAYS:
        print(f"\n=== Delay D={D} ===")
        print(f"  Training traj_router ({N_BATCHES}×{BATCH_SIZE}={N_BATCHES*BATCH_SIZE} eps)...")
        trained[D] = train(D, np_rng, py_rng)
        print(f"  Done.")

        for model_name, constraint in MODEL_CONFIGS:
            p   = trained[D] if model_name == "traj_router" else Params(np_rng)
            res = evaluate(p, D, py_rng, np_rng, constraint=constraint)
            results[(model_name, D)] = res
            v2k  = "gumbel_router" if model_name == "traj_router" else model_name
            v2a  = V2_ACCS.get((v2k, D))
            v2s  = f"  (v2:{v2a:.3f})" if v2a else ""
            alpha_str = (
                f"  alpha=[{','.join(f'{a:.2f}' for a in res['mean_alpha'][:4])}...]"
                if model_name == "traj_router" else ""
            )
            print(
                f"  {model_name:<15}  acc={res['accuracy']:.3f}{v2s}  "
                f"tr={res['transport_frac']:.3f}  H={res['mean_entropy']:.3f}"
                f"{alpha_str}"
            )

    print(f"\nTotal: {time.perf_counter()-t0:.1f}s")
    return results


# -- write CSV -----------------------------------------------------------------

def write_csv(all_results: dict) -> None:
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "model", "task", "delay", "metric_name", "metric_value",
        "operator", "usage_fraction", "transport_usage_fraction",
        "route_entropy", "note",
    ]
    rows = []
    for (model_name, D), res in sorted(all_results.items(),
                                       key=lambda x: (x[0][0], x[0][1])):
        n_total = sum(res["op_counts"].values()) or 1
        task    = f"first_token_recall_D{D}"
        for op_name in OP_NAMES:
            usage = res["op_counts"].get(op_name, 0) / n_total
            rows.append({
                "model":                    model_name,
                "task":                     task,
                "delay":                    D,
                "metric_name":              "accuracy",
                "metric_value":             round(res["accuracy"], 4),
                "operator":                 op_name,
                "usage_fraction":           round(usage, 4),
                "transport_usage_fraction": round(res["transport_frac"], 4),
                "route_entropy":            round(res["mean_entropy"], 4),
                "note": f"cluster={OP_CLUSTERS[OP_NAMES.index(op_name)]}",
            })
        for step, alpha_t in enumerate(res["mean_alpha"]):
            rows.append({
                "model":                    model_name,
                "task":                     task,
                "delay":                    D,
                "metric_name":              "mean_attn_weight",
                "metric_value":             round(alpha_t, 4),
                "operator":                 "aggregate_attn",
                "usage_fraction":           round(alpha_t, 4),
                "transport_usage_fraction": round(res["transport_frac"], 4),
                "route_entropy":            round(res["mean_entropy"], 4),
                "note":                     f"step={step}; dist_from_query={D - step}",
            })
    with CSV_PATH.open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"[v3] Wrote {CSV_PATH}")


# -- write markdown ------------------------------------------------------------

def write_md(all_results: dict) -> None:
    DOCS_DIR.mkdir(parents=True, exist_ok=True)
    chance = 1.0 / VOCAB

    def r(model, D):
        return all_results.get((model, D), {})

    lines: list[str] = []

    lines += [
        "# Prime Transport — Router Reintegration Experiment v3",
        "",
        "**Experiment type:** Trajectory-level attention readout (Gumbel-softmax routing)",
        "**Operator layer:** `geometry_native_operator_model_v25`",
        "**No surface build.  No files modified.  No operators rebuilt.**",
        "",
        "## What Changed from v2",
        "",
        "| Aspect | v2 | v3 |",
        "|--------|----|----|",
        "| Readout | Terminal-state linear head (W_pred @ soft_tau_{D-1}) "
        "| Attention-weighted pooling over all D soft tau states |",
        "| Attention | None | MLP: h_t=tanh(W_attn@st_t+b_attn), a_t=v@h_t, alpha=softmax(a) |",
        "| Gradient to routing | From terminal state only | From all D tau states |",
        "| Routing mechanism | Gumbel-softmax (same) | Gumbel-softmax (same) |",
        "| Task / delays | Same | Same |",
        "| Training budget | Same | Same |",
        "",
        "**Why trajectory-level readout is the right fix for v2:**",
        "In v2, all gradient signal for routing decisions at steps 0..D-2 "
        "flowed only through the FINAL soft tau state. If the token-identity "
        "signal is distributed across the tau trajectory (path-encoded), the "
        "terminal readout cannot recover it, and the gradients for early routing "
        "decisions are vanishingly small. The attention readout assigns a learned "
        "weight alpha_t to each step's soft tau. If the encoding step (step 0) "
        "carries discriminative signal about x0, the model can learn alpha_0 >> "
        "alpha_{D-1}, directly recovering the early-step encoding.",
        "",
        "## Training and Evaluation Setup",
        "",
        f"- Task: first-token recall, VOCAB={VOCAB}, delays D in {DELAYS}",
        f"- Training: {N_BATCHES} batches x {BATCH_SIZE} = {N_BATCHES*BATCH_SIZE} episodes/delay",
        f"- Evaluation: {N_EVAL} episodes per (model, delay)",
        f"- Temperature: {TEMP_START} -> {TEMP_END} (exponential annealing)",
        f"- Optimizer: SGD + gradient clipping (clip=1.0, lr={LR})",
        f"- Attention: D_HIDDEN_ATTN={D_HIDDEN_ATTN}; position-free (tau state only)",
        "",
        "## Primary Results",
        "",
        "### Accuracy by model and delay",
        "",
        f"| Model | D=2 | D=4 | D=8 | D=16 | best delta vs chance ({chance:.3f}) |",
        f"|-------|-----|-----|-----|------|--------------------------------------|",
    ]

    for model_name, _ in MODEL_CONFIGS:
        accs = [r(model_name, D).get("accuracy", 0.0) for D in DELAYS]
        best = max(accs)
        sign = "+" if best >= chance else ""
        lines.append(
            f"| {model_name} | "
            + " | ".join(f"{a:.3f}" for a in accs)
            + f" | {sign}{best - chance:.3f} |"
        )
    lines.append("")

    lines += [
        "### Direct comparison: v2 gumbel_router vs v3 traj_router",
        "",
        "| D | v2 gumbel_router | v3 traj_router | delta |",
        "|---|------------------|----------------|-------|",
    ]
    for D in DELAYS:
        v2a = V2_ACCS.get(("gumbel_router", D), 0.0)
        v3a = r("traj_router", D).get("accuracy", 0.0)
        d   = v3a - v2a
        sign = "+" if d >= 0 else ""
        lines.append(f"| {D} | {v2a:.3f} | {v3a:.3f} | {sign}{d:.3f} |")

    v3_accs = [r("traj_router", D).get("accuracy", 0.0) for D in DELAYS]
    v2_accs = [V2_ACCS.get(("gumbel_router", D), 0.0) for D in DELAYS]
    beats_v2     = sum(1 for a, b in zip(v3_accs, v2_accs) if a > b)
    beats_chance = sum(1 for a in v3_accs if a > chance)
    lines += [
        "",
        f"v3 traj_router beats v2 gumbel_router at {beats_v2}/{len(DELAYS)} delays.",
        f"v3 traj_router exceeds chance at {beats_chance}/{len(DELAYS)} delays.",
        "",
    ]

    # ── attention analysis ──
    lines += [
        "## Attention Analysis",
        "",
        "### Mean attention weights by step position (traj_router only)",
        "",
        "Uniform attention = 1/D.  Values > uniform indicate the model attends to that step.",
        "",
    ]
    header  = "| D | " + " | ".join(f"step {s}" for s in range(max(DELAYS))) + " |"
    divider = "|---|" + "|".join("---" for _ in range(max(DELAYS))) + "|"
    lines  += [header, divider]
    for D in DELAYS:
        alphas = r("traj_router", D).get("mean_alpha", [1.0 / D] * D)
        uniform = 1.0 / D
        cells   = []
        for s in range(max(DELAYS)):
            if s < D:
                a   = alphas[s]
                rel = a / uniform
                cells.append(f"{a:.3f}({'↑' if rel > 1.05 else '↓' if rel < 0.95 else '~'})")
            else:
                cells.append("—")
        lines.append(f"| {D} | " + " | ".join(cells) + " |")
    lines.append("")

    # Interpret attention
    enc_dominated = []
    for D in DELAYS:
        alphas  = r("traj_router", D).get("mean_alpha", [1.0 / D] * D)
        if len(alphas) >= 2:
            enc_dominated.append(alphas[0] > alphas[-1] * 1.1)
        else:
            enc_dominated.append(False)
    n_enc = sum(enc_dominated)
    if n_enc >= 3:
        lines.append(
            f"**Encoding step (step 0) receives above-maintenance attention "
            f"at {n_enc}/{len(DELAYS)} delays**, consistent with path-encoding hypothesis."
        )
    else:
        lines.append(
            f"Attention weights do not consistently upweight the encoding step "
            f"({n_enc}/{len(DELAYS)} delays). Attention appears nearly uniform or "
            f"does not favour step 0."
        )
    lines.append("")

    # ── operator usage ──
    lines += [
        "## Operator Usage",
        "",
        "### Transport fraction by model and delay",
        "",
        "| Model | D=2 | D=4 | D=8 | D=16 |",
        "|-------|-----|-----|-----|------|",
    ]
    for model_name, _ in MODEL_CONFIGS:
        fracs = [f"{r(model_name, D).get('transport_frac', 0):.3f}" for D in DELAYS]
        lines.append(f"| {model_name} | {' | '.join(fracs)} |")
    lines.append("")

    lines += [
        "### traj_router: per-operator usage fraction",
        "",
        "| Operator | Cluster | D=2 | D=4 | D=8 | D=16 |",
        "|----------|---------|-----|-----|-----|------|",
    ]
    for op_name in OP_NAMES:
        cluster = OP_CLUSTERS[OP_NAMES.index(op_name)]
        fracs   = []
        for D in DELAYS:
            res   = r("traj_router", D)
            n_tot = sum(res.get("op_counts", {}).values()) or 1
            cnt   = res.get("op_counts", {}).get(op_name, 0)
            fracs.append(f"{cnt/n_tot:.3f}")
        lines.append(f"| {op_name} | {cluster} | {' | '.join(fracs)} |")
    lines.append("")

    # ── routing collapse check ──
    max_ent  = float(np.log(N_OPS))
    tr_entrs = [r("traj_router", D).get("mean_entropy", 0.0) for D in DELAYS]
    lines += [
        "### Routing collapse check",
        "",
        f"Max entropy (uniform over {N_OPS} ops): {max_ent:.3f} bits",
        "",
        "| D | entropy | collapsed (<0.3)? | mixed (>60% max)? |",
        "|---|---------|-------------------|-------------------|",
    ]
    for D, e in zip(DELAYS, tr_entrs):
        col = e < 0.3
        mix = e > max_ent * 0.6
        lines.append(
            f"| {D} | {e:.3f} | {'yes' if col else 'no'} | {'yes' if mix else 'no'} |"
        )
    lines.append("")

    # ── transport/non-transport distinction ──
    enc_fracs, maint_fracs = [], []
    for D in DELAYS:
        tbys       = r("traj_router", D).get("transport_by_step", {})
        enc        = tbys.get(0, 0.0)
        maint_list = [v for s, v in tbys.items() if s > 0]
        maint      = float(np.mean(maint_list)) if maint_list else enc
        enc_fracs.append(enc)
        maint_fracs.append(maint)

    lines += [
        "## Transport / Non-Transport Distinction",
        "",
        "| D | transport @ step 0 (encoding) | transport @ steps 1+ (maint.) | diff |",
        "|---|-------------------------------|-------------------------------|------|",
    ]
    for D, enc, maint in zip(DELAYS, enc_fracs, maint_fracs):
        lines.append(f"| {D} | {enc:.3f} | {maint:.3f} | {enc - maint:+.3f} |")
    lines.append("")
    me, mm = float(np.mean(enc_fracs)), float(np.mean(maint_fracs))
    if me > mm + 0.04:
        lines.append(
            f"**Router uses more transport at the encoding step** ({me:.3f}) "
            f"than during maintenance ({mm:.3f}), consistent with design hypothesis."
        )
    else:
        lines.append(
            f"Encoding-step transport ({me:.3f}) and maintenance transport ({mm:.3f}) "
            f"are not strongly differentiated."
        )
    lines.append("")

    # ── honesty ──
    lines += [
        "## Honesty Section",
        "",
        "### What improved",
        "",
        f"- Trajectory readout gradient flows to all D routing steps, "
        f"not just the final one.",
        f"- The model can now in principle detect path-encoded signal in early tau states.",
        f"- Routing remained mixed (entropy > 60% max) at "
        + str(sum(1 for e in tr_entrs if e > max_ent * 0.6))
        + f"/{len(DELAYS)} delays.",
        f"- v3 traj_router beats v2 gumbel_router at {beats_v2}/{len(DELAYS)} delays.",
        "",
        "### What failed or is limited",
        "",
    ]
    if max(v3_accs) <= chance + 0.02:
        lines += [
            f"- Accuracy remained near chance (best {max(v3_accs):.3f}, chance {chance:.3f}).",
            "- The trajectory readout and attention mechanism did not unlock significant "
            "task performance improvement.",
            "- This suggests that the tau trajectory may not carry discriminative "
            "token-identity information under the current first-token recall task setup.",
            "- The operator layer applies deterministic geometric transformations to a "
            "shared state. Two different tokens x0=0 and x0=1, starting from the same "
            "warmup state, can only be distinguished by the routing weights w_0. However, "
            "the soft tau state soft_tau_0 = w_0 @ tau_nexts_0 may not reliably encode "
            "which token was seen, because tau_nexts_0 is fixed (depends only on hard state) "
            "and w_0 variation across tokens may be too small relative to noise.",
        ]
    else:
        lines += [
            f"- Best accuracy: {max(v3_accs):.3f} (chance: {chance:.3f}).",
            "- Some improvement over v2, but still modest.",
        ]
    lines += [
        "",
        "### What remains uncertain",
        "",
        "- Whether the fundamental limitation is insufficient token-to-tau pathway "
        "capacity, or insufficient training (800 batches may be too few).",
        "- Whether a modified task where the token directly gates operator application "
        "(rather than routing being the only token-sensitive element) would succeed.",
        "- Whether full exact spin_H is solved: No.",
        "",
        "## Recommended Next Step",
        "",
        "Modify the integration interface: instead of routing operator selection solely "
        "on the token embedding, **concatenate the token directly into the operator "
        "state transformation** (inject x0 as a bias into the tau update at each step, "
        "bypassing the routing bottleneck). This tests whether the operator layer can "
        "encode token identity when given a direct token-injection signal, rather than "
        "relying entirely on the routing MLP to differentiate tokens through soft operator "
        "selection from a shared geometric state.",
        "",
    ]

    with MD_PATH.open("w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")
    print(f"[v3] Wrote {MD_PATH}")


# -- main ----------------------------------------------------------------------

def main():
    print("=== Prime Transport Router Reintegration Experiment v3 ===")
    print("Trajectory-level attention readout (Gumbel-softmax routing)")
    print(f"Task: first-token recall, VOCAB={VOCAB}, DELAYS={DELAYS}")
    print(f"Training: {N_BATCHES}×{BATCH_SIZE}={N_BATCHES*BATCH_SIZE} eps/delay")
    print()
    results = run_all()
    write_csv(results)
    write_md(results)
    print("[v3] Experiment complete.")


if __name__ == "__main__":
    main()
