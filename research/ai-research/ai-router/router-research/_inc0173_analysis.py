#!/usr/bin/env python3
"""INC-0173: WikiText-2 Replication — Native Concentration on Second Dataset.

Identical setup to INC-0171 (PTB confirm), swapping corpus to WikiText-2.
Tests whether the native-concentration replacement result (HOPF within ~8-10%
PPL of BASELINE) survives a distribution shift under identical training conditions.

Conditions: BASELINE, HOPF, PERMUTED (+ DENSE quality ceiling)
Dataset: WikiText-2 (data/lm_proxy/raw/wikitext2/)
Hyperparameters: identical to INC-0171 (h128, K=64, 4000 steps, LR 3e-4)

Kill-list stage: Stage 6/7 close — Sparse Event-Driven Trainability + Hardware
  Efficiency Confirmation (end-to-end).

The gap being filled:
  All evidence from INC-0143 through INC-0170 is on the PPMI-SVD proxy with an
  EMA-based update rule (not backpropagation). "Replaces transformer-like
  computation" is currently a hypothesis. This experiment makes it concrete.

Architecture:
  - 2-layer transformer, hidden_dim=128, 4 attention heads
  - Vocabulary: PTB top-N tokens
  - Attention: standard multi-head self-attention (UNCHANGED across conditions)
  - FFN routing layer (THREE swappable conditions):
      BASELINE:    standard top-K learned gating (linear + softmax → K experts)
      HOPF-ROUTED: L2-norm(hidden) → Hopf angular sector → expert selection
                   (NO learned gate matrix; uses INC-0169 algorithm)
      PERMUTED:    same as HOPF-ROUTED but sector assignments randomly permuted
                   (structure-destroyed control; same parameter count as HOPF)
  - K=64 experts in all conditions

Success (KEEP):
  - HOPF val perplexity within 10% of BASELINE at matched training budget
  - Effective routing ops ≤ 75% of BASELINE at matched val perplexity
  - eff_ratio (HOPF vs DENSE) ≥ 1.5×
  - No expert collapse (all experts receive ≥ 5% load)

Falsification (KILL/REFINE):
  - HOPF fails to converge OR perplexity gap > 20%
  - Expert collapse (single expert > 80% load)
  - eff_ratio < 1.1× end-to-end (geometric advantage does not survive training)

Protocol: screen (1 seed) → confirm (2 seeds) → finalize (4 seeds).
This script runs the screen. Pass --seeds 0,1 for confirm, --seeds 0,1,2,3 for finalize.
"""

import argparse
import json
import math
import os
import sys
import time
from collections import Counter
from typing import Dict, List, Optional, Tuple

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

ROOT = os.path.dirname(os.path.abspath(__file__))
PTB_TRAIN = os.path.join(ROOT, "data/lm_proxy/raw/wikitext2/train.txt")
PTB_VALID = os.path.join(ROOT, "data/lm_proxy/raw/wikitext2/valid.txt")
OUTPUT_DIR = os.path.join(ROOT, "results/analysis")

# ---------------------------------------------------------------------------
# Hyperparameters
# ---------------------------------------------------------------------------
VOCAB_SIZE  = 5000     # top N tokens from PTB
SEQ_LEN     = 32
HIDDEN_DIM  = 128
N_HEADS     = 4
N_LAYERS    = 2
K_EXPERTS   = 64       # number of routing experts
EXPERT_DIM  = 128      # expert FFN hidden width
BATCH_SIZE  = 64
STEPS       = 4000     # screen: 4k steps; confirm/finalize: 8k
EVAL_EVERY  = 400
LR          = 3e-4
GRAD_CLIP   = 1.0
HOPF_DIMS   = (0, 1, 2, 3)  # first 4 dims of hidden state (after PCA; see note)
LAMBDA      = 1.0           # phase transport strength


# ---------------------------------------------------------------------------
# Data
# ---------------------------------------------------------------------------

