#!/usr/bin/env python3
"""run_v6_torch_d24_diagnostic.py

Bounded D=24 gradient/readout diagnostic on the canonical v6_torch path.

Goal: determine WHY attention remains at exactly alpha0=1/D=0.042 at D=24
      despite the model reaching 0.503 accuracy after 10K batches.

Inspects at checkpoints [0, 500, 1000, 2000, 5000, 10000]:

  1. Attention logits by position (are they symmetric / uniform?)
  2. Attention gradient norms by position (is position 0 receiving signal?)
  3. Weight gradient norms: W_attn, v_attn, W_tok_inject, W1, W2
  4. Tau state norm and variance by position
  5. X0-conditioned tau discriminability by position (between-group variance)
  6. Accuracy with normal attention vs. uniform-attention ablation

Canonical settings: D=24, D_HIDDEN=32, BATCH_SIZE=256, LR=0.02, 10K batches.
No architecture changes. No training extensions.
"""
from __future__ import annotations

import csv
import importlib.util
import random as pyrand
import sys
import time
from pathlib import Path
from typing import Dict, List, Tuple

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
SCRIPT_DIR = Path(__file__).parent
REPO_ROOT  = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CSV_OUT = RESULTS_DIR / "prime_transport_router_reintegration_v6_torch_d24_diagnostic.csv"
MD_OUT  = DOCS_DIR    / "prime_transport_router_reintegration_v6_torch_d24_diagnostic.md"

# ---------------------------------------------------------------------------
# Diagnostic configuration
# ---------------------------------------------------------------------------
D           = 24
BUDGET      = 10_000
BATCH_SIZE  = 256
B_DIAG      = 1024     # larger batch for stable gradient estimates
CHECKPOINTS = [0, 500, 1000, 2000, 5000, 10_000]

torch.set_num_threads(1)   # canonical setting

# ---------------------------------------------------------------------------
# Load v6_torch module
# ---------------------------------------------------------------------------

def _load_v6torch():
    spec = importlib.util.spec_from_file_location(
        "v6torch",
        str(SCRIPT_DIR / "run_router_reintegration_v6_torch.py"),
    )
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)  # type: ignore[union-attr]
    return mod


# ---------------------------------------------------------------------------
# Diagnostic forward (unscripted, exposes all intermediates)
# ---------------------------------------------------------------------------

def diagnostic_forward(
    model: nn.Module,
    sids_in: torch.Tensor,
    toks: torch.Tensor,
    x0: torch.Tensor,
    D_: int,
) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
    """Identical semantic to RouterV6.forward but returns intermediates.

    Returns:
        pred_logits:      (B, VOCAB)
        soft_taus_stack:  (B, D, D_TAU) — tau state at each step
        a_scores:         (B, D)         — raw attention logits (with retain_grad)
        alpha:            (B, D)         — attention weights
        loss:             scalar
    """
    sids     = sids_in.clone()
    tau_prev = model.tau0_table[sids]
    soft_taus: List[torch.Tensor] = []

    for t in range(D_):
        tn_batch = model.TN[sids]
        tok_t    = toks[:, t]
        embs     = model.W_emb[tok_t]
        h_in     = torch.cat([embs, tau_prev], dim=1)
        h        = torch.tanh(h_in @ model.W1 + model.b1)
        logits_r = h @ model.W2 + model.b2

        # eval temperature (deterministic, differentiable softmax)
        w = torch.softmax(logits_r / 0.05, dim=1)

        base     = torch.einsum("bi,bij->bj", w, tn_batch)
        tau_prev = (base + model.W_tok_inject[x0]) if t == 0 else base
        soft_taus.append(tau_prev)

        k_hard = w.argmax(dim=1)
        tr_rows = model.TR[sids]
        sids    = tr_rows.gather(1, k_hard.unsqueeze(1)).squeeze(1)

    soft_taus_stack = torch.stack(soft_taus, dim=1)  # (B, D, D_TAU)

    h_attn   = torch.tanh(
        soft_taus_stack @ model.W_attn.t() + model.b_attn
    )
    a_scores = (h_attn * model.v_attn).sum(dim=-1)  # (B, D)
    a_scores.retain_grad()

    alpha       = torch.softmax(a_scores, dim=1)
    pooled      = torch.einsum("bd,bdt->bt", alpha, soft_taus_stack)
    pred_logits = pooled @ model.W_pred + model.b_pred
    loss        = F.cross_entropy(pred_logits, x0)

    return pred_logits, soft_taus_stack, a_scores, alpha, loss


