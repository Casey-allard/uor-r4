#!/usr/bin/env python3
"""INC-0172: MoE Substitution Study — Angular Routing vs Learned Sparse Top-1.

Extends INC-0171 by adding:
  - LEARNED_SPARSE: learned sparse top-1 routing with load-balance auxiliary loss.
    This is a stronger comparison than INC-0171's BASELINE (which used no aux loss).
  - DENSE: standard 2-layer FFN (no routing; quality ceiling).

The key new question:
  Can fixed angular routing (HOPF) substitute for learned sparse top-1 routing with
  load-balance auxiliary loss, under matched training conditions?

New success condition vs INC-0171:
  ppl_ratio_hopf_vs_learned_sparse ≤ 1.10

Concentration guard (validates LEARNED_SPARSE is a meaningful baseline):
  LEARNED_SPARSE eff_b ≤ 0.90 * K at convergence.
  If this fails, LEARNED_SPARSE aux loss is pushing routing toward uniform distribution
  and is NOT a valid sparse routing baseline for comparison purposes.

Aux_coeff policy:
  Use aux_coeff=1e-2 (default). Only check 1e-3 or 3e-2 IF the concentration guard
  fails. Maximum 2 alternative values. 1 seed, 1000 steps only. No broad tuning.

Architecture: identical to INC-0171.
  2-layer transformer, hidden_dim=128, 4 heads, vocab=5000 (PTB), seq_len=32,
  K=64 experts, expert_dim=128, 4000 steps, cosine LR 3e-4, batch=64.

Conditions (default: all five):
  DENSE           standard 2-layer FFN, no routing (quality ceiling)
  BASELINE        learned top-1 gate, no aux loss (INC-0171 baseline; reuse for replication check)
  LEARNED_SPARSE  learned top-1 gate + load-balance auxiliary loss (new)
  HOPF            fixed Hopf angular routing (INC-0169 algorithm, no learned params)
  PERMUTED        fixed permuted routing (structure-destroyed null control)

Protocol: screen (1 seed) → confirm (2 seeds) → finalize (4 seeds) if warranted.
"""

import argparse
import json
import math
import os
from collections import Counter
from typing import Dict, List, Optional, Tuple

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

ROOT = os.path.dirname(os.path.abspath(__file__))
PTB_TRAIN = os.path.join(ROOT, "data/lm_proxy/raw/ptb/ptb.train.txt")
PTB_VALID = os.path.join(ROOT, "data/lm_proxy/raw/ptb/ptb.valid.txt")
OUTPUT_DIR = os.path.join(ROOT, "results/analysis")

# ---------------------------------------------------------------------------
# Hyperparameters (unchanged from INC-0171 for direct comparability)
# ---------------------------------------------------------------------------
VOCAB_SIZE  = 5000
SEQ_LEN     = 32
HIDDEN_DIM  = 128
N_HEADS     = 4
N_LAYERS    = 2
K_EXPERTS   = 64
EXPERT_DIM  = 128
BATCH_SIZE  = 64
STEPS       = 4000
EVAL_EVERY  = 400
LR          = 3e-4
GRAD_CLIP   = 1.0
HOPF_DIMS   = (0, 1, 2, 3)
LAMBDA      = 1.0
AUX_COEFF   = 1e-2   # load-balance auxiliary loss coefficient for LEARNED_SPARSE

# Concentration guard: LEARNED_SPARSE eff_b must be ≤ this fraction of K
CONCENTRATION_GUARD_FRAC = 0.90


# ---------------------------------------------------------------------------
# Data (unchanged from INC-0171)
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
    x = ids[:-1].unfold(0, seq_len, seq_len)
    y = ids[1:].unfold(0, seq_len, seq_len)
    n_complete = min(len(x), len(y))
    x, y = x[:n_complete], y[:n_complete]
    n_batches = len(x) // batch_size
    x = x[:n_batches * batch_size].view(n_batches, batch_size, seq_len).to(device)
    y = y[:n_batches * batch_size].view(n_batches, batch_size, seq_len).to(device)
    return x, y


