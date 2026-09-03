"""Measure the fixed token intervention without invoking the binding/head path."""

from __future__ import annotations

import torch

from ..zoology_language_r4.attention import _pool_roles, frame_assignment

ROLE_NAMES = [
    f"fact_{clause}.{role}"
    for clause in range(4)
    for role in ("owner", "object", "location")
] + ["query.owner", "query.object"]
METRICS = (
    "changed_attention_mass",
    "changed_attention_fraction",
    "weighted_individual_displacement",
    "net_displacement_f64",
    "used_pool_displacement_f32",
    "coherent_used_role_norm",
    "cancellation_retained_fraction",
)


def displacement_metrics(weights, changed, delta, coherent, controlled):
    """Inputs: a[B,R,T], changed[B,T], delta[B,T,64], used pools[B,R,64].

    A=sum a_i ||delta_i||; D=||sum a_i delta_i||. Both use the original
    nonnegative f32 reader weights promoted to f64, without renormalization.
    The fraction M/sum(a) is reported separately from raw attention mass M.
    D/A is undefined at A=0: stored zero is a placeholder, excluded in summaries.
    """
    weights = weights.double()
    if not bool(torch.isfinite(weights).all()) or bool((weights < 0).any()):
        raise ValueError("attention weights must be finite and nonnegative")
    total = weights.sum(-1)
    if not bool(((total - 1).abs() <= 1e-5).all()):
        raise ValueError("frozen attention mass does not sum to one within 1e-5")
    if not bool(torch.isfinite(delta).all()):
        raise ValueError("individual displacement is nonfinite")
    mass = torch.einsum("brt,bt->br", weights, changed.double())
    individual = torch.einsum("brt,bt->br", weights, delta.norm(dim=-1))
    net_vector = torch.einsum("brt,btd->brd", weights, delta)
    net = net_vector.norm(dim=-1)
    used = (controlled.double() - coherent.double()).norm(dim=-1)
    baseline = coherent.double().norm(dim=-1)
    denominator = torch.where(individual > 0, individual, torch.ones_like(individual))
    ratio = torch.where(individual > 0, net / denominator, 0)
    values = torch.stack(
        (mass, mass / total, individual, net, used, baseline, ratio), -1
    )
    if not bool(torch.isfinite(values).all()):
        raise ValueError("diagnostic metrics are nonfinite")
    if bool((net > individual + 1e-12 * (1 + individual)).any()):
        raise ValueError("weighted triangle bound exceeded numerical tolerance")
    if bool(((mass == 0) & (individual != 0)).any()):
        raise ValueError("zero changed-frame attention has nonzero displacement")
    return values, net_vector


@torch.inference_mode()
def measure_batch(model, inputs, lengths, frames):
    """Reconstruct #1079 pools exactly and measure their per-token displacements."""
    if any(module.training for module in model.modules()) or any(
        parameter.requires_grad for parameter in model.parameters()
    ):
        raise ValueError("diagnostic requires the frozen model in eval mode")
    weights = model.reader(inputs, lengths)
    tokens, ends = frame_assignment(inputs, lengths, frames)
    coherent = torch.empty((len(inputs), 5, 3, 64), dtype=torch.float32)
    controlled = torch.empty_like(coherent)
    metrics = torch.empty((len(inputs), 5, 3, len(METRICS)), dtype=torch.float64)
    changed_count = 0
    closure_error = 0.0
    for clause in range(5):
        for size in torch.unique(lengths[:, clause], sorted=True).tolist():
            rows = lengths[:, clause] == size
            true = frames.frame_matrices[tokens[rows, clause, :size]]
            following = true.roll(-1, 1)
            destination = frames.frame_matrices[ends[rows, clause]]
            changed = (true != following).any(dim=(-2, -1))
            changed_count += int(changed.sum())
            embedded = model.core.embedding(inputs[rows, clause, :size]).double()
            blocks = embedded.reshape(-1, size, 16, 4)
            encoded = torch.einsum("btji,btdj->btdi", true, blocks)
            pooled, decoded_values = [], []
            role_weights = weights[rows, clause, :, :size]
            for source in (true, following):
                connection = torch.einsum("bji,btjk->btik", destination, source)
                moved = torch.einsum("btij,btdj->btdi", connection, encoded)
                mixture = torch.einsum("brt,btdi->brdi", role_weights.double(), moved)
                pooled.append(
                    torch.einsum("bij,brdj->brdi", destination, mixture).reshape(
                        -1, 3, 64
                    )
                )
                decoded_values.append(
                    torch.einsum("bij,btdj->btdi", destination, moved).reshape(
                        -1, size, 64
                    )
                )
            coherent[rows, clause], controlled[rows, clause] = (
                pooled[0].float(),
                pooled[1].float(),
            )
            delta = decoded_values[1] - decoded_values[0]
            if bool((delta[~changed] != 0).any()):
                raise ValueError("unchanged source matrix displaced a token value")
            values, net_vector = displacement_metrics(
                role_weights, changed, delta, pooled[0].float(), pooled[1].float()
            )
            closure = (net_vector - (pooled[1] - pooled[0])).abs().max().item()
            closure_error = max(closure_error, closure)
            if closure > 1e-12 * (
                1 + max(pooled[0].abs().max().item(), pooled[1].abs().max().item())
            ):
                raise ValueError(
                    "individual displacements do not reconstruct the f64 pool difference"
                )
            metrics[rows, clause] = values
    # Reuse the frozen helper as the operational reference; do not modify it.
    for permuted, reconstructed in ((False, coherent), (True, controlled)):
        reference = _pool_roles(
            model,
            inputs,
            lengths,
            weights,
            tokens,
            ends,
            frames,
            permute_token_frames=permuted,
        )
        if not torch.equal(reference, reconstructed):
            raise ValueError("diagnostic reconstructed pool differs from #1079 helper")
    return {
        "role_attention": weights,
        "coherent": coherent,
        "controlled": controlled,
        # Fixed model-use mask: exclude only the unused query-location role.
        "metrics": metrics.reshape(len(inputs), 15, len(METRICS))[:, :14],
        "changed_source_matrices": changed_count,
        "max_f64_pool_closure_error": closure_error,
    }