def tokenize(path: str) -> List[str]:
    with open(path) as f:
        return f.read().replace("\n", " <eos> ").split()


def build_vocab(tokens: List[str], max_vocab: int) -> Dict[str, int]:
    counts = Counter(tokens)
    specials = {"<pad>", "<unk>"}
    top = [w for w, _ in counts.most_common(max_vocab) if w not in specials][:max_vocab - 2]
    vocab_list = ["<pad>", "<unk>"] + top
    return {w: i for i, w in enumerate(vocab_list)}


def encode(tokens: List[str], vocab: Dict[str, int]) -> torch.Tensor:
    unk = vocab["<unk>"]
    return torch.tensor([vocab.get(t, unk) for t in tokens], dtype=torch.long)


def make_batches(ids: torch.Tensor, seq_len: int, batch_size: int,
                 device: torch.device) -> Tuple[torch.Tensor, torch.Tensor]:
    total = (len(ids) - 1) // seq_len * seq_len
    ids = ids[:total + 1]
    x = ids[:-1].unfold(0, seq_len, seq_len)   # (N, seq_len)
    y = ids[1:].unfold(0, seq_len, seq_len)    # (N, seq_len)
    n_complete = min(len(x), len(y))
    x, y = x[:n_complete], y[:n_complete]
    n_batches = len(x) // batch_size
    x = x[:n_batches * batch_size].view(n_batches, batch_size, seq_len).to(device)
    y = y[:n_batches * batch_size].view(n_batches, batch_size, seq_len).to(device)
    return x, y


# ---------------------------------------------------------------------------
# Hopf routing (pure torch, same algorithm as hopf_routing_demo.py)
# ---------------------------------------------------------------------------

def _wrap_to_pi_torch(x: torch.Tensor) -> torch.Tensor:
    return (x + math.pi) % (2.0 * math.pi) - math.pi


def _allocate_bins(K: int, min_b: int = 2) -> Tuple[int, int, int]:
    """Find (k1,k2,k3) with k1×k2×k3 ≤ K, product maximised, spread minimised."""
    best: Tuple[int, int, int] = (1, K, 1)
    best_score = None
    for k1 in range(min_b, K + 1):
        for k2 in range(min_b, K + 1):
            max_k3 = K // max(k1 * k2, 1)
            if max_k3 < min_b:
                break
            for k3 in range(min_b, max_k3 + 1):
                prod = k1 * k2 * k3
                score = (prod, 1 if k2 >= k3 else 0,
                         -(abs(k1-k2)+abs(k2-k3)+abs(k1-k3)), k2, -k3)
                if best_score is None or score > best_score:
                    best_score = score
                    best = (k1, k2, k3)
    return best


_BIN_CACHE: Dict[int, Tuple[int, int, int]] = {}


def hopf_sectors(hidden: torch.Tensor, K: int,
                 dims: Tuple[int, int, int, int] = HOPF_DIMS,
                 lam: float = LAMBDA) -> torch.Tensor:
    """
    Assign each hidden vector to a Hopf angular sector.

    Parameters
    ----------
    hidden : (..., D) float tensor, L2-normalised
    K      : number of sectors
    dims   : 4-tuple of dimension indices
    lam    : phase transport lambda

    Returns
    -------
    sectors : (...,) long tensor in [0, K)
    """
    a = hidden[..., dims[0]]
    b = hidden[..., dims[1]]
    c = hidden[..., dims[2]]
    d = hidden[..., dims[3]]

    rho1 = torch.sqrt(a*a + b*b)
    rho2 = torch.sqrt(c*c + d*d)
    denom = torch.clamp(torch.sqrt(rho1**2 + rho2**2), min=1e-12)

    chi_u = torch.clamp((rho2 / denom)**2, 0.0, 1.0 - 1e-12)
    chi   = torch.asin(torch.clamp(rho2 / denom, 0.0, 1.0))

    theta1 = torch.atan2(b, a)
    theta2 = torch.atan2(d, c)
    delta  = _wrap_to_pi_torch(theta1 - theta2)
    alpha  = _wrap_to_pi_torch(0.5 * (theta1 + theta2))
    ta     = _wrap_to_pi_torch(alpha + 0.5 * lam * torch.cos(2.0 * chi) * delta)

    if K not in _BIN_CACHE:
        _BIN_CACHE[K] = _allocate_bins(K)
    kchi, kdelta, kalpha = _BIN_CACHE[K]

    bchi   = torch.clamp((chi_u * kchi).long(),   0, kchi   - 1)
    bdelta = torch.clamp(((delta  + math.pi) / (2*math.pi) * kdelta).long(), 0, kdelta - 1)
    balpha = torch.clamp(((ta     + math.pi) / (2*math.pi) * kalpha).long(), 0, kalpha - 1)

    return bchi * kdelta * kalpha + bdelta * kalpha + balpha


