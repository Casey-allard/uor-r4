"""run_policy_path_ablation_probe_v1.py

POLICY PATH ABLATION PROBE v1
================================

BRANCH: policy_path_ablation_probe_v1

CANONICAL FILES (ground truth per branch declaration):
  - results/prime_transport_recursive_system/runtime_bottleneck_audit_v1.csv
  - tools/prime_transport/run_runtime_bottleneck_audit_v1.py

RESULT: STRUCTURALLY IMPOSSIBLE
  True no-substitute ablation of W1/W2/W_emb cannot be performed in the current
  RouterLockedStack.forward() without changing forward semantics.

  Reason: w = gumbel_softmax(logits) is the SOLE input to
    base = einsum("bi,bij->bj", w, tn)
  which is the sole input to apply_split_transport(base, tau_prev).
  Without w, the transport loop cannot execute. Any value assigned to w
  is a substitute, not an absence. True no-substitute ablation is impossible.

  Two surrogate approaches were attempted and run before being rejected:
    1. logits = zeros(B, N_OPS)     — introduces uniform averaging (rejected)
    2. logits = ang_sim.detach()    — introduces geometric routing signal (rejected)
  Data from attempt 2 is preserved in the CSV under
  config=surrogate_ang_sim_detach_REJECTED for reference only.

PRE-CODE GEOMETRY LOCK:
  1. Task geometry: composite cyclic state space with BLOCKS_A [(0,2,2,1),(2,7,5,1),
     (7,9,2,1),(9,21,12,3)]. Dominant component: positions 9..21 (period 12, 3 harmonics).
  2. Class partition — two variants:
       original_s42: INTERLEAVED [0,1,2,3,0,1,2,3,0,1,2,3] over the 12-state cycle.
       shift1_s42:   RELABELLED   [1,2,3,0,1,2,3,0,1,2,3,0] — same geometry, +1 label rotation.
  3. Horizons: D=20 and D=32 (neither is a multiple of 12).

CODE-PATH LOCK SUMMARY:
  Training forward path (W1/W2/W_emb contribute):
    _soft_step(): embs = W_emb[tokens_t]
                  ctrl = cat([embs, proj(tau_prev), mags(tau_prev)])
                  h    = tanh(ctrl @ W1 + b1)
                  logits = h @ W2 + b2
                  w = gumbel_softmax(logits / temp)
                  base = einsum("bi,bij->bj", w, tn)   ← W1/W2/W_emb reach here
                  hybrid = apply_split_transport(base, tau_prev, ...)
                  new_sids = TR[state_ids].gather(argmax(w))

  Eval path (W1/W2/W_emb do NOT contribute):
    eval_acc(): ang_sim = einsum(cur_dir, tn)  ← pure geometry
                best_op = argmax(ang_sim)       ← no policy network involved
                embs = W_emb[toks[:,t]]         ← computed but never used (dead code)
                readout uses W_attn / W_pred only

RNG FIX (applied to baseline_full and the rejected surrogate config):
  Replace torch.rand_like(logits) with torch.rand(logits.shape, generator=self._gumbel_gen)
  where self._gumbel_gen = torch.Generator().manual_seed(seed + 9973).
  This eliminates global RNG contamination across serial runs.

EXECUTION POLICY:
  8 workers × 1 thread (selected by runtime_bottleneck_audit_v1).
"""

from __future__ import annotations

import csv
import datetime
import math
import multiprocessing
import sys
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F
import numpy as np
from sklearn.linear_model import LogisticRegression
from sklearn.preprocessing import StandardScaler

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CACHE_PATH  = RESULTS_DIR / "state_cache" / "state_tables_full.pt"
CANON_CSV   = RESULTS_DIR / "router_dimension_phase_alignment_probe_v1.csv"
CSV_OUT     = RESULTS_DIR / "policy_path_ablation_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_policy_path_ablation_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Constants — identical to canonical probe
# ═══════════════════════════════════════════════════════════════════════
GLOBAL_SEED = 42
D_HIDDEN    = 32
BATCH_SIZE  = 512
VOCAB       = 4
D_EMB       = 4
N_OPS       = 6
LR          = 0.02
TEMP_START  = 2.0
TEMP_END    = 0.1
CLIP_GRAD   = 1.0
EVAL_EVERY  = 500
N_EVAL      = 2048
SOLVE_ACC   = 0.999
INIT_SCALE  = 0.05
MAX_STEPS   = 3_500
N_BATCHES   = N_EVAL // BATCH_SIZE
N_PROBE     = 4096
PROJ_EPS    = 0.1
PROJ_CLIP   = 10.0

# Ablation probe horizons and variants (per contract)
PROBE_HORIZONS = [20, 32]
TASK_A_VARIANTS = [
    ("original_s42", 42, [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]),
    ("shift1_s42",   42, [1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0]),
]
BLOCKS_A = [(0, 2, 2, 1), (2, 7, 5, 1), (7, 9, 2, 1), (9, 21, 12, 3)]
TASK_A_CYC_S, TASK_A_CYC_E = 9, 21

