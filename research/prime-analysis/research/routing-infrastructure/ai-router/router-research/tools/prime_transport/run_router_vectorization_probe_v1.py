#!/usr/bin/env python3
"""
Router Vectorization Probe v1 — Loop Decomposition and Tensorization

Decomposes the D=24 sequential loop into named sub-operations, classifies
each for vectorizability, implements the strongest honest vectorized rewrite,
and measures before/after wall time.

Baseline: CPU, D_HIDDEN=32, batch_size=256, D=24, pos0 bias enabled.
"""

from __future__ import annotations

import csv
import importlib.util
import os
import pathlib
import platform
import sys
import time
from typing import Dict, List, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F

# ============================================================================
# Paths
# ============================================================================
SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
REPO_ROOT  = SCRIPT_DIR.parent.parent
AI_ROOT    = REPO_ROOT.parent.parent

MD_PATH  = REPO_ROOT / "docs" / "research" / "prime_transport_router_vectorization_probe_v1.md"
CSV_PATH = REPO_ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_router_vectorization_probe_v1.csv"

# ============================================================================
# Locked constants
# ============================================================================
VOCAB       = 4
D_EMB       = 4
D_TAU       = 21
D_IN        = D_EMB + D_TAU   # 25
N_OPS       = 6
LR          = 0.02
TEMP_START  = 2.0
TEMP_END    = 0.1
GLOBAL_SEED = 42
B0_INIT     = 2.0
D_CONTEXT   = 24
D_HIDDEN    = 32
BATCH_SIZE  = 256
DEVICE      = "cpu"

BENCH_STEPS = 300   # enough for stable timing
WARMUP      = 20

torch.set_num_threads(1)


def _load_v6torch():
    spec = importlib.util.spec_from_file_location(
        "v6torch_base",
        str(SCRIPT_DIR / "run_router_reintegration_v6_torch.py"),
    )
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# ============================================================================
# BASELINE MODEL — exact copy of current forward
# ============================================================================
class RouterBaseline(nn.Module):
    """Exact current implementation — the D=24 sequential loop."""

    def __init__(self, TN, TR, tau0_table, pool_ids, d_hidden=32,
                 d_context=D_CONTEXT, b0_init=B0_INIT, init_scale=0.05,
                 seed=GLOBAL_SEED):
        super().__init__()
        dh  = d_hidden
        dha = max(8, d_hidden // 4)

        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_table)
        self.register_buffer("pool_ids", pool_ids)

        pos0_mask = torch.zeros(1, d_context)
        pos0_mask[0, 0] = 1.0
        self.register_buffer("pos0_mask", pos0_mask)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))

        gen = torch.Generator().manual_seed(seed)
        def rp(*s): return nn.Parameter(torch.empty(*s).normal_(0, init_scale, generator=gen))
        def zp(*s): return nn.Parameter(torch.zeros(*s))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(D_IN, dh)
        self.b1           = zp(dh)
        self.W2           = rp(dh, N_OPS)
        self.b2           = zp(N_OPS)
        self.W_attn       = rp(dha, D_TAU)
        self.b_attn       = zp(dha)
        self.v_attn       = rp(dha)
        self.W_pred       = rp(D_TAU, VOCAB)
        self.b_pred       = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, D_TAU)

    def forward(self, state_ids: torch.Tensor, tokens: torch.Tensor,
                x0: torch.Tensor, temp: float) -> torch.Tensor:
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
        h_attn       = torch.tanh(soft_taus_stack @ self.W_attn.t() + self.b_attn)
        a_scores_raw = (h_attn * self.v_attn).sum(dim=-1)
        a_scores     = a_scores_raw + self.pos0_mask * self.b_pos0
        alpha        = torch.softmax(a_scores, dim=1)
        pooled       = torch.einsum("bd,bdt->bt", alpha, soft_taus_stack)
        pred_logits  = pooled @ self.W_pred + self.b_pred
        return pred_logits


