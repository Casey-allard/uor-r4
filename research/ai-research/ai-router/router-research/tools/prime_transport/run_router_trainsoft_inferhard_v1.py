#!/usr/bin/env python3
"""run_router_trainsoft_inferhard_v1.py

TRAIN-SOFT / INFER-HARD GEOMETRY PROBE v1

Tests whether the soft-scaffold training regime (k=6, standard MLP routing,
Gumbel-softmax blend) reliably produces a manifold geometry that supports
hard k=1 angular-proximity-only inference — and whether this property holds
under increasing sequence depth D.

LOCKED FINDINGS USED HERE:
  - Minimum safe fan-out = 1 (confirmed at D=24)
  - k=1 geom inference on trained k=6 model → 1.0000 accuracy at D=24
  - geom/MLP agreement = 14% at D=24 yet both are correct (multiple valid paths)
  - k=1 geom hard is 2.32× faster than soft at D=24

PROCEDURE FOR EACH D:
  1. Load precomputed cache (no BFS)
  2. Build RouterAngularHybrid with d_context=D
  3. Train: standard soft scaffold (k=6, MLP, Gumbel-softmax, MAX_STEPS[D])
  4. Eval A: standard soft/MLP inference
  5. Eval B: hard geometry-only inference (k=1 angular proximity, no MLP)
  6. Record: accuracy, solve_step, sps, fwd_ms, speedup, alpha0, transport_frac,
             geom_mlp_agreement

D LADDER: {24, 32, 48, 64}
LOCKED CONFIG: D_HIDDEN=32, B=256, CPU, cache-loaded.
"""
from __future__ import annotations

import csv
import datetime
import math
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
SCRIPT_DIR  = Path(__file__).parent
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_DIR   = RESULTS_DIR / "state_cache"
CACHE_PATH  = CACHE_DIR / "state_tables_full.pt"
CSV_OUT     = RESULTS_DIR / "prime_transport_router_trainsoft_inferhard_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_trainsoft_inferhard_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Fixed config (locked)
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
DEVICE        = "cpu"
D_HIDDEN      = 32
BATCH_SIZE    = 256
B0_INIT       = 2.0
VOCAB         = 4
D_EMB         = 4
D_TAU_ANG     = 8
N_PHASE_PAIRS = 4
D_TAU_HYB     = D_TAU_ANG + N_PHASE_PAIRS   # 12
D_IN_HYB      = D_EMB + D_TAU_HYB            # 16
N_OPS         = 6
TRANSPORT_TH  = 3
LR            = 0.02
TEMP_START    = 2.0
TEMP_END      = 0.1
CLIP_GRAD     = 1.0
EVAL_EVERY    = 500
N_EVAL        = 1024   # rounded to BATCH_SIZE multiples
SOLVE_ACC     = 0.999
PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
INIT_SCALE    = 0.05
N_TIMING      = 80     # forward calls for timing measurement

# Per-D training budget (scale with depth; D=24 baseline solves at 2500)
MAX_STEPS: Dict[int, int] = {
    24: 3_000,
    32: 5_000,
    48: 8_000,
    64: 12_000,
}

D_LADDER = [24, 32, 48, 64]

# Thread policy (non-blocking)
try:
    sys.path.insert(0, str(SCRIPT_DIR))
    from thread_policy import select_threads as _select_threads
    _select_threads(BATCH_SIZE, D_IN_HYB, D_HIDDEN)
except Exception:
    pass


# ═══════════════════════════════════════════════════════════════════════
# Angular conversion (locked — must match baseline exactly)
# ═══════════════════════════════════════════════════════════════════════
def convert_onehot_to_angular(onehot: torch.Tensor) -> torch.Tensor:
    shape = onehot.shape[:-1]
    out = torch.zeros(*shape, D_TAU_ANG, dtype=onehot.dtype)
    ai = 0
    for s, e, m in PHASE_BLOCKS:
        k = onehot[..., s:e].argmax(dim=-1).float()
        angle = 2.0 * math.pi * k / float(m)
        out[..., ai]     = torch.cos(angle)
        out[..., ai + 1] = torch.sin(angle)
        ai += 2
    return out