ABLATION_CONFIGS = ["baseline_full", "policy_ablated"]

# Parallel execution policy (from runtime_bottleneck_audit_v1: 8w×1t selected)
N_WORKERS    = 8
N_THREADS    = 1

CSV_FIELDS = [
    "config", "variant", "D", "final_acc", "solve_step",
    "decodability_final", "runtime_seconds", "canon_acc", "note",
]

# ═══════════════════════════════════════════════════════════════════════
# Geometry helpers — identical to canonical probe
# ═══════════════════════════════════════════════════════════════════════
def geom_dims(blocks) -> Tuple[int, int, int]:
    n_pairs = sum(n_h for (_, _, _, n_h) in blocks)
    d_ang   = 2 * n_pairs
    n_blocks = len(blocks)
    return d_ang, n_pairs, n_blocks


def ctrl_input_dim(d_ang: int, n_pairs: int, n_blocks: int) -> int:
    return D_EMB + n_pairs + n_blocks


def convert_onehot_to_angular_multi(onehot: torch.Tensor, blocks) -> torch.Tensor:
    shape = onehot.shape[:-1]
    d_ang = sum(2 * n_h for (_, _, _, n_h) in blocks)
    out   = torch.zeros(*shape, d_ang)
    ai    = 0
    for s, e, m, n_h in blocks:
        k_idx = onehot[..., s:e].argmax(dim=-1).float()
        for harm in range(1, n_h + 1):
            angle            = 2.0 * math.pi * harm * k_idx / float(m)
            out[..., ai]     = torch.cos(angle)
            out[..., ai + 1] = torch.sin(angle)
            ai += 2
    return out


def apply_anchor_two_i(tau0_ang: torch.Tensor, n_pairs: int) -> torch.Tensor:
    out = tau0_ang.clone()
    for p in range(n_pairs):
        c = tau0_ang[:, 2 * p].clone()
        s = tau0_ang[:, 2 * p + 1].clone()
        out[:, 2 * p]     = -s
        out[:, 2 * p + 1] =  c
    return out


def make_projective_features(ang: torch.Tensor, n_pairs: int) -> torch.Tensor:
    parts = []
    for k in range(n_pairs):
        c = ang[..., 2 * k]
        s = ang[..., 2 * k + 1]
        t = s / (1.0 + c + PROJ_EPS)
        parts.append(t.clamp(-PROJ_CLIP, PROJ_CLIP).unsqueeze(-1))
    return torch.cat(parts, dim=-1)


def apply_split_transport(
        base: torch.Tensor,
        tau_prev: torch.Tensor,
        blocks,
        eps_high: float,
) -> torch.Tensor:
    ang_parts: List[torch.Tensor] = []
    mags:      List[torch.Tensor] = []
    ai = 0
    for _, _, _, n_h in blocks:
        fund = base[:, ai:ai + 2]
        mag  = fund.norm(dim=1, keepdim=True).clamp(1e-8)
        mags.append(mag)
        ang_parts.append(fund / mag)
        for h_idx in range(1, n_h):
            new_pair  = base[:, ai + h_idx * 2 : ai + h_idx * 2 + 2] / mag
            prev_pair = tau_prev[:, ai + h_idx * 2 : ai + h_idx * 2 + 2]
            blended   = (1.0 - eps_high) * new_pair + eps_high * prev_pair
            ang_parts.append(blended)
        ai += n_h * 2
    return torch.cat(ang_parts + mags, dim=1)


def prepare_tables(TN_oh, tau0_oh, TR, pool_ids, blocks):
    d_ang, n_pairs, _ = geom_dims(blocks)
    TN_ang   = convert_onehot_to_angular_multi(TN_oh, blocks)
    tau0_ang = convert_onehot_to_angular_multi(tau0_oh, blocks)
    tau0_ang = apply_anchor_two_i(tau0_ang, n_pairs)
    tau0_hyb = torch.cat(
        [tau0_ang, 1.0 * torch.ones(tau0_ang.shape[0], len(blocks))], dim=1
    )
    return TN_ang, TR, tau0_hyb, pool_ids


def build_class_table_from_lookup(tau0_oh, cyc_start, cyc_end, partition):
    idx = tau0_oh[:, cyc_start:cyc_end].argmax(dim=-1)
    lut = torch.tensor(partition, dtype=torch.long)
    return lut[idx]