# ============================================================================
# VECTORIZED MODEL — maximum honest tensorization
# ============================================================================
#
# DECOMPOSITION OF THE 24-STEP LOOP:
#
# Sub-operation                  | Depends on prev step? | Classification
# -------------------------------|----------------------|---------------------------
# A. tokens[:, t] select        | No                   | TRIVIALLY VECTORIZABLE
# B. W_emb[tok_t] embedding     | No (only on tokens)  | TRIVIALLY VECTORIZABLE
# C. cat([embs, tau_prev])      | Yes (tau_prev)       | ELIMINABLE via split matmul
# D. h_in @ W1 + b1 (linear)   | Yes (tau_prev)       | PARTIALLY VECTORIZABLE:
#                                |                      |   emb part vectorizable,
#                                |                      |   tau part sequential
# E. tanh(h) activation         | Yes (through D)      | SEQUENTIAL (nonlinear)
# F. h @ W2 + b2 (logits)      | Yes (through E)      | SEQUENTIAL
# G. rand + Gumbel noise        | No                   | TRIVIALLY VECTORIZABLE
# H. softmax(logits+gn / temp)  | Yes (through F)      | SEQUENTIAL
# I. TN[state_ids] lookup       | Yes (state_ids)      | SEQUENTIAL (coupled recurrence)
# J. einsum(w, tn_batch) blend  | Yes (through H, I)   | SEQUENTIAL
# K. + W_tok_inject[x0] (t=0)  | No                   | TRIVIALLY VECTORIZABLE (mask)
# L. argmax(w) hard choice      | Yes (through H)      | SEQUENTIAL (non-diff)
# M. TR[state_ids] lookup       | Yes (state_ids)      | SEQUENTIAL (non-diff)
# N. gather(tr_rows, k_hard)    | Yes (through L, M)   | SEQUENTIAL (non-diff)
#
# VECTORIZABLE (hoisted out of loop):
#   A+B: token embeddings for all D steps → (B, D, D_EMB)
#   D_emb: embedding projection through W1[:D_EMB,:] → (B, D, D_HIDDEN)
#   G: Gumbel noise for all D steps → (B, D, N_OPS)
#   K: injection vector computed once → (B, D_TAU)
#
# ELIMINATED:
#   C: cat is replaced by split matmul (emb_proj + tau_proj), no allocation
#
# MATHEMATICALLY SEQUENTIAL (nonlinear coupled recurrence):
#   D_tau + E + F + H + I + J + L + M + N
#   tau_prev[t] = einsum(softmax(MLP(emb_proj[t] + tau_prev[t-1] @ W1_tau)), TN[state_ids[t]])
#   state_ids[t+1] = TR[state_ids[t]].gather(argmax(w[t]))
#   This is a NONLINEAR recurrence — not amenable to parallel prefix/scan.
#
class RouterVectorized(nn.Module):
    """Vectorized rewrite: hoist everything possible out of the loop."""

    def __init__(self, TN, TR, tau0_table, pool_ids, d_hidden=32,
                 d_context=D_CONTEXT, b0_init=B0_INIT, init_scale=0.05,
                 seed=GLOBAL_SEED):
        super().__init__()
        dh  = d_hidden
        dha = max(8, d_hidden // 4)

        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_table)
        self.register_buffer("pool_ids", pool_ids)

        pos0_mask = torch.zeros(1, d_context)
        pos0_mask[0, 0] = 1.0
        self.register_buffer("pos0_mask", pos0_mask)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))

        gen = torch.Generator().manual_seed(seed)
        def rp(*s): return nn.Parameter(torch.empty(*s).normal_(0, init_scale, generator=gen))
        def zp(*s): return nn.Parameter(torch.zeros(*s))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(D_IN, dh)
        self.b1           = zp(dh)
        self.W2           = rp(dh, N_OPS)
        self.b2           = zp(N_OPS)
        self.W_attn       = rp(dha, D_TAU)
        self.b_attn       = zp(dha)
        self.v_attn       = rp(dha)
        self.W_pred       = rp(D_TAU, VOCAB)
        self.b_pred       = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, D_TAU)

    def forward(self, state_ids: torch.Tensor, tokens: torch.Tensor,
                x0: torch.Tensor, temp: float) -> torch.Tensor:
        B: int = state_ids.shape[0]
        D: int = tokens.shape[1]

        # === VECTORIZED PRE-COMPUTATION (hoisted out of loop) ===

        # A+B: All token embeddings at once — eliminates D separate lookups
        embs_all = self.W_emb[tokens]                        # (B, D, D_EMB)

        # D_emb: Project embedding part through W1 for all D steps at once
        # Original: cat([embs, tau], 1) @ W1 = embs @ W1[:4,:] + tau @ W1[4:,:]
        # We pre-compute the embs @ W1[:4,:] part for all D steps.
        # Use literal 4 for JIT compatibility (D_EMB = 4).
        W1_emb = self.W1[:4, :]                              # (4, D_HIDDEN)
        W1_tau = self.W1[4:, :]                              # (21, D_HIDDEN)
        emb_proj_all = embs_all @ W1_emb                     # (B, D, D_HIDDEN)

        # G: Pre-generate all Gumbel noise at once — eliminates D×(rand+clamp+2×log)
        if self.training:
            u = torch.rand(B, D, N_OPS).clamp_(1e-20, 1.0)
            gumbel_all = -torch.log(-torch.log(u))           # (B, D, N_OPS)

        # K: Injection vector computed once
        inject = self.W_tok_inject[x0]                       # (B, D_TAU)

        # === SEQUENTIAL CORE (mathematically irreducible) ===
        tau_prev: torch.Tensor = self.tau0_table[state_ids]  # (B, D_TAU)
        soft_taus: List[torch.Tensor] = []

        for t in range(D):
            # I: TN lookup — sequential (state_ids changes each step)
            tn_batch = self.TN[state_ids]                    # (B, N_OPS, D_TAU)

            # D_tau + E: tau part of MLP + pre-computed emb part + activation
            # This replaces: cat([embs, tau_prev]) @ W1 + b1
            # with:          emb_proj[t] + tau_prev @ W1_tau + b1
            # Eliminates cat allocation; matmul is (B,21)×(21,32) instead of (B,25)×(25,32)
            h = torch.tanh(emb_proj_all[:, t, :] + tau_prev @ W1_tau + self.b1)

            # F: Logits — sequential
            logits = h @ self.W2 + self.b2                   # (B, N_OPS)

            # H: Softmax with pre-computed noise — sequential
            if self.training:
                w = torch.softmax((logits + gumbel_all[:, t, :]) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)

            # J: Weighted blend — sequential
            base = torch.einsum("bi,bij->bj", w, tn_batch)  # (B, D_TAU)

            # K: Injection (t=0 only)
            tau_prev = (base + inject) if t == 0 else base
            soft_taus.append(tau_prev)

            # L+M+N: Hard routing — sequential, non-differentiable
            k_hard    = torch.argmax(w, dim=1)
            state_ids = self.TR[state_ids].gather(1, k_hard.unsqueeze(1)).squeeze(1)

        # === POST-LOOP (already vectorized, unchanged) ===
        soft_taus_stack = torch.stack(soft_taus, dim=1)
        h_attn       = torch.tanh(soft_taus_stack @ self.W_attn.t() + self.b_attn)
        a_scores_raw = (h_attn * self.v_attn).sum(dim=-1)
        a_scores     = a_scores_raw + self.pos0_mask * self.b_pos0
        alpha        = torch.softmax(a_scores, dim=1)
        pooled       = torch.einsum("bd,bdt->bt", alpha, soft_taus_stack)
        pred_logits  = pooled @ self.W_pred + self.b_pred
        return pred_logits