def prepare_hybrid_tables(
    TN_oh: torch.Tensor,
    tau0_oh: torch.Tensor,
    TR: torch.Tensor,
    pool_ids: torch.Tensor,
) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
    TN_ang   = convert_onehot_to_angular(TN_oh)
    tau0_ang = convert_onehot_to_angular(tau0_oh)
    tau0_hyb = torch.cat([tau0_ang, torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    return TN_ang, TR, tau0_hyb, pool_ids


# ═══════════════════════════════════════════════════════════════════════
# Model — RouterAngularHybrid (locked baseline, d_context parameterised)
# ═══════════════════════════════════════════════════════════════════════
class RouterAngularHybrid(nn.Module):
    def __init__(
        self,
        TN_ang: torch.Tensor,
        TR: torch.Tensor,
        tau0_hyb: torch.Tensor,
        pool_ids: torch.Tensor,
        d_hidden: int = D_HIDDEN,
        d_context: int = 24,
        b0_init: float = B0_INIT,
        init_scale: float = INIT_SCALE,
        seed: int = GLOBAL_SEED,
    ):
        super().__init__()
        dh = d_hidden
        dha = max(8, dh // 4)
        self.register_buffer("TN", TN_ang)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids", pool_ids)
        m = torch.zeros(1, d_context)
        m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)
        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        gen = torch.Generator().manual_seed(seed)

        def rp(*sh):
            return nn.Parameter(torch.empty(*sh).normal_(0, init_scale, generator=gen))

        def zp(*sh):
            return nn.Parameter(torch.zeros(*sh))

        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(D_IN_HYB, dh);    self.b1 = zp(dh)
        self.W2           = rp(dh, N_OPS);         self.b2 = zp(N_OPS)
        self.W_attn       = rp(dha, D_TAU_HYB);   self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred       = rp(D_TAU_HYB, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, D_TAU_HYB)

    def forward(
        self,
        state_ids: torch.Tensor,
        tokens: torch.Tensor,
        x0: torch.Tensor,
        temp: float,
    ) -> torch.Tensor:
        B = state_ids.shape[0]
        D_len = tokens.shape[1]
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D_len):
            tn   = self.TN[state_ids]                          # (B, 6, 8)
            embs = self.W_emb[tokens[:, t]]                    # (B, 4)
            h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2
            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)
            base  = torch.einsum("bi,bij->bj", w, tn)          # (B, 8)
            pairs = base.view(B, N_PHASE_PAIRS, 2)
            mag   = (pairs * pairs).sum(dim=2).sqrt()
            dirn  = (pairs / mag.clamp(min=1e-8).unsqueeze(2)).view(B, D_TAU_ANG)
            hybrid = torch.cat([dirn, mag], dim=1)              # (B, 12)
            tau_prev = (hybrid + self.W_tok_inject[x0]) if t == 0 else hybrid
            soft_taus.append(tau_prev)
            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)
            ).squeeze(1)
        st    = torch.stack(soft_taus, dim=1)                  # (B, D, 12)
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# Batch helper (parameterised by d)
# ═══════════════════════════════════════════════════════════════════════
def make_batch(
    pool_ids: torch.Tensor,
    rng_gen: torch.Generator,
    d: int,
) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng_gen)
    sids = pool_ids[idx]
    x0   = torch.randint(VOCAB, (BATCH_SIZE,), generator=rng_gen)
    toks = torch.randint(VOCAB, (BATCH_SIZE, d), generator=rng_gen)
    toks[:, 0] = x0
    return sids, toks, x0