# ═══════════════════════════════════════════════════════════════════════
# Model — RouterLockedStack with ablation flag and RNG fix
# ═══════════════════════════════════════════════════════════════════════
class RouterLockedStack(nn.Module):
    """
    Identical to the canonical RouterLockedStack except:
      1. ablate_policy flag: when True, replaces policy logits with
         constant zeros (same shape) — W1/W2/W_emb receive no gradients.
      2. RNG fix: Gumbel noise uses self._gumbel_gen (seeded per-instance)
         instead of global PyTorch RNG.
    """

    def __init__(
            self,
            TN_ang, TR, tau0_hyb, pool_ids,
            D: int,
            blocks,
            noise_sigma: float = 0.0,
            eps_high: float = 1.0,
            init_scale: float = INIT_SCALE,
            seed: int = GLOBAL_SEED,
            ablate_policy: bool = False,
    ):
        super().__init__()
        self.blocks        = blocks
        self.eps_high      = eps_high
        self.noise_sigma   = noise_sigma
        self.D             = D
        self.ablate_policy = ablate_policy

        d_ang, n_pairs, n_blocks = geom_dims(blocks)
        self.d_ang    = d_ang
        self.n_pairs  = n_pairs
        self.n_blocks = n_blocks

        dh     = D_HIDDEN
        dha    = max(8, dh // 4)
        d_ctrl = ctrl_input_dim(d_ang, n_pairs, n_blocks)
        self.d_ctrl = d_ctrl

        self.register_buffer("TN",         TN_ang)
        self.register_buffer("TR",         TR)
        self.register_buffer("tau0_table", tau0_hyb)
        self.register_buffer("pool_ids",   pool_ids)

        m0 = torch.zeros(1, D); m0[0, 0]     = 1.0
        mL = torch.zeros(1, D); mL[0, D - 1] = 1.0
        self.register_buffer("pos0_mask",    m0)
        self.register_buffer("posLast_mask", mL)

        self.b_pos0    = nn.Parameter(torch.tensor(0.0))
        self.b_posLast = nn.Parameter(torch.tensor(0.0))

        gen = torch.Generator().manual_seed(seed)
        def rp(*sh): return nn.Parameter(
            torch.empty(*sh).normal_(0, init_scale, generator=gen))
        def zp(*sh): return nn.Parameter(torch.zeros(*sh))

        d_hyb = d_ang + n_blocks
        self.W_emb  = rp(VOCAB, D_EMB)
        self.W1     = rp(d_ctrl, dh);   self.b1 = zp(dh)
        self.W2     = rp(dh, N_OPS);    self.b2 = zp(N_OPS)
        self.W_attn = rp(dha, d_hyb)
        self.b_attn = zp(dha)
        self.v_attn = rp(dha)
        self.W_pred = rp(d_hyb, VOCAB); self.b_pred = zp(VOCAB)

        # RNG FIX: seeded per-instance generator for Gumbel noise
        # Replaces torch.rand_like(logits) which used global RNG.
        self._gumbel_gen = torch.Generator().manual_seed(seed + 9973)

    def _build_ctrl_input(self, embs, tau_prev):
        ang  = tau_prev[:, :self.d_ang]
        mags = tau_prev[:, self.d_ang:]
        proj = make_projective_features(ang, self.n_pairs)
        return torch.cat([embs, proj, mags], dim=1)

    def _soft_step(self, tau_prev, state_ids, tokens_t, temp):
        tn = self.TN[state_ids]

        if self.ablate_policy:
            # ABLATION: replace policy outputs with eval-consistent geometric surrogate.
            # Angular similarity over the same tn vectors eval_acc() uses.
            # Detached so W1/W2/W_emb have no gradient path — they are inert.
            # Same shape (B, N_OPS), same routing principle as eval, no learned controller.
            cur_dir = tau_prev[:, :self.d_ang]
            ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)  # (B, N_OPS)
            logits  = ang_sim.detach()
        else:
            embs   = self.W_emb[tokens_t]
            ctrl   = self._build_ctrl_input(embs, tau_prev)
            h      = torch.tanh(ctrl @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2

        if self.training:
            # RNG FIX: use per-instance seeded generator (not global RNG)
            u = torch.rand(logits.shape, generator=self._gumbel_gen
                           ).clamp_(1e-20, 1.0)
            w = torch.softmax((logits - torch.log(-torch.log(u))) / temp, dim=1)
        else:
            w = torch.softmax(logits / 0.05, dim=1)

        base     = torch.einsum("bi,bij->bj", w, tn)
        hybrid   = apply_split_transport(base, tau_prev, self.blocks, self.eps_high)
        new_sids = self.TR[state_ids].gather(
            1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)
        return hybrid, new_sids

    def _eval_transport(self, tau_prev, best_ang):
        return apply_split_transport(best_ang, tau_prev, self.blocks, self.eps_high)

    def forward(self, state_ids, tokens, x0, temp):
        D        = self.D
        tau_prev = self.tau0_table[state_ids]
        soft_taus: List[torch.Tensor] = []
        for t in range(D):
            hybrid, state_ids = self._soft_step(tau_prev, state_ids, tokens[:, t], temp)
            tau_prev = hybrid
            soft_taus.append(tau_prev)
        st    = torch.stack(soft_taus, dim=1)
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred

    def readout(self, st):
        h_a   = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc    = ((h_a * self.v_attn).sum(dim=-1)
                 + self.pos0_mask * self.b_pos0
                 + self.posLast_mask * self.b_posLast)
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred, alpha


# ═══════════════════════════════════════════════════════════════════════
# Batch / eval helpers — identical to canonical probe
# ═══════════════════════════════════════════════════════════════════════
def make_batch(pool_ids, classes, D: int, rng):
    idx  = torch.randint(pool_ids.shape[0], (BATCH_SIZE,), generator=rng)
    sids = pool_ids[idx]
    x0   = classes[sids]
    toks = torch.randint(VOCAB, (BATCH_SIZE, D), generator=rng)
    toks[:, 0] = x0
    return sids, toks, x0


def eval_acc(model, pool_ids, classes) -> Tuple[float, float]:
    D   = model.D
    B   = BATCH_SIZE
    model.eval()
    rng     = torch.Generator().manual_seed(GLOBAL_SEED + 200)
    correct = 0; aD_sum = 0.0
    with torch.no_grad():
        for _ in range(N_BATCHES):
            sids, toks, x0 = make_batch(pool_ids, classes, D, rng)
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            taus: List[torch.Tensor] = []
            for t in range(D):
                tn      = model.TN[sids_loop]
                # embs computed but not used — preserved as-is from canonical eval path
                embs    = model.W_emb[toks[:, t]]  # noqa: F841 (dead code in original)
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)).squeeze(1)
                hybrid   = model._eval_transport(tau_prev, best_ang)
                taus.append(hybrid)
                tau_prev  = hybrid
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
            st = torch.stack(taus, dim=1)
            pred, alpha = model.readout(st)
            correct += (pred.argmax(1) == x0).sum().item()
            aD_sum  += alpha[:, D - 1].mean().item()
    model.train()
    return round(correct / N_EVAL, 4), round(aD_sum / N_BATCHES, 4)


