"""Learned compound-key attention over four fixed-grammar facts and a null entry.

Role positions are supplied by the frozen grammar, not learned language parsing.
Every match, value representation and vocabulary decision is learned; neither
labels nor an owner/object equality mask enter attention.  Its single attention
tensor is rectangular ``[batch, 1, 1, 5]``.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

import torch
from safetensors.torch import load as load_safetensors
from torch import Tensor, nn
from torch.nn import functional as F

from ..provenance import cid_bytes
from ..zoology_control.model import ZoologyModelOutput
from ..zoology_release.development import _tensor_mapping_cid


@dataclass(frozen=True, slots=True)
class CompoundBindingConfig:
    """The one frozen #1073 model shape; no architecture sweep parameters."""

    vocab_size: int = 4096
    d_model: int = 64
    layer_norm_epsilon: float = 1.0e-5

    def __post_init__(self) -> None:
        if (self.vocab_size, self.d_model, self.layer_norm_epsilon) != (
            4096,
            64,
            1.0e-5,
        ):
            raise ValueError("compound binding requires the frozen 4096/64 config")


MODEL_CONFIG = asdict(CompoundBindingConfig())
MODEL_POLICY = {
    "name": "LearnedFactLevelCompoundBindingV1",
    "issue": 1073,
    "sequence_length": 41,
    "selected_position": 37,
    "fact_owner_positions": [1, 9, 17, 25],
    "fact_object_positions": [4, 12, 20, 28],
    "fact_location_positions": [7, 15, 23, 31],
    "query_owner_position": 35,
    "query_object_position": 37,
    "query": "Wq LN0_128(concat(E(query_owner), E(query_object)))",
    "keys": "shared Wk LN0_128(concat(E(fact_owner), E(fact_object))); learned null key",
    "values": "shared Wv LN0_64(E(fact_location)); learned null value",
    "attention": "softmax(query @ keys.T / sqrt(64)); four facts plus null",
    "output": "affine_LN_64(Wout(attention @ values)) @ E.T; full 4096 vocabulary",
    "projection_bias": False,
    "input_norm_affine": False,
    "output_norm_affine": True,
    "dropout": 0.0,
    "query_residual": False,
    "position_embeddings": False,
    "equality_masks": False,
    "parameter_count": 286976,
    "initialization": {
        "order": ["E", "Wq", "Wk", "Wv", "Wout", "null_key", "null_value", "output_LN"],
        "random_weights": "normal(mean=0, std=0.02), once in listed order",
        "output_LN": "weight=1, bias=0; no RNG draws",
        "implicit_constructor_rng_draws": 0,
    },
    "control": {
        "name": "value_cycle",
        "fact_values": "right cyclic shift by one; destination i gets source (i-1) mod 4",
        "null_value": "unchanged",
        "queries_keys_attention": "unchanged",
        "training": "forbidden",
    },
    "scope": "fixed grammar role extraction with learned binding; not learned English parsing",
}


class _ExplicitLinear(nn.Linear):
    """Allocate parameters without the parent constructor's random reset."""

    def reset_parameters(self) -> None:
        pass


class _ExplicitLayerNorm(nn.LayerNorm):
    """Delay the deterministic affine reset until the declared final step."""

    def reset_parameters(self) -> None:
        pass