# ---------------------------------------------------------------------------
# Hopf routing (unchanged from INC-0171 / INC-0169 algorithm)
# ---------------------------------------------------------------------------

def _wrap_to_pi_torch(x: torch.Tensor) -> torch.Tensor:
    return (x + math.pi) % (2.0 * math.pi) - math.pi


def _allocate_bins(K: int, min_b: int = 2) -> Tuple[int, int, int]:
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
# FFN modules
# ---------------------------------------------------------------------------

class ExpertFFN(nn.Module):
    def __init__(self, in_dim: int, hidden_dim: int, out_dim: int):
        super().__init__()
        self.fc1 = nn.Linear(in_dim, hidden_dim)
        self.fc2 = nn.Linear(hidden_dim, out_dim)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.fc2(F.relu(self.fc1(x)))


class DenseFFN(nn.Module):
    """Standard 2-layer FFN, no routing. Quality ceiling — every token uses full FFN."""

    def __init__(self, d_model: int, ffn_dim: int):
        super().__init__()
        self.fc1 = nn.Linear(d_model, ffn_dim)
        self.fc2 = nn.Linear(ffn_dim, d_model)

    def forward(self, x: torch.Tensor) -> Tuple[torch.Tensor, Dict]:
        out = self.fc2(F.relu(self.fc1(x)))
        # eff_buckets = nan (not applicable for dense FFN)
        return out, {"routing_ops": 0.0, "eff_buckets": float("nan"),
                     "is_dense": True}


class BaselineMoEFFN(nn.Module):
    """Learned top-1 gate, no auxiliary loss. Replicates INC-0171 BASELINE."""

    def __init__(self, d_model: int, k_experts: int, expert_dim: int):
        super().__init__()
        self.k = k_experts
        self.gate = nn.Linear(d_model, k_experts, bias=False)
        self.experts = nn.ModuleList(
            [ExpertFFN(d_model, expert_dim, d_model) for _ in range(k_experts)]
        )

    def forward(self, x: torch.Tensor) -> Tuple[torch.Tensor, Dict]:
        shape = x.shape
        xf = x.view(-1, x.shape[-1])
        weights = F.softmax(self.gate(xf), dim=-1)
        top1 = weights.argmax(dim=-1)
        out = torch.zeros_like(xf)
        for k in range(self.k):
            mask = (top1 == k)
            if mask.any():
                out[mask] = self.experts[k](xf[mask])
        counts = Counter(top1.detach().cpu().tolist())
        p = np.array([counts.get(k, 0) for k in range(self.k)], dtype=float)
        p_sum = p.sum()
        eff_b = float(np.exp(-np.sum((p/p_sum) * np.log(p/p_sum + 1e-30)))) if p_sum > 0 else 0.0
        return out.view(shape), {"routing_ops": float(xf.shape[0]),
                                  "eff_buckets": eff_b,
                                  "sector_counts": counts}


