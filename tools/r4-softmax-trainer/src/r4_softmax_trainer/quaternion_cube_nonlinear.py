"""Finite H4-indexed quaternion-cube nonlinearity for the #973 path.

The accepted sparse recurrent reader and learned artifact are inherited
unchanged.  This candidate replaces only the dense SwiGLU execution with a
parameter-free residual in the current token's H4 frame.  It is an unfitted
f32 mechanism, not a language-quality or table-native execution claim.
"""

from __future__ import annotations

import torch
from torch import Tensor

from .fixed_recurrent_kv_binding import RecurrentNonlinearAudit
from .language_path_generalization import HIDDEN_SIZE
from .position_kv_binding import R4_WIDTH
from .sparse_geometric_kv_binding import (
    R4SparseGeometricCandidateSoftmaxKVBindingV1,
)


POLICY = "R4H4FrameQuaternionCubeResidualV1"
R4_BLOCKS_PER_HIDDEN = HIDDEN_SIZE // R4_WIDTH
H4_FRAME_MAPS_PER_BLOCK = 2
H4_FRAME_COEFFICIENT_PRODUCTS_PER_BLOCK = 2 * R4_WIDTH * R4_WIDTH
QUATERNION_CUBE_SCALAR_PRODUCTS_PER_BLOCK = 14
QUATERNION_CUBE_RECIPROCALS_PER_BLOCK = 1
RESIDUAL_SUBTRACTIONS_PER_BLOCK = R4_WIDTH

if R4_WIDTH != 4 or HIDDEN_SIZE % R4_WIDTH != 0:
    raise RuntimeError("quaternion-cube residual requires ordered R4 blocks")


class R4H4FrameQuaternionCubeResidualV1(
    R4SparseGeometricCandidateSoftmaxKVBindingV1
):
    """Sparse recurrent model with a finite H4-indexed R4 nonlinearity."""

    POLICY_NAME = POLICY
    NONLINEAR_POLICY = POLICY

    def _post_attention_nonlinear(
        self,
        values: Tensor,
        *,
        layer: object,
        current_frames: Tensor,
    ) -> tuple[Tensor, RecurrentNonlinearAudit]:
        """Apply ``F C(F^T n) - n`` after the existing RMS normalization."""

        batch_size = int(values.shape[0])
        if values.ndim != 2 or int(values.shape[1]) != HIDDEN_SIZE:
            raise ValueError("quaternion-cube input must be [batch,hidden]")
        if tuple(current_frames.shape) != (batch_size,):
            raise ValueError("current H4 frames must be [batch]")

        post_attention_layernorm = getattr(layer, "post_attention_layernorm")
        normalized = post_attention_layernorm(values).float()
        normalized_blocks = normalized.view(
            batch_size, R4_BLOCKS_PER_HIDDEN, R4_WIDTH
        )
        frames = self._frames(current_frames, dtype=torch.float32)
        local = torch.einsum("bji,bdj->bdi", frames, normalized_blocks)

        scalar = local[..., :1]
        vector = local[..., 1:]
        scalar_squared = scalar * scalar
        vector_squared_norm = (vector * vector).sum(dim=-1, keepdim=True)
        norm_squared = scalar_squared + vector_squared_norm
        cubed = torch.cat(
            (
                scalar * (scalar_squared - 3.0 * vector_squared_norm),
                (3.0 * scalar_squared - vector_squared_norm) * vector,
            ),
            dim=-1,
        )

        nonzero = (local != 0.0).any(dim=-1, keepdim=True)
        safe_norm_squared = torch.where(
            nonzero,
            norm_squared,
            torch.ones_like(norm_squared),
        )
        inverse_norm_squared = torch.reciprocal(safe_norm_squared)
        mapped_local = cubed * inverse_norm_squared
        mapped_local = torch.where(
            nonzero.expand_as(mapped_local),
            mapped_local,
            torch.zeros_like(mapped_local),
        )
        decoded = torch.einsum("bij,bdj->bdi", frames, mapped_local)
        delta_blocks = decoded - normalized_blocks
        if not bool(torch.isfinite(delta_blocks).all()):
            raise ValueError("quaternion-cube residual produced a non-finite value")

        input_norms = torch.linalg.vector_norm(normalized_blocks, dim=-1)
        output_norms = torch.linalg.vector_norm(decoded, dim=-1)
        norm_errors = (output_norms - input_norms).abs()
        residual_norms = torch.linalg.vector_norm(delta_blocks, dim=-1)
        nonzero_norms = input_norms != 0.0
        safe_residual_bounds = torch.where(
            nonzero_norms,
            2.0 * input_norms,
            torch.ones_like(input_norms),
        )
        residual_bound_ratios = torch.where(
            nonzero_norms,
            residual_norms / safe_residual_bounds,
            torch.zeros_like(residual_norms),
        )

        block_evaluations = batch_size * R4_BLOCKS_PER_HIDDEN
        audit = RecurrentNonlinearAudit(
            policy=self.NONLINEAR_POLICY,
            batch_size=batch_size,
            layer_calls=1,
            dense_mlp_calls=0,
            dense_mlp_weight_products=0,
            r4_block_evaluations=block_evaluations,
            h4_frame_maps=block_evaluations * H4_FRAME_MAPS_PER_BLOCK,
            h4_frame_coefficient_products=(
                block_evaluations
                * H4_FRAME_COEFFICIENT_PRODUCTS_PER_BLOCK
            ),
            quaternion_cube_scalar_products=(
                block_evaluations
                * QUATERNION_CUBE_SCALAR_PRODUCTS_PER_BLOCK
            ),
            quaternion_cube_reciprocals=(
                block_evaluations * QUATERNION_CUBE_RECIPROCALS_PER_BLOCK
            ),
            residual_subtractions=(
                block_evaluations * RESIDUAL_SUBTRACTIONS_PER_BLOCK
            ),
            maximum_block_norm_error=float(norm_errors.amax().detach()),
            maximum_residual_bound_ratio=float(
                residual_bound_ratios.amax().detach()
            ),
        )
        return (
            delta_blocks.reshape(batch_size, HIDDEN_SIZE).to(values.dtype),
            audit,
        )


__all__ = [
    "H4_FRAME_COEFFICIENT_PRODUCTS_PER_BLOCK",
    "H4_FRAME_MAPS_PER_BLOCK",
    "POLICY",
    "QUATERNION_CUBE_RECIPROCALS_PER_BLOCK",
    "QUATERNION_CUBE_SCALAR_PRODUCTS_PER_BLOCK",
    "R4_BLOCKS_PER_HIDDEN",
    "RESIDUAL_SUBTRACTIONS_PER_BLOCK",
    "R4H4FrameQuaternionCubeResidualV1",
]