class CompoundBindingModel(nn.Module):
    """A single learned ordinary Q/K/V operation with no direct query readout."""

    def __init__(self, config: CompoundBindingConfig | None = None) -> None:
        super().__init__()
        if config is None:
            config = CompoundBindingConfig()
        self.config = config
        width = config.d_model
        # Supplying _weight skips Embedding.reset_parameters and its RNG draw.
        self.embedding = nn.Embedding(
            config.vocab_size,
            width,
            _weight=torch.empty(config.vocab_size, width),
        )
        self.query_projection = _ExplicitLinear(2 * width, width, bias=False)
        self.key_projection = _ExplicitLinear(2 * width, width, bias=False)
        self.value_projection = _ExplicitLinear(width, width, bias=False)
        self.output_projection = _ExplicitLinear(width, width, bias=False)
        self.null_key = nn.Parameter(torch.empty(width))
        self.null_value = nn.Parameter(torch.empty(width))
        self.compound_norm = nn.LayerNorm(
            2 * width, eps=config.layer_norm_epsilon, elementwise_affine=False
        )
        self.location_norm = nn.LayerNorm(
            width, eps=config.layer_norm_epsilon, elementwise_affine=False
        )
        self.output_norm = _ExplicitLayerNorm(width, eps=config.layer_norm_epsilon)
        self.lm_head = _ExplicitLinear(width, config.vocab_size, bias=False)
        self.lm_head.weight = self.embedding.weight
        for parameter in (
            self.embedding.weight,
            self.query_projection.weight,
            self.key_projection.weight,
            self.value_projection.weight,
            self.output_projection.weight,
            self.null_key,
            self.null_value,
        ):
            nn.init.normal_(parameter, mean=0.0, std=0.02)
        nn.init.ones_(self.output_norm.weight)
        nn.init.zeros_(self.output_norm.bias)

    def parameter_count(self) -> int:
        """Count unique scalars, counting the shared embedding/head once."""

        return sum(parameter.numel() for parameter in self.parameters())

    def forward_selected(
        self,
        input_ids: Tensor,
        selected_positions: Tensor,
        targets: Tensor | None = None,
        return_attention: bool = False,
        *,
        control: str = "none",
    ) -> ZoologyModelOutput:
        if (
            input_ids.ndim != 2
            or input_ids.shape[0] < 1
            or input_ids.shape[1] != 41
            or input_ids.dtype != torch.long
        ):
            raise ValueError("compound binding requires int64 inputs [batch,41]")
        if (
            selected_positions.shape != (input_ids.shape[0], 1)
            or selected_positions.dtype != torch.long
            or selected_positions.device != input_ids.device
            or not bool((selected_positions == 37).all())
        ):
            raise ValueError("compound binding requires selected position 37 [batch,1]")
        if control not in ("none", "value_cycle"):
            raise ValueError("unknown compound binding control")
        if self.training and control != "none":
            raise ValueError("compound binding controls are forbidden during training")

        # Only these causal role fields are embedded. Unread grammar/future
        # tokens and target IDs never reach the query, keys, values or logits.
        query_owner = self.embedding(input_ids[:, 35:36])
        query_object = self.embedding(input_ids[:, 37:38])
        query = self.query_projection(
            self.compound_norm(torch.cat((query_owner, query_object), dim=-1))
        )
        fact_owner = self.embedding(input_ids[:, 1:33:8])
        fact_object = self.embedding(input_ids[:, 4:33:8])
        keys = self.key_projection(
            self.compound_norm(torch.cat((fact_owner, fact_object), dim=-1))
        )
        values = self.value_projection(
            self.location_norm(self.embedding(input_ids[:, 7:33:8]))
        )
        if control == "value_cycle":
            values = torch.roll(values, shifts=1, dims=1)
        batch_size = input_ids.shape[0]
        keys = torch.cat((keys, self.null_key.expand(batch_size, 1, -1)), dim=1)
        values = torch.cat((values, self.null_value.expand(batch_size, 1, -1)), dim=1)
        scores = torch.matmul(query, keys.transpose(-2, -1)) / 8.0
        weights = torch.softmax(scores, dim=-1)
        hidden = self.output_norm(self.output_projection(torch.matmul(weights, values)))
        logits = self.lm_head(hidden)
        loss = None
        if targets is not None:
            if (
                targets.shape != selected_positions.shape
                or targets.dtype != torch.long
                or targets.device != input_ids.device
                or bool(((targets < 0) | (targets >= self.config.vocab_size)).any())
            ):
                raise ValueError("selected targets must be valid int64 IDs [batch,1]")
            loss = F.cross_entropy(
                logits.reshape(-1, self.config.vocab_size), targets.reshape(-1)
            )
        return ZoologyModelOutput(
            logits=logits,
            loss=loss,
            hidden_states=hidden,
            selected_positions=selected_positions,
            selected_targets=targets,
            attention_weights=(weights.unsqueeze(1),) if return_attention else None,
        )


def load_model(preparation: Mapping[str, Any]) -> CompoundBindingModel:
    """Load exactly a content-bound compound model, requiring its execution policy."""

    source = preparation["source"]
    record = source["model"]
    if (
        record.get("model_policy") != MODEL_POLICY
        or record.get("config") != MODEL_CONFIG
    ):
        raise ValueError("compound model policy or config differs or is absent")
    payload = (Path(source["root"]) / record["path"]).read_bytes()
    if len(payload) != record["bytes"] or cid_bytes(payload) != record["cid"]:
        raise ValueError("source model file changed")
    state = load_safetensors(payload)
    if _tensor_mapping_cid(state) != record["state_cid"]:
        raise ValueError("source model tensor identity differs")
    model = CompoundBindingModel(CompoundBindingConfig(**record["config"]))
    missing, unexpected = model.load_state_dict(state, strict=False)
    if missing != ["lm_head.weight"] or unexpected:
        raise ValueError("model must omit exactly the tied lm_head.weight")
    model.requires_grad_(False)
    model.eval()
    observed_state = {
        name: value
        for name, value in model.state_dict().items()
        if name != "lm_head.weight"
    }
    if _tensor_mapping_cid(observed_state) != record["state_cid"]:
        raise ValueError("loaded model tensors differ")
    return model