# ---------------------------------------------------------------------------
# X0-conditioned between-group variance of tau at position t
# ---------------------------------------------------------------------------

def x0_between_var(tau_t: torch.Tensor, x0: torch.Tensor, n_classes: int = 4) -> float:
    """How much do tau states at position t differ depending on x0?

    High value → tau_t is discriminative for x0 (model CAN use this position)
    Low value  → tau_t carries little x0 information (injection washed out)
    """
    means: List[torch.Tensor] = []
    for c in range(n_classes):
        mask = (x0 == c)
        if mask.sum() > 1:
            means.append(tau_t[mask].mean(dim=0))
    if len(means) < 2:
        return 0.0
    means_stack = torch.stack(means, dim=0)  # (n_classes, D_TAU)
    return float(means_stack.var(dim=0).mean().item())


# ---------------------------------------------------------------------------
# Run single diagnostic checkpoint
# ---------------------------------------------------------------------------

def run_checkpoint_diagnostic(
    model: nn.Module,
    pool_ids: torch.Tensor,
    rng_seed: int,
) -> dict:
    """Run diagnostic forward pass and collect all metrics."""
    model.train()  # need gradients for parameter .grad population
    model.zero_grad()

    # Sample diagnostic batch
    torch.manual_seed(rng_seed)
    idx  = torch.randint(0, len(pool_ids), (B_DIAG,))
    sids = pool_ids[idx]
    toks = torch.randint(0, 4, (B_DIAG, D))
    x0   = torch.randint(0, 4, (B_DIAG,))
    toks[:, 0] = x0

    with torch.enable_grad():
        pred_logits, soft_taus_stack, a_scores, alpha, loss = diagnostic_forward(
            model, sids, toks, x0, D
        )
        loss.backward()

    # ---- Per-position metrics ----
    position_metrics = []
    with torch.no_grad():
        for t in range(D):
            tau_t = soft_taus_stack[:, t].detach()   # (B, D_TAU)

            # Attention logit and weight at this position
            a_logit_mean  = float(a_scores[:, t].detach().mean().item())
            a_weight_mean = float(alpha[:, t].detach().mean().item())

            # Gradient of loss w.r.t. attention logit at position t
            if a_scores.grad is not None:
                a_grad_abs_mean = float(a_scores.grad[:, t].abs().mean().item())
            else:
                a_grad_abs_mean = float("nan")

            # Tau state stats
            tau_norm = float(tau_t.norm(dim=1).mean().item())
            tau_var  = float(tau_t.var(dim=0).mean().item())

            # X0-conditioned discriminability at this position
            tau_x0_bv = x0_between_var(tau_t, x0)

            position_metrics.append({
                "position":             t,
                "attention_logit_mean": round(a_logit_mean, 6),
                "attention_weight_mean":round(a_weight_mean, 6),
                "attention_grad_norm":  round(a_grad_abs_mean, 8),
                "tau_state_norm":       round(tau_norm, 6),
                "tau_state_variance":   round(tau_var, 6),
                "tau_x0_between_var":   round(tau_x0_bv, 8),
            })

    # ---- Weight gradient norms ----
    def gnorm(p: nn.Parameter) -> float:
        return float(p.grad.norm().item()) if p.grad is not None else float("nan")

    weight_grads = {
        "W_attn_grad_norm":       round(gnorm(model.W_attn),       6),
        "v_attn_grad_norm":       round(gnorm(model.v_attn),       6),
        "W_tok_inject_grad_norm": round(gnorm(model.W_tok_inject), 6),
        "W1_grad_norm":           round(gnorm(model.W1),           6),
        "W2_grad_norm":           round(gnorm(model.W2),           6),
        "W_pred_grad_norm":       round(gnorm(model.W_pred),       6),
    }

    # ---- Normal accuracy ----
    with torch.no_grad():
        accuracy_normal = float((pred_logits.detach().argmax(1) == x0).float().mean().item())

    # ---- Uniform-attention ablation accuracy ----
    with torch.no_grad():
        model.eval()
        uniform_alpha  = torch.ones(B_DIAG, D) / D
        tau_det        = soft_taus_stack.detach()
        pooled_uniform = torch.einsum("bd,bdt->bt", uniform_alpha, tau_det)
        pred_uniform   = pooled_uniform @ model.W_pred + model.b_pred
        acc_uniform    = float((pred_uniform.argmax(1) == x0).float().mean().item())
        model.train()

    # ---- Attention logit variance across positions (overall symmetry metric) ----
    with torch.no_grad():
        # Per-sample variance of a_scores across D positions → mean over batch
        a_logit_pos_var = float(a_scores.detach().var(dim=1).mean().item())
        a_logit_max_pos = int(a_scores.detach().mean(dim=0).argmax().item())

    return {
        "position_metrics":     position_metrics,
        "weight_grads":         weight_grads,
        "accuracy_normal":      round(accuracy_normal, 4),
        "accuracy_uniform_attn":round(acc_uniform, 4),
        "loss":                 round(float(loss.item()), 4),
        "a_logit_pos_var":      round(a_logit_pos_var, 6),
        "a_logit_max_pos":      a_logit_max_pos,
    }