class LearnedSparseMoEFFN(nn.Module):
    """Learned sparse top-1 routing with load-balance auxiliary loss.

    Same top-1 dispatch as BaselineMoEFFN, but adds a load-balance auxiliary loss
    that encourages equal token distribution across experts during training:

        L_aux = aux_coeff * K * sum_i(f_i * p_i)

    where f_i = fraction of tokens dispatched to expert i (stop-gradient),
    and p_i = mean gate probability for expert i (differentiable).

    This is the auxiliary loss described in the sparse routing literature for
    encouraging load balance without collapsing to a single expert.

    The aux_loss is returned as a scalar tensor so the training loop can
    backpropagate through it.
    """

    def __init__(self, d_model: int, k_experts: int, expert_dim: int,
                 aux_coeff: float = AUX_COEFF):
        super().__init__()
        self.k = k_experts
        self.aux_coeff = aux_coeff
        self.gate = nn.Linear(d_model, k_experts, bias=False)
        self.experts = nn.ModuleList(
            [ExpertFFN(d_model, expert_dim, d_model) for _ in range(k_experts)]
        )

    def forward(self, x: torch.Tensor) -> Tuple[torch.Tensor, Dict]:
        shape = x.shape
        xf = x.view(-1, x.shape[-1])           # (N, D)
        gate_logits = self.gate(xf)             # (N, K)
        gate_probs = F.softmax(gate_logits, dim=-1)  # (N, K)
        top1 = gate_probs.argmax(dim=-1)        # (N,)

        out = torch.zeros_like(xf)
        for k in range(self.k):
            mask = (top1 == k)
            if mask.any():
                out[mask] = self.experts[k](xf[mask])

        # Load-balance auxiliary loss: encourages equal token distribution.
        # f_i = fraction dispatched to expert i (stop-gradient — treat as constant)
        # p_i = mean gate probability for expert i (differentiable)
        N = float(xf.shape[0])
        counts_tensor = torch.stack(
            [(top1 == k).float().sum() for k in range(self.k)]
        )  # (K,)
        f_i = (counts_tensor / N).detach()      # stop-gradient on dispatch fraction
        p_i = gate_probs.mean(dim=0)            # mean gate probability, differentiable
        aux_loss = self.aux_coeff * self.k * (f_i * p_i).sum()

        # Routing metrics (for monitoring; use detached values)
        counts = Counter(top1.detach().cpu().tolist())
        p_np = np.array([counts.get(k, 0) for k in range(self.k)], dtype=float)
        p_sum = p_np.sum()
        eff_b = float(np.exp(-np.sum((p_np/p_sum) * np.log(p_np/p_sum + 1e-30)))) if p_sum > 0 else 0.0

        return out.view(shape), {
            "routing_ops": float(xf.shape[0]),
            "eff_buckets": eff_b,
            "aux_loss": aux_loss,           # tensor — accumulated by training loop
            "aux_loss_val": aux_loss.item(),
            "sector_counts": counts,
        }


class HopfRoutedFFN(nn.Module):
    """Fixed Hopf angular routing (INC-0169 algorithm). No learned gate matrix."""

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
        xn = F.normalize(xf, p=2, dim=-1)
        sectors = hopf_sectors(xn, self.k)
        perm = self._get_perm(xf.device)
        if perm is not None:
            sectors = perm[sectors]
        out = torch.zeros_like(xf)
        for k in range(self.k):
            mask = (sectors == k)
            if mask.any():
                out[mask] = self.experts[k](xf[mask])
        counts = Counter(sectors.detach().cpu().tolist())
        p = np.array([counts.get(k, 0) for k in range(self.k)], dtype=float)
        p_sum = p.sum()
        eff_b = float(np.exp(-np.sum((p/p_sum) * np.log(p/p_sum + 1e-30)))) if p_sum > 0 else 0.0
        return out.view(shape), {"routing_ops": float(xf.shape[0]),
                                  "eff_buckets": eff_b,
                                  "sector_counts": counts}


# ---------------------------------------------------------------------------
# Transformer (unchanged from INC-0171)
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
        attn_out, _ = self.attn(x, x, x, attn_mask=causal_mask, need_weights=False)
        x = self.norm1(x + attn_out)
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
        logits = self.head(h)
        return logits, routing_infos


# ---------------------------------------------------------------------------
# Metrics helpers
# ---------------------------------------------------------------------------

def aggregate_routing_info(info_list: List[List[Dict]]) -> Dict:
    """Aggregate routing metrics across batches and layers, ignoring DENSE NaNs."""
    total_ops = 0.0
    eff_vals = []
    aux_loss_vals = []
    n = len(info_list) * len(info_list[0]) if info_list else 1
    for batch_infos in info_list:
        for layer_info in batch_infos:
            total_ops += layer_info.get("routing_ops", 0.0)
            ev = layer_info.get("eff_buckets", float("nan"))
            if not (isinstance(ev, float) and math.isnan(ev)):
                eff_vals.append(ev)
            if "aux_loss_val" in layer_info:
                aux_loss_vals.append(layer_info["aux_loss_val"])
    mean_eff = float(np.mean(eff_vals)) if eff_vals else float("nan")
    mean_aux = float(np.mean(aux_loss_vals)) if aux_loss_vals else float("nan")
    return {
        "routing_ops": total_ops / n,
        "eff_buckets": mean_eff,
        "mean_aux_loss": mean_aux,
    }


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