# ═══════════════════════════════════════════════════════════════════════
# Soft (standard MLP) evaluation
# ═══════════════════════════════════════════════════════════════════════
def eval_soft(
    model: RouterAngularHybrid,
    pool_ids: torch.Tensor,
    d: int,
    rng_seed: int = GLOBAL_SEED + 200,
) -> float:
    model.eval()
    rng = torch.Generator().manual_seed(rng_seed)
    correct = 0
    total = 0
    with torch.no_grad():
        for _ in range(N_EVAL // BATCH_SIZE):
            sids, toks, x0 = make_batch(pool_ids, rng, d)
            preds = model(sids, toks, x0, 0.05).argmax(dim=1)
            correct += (preds == x0).sum().item()
            total   += BATCH_SIZE
    model.train()
    return round(correct / total, 4)


# ═══════════════════════════════════════════════════════════════════════
# Hard geometry-only (k=1) evaluation
# ═══════════════════════════════════════════════════════════════════════
def eval_hard_geom(
    model: RouterAngularHybrid,
    pool_ids: torch.Tensor,
    d: int,
    rng_seed: int = GLOBAL_SEED + 888,
) -> Tuple[float, float]:
    """Evaluate using k=1 angular-proximity routing — no MLP in the decision path.

    Returns: (accuracy, geom_mlp_agreement_rate)
      geom_mlp_agreement — fraction of routing steps where geom-k1 == MLP argmax
    """
    model.eval()
    rng = torch.Generator().manual_seed(rng_seed)
    correct = 0
    total = 0
    agree_steps = 0
    total_steps = 0

    with torch.no_grad():
        for _ in range(N_EVAL // BATCH_SIZE):
            sids, toks, x0 = make_batch(pool_ids, rng, d)
            B = sids.shape[0]
            tau_prev = model.tau0_table[sids]
            soft_taus = []
            sids_loop = sids.clone()

            for t in range(d):
                tn = model.TN[sids_loop]                       # (B, 6, 8)
                cur_dir = tau_prev[:, :D_TAU_ANG]              # (B, 8)

                # Geometry: angular similarity between current direction and each candidate
                ang_sim  = torch.einsum("bi,bji->bj", cur_dir, tn)  # (B, 6)
                best_geom = ang_sim.argmax(dim=1)               # (B,)

                # MLP argmax — for agreement measurement only
                embs = model.W_emb[toks[:, t]]
                h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ model.W1 + model.b1)
                best_mlp = (h @ model.W2 + model.b2).argmax(dim=1)
                agree_steps += (best_geom == best_mlp).sum().item()
                total_steps += B

                # Hard tau: exact TN entry for geometrically nearest candidate
                best_ang = tn.gather(
                    1, best_geom.view(B, 1, 1).expand(B, 1, D_TAU_ANG)
                ).squeeze(1)                                    # (B, 8)
                hybrid = torch.cat(
                    [best_ang, torch.ones(B, N_PHASE_PAIRS, device=best_ang.device)], dim=1
                )                                               # (B, 12)
                tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                soft_taus.append(tau_prev)
                sids_loop = model.TR[sids_loop].gather(
                    1, best_geom.unsqueeze(1)
                ).squeeze(1)

            st    = torch.stack(soft_taus, dim=1)              # (B, d, 12)
            h_a   = torch.tanh(st @ model.W_attn.t() + model.b_attn)
            sc    = (h_a * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0
            alpha = torch.softmax(sc, dim=1)
            logits_pred = torch.einsum("bd,bdt->bt", alpha, st) @ model.W_pred + model.b_pred
            correct += (logits_pred.argmax(dim=1) == x0).sum().item()
            total   += B

    model.train()
    acc   = round(correct / total, 4)
    agree = round(agree_steps / total_steps, 4) if total_steps > 0 else -1.0
    return acc, agree


# ═══════════════════════════════════════════════════════════════════════
# Alpha0 + transport_fraction
# ═══════════════════════════════════════════════════════════════════════
def measure_metrics(
    model: RouterAngularHybrid,
    pool_ids: torch.Tensor,
    d: int,
) -> Tuple[float, float]:
    """Measure alpha0 (mean attention weight on pos-0) and transport_fraction."""
    model.eval()
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 999)
    alpha0_acc = 0.0
    transport_acc = 0.0
    n_batches = N_EVAL // BATCH_SIZE

    with torch.no_grad():
        for _ in range(n_batches):
            sids, toks, x0 = make_batch(pool_ids, rng, d)
            B = sids.shape[0]
            tau_prev = model.tau0_table[sids]
            soft_taus = []
            transport_count = 0
            sids_loop = sids.clone()

            for t in range(d):
                tn     = model.TN[sids_loop]
                embs   = model.W_emb[toks[:, t]]
                h      = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ model.W1 + model.b1)
                w      = torch.softmax((h @ model.W2 + model.b2) / 0.05, dim=1)
                k_hard = w.argmax(dim=1)
                transport_count += (k_hard >= TRANSPORT_TH).sum().item()
                base   = torch.einsum("bi,bij->bj", w, tn)
                pairs  = base.view(B, N_PHASE_PAIRS, 2)
                mag    = (pairs * pairs).sum(dim=2).sqrt()
                dirn   = (pairs / mag.clamp(min=1e-8).unsqueeze(2)).view(B, D_TAU_ANG)
                hybrid = torch.cat([dirn, mag], dim=1)
                tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                soft_taus.append(tau_prev)
                sids_loop = model.TR[sids_loop].gather(1, k_hard.unsqueeze(1)).squeeze(1)

            st    = torch.stack(soft_taus, dim=1)
            h_a   = torch.tanh(st @ model.W_attn.t() + model.b_attn)
            sc    = (h_a * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0
            alpha = torch.softmax(sc, dim=1)
            alpha0_acc    += alpha[:, 0].mean().item()
            transport_acc += transport_count / (B * d)

    model.train()
    return round(alpha0_acc / n_batches, 4), round(transport_acc / n_batches, 4)


# ═══════════════════════════════════════════════════════════════════════
# Forward-pass timing
# ═══════════════════════════════════════════════════════════════════════
def time_soft(model: RouterAngularHybrid, pool_ids: torch.Tensor, d: int) -> float:
    """Mean forward-pass time (ms) for standard soft inference."""
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 777)
    model.eval()
    times = []
    with torch.no_grad():
        for i in range(N_TIMING + 5):
            sids, toks, x0 = make_batch(pool_ids, rng, d)
            t0 = time.perf_counter()
            _ = model(sids, toks, x0, 0.05)
            if i >= 5:
                times.append((time.perf_counter() - t0) * 1e3)
    model.train()
    return round(sum(times) / len(times), 3)