# ---------------------------------------------------------------------------
# Sync trained parameters (skip large buffers TN / TR / tau0_table / pool_ids)
# ---------------------------------------------------------------------------

def sync_params_to_unscripted(scripted, model: nn.Module) -> None:
    """Copy trained parameters from scripted → unscripted model (skip buffers)."""
    sd = {
        k: v for k, v in scripted.state_dict().items()
        if k not in ("TN", "TR", "tau0_table", "pool_ids")
    }
    model.load_state_dict(sd, strict=False)


# ---------------------------------------------------------------------------
# Training loop with checkpoint diagnostics
# ---------------------------------------------------------------------------

def train_with_diagnostics(
    scripted,
    model: nn.Module,
    pool_ids: torch.Tensor,
    mod,
) -> List[dict]:
    """Train scripted model; at each checkpoint, sync to unscripted model and diagnose."""
    optimizer = torch.optim.SGD(scripted.parameters(), lr=mod.LR)
    torch.manual_seed(mod.GLOBAL_SEED + D)
    scripted.train()

    ckpt_set = set(CHECKPOINTS)
    results  = []

    print(f"  Training D={D}, D_HIDDEN=32, B={BATCH_SIZE}, budget={BUDGET:,}")
    t_start   = time.perf_counter()
    last_print = 0

    for bi in range(BUDGET + 1):
        if bi in ckpt_set:
            t_ckpt = time.perf_counter()
            sync_params_to_unscripted(scripted, model)
            diag = run_checkpoint_diagnostic(model, pool_ids, rng_seed=bi + 99)
            scripted.train()  # restore training mode after diagnostic

            elapsed = t_ckpt - t_start
            print(
                f"  ckpt={bi:>6}  loss={diag['loss']:.3f}  "
                f"acc={diag['accuracy_normal']:.3f}  "
                f"acc_unif={diag['accuracy_uniform_attn']:.3f}  "
                f"a_pos_var={diag['a_logit_pos_var']:.4f}  "
                f"max_pos={diag['a_logit_max_pos']}  "
                f"W_attn_g={diag['weight_grads']['W_attn_grad_norm']:.4f}  "
                f"W_inj_g={diag['weight_grads']['W_tok_inject_grad_norm']:.4f}  "
                f"{elapsed:.1f}s"
            )
            diag["checkpoint"] = bi
            results.append(diag)

        if bi >= BUDGET:
            break

        # Training step
        frac = bi / max(BUDGET - 1, 1)
        temp = float(mod.TEMP_START * (mod.TEMP_END / mod.TEMP_START) ** frac)

        idx  = torch.randint(0, len(pool_ids), (BATCH_SIZE,))
        sids = pool_ids[idx]
        toks = torch.randint(0, 4, (BATCH_SIZE, D))
        x0   = torch.randint(0, 4, (BATCH_SIZE,))
        toks[:, 0] = x0

        pred = scripted(sids, toks, x0, temp)
        loss = F.cross_entropy(pred, x0)
        optimizer.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(scripted.parameters(), 1.0)
        optimizer.step()

    return results