# ---------------------------------------------------------------------------
# Expert FFN
# ---------------------------------------------------------------------------

class ExpertFFN(nn.Module):
    def __init__(self, in_dim: int, hidden_dim: int, out_dim: int):
        super().__init__()
        self.fc1 = nn.Linear(in_dim, hidden_dim)
        self.fc2 = nn.Linear(hidden_dim, out_dim)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.fc2(F.gelu(self.fc1(x)))


# ---------------------------------------------------------------------------
# Three FFN routing variants
# ---------------------------------------------------------------------------

class BaselineMoEFFN(nn.Module):
    """Standard top-K learned gating (linear + softmax → K experts)."""

    def __init__(self, d_model: int, k_experts: int, expert_dim: int):
        super().__init__()
        self.k = k_experts
        self.gate = nn.Linear(d_model, k_experts, bias=False)
        self.experts = nn.ModuleList(
            [ExpertFFN(d_model, expert_dim, d_model) for _ in range(k_experts)]
        )

    def forward(self, x: torch.Tensor) -> Tuple[torch.Tensor, Dict]:
        shape = x.shape                    # (..., D)
        xf = x.view(-1, x.shape[-1])      # (N, D)

        weights = F.softmax(self.gate(xf), dim=-1)  # (N, K)
        top1 = weights.argmax(dim=-1)               # (N,) — hard routing (top-1)

        out = torch.zeros_like(xf)
        for k in range(self.k):
            mask = (top1 == k)
            if mask.any():
                out[mask] = self.experts[k](xf[mask])

        routing_ops = float(xf.shape[0])   # each token visits 1 expert
        counts = Counter(top1.detach().cpu().tolist())
        p = np.array([counts.get(k, 0) for k in range(self.k)], dtype=float)
        p_sum = p.sum()
        eff_b = float(np.exp(-np.sum((p/p_sum) * np.log(p/p_sum + 1e-30)))) if p_sum > 0 else 0.0
        return out.view(shape), {"routing_ops": routing_ops, "eff_buckets": eff_b,
                                 "sector_counts": counts}


