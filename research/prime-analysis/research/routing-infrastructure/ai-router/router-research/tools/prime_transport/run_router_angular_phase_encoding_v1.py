#!/usr/bin/env python3
"""run_router_angular_phase_encoding_v1.py

Angular Phase Encoding experiment: direct test of Training-Rule Alignment
Analysis v1, mismatch M1 (one-hot destroys cyclic adjacency).

Replaces the 21-dim one-hot tau encoding with 8-dim angular (cos/sin)
encoding.  Each cyclic phase k in Z/m becomes (cos(2*pi*k/m), sin(2*pi*k/m)).

Head-to-head comparison:
  BASELINE — current one-hot tau (D_TAU=21, D_IN=25)
  ANGULAR  — cos/sin angular tau  (D_TAU=8,  D_IN=12)

Everything else is identical: same operators, same task, same D=24, same
training setup, same seeds, same temperature schedule, same gradient clipping.
"""
from __future__ import annotations

import csv
import importlib.util
import math
import time
from pathlib import Path
from typing import Dict, List, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F

# ============================================================================
# Paths
# ============================================================================
SCRIPT_DIR  = Path(__file__).parent
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CSV_OUT     = RESULTS_DIR / "prime_transport_router_angular_phase_encoding_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_angular_phase_encoding_v1.md"

RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ============================================================================
# Locked canonical v6 hyperparameters
# ============================================================================
VOCAB       = 4
D_EMB       = 4
N_OPS       = 6
LR          = 0.02
TEMP_START  = 2.0
TEMP_END    = 0.1
GLOBAL_SEED = 42
B0_INIT     = 2.0
D_CONTEXT   = 24
BATCH_SIZE  = 256
D_HIDDEN    = 32
BENCH_BUDGET = 3000
N_EVAL       = 1000

# One-hot encoding dimensions
D_TAU_ONEHOT = 21       # 2 + 5 + 2 + 12
D_IN_ONEHOT  = D_EMB + D_TAU_ONEHOT  # 25

# Angular encoding dimensions
D_TAU_ANG    = 8        # 4 phase pairs × 2 (cos, sin)
D_IN_ANG     = D_EMB + D_TAU_ANG     # 12

# Phase block structure: (start_in_onehot, end_in_onehot, modulus)
PHASE_BLOCKS = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]

_TRANSPORT_THRESHOLD = 3

# Checkpoint schedule for convergence comparison
CHECKPOINTS = [0, 250, 500, 1000, 1500, 2000, 2500, 3000]

torch.set_num_threads(1)

# ============================================================================
# Load v6_torch
# ============================================================================
def _load_v6torch():
    spec = importlib.util.spec_from_file_location(
        "v6torch_base",
        str(SCRIPT_DIR / "run_router_reintegration_v6_torch.py"),
    )
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# ============================================================================
# Angular encoding conversion
# ============================================================================
def convert_onehot_to_angular(onehot: torch.Tensor) -> torch.Tensor:
    """Convert (..., 21) one-hot tau → (..., 8) angular (cos/sin pairs).

    For each phase block of modulus m with one-hot index k:
        output = (cos(2*pi*k/m), sin(2*pi*k/m))

    This preserves cyclic adjacency: neighboring phases on Z/m
    map to nearby points on the unit circle S^1.
    """
    shape = onehot.shape[:-1]
    out = torch.zeros(*shape, D_TAU_ANG, dtype=onehot.dtype)
    ang_idx = 0
    for start, end, m in PHASE_BLOCKS:
        block = onehot[..., start:end]           # (..., m)
        k = block.argmax(dim=-1).float()          # (...,)
        angle = 2.0 * math.pi * k / float(m)
        out[..., ang_idx]     = torch.cos(angle)
        out[..., ang_idx + 1] = torch.sin(angle)
        ang_idx += 2
    return out