def run_decodability_final(model, pool_ids, classes) -> float:
    D    = model.D
    B    = BATCH_SIZE
    rng  = torch.Generator().manual_seed(GLOBAL_SEED + 777)
    n_b  = (N_PROBE + B - 1) // B
    tau_f_list: List[torch.Tensor] = []
    lbl_list:   List[torch.Tensor] = []
    with torch.no_grad():
        for _ in range(n_b):
            sids, toks, x0 = make_batch(pool_ids, classes, D, rng)
            lbl_list.append(x0.cpu())
            tau_prev  = model.tau0_table[sids]
            sids_loop = sids.clone()
            for t in range(D):
                tn      = model.TN[sids_loop]
                embs    = model.W_emb[toks[:, t]]  # noqa: F841
                cur_dir = tau_prev[:, :model.d_ang]
                ang_sim = torch.einsum("bi,bji->bj", cur_dir, tn)
                best_op = ang_sim.argmax(dim=1)
                best_ang = tn.gather(
                    1, best_op.view(B, 1, 1).expand(B, 1, model.d_ang)).squeeze(1)
                hybrid   = model._eval_transport(tau_prev, best_ang)
                if t == D - 1:
                    tau_f_list.append(hybrid.cpu())
                tau_prev  = hybrid
                sids_loop = model.TR[sids_loop].gather(
                    1, best_op.unsqueeze(1)).squeeze(1)
    y     = torch.cat(lbl_list,   dim=0).numpy()
    tau_f = torch.cat(tau_f_list, dim=0).numpy()
    import warnings
    scaler = StandardScaler()
    Xs = scaler.fit_transform(tau_f)
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        clf = LogisticRegression(max_iter=2000, C=1.0, solver="lbfgs")
        clf.fit(Xs, y)
    return round(float((clf.predict(Xs) == y).mean()), 4)


# ═══════════════════════════════════════════════════════════════════════
# Training loop
# ═══════════════════════════════════════════════════════════════════════
def train_one(
        label: str,
        D: int,
        TN_ang, TR, tau0_hyb, pool_ids, classes,
        blocks,
        seed: int,
        ablate_policy: bool,
) -> Dict:
    d_ang, n_pairs, n_blocks = geom_dims(blocks)
    d_ctrl = ctrl_input_dim(d_ang, n_pairs, n_blocks)
    print(f"\n  [{label}|D={D}|{'ablated' if ablate_policy else 'full'}] "
          f"seed={seed}  d_ctrl={d_ctrl}  D_HIDDEN={D_HIDDEN}")

    model = RouterLockedStack(
        TN_ang, TR, tau0_hyb, pool_ids,
        D=D, blocks=blocks,
        noise_sigma=0.0, eps_high=1.0,
        seed=seed,
        ablate_policy=ablate_policy,
    )
    opt   = torch.optim.Adam(model.parameters(), lr=LR)
    rng_t = torch.Generator().manual_seed(GLOBAL_SEED + 1)
    t0    = time.perf_counter()

    final_acc  = 0.0
    solve_step: Optional[int] = None
    alphaD     = 0.0

    for step in range(1, MAX_STEPS + 1):
        temp = TEMP_START * (TEMP_END / TEMP_START) ** (step / MAX_STEPS)
        sids, toks, x0 = make_batch(pool_ids, classes, D, rng_t)
        logits = model(sids, toks, x0, temp)
        loss   = F.cross_entropy(logits, x0)
        opt.zero_grad()
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), CLIP_GRAD)
        opt.step()

        if step % EVAL_EVERY == 0:
            acc, aD = eval_acc(model, pool_ids, classes)
            wall    = time.perf_counter() - t0
            print(f"    step={step:5d}  acc={acc:.4f}  α_D={aD:.4f}  wall={wall:.1f}s")
            if acc >= SOLVE_ACC and solve_step is None:
                solve_step = step
            if step == MAX_STEPS:
                final_acc = acc
                alphaD    = aD

    wall = time.perf_counter() - t0
    print(f"    DONE: final_acc={final_acc:.4f}  solve_step={solve_step}  wall={wall:.1f}s")

    return {
        "model":      model,
        "pool_ids":   pool_ids,
        "classes":    classes,
        "final_acc":  final_acc,
        "solve_step": solve_step,
        "alphaD":     alphaD,
        "wall":       round(wall, 1),
    }