class HopfRoutedFFN(nn.Module):
    """
    Hopf angular routing: L2-norm(hidden) → Hopf sector → expert.
    No learned gate matrix. Routing is purely geometric.
    """

    def __init__(self, d_model: int, k_experts: int, expert_dim: int,
                 perm_seed: Optional[int] = None):
        super().__init__()
        self.k = k_experts
        self.experts = nn.ModuleList(
            [ExpertFFN(d_model, expert_dim, d_model) for _ in range(k_experts)]
        )
        self.perm_seed = perm_seed
        self._perm: Optional[torch.Tensor] = None

    def _get_perm(self, device: torch.device) -> Optional[torch.Tensor]:
        if self.perm_seed is None:
            return None
        if self._perm is None:
            rng = np.random.RandomState(self.perm_seed)
            self._perm = torch.tensor(rng.permutation(self.k), dtype=torch.long,
                                      device=device)
        return self._perm.to(device)

    def forward(self, x: torch.Tensor) -> Tuple[torch.Tensor, Dict]:
        shape = x.shape
        xf = x.view(-1, x.shape[-1])

        # L2-normalise for routing (but pass original xf to experts)
        xn = F.normalize(xf, p=2, dim=-1)
        sectors = hopf_sectors(xn, self.k)            # (N,)

        perm = self._get_perm(xf.device)
        if perm is not None:
            sectors = perm[sectors]                   # permuted control

        out = torch.zeros_like(xf)
        for k in range(self.k):
            mask = (sectors == k)
            if mask.any():
                out[mask] = self.experts[k](xf[mask])

        routing_ops = float(xf.shape[0])
        counts = Counter(sectors.detach().cpu().tolist())
        p = np.array([counts.get(k, 0) for k in range(self.k)], dtype=float)
        p_sum = p.sum()
        eff_b = float(np.exp(-np.sum((p/p_sum) * np.log(p/p_sum + 1e-30)))) if p_sum > 0 else 0.0
        return out.view(shape), {"routing_ops": routing_ops, "eff_buckets": eff_b,
                                 "sector_counts": counts}


# ---------------------------------------------------------------------------
# Transformer block
# ---------------------------------------------------------------------------

class TransformerBlock(nn.Module):
    def __init__(self, d_model: int, n_heads: int, ffn: nn.Module):
        super().__init__()
        self.attn = nn.MultiheadAttention(d_model, n_heads, batch_first=True)
        self.norm1 = nn.LayerNorm(d_model)
        self.norm2 = nn.LayerNorm(d_model)
        self.ffn = ffn

    def forward(self, x: torch.Tensor,
                causal_mask: Optional[torch.Tensor] = None) -> Tuple[torch.Tensor, Dict]:
        # Self-attention
        attn_out, _ = self.attn(x, x, x, attn_mask=causal_mask, need_weights=False)
        x = self.norm1(x + attn_out)
        # FFN routing
        ffn_out, routing_info = self.ffn(x)
        x = self.norm2(x + ffn_out)
        return x, routing_info


class SmallTransformerLM(nn.Module):
    def __init__(self, vocab_size: int, d_model: int, n_heads: int,
                 n_layers: int, ffn_factory, seq_len: int):
        super().__init__()
        self.d_model = d_model
        self.seq_len = seq_len
        self.embed = nn.Embedding(vocab_size, d_model)
        self.pos   = nn.Embedding(seq_len, d_model)
        self.blocks = nn.ModuleList([
            TransformerBlock(d_model, n_heads, ffn_factory(layer_idx=i))
            for i in range(n_layers)
        ])
        self.head = nn.Linear(d_model, vocab_size, bias=False)
        self._causal_mask: Optional[torch.Tensor] = None

    def _get_causal_mask(self, T: int, device: torch.device) -> torch.Tensor:
        if self._causal_mask is None or self._causal_mask.shape[0] != T:
            mask = torch.triu(torch.full((T, T), float('-inf'), device=device), diagonal=1)
            self._causal_mask = mask
        return self._causal_mask.to(device)

    def forward(self, x: torch.Tensor) -> Tuple[torch.Tensor, List[Dict]]:
        B, T = x.shape
        positions = torch.arange(T, device=x.device).unsqueeze(0)
        h = self.embed(x) + self.pos(positions)
        mask = self._get_causal_mask(T, x.device)

        routing_infos = []
        for block in self.blocks:
            h, info = block(h, causal_mask=mask)
            routing_infos.append(info)

        logits = self.head(h)   # (B, T, V)
        return logits, routing_infos


# ---------------------------------------------------------------------------
# Routing metrics helpers
# ---------------------------------------------------------------------------