# ============================================================================
# BASELINE model: one-hot tau (identical to RouterV6Scaled)
# ============================================================================
class RouterBaseline(nn.Module):
    """One-hot tau baseline.  Identical to RouterV6Scaled."""

    def __init__(
        self,
        TN: torch.Tensor,           # (N, N_OPS, 21)
        TR: torch.Tensor,           # (N, N_OPS)
        tau0_table: torch.Tensor,   # (N, 21)
        pool_ids: torch.Tensor,
        d_hidden: int = D_HIDDEN,
        d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT,
        init_scale: float = 0.05,
        seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh  = d_hidden
        dha = max(8, d_hidden // 4)
        d_tau = 21
        d_in  = D_EMB + d_tau

        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_table)
        self.register_buffer("pool_ids", pool_ids)
        pos0_mask = torch.zeros(1, d_context)
        pos0_mask[0, 0] = 1.0
        self.register_buffer("pos0_mask", pos0_mask)

        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        gen = torch.Generator().manual_seed(seed)

        def rp(*shape: int) -> nn.Parameter:
            return nn.Parameter(
                torch.empty(*shape).normal_(0.0, init_scale, generator=gen))

        def zp(*shape: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*shape))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(d_in, dh)
        self.b1           = zp(dh)
        self.W2           = rp(dh, N_OPS)
        self.b2           = zp(N_OPS)
        self.W_attn       = rp(dha, d_tau)
        self.b_attn       = zp(dha)
        self.v_attn       = rp(dha)
        self.W_pred       = rp(d_tau, VOCAB)
        self.b_pred       = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

    def forward(
        self,
        state_ids: torch.Tensor,
        tokens: torch.Tensor,
        x0: torch.Tensor,
        temp: float,
    ) -> torch.Tensor:
        B: int = state_ids.shape[0]
        D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []

        for t in range(D):
            tn_batch = self.TN[state_ids]
            tok_t    = tokens[:, t]
            embs     = self.W_emb[tok_t]
            h_in     = torch.cat([embs, tau_prev], dim=1)
            h        = torch.tanh(h_in @ self.W1 + self.b1)
            logits   = h @ self.W2 + self.b2

            if self.training:
                u  = torch.rand_like(logits).clamp_(1e-20, 1.0)
                gn = -torch.log(-torch.log(u))
                w  = torch.softmax((logits + gn) / temp, dim=1)
            else:
                w  = torch.softmax(logits / 0.05, dim=1)

            base     = torch.einsum("bi,bij->bj", w, tn_batch)
            tau_prev = (base + self.W_tok_inject[x0]) if t == 0 else base
            soft_taus.append(tau_prev)

            k_hard    = torch.argmax(w, dim=1)
            tr_rows   = self.TR[state_ids]
            state_ids = tr_rows.gather(1, k_hard.unsqueeze(1)).squeeze(1)

        soft_taus_stack = torch.stack(soft_taus, dim=1)
        h_attn       = torch.tanh(
            soft_taus_stack @ self.W_attn.t() + self.b_attn)
        a_scores_raw = (h_attn * self.v_attn).sum(dim=-1)
        a_scores     = a_scores_raw + self.pos0_mask * self.b_pos0
        alpha        = torch.softmax(a_scores, dim=1)
        pooled       = torch.einsum("bd,bdt->bt", alpha, soft_taus_stack)
        return pooled @ self.W_pred + self.b_pred


# ============================================================================
# ANGULAR model: cos/sin tau encoding
# ============================================================================
class RouterAngular(nn.Module):
    """Angular (cos/sin) tau encoding.  Same recurrence, same operators,
    but tau represented as 4 × (cos, sin) pairs = 8 dims instead of 21.

    The soft mixture of angular TN entries produces a circular mean —
    the geometrically correct averaging operation on S^1.
    """

    def __init__(
        self,
        TN: torch.Tensor,           # (N, N_OPS, 8)  angular-encoded
        TR: torch.Tensor,           # (N, N_OPS)
        tau0_table: torch.Tensor,   # (N, 8)  angular-encoded
        pool_ids: torch.Tensor,
        d_hidden: int = D_HIDDEN,
        d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT,
        init_scale: float = 0.05,
        seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh  = d_hidden
        dha = max(8, d_hidden // 4)
        d_tau = 8
        d_in  = D_EMB + d_tau

        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_table)
        self.register_buffer("pool_ids", pool_ids)
        pos0_mask = torch.zeros(1, d_context)
        pos0_mask[0, 0] = 1.0
        self.register_buffer("pos0_mask", pos0_mask)

        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        gen = torch.Generator().manual_seed(seed)

        def rp(*shape: int) -> nn.Parameter:
            return nn.Parameter(
                torch.empty(*shape).normal_(0.0, init_scale, generator=gen))

        def zp(*shape: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*shape))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(d_in, dh)
        self.b1           = zp(dh)
        self.W2           = rp(dh, N_OPS)
        self.b2           = zp(N_OPS)
        self.W_attn       = rp(dha, d_tau)
        self.b_attn       = zp(dha)
        self.v_attn       = rp(dha)
        self.W_pred       = rp(d_tau, VOCAB)
        self.b_pred       = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

    def forward(
        self,
        state_ids: torch.Tensor,
        tokens: torch.Tensor,
        x0: torch.Tensor,
        temp: float,
    ) -> torch.Tensor:
        B: int = state_ids.shape[0]
        D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []

        for t in range(D):
            tn_batch = self.TN[state_ids]
            tok_t    = tokens[:, t]
            embs     = self.W_emb[tok_t]
            h_in     = torch.cat([embs, tau_prev], dim=1)
            h        = torch.tanh(h_in @ self.W1 + self.b1)
            logits   = h @ self.W2 + self.b2

            if self.training:
                u  = torch.rand_like(logits).clamp_(1e-20, 1.0)
                gn = -torch.log(-torch.log(u))
                w  = torch.softmax((logits + gn) / temp, dim=1)
            else:
                w  = torch.softmax(logits / 0.05, dim=1)

            base     = torch.einsum("bi,bij->bj", w, tn_batch)
            tau_prev = (base + self.W_tok_inject[x0]) if t == 0 else base
            soft_taus.append(tau_prev)

            k_hard    = torch.argmax(w, dim=1)
            tr_rows   = self.TR[state_ids]
            state_ids = tr_rows.gather(1, k_hard.unsqueeze(1)).squeeze(1)

        soft_taus_stack = torch.stack(soft_taus, dim=1)
        h_attn       = torch.tanh(
            soft_taus_stack @ self.W_attn.t() + self.b_attn)
        a_scores_raw = (h_attn * self.v_attn).sum(dim=-1)
        a_scores     = a_scores_raw + self.pos0_mask * self.b_pos0
        alpha        = torch.softmax(a_scores, dim=1)
        pooled       = torch.einsum("bd,bdt->bt", alpha, soft_taus_stack)
        return pooled @ self.W_pred + self.b_pred


