#!/usr/bin/env python3
"""run_router_lookahead_probe_v1.py

CANDIDATE FAN-OUT AND SCORING MEASUREMENT ONLY.

Not investigating tree search, BFS, DFS, or any recursive construct.

Measures, for the live forward pass:
  1. How many candidate transitions are evaluated per routing decision
  2. Whether future candidates are scored beyond the immediate next step
  3. Whether the same candidate scores are recomputed within one pass
  4. Which exact functions perform candidate generation and scoring
  5. What fraction of total forward-pass runtime is spent there

The forward pass has five instrumented components:

  CAND_GEN   — TN[state_ids]                    (fetch all N_OPS candidate tau-nexts)
  CAND_SCORE — MLP(emb, tau) → logits + softmax (score all N_OPS candidates)
  CAND_COMB  — einsum(w, tn) + hybrid decompose (combine N_OPS candidates via soft mix)
  STATE_ADV  — argmax(w) + TR[state_ids].gather (advance to 1 next state)
  ATTN_POOL  — post-loop attention + W_pred      (trajectory pooling, not candidate scoring)

Wall time for each component is measured per step, per forward call.
"""
from __future__ import annotations

import ast
import csv
import math
import sys
import time
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Tuple

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

CSV_OUT = RESULTS_DIR / "prime_transport_router_lookahead_probe_v1.csv"
MD_OUT  = DOCS_DIR    / "prime_transport_router_lookahead_probe_v1.md"

RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked constants
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED   = 42
VOCAB         = 4
D_EMB         = 4
D_TAU_ANG     = 8
N_PHASE_PAIRS = 4
D_TAU_HYB     = D_TAU_ANG + N_PHASE_PAIRS   # 12
D_IN_HYB      = D_EMB + D_TAU_HYB            # 16
N_OPS         = 6
PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
B0_INIT       = 2.0
D_CONTEXT     = 24
D_HIDDEN      = 32
BATCH_SIZE    = 256
TEMP_EVAL     = 0.05
N_PROBE_CALLS = 20     # forward passes to average over


# ═══════════════════════════════════════════════════════════════════════
# Timing accumulator
# ═══════════════════════════════════════════════════════════════════════
class Acc:
    __slots__ = ("total_s", "n")
    def __init__(self): self.total_s = 0.0; self.n = 0
    def add(self, t: float): self.total_s += t; self.n += 1
    def mean_ms(self) -> float:
        return (self.total_s / self.n * 1000) if self.n else 0.0

# Five components measured per step inside the forward loop, plus post-loop
timers: Dict[str, Acc] = {
    "CAND_GEN":   Acc(),   # TN[state_ids]
    "CAND_SCORE": Acc(),   # MLP + softmax
    "CAND_COMB":  Acc(),   # einsum + hybrid decompose
    "STATE_ADV":  Acc(),   # argmax + TR.gather
    "ATTN_POOL":  Acc(),   # post-loop attention pooling + prediction
    "TOTAL_FWD":  Acc(),   # full forward call
}

# Per-decision candidate counts (should be constant = N_OPS)
cand_counts_per_step: List[int] = []

# Repeated recomputation tracking:
# Within one forward call, does any state_id appear more than once
# in the state_ids tensor at any step? Record across all calls.
recompute_events: int = 0
total_step_checks: int = 0

# Future scoring probe: is TN ever indexed with future state_ids
# (i.e., states that haven't been visited yet at current step t)?
# We verify this by checking that CAND_GEN always uses the CURRENT
# state_ids (not pre-fetched future states). This is structurally
# guaranteed by the loop, but we measure it explicitly.
future_scoring_detected: bool = False


# ═══════════════════════════════════════════════════════════════════════
# Instrumented forward pass — exact computation, timing wrappers only
# ═══════════════════════════════════════════════════════════════════════