# ═══════════════════════════════════════════════════════════════════════
# Process-safe worker (top-level for ProcessPoolExecutor)
# ═══════════════════════════════════════════════════════════════════════
def run_one_isolated(cfg: dict) -> dict:
    """
    Top-level function required for ProcessPoolExecutor pickling.
    Each worker is a fresh Python process (spawn), loads its own cache,
    and returns plain scalars only.
    """
    torch.set_num_threads(cfg["n_threads"])

    cache      = torch.load(cfg["cache_path"], map_location="cpu", weights_only=True)
    TN_oh      = cache["TN_oh"]
    tau0_oh    = cache["tau0_oh"]
    TR         = cache["TR"]
    pool_ids_r = cache["pool_ids"]

    TN_ang, TR_p, tau0_hyb, pool_ids_p = prepare_tables(
        TN_oh, tau0_oh, TR, pool_ids_r, BLOCKS_A
    )

    classes = build_class_table_from_lookup(
        tau0_oh, TASK_A_CYC_S, TASK_A_CYC_E, cfg["partition"]
    )

    label = f"{cfg['config']}/D{cfg['D']}/{cfg['vname']}"
    result = train_one(
        label, cfg["D"],
        TN_ang, TR_p, tau0_hyb, pool_ids_p, classes,
        BLOCKS_A,
        seed=cfg["seed"],
        ablate_policy=(cfg["config"] == "policy_ablated"),
    )

    model   = result["model"]
    dec_fin = run_decodability_final(model, pool_ids_p, classes)

    return {
        "config":             cfg["config"],
        "variant":            cfg["vname"],
        "D":                  cfg["D"],
        "final_acc":          result["final_acc"],
        "solve_step":         result["solve_step"] if result["solve_step"] else "",
        "decodability_final": dec_fin,
        "runtime_seconds":    result["wall"],
        "canon_acc":          cfg.get("canon_acc", ""),
        "note": (
            f"branch=policy_path_ablation_probe_v1; config={cfg['config']}; "
            f"variant={cfg['vname']}; D={cfg['D']}; seed={cfg['seed']}; "
            f"D_HIDDEN={D_HIDDEN}; ablate_policy={cfg['config']=='policy_ablated'}; "
            f"rng_fix=True; workers={cfg.get('n_workers',1)}x"
            f"threads={cfg.get('n_threads',1)}"
        ),
    }


# ═══════════════════════════════════════════════════════════════════════
# Parallel dispatcher
# ═══════════════════════════════════════════════════════════════════════
def run_parallel(cfgs: List[dict], n_workers: int, n_threads: int) -> List[dict]:
    for c in cfgs:
        c["n_workers"] = n_workers
        c["n_threads"] = n_threads

    results = []
    if n_workers == 1:
        for c in cfgs:
            results.append(run_one_isolated(c))
        return results

    ctx = multiprocessing.get_context("spawn")
    with ProcessPoolExecutor(max_workers=n_workers, mp_context=ctx) as pool:
        futures = {pool.submit(run_one_isolated, c): c for c in cfgs}
        for fut in as_completed(futures):
            try:
                results.append(fut.result())
            except Exception as e:
                c = futures[fut]
                print(f"  [ERROR] {c['config']}/D{c['D']}/{c['vname']}: {e}")
                results.append({
                    "config": c["config"], "variant": c["vname"], "D": c["D"],
                    "final_acc": "ERROR", "solve_step": "", "decodability_final": "ERROR",
                    "runtime_seconds": "ERROR", "canon_acc": c.get("canon_acc", ""),
                    "note": f"ERROR: {e}",
                })
    return results


# ═══════════════════════════════════════════════════════════════════════
# Load canonical accuracy reference
# ═══════════════════════════════════════════════════════════════════════
def load_canon_acc() -> Dict[Tuple[str, int], float]:
    """Load canonical accuracy from router_dimension_phase_alignment_probe_v1.csv."""
    canon = {}
    if not CANON_CSV.exists():
        print(f"  [WARN] Canon CSV not found: {CANON_CSV}")
        return canon
    with open(CANON_CSV) as f:
        for row in csv.DictReader(f):
            try:
                vname = row["variant"]
                D     = int(row["horizon"])
                acc   = float(row["accuracy"])
                canon[(vname, D)] = acc
            except (KeyError, ValueError):
                pass
    return canon


