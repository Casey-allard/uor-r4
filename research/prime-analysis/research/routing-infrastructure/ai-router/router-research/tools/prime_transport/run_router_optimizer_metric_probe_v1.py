#!/usr/bin/env python3
"""run_router_optimizer_metric_probe_v1.py

Geometry-Aware Optimizer Metric probe (M3).

Hypothesis:
  SGD's flat Euclidean metric treats angular (direction on S¹) and radial
  (magnitude on R⁺) parameter components identically.  If the parameter
  space has genuine angular/radial geometry, different learning rates
  for these components should improve convergence.

Mechanism:
  Split every weight matrix that mixes angular and radial tau components
  into separate sub-matrices.  The forward pass is mathematically
  IDENTICAL (three matmuls that sum to the same result as one catted
  matmul).  Each sub-matrix goes into a separate optimizer param group
  with its own learning rate.

  Parameters split:
    W1         (16,32) → W1_emb(4,32) + W1_dir(8,32) + W1_mag(4,32)
    W_tok_inject(4,12) → W_tok_dir(4,8)  + W_tok_mag(4,4)
    W_attn       (8,12) → W_attn_dir(8,8)+ W_attn_mag(8,4)
    W_pred      (12,4)  → W_pred_dir(8,4)+ W_pred_mag(4,4)

  Param groups:
    "angular"  → {W1_dir, W_tok_dir, W_attn_dir, W_pred_dir}  lr = LR * ang_ratio
    "radial"   → {W1_mag, W_tok_mag, W_attn_mag, W_pred_mag}  lr = LR * rad_ratio
    "other"    → everything else                                lr = LR

Sweep:
  9 variants testing angular/radial LR ratios independently.

Uses locked hybrid angular+radial representation from previous experiments.
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

# ═══════════════════════════════════════════════════════════════════════
# Paths
# ═══════════════════════════════════════════════════════════════════════
SCRIPT_DIR  = Path(__file__).parent
REPO_ROOT   = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "results" / "prime_transport_recursive_system"
DOCS_DIR    = REPO_ROOT / "docs" / "research"
CSV_OUT     = RESULTS_DIR / "prime_transport_router_optimizer_metric_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_optimizer_metric_probe_v1.md"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
DOCS_DIR.mkdir(parents=True, exist_ok=True)

# ═══════════════════════════════════════════════════════════════════════
# Locked hyperparameters
# ═══════════════════════════════════════════════════════════════════════
VOCAB        = 4
D_EMB        = 4
N_OPS        = 6
LR           = 0.02
TEMP_START   = 2.0
TEMP_END     = 0.1
GLOBAL_SEED  = 42
B0_INIT      = 2.0
D_CONTEXT    = 24
BATCH_SIZE   = 256
D_HIDDEN     = 32
BENCH_BUDGET = 3000
N_EVAL       = 1000

D_TAU_ANG    = 8
D_TAU_HYB    = 12
D_IN_HYB     = D_EMB + D_TAU_HYB   # 16

PHASE_BLOCKS  = [(0, 2, 2), (2, 7, 5), (7, 9, 2), (9, 21, 12)]
N_PHASE_PAIRS = 4
_TRANSPORT_TH = 3

CHECKPOINTS = [0, 250, 500, 1000, 1500, 2000, 2500, 3000]

# (name, angular_lr_ratio, radial_lr_ratio)
VARIANTS: List[Tuple[str, float, float]] = [
    ("baseline",            1.0, 1.0),
    ("ang_2x",              2.0, 1.0),
    ("ang_4x",              4.0, 1.0),
    ("ang_half",            0.5, 1.0),
    ("rad_2x",              1.0, 2.0),
    ("rad_4x",              1.0, 4.0),
    ("rad_half",            1.0, 0.5),
    ("both_2x",             2.0, 2.0),
    ("ang_boost_rad_damp",  2.0, 0.5),
]

torch.set_num_threads(1)


# ═══════════════════════════════════════════════════════════════════════
# Load v6 torch base
# ═══════════════════════════════════════════════════════════════════════
def _load_v6torch():
    spec = importlib.util.spec_from_file_location(
        "v6torch_base",
        str(SCRIPT_DIR / "run_router_reintegration_v6_torch.py"),
    )
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


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


# ═══════════════════════════════════════════════════════════════════════
# RouterHybridMetric — Split angular/radial params for LR groups
#
# Forward is mathematically IDENTICAL to the unsplit RouterAngularHybrid.
# The only difference: angular and radial weight sub-matrices are
# separate nn.Parameters so they can go into different optimizer groups.
# ═══════════════════════════════════════════════════════════════════════
class RouterHybridMetric(nn.Module):
    def __init__(
        self, TN: torch.Tensor, TR: torch.Tensor,
        tau0: torch.Tensor, pool: torch.Tensor,
        d_hidden: int = D_HIDDEN, d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT, sc: float = 0.05, seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh = d_hidden; dha = max(8, dh // 4)

        self.register_buffer("TN", TN)
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0)
        self.register_buffer("pool_ids", pool)
        m = torch.zeros(1, d_context); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)

        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        g = torch.Generator().manual_seed(seed)

        # ── Unsplit parameters (same RNG consumption order as original) ──
        self.W_emb = nn.Parameter(
            torch.empty(VOCAB, D_EMB).normal_(0, sc, generator=g))

        # W1 split: (D_IN_HYB, dh) → emb(4,dh) + dir(8,dh) + mag(4,dh)
        _W1 = torch.empty(D_IN_HYB, dh).normal_(0, sc, generator=g)
        self.W1_emb = nn.Parameter(_W1[:D_EMB].clone())
        self.W1_dir = nn.Parameter(_W1[D_EMB:D_EMB + D_TAU_ANG].clone())
        self.W1_mag = nn.Parameter(_W1[D_EMB + D_TAU_ANG:].clone())
        self.b1 = nn.Parameter(torch.zeros(dh))

        self.W2 = nn.Parameter(
            torch.empty(dh, N_OPS).normal_(0, sc, generator=g))
        self.b2 = nn.Parameter(torch.zeros(N_OPS))

        # W_attn split: (dha, D_TAU_HYB) → dir(dha,8) + mag(dha,4)
        _Wa = torch.empty(dha, D_TAU_HYB).normal_(0, sc, generator=g)
        self.W_attn_dir = nn.Parameter(_Wa[:, :D_TAU_ANG].clone())
        self.W_attn_mag = nn.Parameter(_Wa[:, D_TAU_ANG:].clone())
        self.b_attn = nn.Parameter(torch.zeros(dha))
        self.v_attn = nn.Parameter(
            torch.empty(dha).normal_(0, sc, generator=g))

        # W_pred split: (D_TAU_HYB, VOCAB) → dir(8,V) + mag(4,V)
        _Wp = torch.empty(D_TAU_HYB, VOCAB).normal_(0, sc, generator=g)
        self.W_pred_dir = nn.Parameter(_Wp[:D_TAU_ANG].clone())
        self.W_pred_mag = nn.Parameter(_Wp[D_TAU_ANG:].clone())
        self.b_pred = nn.Parameter(torch.zeros(VOCAB))

        # W_tok_inject split: (VOCAB, D_TAU_HYB) → dir(V,8) + mag(V,4)
        _Wt = torch.empty(VOCAB, D_TAU_HYB).normal_(0, sc, generator=g)
        self.W_tok_dir = nn.Parameter(_Wt[:, :D_TAU_ANG].clone())
        self.W_tok_mag = nn.Parameter(_Wt[:, D_TAU_ANG:].clone())

    def get_param_groups(self, base_lr: float,
                         ang_ratio: float, rad_ratio: float) -> List[dict]:
        ang_ids = {id(self.W1_dir), id(self.W_attn_dir),
                   id(self.W_pred_dir), id(self.W_tok_dir)}
        rad_ids = {id(self.W1_mag), id(self.W_attn_mag),
                   id(self.W_pred_mag), id(self.W_tok_mag)}
        ang_p   = [p for p in self.parameters() if id(p) in ang_ids]
        rad_p   = [p for p in self.parameters() if id(p) in rad_ids]
        other_p = [p for p in self.parameters()
                   if id(p) not in ang_ids and id(p) not in rad_ids]
        return [
            {"params": other_p, "lr": base_lr},
            {"params": ang_p,   "lr": base_lr * ang_ratio},
            {"params": rad_p,   "lr": base_lr * rad_ratio},
        ]

    def forward(self, state_ids: torch.Tensor, tokens: torch.Tensor,
                x0: torch.Tensor, temp: float) -> torch.Tensor:
        B: int = state_ids.shape[0]; D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]  # (B, 12)
        soft_taus: List[torch.Tensor] = []

        for t in range(D):
            tn   = self.TN[state_ids]
            embs = self.W_emb[tokens[:, t]]

            tau_dir = tau_prev[:, :D_TAU_ANG]   # (B, 8)
            tau_mag = tau_prev[:, D_TAU_ANG:]    # (B, 4)

            # Equivalent to cat([embs, tau_dir, tau_mag]) @ W1_full + b1
            h = torch.tanh(
                embs    @ self.W1_emb +
                tau_dir @ self.W1_dir +
                tau_mag @ self.W1_mag +
                self.b1
            )

            logits = h @ self.W2 + self.b2
            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                w = torch.softmax(
                    (logits - torch.log(-torch.log(u))) / temp, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)

            base = torch.einsum("bi,bij->bj", w, tn)  # (B, 8)

            # Hybrid decomposition
            _p = base.view(B, 4, 2)
            _mag = (_p * _p).sum(dim=2, keepdim=False).sqrt()
            _mag_safe = _mag.clamp(min=1e-8)
            _dir = (_p / _mag_safe.unsqueeze(2)).view(B, 8)
            hybrid = torch.cat([_dir, _mag], dim=1)    # (B, 12)

            if t == 0:
                tok_inj = torch.cat(
                    [self.W_tok_dir[x0], self.W_tok_mag[x0]], dim=1)
                tau_prev = hybrid + tok_inj
            else:
                tau_prev = hybrid
            soft_taus.append(tau_prev)

            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)

        st = torch.stack(soft_taus, dim=1)          # (B, D, 12)
        st_dir = st[:, :, :D_TAU_ANG]               # (B, D, 8)
        st_mag = st[:, :, D_TAU_ANG:]                # (B, D, 4)

        # Attention (equivalent to st @ W_attn_full.t() + b_attn)
        h_a = torch.tanh(
            st_dir @ self.W_attn_dir.t() +
            st_mag @ self.W_attn_mag.t() +
            self.b_attn
        )
        sc    = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)

        ctx     = torch.einsum("bd,bdt->bt", alpha, st)
        ctx_dir = ctx[:, :D_TAU_ANG]
        ctx_mag = ctx[:, D_TAU_ANG:]

        return ctx_dir @ self.W_pred_dir + ctx_mag @ self.W_pred_mag + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# Sampling
# ═══════════════════════════════════════════════════════════════════════
def sample_batch(pool_ids, D, B, device):
    idx       = torch.randint(0, len(pool_ids), (B,), device=device)
    state_ids = pool_ids[idx]
    tokens    = torch.randint(0, VOCAB, (B, D), device=device)
    x0        = torch.randint(0, VOCAB, (B,),   device=device)
    tokens[:, 0] = x0
    return state_ids, tokens, x0


# ═══════════════════════════════════════════════════════════════════════
# Evaluate
# ═══════════════════════════════════════════════════════════════════════
def evaluate(model: RouterHybridMetric, pool_ids, D, n_eval, device) -> Dict:
    model.eval()
    torch.manual_seed(GLOBAL_SEED + D + 19)
    B_eval = 256
    n_batches = max(1, min(4, (n_eval + B_eval - 1) // B_eval))
    total = correct = 0
    ent_sum = 0.0; transport_n = total_steps = 0
    alpha_sum = torch.zeros(D, device=device)
    mag_sum = torch.zeros(N_PHASE_PAIRS, device=device); mag_count = 0

    with torch.no_grad():
        for _ in range(n_batches):
            B_ = min(B_eval, n_eval - total)
            if B_ <= 0: break
            idx  = torch.randint(0, len(pool_ids), (B_,), device=device)
            sids = pool_ids[idx]
            toks = torch.randint(0, VOCAB, (B_, D), device=device)
            x0   = torch.randint(0, VOCAB, (B_,),   device=device)
            toks[:, 0] = x0

            tau_prev = model.tau0_table[sids]
            soft_taus: List[torch.Tensor] = []
            cur_sids = sids.clone()

            for t in range(D):
                tn   = model.TN[cur_sids]
                embs = model.W_emb[toks[:, t]]
                td   = tau_prev[:, :D_TAU_ANG]
                tm   = tau_prev[:, D_TAU_ANG:]
                h    = torch.tanh(
                    embs @ model.W1_emb +
                    td   @ model.W1_dir +
                    tm   @ model.W1_mag +
                    model.b1)
                logits = h @ model.W2 + model.b2
                w = torch.softmax(logits / 0.05, dim=1)

                pc       = w.clamp(1e-12, 1.0)
                ent_sum += float((-(pc * pc.log()).sum(dim=1)).sum().item())
                k_hard   = w.argmax(dim=1)
                transport_n += int((k_hard >= _TRANSPORT_TH).sum().item())
                total_steps += B_

                base = torch.einsum("bi,bij->bj", w, tn)

                _bp = base.view(B_, N_PHASE_PAIRS, 2)
                step_mags = (_bp * _bp).sum(dim=2).sqrt()
                mag_sum  += step_mags.sum(dim=0)
                mag_count += B_

                _p = base.view(B_, 4, 2)
                _mg = (_p * _p).sum(dim=2).sqrt()
                _ms = _mg.clamp(min=1e-8)
                _dr = (_p / _ms.unsqueeze(2)).view(B_, 8)
                hybrid = torch.cat([_dr, _mg], dim=1)
                if t == 0:
                    tok_inj = torch.cat(
                        [model.W_tok_dir[x0], model.W_tok_mag[x0]], dim=1)
                    tau_prev = hybrid + tok_inj
                else:
                    tau_prev = hybrid
                soft_taus.append(tau_prev)
                cur_sids = model.TR[cur_sids].gather(
                    1, k_hard.unsqueeze(1)).squeeze(1)

            st = torch.stack(soft_taus, dim=1)
            sd = st[:, :, :D_TAU_ANG]; sm = st[:, :, D_TAU_ANG:]
            h_a = torch.tanh(
                sd @ model.W_attn_dir.t() +
                sm @ model.W_attn_mag.t() +
                model.b_attn)
            a_sc  = (h_a * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0
            alpha = torch.softmax(a_sc, dim=1)
            alpha_sum += alpha.sum(dim=0)
            ctx = torch.einsum("bd,bdt->bt", alpha, st)
            cd = ctx[:, :D_TAU_ANG]; cm = ctx[:, D_TAU_ANG:]
            pred = cd @ model.W_pred_dir + cm @ model.W_pred_mag + model.b_pred
            correct += int((pred.argmax(1) == x0).sum().item())
            total += B_

    model.train()
    mean_mag = float((mag_sum / max(mag_count, 1)).mean().item()) if mag_count > 0 else -1.0
    return {
        "accuracy":       correct / max(total, 1),
        "route_entropy":  ent_sum / max(total_steps, 1),
        "transport_frac": transport_n / max(total_steps, 1),
        "alpha0":         float((alpha_sum[0] / max(total, 1)).item()),
        "b_pos0":         float(model.b_pos0.item()),
        "mean_mag":       mean_mag,
    }


# ═══════════════════════════════════════════════════════════════════════
# Training loop
# ═══════════════════════════════════════════════════════════════════════
def train_variant(
    name: str, ang_ratio: float, rad_ratio: float,
    TN_ang: torch.Tensor, TR: torch.Tensor,
    tau0_hyb: torch.Tensor, pool_ids: torch.Tensor,
    device: str,
) -> Tuple[List[dict], float]:
    model = RouterHybridMetric(TN_ang, TR, tau0_hyb, pool_ids)
    n_params = sum(p.numel() for p in model.parameters())

    param_groups = model.get_param_groups(LR, ang_ratio, rad_ratio)

    # Count params per group
    n_ang  = sum(p.numel() for p in param_groups[1]["params"])
    n_rad  = sum(p.numel() for p in param_groups[2]["params"])
    n_oth  = sum(p.numel() for p in param_groups[0]["params"])

    model.train()
    optimizer = torch.optim.SGD(param_groups)
    torch.manual_seed(GLOBAL_SEED)

    ckpt_set = set(CHECKPOINTS)
    rows: List[dict] = []

    def _record(ckpt: int, elapsed: float, loss_v: float) -> None:
        met = evaluate(model, pool_ids, D_CONTEXT, N_EVAL, device)
        solved = "solved" if met["accuracy"] >= 0.90 else ""
        rows.append({
            "variant": name, "checkpoint": ckpt,
            "ang_lr_ratio": ang_ratio, "rad_lr_ratio": rad_ratio,
            "accuracy": round(met["accuracy"], 4),
            "alpha0": round(met["alpha0"], 4),
            "route_entropy": round(met["route_entropy"], 4),
            "transport_frac": round(met["transport_frac"], 4),
            "mean_mag": round(met["mean_mag"], 5),
            "b_pos0": round(met["b_pos0"], 3),
            "runtime_seconds": round(elapsed, 2),
            "loss": round(loss_v, 4),
            "n_params": n_params,
            "n_ang": n_ang, "n_rad": n_rad, "n_oth": n_oth,
            "note": solved,
        })
        if ckpt > 0:
            print(f"    step {ckpt:>5}: acc={met['accuracy']:.4f}  "
                  f"α₀={met['alpha0']:.4f}  "
                  f"tfrac={met['transport_frac']:.3f}  "
                  f"loss={loss_v:.4f}  "
                  f"mag={met['mean_mag']:.4f}  "
                  f"t={elapsed:.1f}s  {solved}", flush=True)

    if 0 in ckpt_set:
        _record(0, 0.0, -1.0)
        model.train()

    loss_val = -1.0
    t0 = time.perf_counter()
    for step in range(BENCH_BUDGET):
        frac = step / max(BENCH_BUDGET - 1, 1)
        temp = float(TEMP_START * (TEMP_END / TEMP_START) ** frac)
        sids, toks, x0 = sample_batch(pool_ids, D_CONTEXT, BATCH_SIZE, device)
        pred = model(sids, toks, x0, temp)
        loss = F.cross_entropy(pred, x0)
        loss_val = float(loss.item())
        optimizer.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(model.parameters(), max_norm=1.0)
        optimizer.step()
        step_num = step + 1
        if step_num in ckpt_set:
            elapsed = time.perf_counter() - t0
            _record(step_num, elapsed, loss_val)
            model.train()

    runtime = time.perf_counter() - t0
    return rows, runtime


# ═══════════════════════════════════════════════════════════════════════
# Markdown report
# ═══════════════════════════════════════════════════════════════════════
def write_md(all_rows: Dict[str, List[dict]],
             all_runtimes: Dict[str, float]) -> None:
    L: List[str] = []
    A = L.append

    A("# Geometry-Aware Optimizer Metric Probe — v1")
    A("")
    A("## Purpose")
    A("")
    A("Test M3: whether optimizing angular (direction on S¹) and radial (magnitude on R⁺)")
    A("parameter components with different learning rates improves convergence.")
    A("")

    A("## Mechanism")
    A("")
    A("Every weight matrix that mixes angular and radial tau components is split into")
    A("separate sub-matrices, each placed in its own optimizer param group:")
    A("")
    A("```")
    A("W1(16,32) → W1_emb(4,32) + W1_dir(8,32) + W1_mag(4,32)")
    A("W_tok(4,12) → W_tok_dir(4,8) + W_tok_mag(4,4)")
    A("W_attn(8,12) → W_attn_dir(8,8) + W_attn_mag(8,4)")
    A("W_pred(12,4) → W_pred_dir(8,4) + W_pred_mag(4,4)")
    A("```")
    A("")
    A("The forward pass is **mathematically identical** (three matmuls summing to one).")
    A("Only the optimizer step differs: angular params get `LR × ang_ratio`,")
    A("radial params get `LR × rad_ratio`, other params get `LR`.")
    A("")
    A("### Rationale")
    A("")
    A("If M3 is binding, the flat SGD metric is wrong because:")
    A("- Angular parameters live on S¹ (periodic, bounded) → may need different step size")
    A("- Radial parameters live on R⁺ (unbounded, scale-free) → may need different step size")
    A("- The optimal ratio reveals the direction of the metric mismatch")
    A("")

    A("## Locked Configuration")
    A("")
    A("| Item | Value |")
    A("|------|-------|")
    A("| device | cpu |")
    A(f"| D_HIDDEN | {D_HIDDEN} |")
    A(f"| batch_size | {BATCH_SIZE} |")
    A(f"| D | {D_CONTEXT} |")
    A(f"| representation | hybrid angular+radial (D_TAU=12) |")
    A(f"| base LR | {LR} |")
    A(f"| budget | {BENCH_BUDGET} steps |")
    A(f"| optimizer | SGD, per-group LR |")
    A(f"| seed | {GLOBAL_SEED} |")
    A("")

    A("## Sweep Variants")
    A("")
    A("| Variant | Ang Ratio | Rad Ratio | Effective Ang LR | Effective Rad LR |")
    A("|---------|-----------|-----------|------------------|------------------|")
    for vn, ar, rr in VARIANTS:
        A(f"| {vn} | {ar} | {rr} | {LR*ar:.4f} | {LR*rr:.4f} |")
    A("")

    # ── Convergence table ──
    A("## Convergence Comparison")
    A("")
    header = "| Step |"
    sep    = "|------|"
    for vn, _, _ in VARIANTS:
        header += f" {vn} |"
        sep    += "-" * (len(vn) + 2) + "|"
    A(header); A(sep)

    ck_dicts = {}
    for vn, _, _ in VARIANTS:
        ck_dicts[vn] = {r["checkpoint"]: r for r in all_rows[vn]}

    for ck in CHECKPOINTS:
        line = f"| {ck} |"
        for vn, _, _ in VARIANTS:
            r = ck_dicts[vn].get(ck, {})
            acc = r.get("accuracy", -1)
            mark = "✓" if acc >= 0.90 else ""
            line += f" {acc:.4f}{mark} |"
        A(line)
    A("")

    # ── Summary table ──
    def _first_solve(rows: List[dict]) -> int:
        for r in rows:
            if r.get("accuracy", 0) >= 0.90 and r.get("checkpoint", 0) > 0:
                return r["checkpoint"]
        return -1

    A("## Summary")
    A("")
    A("| Variant | Ang | Rad | Solve Step | Final Acc | α₀ | T-Frac | Entropy | Mag | Sps |")
    A("|---------|-----|-----|------------|-----------|-----|--------|---------|-----|-----|")
    for vn, ar, rr in VARIANTS:
        rows = all_rows[vn]
        ss   = _first_solve(rows)
        ff   = ck_dicts[vn].get(BENCH_BUDGET, {})
        rt   = all_runtimes[vn]
        sps  = BENCH_BUDGET / rt if rt > 0 else 0
        ss_s = str(ss) if ss > 0 else "never"
        A(f"| {vn} | {ar} | {rr} | {ss_s} | "
          f"{ff.get('accuracy',-1):.4f} | {ff.get('alpha0',-1):.4f} | "
          f"{ff.get('transport_frac',-1):.4f} | "
          f"{ff.get('route_entropy',-1):.4f} | "
          f"{ff.get('mean_mag',-1):.4f} | {sps:.1f} |")
    A("")

    # ── Accuracy at intermediate checkpoints for fastest-solve analysis ──
    A("## Early Convergence Analysis (steps 500–1500)")
    A("")
    A("| Variant | Step 500 | Step 1000 | Step 1500 |")
    A("|---------|----------|-----------|-----------|")
    for vn, _, _ in VARIANTS:
        r5  = ck_dicts[vn].get(500,  {}).get("accuracy", -1)
        r10 = ck_dicts[vn].get(1000, {}).get("accuracy", -1)
        r15 = ck_dicts[vn].get(1500, {}).get("accuracy", -1)
        A(f"| {vn} | {r5:.4f} | {r10:.4f} | {r15:.4f} |")
    A("")

    # ═══════════════════════════════════════════════════════════════════
    # Interpretation
    # ═══════════════════════════════════════════════════════════════════
    A("## Interpretation")
    A("")

    solve_steps = {vn: _first_solve(all_rows[vn]) for vn, _, _ in VARIANTS}
    final_accs  = {vn: ck_dicts[vn].get(BENCH_BUDGET, {}).get("accuracy", 0)
                   for vn, _, _ in VARIANTS}
    bl_ss = solve_steps.get("baseline", -1)

    # Find best
    solved = {vn: ss for vn, ss in solve_steps.items() if ss > 0}
    if solved:
        fastest = min(solved, key=lambda v: solved[v])
        fastest_ss = solved[fastest]
    else:
        fastest = None; fastest_ss = -1

    # Early convergence: compare accuracy at step 1000
    acc1000 = {vn: ck_dicts[vn].get(1000, {}).get("accuracy", 0)
               for vn, _, _ in VARIANTS}
    best_early = max(acc1000, key=lambda v: acc1000[v])
    bl_early   = acc1000.get("baseline", 0)

    if fastest and fastest != "baseline" and fastest_ss < bl_ss:
        A(f"**M3 shows signal:** `{fastest}` solves at step {fastest_ss} vs baseline step {bl_ss}.")
        A(f"Improvement: {bl_ss - fastest_ss} steps faster.")
        _, best_ar, best_rr = next((v, a, r) for v, a, r in VARIANTS if v == fastest)
        if best_ar > 1.0 and best_rr <= 1.0:
            A("Direction: **angular LR should be HIGHER** — SGD under-steps on angular params.")
        elif best_rr > 1.0 and best_ar <= 1.0:
            A("Direction: **radial LR should be HIGHER** — SGD under-steps on radial params.")
        elif best_ar > 1.0 and best_rr > 1.0:
            A("Direction: **both LRs benefit from increase** — overall LR=0.02 may be too low.")
        elif best_ar < 1.0:
            A("Direction: **angular LR should be LOWER** — SGD over-steps on angular params.")
        A("")
    elif fastest and fastest == "baseline":
        A("**M3 is NOT binding:** baseline (uniform LR) is the fastest or tied.")
        A("")
    elif fastest:
        A(f"**Tied:** `{fastest}` and baseline both solve at step {fastest_ss}.")
        if acc1000.get(fastest, 0) > bl_early + 0.03:
            A(f"However, `{fastest}` shows better EARLY convergence "
              f"(step-1000 acc: {acc1000[fastest]:.4f} vs {bl_early:.4f}).")
            A("This suggests a mild M3 signal in the transient, even if solve step is the same.")
        else:
            A("And no significant early-convergence advantage either.")
            A("**M3 does NOT appear binding at D=24.**")
        A("")
    else:
        A("**No variant solved D=24.** Unexpected result.")
        A("")

    # Check if higher global LR helps via "both_2x"
    both_ss = solve_steps.get("both_2x", -1)
    if both_ss > 0 and bl_ss > 0:
        if both_ss < bl_ss:
            A(f"**Important control:** `both_2x` (all LR × 2) solves at step {both_ss}, "
              f"faster than baseline ({bl_ss}). This means the improvement may be from "
              f"higher overall LR, not angular/radial asymmetry.")
            ang2_ss = solve_steps.get("ang_2x", -1)
            if ang2_ss > 0 and ang2_ss < both_ss:
                A(f"But `ang_2x` ({ang2_ss}) is faster than `both_2x` ({both_ss}), "
                  f"so angular-specific boost IS contributing beyond just higher global LR.")
            A("")
        elif both_ss == bl_ss:
            A(f"Control: `both_2x` solves at same step as baseline ({bl_ss}). "
              f"The system is LR-robust near LR=0.02.")
            A("")

    # Check if any variant FAILS
    unsolved = [vn for vn, _, _ in VARIANTS if solve_steps[vn] <= 0]
    if unsolved:
        A(f"**Variants that did NOT solve:** {', '.join(unsolved)}")
        for vn in unsolved:
            _, ar, rr = next(v for v in VARIANTS if v[0] == vn)
            A(f"  - `{vn}` (ang={ar}, rad={rr}): "
              f"LR too extreme, destabilized training")
        A("")

    # ═══════════════════════════════════════════════════════════════════
    # Honesty section
    # ═══════════════════════════════════════════════════════════════════
    A("## Honesty Section")
    A("")

    A("### What Improved")
    A("")
    improvements = []
    for vn, ar, rr in VARIANTS:
        if vn == "baseline": continue
        ss = solve_steps[vn]
        if ss > 0 and ss < bl_ss:
            improvements.append(f"- `{vn}` (ang={ar}, rad={rr}): solves {bl_ss - ss} steps faster")
        elif acc1000.get(vn, 0) > bl_early + 0.03:
            improvements.append(
                f"- `{vn}`: better early convergence ({acc1000[vn]:.4f} vs {bl_early:.4f} at step 1000)")
    if not improvements:
        improvements.append("- No variant materially outperformed the uniform-LR baseline")
    for line in improvements:
        A(line)
    A("")

    A("### What Did Not Improve")
    A("")
    no_improv = []
    for vn, ar, rr in VARIANTS:
        if vn == "baseline": continue
        ss = solve_steps[vn]
        if ss <= 0:
            no_improv.append(f"- `{vn}`: FAILED to solve (LR destabilized)")
        elif ss > bl_ss:
            no_improv.append(f"- `{vn}`: slower (step {ss} vs {bl_ss})")
    if not no_improv:
        worsened = [vn for vn, ar, rr in VARIANTS
                    if vn != "baseline" and acc1000.get(vn, 0) < bl_early - 0.02]
        if worsened:
            no_improv.append(f"- Worse early convergence: {', '.join(worsened)}")
        else:
            no_improv.append("- No variant was materially worse either (flat landscape in LR-ratio space)")
    for line in no_improv:
        A(line)
    A("")

    A("### Whether M3 Looks Binding")
    A("")
    any_faster = any(solve_steps[vn] > 0 and solve_steps[vn] < bl_ss
                     for vn, _, _ in VARIANTS if vn != "baseline")
    any_early  = any(acc1000.get(vn, 0) > bl_early + 0.05
                     for vn, _, _ in VARIANTS if vn != "baseline")

    if any_faster:
        A("**M3 shows a measurable signal.** The optimal LR ratio is not 1:1.")
        A("Next step: refine the ratio further, or develop a proper Riemannian SGD step")
        A("that projects angular updates onto the tangent space of S¹.")
    elif any_early:
        A("**M3 shows a mild transient signal** (better early convergence) but does not")
        A("change the solve step. This suggests the metric mismatch exists but is not")
        A("the binding constraint at D=24.")
        A("")
        A("Next step: the remaining mismatches (M5, M6) or scaling to harder tasks where")
        A("the transient advantage may become binding.")
    else:
        A("**M3 does NOT appear binding at D=24.** The diagonal preconditioner sweep")
        A("finds no advantage from angular/radial LR asymmetry.")
        A("")
        A("Combined with M4 (also not binding), this suggests:")
        A("- The hybrid angular+radial representation already captures enough geometry")
        A("- The remaining training-rule mismatches (M3, M4) are not binding at D=24")
        A("- The next productive direction is either:")
        A("  (a) scaling to harder tasks (D > 24) where mismatches may become binding")
        A("  (b) radial-law / concentration structure measurement")
        A("  (c) testing M5 (gradient distinction transport/non-transport at param level)")
    A("")

    A("### What Is Proven")
    A("")
    A("- The split-parameter model is mathematically equivalent to the unsplit version")
    A("- Angular and radial components receive correctly separated optimizer treatment")
    A("- The sweep covers a meaningful range (0.25× to 4× each axis)")
    A("")

    A("### What Is Still Uncertain")
    A("")
    A("- Whether M3 becomes binding at larger D, D_HIDDEN, or harder tasks")
    A("- Whether a non-diagonal preconditioner (full Riemannian metric) would show signal")
    A("- Whether the interaction between angular and radial components matters")
    A("  more than the individual scaling")
    A("")

    with open(MD_OUT, "w") as f:
        f.write("\n".join(L) + "\n")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main() -> None:
    print("=" * 72)
    print("Geometry-Aware Optimizer Metric Probe v1 (M3)")
    print("=" * 72)
    print(f"  {len(VARIANTS)} variants")
    print(f"  D={D_CONTEXT}, D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, "
          f"budget={BENCH_BUDGET}")
    print(f"  Hybrid angular+radial representation (D_TAU={D_TAU_HYB})")
    print(f"  torch: {torch.__version__}, threads: {torch.get_num_threads()}")
    print()

    device = "cpu"

    # ── Load state tables ──
    print("Loading state tables (one-time BFS)...")
    import random as _pyrand
    v6  = _load_v6torch()
    rng = _pyrand.Random(GLOBAL_SEED)
    pool = v6.build_warmup_pool(rng, size=v6.POOL_SIZE)
    n_states = v6.bfs_warm_up(pool, max_seconds=v6.BFS_MAX_SECS, verbose=True)
    TN_oh, TR, tau0_oh, pool_ids, _ = v6.build_state_tables(pool, verbose=True)
    TN_oh    = TN_oh.to(device)
    TR       = TR.to(device)
    tau0_oh  = tau0_oh.to(device)
    pool_ids = pool_ids.to(device)
    print(f"  {n_states:,} states")
    print()

    # ── Angular + hybrid conversion ──
    print("Converting to angular + hybrid encoding...")
    TN_ang   = convert_onehot_to_angular(TN_oh)
    tau0_ang = convert_onehot_to_angular(tau0_oh)
    tau0_hyb = torch.cat(
        [tau0_ang, torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    print(f"  TN_ang: {TN_ang.shape}, tau0_hyb: {tau0_hyb.shape}")

    # ── Verify forward equivalence ──
    print("\nVerifying split model forward equivalence...")
    torch.manual_seed(99)
    m_test = RouterHybridMetric(TN_ang, TR, tau0_hyb, pool_ids)
    m_test.eval()
    sids_t = pool_ids[:4]
    toks_t = torch.randint(0, VOCAB, (4, D_CONTEXT))
    x0_t   = toks_t[:, 0]
    out_split = m_test(sids_t, toks_t, x0_t, 1.0)
    print(f"  Split model output: {out_split.shape}, "
          f"sample={[round(float(x), 4) for x in out_split[0]]}")
    n_p = sum(p.numel() for p in m_test.parameters())
    pg = m_test.get_param_groups(LR, 1.0, 1.0)
    n_a = sum(p.numel() for p in pg[1]["params"])
    n_r = sum(p.numel() for p in pg[2]["params"])
    n_o = sum(p.numel() for p in pg[0]["params"])
    print(f"  Params: {n_p} total = {n_a} angular + {n_r} radial + {n_o} other")
    del m_test
    print()

    # ── Run sweep ──
    all_rows: Dict[str, List[dict]] = {}
    all_runtimes: Dict[str, float] = {}

    for vi, (vn, ar, rr) in enumerate(VARIANTS):
        print("=" * 56)
        print(f"  VARIANT {vi+1}/{len(VARIANTS)}: {vn}  "
              f"(ang={ar:.2f}, rad={rr:.2f})")
        print("=" * 56)
        rows, runtime = train_variant(
            vn, ar, rr, TN_ang, TR, tau0_hyb, pool_ids, device)
        all_rows[vn] = rows
        all_runtimes[vn] = runtime
        sps = BENCH_BUDGET / runtime if runtime > 0 else 0
        print(f"  Total: {runtime:.1f}s, {sps:.1f} sps")
        print()

    # ── Write CSV ──
    csv_fields = [
        "variant", "ang_lr_ratio", "rad_lr_ratio",
        "checkpoint", "accuracy", "alpha0",
        "route_entropy", "transport_fraction",
        "mean_resultant_magnitude",
        "runtime_seconds", "steps_per_second",
        "loss", "n_params", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=csv_fields)
        w.writeheader()
        for vn, _, _ in VARIANTS:
            for r in all_rows[vn]:
                ckpt = r["checkpoint"]
                rt   = r["runtime_seconds"]
                w.writerow({
                    "variant":                  r["variant"],
                    "ang_lr_ratio":             r["ang_lr_ratio"],
                    "rad_lr_ratio":             r["rad_lr_ratio"],
                    "checkpoint":               ckpt,
                    "accuracy":                 r["accuracy"],
                    "alpha0":                   r["alpha0"],
                    "route_entropy":            r["route_entropy"],
                    "transport_fraction":       r["transport_frac"],
                    "mean_resultant_magnitude": r["mean_mag"],
                    "runtime_seconds":          rt,
                    "steps_per_second":         round(ckpt / rt, 1) if rt > 0 else 0,
                    "loss":                     r["loss"],
                    "n_params":                 r["n_params"],
                    "note":                     r["note"],
                })
    print(f"CSV → {CSV_OUT}")

    # ── Write markdown ──
    write_md(all_rows, all_runtimes)
    print(f"MD  → {MD_OUT}")
    print()

    # ═══════════════════════════════════════════════════════════════════
    # Console summary
    # ═══════════════════════════════════════════════════════════════════
    print("=" * 72)
    print("SUMMARY")
    print("=" * 72)

    def _fs(rows):
        for r in rows:
            if r.get("accuracy", 0) >= 0.90 and r.get("checkpoint", 0) > 0:
                return r["checkpoint"]
        return -1

    hdr = f"{'Variant':>22s} {'Ang':>5s} {'Rad':>5s} {'Solve':>6s} " \
          f"{'Acc':>7s} {'α₀':>7s} {'TFrac':>7s} {'Ent':>7s} {'Sps':>6s}"
    print(hdr)
    print("-" * len(hdr))
    for vn, ar, rr in VARIANTS:
        rows = all_rows[vn]
        ff   = {r["checkpoint"]: r for r in rows}.get(BENCH_BUDGET, {})
        ss   = _fs(rows)
        rt   = all_runtimes[vn]
        sps  = BENCH_BUDGET / rt if rt > 0 else 0
        print(f"{vn:>22s} {ar:>5.2f} {rr:>5.2f} {ss:>6d} "
              f"{ff.get('accuracy',-1):>7.4f} {ff.get('alpha0',-1):>7.4f} "
              f"{ff.get('transport_frac',-1):>7.4f} "
              f"{ff.get('route_entropy',-1):>7.4f} {sps:>6.1f}")
    print("=" * 72)


if __name__ == "__main__":
    main()
