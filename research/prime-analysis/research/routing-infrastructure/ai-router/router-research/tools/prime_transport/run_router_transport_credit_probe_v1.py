#!/usr/bin/env python3
"""run_router_transport_credit_probe_v1.py

Transport-Aware Credit Assignment probe (M4).

Hypothesis:
  The current backward pass treats transport operators (ops 3-5) and
  non-transport operators (ops 0-2) identically.  If the geometric
  adjoint of transport differs from the Euclidean adjoint, the learning
  rule assigns incorrect credit to transport operator selection.

Mechanism:
  A gradient hook on the operator-selection logits scales the gradient
  for transport channels (indices 3,4,5) by a factor gamma.

    gamma = 1.0  →  standard backward (baseline)
    gamma > 1.0  →  amplify transport credit
    gamma < 1.0  →  dampen transport credit
    gamma = 0.0  →  block transport credit entirely

  The forward pass is IDENTICAL for all gamma values.
  Only the backward gradient through the transport logits changes.

  The hook also records gradient norms (pre-scaling) for transport vs
  non-transport channels, providing direct evidence about the natural
  gradient distribution.

Sweep:
  gamma ∈ {0.0, 0.25, 0.5, 1.0, 1.5, 2.0, 3.0}

Uses locked hybrid angular+radial representation from previous experiment.
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
CSV_OUT     = RESULTS_DIR / "prime_transport_router_transport_credit_probe_v1.csv"
MD_OUT      = DOCS_DIR    / "prime_transport_router_transport_credit_probe_v1.md"
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
_TRANSPORT_TH = 3   # ops with index >= 3 are transport

CHECKPOINTS = [0, 250, 500, 1000, 1500, 2000, 2500, 3000]

GAMMA_VALUES = [0.0, 0.25, 0.5, 1.0, 1.5, 2.0, 3.0]

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


# ═══════════════════════════════════════════════════════════════════════
# Angular conversion
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


# ═══════════════════════════════════════════════════════════════════════
# RouterHybridTC — Hybrid router with transport credit gradient hook
#
# Forward: identical for all gamma values
# Backward: gradient on transport logit channels scaled by gamma
# ═══════════════════════════════════════════════════════════════════════
class RouterHybridTC(nn.Module):
    def __init__(
        self, TN: torch.Tensor, TR: torch.Tensor,
        tau0: torch.Tensor, pool: torch.Tensor,
        gamma_transport: float = 1.0,
        d_hidden: int = D_HIDDEN, d_context: int = D_CONTEXT,
        b0_init: float = B0_INIT, sc: float = 0.05, seed: int = GLOBAL_SEED,
    ) -> None:
        super().__init__()
        dh = d_hidden; dha = max(8, dh // 4)
        d_tau = D_TAU_HYB; d_in = D_IN_HYB
        self.gamma_transport = gamma_transport

        # Gradient tracking accumulators (not parameters)
        self.grad_transport_sum: float = 0.0
        self.grad_nontransport_sum: float = 0.0
        self.grad_hook_count: int = 0

        self.register_buffer("TN", TN)           # (N, 6, 8) angular
        self.register_buffer("TR", TR)
        self.register_buffer("tau0_table", tau0)  # (N, 12) hybrid
        self.register_buffer("pool_ids", pool)
        m = torch.zeros(1, d_context); m[0, 0] = 1.0
        self.register_buffer("pos0_mask", m)

        self.b_pos0 = nn.Parameter(torch.tensor(b0_init))
        g = torch.Generator().manual_seed(seed)
        def rp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.empty(*s).normal_(0.0, sc, generator=g))
        def zp(*s: int) -> nn.Parameter:
            return nn.Parameter(torch.zeros(*s))
        self.W_emb = rp(VOCAB, D_EMB)
        self.W1 = rp(d_in, dh);   self.b1 = zp(dh)
        self.W2 = rp(dh, N_OPS);  self.b2 = zp(N_OPS)
        self.W_attn = rp(dha, d_tau); self.b_attn = zp(dha); self.v_attn = rp(dha)
        self.W_pred = rp(d_tau, VOCAB); self.b_pred = zp(VOCAB)
        self.W_tok_inject = rp(VOCAB, d_tau)

    def _make_logit_hook(self):
        """Return a closure for register_hook on the Gumbel-logits tensor."""
        gamma = self.gamma_transport

        def hook(grad: torch.Tensor) -> torch.Tensor:
            # Track pre-scaling norms
            self.grad_nontransport_sum += float(grad[:, :_TRANSPORT_TH].norm().item())
            self.grad_transport_sum    += float(grad[:, _TRANSPORT_TH:].norm().item())
            self.grad_hook_count       += 1
            if gamma == 1.0:
                return grad          # no-op, avoid clone overhead
            scaled = grad.clone()
            scaled[:, _TRANSPORT_TH:] *= gamma
            return scaled

        return hook

    def reset_grad_stats(self) -> None:
        self.grad_transport_sum = 0.0
        self.grad_nontransport_sum = 0.0
        self.grad_hook_count = 0

    def get_grad_stats(self) -> Dict:
        n = max(self.grad_hook_count, 1)
        return {
            "mean_grad_transport":    self.grad_transport_sum / n,
            "mean_grad_nontransport": self.grad_nontransport_sum / n,
            "grad_ratio_t_over_nt":   (self.grad_transport_sum / n) /
                                      max(self.grad_nontransport_sum / n, 1e-12),
            "hook_calls": self.grad_hook_count,
        }

    def forward(self, state_ids: torch.Tensor, tokens: torch.Tensor,
                x0: torch.Tensor, temp: float) -> torch.Tensor:
        B: int = state_ids.shape[0]; D: int = tokens.shape[1]
        tau_prev: torch.Tensor = self.tau0_table[state_ids]  # (B, 12)
        soft_taus: List[torch.Tensor] = []

        for t in range(D):
            tn   = self.TN[state_ids]             # (B, 6, 8)
            embs = self.W_emb[tokens[:, t]]       # (B, 4)
            h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ self.W1 + self.b1)
            logits = h @ self.W2 + self.b2        # (B, 6)

            if self.training:
                u = torch.rand_like(logits).clamp_(1e-20, 1.0)
                gumbel_logits = (logits - torch.log(-torch.log(u))) / temp

                # ── Transport credit hook (backward-only modification) ──
                gumbel_logits.register_hook(self._make_logit_hook())

                w = torch.softmax(gumbel_logits, dim=1)
            else:
                w = torch.softmax(logits / 0.05, dim=1)

            base = torch.einsum("bi,bij->bj", w, tn)   # (B, 8)

            # Hybrid decomposition: direction(8) + magnitude(4)
            _p = base.view(B, 4, 2)
            _mag = (_p * _p).sum(dim=2, keepdim=False).sqrt()
            _mag_safe = _mag.clamp(min=1e-8)
            _dir = (_p / _mag_safe.unsqueeze(2)).view(B, 8)
            hybrid = torch.cat([_dir, _mag], dim=1)      # (B, 12)

            tau_prev = (hybrid + self.W_tok_inject[x0]) if t == 0 else hybrid
            soft_taus.append(tau_prev)

            state_ids = self.TR[state_ids].gather(
                1, torch.argmax(w, dim=1).unsqueeze(1)).squeeze(1)

        st   = torch.stack(soft_taus, dim=1)  # (B, D, 12)
        h_a  = torch.tanh(st @ self.W_attn.t() + self.b_attn)
        sc   = (h_a * self.v_attn).sum(dim=-1) + self.pos0_mask * self.b_pos0
        alpha = torch.softmax(sc, dim=1)
        return torch.einsum("bd,bdt->bt", alpha, st) @ self.W_pred + self.b_pred


# ═══════════════════════════════════════════════════════════════════════
# Sampling
# ═══════════════════════════════════════════════════════════════════════
def sample_batch(
    pool_ids: torch.Tensor, D: int, B: int, device: str,
) -> Tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    idx       = torch.randint(0, len(pool_ids), (B,), device=device)
    state_ids = pool_ids[idx]
    tokens    = torch.randint(0, VOCAB, (B, D), device=device)
    x0        = torch.randint(0, VOCAB, (B,),   device=device)
    tokens[:, 0] = x0
    return state_ids, tokens, x0


# ═══════════════════════════════════════════════════════════════════════
# Evaluation (no hooks, no gamma — pure forward)
# ═══════════════════════════════════════════════════════════════════════
def evaluate(
    model: RouterHybridTC,
    pool_ids: torch.Tensor,
    D: int,
    n_eval: int,
    device: str,
) -> Dict:
    model.eval()
    torch.manual_seed(GLOBAL_SEED + D + 17)
    B_eval = 256
    n_batches = max(1, min(4, (n_eval + B_eval - 1) // B_eval))
    total = correct = 0
    ent_sum = 0.0
    transport_n = total_steps = 0
    alpha_sum = torch.zeros(D, device=device)

    # Per-operator usage tracking
    op_counts = torch.zeros(N_OPS, device=device)

    # Magnitude tracking
    mag_sum    = torch.zeros(N_PHASE_PAIRS, device=device)
    mag_count  = 0

    with torch.no_grad():
        for _ in range(n_batches):
            B_ = min(B_eval, n_eval - total)
            if B_ <= 0:
                break
            idx  = torch.randint(0, len(pool_ids), (B_,), device=device)
            sids = pool_ids[idx]
            toks = torch.randint(0, VOCAB, (B_, D), device=device)
            x0   = torch.randint(0, VOCAB, (B_,),   device=device)
            toks[:, 0] = x0

            tau_prev  = model.tau0_table[sids]
            soft_taus: List[torch.Tensor] = []
            cur_sids  = sids.clone()

            for t in range(D):
                tn   = model.TN[cur_sids]
                embs = model.W_emb[toks[:, t]]
                h    = torch.tanh(torch.cat([embs, tau_prev], dim=1) @ model.W1 + model.b1)
                logits = h @ model.W2 + model.b2
                w    = torch.softmax(logits / 0.05, dim=1)

                pc       = w.clamp(1e-12, 1.0)
                ent_sum += float((-(pc * pc.log()).sum(dim=1)).sum().item())
                k_hard   = w.argmax(dim=1)
                transport_n += int((k_hard >= _TRANSPORT_TH).sum().item())
                total_steps += B_

                # Per-op counts
                for op_idx in range(N_OPS):
                    op_counts[op_idx] += int((k_hard == op_idx).sum().item())

                base = torch.einsum("bi,bij->bj", w, tn)

                # Magnitude tracking
                _bp = base.view(B_, N_PHASE_PAIRS, 2)
                step_mags = (_bp * _bp).sum(dim=2).sqrt()
                mag_sum   += step_mags.sum(dim=0)
                mag_count += B_

                # Hybrid decomposition
                _p = base.view(B_, 4, 2)
                _mag = (_p * _p).sum(dim=2).sqrt()
                _mag_safe = _mag.clamp(min=1e-8)
                _dir = (_p / _mag_safe.unsqueeze(2)).view(B_, 8)
                hybrid = torch.cat([_dir, _mag], dim=1)
                tau_prev = (hybrid + model.W_tok_inject[x0]) if t == 0 else hybrid
                soft_taus.append(tau_prev)

                cur_sids = model.TR[cur_sids].gather(
                    1, k_hard.unsqueeze(1)).squeeze(1)

            st   = torch.stack(soft_taus, dim=1)
            h_a  = torch.tanh(st @ model.W_attn.t() + model.b_attn)
            a_sc = (h_a * model.v_attn).sum(dim=-1) + model.pos0_mask * model.b_pos0
            alpha = torch.softmax(a_sc, dim=1)
            alpha_sum += alpha.sum(dim=0)
            pred = torch.einsum("bd,bdt->bt", alpha, st) @ model.W_pred + model.b_pred
            correct += int((pred.argmax(1) == x0).sum().item())
            total   += B_

    model.train()

    mean_mag = float((mag_sum / max(mag_count, 1)).mean().item()) if mag_count > 0 else -1.0
    op_frac  = op_counts / max(float(op_counts.sum().item()), 1.0)

    return {
        "accuracy":       correct / max(total, 1),
        "route_entropy":  ent_sum / max(total_steps, 1),
        "transport_frac": transport_n / max(total_steps, 1),
        "alpha0":         float((alpha_sum[0] / max(total, 1)).item()),
        "b_pos0":         float(model.b_pos0.item()),
        "mean_mag":       mean_mag,
        "op_frac":        [round(float(x.item()), 4) for x in op_frac],
    }


# ═══════════════════════════════════════════════════════════════════════
# Training loop for one gamma value
# ═══════════════════════════════════════════════════════════════════════
def train_variant(
    gamma: float,
    TN_ang: torch.Tensor, TR: torch.Tensor,
    tau0_hyb: torch.Tensor, pool_ids: torch.Tensor,
    device: str,
) -> Tuple[List[dict], float, Dict]:
    variant = f"hybrid_gamma_{gamma:.2f}"

    model = RouterHybridTC(
        TN_ang, TR, tau0_hyb, pool_ids,
        gamma_transport=gamma,
    )
    n_params = sum(p.numel() for p in model.parameters())

    model.train()
    optimizer = torch.optim.SGD(model.parameters(), lr=LR)
    torch.manual_seed(GLOBAL_SEED)

    ckpt_set = set(CHECKPOINTS)
    rows: List[dict] = []

    def _record(ckpt: int, elapsed: float, loss_v: float) -> None:
        met = evaluate(model, pool_ids, D_CONTEXT, N_EVAL, device)
        gstat = model.get_grad_stats()
        solved = "solved" if met["accuracy"] >= 0.90 else ""
        rows.append({
            "variant": variant, "checkpoint": ckpt,
            "accuracy": round(met["accuracy"], 4),
            "alpha0": round(met["alpha0"], 4),
            "route_entropy": round(met["route_entropy"], 4),
            "transport_frac": round(met["transport_frac"], 4),
            "mean_mag": round(met["mean_mag"], 5),
            "op_frac": met["op_frac"],
            "b_pos0": round(met["b_pos0"], 3),
            "runtime_seconds": round(elapsed, 2),
            "loss": round(loss_v, 4),
            "n_params": n_params,
            "gamma": gamma,
            "mean_grad_transport": round(gstat["mean_grad_transport"], 6),
            "mean_grad_nontransport": round(gstat["mean_grad_nontransport"], 6),
            "grad_ratio": round(gstat["grad_ratio_t_over_nt"], 4),
            "note": solved,
        })
        if ckpt > 0:
            print(f"    step {ckpt:>5}: acc={met['accuracy']:.4f}  "
                  f"α₀={met['alpha0']:.4f}  tfrac={met['transport_frac']:.3f}  "
                  f"loss={loss_v:.4f}  "
                  f"g_t={gstat['mean_grad_transport']:.4f}  "
                  f"g_nt={gstat['mean_grad_nontransport']:.4f}  "
                  f"t={elapsed:.1f}s  {solved}", flush=True)
        model.reset_grad_stats()

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
    final_gstats = model.get_grad_stats()
    return rows, runtime, final_gstats


# ═══════════════════════════════════════════════════════════════════════
# Markdown report writer
# ═══════════════════════════════════════════════════════════════════════
def write_md(all_variant_rows: Dict[float, List[dict]],
             all_runtimes: Dict[float, float]) -> None:
    L: List[str] = []
    A = L.append

    A("# Transport-Aware Credit Assignment Probe — v1")
    A("")
    A("## Purpose")
    A("")
    A("Test M4: whether the current backward pass is misaligned because it treats")
    A("transport operators (ops 3–5: coupled kick, fiber lift, radial transport) and")
    A("non-transport operators (ops 0–2: torus advance, swap, twist) identically.")
    A("")
    A("## Mechanism")
    A("")
    A("A **gradient hook** on the operator-selection logits scales the backward gradient")
    A("for transport channels by a factor γ (gamma).")
    A("")
    A("```")
    A("logits = h @ W2 + b2          # (B, 6) operator selection logits")
    A("")
    A("# During backward only:")
    A("∂L/∂logits[:, 0:3] unchanged  # non-transport (ops 0,1,2)")
    A("∂L/∂logits[:, 3:6] *= γ       # transport (ops 3,4,5)")
    A("```")
    A("")
    A("- **Forward pass is IDENTICAL for all γ values**")
    A("- Only the backward gradient through transport logits is scaled")
    A("- γ=1.0 is the standard backward (baseline)")
    A("- γ>1.0: model receives amplified credit signal for transport ops")
    A("- γ<1.0: model receives dampened credit signal for transport ops")
    A("- γ=0.0: model receives zero gradient for transport op selection")
    A("")
    A("### Rationale")
    A("")
    A("If M4 is binding, transport operators have a geometric adjoint that differs from")
    A("the flat Euclidean adjoint. The optimal γ compensates for this difference:")
    A("- γ_opt > 1.0 → Euclidean adjoint underweights transport (geometric adjoint is larger)")
    A("- γ_opt < 1.0 → Euclidean adjoint overweights transport (geometric adjoint is smaller)")
    A("- γ_opt = 1.0 → Euclidean adjoint is fine at this level (M4 not binding)")
    A("")

    A("## Locked Configuration")
    A("")
    A("| Item | Value |")
    A("|------|-------|")
    A("| device | cpu |")
    A(f"| D_HIDDEN | {D_HIDDEN} |")
    A(f"| batch_size | {BATCH_SIZE} |")
    A(f"| D (delay) | {D_CONTEXT} |")
    A(f"| representation | hybrid angular+radial (D_TAU=12) |")
    A(f"| training budget | {BENCH_BUDGET} steps |")
    A(f"| optimizer | SGD, lr={LR} |")
    A(f"| temperature | {TEMP_START} → {TEMP_END} (exponential) |")
    A(f"| grad clip | 1.0 |")
    A(f"| pos0 bias init | {B0_INIT} |")
    A(f"| seed | {GLOBAL_SEED} |")
    A("")

    A(f"## Gamma Sweep: {GAMMA_VALUES}")
    A("")

    # ── Convergence table ──
    A("## Convergence Comparison")
    A("")
    header = "| Step |"
    sep    = "|------|"
    for g in GAMMA_VALUES:
        header += f" γ={g:.2f} Acc |"
        sep    += "------------|"
    A(header)
    A(sep)

    ck_dicts = {}
    for g, rows in all_variant_rows.items():
        ck_dicts[g] = {r["checkpoint"]: r for r in rows}

    for ck in CHECKPOINTS:
        line = f"| {ck} |"
        for g in GAMMA_VALUES:
            r = ck_dicts.get(g, {}).get(ck, {})
            acc = r.get("accuracy", -1)
            solved = "✓" if acc >= 0.90 else ""
            line += f" {acc:.4f}{solved} |"
        A(line)
    A("")

    # ── First solve step ──
    def _first_solve(rows: List[dict]) -> int:
        for r in rows:
            if r.get("accuracy", 0) >= 0.90 and r.get("checkpoint", 0) > 0:
                return r["checkpoint"]
        return -1

    A("## Solve Step by Gamma")
    A("")
    A("| γ | First Solve Step | Final Accuracy | Final α₀ | Transport Frac | Steps/sec |")
    A("|---|-----------------|----------------|----------|----------------|-----------|")
    for g in GAMMA_VALUES:
        rows = all_variant_rows[g]
        ss = _first_solve(rows)
        ff = ck_dicts[g].get(BENCH_BUDGET, {})
        rt = all_runtimes[g]
        sps = BENCH_BUDGET / rt if rt > 0 else 0
        ss_str = str(ss) if ss > 0 else "never"
        A(f"| {g:.2f} | {ss_str} | {ff.get('accuracy',-1):.4f} | "
          f"{ff.get('alpha0',-1):.4f} | {ff.get('transport_frac',-1):.4f} | {sps:.1f} |")
    A("")

    # ── Gradient norms ──
    A("## Gradient Norm Analysis")
    A("")
    A("Pre-scaling gradient norms (measured by hook before gamma is applied):")
    A("")
    A("| γ | Step | Mean ‖∇_transport‖ | Mean ‖∇_non-transport‖ | Ratio (T/NT) |")
    A("|---|------|---------------------|------------------------|--------------|")
    for g in GAMMA_VALUES:
        for ck in [500, 1500, 3000]:
            r = ck_dicts[g].get(ck, {})
            gt  = r.get("mean_grad_transport", -1)
            gnt = r.get("mean_grad_nontransport", -1)
            gr  = r.get("grad_ratio", -1)
            A(f"| {g:.2f} | {ck} | {gt:.6f} | {gnt:.6f} | {gr:.4f} |")
    A("")

    A("**Interpretation:** If the gradient ratio is consistently ≠ 1.0 across all γ,")
    A("it reveals the natural asymmetry in credit assignment between transport and")
    A("non-transport operators. If the ratio changes with γ, it shows how the scaling")
    A("propagates through the coupled system.")
    A("")

    # ── Transport fraction evolution ──
    A("## Transport Fraction by Gamma")
    A("")
    A("| Step |" + "".join(f" γ={g:.2f} |" for g in GAMMA_VALUES))
    A("|------|" + "--------|" * len(GAMMA_VALUES))
    for ck in CHECKPOINTS:
        line = f"| {ck} |"
        for g in GAMMA_VALUES:
            r = ck_dicts[g].get(ck, {})
            tf = r.get("transport_frac", -1)
            line += f" {tf:.4f} |" if tf >= 0 else " — |"
        A(line)
    A("")

    # ── Route entropy by gamma ──
    A("## Route Entropy by Gamma")
    A("")
    A("| Step |" + "".join(f" γ={g:.2f} |" for g in GAMMA_VALUES))
    A("|------|" + "--------|" * len(GAMMA_VALUES))
    for ck in CHECKPOINTS:
        line = f"| {ck} |"
        for g in GAMMA_VALUES:
            r = ck_dicts[g].get(ck, {})
            re = r.get("route_entropy", -1)
            line += f" {re:.4f} |" if re >= 0 else " — |"
        A(line)
    A("")

    # ── Per-operator usage at final checkpoint ──
    A("## Per-Operator Usage at Final Checkpoint")
    A("")
    A("| γ | Op0 (Tb) | Op1 (Tx) | Op2 (Ty) | Op3 (Tc) | Op4 (Tz') | Op5 (Tr*) |")
    A("|---|----------|----------|----------|----------|-----------|-----------|")
    for g in GAMMA_VALUES:
        ff = ck_dicts[g].get(BENCH_BUDGET, {})
        op = ff.get("op_frac", [-1]*6)
        A(f"| {g:.2f} | {op[0]:.4f} | {op[1]:.4f} | {op[2]:.4f} | "
          f"{op[3]:.4f} | {op[4]:.4f} | {op[5]:.4f} |")
    A("")

    # ═══════════════════════════════════════════════════════════════════
    # Interpretation
    # ═══════════════════════════════════════════════════════════════════
    A("## Interpretation")
    A("")

    # Find optimal gamma
    solve_steps = {}
    final_accs = {}
    for g in GAMMA_VALUES:
        solve_steps[g] = _first_solve(all_variant_rows[g])
        ff = ck_dicts[g].get(BENCH_BUDGET, {})
        final_accs[g] = ff.get("accuracy", 0)

    # Solved gammas
    solved_gammas = [g for g in GAMMA_VALUES if solve_steps[g] > 0]
    unsolved_gammas = [g for g in GAMMA_VALUES if solve_steps[g] <= 0]

    if solved_gammas:
        fastest = min(solved_gammas, key=lambda g: solve_steps[g])
        baseline_ss = solve_steps.get(1.0, -1)

        A(f"**Gamma sweep results:** {len(solved_gammas)}/{len(GAMMA_VALUES)} gamma values solve D=24.")
        A("")

        if fastest != 1.0 and baseline_ss > 0 and solve_steps[fastest] < baseline_ss:
            diff = baseline_ss - solve_steps[fastest]
            A(f"**M4 shows signal:** γ={fastest:.2f} solves {diff} steps earlier than baseline (γ=1.0).")
            A(f"  - Baseline (γ=1.0) solves at step {baseline_ss}")
            A(f"  - Best (γ={fastest:.2f}) solves at step {solve_steps[fastest]}")
            if fastest > 1.0:
                A(f"  - Direction: transport credit should be AMPLIFIED (Euclidean adjoint underweights)")
            else:
                A(f"  - Direction: transport credit should be DAMPENED (Euclidean adjoint overweights)")
            A("")
        elif fastest == 1.0:
            A("**M4 is NOT binding:** γ=1.0 (standard backward) is already optimal or tied for fastest.")
            A("The Euclidean adjoint appears adequate at this regime for operator selection credit.")
            A("")
        else:
            if baseline_ss <= 0:
                A(f"**Baseline (γ=1.0) did not solve** but γ={fastest:.2f} did — strong M4 signal.")
            else:
                A(f"**Tied:** γ={fastest:.2f} and γ=1.0 both solve at the same checkpoint.")
            A("")
    else:
        A("**No gamma value solved D=24.** This is unexpected given the hybrid baseline")
        A("solved in the previous experiment. Possible seed sensitivity or hook overhead issue.")
        A("")

    if unsolved_gammas:
        A(f"**Gammas that did NOT solve:** {', '.join(f'{g:.2f}' for g in unsolved_gammas)}")
        if 0.0 in unsolved_gammas:
            A("  - γ=0.0 (zero transport credit) failing confirms transport operators")
            A("    need gradient signal — they cannot be learned from non-transport gradients alone.")
        A("")

    # ── Gradient analysis interpretation ──
    A("### Gradient Distribution Analysis")
    A("")
    # Use gamma=1.0 data for the natural gradient distribution
    g1_rows = all_variant_rows.get(1.0, [])
    if g1_rows:
        final_g1 = ck_dicts.get(1.0, {}).get(BENCH_BUDGET, {})
        gt  = final_g1.get("mean_grad_transport", 0)
        gnt = final_g1.get("mean_grad_nontransport", 0)
        if gnt > 0:
            ratio = gt / gnt
            if ratio > 1.2:
                A(f"Transport gradient norm is {ratio:.2f}× larger than non-transport.")
                A("This means the current backward pass naturally gives MORE credit to transport")
                A("operators than non-transport — the Euclidean adjoint may be overweighting transport.")
            elif ratio < 0.8:
                A(f"Transport gradient norm is {ratio:.2f}× that of non-transport (smaller).")
                A("This means transport operators receive LESS credit than non-transport —")
                A("the Euclidean adjoint may be underweighting transport contribution.")
            else:
                A(f"Transport and non-transport gradient norms are roughly equal (ratio={ratio:.2f}).")
                A("The Euclidean adjoint distributes credit approximately uniformly, as expected")
                A("for a system where M4 is not strongly binding.")
        A("")

    # ═══════════════════════════════════════════════════════════════════
    # Honesty section
    # ═══════════════════════════════════════════════════════════════════
    A("## Honesty Section")
    A("")

    A("### What Improved")
    A("")
    improvements = []
    if solved_gammas:
        best = min(solved_gammas, key=lambda g: solve_steps[g])
        bl_ss = solve_steps.get(1.0, 99999)
        if best != 1.0 and solve_steps[best] < bl_ss:
            improvements.append(
                f"- γ={best:.2f} converges {bl_ss - solve_steps[best]} steps faster than baseline"
            )
    for g in GAMMA_VALUES:
        if g != 1.0:
            ff = final_accs.get(g, 0)
            bl = final_accs.get(1.0, 0)
            if ff > bl + 0.005:
                improvements.append(f"- γ={g:.2f} has higher final accuracy ({ff:.4f} vs {bl:.4f})")
    if not improvements:
        improvements.append("- No gamma value materially outperformed the γ=1.0 baseline")
    for line in improvements:
        A(line)
    A("")

    A("### What Did Not Improve")
    A("")
    no_improv = []
    if 1.0 in solved_gammas:
        bl_ss = solve_steps[1.0]
        for g in GAMMA_VALUES:
            if g != 1.0 and solve_steps.get(g, -1) >= bl_ss:
                pass  # don't list every single one
        slower = [g for g in GAMMA_VALUES if g != 1.0
                  and 0 < solve_steps.get(g, -1) >= bl_ss]
        if slower:
            no_improv.append(f"- {len(slower)} gamma value(s) converge same or slower than baseline")
    if unsolved_gammas:
        no_improv.append(f"- {len(unsolved_gammas)} gamma value(s) did not solve within budget")
    if not no_improv:
        no_improv.append("- All gamma values performed comparably")
    for line in no_improv:
        A(line)
    A("")

    A("### Whether M4 Looks Binding, or M3 Should Be Tested Next")
    A("")
    if solved_gammas:
        best = min(solved_gammas, key=lambda g: solve_steps[g])
        bl_ss = solve_steps.get(1.0, 99999)
        if best != 1.0 and solve_steps[best] < bl_ss - 250:
            A("**M4 appears binding.** The gradient scaling sweep found a non-trivial optimum.")
            A(f"Recommended next step: develop a principled transport-aware optimizer that")
            A(f"incorporates the γ≈{best:.1f} insight into a proper geometric preconditioner.")
        else:
            A("**M4 does NOT appear binding at this regime.** The simple gradient scaling sweep")
            A("does not find a non-trivial optimum — γ=1.0 is competitive or best.")
            A("")
            A("This does NOT prove M4 is irrelevant in all regimes. It means:")
            A("- At D=24 with D_HIDDEN=32, the Euclidean adjoint is adequate")
            A("- The next binding mismatch is more likely **M3** (flat metric / optimizer geometry)")
            A("- M4 may become binding at larger D or D_HIDDEN where transport structure matters more")
            A("")
            A("**Recommended next step:** Test M3 — a geometry-aware optimizer/preconditioner")
            A("that respects the torus/cone metric rather than treating all parameter directions equally.")
    else:
        A("**Inconclusive.** No variant solved, so M4 binding status cannot be determined.")
    A("")

    A("### What Is Proven")
    A("")
    A("- The gradient hook correctly tracks and scales transport vs non-transport credit")
    A("- The natural gradient distribution between transport and non-transport is measurable")
    if 0.0 in unsolved_gammas:
        A("- Zero transport credit (γ=0) prevents solving → transport selection gradients are necessary")
    A("")

    A("### What Is Still Uncertain")
    A("")
    A("- Whether M4 becomes binding at larger D, D_HIDDEN, or harder tasks")
    A("- Whether a more sophisticated transport-aware modification (beyond scalar scaling)")
    A("  would reveal a stronger signal")
    A("- The interaction between M3 (metric) and M4 (adjoint) — they may be coupled")
    A("")

    with open(MD_OUT, "w") as f:
        f.write("\n".join(L) + "\n")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════
def main() -> None:
    print("=" * 72)
    print("Transport-Aware Credit Assignment Probe v1 (M4)")
    print("=" * 72)
    print(f"  Gamma sweep: {GAMMA_VALUES}")
    print(f"  D={D_CONTEXT}, D_HIDDEN={D_HIDDEN}, B={BATCH_SIZE}, "
          f"budget={BENCH_BUDGET}")
    print(f"  Hybrid angular+radial representation (D_TAU={D_TAU_HYB})")
    print(f"  Checkpoints: {CHECKPOINTS}")
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
    tau0_hyb = torch.cat([tau0_ang, torch.ones(tau0_ang.shape[0], N_PHASE_PAIRS)], dim=1)
    print(f"  TN_ang: {TN_ang.shape}, tau0_hyb: {tau0_hyb.shape}")
    print()

    # ── Run sweep ──
    all_variant_rows: Dict[float, List[dict]] = {}
    all_runtimes: Dict[float, float] = {}

    for gi, gamma in enumerate(GAMMA_VALUES):
        print("=" * 56)
        print(f"  VARIANT {gi+1}/{len(GAMMA_VALUES)}: γ = {gamma:.2f}")
        print("=" * 56)
        rows, runtime, gstats = train_variant(
            gamma, TN_ang, TR, tau0_hyb, pool_ids, device)
        all_variant_rows[gamma] = rows
        all_runtimes[gamma] = runtime
        sps = BENCH_BUDGET / runtime if runtime > 0 else 0
        print(f"  Total: {runtime:.1f}s, {sps:.1f} sps")
        print(f"  Grad stats (final window): "
              f"transport={gstats['mean_grad_transport']:.5f}, "
              f"non-transport={gstats['mean_grad_nontransport']:.5f}, "
              f"ratio={gstats['grad_ratio_t_over_nt']:.4f}")
        print()

    # ── Write CSV ──
    csv_fields = [
        "variant", "gamma", "checkpoint", "accuracy", "alpha0",
        "route_entropy", "transport_fraction",
        "mean_resultant_magnitude",
        "mean_grad_transport", "mean_grad_nontransport", "grad_ratio",
        "op0_frac", "op1_frac", "op2_frac", "op3_frac", "op4_frac", "op5_frac",
        "runtime_seconds", "steps_per_second",
        "loss", "n_params", "note",
    ]
    with open(CSV_OUT, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=csv_fields)
        w.writeheader()
        for gamma in GAMMA_VALUES:
            for r in all_variant_rows[gamma]:
                ckpt = r["checkpoint"]
                rt   = r["runtime_seconds"]
                op   = r.get("op_frac", [-1]*6)
                w.writerow({
                    "variant":                  r["variant"],
                    "gamma":                    r["gamma"],
                    "checkpoint":               ckpt,
                    "accuracy":                 r["accuracy"],
                    "alpha0":                   r["alpha0"],
                    "route_entropy":            r["route_entropy"],
                    "transport_fraction":       r["transport_frac"],
                    "mean_resultant_magnitude": r["mean_mag"],
                    "mean_grad_transport":      r["mean_grad_transport"],
                    "mean_grad_nontransport":   r["mean_grad_nontransport"],
                    "grad_ratio":               r["grad_ratio"],
                    "op0_frac":                 op[0] if len(op) > 0 else -1,
                    "op1_frac":                 op[1] if len(op) > 1 else -1,
                    "op2_frac":                 op[2] if len(op) > 2 else -1,
                    "op3_frac":                 op[3] if len(op) > 3 else -1,
                    "op4_frac":                 op[4] if len(op) > 4 else -1,
                    "op5_frac":                 op[5] if len(op) > 5 else -1,
                    "runtime_seconds":          rt,
                    "steps_per_second":         round(ckpt / rt, 1) if rt > 0 else 0,
                    "loss":                     r["loss"],
                    "n_params":                 r["n_params"],
                    "note":                     r["note"],
                })
    print(f"CSV → {CSV_OUT}")

    # ── Write markdown ──
    write_md(all_variant_rows, all_runtimes)
    print(f"MD  → {MD_OUT}")
    print()

    # ═══════════════════════════════════════════════════════════════════
    # Console summary
    # ═══════════════════════════════════════════════════════════════════
    print("=" * 72)
    print("SUMMARY")
    print("=" * 72)

    def _fs(rows: List[dict]) -> int:
        for r in rows:
            if r.get("accuracy", 0) >= 0.90 and r.get("checkpoint", 0) > 0:
                return r["checkpoint"]
        return -1

    hdr = f"{'gamma':>8s} {'SolveStep':>10s} {'FinalAcc':>9s} {'α₀':>7s} " \
          f"{'TFrac':>7s} {'RouteEnt':>9s} {'Sps':>7s} " \
          f"{'GradT':>8s} {'GradNT':>8s} {'Ratio':>7s}"
    print(hdr)
    print("-" * len(hdr))
    for g in GAMMA_VALUES:
        rows = all_variant_rows[g]
        ff   = {r["checkpoint"]: r for r in rows}.get(BENCH_BUDGET, {})
        ss   = _fs(rows)
        rt   = all_runtimes[g]
        sps  = BENCH_BUDGET / rt if rt > 0 else 0
        print(f"{g:>8.2f} {ss:>10d} {ff.get('accuracy',-1):>9.4f} "
              f"{ff.get('alpha0',-1):>7.4f} "
              f"{ff.get('transport_frac',-1):>7.4f} "
              f"{ff.get('route_entropy',-1):>9.4f} "
              f"{sps:>7.1f} "
              f"{ff.get('mean_grad_transport',0):>8.5f} "
              f"{ff.get('mean_grad_nontransport',0):>8.5f} "
              f"{ff.get('grad_ratio',0):>7.4f}")
    print("=" * 72)


if __name__ == "__main__":
    main()