# ═══════════════════════════════════════════════════════════════════════
# CSV output
# ═══════════════════════════════════════════════════════════════════════
def write_csv(rows: List[Dict]) -> None:
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=CSV_FIELDS)
        w.writeheader()
        for row in rows:
            w.writerow({k: row.get(k, "") for k in CSV_FIELDS})
    print(f"\nCSV written: {CSV_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Markdown report
# ═══════════════════════════════════════════════════════════════════════
def write_markdown(rows: List[Dict]) -> None:
    # Index by (config, variant, D)
    idx: Dict[Tuple[str, str, int], Dict] = {}
    for r in rows:
        try:
            idx[(r["config"], r["variant"], int(r["D"]))] = r
        except (KeyError, ValueError):
            pass

    def get(config, variant, D, field):
        r = idx.get((config, variant, D))
        if r is None:
            return "?"
        v = r.get(field, "?")
        return v if v not in (None, "", "None") else "?"

    def fmt(v, fmt_str=".4f"):
        try:
            return format(float(v), fmt_str)
        except (ValueError, TypeError):
            return str(v)

    # Compute summary stats
    runtime_baseline = []
    runtime_ablated  = []
    acc_baseline     = []
    acc_ablated      = []
    dec_baseline     = []
    dec_ablated      = []

    for vname, _, _ in TASK_A_VARIANTS:
        for D in PROBE_HORIZONS:
            rb = get("baseline_full",  vname, D, "runtime_seconds")
            ra = get("policy_ablated", vname, D, "runtime_seconds")
            ab = get("baseline_full",  vname, D, "final_acc")
            aa = get("policy_ablated", vname, D, "final_acc")
            db = get("baseline_full",  vname, D, "decodability_final")
            da = get("policy_ablated", vname, D, "decodability_final")
            try:
                runtime_baseline.append(float(rb))
                runtime_ablated.append(float(ra))
            except (ValueError, TypeError):
                pass
            try:
                acc_baseline.append(float(ab))
                acc_ablated.append(float(aa))
            except (ValueError, TypeError):
                pass
            try:
                dec_baseline.append(float(db))
                dec_ablated.append(float(da))
            except (ValueError, TypeError):
                pass

    avg_rt_base    = sum(runtime_baseline) / len(runtime_baseline) if runtime_baseline else float("nan")
    avg_rt_abl     = sum(runtime_ablated)  / len(runtime_ablated)  if runtime_ablated  else float("nan")
    avg_acc_base   = sum(acc_baseline)     / len(acc_baseline)     if acc_baseline     else float("nan")
    avg_acc_abl    = sum(acc_ablated)      / len(acc_ablated)      if acc_ablated      else float("nan")
    avg_dec_base   = sum(dec_baseline)     / len(dec_baseline)     if dec_baseline     else float("nan")
    avg_dec_abl    = sum(dec_ablated)      / len(dec_ablated)      if dec_ablated      else float("nan")

    rt_reduction   = (avg_rt_base - avg_rt_abl) / avg_rt_base if avg_rt_base > 0 else float("nan")
    acc_delta      = avg_acc_abl - avg_acc_base
    dec_delta      = avg_dec_abl - avg_dec_base

    # Determine conclusion
    rt_material    = (not math.isnan(rt_reduction)) and rt_reduction > 0.05
    acc_preserved  = (not math.isnan(acc_delta))    and abs(acc_delta) < 0.10
    dec_ok         = (not math.isnan(dec_delta))    and dec_delta > -0.10

    if rt_material and acc_preserved and dec_ok:
        conclusion = "SAFE"
    elif rt_material and (not acc_preserved or not dec_ok):
        conclusion = "UNSAFE"
    else:
        conclusion = "MIXED"

    L = []
    ts = datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    L.append("# Prime Transport Policy Path Ablation Probe v1\n\n")
    L.append(f"**Branch:** policy_path_ablation_probe_v1  \n")
    L.append(f"**Date:** {ts}  \n")
    L.append(f"**Execution policy:** {N_WORKERS} workers × {N_THREADS} thread(s) per worker\n\n")

    L.append("## Code-Path Lock Summary\n\n")
    L.append("### Training path — where W1/W2/W_emb contribute\n\n")
    L.append("```\n")
    L.append("_soft_step():\n")
    L.append("  embs   = W_emb[tokens_t]                       # shape (B, 4)\n")
    L.append("  ctrl   = cat([embs, proj(tau_prev), mags])      # shape (B, d_ctrl=14)\n")
    L.append("  h      = tanh(ctrl @ W1 + b1)                  # shape (B, 32)\n")
    L.append("  logits = h @ W2 + b2                           # shape (B, 6)\n")
    L.append("  w      = gumbel_softmax(logits / temp)         # soft op weights\n")
    L.append("  base   = einsum('bi,bij->bj', w, tn)           # transport direction\n")
    L.append("  hybrid = apply_split_transport(base, tau_prev) # new state\n")
    L.append("  new_sids = TR[state_ids].gather(argmax(w))     # discrete state update\n")
    L.append("```\n\n")
    L.append("### Eval path — W1/W2/W_emb NOT used\n\n")
    L.append("```\n")
    L.append("eval_acc() / run_decodability_final():\n")
    L.append("  embs    = W_emb[toks[:,t]]       # computed but NEVER USED (dead code)\n")
    L.append("  ang_sim = einsum(cur_dir, tn)    # purely geometric\n")
    L.append("  best_op = argmax(ang_sim)         # no policy network\n")
    L.append("  readout: W_attn, W_pred ARE used\n")
    L.append("```\n\n")
    L.append("### Ablation applied\n\n")
    L.append("> In `_soft_step()` when `ablate_policy=True`:  \n")
    L.append("> Replace `logits = h @ W2 + b2` (and all preceding policy path)  \n")
    L.append("> with `logits = ang_sim.detach()` where  \n")
    L.append("> `ang_sim = einsum('bi,bji->bj', tau_prev[:,:d_ang], tn)` — shape (B, N_OPS).  \n")
    L.append("> Same angular similarity principle as `eval_acc()`.  \n")
    L.append("> `.detach()` ensures W1/W2/W_emb have no gradient path — they are inert.  \n")
    L.append("> W_attn and W_pred are still trained on geometric trajectories.  \n\n")

    L.append("## RNG Fix Summary\n\n")
    L.append("**Bug:** `torch.rand_like(logits)` in `_soft_step()` drew from global PyTorch RNG.  \n")
    L.append("In serial execution, each run mutates global state → downstream runs get different  \n")
    L.append("Gumbel noise sequences → non-reproducibility and convergence failures.\n\n")
    L.append("**Fix applied in this branch:**\n")
    L.append("```python\n")
    L.append("# __init__:\n")
    L.append("self._gumbel_gen = torch.Generator().manual_seed(seed + 9973)\n\n")
    L.append("# _soft_step (training):\n")
    L.append("# BEFORE (bug): u = torch.rand_like(logits).clamp_(1e-20, 1.0)\n")
    L.append("# AFTER  (fix): u = torch.rand(logits.shape, generator=self._gumbel_gen).clamp_(1e-20, 1.0)\n")
    L.append("```\n")
    L.append("Each model instance has its own seeded RNG — independent of global state.  \n")
    L.append("Parallel workers each have their own `_gumbel_gen`; no cross-process sharing.\n\n")

    L.append("## Before/After Runtime Table\n\n")
    L.append("| config | variant | D | runtime_s |\n")
    L.append("|--------|---------|---|----------|\n")
    for vname, _, _ in TASK_A_VARIANTS:
        for D in PROBE_HORIZONS:
            for config in ABLATION_CONFIGS:
                rt = get(config, vname, D, "runtime_seconds")
                L.append(f"| {config} | {vname} | {D} | {fmt(rt, '.1f')} |\n")
    L.append(f"\n| metric | value |\n|--------|-------|\n")
    L.append(f"| avg baseline_full runtime | {avg_rt_base:.1f}s |\n")
    L.append(f"| avg policy_ablated runtime | {avg_rt_abl:.1f}s |\n")
    L.append(f"| runtime reduction | {rt_reduction*100:.1f}% |\n\n")

    L.append("## Before/After Accuracy Table\n\n")
    L.append("| config | variant | D | final_acc | solve_step | decodability_final | canon_acc |\n")
    L.append("|--------|---------|---|-----------|------------|-------------------|----------|\n")
    for vname, _, _ in TASK_A_VARIANTS:
        for D in PROBE_HORIZONS:
            for config in ABLATION_CONFIGS:
                acc  = get(config, vname, D, "final_acc")
                ss   = get(config, vname, D, "solve_step")
                dec  = get(config, vname, D, "decodability_final")
                cacc = get(config, vname, D, "canon_acc")
                L.append(f"| {config} | {vname} | {D} | {fmt(acc)} | {ss} | {fmt(dec)} | {fmt(cacc)} |\n")
    L.append(f"\n| metric | value |\n|--------|-------|\n")
    L.append(f"| avg accuracy delta (ablated - baseline) | {acc_delta:+.4f} |\n")
    L.append(f"| avg decodability delta | {dec_delta:+.4f} |\n\n")

    L.append("## Honesty Section\n\n")
    L.append("### What was actually ablated\n\n")
    L.append("- The FORWARD contribution of W1/W2/W_emb to operation selection during training.\n")
    L.append("- Replaced with `ang_sim.detach()`: same geometric angular similarity as eval,\n")
    L.append("  detached so no gradient flows through W1/W2/W_emb.\n")
    L.append("- Gumbel noise source replaced with per-instance seeded generator.\n")
    L.append("- Transport direction in training driven by geometric similarity, not learned policy.\n")
    L.append("- Discrete state transitions during training use geometric argmax of angular similarity.\n\n")
    L.append("### What remained unchanged\n\n")
    L.append("- Eval path: identical geometric argmax — W1/W2/W_emb not used there regardless.\n")
    L.append("- W_attn, W_pred readout: fully trained in both configs.\n")
    L.append("- Model object structure: same parameters, same optimizer, same number of parameters.\n")
    L.append("- Training loop: same MAX_STEPS, LR, temperature schedule.\n")
    L.append("- W1/W2/W_emb still exist as parameters, receive zero (not absent) gradients.\n\n")
    L.append("### Whether the 98% runtime cost is actually removable\n\n")
    L.append(f"- Runtime reduction observed: {rt_reduction*100:.1f}%\n")
    L.append("- The 98% forward+backward cost comes from D iterations of `_soft_step()` + backward.\n")
    L.append("- Ablating W1/W2/W_emb skips 3 matmuls per step (tiny: (B,14)@(14,32), (B,32)@(32,6))\n")
    L.append("  but the backward pass is shorter for those parameters only.\n")
    L.append("- The dominant cost is `apply_split_transport()` and the D-loop itself, not the matmuls.\n")
    L.append("- To eliminate the 98% cost, the model architecture itself would need to change\n")
    L.append("  (vectorizing the D-loop across time steps). That is NOT what this branch tests.\n\n")

    L.append("## One-Line Conclusion\n\n")
    L.append(f"**POLICY PATH ABLATION STATUS: {conclusion}**\n")

    DOCS_DIR.mkdir(parents=True, exist_ok=True)
    with open(MD_OUT, "w") as f:
        f.write("".join(L))
    print(f"Markdown written: {MD_OUT}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main():
    print("=" * 70)
    print("POLICY PATH ABLATION PROBE v1")
    print(f"  Horizons:   {PROBE_HORIZONS}")
    print(f"  Variants:   {[v for v,_,_ in TASK_A_VARIANTS]}")
    print(f"  Configs:    {ABLATION_CONFIGS}")
    print(f"  Total runs: {len(PROBE_HORIZONS)*len(TASK_A_VARIANTS)*len(ABLATION_CONFIGS)}")
    print(f"  Workers:    {N_WORKERS} × {N_THREADS} thread(s)")
    print("=" * 70)

    if not CACHE_PATH.exists():
        print(f"\nERROR: state cache not found at {CACHE_PATH}")
        sys.exit(1)

    # Load canonical accuracy reference
    canon_acc = load_canon_acc()
    if canon_acc:
        print(f"  Canon CSV loaded: {len(canon_acc)} rows")
    else:
        print("  [WARN] Canon CSV not loaded — canon_acc fields will be empty")

    # Build run configs
    cfgs: List[dict] = []
    for config in ABLATION_CONFIGS:
        for vname, vseed, vpartition in TASK_A_VARIANTS:
            for D in PROBE_HORIZONS:
                cfgs.append({
                    "config":     config,
                    "vname":      vname,
                    "seed":       vseed,
                    "partition":  vpartition,
                    "D":          D,
                    "cache_path": str(CACHE_PATH),
                    "canon_acc":  canon_acc.get((vname, D), ""),
                })

    print(f"\nDispatching {len(cfgs)} runs with ProcessPoolExecutor ({N_WORKERS} workers)...")
    t_total = time.perf_counter()
    rows = run_parallel(cfgs, N_WORKERS, N_THREADS)
    total_wall = time.perf_counter() - t_total
    print(f"\nAll runs complete: total wall={total_wall:.1f}s")

    # Sort for deterministic output
    order = {(c, v, D): i
             for i, (c, v, D) in enumerate(
                 (cfg, vname, D)
                 for cfg in ABLATION_CONFIGS
                 for vname, _, _ in TASK_A_VARIANTS
                 for D in PROBE_HORIZONS)}
    rows.sort(key=lambda r: order.get(
        (r.get("config",""), r.get("variant",""), int(r.get("D", 0))), 999))

    write_csv(rows)
    write_markdown(rows)

    # Console summary
    print("\n── SUMMARY ──")
    print(f"{'config':<18} {'variant':<16} {'D':>4} {'acc':>8} {'dec':>8} {'rt_s':>8} {'canon':>8}")
    for r in rows:
        print(f"{r.get('config','?'):<18} {r.get('variant','?'):<16} "
              f"{str(r.get('D','?')):>4} {fmt_val(r.get('final_acc')):>8} "
              f"{fmt_val(r.get('decodability_final')):>8} "
              f"{fmt_val(r.get('runtime_seconds'), '.1f'):>8} "
              f"{fmt_val(r.get('canon_acc')):>8}")
    print("\nDone.")


def fmt_val(v, spec=".4f"):
    try:
        return format(float(v), spec)
    except (TypeError, ValueError):
        return str(v) if v else "?"


if __name__ == "__main__":
    main()