# ---------------------------------------------------------------------------
# Training
# ---------------------------------------------------------------------------

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
    aux_coeff: float = AUX_COEFF,
) -> Dict:
    torch.manual_seed(seed)
    np.random.seed(seed)

    def ffn_factory(layer_idx: int) -> nn.Module:
        if condition == "DENSE":
            return DenseFFN(HIDDEN_DIM, ffn_dim=4 * HIDDEN_DIM)
        elif condition == "BASELINE":
            return BaselineMoEFFN(HIDDEN_DIM, K_EXPERTS, EXPERT_DIM)
        elif condition == "LEARNED_SPARSE":
            return LearnedSparseMoEFFN(HIDDEN_DIM, K_EXPERTS, EXPERT_DIM,
                                       aux_coeff=aux_coeff)
        elif condition == "HOPF":
            return HopfRoutedFFN(HIDDEN_DIM, K_EXPERTS, EXPERT_DIM)
        elif condition == "PERMUTED":
            return HopfRoutedFFN(HIDDEN_DIM, K_EXPERTS, EXPERT_DIM,
                                 perm_seed=seed + layer_idx * 100)
        raise ValueError(f"Unknown condition: {condition}")

    model = SmallTransformerLM(
        vocab_size=len(vocab),
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
            logits, train_infos = model(xb)
            B, T, V = logits.shape
            loss = F.cross_entropy(logits.view(B*T, V), yb.view(B*T))

            # Accumulate aux_loss from all layers (only LEARNED_SPARSE returns it)
            for layer_info in train_infos:
                al = layer_info.get("aux_loss")
                if al is not None and isinstance(al, torch.Tensor):
                    loss = loss + al

            optimizer.zero_grad()
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), GRAD_CLIP)
            optimizer.step()
            scheduler.step()
            step += 1

            if step % EVAL_EVERY == 0 or step == steps:
                val_ppl, val_routing = evaluate(model, val_x, val_y)
                aux_str = ""
                if not math.isnan(val_routing["mean_aux_loss"]):
                    aux_str = f"  aux={val_routing['mean_aux_loss']:.4f}"
                eff_str = f"{val_routing['eff_buckets']:.2f}" if not math.isnan(
                    val_routing["eff_buckets"]) else "N/A"
                print(f"    step={step:5d}  val_ppl={val_ppl:.2f}  "
                      f"eff_b={eff_str}{aux_str}")
                log.append({
                    "step": step,
                    "val_ppl": val_ppl,
                    "eff_buckets": val_routing["eff_buckets"],
                    "routing_ops": val_routing["routing_ops"],
                    "mean_aux_loss": val_routing["mean_aux_loss"],
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
    ap = argparse.ArgumentParser(
        description="INC-0172: MoE substitution study — angular routing vs learned sparse top-1."
    )
    ap.add_argument("--seeds", type=str, default="0",
                    help="Comma-separated seed list (0=screen, 0,1=confirm, 0,1,2,3=finalize)")
    ap.add_argument("--conditions", type=str,
                    default="DENSE,BASELINE,LEARNED_SPARSE,HOPF,PERMUTED")
    ap.add_argument("--steps", type=int, default=STEPS)
    ap.add_argument("--device", type=str, default="cpu")
    ap.add_argument("--aux_coeff", type=float, default=AUX_COEFF,
                    help="Load-balance aux loss coefficient for LEARNED_SPARSE. "
                         "Use 1e-2 (default). Only change if concentration guard fails.")
    ap.add_argument("--output", type=str,
                    default=os.path.join(OUTPUT_DIR, "inc0172_moe_substitution.json"))
    args = ap.parse_args()

    seeds = [int(s.strip()) for s in args.seeds.split(",")]
    conditions = [c.strip() for c in args.conditions.split(",")]
    device = torch.device(args.device)

    print("=" * 72)
    print("INC-0172: MOE SUBSTITUTION STUDY — ANGULAR ROUTING vs LEARNED SPARSE TOP-1")
    print("=" * 72)
    print(f"  Conditions:  {conditions}")
    print(f"  Seeds:       {seeds}")
    print(f"  Steps:       {args.steps}")
    print(f"  Device:      {device}")
    print(f"  K experts:   {K_EXPERTS}")
    print(f"  Arch:        {N_LAYERS}L × h{HIDDEN_DIM} × {N_HEADS}h")
    print(f"  aux_coeff:   {args.aux_coeff} (for LEARNED_SPARSE)")
    print(f"  Conc. guard: LEARNED_SPARSE eff_b ≤ {CONCENTRATION_GUARD_FRAC * K_EXPERTS:.1f} (={CONCENTRATION_GUARD_FRAC:.0%} of K={K_EXPERTS})")
    print()

    print(f"Loading PTB: {PTB_TRAIN}")
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
        for seed in seeds:
            print(f"\n  Seed {seed}:")
            import time
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
                aux_coeff=args.aux_coeff,
            )
            result["elapsed_s"] = time.time() - t0
            all_results.append(result)
            eff_str = f"{result['final_eff_buckets']:.2f}" if not math.isnan(
                result['final_eff_buckets']) else "N/A"
            print(f"  → val_ppl={result['final_val_ppl']:.2f}  "
                  f"eff_b={eff_str}  elapsed={result['elapsed_s']:.0f}s")

    # -------------------------------------------------------------------------
    # Summary and verdict
    # -------------------------------------------------------------------------
    print("\n" + "=" * 72)
    print("SUMMARY")
    print("=" * 72)
    print(f"\n{'Condition':16s}  {'Seeds':5s}  {'Val PPL':8s}  {'eff_b':8s}  {'params':10s}")
    print("-" * 60)

    by_condition: Dict[str, list] = {}
    for r in all_results:
        by_condition.setdefault(r["condition"], []).append(r)

    summary = {}
    for c in conditions:
        rows = by_condition.get(c, [])
        ppls = [r["final_val_ppl"] for r in rows]
        effs = [r["final_eff_buckets"] for r in rows
                if not math.isnan(r["final_eff_buckets"])]
        mean_ppl = float(np.mean(ppls)) if ppls else float("nan")
        mean_eff = float(np.mean(effs)) if effs else float("nan")
        n_params = rows[0]["n_params"] if rows else 0
        eff_str = f"{mean_eff:.2f}" if not math.isnan(mean_eff) else "N/A"
        print(f"{c:16s}  {len(rows):5d}  {mean_ppl:8.2f}  {eff_str:8s}  {n_params:10,}")
        summary[c] = {
            "mean_val_ppl": mean_ppl,
            "mean_eff_buckets": mean_eff,
            "n_seeds": len(rows),
            "n_params": n_params,
        }

    # Key comparison metrics
    bl_rows  = by_condition.get("BASELINE", [])
    ls_rows  = by_condition.get("LEARNED_SPARSE", [])
    hop_rows = by_condition.get("HOPF", [])
    perm_rows = by_condition.get("PERMUTED", [])

    bl_ppl  = float(np.mean([r["final_val_ppl"] for r in bl_rows]))  if bl_rows  else float("nan")
    ls_ppl  = float(np.mean([r["final_val_ppl"] for r in ls_rows]))  if ls_rows  else float("nan")
    hop_ppl = float(np.mean([r["final_val_ppl"] for r in hop_rows])) if hop_rows else float("nan")
    bl_eff  = float(np.mean([r["final_eff_buckets"] for r in bl_rows]))  if bl_rows  else float("nan")
    ls_eff  = float(np.mean([r["final_eff_buckets"] for r in ls_rows]))  if ls_rows  else float("nan")
    hop_eff = float(np.mean([r["final_eff_buckets"] for r in hop_rows])) if hop_rows else float("nan")

    ppl_hopf_vs_baseline       = hop_ppl / bl_ppl if not math.isnan(bl_ppl)  else float("nan")
    ppl_hopf_vs_learned_sparse = hop_ppl / ls_ppl if not math.isnan(ls_ppl)  else float("nan")
    eff_ratio_dense_vs_hopf    = K_EXPERTS / hop_eff if not math.isnan(hop_eff) else float("nan")
    concentration_guard_limit  = CONCENTRATION_GUARD_FRAC * K_EXPERTS

    print("\n" + "-" * 72)
    print("KEY METRICS")
    print("-" * 72)

    # PPL comparisons
    def _fmt_ratio(val: float, threshold: float, invert: bool = False) -> str:
        if math.isnan(val):
            return "N/A"
        ok = val <= threshold if not invert else val >= threshold
        return f"{val:.3f}  ({'OK' if ok else 'FAIL'} — threshold {'≤' if not invert else '≥'} {threshold:.2f})"

    print(f"  HOPF / BASELINE PPL ratio:          {_fmt_ratio(ppl_hopf_vs_baseline, 1.10)}")
    print(f"  HOPF / LEARNED_SPARSE PPL ratio:    {_fmt_ratio(ppl_hopf_vs_learned_sparse, 1.10)}")
    print(f"  DENSE / HOPF eff_ratio (vs K=64):   {_fmt_ratio(eff_ratio_dense_vs_hopf, 1.3, invert=True)}")

    # Concentration guard
    concentration_ok = (not math.isnan(ls_eff)) and ls_eff <= concentration_guard_limit
    conc_str = f"{ls_eff:.2f}" if not math.isnan(ls_eff) else "N/A"
    conc_status = "OK" if concentration_ok else "FAIL (REFINE aux_coeff — max 2 values, 1 seed only)"
    print(f"  LEARNED_SPARSE eff_b concentration: {conc_str}  ({conc_status} — guard ≤ {concentration_guard_limit:.1f})")

    # Verdict
    print()
    ppl_ok      = (not math.isnan(ppl_hopf_vs_learned_sparse)) and ppl_hopf_vs_learned_sparse <= 1.10
    repl_ok     = (not math.isnan(ppl_hopf_vs_baseline))       and ppl_hopf_vs_baseline <= 1.10
    no_collapse = (not math.isnan(hop_eff))                     and hop_eff > K_EXPERTS / 4
    conc_guard  = concentration_ok

    if ppl_ok and repl_ok and no_collapse and conc_guard:
        verdict = "KEEP"
    elif (not math.isnan(ppl_hopf_vs_learned_sparse)) and ppl_hopf_vs_learned_sparse > 1.15:
        verdict = "KILL"
    elif not conc_guard:
        verdict = "REFINE — concentration guard failed; check aux_coeff before proceeding"
    else:
        verdict = "REFINE"

    print(f"  VERDICT: {verdict}")
    print()
    if verdict == "KEEP":
        print("  Claim upgrade: fixed routing substitutes for learned sparse top-1")
        print("  routing (with load-balance aux loss) at < 10% PPL cost, K=64, 2-layer toy.")
    elif verdict == "KILL":
        print(f"  Claim KILLED: HOPF gap vs LEARNED_SPARSE = {ppl_hopf_vs_learned_sparse:.3f}")
        print("  Fixed routing cannot substitute for learned sparse routing at this scale.")
    else:
        print("  Refine before upgrading claim. See concentration guard / PPL ratio above.")

    # Save results
    output_data = {
        "increment": "INC-0172",
        "verdict": verdict.split(" — ")[0],  # just KEEP/KILL/REFINE for machine readers
        "verdict_full": verdict,
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
            "aux_coeff": args.aux_coeff,
        },
        "summary": summary,
        "key_metrics": {
            "ppl_ratio_hopf_vs_baseline": ppl_hopf_vs_baseline,
            "ppl_ratio_hopf_vs_learned_sparse": ppl_hopf_vs_learned_sparse,
            "eff_ratio_dense_vs_hopf": eff_ratio_dense_vs_hopf,
            "learned_sparse_eff_b": ls_eff,
            "concentration_guard_limit": concentration_guard_limit,
            "concentration_guard_passed": concentration_ok,
        },
        "results": all_results,
    }
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w") as f:
        json.dump(output_data, f, indent=2)
    print(f"Results saved: {args.output}")


if __name__ == "__main__":
    main()