def aggregate_routing_info(info_list: List[List[Dict]]) -> Dict:
    """Aggregate routing metrics across layers and batches."""
    total_ops = 0.0
    total_eff = 0.0
    n = len(info_list) * len(info_list[0]) if info_list else 1
    for batch_infos in info_list:
        for layer_info in batch_infos:
            total_ops += layer_info["routing_ops"]
            total_eff += layer_info["eff_buckets"]
    return {
        "routing_ops": total_ops / n,
        "eff_buckets": total_eff / n,
    }


# ---------------------------------------------------------------------------
# Training + evaluation
# ---------------------------------------------------------------------------

def evaluate(model: SmallTransformerLM, x_batches: torch.Tensor,
             y_batches: torch.Tensor, max_batches: int = 50) -> Tuple[float, Dict]:
    model.eval()
    total_loss = 0.0
    n_batches = min(len(x_batches), max_batches)
    all_infos = []
    with torch.no_grad():
        for i in range(n_batches):
            logits, infos = model(x_batches[i])
            B, T, V = logits.shape
            loss = F.cross_entropy(logits.view(B*T, V), y_batches[i].view(B*T))
            total_loss += loss.item()
            all_infos.append(infos)
    ppl = math.exp(total_loss / n_batches)
    routing = aggregate_routing_info(all_infos)
    model.train()
    return ppl, routing