# ---------------------------------------------------------------------------
# Write CSV
# ---------------------------------------------------------------------------

def write_csv(all_results: List[dict]) -> None:
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)

    fieldnames = [
        "checkpoint",
        "position",
        "attention_logit_mean",
        "attention_weight_mean",
        "attention_grad_norm",
        "tau_state_norm",
        "tau_state_variance",
        "tau_x0_between_var",
        "router_grad_norm",       # W2 grad norm (named as per spec)
        "W_attn_grad_norm",
        "v_attn_grad_norm",
        "W_tok_inject_grad_norm",
        "W1_grad_norm",
        "W_pred_grad_norm",
        "accuracy_normal",
        "accuracy_uniform_attn",
        "attn_logit_pos_var",
        "attn_max_pos",
        "loss",
        "note",
    ]

    rows = []
    for diag in all_results:
        ckpt = diag["checkpoint"]
        wg   = diag["weight_grads"]
        for pm in diag["position_metrics"]:
            rows.append({
                "checkpoint":              ckpt,
                "position":                pm["position"],
                "attention_logit_mean":    pm["attention_logit_mean"],
                "attention_weight_mean":   pm["attention_weight_mean"],
                "attention_grad_norm":     pm["attention_grad_norm"],
                "tau_state_norm":          pm["tau_state_norm"],
                "tau_state_variance":      pm["tau_state_variance"],
                "tau_x0_between_var":      pm["tau_x0_between_var"],
                "router_grad_norm":        wg["W2_grad_norm"],
                "W_attn_grad_norm":        wg["W_attn_grad_norm"],
                "v_attn_grad_norm":        wg["v_attn_grad_norm"],
                "W_tok_inject_grad_norm":  wg["W_tok_inject_grad_norm"],
                "W1_grad_norm":            wg["W1_grad_norm"],
                "W_pred_grad_norm":        wg["W_pred_grad_norm"],
                "accuracy_normal":         diag["accuracy_normal"],
                "accuracy_uniform_attn":   diag["accuracy_uniform_attn"],
                "attn_logit_pos_var":      diag["a_logit_pos_var"],
                "attn_max_pos":            diag["a_logit_max_pos"],
                "loss":                    diag["loss"],
                "note": (
                    f"D={D},D_H=32,B={BATCH_SIZE},budget={BUDGET},"
                    f"B_diag={B_DIAG},canonical_v6_torch"
                ),
            })

    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"  CSV: {CSV_OUT}")


# ---------------------------------------------------------------------------
# Analysis helpers
# ---------------------------------------------------------------------------

