"""A fixed causal owner residual at the existing query-object embedding."""

from __future__ import annotations

from collections.abc import Mapping
from typing import Any

from torch import Tensor, nn

from ..zoology_control.model import ZoologyFigure2Config, ZoologyFigure2Model
from ..zoology_r4_inference.campaign import _load_model as _load_source_model

QUERY_ENCODING = {
    "policy": "JointQueryOwnerResidualV1",
    "sequence_length": 41,
    "owner_position": 35,
    "query_position": 37,
    "owner_coefficient": 1.0,
    "formula": "x[37] = E(token[37]) + P[37] + E(token[35]); all other positions unchanged",
    "placement": "before source embedding dropout and attention trunk",
    "additional_parameters": 0,
}


def _owner_residual(
    module: nn.Module, arguments: tuple[Tensor, ...], output: Tensor
) -> Tensor:
    input_ids = arguments[0]
    if input_ids.ndim != 2 or input_ids.shape[1] != QUERY_ENCODING["sequence_length"]:
        raise ValueError("joint query encoding requires exactly 41 input tokens")
    if module.project_in is not None:
        raise ValueError("joint query encoding requires the source embedding width")
    joint = output.clone()
    query, owner = QUERY_ENCODING["query_position"], QUERY_ENCODING["owner_position"]
    joint[:, query, :] = output[:, query, :] + module.word_embeddings(
        input_ids[:, owner]
    )
    return joint


def install_joint_query_embedding(model: ZoologyFigure2Model) -> ZoologyFigure2Model:
    """Install once without replacing parameters, initializing tensors or drawing RNG."""
    embeddings = model.backbone.embeddings
    if hasattr(embeddings, "_joint_query_owner_residual_handle"):
        raise ValueError("joint query encoding is already installed")
    if embeddings.project_in is not None:
        raise ValueError("joint query encoding requires the source embedding width")
    embeddings._joint_query_owner_residual_handle = embeddings.register_forward_hook(
        _owner_residual
    )
    return model


class ZoologyJointQueryModel(ZoologyFigure2Model):
    """The source initialization and state layout with one parameter-free hook."""

    def __init__(self, config: ZoologyFigure2Config | None = None) -> None:
        super().__init__(config)
        install_joint_query_embedding(self)


def load_model(preparation: Mapping[str, Any]) -> ZoologyFigure2Model:
    """Require adapted artifact metadata, verify source weights, then install once.

    The safetensors payload intentionally retains the source state layout. Its
    content-bound JSON envelope supplies the parameter-free execution policy.
    No unadapted model escapes this loader.
    """
    if preparation["source"]["model"].get("query_encoding") != QUERY_ENCODING:
        raise ValueError("model artifact joint query encoding differs or is absent")
    return install_joint_query_embedding(_load_source_model(preparation))