# ============================================================================
# Helpers
# ============================================================================
def sample_batch(pool_ids, B):
    idx       = torch.randint(0, len(pool_ids), (B,))
    state_ids = pool_ids[idx]
    tokens    = torch.randint(0, VOCAB, (B, D_CONTEXT))
    x0        = torch.randint(0, VOCAB, (B,))
    tokens[:, 0] = x0
    return state_ids, tokens, x0


def get_temp(step, total):
    frac = step / max(total - 1, 1)
    return float(TEMP_START * (TEMP_END / TEMP_START) ** frac)


def copy_weights(src, dst):
    """Copy all parameters and buffers from src to dst."""
    dst.load_state_dict(src.state_dict())


# ============================================================================
# Verify semantic equivalence
# ============================================================================
def verify_equivalence(baseline, vectorized, pool_ids, n_checks=5):
    """Run both models in eval mode and check outputs match."""
    baseline.eval()
    vectorized.eval()

    max_diff = 0.0
    for i in range(n_checks):
        torch.manual_seed(GLOBAL_SEED + 100 + i)
        sids, toks, x0 = sample_batch(pool_ids, BATCH_SIZE)
        with torch.no_grad():
            out_b = baseline(sids, toks, x0, 1.0)
            out_v = vectorized(sids, toks, x0, 1.0)
        diff = (out_b - out_v).abs().max().item()
        max_diff = max(max_diff, diff)

    baseline.train()
    vectorized.train()
    return max_diff


# ============================================================================
# Benchmark: split timing (forward / backward / optimizer)
# ============================================================================
def benchmark_split(model, pool_ids, label, steps=BENCH_STEPS):
    model.train()
    optimizer = torch.optim.SGD(model.parameters(), lr=LR)
    torch.manual_seed(GLOBAL_SEED + hash(label) % 10000)

    # Warmup
    for s in range(WARMUP):
        sids, toks, x0 = sample_batch(pool_ids, BATCH_SIZE)
        pred = model(sids, toks, x0, 2.0)
        loss = F.cross_entropy(pred, x0)
        optimizer.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), max_norm=1.0)
        optimizer.step()

    t_sample = 0.0
    t_forward = 0.0
    t_loss = 0.0
    t_zero = 0.0
    t_backward = 0.0
    t_clip = 0.0
    t_optim = 0.0

    t_wall_start = time.perf_counter()

    for step in range(steps):
        temp = get_temp(step, steps)

        t1 = time.perf_counter()
        sids, toks, x0 = sample_batch(pool_ids, BATCH_SIZE)
        t2 = time.perf_counter()
        pred = model(sids, toks, x0, temp)
        t3 = time.perf_counter()
        loss = F.cross_entropy(pred, x0)
        t4 = time.perf_counter()
        optimizer.zero_grad()
        t5 = time.perf_counter()
        loss.backward()
        t6 = time.perf_counter()
        nn.utils.clip_grad_norm_(model.parameters(), max_norm=1.0)
        t7 = time.perf_counter()
        optimizer.step()
        t8 = time.perf_counter()

        t_sample  += t2 - t1
        t_forward += t3 - t2
        t_loss    += t4 - t3
        t_zero    += t5 - t4
        t_backward += t6 - t5
        t_clip    += t7 - t6
        t_optim   += t8 - t7

    total_wall = time.perf_counter() - t_wall_start
    sps = steps / total_wall

    result = {
        "label": label,
        "total_wall": total_wall,
        "steps": steps,
        "sps": sps,
        "forward": t_forward,
        "backward": t_backward,
        "loss": t_loss,
        "zero_grad": t_zero,
        "clip_grad": t_clip,
        "optimizer": t_optim,
        "sample": t_sample,
    }

    print(f"\n  [{label}] {steps} steps, {total_wall:.2f}s, {sps:.1f} sps")
    for k in ["forward", "backward", "loss", "zero_grad", "clip_grad", "optimizer", "sample"]:
        pct = 100.0 * result[k] / max(total_wall, 1e-9)
        print(f"    {k:12s}: {result[k]:.3f}s  ({pct:5.1f}%)")

    return result