def _summarize(all_results: List[dict]) -> dict:
    """Extract key findings for MD."""
    final = all_results[-1]
    early = all_results[1] if len(all_results) > 1 else all_results[0]

    pm_final = {p["position"]: p for p in final["position_metrics"]}
    pm_early = {p["position"]: p for p in early["position_metrics"]}

    # Is attention logit at position 0 different from others at final checkpoint?
    logits_final = [pm_final[t]["attention_logit_mean"] for t in range(D)]
    logit_p0     = logits_final[0]
    logit_mean   = sum(logits_final) / D
    logit_max    = max(range(D), key=lambda t: logits_final[t])

    # Is gradient at position 0 weaker than others?
    grads_final  = [pm_final[t]["attention_grad_norm"] for t in range(D)]
    grad_p0      = grads_final[0]
    grad_max_pos = max(range(D), key=lambda t: grads_final[t])
    grad_mean    = sum(grads_final) / D

    # Is tau_0 discriminative for x0?
    x0bv_final = [pm_final[t]["tau_x0_between_var"] for t in range(D)]
    x0bv_p0    = x0bv_final[0]
    x0bv_max_t = max(range(D), key=lambda t: x0bv_final[t])
    x0bv_mean  = sum(x0bv_final) / D

    # Accuracy delta between normal and uniform-attention
    acc_n = final["accuracy_normal"]
    acc_u = final["accuracy_uniform_attn"]
    acc_delta = acc_n - acc_u

    return {
        "logit_p0":           logit_p0,
        "logit_mean_others":  (sum(logits_final[1:]) / (D - 1)),
        "logit_max_pos":      logit_max,
        "grad_p0":            grad_p0,
        "grad_mean":          grad_mean,
        "grad_max_pos":       grad_max_pos,
        "x0bv_p0":            x0bv_p0,
        "x0bv_max_t":         x0bv_max_t,
        "x0bv_mean":          x0bv_mean,
        "acc_normal":         acc_n,
        "acc_uniform":        acc_u,
        "acc_delta":          acc_delta,
        "a_pos_var_final":    final["a_logit_pos_var"],
        "a_pos_var_early":    early["a_logit_pos_var"],
        "wg_attn":            final["weight_grads"]["W_attn_grad_norm"],
        "wg_inject":          final["weight_grads"]["W_tok_inject_grad_norm"],
        "wg_router":          final["weight_grads"]["W2_grad_norm"],
    }


# ---------------------------------------------------------------------------
# Write MD
# ---------------------------------------------------------------------------