# ============================================================================
# Sampling
# ============================================================================
def sample_batch(
    pool_ids: torch.Tensor, D: int, B: int, device: str,
) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    idx       = torch.randint(0, len(pool_ids), (B,), device=device)
    state_ids = pool_ids[idx]
    tokens    = torch.randint(0, VOCAB, (B, D), device=device)
    x0        = torch.randint(0, VOCAB, (B,),   device=device)
    tokens[:, 0] = x0
    return state_ids, tokens, x0


# ============================================================================
# Evaluation (generic: reads d_tau from model)
# ============================================================================
def evaluate(
    model: nn.Module,
    pool_ids: torch.Tensor,
    D: int,
    n_eval: int,
    device: str,
) -> Dict[str, float]:
    model.eval()
    torch.manual_seed(GLOBAL_SEED + D + 13)

    d_tau       = model.tau0_table.shape[1]
    B_eval      = 256
    n_batches   = max(1, min(4, (n_eval + B_eval - 1) // B_eval))
    total       = 0
    correct     = 0
    ent_sum     = 0.0
    transport_n = 0
    total_steps = 0
    alpha_sum   = torch.zeros(D, device=device)
    grad_norm_sum = 0.0
    grad_norm_cnt = 0

    with torch.no_grad():
        for _ in range(n_batches):
            B_ = min(B_eval, n_eval - total)
            if B_ <= 0:
                break
            idx       = torch.randint(0, len(pool_ids), (B_,), device=device)
            state_ids = pool_ids[idx]
            tokens    = torch.randint(0, VOCAB, (B_, D), device=device)
            x0        = torch.randint(0, VOCAB, (B_,),   device=device)
            tokens[:, 0] = x0

            tau_prev  = model.tau0_table[state_ids]
            soft_taus: List[torch.Tensor] = []
            sids      = state_ids.clone()

            for t in range(D):
                tn_batch = model.TN[sids]
                embs     = model.W_emb[tokens[:, t]]
                h_in     = torch.cat([embs, tau_prev], dim=1)
                h        = torch.tanh(h_in @ model.W1 + model.b1)
                logits   = h @ model.W2 + model.b2
                w        = torch.softmax(logits / 0.05, dim=1)

                pc       = w.clamp(1e-12, 1.0)
                ent_sum += float((-(pc * pc.log()).sum(dim=1)).sum().item())

                k_hard       = w.argmax(dim=1)
                transport_n += int((k_hard >= _TRANSPORT_THRESHOLD).sum().item())
                total_steps += B_

                base     = torch.einsum("bi,bij->bj", w, tn_batch)
                tau_prev = (base + model.W_tok_inject[x0]) if t == 0 else base
                soft_taus.append(tau_prev)

                tr_rows  = model.TR[sids]
                sids     = tr_rows.gather(1, k_hard.unsqueeze(1)).squeeze(1)

            soft_taus_stack = torch.stack(soft_taus, dim=1)
            h_attn   = torch.tanh(
                soft_taus_stack @ model.W_attn.t() + model.b_attn)
            a_raw    = (h_attn * model.v_attn).sum(dim=-1)
            a_scores = a_raw + model.pos0_mask * model.b_pos0
            alpha    = torch.softmax(a_scores, dim=1)
            alpha_sum += alpha.sum(dim=0)

            pred    = (torch.einsum("bd,bdt->bt", alpha, soft_taus_stack)
                       @ model.W_pred + model.b_pred)
            correct += int((pred.argmax(1) == x0).sum().item())
            total   += B_

    model.train()
    return {
        "accuracy":       correct / max(total, 1),
        "route_entropy":  ent_sum / max(total_steps, 1),
        "transport_frac": transport_n / max(total_steps, 1),
        "alpha0":         float((alpha_sum[0] / max(total, 1)).item()),
        "b_pos0":         float(model.b_pos0.item()),
    }


# ============================================================================
# Training + checkpoint evaluation
# ============================================================================
def train_and_evaluate(
    variant_name: str,
    model: nn.Module,
    pool_ids: torch.Tensor,
    device: str,
) -> Tuple[List[dict], float]:
    """Train model for BENCH_BUDGET steps, evaluating at each checkpoint.
    Returns (list_of_checkpoint_rows, total_runtime_seconds).
    """
    try:
        scripted = torch.jit.script(model)
        jit_ok = True
    except Exception as e:
        print(f"    JIT failed ({e}), using eager", flush=True)
        scripted = model
        jit_ok = False

    scripted.train()
    optimizer = torch.optim.SGD(scripted.parameters(), lr=LR)
    torch.manual_seed(GLOBAL_SEED)

    n_params = sum(p.numel() for p in model.parameters())
    d_tau    = model.tau0_table.shape[1]
    d_in     = D_EMB + d_tau

    ckpt_set   = set(CHECKPOINTS)
    rows: List[dict] = []

    # Evaluate at step 0 (before training)
    if 0 in ckpt_set:
        # Sync for scripted
        sd = {k: v for k, v in scripted.state_dict().items()
              if k not in {"TN", "TR", "tau0_table", "pool_ids", "pos0_mask"}}
        model.load_state_dict(sd, strict=False)
        metrics = evaluate(model, pool_ids, D_CONTEXT, N_EVAL, device)
        rows.append({
            "variant": variant_name,
            "checkpoint": 0,
            "accuracy": round(metrics["accuracy"], 4),
            "alpha0": round(metrics["alpha0"], 4),
            "route_entropy": round(metrics["route_entropy"], 4),
            "transport_frac": round(metrics["transport_frac"], 4),
            "b_pos0": round(metrics["b_pos0"], 3),
            "runtime_seconds": 0.0,
            "loss": -1.0,
            "d_tau": d_tau,
            "d_in": d_in,
            "n_params": n_params,
            "jit": "yes" if jit_ok else "no",
            "note": "pre-training",
        })
        scripted.train()

    loss_val = -1.0
    t0 = time.perf_counter()

    for step in range(BENCH_BUDGET):
        frac = step / max(BENCH_BUDGET - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)

        sids, toks, x0 = sample_batch(pool_ids, D_CONTEXT, BATCH_SIZE, device)
        pred = scripted(sids, toks, x0, temp)
        loss = F.cross_entropy(pred, x0)
        loss_val = float(loss.item())

        optimizer.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(scripted.parameters(), max_norm=1.0)
        optimizer.step()

        step_num = step + 1
        if step_num in ckpt_set:
            elapsed = time.perf_counter() - t0
            sd = {k: v for k, v in scripted.state_dict().items()
                  if k not in {"TN", "TR", "tau0_table", "pool_ids", "pos0_mask"}}
            model.load_state_dict(sd, strict=False)
            metrics = evaluate(model, pool_ids, D_CONTEXT, N_EVAL, device)
            solved = "solved" if metrics["accuracy"] >= 0.90 else ""
            rows.append({
                "variant": variant_name,
                "checkpoint": step_num,
                "accuracy": round(metrics["accuracy"], 4),
                "alpha0": round(metrics["alpha0"], 4),
                "route_entropy": round(metrics["route_entropy"], 4),
                "transport_frac": round(metrics["transport_frac"], 4),
                "b_pos0": round(metrics["b_pos0"], 3),
                "runtime_seconds": round(elapsed, 2),
                "loss": round(loss_val, 4),
                "d_tau": d_tau,
                "d_in": d_in,
                "n_params": n_params,
                "jit": "yes" if jit_ok else "no",
                "note": solved,
            })
            print(f"    step {step_num:>5}: acc={metrics['accuracy']:.4f}  "
                  f"alpha0={metrics['alpha0']:.4f}  "
                  f"loss={loss_val:.4f}  "
                  f"ent={metrics['route_entropy']:.4f}  "
                  f"t={elapsed:.1f}s  {solved}", flush=True)
            scripted.train()

    runtime = time.perf_counter() - t0
    return rows, runtime


# ============================================================================
# Markdown writer
# ============================================================================
def write_md(baseline_rows: List[dict], angular_rows: List[dict],
             runtime_baseline: float, runtime_angular: float) -> None:
    lines: List[str] = []
    A = lines.append

    A("# Angular Phase Encoding — Experiment v1")
    A("")
    A("## Purpose")
    A("")
    A("Direct test of Training-Rule Alignment Analysis v1, mismatch M1:")
    A("one-hot tau encoding destroys cyclic adjacency of the discrete torus")
    A("T⁴ = Z/2 × Z/5 × Z/2 × Z/12.")
    A("")
    A("## Angular Encoding")
    A("")
    A("Each cyclic phase k ∈ Z/m is encoded as (cos(2πk/m), sin(2πk/m)).")
    A("")
    A("| Phase | Modulus | One-Hot Dims | Angular Dims |")
    A("|-------|---------|-------------|-------------|")
    A("| swap_phase | Z/2 | 2 | 2 (cos/sin) |")
    A("| coupled_phase | Z/5 | 5 | 2 (cos/sin) |")
    A("| twist_phase | Z/2 | 2 | 2 (cos/sin) |")
    A("| lift_phase | Z/12 | 12 | 2 (cos/sin) |")
    A("| **Total D_TAU** | | **21** | **8** |")
    A("| **D_IN (emb+tau)** | | **25** | **12** |")
    A("")
    A("**Key property**: In angular encoding, adjacent phases on Z/m are")
    A("adjacent on S¹ — including the wrap-around (e.g., lift_phase=11 and")
    A("lift_phase=0 are nearby). In one-hot encoding, ALL distinct phases are")
    A("equidistant (distance √2).")
    A("")
    A("**Soft mixture meaning**: Weighted average of angular TN entries produces a")
    A("**circular mean** — the geometrically correct averaging operation on S¹.")
    A("In one-hot encoding, the same operation produces a probability vector with")
    A("no geometric meaning on the torus.")
    A("")

    A("## Locked Configuration")
    A("")
    A("| Item | Value |")
    A("|------|-------|")
    A(f"| device | cpu |")
    A(f"| D_HIDDEN | {D_HIDDEN} |")
    A(f"| batch_size | {BATCH_SIZE} |")
    A(f"| D (delay) | {D_CONTEXT} |")
    A(f"| training budget | {BENCH_BUDGET} steps |")
    A(f"| optimizer | SGD, lr={LR} |")
    A(f"| temperature | {TEMP_START} → {TEMP_END} (exponential) |")
    A(f"| grad clip | 1.0 |")
    A(f"| pos0 bias init | {B0_INIT} |")
    A(f"| seed | {GLOBAL_SEED} |")
    A("")

    A("## Convergence Comparison")
    A("")
    A("| Step | Baseline Acc | Angular Acc | Baseline α₀ | Angular α₀ | Baseline Ent | Angular Ent |")
    A("|------|-------------|-------------|-------------|-------------|-------------|-------------|")

    bl_by_ckpt = {r["checkpoint"]: r for r in baseline_rows}
    an_by_ckpt = {r["checkpoint"]: r for r in angular_rows}
    all_ckpts = sorted(set(list(bl_by_ckpt.keys()) + list(an_by_ckpt.keys())))

    for ck in all_ckpts:
        bl = bl_by_ckpt.get(ck, {})
        an = an_by_ckpt.get(ck, {})
        ba = f"{bl.get('accuracy', -1):.4f}" if bl else "—"
        aa = f"{an.get('accuracy', -1):.4f}" if an else "—"
        b0b = f"{bl.get('alpha0', -1):.4f}" if bl else "—"
        b0a = f"{an.get('alpha0', -1):.4f}" if an else "—"
        beb = f"{bl.get('route_entropy', -1):.4f}" if bl else "—"
        bea = f"{an.get('route_entropy', -1):.4f}" if an else "—"
        A(f"| {ck} | {ba} | {aa} | {b0b} | {b0a} | {beb} | {bea} |")
    A("")

    # Final comparison
    bl_final = bl_by_ckpt.get(BENCH_BUDGET, {})
    an_final = an_by_ckpt.get(BENCH_BUDGET, {})
    bl_acc = bl_final.get("accuracy", -1)
    an_acc = an_final.get("accuracy", -1)
    bl_a0  = bl_final.get("alpha0", -1)
    an_a0  = an_final.get("alpha0", -1)
    bl_ent = bl_final.get("route_entropy", -1)
    an_ent = an_final.get("route_entropy", -1)
    bl_np  = bl_final.get("n_params", -1)
    an_np  = an_final.get("n_params", -1)

    A("## Final Summary")
    A("")
    A("| Metric | Baseline (one-hot) | Angular (cos/sin) | Δ |")
    A("|--------|-------------------|-------------------|---|")
    if bl_acc >= 0 and an_acc >= 0:
        A(f"| Final accuracy | {bl_acc:.4f} | {an_acc:.4f} | {an_acc - bl_acc:+.4f} |")
    if bl_a0 >= 0 and an_a0 >= 0:
        A(f"| Final α₀ | {bl_a0:.4f} | {an_a0:.4f} | {an_a0 - bl_a0:+.4f} |")
    if bl_ent >= 0 and an_ent >= 0:
        A(f"| Final route entropy | {bl_ent:.4f} | {an_ent:.4f} | {an_ent - bl_ent:+.4f} |")
    A(f"| Parameters | {bl_np} | {an_np} | {an_np - bl_np:+d} |")
    A(f"| D_TAU | {D_TAU_ONEHOT} | {D_TAU_ANG} | −{D_TAU_ONEHOT - D_TAU_ANG} |")
    A(f"| D_IN | {D_IN_ONEHOT} | {D_IN_ANG} | −{D_IN_ONEHOT - D_IN_ANG} |")
    A(f"| Training time (s) | {runtime_baseline:.1f} | {runtime_angular:.1f} | {runtime_angular - runtime_baseline:+.1f} |")
    sps_bl = BENCH_BUDGET / runtime_baseline if runtime_baseline > 0 else 0
    sps_an = BENCH_BUDGET / runtime_angular if runtime_angular > 0 else 0
    A(f"| Steps/sec | {sps_bl:.1f} | {sps_an:.1f} | {sps_an - sps_bl:+.1f} |")
    A("")

    # Determine which solved D=24
    bl_solved = bl_acc >= 0.90
    an_solved = an_acc >= 0.90

    # Find first checkpoint where each solved
    bl_first_solve = None
    an_first_solve = None
    for ck in all_ckpts:
        if bl_first_solve is None and bl_by_ckpt.get(ck, {}).get("accuracy", 0) >= 0.90:
            bl_first_solve = ck
        if an_first_solve is None and an_by_ckpt.get(ck, {}).get("accuracy", 0) >= 0.90:
            an_first_solve = ck

    A("## Key Questions")
    A("")
    A(f"**Did both solve D=24?** Baseline: {'YES' if bl_solved else 'NO'}, "
      f"Angular: {'YES' if an_solved else 'NO'}")
    A("")
    if bl_first_solve is not None or an_first_solve is not None:
        A(f"**First solve checkpoint:** Baseline: {bl_first_solve or 'never'}, "
          f"Angular: {an_first_solve or 'never'}")
        if bl_first_solve and an_first_solve:
            if an_first_solve < bl_first_solve:
                A(f"  → Angular converges **faster** ({bl_first_solve - an_first_solve} steps earlier)")
            elif an_first_solve > bl_first_solve:
                A(f"  → Baseline converges **faster** ({an_first_solve - bl_first_solve} steps earlier)")
            else:
                A("  → Both converge at the same checkpoint")
        A("")

    A(f"**Does angular encoding improve accuracy?** ", )
    if an_acc > bl_acc + 0.005:
        A(f"YES — +{an_acc - bl_acc:.4f}")
    elif an_acc < bl_acc - 0.005:
        A(f"NO — angular is worse by {bl_acc - an_acc:.4f}")
    else:
        A(f"MARGINAL — difference is {an_acc - bl_acc:+.4f} (within noise)")
    A("")

    A(f"**Does angular encoding change α₀?** ", )
    if an_a0 > bl_a0 + 0.01:
        A(f"YES — α₀ increased by {an_a0 - bl_a0:+.4f} (stronger pos-0 attention)")
    elif an_a0 < bl_a0 - 0.01:
        A(f"YES — α₀ decreased by {an_a0 - bl_a0:+.4f}")
    else:
        A(f"MARGINAL — difference is {an_a0 - bl_a0:+.4f}")
    A("")

    A(f"**Is angular faster at runtime?** ", )
    if sps_an > sps_bl * 1.05:
        A(f"YES — {sps_an/sps_bl:.2f}× faster ({sps_an:.1f} vs {sps_bl:.1f} sps)")
    elif sps_an < sps_bl * 0.95:
        A(f"NO — {sps_bl/sps_an:.2f}× slower")
    else:
        A(f"SIMILAR — {sps_an:.1f} vs {sps_bl:.1f} sps")
    A("")

    A("## Interpretation")
    A("")
    if an_acc >= bl_acc - 0.005 and sps_an > sps_bl * 0.95:
        if an_acc > bl_acc + 0.005:
            A("Angular encoding **improves learning** while maintaining or improving runtime.")
            A("This supports the hypothesis that one-hot encoding was fighting the torus geometry.")
            A("The MLP learns more effectively when cyclic adjacency is explicit in the coordinate system.")
        elif an_first_solve and bl_first_solve and an_first_solve < bl_first_solve:
            A("Angular encoding achieves **faster convergence** with comparable final accuracy.")
            A("This suggests that explicit cyclic adjacency helps early learning, even if the MLP")
            A("eventually recovers the torus structure from one-hot data given enough training.")
        else:
            A("Angular encoding achieves **comparable accuracy** with fewer parameters and dimensions.")
            A("The MLP is powerful enough to learn the cyclic structure from one-hot data at this scale.")
            A("The representation mismatch (M1) exists but does not materially limit learning at D=24.")
    else:
        A("Angular encoding did NOT improve over the one-hot baseline.")
        A("The MLP successfully learns the cyclic structure from one-hot data.")
        A("The representation mismatch (M1) is real but not the binding constraint at this scale.")
    A("")

    A("## Honesty Section")
    A("")
    A("### What Improved")
    A("")
    improvements = []
    if an_acc > bl_acc + 0.005:
        improvements.append(f"Final accuracy: +{an_acc - bl_acc:.4f}")
    if an_first_solve and bl_first_solve and an_first_solve < bl_first_solve:
        improvements.append(f"Convergence speed: solved {bl_first_solve - an_first_solve} steps earlier")
    if sps_an > sps_bl * 1.05:
        improvements.append(f"Runtime: {sps_an/sps_bl:.2f}× faster (smaller matrices)")
    if an_np < bl_np:
        improvements.append(f"Parameter count: {bl_np - an_np} fewer parameters")
    if not improvements:
        improvements.append("Nothing measurably improved over baseline")
    for imp in improvements:
        A(f"- {imp}")
    A("")

    A("### What Did Not Improve")
    A("")
    non_improvements = []
    if an_acc <= bl_acc + 0.005:
        non_improvements.append(f"Final accuracy: {an_acc - bl_acc:+.4f} (no material gain)")
    if not (an_first_solve and bl_first_solve and an_first_solve < bl_first_solve):
        non_improvements.append("Convergence speed: not clearly faster")
    if sps_an <= sps_bl * 1.05:
        non_improvements.append(f"Runtime: {sps_an:.1f} vs {sps_bl:.1f} sps")
    if not non_improvements:
        non_improvements.append("Everything improved")
    for ni in non_improvements:
        A(f"- {ni}")
    A("")

    A("### Next Mismatch If This Only Partially Helps")
    A("")
    A("If angular encoding helps convergence but not final accuracy, the next")
    A("mismatch to investigate is M2: the soft mixture still computes a **Euclidean**")
    A("weighted average of angular vectors rather than a proper circular mean with")
    A("normalization to the unit circle. A follow-up experiment would replace the")
    A("einsum mixture with per-phase-pair normalization after averaging.")
    A("")
    A("If angular encoding does not help at all, the binding constraint is likely")
    A("NOT the tau coordinate system but rather the sequential recurrence structure")
    A("itself (the D=24 loop), and the mismatches M3/M4 (SGD metric, Euclidean adjoint)")
    A("may be more relevant targets.")
    A("")

    with open(MD_OUT, "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"MD  → {MD_OUT}")


# ============================================================================
# Main
# ============================================================================
def main() -> None:
    print("=" * 70)
    print("Angular Phase Encoding Experiment v1")
    print("=" * 70)
    print(f"  Baseline: one-hot tau, D_TAU={D_TAU_ONEHOT}, D_IN={D_IN_ONEHOT}")
    print(f"  Angular:  cos/sin tau, D_TAU={D_TAU_ANG}, D_IN={D_IN_ANG}")
    print(f"  D={D_CONTEXT}, D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, "
          f"budget={BENCH_BUDGET}")
    print(f"  Checkpoints: {CHECKPOINTS}")
    print(f"  torch: {torch.__version__}, threads: {torch.get_num_threads()}")
    print()

    # ---- BFS warm-up (shared) ----
    print("Loading state tables (one-time BFS)...", flush=True)
    import random as _pyrand
    v6  = _load_v6torch()
    rng = _pyrand.Random(GLOBAL_SEED)
    pool = v6.build_warmup_pool(rng, size=v6.POOL_SIZE)
    n_st = v6.bfs_warm_up(pool, max_seconds=v6.BFS_MAX_SECS, verbose=True)
    TN_oh, TR, tau0_oh, pool_ids, _ = v6.build_state_tables(pool, verbose=True)
    print(f"  {n_st:,} states\n")

    # ---- Convert to angular encoding ----
    print("Converting state tables to angular encoding...", flush=True)
    TN_ang   = convert_onehot_to_angular(TN_oh)      # (N, 6, 8)
    tau0_ang = convert_onehot_to_angular(tau0_oh)     # (N, 8)
    print(f"  TN:   {TN_oh.shape} → {TN_ang.shape}")
    print(f"  tau0: {tau0_oh.shape} → {tau0_ang.shape}")

    # Verify angular encoding is correct for a sample
    sample_idx = 42
    oh_vec = TN_oh[sample_idx, 0, :]  # one-hot tau for state 42, op 0
    ang_vec = TN_ang[sample_idx, 0, :]
    print(f"  Sample verification (state={sample_idx}, op=0):")
    print(f"    one-hot: {oh_vec.tolist()}")
    print(f"    angular: {[round(x, 4) for x in ang_vec.tolist()]}")
    # Each cos/sin pair should have magnitude 1.0
    for i in range(4):
        mag = math.sqrt(ang_vec[2*i].item()**2 + ang_vec[2*i+1].item()**2)
        print(f"    phase pair {i}: magnitude={mag:.6f}", end="")
        print("  ✓" if abs(mag - 1.0) < 1e-5 else "  ✗ ERROR")
    print()

    device = "cpu"

    # ---- Run BASELINE (one-hot) ----
    print("=" * 50)
    print("BASELINE: one-hot tau (D_TAU=21, D_IN=25)")
    print("=" * 50, flush=True)
    model_bl = RouterBaseline(
        TN_oh, TR, tau0_oh, pool_ids,
        d_hidden=D_HIDDEN, seed=GLOBAL_SEED,
    )
    n_params_bl = sum(p.numel() for p in model_bl.parameters())
    print(f"  Parameters: {n_params_bl}")
    baseline_rows, runtime_bl = train_and_evaluate(
        "baseline_onehot", model_bl, pool_ids, device)
    sps_bl = BENCH_BUDGET / runtime_bl
    print(f"  Total: {runtime_bl:.1f}s, {sps_bl:.1f} sps\n")

    # ---- Run ANGULAR (cos/sin) ----
    print("=" * 50)
    print("ANGULAR: cos/sin tau (D_TAU=8, D_IN=12)")
    print("=" * 50, flush=True)
    model_an = RouterAngular(
        TN_ang, TR, tau0_ang, pool_ids,
        d_hidden=D_HIDDEN, seed=GLOBAL_SEED,
    )
    n_params_an = sum(p.numel() for p in model_an.parameters())
    print(f"  Parameters: {n_params_an}")
    angular_rows, runtime_an = train_and_evaluate(
        "angular_cossin", model_an, pool_ids, device)
    sps_an = BENCH_BUDGET / runtime_an
    print(f"  Total: {runtime_an:.1f}s, {sps_an:.1f} sps\n")

    # ---- Write CSV ----
    all_rows = baseline_rows + angular_rows
    fields = [
        "variant", "checkpoint", "accuracy", "alpha0", "route_entropy",
        "transport_frac", "b_pos0", "runtime_seconds", "loss",
        "d_tau", "d_in", "n_params", "jit", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        w.writerows(all_rows)
    print(f"CSV → {CSV_OUT}")

    # ---- Write Markdown ----
    write_md(baseline_rows, angular_rows, runtime_bl, runtime_an)

    # ---- Summary ----
    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)
    bl_final = [r for r in baseline_rows if r["checkpoint"] == BENCH_BUDGET]
    an_final = [r for r in angular_rows if r["checkpoint"] == BENCH_BUDGET]
    if bl_final and an_final:
        bl = bl_final[0]
        an = an_final[0]
        print(f"  {'':>20}  {'Baseline':>10}  {'Angular':>10}  {'Δ':>10}")
        print(f"  {'Accuracy':>20}  {bl['accuracy']:>10.4f}  {an['accuracy']:>10.4f}  {an['accuracy']-bl['accuracy']:>+10.4f}")
        print(f"  {'Alpha0':>20}  {bl['alpha0']:>10.4f}  {an['alpha0']:>10.4f}  {an['alpha0']-bl['alpha0']:>+10.4f}")
        print(f"  {'Route entropy':>20}  {bl['route_entropy']:>10.4f}  {an['route_entropy']:>10.4f}  {an['route_entropy']-bl['route_entropy']:>+10.4f}")
        print(f"  {'Parameters':>20}  {bl['n_params']:>10}  {an['n_params']:>10}  {an['n_params']-bl['n_params']:>+10}")
        print(f"  {'Steps/sec':>20}  {sps_bl:>10.1f}  {sps_an:>10.1f}  {sps_an-sps_bl:>+10.1f}")
    print("=" * 70)


if __name__ == "__main__":
    main()