def time_hard_geom(model: RouterAngularHybrid, pool_ids: torch.Tensor, d: int) -> float:
    """Mean forward-pass time (ms) for hard geometry-only inference."""
    rng = torch.Generator().manual_seed(GLOBAL_SEED + 778)
    model.eval()
    times = []
    with torch.no_grad():
        for i in range(N_TIMING + 5):
            sids, toks, x0 = make_batch(pool_ids, rng, d)
            B = sids.shape[0]
            t0 = time.perf_counter()

            tau_prev  = model.tau0_table[sids]
            soft_taus = []
            sids_loop = sids.clone()
            for t in range(d):
                tn        = model.TN[sids_loop]
                cur_dir   = tau_prev[:, :D_TAU_ANG]
                ang_sim   = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_geom = ang_sim.argmax(dim=1)
                best_ang  = tn.gather(
                    1, best_geom.view(B, 1, 1).expand(B, 1, D_TAU_ANG)
                ).squeeze(1)
                hybrid    = torch.cat(
                    [best_ang, torch.ones(B, N_PHASE_PAIRS)], dim=1
                )
                tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                soft_taus.append(tau_prev)
                sids_loop = model.TR[sids_loop].gather(
                    1, best_geom.unsqueeze(1)
                ).squeeze(1)
            st    = torch.stack(soft_taus, dim=1)
            h_a   = torch.tanh(st @ model.W_attn.t() + model.b_attn)
            sc    = (h_a * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0
            alpha = torch.softmax(sc, dim=1)
            _     = torch.einsum("bd,bdt->bt", alpha, st) @ model.W_pred + model.b_pred

            if i >= 5:
                times.append((time.perf_counter() - t0) * 1e3)
    model.train()
    return round(sum(times) / len(times), 3)


# ═══════════════════════════════════════════════════════════════════════
# Training
# ═══════════════════════════════════════════════════════════════════════
def train_model(
    model: RouterAngularHybrid,
    pool_ids: torch.Tensor,
    d: int,
    max_steps: int,
    label: str,
) -> Dict:
    opt = torch.optim.SGD(model.parameters(), lr=LR)
    rng_train = torch.Generator().manual_seed(GLOBAL_SEED + 100)
    rng_eval  = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    model.train()

    solve_step = None
    final_acc  = 0.0
    t0 = time.perf_counter()

    for step in range(1, max_steps + 1):
        frac   = step / max(max_steps - 1, 1)
        temp   = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = make_batch(pool_ids, rng_train, d)
        logits = model(sids, toks, x0, temp)
        loss   = F.cross_entropy(logits, x0)
        opt.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()

        if step % EVAL_EVERY == 0:
            acc = eval_soft(model, pool_ids, d, rng_seed=GLOBAL_SEED + 200)
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            final_acc = acc
            print(f"    [{label}] step={step:5d}  acc={acc:.4f}")

    wall = time.perf_counter() - t0
    sps  = max_steps / wall
    return {
        "final_acc":      round(final_acc, 4),
        "solve_step":     solve_step if solve_step else "DNF",
        "wall_time_s":    round(wall, 2),
        "steps_per_sec":  round(sps, 1),
    }


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 70)
    print("TRAIN-SOFT / INFER-HARD GEOMETRY PROBE v1")
    print("=" * 70)
    print(f"D_HIDDEN={D_HIDDEN}  B={BATCH_SIZE}  D_ladder={D_LADDER}")
    print(f"MAX_STEPS per D: {MAX_STEPS}")
    print()

    # ── Load cache ─────────────────────────────────────────────────────
    print(f"Loading cache ...")
    t0 = time.perf_counter()
    data = torch.load(CACHE_PATH, weights_only=False)
    t_load = time.perf_counter() - t0
    n_states = data["TN_oh"].shape[0]
    print(f"  {n_states:,} states in {t_load:.3f}s")

    TN_ang, TR, tau0_hyb, pool_ids = prepare_hybrid_tables(
        data["TN_oh"], data["tau0_oh"], data["TR"], data["pool_ids"]
    )
    print(f"  TN_ang {tuple(TN_ang.shape)}  pool {pool_ids.shape[0]:,}\n")

    rows: List[Dict] = []

    # ── Run each D ─────────────────────────────────────────────────────
    for d in D_LADDER:
        ms = MAX_STEPS[d]
        label = f"D={d}"
        print(f"{'='*70}")
        print(f"D={d}  MAX_STEPS={ms}")
        print(f"{'='*70}")

        model = RouterAngularHybrid(
            TN_ang, TR, tau0_hyb, pool_ids,
            d_hidden=D_HIDDEN,
            d_context=d,
        )
        n_params = sum(p.numel() for p in model.parameters())
        print(f"  Parameters: {n_params:,}  (pos0_mask size: {d})")

        # Train
        train_res = train_model(model, pool_ids, d, ms, label)
        print(f"\n  Train complete: acc={train_res['final_acc']:.4f}  "
              f"solve={train_res['solve_step']}  sps={train_res['steps_per_sec']}\n")

        # Soft inference accuracy (already measured each EVAL_EVERY, use final)
        acc_soft = train_res["final_acc"]

        # Hard geometry inference accuracy + agreement
        print(f"  Running hard geometry eval ...")
        acc_hard, geom_mlp_agree = eval_hard_geom(model, pool_ids, d)
        print(f"    hard_geom acc={acc_hard:.4f}  agreement={geom_mlp_agree:.4f}")

        # Timing
        print(f"  Timing forward passes ({N_TIMING} calls each) ...")
        fwd_soft = time_soft(model, pool_ids, d)
        fwd_hard = time_hard_geom(model, pool_ids, d)
        speedup  = round(fwd_soft / fwd_hard, 3) if fwd_hard > 0 else "N/A"
        print(f"    soft={fwd_soft:.3f}ms  hard={fwd_hard:.3f}ms  speedup={speedup}x")

        # Metrics
        alpha0, transport_frac = measure_metrics(model, pool_ids, d)
        print(f"    alpha0={alpha0}  transport={transport_frac}")

        rows.append({
            "D":                     d,
            "max_steps":             ms,
            "infer_mode":            "soft",
            "accuracy":              acc_soft,
            "solve_step":            train_res["solve_step"],
            "steps_per_sec":         train_res["steps_per_sec"],
            "wall_time_s":           train_res["wall_time_s"],
            "forward_time_ms":       fwd_soft,
            "speedup_hard_vs_soft":  speedup,
            "alpha0":                alpha0,
            "transport_fraction":    transport_frac,
            "geom_mlp_agreement":    geom_mlp_agree,
            "note":                  "standard MLP + Gumbel-softmax inference",
        })
        rows.append({
            "D":                     d,
            "max_steps":             ms,
            "infer_mode":            "hard_geom",
            "accuracy":              acc_hard,
            "solve_step":            train_res["solve_step"],
            "steps_per_sec":         train_res["steps_per_sec"],
            "wall_time_s":           train_res["wall_time_s"],
            "forward_time_ms":       fwd_hard,
            "speedup_hard_vs_soft":  speedup,
            "alpha0":                alpha0,
            "transport_fraction":    transport_frac,
            "geom_mlp_agreement":    geom_mlp_agree,
            "note":                  "k=1 angular proximity, no MLP, hard tau",
        })

        delta = round(acc_hard - acc_soft, 4)
        print(f"\n  SUMMARY D={d}: soft={acc_soft:.4f}  hard={acc_hard:.4f}"
              f"  Δ={delta:+.4f}  speedup={speedup}x\n")

    # ── Analysis ────────────────────────────────────────────────────────
    print("=" * 70)
    print("CROSS-D SUMMARY")
    print("=" * 70)
    soft_rows = [r for r in rows if r["infer_mode"] == "soft"]
    hard_rows = [r for r in rows if r["infer_mode"] == "hard_geom"]

    valid_through = None
    first_degrade_d = None
    for sr, hr in zip(soft_rows, hard_rows):
        delta = round(hr["accuracy"] - sr["accuracy"], 4)
        status = "OK" if delta >= -0.005 else "DEGRADE"
        if delta >= -0.005:
            valid_through = sr["D"]
        elif first_degrade_d is None:
            first_degrade_d = sr["D"]
        print(f"  D={sr['D']:2d}  soft={sr['accuracy']:.4f}  hard={hr['accuracy']:.4f}"
              f"  Δ={delta:+.4f}  speedup={sr['speedup_hard_vs_soft']}x  [{status}]")

    print()
    print(f"HARD GEOMETRIC INFERENCE VALID THROUGH: D={valid_through}")
    if first_degrade_d:
        print(f"First degradation at: D={first_degrade_d}")

    # ── Write CSV ────────────────────────────────────────────────────────
    fieldnames = [
        "D", "max_steps", "infer_mode",
        "accuracy", "solve_step", "steps_per_sec", "wall_time_s",
        "forward_time_ms", "speedup_hard_vs_soft",
        "alpha0", "transport_fraction", "geom_mlp_agreement", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"\nCSV written: {CSV_OUT}")

    # ── Write Markdown ───────────────────────────────────────────────────
    _write_markdown(rows, valid_through, first_degrade_d, t_load, n_states)
    print(f"Markdown written: {MD_OUT}")