def write_md(all_results: List[dict]) -> None:
    DOCS_DIR.mkdir(parents=True, exist_ok=True)
    s = _summarize(all_results)

    # Build checkpoint table
    hdr = ("| ckpt | loss | acc | acc_unif | Δacc | a_pos_var | "
           "W_attn_g | W_inj_g | W_router_g | note |")
    sep = "|---|---|---|---|---|---|---|---|---|---|"
    rows = []
    for diag in all_results:
        ckpt = diag["checkpoint"]
        wg   = diag["weight_grads"]
        acc  = diag["accuracy_normal"]
        acc_u= diag["accuracy_uniform_attn"]
        note = (
            "INIT" if ckpt == 0 else
            "learning" if acc < 0.95 and acc > 0.30 else
            "solved" if acc >= 0.95 else "chance"
        )
        rows.append(
            f"| {ckpt:>6} | {diag['loss']:.3f} | {acc:.3f} | {acc_u:.3f} | "
            f"{acc-acc_u:+.3f} | {diag['a_logit_pos_var']:.4f} | "
            f"{wg['W_attn_grad_norm']:.4f} | {wg['W_tok_inject_grad_norm']:.4f} | "
            f"{wg['W2_grad_norm']:.4f} | {note} |"
        )
    ckpt_table = "\n".join([hdr, sep] + rows)

    # Attention logit table at final checkpoint: position 0 vs. selected others
    final_pm = {p["position"]: p for p in all_results[-1]["position_metrics"]}
    attn_hdr = "| pos | attn_logit | attn_weight | attn_grad | tau_norm | tau_var | tau_x0_bv |"
    attn_sep = "|---|---|---|---|---|---|---|"
    probe_pos = [0, 1, 2, 3, 6, 12, 18, 23]
    attn_rows = []
    for t in probe_pos:
        pm = final_pm[t]
        flag = " ← inject pos" if t == 0 else ""
        attn_rows.append(
            f"| {t}{flag} | {pm['attention_logit_mean']:.4f} | "
            f"{pm['attention_weight_mean']:.4f} | {pm['attention_grad_norm']:.6f} | "
            f"{pm['tau_state_norm']:.4f} | {pm['tau_state_variance']:.4f} | "
            f"{pm['tau_x0_between_var']:.6f} |"
        )
    attn_table = "\n".join([attn_hdr, attn_sep] + attn_rows)

    # Diagnosis
    attn_symmetric    = s["a_pos_var_final"] < 0.01
    grad_p0_weak      = s["grad_p0"] < 0.3 * s["grad_mean"] if s["grad_mean"] > 0 else False
    tau0_disc         = s["x0bv_p0"] > 2.0 * s["x0bv_mean"]
    indirect_route    = s["acc_delta"] < 0.05  # normal ≈ uniform → attention not used

    if indirect_route:
        mechanism = (
            "**Indirect heuristic confirmed:** normal accuracy ≈ uniform-attention "
            f"accuracy (Δ={s['acc_delta']:+.3f}). The model is solving D=24 through "
            "routing trajectory statistics, not through step-0 attention retrieval. "
            "The attention mechanism is bypassed."
        )
    else:
        mechanism = (
            "**Attention does contribute:** normal accuracy > uniform-attention "
            f"accuracy by {s['acc_delta']:.3f}. The model uses attention, "
            "but not concentrated on position 0."
        )

    if attn_symmetric:
        sym_note = (
            "**Attention logits remain nearly symmetric** "
            f"(position variance = {s['a_pos_var_final']:.4f} ≈ 0). "
            "The model has not learned any position preference."
        )
    else:
        sym_note = (
            f"Attention logits show mild asymmetry "
            f"(position variance = {s['a_pos_var_final']:.4f}), "
            f"with maximum at position {s['logit_max_pos']}."
        )

    if tau0_disc:
        disc_note = (
            f"**Tau_0 is discriminative:** x0-conditioned between-group variance at "
            f"position 0 ({s['x0bv_p0']:.6f}) is above the position mean ({s['x0bv_mean']:.6f}). "
            "The injection signal survives in tau_0; the bottleneck is in the attention scorer."
        )
    else:
        disc_note = (
            f"**Tau_0 discriminability is LOW:** x0-conditioned between-group variance at "
            f"position 0 ({s['x0bv_p0']:.6f}) ≤ position mean ({s['x0bv_mean']:.6f}). "
            "The injection signal may be washed out by the large operator base vector."
        )

    if grad_p0_weak:
        grad_note = (
            f"**Gradient to position 0 is materially weaker** "
            f"(grad_norm[0]={s['grad_p0']:.6f} vs. mean={s['grad_mean']:.6f} — "
            f"ratio={s['grad_p0']/max(s['grad_mean'],1e-9):.2f}). "
            "BPTT signal dilution is contributing."
        )
    else:
        grad_note = (
            f"Gradient to position 0 is not materially weaker than other positions "
            f"(grad_norm[0]={s['grad_p0']:.6f} vs. mean={s['grad_mean']:.6f}). "
            "BPTT dilution through the attention path is NOT the primary bottleneck."
        )

    # Next step
    if indirect_route:
        next_step = (
            "**make one specific readout interface fix:** "
            "add an explicit position-0 bias to the attention scorer at initialization "
            "(a_scores[:, 0] += constant) to break the symmetry and give step-0 retrieval "
            "an inductive bias. This is a targeted single-parameter initialization change, "
            "not an architectural modification."
        )
    elif not tau0_disc:
        next_step = (
            "**make one specific readout interface fix:** "
            "scale up W_tok_inject initialization so the injection signal at position 0 "
            "is stronger relative to the operator base vector. This is a targeted "
            "initialization change, not a semantic change."
        )
    else:
        next_step = (
            "**extend D=24 budget:** tau_0 IS discriminative and attention gradients "
            "at position 0 are not suppressed; the model is still learning. "
            "Extend budget to 20K–30K batches to see if attention concentration develops."
        )

    md = f"""# prime_transport_router_reintegration_v6_torch_d24_diagnostic

**Branch:** bounded readout diagnostic  
**Date:** 2026-04-07  
**Surface:** canonical v6_torch, D=24, D_HIDDEN=32  
**Goal:** determine why alpha0 = 1/D=0.042 at D=24 despite 0.503 accuracy after 10K batches

---

## 1. Experiment Setup

- D=24, D_HIDDEN=32 (canonical), BATCH_SIZE={BATCH_SIZE}, BUDGET={BUDGET:,}
- Diagnostic batch B_DIAG={B_DIAG} for stable gradient estimates
- torch.set_num_threads(1) (canonical setting)
- Checkpoints: {CHECKPOINTS}
- Diagnostic forward: unscripted (allows retain_grad on a_scores)
- Training forward: torch.jit.script (canonical speed)
- No architecture changes. No training extensions.

---

## 2. Checkpoint Summary

{ckpt_table}

**Δacc = normal_accuracy − uniform_attention_accuracy**  
If Δacc ≈ 0 at a checkpoint, the model is NOT using attention for retrieval.

---

## 3. Per-Position Attention Analysis (final checkpoint = {CHECKPOINTS[-1]})

{attn_table}

chance = 0.250. Expected alpha_weight if uniform = {1/D:.4f}.

---

## 4. Diagnostic Findings

### 4.1 Is attention symmetric?

{sym_note}

Attention position variance evolution:
- Early (ckpt={CHECKPOINTS[1]}): {s['a_pos_var_early']:.4f}
- Final (ckpt={CHECKPOINTS[-1]}):  {s['a_pos_var_final']:.4f}

### 4.2 Is the gradient to step-0 attention materially weaker?

{grad_note}

Weight gradient norms at final checkpoint:
- W_attn:       {s['wg_attn']:.4f}
- W_tok_inject: {s['wg_inject']:.4f}  
- W2 (router):  {s['wg_router']:.4f}

### 4.3 Are tau states at position 0 discriminative for x0?

{disc_note}

Position with highest x0-discriminability: t={s['x0bv_max_t']}  
Position 0 x0-between-var: {s['x0bv_p0']:.6f}  
Position mean x0-between-var: {s['x0bv_mean']:.6f}

### 4.4 Is D=24 accuracy from indirect routing, not step-0 retrieval?

{mechanism}

---

## 5. Root Cause Summary

At final checkpoint (ckpt={CHECKPOINTS[-1]}):
- Accuracy (normal attention):  {s['acc_normal']:.3f}
- Accuracy (uniform attention):  {s['acc_uniform']:.3f}
- Attention position variance:   {s['a_pos_var_final']:.4f}
- Tau_0 x0-discriminability:     {s['x0bv_p0']:.6f}
- Position-0 attn gradient norm: {s['grad_p0']:.6f}
- Mean attn gradient norm:       {s['grad_mean']:.6f}

---

## 6. Honesty Section

- Were any core files modified? No
- Were any operators rebuilt? No
- Is full exact spin_H solved? No
- Were any architecture changes made? No
- Is this a training extension? No — same 10K budget as canonical

---

## 7. Next Step (exactly one)

{next_step}
"""

    with open(MD_OUT, "w") as f:
        f.write(md)
    print(f"  MD:  {MD_OUT}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    print("=" * 60)
    print(f"D=24 Gradient/Readout Diagnostic (canonical v6_torch)")
    print("=" * 60)
    print(f"  D={D}, D_HIDDEN=32, B={BATCH_SIZE}, budget={BUDGET:,}")
    print(f"  B_DIAG={B_DIAG}, checkpoints={CHECKPOINTS}", flush=True)

    # Load v6_torch infrastructure
    print("\nLoading v6_torch module...", flush=True)
    mod = _load_v6torch()

    # BFS warm-up
    py_rng = pyrand.Random(mod.GLOBAL_SEED)
    pool   = mod.build_warmup_pool(py_rng, size=mod.POOL_SIZE)
    mod.bfs_warm_up(pool, max_seconds=mod.BFS_MAX_SECS, verbose=True)

    # State tables
    TN, TR, tau0_table, pool_ids, _ = mod.build_state_tables(pool, verbose=True)

    # Build BOTH scripted (for training) and unscripted (for diagnostics)
    model = mod.RouterV6(TN, TR, tau0_table, pool_ids,
                         seed=mod.GLOBAL_SEED + D)
    model.train()
    scripted = torch.jit.script(model)

    # Run diagnostic training
    t0      = time.perf_counter()
    results = train_with_diagnostics(scripted, model, pool_ids, mod)
    print(f"\nTotal: {time.perf_counter()-t0:.1f}s", flush=True)

    # Write deliverables
    print("\n=== Writing deliverables ===", flush=True)
    write_csv(results)
    write_md(results)

    print("Done.")


if __name__ == "__main__":
    main()