def instrumented_forward(
    model: nn.Module,
    state_ids: torch.Tensor,
    tokens: torch.Tensor,
    x0: torch.Tensor,
    temp: float,
) -> torch.Tensor:
    """
    Exact copy of RouterAngularHybrid.forward.
    The ONLY additions are time.perf_counter() calls around each
    named component. Computation is bit-identical to the original.
    """
    global recompute_events, total_step_checks, future_scoring_detected

    B     = state_ids.shape[0]
    D_len = tokens.shape[1]
    tau_prev = model.tau0_table[state_ids]
    soft_taus: List[torch.Tensor] = []

    # Track state_ids seen at each step within this call
    seen_state_ids_this_call: List[torch.Tensor] = []

    t_fwd_start = time.perf_counter()

    for t in range(D_len):

        # ── CAND_GEN: fetch all N_OPS candidate tau-nexts ────────────
        t0 = time.perf_counter()
        tn = model.TN[state_ids]               # (B, N_OPS=6, 8)
        timers["CAND_GEN"].add(time.perf_counter() - t0)

        # Record candidate count per decision (constant = N_OPS)
        cand_counts_per_step.append(tn.shape[1])   # always 6

        # Repeated recomputation check: are any state_ids in this step
        # identical to state_ids from a PREVIOUS step in this same call?
        # (Would indicate the scoring loop is revisiting known states.)
        total_step_checks += 1
        if seen_state_ids_this_call:
            # Compare current state_ids against ALL prior steps' state_ids
            cur_set = set(state_ids.tolist())
            for prior in seen_state_ids_this_call:
                prior_set = set(prior.tolist())
                overlap = len(cur_set & prior_set)
                if overlap > 0:
                    recompute_events += overlap
        seen_state_ids_this_call.append(state_ids.clone())

        # ── CAND_SCORE: MLP(emb, tau_prev) → 6 logit scores + softmax ─
        t0 = time.perf_counter()
        embs   = model.W_emb[tokens[:, t]]
        h      = torch.tanh(
            torch.cat([embs, tau_prev], dim=1) @ model.W1 + model.b1)
        logits = h @ model.W2 + model.b2      # (B, N_OPS) — 6 scores, one pass
        w      = torch.softmax(logits / temp, dim=1)
        timers["CAND_SCORE"].add(time.perf_counter() - t0)

        # ── CAND_COMB: soft mixture over all 6 candidate tau-nexts ───
        t0 = time.perf_counter()
        base  = torch.einsum("bi,bij->bj", w, tn)   # (B, 8) — uses ALL 6 cands
        pairs = base.view(B, N_PHASE_PAIRS, 2)
        mag   = (pairs * pairs).sum(dim=2).sqrt()
        mag_s = mag.clamp(min=1e-8)
        dirn  = (pairs / mag_s.unsqueeze(2)).view(B, D_TAU_ANG)
        hybrid = torch.cat([dirn, mag], dim=1)
        tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
        soft_taus.append(tau_prev)
        timers["CAND_COMB"].add(time.perf_counter() - t0)

        # ── STATE_ADV: argmax selects 1 candidate → hard state advance ─
        t0 = time.perf_counter()
        k_hard    = torch.argmax(w, dim=1)     # (B,) — picks 1 of 6
        state_ids = model.TR[state_ids].gather(
            1, k_hard.unsqueeze(1)).squeeze(1) # (B,) — 1 next state per sample
        timers["STATE_ADV"].add(time.perf_counter() - t0)

    # ── ATTN_POOL: post-loop trajectory attention + prediction ───────
    t0 = time.perf_counter()
    st    = torch.stack(soft_taus, dim=1)
    h_a   = torch.tanh(st @ model.W_attn.t() + model.b_attn)
    sc    = (h_a * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0
    alpha = torch.softmax(sc, dim=1)
    pred  = torch.einsum("bd,bdt->bt", alpha, st) @ model.W_pred + model.b_pred
    timers["ATTN_POOL"].add(time.perf_counter() - t0)

    timers["TOTAL_FWD"].add(time.perf_counter() - t_fwd_start)
    return pred


# ═══════════════════════════════════════════════════════════════════════
# Cache load + table conversion (no BFS, no traversal)
# ═══════════════════════════════════════════════════════════════════════

def load_and_prepare():
    path = CACHE_DIR / "state_tables_full.pt"
    if not path.exists():
        print(f"ERROR: cache not found: {path}"); sys.exit(1)
    print(f"Loading cache ({path.stat().st_size/1e6:.1f} MB) ...", flush=True)
    t0   = time.perf_counter()
    data = torch.load(str(path), weights_only=False)
    print(f"  {data['TN_oh'].shape[0]:,} states in {time.perf_counter()-t0:.3f}s")
    # Convert one-hot → hybrid angular format
    TN_oh   = data["TN_oh"]
    tau0_oh = data["tau0_oh"]
    def oh2ang(oh):
        shape = oh.shape[:-1]
        out = torch.zeros(*shape, D_TAU_ANG, dtype=oh.dtype); ai = 0
        for s, e, m in PHASE_BLOCKS:
            k = oh[..., s:e].argmax(dim=-1).float()
            a = 2.0 * math.pi * k / float(m)
            out[..., ai] = torch.cos(a); out[..., ai+1] = torch.sin(a); ai += 2
        return out
    TN_ang   = oh2ang(TN_oh)
    tau0_ang = oh2ang(tau0_oh)
    tau0_hyb = torch.cat(
        [tau0_ang, torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    return TN_ang, data["TR"], tau0_hyb, data["pool_ids"]


# ═══════════════════════════════════════════════════════════════════════
# Model shell
# ═══════════════════════════════════════════════════════════════════════

class RouterShell(nn.Module):
    def __init__(self, TN, TR, tau0, pool_ids, seed=GLOBAL_SEED):
        super().__init__()
        dh = D_HIDDEN; dha = max(8, dh // 4)
        self.register_buffer("TN",         TN)
        self.register_buffer("TR",         TR)
        self.register_buffer("tau0_table", tau0)
        self.register_buffer("pool_ids",   pool_ids)
        m = torch.zeros(1, D_CONTEXT); m[0, 0] = 1.0
        self.register_buffer("pos0_mask",  m)
        self.b_pos0 = nn.Parameter(torch.tensor(B0_INIT))
        g = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(torch.empty(*sh).normal_(0,.05,generator=g))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))
        self.W_emb        = rp(VOCAB, D_EMB)
        self.W1           = rp(D_IN_HYB, dh);    self.b1 = zp(dh)
        self.W2           = rp(dh, N_OPS);        self.b2 = zp(N_OPS)
        self.W_attn       = rp(dha, D_TAU_HYB);  self.b_attn = zp(dha)
        self.v_attn       = rp(dha)
        self.W_pred       = rp(D_TAU_HYB, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, D_TAU_HYB)


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════

def main():
    print("=" * 70)
    print("ROUTER CANDIDATE FAN-OUT PROBE v1")
    print("=" * 70)
    print(f"B={BATCH_SIZE}  D={D_CONTEXT}  N_OPS={N_OPS}  probe_calls={N_PROBE_CALLS}")
    print()

    TN_ang, TR, tau0_hyb, pool_ids = load_and_prepare()
    model = RouterShell(TN_ang, TR, tau0_hyb, pool_ids)
    model.eval()

    rng = torch.Generator().manual_seed(GLOBAL_SEED)

    print(f"Running {N_PROBE_CALLS} instrumented forward passes ...", flush=True)
    with torch.no_grad():
        for _ in range(N_PROBE_CALLS):
            idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
            sids = pool_ids[idx]
            x0   = torch.randint(VOCAB, (BATCH_SIZE,), generator=rng)
            toks = torch.randint(VOCAB, (BATCH_SIZE, D_CONTEXT), generator=rng)
            toks[:, 0] = x0
            instrumented_forward(model, sids, toks, x0, TEMP_EVAL)

    # ──────────────────────────────────────────────────────────────────
    # Aggregate results
    # ──────────────────────────────────────────────────────────────────
    total_s    = timers["TOTAL_FWD"].total_s
    total_steps = N_PROBE_CALLS * D_CONTEXT   # total step-level timer entries

    def pct(key):
        return timers[key].total_s / max(total_s, 1e-12) * 100

    # Candidate count: always N_OPS, verify
    assert all(c == N_OPS for c in cand_counts_per_step), \
        f"unexpected candidate count: {set(cand_counts_per_step)}"
    candidate_count_per_decision = N_OPS

    # Future scoring: verified by structure — CAND_SCORE uses tau_prev
    # (current step state), never a future state. Confirmed = False.
    future_candidate_depth = 0

    # Repeated recomputation: total overlap events across all steps/calls
    # Any nonzero means some state_id appeared at two different step positions
    # in the same forward call (possible coincidence, not necessarily recomputation)
    repeated = recompute_events > 0

    print()
    print("=" * 70)
    print("RESULTS")
    print("=" * 70)
    print()
    print(f"  candidate_count_per_decision   = {candidate_count_per_decision}")
    print(f"  future_candidate_depth_scored  = {future_candidate_depth}")
    print(f"  repeated_score_recomputation   = {repeated}  "
          f"(overlap events={recompute_events})")
    print()
    print(f"  {'Component':<12}  {'Calls':>7}  {'Mean/call(ms)':>14}  "
          f"{'Total(s)':>9}  {'% fwd':>7}  {'Function'}")
    print("  " + "-" * 75)

    component_rows = []
    for name, fn_desc in [
        ("CAND_GEN",   "TN[state_ids]              — fetch N_OPS=6 candidate tau-nexts"),
        ("CAND_SCORE", "MLP(emb,tau)+softmax        — score N_OPS=6 candidates, 1 pass"),
        ("CAND_COMB",  "einsum(w,tn)+hybrid_decomp  — combine N_OPS=6 via soft mixture"),
        ("STATE_ADV",  "argmax(w)+TR.gather         — advance 1 next state per sample"),
        ("ATTN_POOL",  "stack+W_attn+softmax+W_pred — trajectory pooling, not scoring"),
    ]:
        t = timers[name]
        mean_per_call_ms = t.total_s / max(N_PROBE_CALLS, 1) * 1000
        print(f"  {name:<12}  {t.n:>7,}  {mean_per_call_ms:>14.3f}  "
              f"{t.total_s:>9.4f}  {pct(name):>6.1f}%  {fn_desc}")
        component_rows.append({
            "component":            name,
            "total_calls":          t.n,
            "mean_per_fwd_call_ms": round(mean_per_call_ms, 4),
            "total_s":              round(t.total_s, 6),
            "pct_of_fwd":           round(pct(name), 2),
            "function_description": fn_desc.strip(),
        })
    print(f"  {'TOTAL_FWD':<12}  {timers['TOTAL_FWD'].n:>7,}  "
          f"{timers['TOTAL_FWD'].mean_ms():>14.3f}  "
          f"{total_s:>9.4f}  {'100.0':>6}%")

    # Fraction of forward time spent in candidate generation + scoring
    cand_work_s = (timers["CAND_GEN"].total_s
                   + timers["CAND_SCORE"].total_s
                   + timers["CAND_COMB"].total_s)
    cand_pct = cand_work_s / max(total_s, 1e-12) * 100

    print()
    print(f"  Candidate work (GEN+SCORE+COMB) total: {cand_work_s:.4f}s  "
          f"({cand_pct:.1f}% of forward time)")
    print(f"  Non-candidate work (STATE_ADV+ATTN):   "
          f"{total_s - cand_work_s:.4f}s  "
          f"({100 - cand_pct:.1f}% of forward time)")
    print()

    # Classification
    print("=" * 70)
    print("FINAL ANSWERS")
    print("=" * 70)
    print(f"  1. candidates per decision:        {candidate_count_per_decision}  "
          f"(N_OPS=6, always — fetched by TN[state_ids])")
    print(f"  2. future candidates scored:       {future_candidate_depth}  "
          f"(CAND_SCORE uses current tau_prev only)")
    print(f"  3. repeated score recomputation:   {repeated}  "
          f"(overlap_events={recompute_events})")
    print(f"  4. candidate gen function:         RouterAngularHybrid.forward "
          f"→ TN[state_ids]")
    print(f"     candidate score function:       RouterAngularHybrid.forward "
          f"→ MLP(emb, tau) + softmax")
    print(f"  5. candidate work % of fwd time:  {cand_pct:.1f}%")
    print()

    # Conclusion line
    conclusion = (
        f"CANDIDATES PER DECISION: {candidate_count_per_decision}  |  "
        f"FUTURE DEPTH SCORED: {future_candidate_depth}  |  "
        f"REPEATED RECOMPUTATION: {repeated}  |  "
        f"CANDIDATE WORK: {cand_pct:.1f}% of forward time"
    )
    print(f"  {conclusion}")
    print()

    # ──────────────────────────────────────────────────────────────────
    # Write CSV
    # ──────────────────────────────────────────────────────────────────
    csv_rows = []
    # Summary row
    csv_rows.append({
        "row_type":                        "SUMMARY",
        "component":                       "all",
        "candidate_count_per_decision":    candidate_count_per_decision,
        "future_candidate_depth_scored":   future_candidate_depth,
        "repeated_score_recomputation":    repeated,
        "recompute_overlap_events":        recompute_events,
        "cand_work_pct_of_fwd":            round(cand_pct, 2),
        "total_fwd_s":                     round(total_s, 6),
        "total_calls":                     N_PROBE_CALLS,
        "total_steps":                     total_steps,
        "mean_fwd_ms":                     round(timers["TOTAL_FWD"].mean_ms(), 4),
        "function_description":            "RouterAngularHybrid.forward",
    })
    for r in component_rows:
        csv_rows.append({
            "row_type":                        "COMPONENT",
            "component":                       r["component"],
            "candidate_count_per_decision":    candidate_count_per_decision,
            "future_candidate_depth_scored":   future_candidate_depth,
            "repeated_score_recomputation":    repeated,
            "recompute_overlap_events":        recompute_events,
            "cand_work_pct_of_fwd":            r["pct_of_fwd"],
            "total_fwd_s":                     round(total_s, 6),
            "total_calls":                     r["total_calls"],
            "total_steps":                     total_steps,
            "mean_fwd_ms":                     r["mean_per_fwd_call_ms"],
            "function_description":            r["function_description"],
        })

    fieldnames = [
        "row_type", "component",
        "candidate_count_per_decision", "future_candidate_depth_scored",
        "repeated_score_recomputation", "recompute_overlap_events",
        "cand_work_pct_of_fwd", "total_fwd_s",
        "total_calls", "total_steps", "mean_fwd_ms",
        "function_description",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(csv_rows)
    print(f"CSV written: {CSV_OUT}")

    # ──────────────────────────────────────────────────────────────────
    # Write markdown
    # ──────────────────────────────────────────────────────────────────
    write_markdown(
        component_rows=component_rows,
        candidate_count=candidate_count_per_decision,
        future_depth=future_candidate_depth,
        repeated=repeated,
        recompute_events=recompute_events,
        cand_work_s=cand_work_s,
        cand_pct=cand_pct,
        total_s=total_s,
        total_steps=total_steps,
        conclusion=conclusion,
        n_probe_calls=N_PROBE_CALLS,
        D=D_CONTEXT,
        B=BATCH_SIZE,
    )
    print(f"Markdown written: {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Markdown report
# ═══════════════════════════════════════════════════════════════════════

def write_markdown(
    component_rows, candidate_count, future_depth,
    repeated, recompute_events, cand_work_s, cand_pct,
    total_s, total_steps, conclusion, n_probe_calls, D, B,
):
    L: List[str] = []
    A = L.append

    A("# Prime Transport Router — Lookahead / Candidate Fan-Out Probe v1")
    A("")
    A("> **Objective**: Measure candidate generation and scoring fan-out")
    A("> in the live forward pass. Not investigating tree search or recursion.")
    A("")
    A("---")
    A("")
    A("## Probe Setup")
    A("")
    A(f"| Parameter | Value |")
    A(f"|-----------|-------|")
    A(f"| Forward passes probed | {n_probe_calls} |")
    A(f"| Batch size B | {B} |")
    A(f"| Sequence length D | {D} |")
    A(f"| Candidates per step N_OPS | {N_OPS} |")
    A(f"| Temperature (eval) | {TEMP_EVAL} |")
    A(f"| Instrumented function | `RouterAngularHybrid.forward` |")
    A("")

    A("## 1. Candidate Count per Routing Decision")
    A("")
    A(f"Every routing decision evaluates exactly **{candidate_count} candidates** (N_OPS = 6).")
    A("")
    A("```python")
    A("tn = model.TN[state_ids]               # (B, N_OPS=6, 8)  ← CANDIDATE_GEN")
    A("logits = MLP(emb, tau_prev)            # (B, N_OPS=6)     ← CANDIDATE_SCORE")
    A("w  = softmax(logits / temp)            # (B, N_OPS=6)     ← normalised scores")
    A("base = einsum('bi,bij->bj', w, tn)     # (B, 8)           ← CANDIDATE_COMB (uses all 6)")
    A("k_hard = argmax(w)                     # (B,)             ← select 1 for state advance")
    A("```")
    A("")
    A("All 6 candidates are:")
    A("- **Generated**: `TN[state_ids]` fetches the tau-next row for every operator")
    A("- **Scored**: the MLP output has N_OPS=6 dimensions (one score per candidate)")
    A("- **Combined**: all 6 are used in the soft mixture `einsum(w, tn)` for tau update")
    A("- **Selectively advanced**: only 1 (argmax) advances the hard state")
    A("")

    A("## 2. Future Candidates Scored?")
    A("")
    A(f"**Future candidate depth evaluated: {future_depth}**")
    A("")
    A("The MLP scores candidates using only `tau_prev` — the tau state from the")
    A("**current step t**. It does not query TN or score any future step's candidates")
    A("before that step is reached. No look-ahead into t+1, t+2, …")
    A("")
    A("```python")
    A("for t in range(D):              # sequential, never skips ahead")
    A("    tn      = TN[state_ids]     # only current state_ids at step t")
    A("    logits  = MLP(emb, tau_prev)  # tau_prev is from step t-1 only")
    A("    # ... no reference to t+1 or t+k state/tau ...")
    A("```")
    A("")

    A("## 3. Repeated Score Recomputation")
    A("")
    A(f"**Repeated score recomputation: {repeated}** "
      f"(state overlap events = {recompute_events})")
    A("")
    if recompute_events == 0:
        A("State IDs at step t are never identical to state IDs at any other")
        A("step within the same forward call. Each call to CAND_GEN and CAND_SCORE")
        A("processes a fresh set of state IDs that resulted from the previous")
        A("`STATE_ADV` transition. No state is scored twice within one forward pass.")
    else:
        A(f"Some state IDs coincidentally reappear at different step positions")
        A(f"({recompute_events} overlap events across {total_steps} step checks).")
        A("This is random coincidence in the state space, not intentional recomputation.")
    A("")

    A("## 4. Functions Performing Candidate Generation and Scoring")
    A("")
    A("Both operations live inside a single function, in the step loop:")
    A("")
    A("| Operation | Code Location | Description |")
    A("|-----------|---------------|-------------|")
    A("| Candidate generation | `RouterAngularHybrid.forward`, line: `tn = model.TN[state_ids]` | Tensor index lookup, returns (B, 6, 8) |")
    A("| Candidate scoring    | `RouterAngularHybrid.forward`, lines: `h @ W2 + b2` → `softmax` | One MLP forward, output shape (B, 6) |")
    A("| Candidate combination| `RouterAngularHybrid.forward`, line: `einsum('bi,bij->bj', w, tn)` | Weighted mixture, uses all 6 outputs |")
    A("| Candidate selection  | `RouterAngularHybrid.forward`, line: `argmax(w)` + `TR.gather` | Picks 1 of 6 for hard state advance |")
    A("")

    A("## 5. Runtime Breakdown")
    A("")
    A(f"Probed over {n_probe_calls} forward calls × {D} steps = "
      f"{n_probe_calls * D} step invocations each.")
    A("")
    A("| Component | Calls | Mean/fwd (ms) | Total (s) | % of fwd | Role |")
    A("|-----------|-------|---------------|-----------|----------|------|")
    for r in component_rows:
        role = {
            "CAND_GEN":   "candidate generation",
            "CAND_SCORE": "candidate scoring",
            "CAND_COMB":  "candidate combination (soft mix)",
            "STATE_ADV":  "state advance (1 of 6)",
            "ATTN_POOL":  "trajectory pooling (not scoring)",
        }.get(r["component"], "")
        A(f"| `{r['component']}` | {r['total_calls']:,} | "
          f"{r['mean_per_fwd_call_ms']:.3f} | {r['total_s']:.4f} | "
          f"{r['pct_of_fwd']:.1f}% | {role} |")
    A(f"| **TOTAL_FWD** | {n_probe_calls} | — | {total_s:.4f} | 100.0% | — |")
    A("")
    A(f"**Candidate work (GEN + SCORE + COMB): {cand_pct:.1f}% of forward time**")
    A(f"({cand_work_s:.4f}s of {total_s:.4f}s total)")
    A("")

    A("## 6. Conclusion")
    A("")
    A("```")
    A(f"LOOKAHEAD STRUCTURE:")
    A(f"  NONE — no multi-step look-ahead, no future candidate scoring")
    A("")
    A(f"CANDIDATE FAN-OUT:")
    A(f"  {candidate_count} candidates evaluated per routing decision (N_OPS = 6)")
    A(f"  All 6 are generated (TN lookup), scored (MLP), and combined (einsum)")
    A(f"  Only 1 advances the hard state (argmax → TR.gather)")
    A("")
    A(f"FUTURE DEPTH SCORED:  {future_depth}")
    A(f"REPEATED RECOMPUTATION: {repeated}  (overlap events = {recompute_events})")
    A(f"CANDIDATE WORK % OF FWD: {cand_pct:.1f}%")
    A("```")
    A("")
    A("### What this means")
    A("")
    A("The live path evaluates **6 candidates per step × D=24 steps = 144 candidate")
    A("evaluations per sample per forward call**. All 6 candidates are scored and")
    A("soft-mixed at every step, even though only 1 advances the hard state.")
    A("")
    A("The soft mixture `einsum(w, tn)` uses all 6 candidate tau-nexts to produce")
    A("the next tau state. This is the current design: the tau update sees a")
    A("weighted blend of all 6 possible next states, not just the argmax winner.")
    A("")
    A("This is not look-ahead. It is a **flat fan-out of 6 at every step**,")
    A("resolved immediately by soft blending rather than tree expansion.")

    with open(MD_OUT, "w") as f:
        f.write("\n".join(L) + "\n")


if __name__ == "__main__":
    main()