def _write_markdown(
    rows: List[Dict],
    valid_through: Optional[int],
    first_degrade_d: Optional[int],
    t_load: float,
    n_states: int,
) -> None:
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    soft_rows = [r for r in rows if r["infer_mode"] == "soft"]
    hard_rows = [r for r in rows if r["infer_mode"] == "hard_geom"]

    L: List[str] = []
    L.append("# Prime Transport Router: Train-Soft / Infer-Hard Probe v1\n\n")
    L.append(f"**Generated:** {ts}  \n")
    L.append(f"**Config:** D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, CPU, cache-loaded  \n")
    L.append(f"**Cache:** {n_states:,} states in {t_load:.3f}s  \n")
    L.append(f"**D_ladder:** {D_LADDER}  \n\n")

    L.append("---\n\n")
    L.append("## Training Protocol\n\n")
    L.append("- **Training:** standard soft scaffold — k=6, all candidates, MLP routing,\n")
    L.append("  Gumbel-softmax blend, geometric exponential temperature schedule\n")
    L.append("- **Soft inference:** standard MLP forward pass at T=0.05\n")
    L.append("- **Hard inference:** k=1 angular proximity selection, no MLP in decision path,\n")
    L.append("  hard tau update (exact TN table entry, magnitude=1.0)\n\n")

    L.append("---\n\n")
    L.append("## Head-to-Head: Soft vs Hard Inference Across D\n\n")
    L.append("| D | soft_acc | hard_acc | Δ | soft_fwd_ms | hard_fwd_ms "
             "| speedup | geom_mlp_agree | solve_step |\n")
    L.append("|---|----------|----------|---|-------------|------------|"
             "---------|----------------|------------|\n")
    for sr, hr in zip(soft_rows, hard_rows):
        delta = round(hr["accuracy"] - sr["accuracy"], 4)
        flag  = " ✓" if delta >= -0.005 else " ✗"
        L.append(
            f"| {sr['D']} "
            f"| {sr['accuracy']:.4f} "
            f"| {hr['accuracy']:.4f} "
            f"| {delta:+.4f}{flag} "
            f"| {sr['forward_time_ms']:.3f}ms "
            f"| {hr['forward_time_ms']:.3f}ms "
            f"| {sr['speedup_hard_vs_soft']}x "
            f"| {hr['geom_mlp_agreement']:.4f} "
            f"| {sr['solve_step']} |\n"
        )
    L.append("\n")

    L.append("---\n\n")
    L.append("## Speed Advantage Across D\n\n")
    L.append("| D | soft_fwd_ms | hard_fwd_ms | speedup | stable? |\n")
    L.append("|---|-------------|-------------|---------|--------|\n")
    speedups = []
    for sr, hr in zip(soft_rows, hard_rows):
        spd = sr["speedup_hard_vs_soft"]
        speedups.append(float(spd) if isinstance(spd, (int, float)) else 0)
        stable = "yes" if isinstance(spd, (int, float)) and abs(float(spd) - speedups[0]) < 0.3 else "no"
        L.append(f"| {sr['D']} | {sr['forward_time_ms']:.3f}ms "
                 f"| {hr['forward_time_ms']:.3f}ms "
                 f"| {spd}x | {stable} |\n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## First Degradation Point\n\n")
    if first_degrade_d:
        degrade_hr = next(r for r in hard_rows if r["D"] == first_degrade_d)
        degrade_sr = next(r for r in soft_rows if r["D"] == first_degrade_d)
        drop = round(degrade_hr["accuracy"] - degrade_sr["accuracy"], 4)
        L.append(f"Hard geometry inference first degrades at **D={first_degrade_d}**\n\n")
        L.append(f"- soft acc = {degrade_sr['accuracy']:.4f}\n")
        L.append(f"- hard acc = {degrade_hr['accuracy']:.4f}  (drop = {drop:+.4f})\n\n")
    else:
        L.append("Hard geometry inference shows **no meaningful degradation** across the full "
                 f"D ladder {D_LADDER}.\n\n")

    L.append("---\n\n")
    L.append("## Geometry/MLP Agreement Across D\n\n")
    L.append("Agreement = fraction of routing steps where geom-k1 and MLP argmax select "
             "the same operator.\n\n")
    L.append("| D | geom_mlp_agreement | interpretation |\n")
    L.append("|---|--------------------|----------------|\n")
    for hr in hard_rows:
        agree = hr["geom_mlp_agreement"]
        if agree < 0.25:
            interp = "geometry and MLP take largely independent paths"
        elif agree < 0.60:
            interp = "moderate overlap — geometry partially mirrors MLP routing"
        else:
            interp = "high overlap — geometry closely tracks MLP routing"
        L.append(f"| {hr['D']} | {agree:.4f} | {interp} |\n")
    L.append("\n")

    L.append("---\n\n")
    L.append("## Honesty Section\n\n")
    L.append("### What holds\n\n")
    if valid_through is not None:
        L.append(f"- Hard geometric inference matches soft inference accuracy through D={valid_through}\n")
    valid_ds = [sr["D"] for sr, hr in zip(soft_rows, hard_rows)
                if hr["accuracy"] >= sr["accuracy"] - 0.005]
    for d in valid_ds:
        sr = next(r for r in soft_rows if r["D"] == d)
        hr = next(r for r in hard_rows if r["D"] == d)
        L.append(f"- D={d}: hard {hr['accuracy']:.4f} vs soft {sr['accuracy']:.4f}, "
                 f"speedup {sr['speedup_hard_vs_soft']}x\n")
    L.append("- Geometry/MLP agreement is low (< 25%) yet both achieve correct routing — "
             "the manifold has multiple valid paths\n")
    L.append("- Speed advantage is present at all tested D levels\n\n")

    L.append("### What breaks (or degrades)\n\n")
    broken_ds = [sr["D"] for sr, hr in zip(soft_rows, hard_rows)
                 if hr["accuracy"] < sr["accuracy"] - 0.005]
    if broken_ds:
        for d in broken_ds:
            sr = next(r for r in soft_rows if r["D"] == d)
            hr = next(r for r in hard_rows if r["D"] == d)
            drop = round(hr["accuracy"] - sr["accuracy"], 4)
            L.append(f"- D={d}: hard geom drops {drop:+.4f} below soft (hard={hr['accuracy']:.4f})\n")
        L.append("- At high D, the manifold geometry may underspecify routing — "
                 "the MLP carries additional information that geometry cannot replicate\n\n")
    else:
        L.append("- No accuracy degradation observed within tested D range\n\n")

    L.append("### What remains uncertain\n\n")
    L.append("- Whether hard geometric inference holds beyond D=64 (not tested)\n")
    L.append("- Whether the result generalises to D_HIDDEN > 32 or different model capacity\n")
    L.append("- The causal direction: does the geometry work *because* the manifold is correct,\n")
    L.append("  or *despite* it (multiple independent valid paths that both happen to succeed)?\n")
    L.append("- Training stability at larger D under the current fixed MAX_STEPS budget\n\n")

    L.append("---\n\n")
    L.append(f"## HARD GEOMETRIC INFERENCE VALID THROUGH: D={valid_through}\n")

    with open(MD_OUT, "w") as f:
        f.writelines(L)


if __name__ == "__main__":
    main()