def run_condition(
    condition: str,
    vocab: Dict[str, int],
    train_x: torch.Tensor,
    train_y: torch.Tensor,
    val_x: torch.Tensor,
    val_y: torch.Tensor,
    device: torch.device,
    seed: int,
    steps: int,
) -> Dict:
    torch.manual_seed(seed)
    np.random.seed(seed)

    vocab_size = len(vocab)

    def ffn_factory(layer_idx: int) -> nn.Module:
        if condition == "BASELINE":
            return BaselineMoEFFN(HIDDEN_DIM, K_EXPERTS, EXPERT_DIM)
        elif condition == "HOPF":
            return HopfRoutedFFN(HIDDEN_DIM, K_EXPERTS, EXPERT_DIM)
        elif condition == "PERMUTED":
            return HopfRoutedFFN(HIDDEN_DIM, K_EXPERTS, EXPERT_DIM,
                                 perm_seed=seed + layer_idx * 100)
        raise ValueError(f"Unknown condition: {condition}")

    model = SmallTransformerLM(
        vocab_size=vocab_size,
        d_model=HIDDEN_DIM,
        n_heads=N_HEADS,
        n_layers=N_LAYERS,
        ffn_factory=ffn_factory,
        seq_len=SEQ_LEN,
    ).to(device)

    n_params = sum(p.numel() for p in model.parameters() if p.requires_grad)
    print(f"  [{condition}] params={n_params:,}  device={device}")

    optimizer = torch.optim.Adam(model.parameters(), lr=LR)
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=steps)

    log = []
    step = 0
    epoch = 0
    n_train = len(train_x)

    model.train()
    while step < steps:
        epoch += 1
        perm = torch.randperm(n_train)
        for bi in perm:
            if step >= steps:
                break
            xb, yb = train_x[bi].to(device), train_y[bi].to(device)
            logits, _ = model(xb)
            B, T, V = logits.shape
            loss = F.cross_entropy(logits.view(B*T, V), yb.view(B*T))
            optimizer.zero_grad()
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), GRAD_CLIP)
            optimizer.step()
            scheduler.step()
            step += 1

            if step % EVAL_EVERY == 0 or step == steps:
                val_ppl, val_routing = evaluate(model, val_x, val_y)
                print(f"    step={step:5d}  val_ppl={val_ppl:.2f}  "
                      f"eff_b={val_routing['eff_buckets']:.2f}  "
                      f"ops={val_routing['routing_ops']:.1f}")
                log.append({
                    "step": step,
                    "val_ppl": val_ppl,
                    "eff_buckets": val_routing["eff_buckets"],
                    "routing_ops": val_routing["routing_ops"],
                })

    final_ppl = log[-1]["val_ppl"] if log else float("nan")
    final_eff = log[-1]["eff_buckets"] if log else float("nan")
    return {
        "condition": condition,
        "seed": seed,
        "n_params": n_params,
        "final_val_ppl": final_ppl,
        "final_eff_buckets": final_eff,
        "log": log,
    }


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seeds", type=str, default="0",
                    help="Comma-separated seed list (e.g. 0 for screen, 0,1 for confirm)")
    ap.add_argument("--conditions", type=str, default="BASELINE,HOPF,PERMUTED")
    ap.add_argument("--steps", type=int, default=STEPS)
    ap.add_argument("--device", type=str, default="cpu")
    ap.add_argument("--output", type=str,
                    default=os.path.join(OUTPUT_DIR, "inc0173_wt2_replication.json"))
    args = ap.parse_args()

    seeds = [int(s.strip()) for s in args.seeds.split(",")]
    conditions = [c.strip() for c in args.conditions.split(",")]
    device = torch.device(args.device)

    print("=" * 72)
    print("INC-0173: WT2 REPLICATION — HOPF vs BASELINE vs PERMUTED (SECOND DATASET)")
    print("=" * 72)
    print(f"  Conditions: {conditions}")
    print(f"  Seeds:      {seeds}")
    print(f"  Steps:      {args.steps}")
    print(f"  Device:     {device}")
    print(f"  K experts:  {K_EXPERTS}")
    print(f"  Arch:       {N_LAYERS}L × h{HIDDEN_DIM} × {N_HEADS}h")
    print()

    # Load PTB
    print(f"Loading WikiText-2: {PTB_TRAIN}")
    train_tokens = tokenize(PTB_TRAIN)
    valid_tokens = tokenize(PTB_VALID)
    vocab = build_vocab(train_tokens, VOCAB_SIZE)
    print(f"  Vocab: {len(vocab)}  Train tokens: {len(train_tokens):,}  "
          f"Val tokens: {len(valid_tokens):,}")

    train_ids = encode(train_tokens, vocab)
    valid_ids = encode(valid_tokens, vocab)
    train_x, train_y = make_batches(train_ids, SEQ_LEN, BATCH_SIZE, device)
    val_x, val_y     = make_batches(valid_ids,  SEQ_LEN, BATCH_SIZE, device)
    print(f"  Train batches: {len(train_x)}  Val batches: {len(val_x)}")
    print()

    all_results = []

    for condition in conditions:
        print(f"\n{'='*40}")
        print(f"CONDITION: {condition}")
        print(f"{'='*40}")
        seed_results = []
        for seed in seeds:
            print(f"\n  Seed {seed}:")
            t0 = time.time()
            result = run_condition(
                condition=condition,
                vocab=vocab,
                train_x=train_x,
                train_y=train_y,
                val_x=val_x,
                val_y=val_y,
                device=device,
                seed=seed,
                steps=args.steps,
            )
            result["elapsed_s"] = time.time() - t0
            seed_results.append(result)
            print(f"  → val_ppl={result['final_val_ppl']:.2f}  "
                  f"eff_b={result['final_eff_buckets']:.2f}  "
                  f"elapsed={result['elapsed_s']:.0f}s")
        all_results.extend(seed_results)

    # Summary
    print("\n" + "=" * 72)
    print("SUMMARY")
    print("=" * 72)
    print(f"\n{'Condition':12s}  {'Seeds':6s}  {'Val PPL':8s}  "
          f"{'eff_b':8s}  {'ops':6s}")
    print("-" * 55)

    baseline_ppl = None
    by_condition = {}
    for r in all_results:
        c = r["condition"]
        if c not in by_condition:
            by_condition[c] = []
        by_condition[c].append(r)

    for c in conditions:
        results = by_condition.get(c, [])
        ppls = [r["final_val_ppl"] for r in results]
        effs = [r["final_eff_buckets"] for r in results]
        mean_ppl = float(np.mean(ppls))
        mean_eff = float(np.mean(effs))
        mean_ops = K_EXPERTS  # DENSE baseline ops = K per token
        if c == "BASELINE":
            baseline_ppl = mean_ppl
        print(f"{c:12s}  {len(results):6d}  {mean_ppl:8.2f}  {mean_eff:8.2f}  "
              f"{mean_ops:6.0f}")

    # Verdict
    baseline_results = by_condition.get("BASELINE", [])
    hopf_results = by_condition.get("HOPF", [])

    if baseline_results and hopf_results:
        print()
        bl_ppl = float(np.mean([r["final_val_ppl"] for r in baseline_results]))
        hp_ppl = float(np.mean([r["final_val_ppl"] for r in hopf_results]))
        bl_eff = float(np.mean([r["final_eff_buckets"] for r in baseline_results]))
        hp_eff = float(np.mean([r["final_eff_buckets"] for r in hopf_results]))

        ppl_ratio = hp_ppl / bl_ppl
        eff_ratio = bl_eff / max(hp_eff, 1e-9)  # BASELINE / HOPF = how many more ops baseline uses
        eff_ratio_vs_dense = K_EXPERTS / max(hp_eff, 1e-9)

        ppl_ok     = ppl_ratio <= 1.10
        eff_ok     = eff_ratio >= 1.3   # HOPF uses 1.3× fewer effective ops than BASELINE
        dense_ok   = eff_ratio_vs_dense >= 1.5

        print(f"  HOPF / BASELINE val PPL ratio:  {ppl_ratio:.3f}  "
              f"({'OK' if ppl_ok else 'FAIL'} — threshold ≤ 1.10)")
        print(f"  BASELINE / HOPF eff_ratio:      {eff_ratio:.3f}×  "
              f"({'OK' if eff_ok else 'FAIL'} — threshold ≥ 1.3×)")
        print(f"  DENSE / HOPF eff_ratio:         {eff_ratio_vs_dense:.3f}×  "
              f"({'OK' if dense_ok else 'FAIL'} — threshold ≥ 1.5×)")

        if ppl_ok and dense_ok:
            verdict = "KEEP"
        elif ppl_ratio > 1.20:
            verdict = "KILL"
        else:
            verdict = "REFINE"

        print(f"\n  VERDICT: {verdict}")
    else:
        verdict = "INCOMPLETE"
        eff_ratio = float("nan")
        eff_ratio_vs_dense = float("nan")
        ppl_ratio = float("nan")

    # Save
    output = {
        "increment": "INC-0173",
        "verdict": verdict,
        "config": {
            "vocab_size": VOCAB_SIZE,
            "seq_len": SEQ_LEN,
            "hidden_dim": HIDDEN_DIM,
            "n_heads": N_HEADS,
            "n_layers": N_LAYERS,
            "k_experts": K_EXPERTS,
            "expert_dim": EXPERT_DIM,
            "steps": args.steps,
            "seeds": seeds,
            "conditions": conditions,
        },
        "summary": {
            c: {
                "mean_val_ppl": float(np.mean([r["final_val_ppl"] for r in by_condition.get(c, [])])),
                "mean_eff_buckets": float(np.mean([r["final_eff_buckets"] for r in by_condition.get(c, [])])),
                "n_seeds": len(by_condition.get(c, [])),
            }
            for c in conditions
        },
        "ppl_ratio_hopf_vs_baseline": float(ppl_ratio),
        "eff_ratio_baseline_vs_hopf": float(eff_ratio),
        "eff_ratio_dense_vs_hopf": float(eff_ratio_vs_dense),
        "results": all_results,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w") as f:
        json.dump(output, f, indent=2)
    print(f"\nResults saved: {args.output}")
    print("=" * 72)


if __name__ == "__main__":
    main()