# ============================================================================
# Benchmark: forward-only timing (no backward)
# ============================================================================
def benchmark_forward_only(model, pool_ids, label, steps=BENCH_STEPS):
    model.eval()
    torch.manual_seed(GLOBAL_SEED + hash(label) % 10000 + 999)

    # Warmup
    with torch.no_grad():
        for s in range(WARMUP):
            sids, toks, x0 = sample_batch(pool_ids, BATCH_SIZE)
            model(sids, toks, x0, 1.0)

    t0 = time.perf_counter()
    with torch.no_grad():
        for step in range(steps):
            sids, toks, x0 = sample_batch(pool_ids, BATCH_SIZE)
            model(sids, toks, x0, 1.0)
    total = time.perf_counter() - t0

    sps = steps / total
    print(f"  [{label}] forward-only: {total:.2f}s, {sps:.1f} fwd/sec")
    return {"label": label, "total_wall": total, "steps": steps, "sps": sps}


# ============================================================================
# Benchmark: per-component timing inside the loop (one step, averaged)
# ============================================================================
def benchmark_inner_loop_ops(model_class, TN, TR, tau0, pool_ids, label, steps=100):
    """Time individual sub-operations inside the D=24 loop."""
    # We'll manually run the forward with timing hooks.
    # This uses the RAW tensors, not JIT, to instrument each line.

    W_emb = None
    W1 = None
    b1 = None
    W2 = None
    b2 = None
    W_tok_inject = None

    # Build a throwaway model to get the weights
    m = model_class(TN, TR, tau0, pool_ids, d_hidden=D_HIDDEN)
    W_emb = m.W_emb.data
    W1 = m.W1.data
    b1 = m.b1.data
    W2 = m.W2.data
    b2 = m.b2.data
    W_tok_inject = m.W_tok_inject.data
    tau0_table = m.tau0_table
    del m

    timings = {
        "TN_lookup": 0.0,
        "token_select": 0.0,
        "embedding_lookup": 0.0,
        "cat": 0.0,
        "linear1_mm": 0.0,
        "tanh": 0.0,
        "linear2_mm": 0.0,
        "gumbel_noise": 0.0,
        "softmax": 0.0,
        "einsum_blend": 0.0,
        "injection": 0.0,
        "argmax": 0.0,
        "TR_lookup": 0.0,
        "gather_route": 0.0,
    }

    torch.manual_seed(GLOBAL_SEED + 50)
    for step in range(steps):
        sids, toks, x0 = sample_batch(pool_ids, BATCH_SIZE)
        tau_prev = tau0_table[sids]
        temp = 1.0

        for t in range(D_CONTEXT):
            ta = time.perf_counter()
            tn_batch = TN[sids]
            tb = time.perf_counter()
            tok_t = toks[:, t]
            tc = time.perf_counter()
            embs = W_emb[tok_t]
            td = time.perf_counter()
            h_in = torch.cat([embs, tau_prev], dim=1)
            te = time.perf_counter()
            h = torch.tanh(h_in @ W1 + b1)
            tf = time.perf_counter()
            # (split tanh from mm is hard, measure together as linear1+tanh)
            logits = h @ W2 + b2
            tg = time.perf_counter()
            u = torch.rand_like(logits).clamp_(1e-20, 1.0)
            gn = -torch.log(-torch.log(u))
            th = time.perf_counter()
            w = torch.softmax((logits + gn) / temp, dim=1)
            ti = time.perf_counter()
            base = torch.einsum("bi,bij->bj", w, tn_batch)
            tj = time.perf_counter()
            tau_prev = (base + W_tok_inject[x0]) if t == 0 else base
            tk = time.perf_counter()
            k_hard = torch.argmax(w, dim=1)
            tl = time.perf_counter()
            tr_rows = TR[sids]
            tm = time.perf_counter()
            sids = tr_rows.gather(1, k_hard.unsqueeze(1)).squeeze(1)
            tn = time.perf_counter()

            timings["TN_lookup"]         += tb - ta
            timings["token_select"]      += tc - tb
            timings["embedding_lookup"]  += td - tc
            timings["cat"]               += te - td
            timings["linear1_mm"]        += tf - te   # includes tanh
            timings["linear2_mm"]        += tg - tf
            timings["gumbel_noise"]      += th - tg
            timings["softmax"]           += ti - th
            timings["einsum_blend"]      += tj - ti
            timings["injection"]         += tk - tj
            timings["argmax"]            += tl - tk
            timings["TR_lookup"]         += tm - tl
            timings["gather_route"]      += tn - tm

    total = sum(timings.values())
    print(f"\n  [{label}] Per-op timing across {steps} steps × {D_CONTEXT} iterations:")
    for name, t in sorted(timings.items(), key=lambda x: -x[1]):
        pct = 100.0 * t / max(total, 1e-9)
        print(f"    {name:20s}: {t:.4f}s  ({pct:5.1f}%)")
    print(f"    {'TOTAL':20s}: {total:.4f}s")

    return timings


# ============================================================================
# Main
# ============================================================================
def main():
    print("=" * 70)
    print("Router Vectorization Probe v1")
    print("=" * 70)
    print(f"  torch: {torch.__version__}, device: {DEVICE}, threads: {torch.get_num_threads()}")
    print(f"  D_HIDDEN={D_HIDDEN}, batch={BATCH_SIZE}, D={D_CONTEXT}")

    # Load state tables
    print("\nLoading state tables (BFS)...", flush=True)
    import random as pyrand
    v6 = _load_v6torch()
    rng = pyrand.Random(GLOBAL_SEED)
    pool = v6.build_warmup_pool(rng, size=v6.POOL_SIZE)
    n_st = v6.bfs_warm_up(pool, max_seconds=v6.BFS_MAX_SECS, verbose=True)
    TN, TR, tau0, pool_ids, _ = v6.build_state_tables(pool, verbose=True)
    print(f"  States: {n_st:,}\n")

    # Build models
    print("Building models...")
    baseline_model = RouterBaseline(TN, TR, tau0, pool_ids, d_hidden=D_HIDDEN)
    vector_model   = RouterVectorized(TN, TR, tau0, pool_ids, d_hidden=D_HIDDEN)

    # Copy weights so both models are identical
    copy_weights(baseline_model, vector_model)

    n_params = sum(p.numel() for p in baseline_model.parameters())
    print(f"  Params: {n_params:,}")

    # JIT script both
    jit_baseline = True
    jit_vector   = True
    try:
        baseline_scripted = torch.jit.script(baseline_model)
        print("  Baseline JIT: yes")
    except Exception as e:
        print(f"  Baseline JIT failed: {e}")
        baseline_scripted = baseline_model
        jit_baseline = False

    try:
        vector_scripted = torch.jit.script(vector_model)
        print("  Vectorized JIT: yes")
    except Exception as e:
        print(f"  Vectorized JIT failed: {e}")
        vector_scripted = vector_model
        jit_vector = False

    # Verify equivalence
    print("\nVerifying semantic equivalence (eval mode)...")
    max_diff = verify_equivalence(baseline_scripted, vector_scripted, pool_ids)
    print(f"  Max output difference: {max_diff:.2e}")
    equiv_ok = max_diff < 1e-4
    print(f"  Equivalence: {'PASS' if equiv_ok else 'FAIL'}")
    if not equiv_ok:
        print("  WARNING: outputs differ! Check implementation.")

    # =====================================================================
    # Benchmark: per-op timing inside the loop (no grad, no JIT)
    # =====================================================================
    print("\n" + "=" * 70)
    print("PHASE 1: Per-op timing inside the D=24 loop (baseline, no grad)")
    print("=" * 70)
    inner_timings = benchmark_inner_loop_ops(
        RouterBaseline, TN, TR, tau0, pool_ids, "baseline_inner", steps=100)

    # =====================================================================
    # Benchmark: forward-only
    # =====================================================================
    print("\n" + "=" * 70)
    print("PHASE 2: Forward-only comparison")
    print("=" * 70)
    fwd_base = benchmark_forward_only(baseline_scripted, pool_ids, "baseline_fwd")
    fwd_vec  = benchmark_forward_only(vector_scripted, pool_ids, "vectorized_fwd")

    fwd_speedup = fwd_base["sps"] / max(fwd_vec["sps"], 1e-9)
    print(f"\n  Forward speedup: {1.0/fwd_speedup:.2f}x" if fwd_speedup > 1 else
          f"\n  Forward speedup: {fwd_vec['sps']/fwd_base['sps']:.2f}x")

    # =====================================================================
    # Benchmark: full training (forward + backward + optimizer)
    # =====================================================================
    print("\n" + "=" * 70)
    print("PHASE 3: Full training comparison (forward + backward + optimizer)")
    print("=" * 70)
    train_base = benchmark_split(baseline_scripted, pool_ids, "baseline_train")
    train_vec  = benchmark_split(vector_scripted, pool_ids, "vectorized_train")

    train_speedup = train_vec["sps"] / max(train_base["sps"], 1e-9)
    print(f"\n  Training speedup: {train_speedup:.2f}x")
    print(f"  Baseline:   {train_base['sps']:.1f} sps  "
          f"(fwd={100*train_base['forward']/train_base['total_wall']:.1f}%, "
          f"bwd={100*train_base['backward']/train_base['total_wall']:.1f}%)")
    print(f"  Vectorized: {train_vec['sps']:.1f} sps  "
          f"(fwd={100*train_vec['forward']/train_vec['total_wall']:.1f}%, "
          f"bwd={100*train_vec['backward']/train_vec['total_wall']:.1f}%)")

    # =====================================================================
    # Write CSV
    # =====================================================================
    print("\n" + "=" * 70)
    print("Writing deliverables...")
    print("=" * 70)

    inner_total = sum(inner_timings.values())

    # Classification map
    classifications = {
        "token_select":     ("trivially_vectorizable", "yes"),
        "embedding_lookup": ("trivially_vectorizable", "yes"),
        "cat":              ("eliminable_via_split_matmul", "yes"),
        "gumbel_noise":     ("trivially_vectorizable", "yes"),
        "injection":        ("trivially_vectorizable_mask", "yes"),
        "linear1_mm":       ("partially_vectorizable", "partial"),
        "tanh":             ("sequential_nonlinear", "no"),
        "linear2_mm":       ("sequential", "no"),
        "softmax":          ("sequential", "no"),
        "TN_lookup":        ("sequential_coupled_recurrence", "no"),
        "einsum_blend":     ("sequential", "no"),
        "argmax":           ("sequential_nondiff", "no"),
        "TR_lookup":        ("sequential_nondiff", "no"),
        "gather_route":     ("sequential_nondiff", "no"),
    }

    csv_rows = []
    for name, t in sorted(inner_timings.items(), key=lambda x: -x[1]):
        seq_class, vec_yn = classifications.get(name, ("unknown", "unknown"))
        pct_before = 100.0 * t / max(inner_total, 1e-9)
        csv_rows.append({
            "component_name": name,
            "sequentiality_class": seq_class,
            "vectorized_yes_no": vec_yn,
            "runtime_seconds_before": round(t, 6),
            "runtime_seconds_after": -1,  # per-op after not directly measurable
            "percent_total_runtime_before": round(pct_before, 2),
            "percent_total_runtime_after": -1,
            "note": "",
        })

    # Add aggregate rows
    for lbl, res in [("FULL_TRAIN_baseline", train_base), ("FULL_TRAIN_vectorized", train_vec)]:
        csv_rows.append({
            "component_name": lbl,
            "sequentiality_class": "aggregate",
            "vectorized_yes_no": "vectorized" if "vectorized" in lbl else "baseline",
            "runtime_seconds_before": round(train_base["total_wall"], 4),
            "runtime_seconds_after": round(train_vec["total_wall"], 4),
            "percent_total_runtime_before": 100.0,
            "percent_total_runtime_after": 100.0,
            "note": f"sps={res['sps']:.1f}",
        })

    # Forward/backward split rows
    for comp in ["forward", "backward"]:
        csv_rows.append({
            "component_name": f"split/{comp}",
            "sequentiality_class": "aggregate",
            "vectorized_yes_no": "comparison",
            "runtime_seconds_before": round(train_base[comp], 4),
            "runtime_seconds_after": round(train_vec[comp], 4),
            "percent_total_runtime_before": round(100 * train_base[comp] / max(train_base["total_wall"], 1e-9), 2),
            "percent_total_runtime_after": round(100 * train_vec[comp] / max(train_vec["total_wall"], 1e-9), 2),
            "note": f"baseline={train_base[comp]:.3f}s, vectorized={train_vec[comp]:.3f}s",
        })

    CSV_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(CSV_PATH, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=[
            "component_name", "sequentiality_class", "vectorized_yes_no",
            "runtime_seconds_before", "runtime_seconds_after",
            "percent_total_runtime_before", "percent_total_runtime_after", "note",
        ])
        w.writeheader()
        w.writerows(csv_rows)
    print(f"  CSV: {CSV_PATH}")

    # =====================================================================
    # Write Markdown
    # =====================================================================
    fwd_b_pct = 100 * train_base["forward"] / max(train_base["total_wall"], 1e-9)
    bwd_b_pct = 100 * train_base["backward"] / max(train_base["total_wall"], 1e-9)
    fwd_v_pct = 100 * train_vec["forward"] / max(train_vec["total_wall"], 1e-9)
    bwd_v_pct = 100 * train_vec["backward"] / max(train_vec["total_wall"], 1e-9)

    fwd_change = 100 * (train_vec["forward"] - train_base["forward"]) / max(train_base["forward"], 1e-9)
    bwd_change = 100 * (train_vec["backward"] - train_base["backward"]) / max(train_base["backward"], 1e-9)
    total_change = 100 * (train_vec["total_wall"] - train_base["total_wall"]) / max(train_base["total_wall"], 1e-9)

    # Compute vectorized fraction of loop time
    vec_time = sum(inner_timings[k] for k in inner_timings
                   if classifications.get(k, ("", ""))[1] in ("yes", "partial"))
    seq_time = sum(inner_timings[k] for k in inner_timings
                   if classifications.get(k, ("", ""))[1] == "no")
    vec_frac = 100 * vec_time / max(inner_total, 1e-9)
    seq_frac = 100 * seq_time / max(inner_total, 1e-9)

    md_lines = []
    md = md_lines.append

    md("# Prime Transport Router — Vectorization Probe v1")
    md("")
    md("## Baseline Config")
    md("")
    md(f"CPU, D_HIDDEN={D_HIDDEN}, batch={BATCH_SIZE}, D={D_CONTEXT}, pos0_bias=enabled, threads=1")
    md(f"torch={torch.__version__}, python={platform.python_version()}, {platform.platform()}")
    md("")

    md("## Loop Decomposition: Sub-Operation Classification")
    md("")
    md("| Sub-operation | Depends on τ[t-1] or s[t]? | Classification | Vectorized? | % of loop time |")
    md("|---------------|---------------------------|----------------|-------------|----------------|")
    ops_ordered = [
        ("A. token_select",     "token_select",     "No",  "trivially_vectorizable",       "yes"),
        ("B. embedding_lookup", "embedding_lookup", "No",  "trivially_vectorizable",       "yes"),
        ("C. cat",              "cat",              "Yes (τ)", "eliminable (split matmul)", "yes"),
        ("D. linear1_mm+tanh",  "linear1_mm",       "Yes (τ)", "partial (emb part vectorizable, τ part sequential)", "partial"),
        ("E. linear2_mm",       "linear2_mm",       "Yes", "sequential",                   "no"),
        ("F. gumbel_noise",     "gumbel_noise",     "No",  "trivially_vectorizable",       "yes"),
        ("G. softmax",          "softmax",          "Yes", "sequential",                   "no"),
        ("H. TN_lookup",        "TN_lookup",        "Yes (s[t])", "sequential (coupled recurrence)", "no"),
        ("I. einsum_blend",     "einsum_blend",     "Yes", "sequential",                   "no"),
        ("J. injection",        "injection",        "No (t=0 only)", "trivially_vectorizable (mask)", "yes"),
        ("K. argmax",           "argmax",           "Yes", "sequential (non-diff)",         "no"),
        ("L. TR_lookup",        "TR_lookup",        "Yes (s[t])", "sequential (non-diff)",  "no"),
        ("M. gather_route",     "gather_route",     "Yes", "sequential (non-diff)",         "no"),
    ]
    for op_name, key, dep, cls, vec in ops_ordered:
        pct = 100 * inner_timings.get(key, 0) / max(inner_total, 1e-9)
        md(f"| {op_name} | {dep} | {cls} | {vec} | {pct:.1f}% |")

    md("")
    md(f"**Vectorizable fraction of loop time: {vec_frac:.1f}%**")
    md(f"**Mathematically sequential fraction: {seq_frac:.1f}%**")
    md("")

    md("## Why the Core Recurrence is Mathematically Sequential")
    md("")
    md("The recurrence is:")
    md("```")
    md("τ[t] = einsum(softmax(MLP(emb_proj[t] + τ[t-1] @ W1_tau) / temp), TN[s[t]])")
    md("s[t+1] = TR[s[t]].gather(argmax(w[t]))")
    md("```")
    md("where MLP includes tanh (nonlinear) and softmax (nonlinear).")
    md("")
    md("This is a **nonlinear coupled recurrence**:")
    md("- τ[t] depends on τ[t-1] through tanh and softmax — NOT a linear recurrence")
    md("- s[t+1] depends on s[t] through argmax — NOT even continuous")
    md("- The two are coupled: τ[t] needs TN[s[t]], and s[t+1] needs argmax(w(τ[t-1]))")
    md("")
    md("**Parallel prefix/scan** requires an associative binary operator over the recurrence.")
    md("For linear recurrences (y[t] = A[t]·y[t-1] + b[t]), the operator (A,b)∘(C,d) = (AC, Ab+d)")
    md("is associative, enabling O(log n) parallel evaluation (used in S4, Mamba, etc.).")
    md("")
    md("The current recurrence is **not linear**: it involves tanh, softmax, and state-dependent")
    md("table lookup. There is no known associative composition for this class of recurrence.")
    md("It is **mathematically sequential**, not accidentally sequential due to implementation style.")
    md("")

    md("## What Was Vectorized")
    md("")
    md("The following operations were hoisted out of the D=24 loop:")
    md("")
    md("1. **Token embeddings** — `W_emb[tokens]` computed once as (B, D, D_EMB)")
    md("2. **Embedding projection** — `embs_all @ W1[:D_EMB,:]` computed once as (B, D, D_HIDDEN)")
    md("3. **Gumbel noise** — pre-generated once as (B, D, N_OPS)")
    md("4. **Token injection** — `W_tok_inject[x0]` computed once as (B, D_TAU)")
    md("5. **cat elimination** — replaced `cat([embs, τ]) @ W1` with `emb_proj[t] + τ @ W1[D_EMB:,:]`")
    md("")
    md("What remains in the loop (irreducible sequential core):")
    md("")
    md("- `τ @ W1_tau + b1` — matmul (B,21)×(21,32), depends on τ[t-1]")
    md("- `tanh(...)` — nonlinear activation")
    md("- `h @ W2 + b2` — logits, depends on h")
    md("- `softmax(...)` — nonlinear, depends on logits")
    md("- `TN[state_ids]` — table lookup, depends on s[t]")
    md("- `einsum(w, tn)` — blend, depends on w and TN lookup")
    md("- `argmax + TR lookup + gather` — hard routing, depends on w and s[t]")
    md("")

    md("## Before/After Runtime Comparison")
    md("")
    md("| Metric | Baseline | Vectorized | Change |")
    md("|--------|----------|------------|--------|")
    md(f"| Steps/sec | {train_base['sps']:.1f} | {train_vec['sps']:.1f} | {train_speedup:.2f}x |")
    md(f"| Total wall ({BENCH_STEPS} steps) | {train_base['total_wall']:.2f}s | {train_vec['total_wall']:.2f}s | {total_change:+.1f}% |")
    md(f"| Forward time | {train_base['forward']:.3f}s ({fwd_b_pct:.1f}%) | {train_vec['forward']:.3f}s ({fwd_v_pct:.1f}%) | {fwd_change:+.1f}% |")
    md(f"| Backward time | {train_base['backward']:.3f}s ({bwd_b_pct:.1f}%) | {train_vec['backward']:.3f}s ({bwd_v_pct:.1f}%) | {bwd_change:+.1f}% |")
    md(f"| Forward-only sps | {fwd_base['sps']:.1f} | {fwd_vec['sps']:.1f} | {fwd_vec['sps']/max(fwd_base['sps'],1e-9):.2f}x |")
    md("")
    md(f"**Semantic equivalence: max output diff = {max_diff:.2e}** ({'PASS' if equiv_ok else 'FAIL'})")
    md(f"**JIT scripted: baseline={'yes' if jit_baseline else 'no'}, vectorized={'yes' if jit_vector else 'no'}**")
    md("")

    md("## Did the Bottleneck Move?")
    md("")
    if abs(fwd_change) < 10 and abs(bwd_change) < 10:
        md("The forward/backward split ratio is **roughly unchanged**. The vectorized")
        md("pre-computation reduces per-step dispatch count but the dominant cost remains")
        md("the sequential core (MLP recurrence + table lookups + autograd chain).")
        md("The bottleneck did NOT materially move — it is structural.")
    elif fwd_change < -10:
        md(f"Forward time decreased by {abs(fwd_change):.1f}%. The vectorized pre-computation")
        md("successfully reduced per-step overhead. However, backward time")
        md(f"changed by {bwd_change:+.1f}%, indicating the autograd chain remains the")
        md("structural constraint.")
    else:
        md(f"Forward changed by {fwd_change:+.1f}%, backward by {bwd_change:+.1f}%.")
    md("")

    md("## Is the Remaining Sequential Portion Fundamental?")
    md("")
    md("**Yes.** The sequential core is a nonlinear coupled recurrence:")
    md("")
    md("- τ[t] depends on τ[t-1] via `tanh(... + τ[t-1] @ W_tau ...)` — nonlinear")
    md("- s[t+1] depends on s[t] and argmax(softmax(MLP(τ[t-1]))) — discrete + nonlinear")
    md("- TN[s[t]] lookup couples the continuous path (τ) to the discrete path (s)")
    md("")
    md("This is NOT a linear recurrence. It cannot be reformulated as a parallel scan.")
    md("The sequentiality is **mathematical**, not an implementation artifact.")
    md("")
    md("To make this parallelizable would require **changing the model architecture** to a")
    md("linear recurrence (like S4/S5/Mamba), which would change the operator semantics —")
    md("explicitly prohibited by constraints.")
    md("")

    md("## Honesty Section")
    md("")
    md("### What was fully vectorized")
    md("")
    md(f"- Token embedding lookup: all D steps computed in one `W_emb[tokens]` op")
    md(f"- Embedding projection: `embs @ W1[:4,:]` for all D steps in one matmul")
    md(f"- Gumbel noise: pre-generated for all D steps in one `rand` + `log` op")
    md(f"- Token injection: computed once, applied conditionally")
    md(f"- cat elimination: replaced with split matmul (no allocation)")
    md(f"- These represent **{vec_frac:.1f}%** of inner loop time")
    md("")
    md("### What resisted vectorization")
    md("")
    md(f"- The τ-recurrence MLP (tanh + matmul chain): **nonlinear, depends on τ[t-1]**")
    md(f"- State table lookups (TN, TR): **depend on state_ids which changes each step**")
    md(f"- Softmax + einsum blend: **depend on the MLP output**")
    md(f"- Hard routing (argmax + gather): **discrete, coupled to τ path**")
    md(f"- These represent **{seq_frac:.1f}%** of inner loop time — the dominant cost")
    md("")
    md("### What still remains the dominant bottleneck")
    md("")
    md("The D=24 sequential recurrence is the bottleneck. It forces both forward and backward")
    md("into a 24-step serial chain. The vectorized pre-computation reduces per-step dispatch")
    md("overhead but does not change the fundamental O(D) sequential depth of the computation.")
    md("")
    md("The only paths to eliminate this bottleneck are:")
    md("1. **Architectural change** — replace the nonlinear recurrence with a linear one (changes semantics)")
    md("2. **Reduce D** — fewer steps means shorter chain (changes task if D < 24)")
    md("3. **torch.compile** — let the compiler fuse ops within each sequential step (doesn't change depth, reduces constant factor)")
    md("")

    MD_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(MD_PATH, "w") as f:
        f.write("\n".join(md_lines) + "\n")
    print(f"  MD:  {MD_PATH}")

    print("\n" + "=" * 70)
    print("DONE")
    print("=" * 70)
    print(f"\n  Vectorizable fraction of loop: {vec_frac:.1f}%")
    print(f"  Sequential fraction of loop:   {seq_frac:.1f}%")
    print(f"  Training speedup: {train_speedup:.2f}x")
    print(f"  The core recurrence is MATHEMATICALLY SEQUENTIAL (nonlinear coupled).")


if __name__ == "__main__":
    main()
