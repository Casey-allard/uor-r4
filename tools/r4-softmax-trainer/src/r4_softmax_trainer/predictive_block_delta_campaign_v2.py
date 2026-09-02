"""Matched-control correction for the #973 predictive block-delta cheap gate.

V2 leaves ``R4PredictiveBlockDeltaBindingV1`` unchanged.  It independently
fits native delta and additive/Hebbian arms on the pre-bound, non-overlapping
revealed-V4 pair slice 32 through 63.  Native capacity alone controls V5
authorization; additive language validity and delta attribution are reported
separately.  No code in this module selects, creates, or opens V5.
"""

from __future__ import annotations

import json
import math
import os
import time
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

import torch
from torch import Tensor
from torch.nn import functional as F

from .h4_spin_frame_sidecar import H4SpinFrameArtifactV1
from .language_path_generalization_campaign import _exact_geometry
from .layerwise_normalized_retained_readout_campaign import (
    PREDECESSOR_ARTIFACT_BYTES,
    PREDECESSOR_ARTIFACT_CID,
    PREDECESSOR_POLICY,
    PREDECESSOR_RESULT_CID,
    _verify_predecessor,
)
from .predictive_block_delta_campaign import (
    ABSOLUTE_GAIN_THRESHOLD,
    ADAM_BETAS,
    ADAM_EPSILON,
    BATCH_DIRECTIONS,
    GRADIENT_CLIP,
    H4_FRAME_ARTIFACT_CID,
    H4_FRAME_FILE_CID,
    HARD_WALL_SECONDS,
    INTERVENTION_LOSS_THRESHOLD,
    LEARNING_RATE,
    MAXIMUM_UPDATES,
    MODEL_POLICY,
    PROBE_DIRECTIONS,
    PROBE_PAIRS,
    PROBE_TARGETS,
    ROOT_TABLE_KAPPA,
    PRODUCT_TABLE_KAPPA,
    TRAINABLE_PARAMETERS,
    V1_IMPLEMENTATION_TREE_CID,
    V4_COMMITMENT_CID,
    V4_GEOMETRIC_ARTIFACT_CID,
    V4_POOLED_ARTIFACT_CID,
    V4_POPULATION_CID,
    V4_POPULATION_RELATIVE_PATH,
    V4_REVEAL_CID,
    V4_REVEAL_RELATIVE_PATH,
    ProbeScore,
    _batch,
    _boolean_field,
    _cid_field,
    _directions,
    _exact_mapping,
    _float_field,
    _integer_field,
    _output,
    _probe_score_from_record,
    _read_canonical_json,
    _suffix_log_probabilities,
    _validate_cached_result as _validate_v1_result,
    _verify_self_cid,
    _with_self_cid,
    destroy_disposable_weights,
    score_probe,
    transport_mechanics,
)
from .prompt_conditioning_v4 import (
    PROMPT_TOKENS,
    PromptConditioningPair,
    load_revealed_prompt_conditioning_population,
)
from .provenance import (
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
    tree_cid,
)


ISSUE = 973
POLICY = "R4PredictiveBlockDeltaPromptCapacityV2"
RESULT_SCHEMA = "uor-r4.predictive-block-delta-admission/2"
RESULT_RELATIVE_PATH = "preflight/predictive-block-delta-admission-v2.json"
SELECTOR_SCHEMA = "uor-r4.predictive-block-delta-v2-selector/1"
SELECTOR_CID = (
    "blake3:285be20c9c41267dbf925ea7d24d198b41a9014653ff62b1bdb64c8e2ee4fd5a"
)

PAIR_START = 32
PAIR_STOP = 64
SLICE_RECORDS_CID = (
    "blake3:1971bd80a9617a762c909ebc5585d2e594b4dbb3f951447aa50cf015796e1e2b"
)
SLICE_IDENTITIES_CID = (
    "blake3:22a939cd8c1549933d14ca295e2a85d96bdec82dbc8661719a09bcb420157687"
)
V1_RESULT_CID = (
    "blake3:004abd0ab27e63065c4961863123c8e086ff1b88ea12162de558a0bdaac8dac8"
)
V1_VERDICT = "PREDICTIVE_BINDING_NOT_OBSERVABLE"
V2_IMPLEMENTATION_TREE_CID = (
    "blake3:603791e7b74a682855507797a6ab3533e36f963914fc2d307549e97e87bde366"
)

NATIVE_ADMIT = "PREDICTIVE_BINDING_NATIVE_CAPACITY_ADMIT"
NATIVE_MISS = "PREDICTIVE_BINDING_NATIVE_CAPACITY_MISS"
INVALID = "INVALID_PREDICTIVE_BINDING_V2_PREFLIGHT"
ADDITIVE_NO_STABLE_CAPACITY = "ADDITIVE_CONTROL_NO_STABLE_CAPACITY"
DELTA_SUPERIORITY = "DELTA_PROMPT_SPECIFIC_SUPERIORITY"
DELTA_SUPERIORITY_NOT_ESTABLISHED = (
    "DELTA_PROMPT_SPECIFIC_SUPERIORITY_NOT_ESTABLISHED"
)

LANGUAGE_NLL_TOLERANCE = 0.05
INITIALIZATION_SEED = 9_739
CPU_THREADS = 8

FitIntervention = Literal["native", "no_delta"]


@dataclass(frozen=True, slots=True)
class FrozenV2Inputs:
    predecessor: Any
    predecessor_artifact_path: Path
    frames: H4SpinFrameArtifactV1
    pairs: tuple[PromptConditioningPair, ...]
    records: Mapping[str, Any]


def _pair_identities(pairs: Sequence[PromptConditioningPair]) -> list[dict[str, Any]]:
    return [
        {
            "pair_index": pair.pair_index,
            "left": {
                "source_story_ordinal": pair.left.source_story_ordinal,
                "story_cid": pair.left.story_cid,
            },
            "right": {
                "source_story_ordinal": pair.right.source_story_ordinal,
                "story_cid": pair.right.story_cid,
            },
        }
        for pair in pairs
    ]


def _selector(pairs: Sequence[PromptConditioningPair]) -> dict[str, Any]:
    """Return the exact public, predeclared V4 pair selector."""

    unsigned = {
        "schema": SELECTOR_SCHEMA,
        "v4_population_cid": V4_POPULATION_CID,
        "pair_start": PAIR_START,
        "pair_end_exclusive": PAIR_STOP,
        "pairs": [
            {
                "pair_index": pair.pair_index,
                "left_source_story_ordinal": pair.left.source_story_ordinal,
                "left_story_cid": pair.left.story_cid,
                "right_source_story_ordinal": pair.right.source_story_ordinal,
                "right_story_cid": pair.right.story_cid,
            }
            for pair in pairs
        ],
    }
    selector = _with_self_cid(unsigned, "selector_cid")
    if selector["selector_cid"] != SELECTOR_CID:
        raise ValueError("revealed V4 V2 selector differs from the public freeze")
    return selector


def load_frozen_v2_inputs(
    *,
    predecessor_root: Path,
    revealed_v4_root: Path,
    frame_sidecar_path: Path,
    v1_result_path: Path,
) -> FrozenV2Inputs:
    """Verify V1 and bind exact revealed-V4 pairs 32 through 63 before fitting."""

    v1_result = _read_canonical_json(v1_result_path.resolve())
    _validate_v1_result(v1_result)
    if (
        v1_result.get("result_cid") != V1_RESULT_CID
        or v1_result.get("verdict") != V1_VERDICT
        or v1_result.get("admitted") is not False
    ):
        raise ValueError("predictive V1 historical result differs from the V2 freeze")

    predecessor, artifact_path = _verify_predecessor(predecessor_root.resolve())
    frames = H4SpinFrameArtifactV1.load(frame_sidecar_path)
    if (
        frames.artifact_cid != H4_FRAME_ARTIFACT_CID
        or frames.file_cid != H4_FRAME_FILE_CID
    ):
        raise ValueError("H4 spin-frame sidecar differs from the frozen V2 input")

    revealed_v4_root = revealed_v4_root.resolve()
    reveal = _read_canonical_json(revealed_v4_root / V4_REVEAL_RELATIVE_PATH)
    _verify_self_cid(reveal, "reveal_cid")
    if (
        reveal.get("reveal_cid") != V4_REVEAL_CID
        or reveal.get("commitment_cid") != V4_COMMITMENT_CID
        or reveal.get("population_cid") != V4_POPULATION_CID
        or reveal.get("baseline_artifact_cid") != PREDECESSOR_ARTIFACT_CID
        or reveal.get("geometric_artifact_cid") != V4_GEOMETRIC_ARTIFACT_CID
        or reveal.get("pooled_artifact_cid") != V4_POOLED_ARTIFACT_CID
        or reveal.get("reveal_count") != 1
    ):
        raise ValueError("revealed V4 binding differs from the V2 freeze")
    population_path = revealed_v4_root / V4_POPULATION_RELATIVE_PATH
    if cid_file(population_path) != V4_POPULATION_CID:
        raise ValueError("revealed V4 population differs from its exact CID")
    population = load_revealed_prompt_conditioning_population(revealed_v4_root)
    pairs = tuple(population.pairs[PAIR_START:PAIR_STOP])
    if (
        len(pairs) != PROBE_PAIRS
        or tuple(pair.pair_index for pair in pairs) != tuple(range(PAIR_START, PAIR_STOP))
        or cid_bytes(canonical_json_bytes([pair.record() for pair in pairs]))
        != SLICE_RECORDS_CID
    ):
        raise ValueError("revealed V4 V2 slice records differ")
    identities = _pair_identities(pairs)
    if cid_bytes(canonical_json_bytes(identities)) != SLICE_IDENTITIES_CID:
        raise ValueError("revealed V4 V2 ordered identities differ")
    selector = _selector(pairs)

    return FrozenV2Inputs(
        predecessor=predecessor,
        predecessor_artifact_path=artifact_path,
        frames=frames,
        pairs=pairs,
        records={
            "predictive_v1": {
                "result_cid": V1_RESULT_CID,
                "verdict": V1_VERDICT,
                "admitted": False,
                "implementation_tree_cid": V1_IMPLEMENTATION_TREE_CID,
            },
            "predecessor": {
                "policy": PREDECESSOR_POLICY,
                "result_cid": PREDECESSOR_RESULT_CID,
                "artifact_cid": PREDECESSOR_ARTIFACT_CID,
                "artifact_bytes": PREDECESSOR_ARTIFACT_BYTES,
            },
            "revealed_v4": {
                "population_cid": V4_POPULATION_CID,
                "commitment_cid": V4_COMMITMENT_CID,
                "reveal_cid": V4_REVEAL_CID,
                "pair_start": PAIR_START,
                "pair_stop_exclusive": PAIR_STOP,
                "pairs": PROBE_PAIRS,
                "directions": PROBE_DIRECTIONS,
                "targets": PROBE_TARGETS,
                "slice_records_cid": SLICE_RECORDS_CID,
                "ordered_identities_cid": SLICE_IDENTITIES_CID,
            },
            "selector": selector,
            "h4_spin_frames": {
                "artifact_cid": frames.artifact_cid,
                "file_cid": frames.file_cid,
                "root_table_kappa": frames.h4_root_table_kappa,
                "multiplication_table_kappa": frames.h4_multiplication_table_kappa,
            },
        },
    )


def _paired_loss(
    model: Any,
    directions: Sequence[Any],
    device: torch.device,
    *,
    intervention: FitIntervention,
) -> Tensor:
    inputs, targets = _batch(directions, device=device)
    output = _output(
        model(inputs, intervention=intervention), rows=len(inputs)
    )
    own, foreign = _suffix_log_probabilities(output, targets)
    return -own.mean() + F.softplus(-(own - foreign)).mean()


def _batch_schedule_cid(direction_count: int, updates: int) -> str:
    schedule = [
        [
            (update * BATCH_DIRECTIONS + offset) % direction_count
            for offset in range(BATCH_DIRECTIONS)
        ]
        for update in range(updates)
    ]
    return cid_bytes(canonical_json_bytes(schedule))


def fit_independent_arm(
    model: Any,
    pairs: Sequence[PromptConditioningPair],
    *,
    intervention: FitIntervention,
    device: torch.device,
    maximum_updates: int,
    hard_wall_seconds: float,
) -> dict[str, Any]:
    """Fit one independent arm with the frozen optimizer and batch schedule."""

    if not 1 <= maximum_updates <= MAXIMUM_UPDATES or hard_wall_seconds <= 0.0:
        raise ValueError("V2 fit dose or remaining wall is invalid")
    parameters = tuple(model.trainable_parameters())
    frozen = tuple(model.frozen_base_parameters())
    if (
        sum(parameter.numel() for parameter in parameters) != TRAINABLE_PARAMETERS
        or any(parameter.requires_grad for parameter in frozen)
        or not all(parameter.requires_grad for parameter in parameters)
    ):
        raise ValueError("V2 trainable/frozen boundary differs")
    base_before = model.export_qualified_base_artifact()
    binding_before = model.export_binding_artifact()
    directions = _directions(pairs)
    optimizer = torch.optim.AdamW(
        parameters,
        lr=LEARNING_RATE,
        betas=ADAM_BETAS,
        eps=ADAM_EPSILON,
        weight_decay=0.0,
    )
    coverage = [torch.zeros_like(parameter, dtype=torch.bool) for parameter in parameters]
    started = time.monotonic()
    final_loss = math.nan
    final_gradient_norm = math.nan
    model.train()
    for update in range(maximum_updates):
        if time.monotonic() - started > hard_wall_seconds:
            raise TimeoutError("independent V2 fit exceeded the remaining whole-gate wall")
        offset = (update * BATCH_DIRECTIONS) % len(directions)
        batch = tuple(
            directions[(offset + index) % len(directions)]
            for index in range(BATCH_DIRECTIONS)
        )
        optimizer.zero_grad(set_to_none=True)
        loss = _paired_loss(model, batch, device, intervention=intervention)
        if not torch.isfinite(loss).item():
            raise RuntimeError("independent V2 paired loss is nonfinite")
        loss.backward()
        for observed, parameter in zip(coverage, parameters, strict=True):
            if parameter.grad is not None:
                observed.logical_or_(
                    parameter.grad.detach().isfinite() & parameter.grad.detach().ne(0)
                )
        norm = torch.nn.utils.clip_grad_norm_(parameters, GRADIENT_CLIP)
        if not torch.isfinite(norm).item():
            raise RuntimeError("independent V2 gradient norm is nonfinite")
        optimizer.step()
        final_loss = float(loss.detach().cpu())
        final_gradient_norm = float(norm.detach().cpu())
    fitted = model.export_binding_artifact()
    gradient_seen = sum(int(values.sum().item()) for values in coverage)
    return {
        "intervention": intervention,
        "updates": maximum_updates,
        "elapsed_seconds": time.monotonic() - started,
        "final_loss": final_loss,
        "final_gradient_norm": final_gradient_norm,
        "gradient_values_seen": gradient_seen,
        "gradient_values_required": TRAINABLE_PARAMETERS,
        "all_trainable_values_received_finite_nonzero_gradient": (
            gradient_seen == TRAINABLE_PARAMETERS
        ),
        "qualified_base_unchanged": model.export_qualified_base_artifact() == base_before,
        "initial_binding_cid": cid_bytes(binding_before),
        "fitted_binding_cid": cid_bytes(fitted),
        "batch_schedule_cid": _batch_schedule_cid(len(directions), maximum_updates),
    }


def native_capacity_decision(
    *, full: ProbeScore, state_off: ProbeScore, mechanics: Mapping[str, Any]
) -> dict[str, Any]:
    gates = {
        "mechanics_passed": mechanics.get("passed") is True,
        "forbidden_reads_zero": full.forbidden_reads == state_off.forbidden_reads == 0,
        "absolute_gain": full.mean_gain_nats_per_token >= ABSOLUTE_GAIN_THRESHOLD,
        "directional_wins": full.wins >= 52,
        "state_load_bearing": (
            full.mean_gain_nats_per_token - state_off.mean_gain_nats_per_token
            >= INTERVENTION_LOSS_THRESHOLD
        ),
        "language_valid": (
            full.own_nll_nats_per_token
            <= state_off.own_nll_nats_per_token + LANGUAGE_NLL_TOLERANCE
        ),
    }
    integrity = gates["mechanics_passed"] and gates["forbidden_reads_zero"]
    admitted = integrity and all(
        gates[name]
        for name in (
            "absolute_gain",
            "directional_wins",
            "state_load_bearing",
            "language_valid",
        )
    )
    return {
        "verdict": NATIVE_ADMIT if admitted else (NATIVE_MISS if integrity else INVALID),
        "admitted": admitted,
        "gates": gates,
        "thresholds": {
            "absolute_gain_nats_per_token": ABSOLUTE_GAIN_THRESHOLD,
            "wins": 52,
            "state_load_bearing_nats_per_token": INTERVENTION_LOSS_THRESHOLD,
            "own_nll_above_state_off_maximum": LANGUAGE_NLL_TOLERANCE,
        },
    }


def additive_attribution_decision(
    *, full: ProbeScore, additive: ProbeScore, state_off: ProbeScore
) -> dict[str, Any]:
    language_valid = (
        additive.own_nll_nats_per_token
        <= state_off.own_nll_nats_per_token + LANGUAGE_NLL_TOLERANCE
    )
    nll_no_worse = full.own_nll_nats_per_token <= additive.own_nll_nats_per_token
    gain_superior = (
        full.mean_gain_nats_per_token - additive.mean_gain_nats_per_token
        >= INTERVENTION_LOSS_THRESHOLD
    )
    attributed = language_valid and nll_no_worse and gain_superior
    if not language_valid:
        verdict = ADDITIVE_NO_STABLE_CAPACITY
    elif attributed:
        verdict = DELTA_SUPERIORITY
    else:
        verdict = DELTA_SUPERIORITY_NOT_ESTABLISHED
    return {
        "verdict": verdict,
        "additive_language_valid": language_valid,
        "delta_prompt_specific_superiority": attributed,
        "gates": {
            "additive_language_valid": language_valid,
            "native_own_nll_no_worse": nll_no_worse,
            "native_gain_superior": gain_superior,
        },
        "thresholds": {
            "additive_own_nll_above_state_off_maximum": LANGUAGE_NLL_TOLERANCE,
            "native_minus_additive_gain_nats_per_token": INTERVENTION_LOSS_THRESHOLD,
        },
        "claim": (
            "STABILITY_LOAD_BEARING_PROMPT_SUPERIORITY_UNCLAIMED"
            if not language_valid
            else (
                "PROMPT_SPECIFIC_DELTA_SUPERIORITY"
                if attributed
                else "PROMPT_SPECIFIC_DELTA_SUPERIORITY_UNCLAIMED"
            )
        ),
    }


def _mechanics_verdict(mechanics: Mapping[str, Any]) -> bool:
    return bool(
        mechanics["all_frame_identity_maximum_delta"] <= 2e-5
        and mechanics["all_frame_step_connection_maximum_delta"] <= 2e-5
        and mechanics["transported_matrix_read_covariance_maximum_delta"] <= 2e-5
        and mechanics["full_delta_strict_causal_prefix_maximum_logits_delta"]
        <= 2e-5
        and mechanics[
            "full_delta_unobserved_target_mutation_maximum_prefix_delta"
        ]
        <= 2e-5
        and mechanics[
            "additive_strict_causal_prefix_maximum_logits_delta"
        ]
        <= 2e-5
        and mechanics[
            "additive_unobserved_target_mutation_maximum_prefix_delta"
        ]
        <= 2e-5
        and mechanics["state_off_v1_maximum_logits_delta"] == 0.0
        and mechanics["full_delta_artifact_replay_maximum_logits_delta"] == 0.0
        and mechanics["additive_artifact_replay_maximum_logits_delta"] == 0.0
        and mechanics["transport_permutation_head_effect"] > 0.0
        and mechanics["full_delta_binding_observable_maximum_head_logits"] > 0.0
        and mechanics[
            "additive_binding_observable_maximum_head_logits"
        ]
        > 0.0
        and mechanics["equal_runtime_intervention_work"]
        and mechanics["equal_probe_work"]
        and mechanics["forbidden_reads"] == 0
        and mechanics["probe_forbidden_reads"] == 0
        and mechanics["initial_binding_values_byte_identical"]
        and mechanics["initial_qualified_base_byte_identical"]
        and mechanics["equal_optimizer_batch_update_work"]
        and mechanics["full_delta_complete_gradient_coverage"]
        and mechanics["additive_complete_gradient_coverage"]
        and mechanics["both_qualified_bases_unchanged"]
    )


def fitted_mechanics(
    full_delta: Any,
    additive: Any,
    pairs: Sequence[PromptConditioningPair],
    *,
    replay_factory: Callable[[], Any],
    device: torch.device,
    full_fit: Mapping[str, Any],
    additive_fit: Mapping[str, Any],
    initial_binding_values_byte_identical: bool,
    initial_qualified_base_byte_identical: bool,
) -> dict[str, Any]:
    """Exercise both independently fitted arms before their values are destroyed."""

    directions = _directions(pairs)
    inputs, _targets = _batch(directions[:2], device=device)
    full_delta.eval()
    additive.eval()
    transport_checks = transport_mechanics(full_delta, device=device)
    full_artifact = full_delta.export_binding_artifact()
    additive_artifact = additive.export_binding_artifact()
    full_replay = replay_factory()
    additive_replay = replay_factory()
    full_replay.load_binding_artifact(full_artifact)
    additive_replay.load_binding_artifact(additive_artifact)
    full_replay.eval()
    additive_replay.eval()
    try:
        with torch.no_grad():
            full_output = _output(
                full_delta(inputs, intervention="native"), rows=len(inputs)
            )
            additive_output = _output(
                additive(inputs, intervention="no_delta"), rows=len(inputs)
            )
            state_off = _output(
                full_delta(inputs, intervention="state_off"), rows=len(inputs)
            )
            deranged = _output(
                full_delta(inputs, intervention="transport_permuted"),
                rows=len(inputs),
            )
            full_replay_output = _output(
                full_replay(inputs, intervention="native"), rows=len(inputs)
            )
            additive_replay_output = _output(
                additive_replay(inputs, intervention="no_delta"), rows=len(inputs)
            )
            prefix = inputs[:, :PROMPT_TOKENS]
            full_prefix = full_delta(prefix, intervention="native")
            additive_prefix = additive(prefix, intervention="no_delta")
            mutated = inputs.clone()
            mutation_start = PROMPT_TOKENS + 2
            mutated[:, mutation_start:] = torch.flip(
                mutated[:, mutation_start:], dims=(1,)
            )
            full_mutated = _output(
                full_delta(mutated, intervention="native"), rows=len(inputs)
            )
            additive_mutated = _output(
                additive(mutated, intervention="no_delta"), rows=len(inputs)
            )
        full_causal = float(
            (
                full_output.logits[:, : prefix.shape[1]]
                - full_prefix.logits
            )
            .abs()
            .max()
            .cpu()
        )
        additive_causal = float(
            (
                additive_output.logits[:, : prefix.shape[1]]
                - additive_prefix.logits
            )
            .abs()
            .max()
            .cpu()
        )
        full_counterfactual = float(
            (
                full_output.logits[:, :mutation_start]
                - full_mutated.logits[:, :mutation_start]
            )
            .abs()
            .max()
            .cpu()
        )
        additive_counterfactual = float(
            (
                additive_output.logits[:, :mutation_start]
                - additive_mutated.logits[:, :mutation_start]
            )
            .abs()
            .max()
            .cpu()
        )
        signatures = tuple(
            tuple(int(value) for value in output.audit.work_signature())
            for output in (
                full_output,
                additive_output,
                state_off,
                deranged,
                full_replay_output,
                additive_replay_output,
            )
        )
        mechanics: dict[str, Any] = {
            **transport_checks,
            "full_delta_strict_causal_prefix_maximum_logits_delta": full_causal,
            "full_delta_unobserved_target_mutation_maximum_prefix_delta": (
                full_counterfactual
            ),
            "additive_strict_causal_prefix_maximum_logits_delta": additive_causal,
            "additive_unobserved_target_mutation_maximum_prefix_delta": (
                additive_counterfactual
            ),
            "state_off_v1_maximum_logits_delta": float(
                (state_off.logits - state_off.base_logits).abs().max().cpu()
            ),
            "full_delta_artifact_replay_maximum_logits_delta": float(
                (full_output.logits - full_replay_output.logits).abs().max().cpu()
            ),
            "additive_artifact_replay_maximum_logits_delta": float(
                (additive_output.logits - additive_replay_output.logits)
                .abs()
                .max()
                .cpu()
            ),
            "transport_permutation_head_effect": float(
                (full_output.head_logits - deranged.head_logits).abs().max().cpu()
            ),
            "full_delta_binding_observable_maximum_head_logits": float(
                full_output.head_logits.abs().max().cpu()
            ),
            "additive_binding_observable_maximum_head_logits": float(
                additive_output.head_logits.abs().max().cpu()
            ),
            "equal_runtime_intervention_work": len(set(signatures)) == 1,
            "equal_probe_work": False,
            "forbidden_reads": sum(
                int(output.audit.forbidden_reads)
                for output in (
                    full_output,
                    additive_output,
                    state_off,
                    deranged,
                    full_replay_output,
                    additive_replay_output,
                )
            ),
            "probe_forbidden_reads": -1,
            "initial_binding_values_byte_identical": (
                initial_binding_values_byte_identical
            ),
            "initial_qualified_base_byte_identical": (
                initial_qualified_base_byte_identical
            ),
            "equal_optimizer_batch_update_work": (
                full_fit["updates"] == additive_fit["updates"]
                and full_fit["batch_schedule_cid"]
                == additive_fit["batch_schedule_cid"]
            ),
            "full_delta_complete_gradient_coverage": full_fit[
                "all_trainable_values_received_finite_nonzero_gradient"
            ],
            "additive_complete_gradient_coverage": additive_fit[
                "all_trainable_values_received_finite_nonzero_gradient"
            ],
            "both_qualified_bases_unchanged": (
                full_fit["qualified_base_unchanged"]
                and additive_fit["qualified_base_unchanged"]
            ),
        }
    finally:
        full_replay_destroyed = destroy_disposable_weights(full_replay)
        additive_replay_destroyed = destroy_disposable_weights(additive_replay)
    mechanics["full_delta_replay_values_destroyed"] = full_replay_destroyed
    mechanics["additive_replay_values_destroyed"] = additive_replay_destroyed
    mechanics["passed"] = False
    return mechanics


def _bind_probe_work(
    mechanics: dict[str, Any],
    *,
    full: ProbeScore,
    additive: ProbeScore,
    state_off: ProbeScore,
) -> None:
    mechanics["equal_probe_work"] = (
        full.work_signature == additive.work_signature == state_off.work_signature
    )
    mechanics["probe_forbidden_reads"] = (
        full.forbidden_reads + additive.forbidden_reads + state_off.forbidden_reads
    )
    mechanics["passed"] = _mechanics_verdict(mechanics)


def _validate_selector(value: object) -> None:
    selector = _exact_mapping(
        value,
        keys=(
            "schema",
            "v4_population_cid",
            "pair_start",
            "pair_end_exclusive",
            "pairs",
            "selector_cid",
        ),
        label="inputs.selector",
    )
    _verify_self_cid(selector, "selector_cid")
    if (
        selector["schema"] != SELECTOR_SCHEMA
        or selector["v4_population_cid"] != V4_POPULATION_CID
        or selector["pair_start"] != PAIR_START
        or selector["pair_end_exclusive"] != PAIR_STOP
        or selector["selector_cid"] != SELECTOR_CID
    ):
        raise ValueError("cached V2 selector binding differs")
    pairs = selector["pairs"]
    if not isinstance(pairs, list) or len(pairs) != PROBE_PAIRS:
        raise ValueError("cached V2 selector pair count differs")
    for offset, value in enumerate(pairs):
        pair = _exact_mapping(
            value,
            keys=(
                "pair_index",
                "left_source_story_ordinal",
                "left_story_cid",
                "right_source_story_ordinal",
                "right_story_cid",
            ),
            label=f"inputs.selector.pairs[{offset}]",
        )
        if _integer_field(
            pair["pair_index"], label=f"selector pair {offset} index"
        ) != PAIR_START + offset:
            raise ValueError("cached V2 selector pair ordering differs")
        _integer_field(
            pair["left_source_story_ordinal"],
            label=f"selector pair {offset} left ordinal",
        )
        _integer_field(
            pair["right_source_story_ordinal"],
            label=f"selector pair {offset} right ordinal",
        )
        _cid_field(
            pair["left_story_cid"], label=f"selector pair {offset} left CID"
        )
        _cid_field(
            pair["right_story_cid"], label=f"selector pair {offset} right CID"
        )


def _fit_from_record(
    value: object, *, expected_intervention: FitIntervention, label: str
) -> dict[str, Any]:
    fit = _exact_mapping(
        value,
        keys=(
            "intervention",
            "updates",
            "elapsed_seconds",
            "final_loss",
            "final_gradient_norm",
            "gradient_values_seen",
            "gradient_values_required",
            "all_trainable_values_received_finite_nonzero_gradient",
            "qualified_base_unchanged",
            "initial_binding_cid",
            "fitted_binding_cid",
            "batch_schedule_cid",
        ),
        label=label,
    )
    if fit["intervention"] != expected_intervention:
        raise ValueError(f"cached {label} intervention differs")
    updates = _integer_field(fit["updates"], label=f"{label}.updates", minimum=1)
    if updates != MAXIMUM_UPDATES:
        raise ValueError(f"cached {label} does not use the exact frozen dose")
    elapsed = _float_field(
        fit["elapsed_seconds"], label=f"{label}.elapsed_seconds", minimum=0.0
    )
    _float_field(fit["final_loss"], label=f"{label}.final_loss", minimum=0.0)
    _float_field(
        fit["final_gradient_norm"],
        label=f"{label}.final_gradient_norm",
        minimum=0.0,
    )
    seen = _integer_field(
        fit["gradient_values_seen"], label=f"{label}.gradient_values_seen"
    )
    required = _integer_field(
        fit["gradient_values_required"], label=f"{label}.gradient_values_required"
    )
    coverage = _boolean_field(
        fit["all_trainable_values_received_finite_nonzero_gradient"],
        label=f"{label}.gradient_coverage",
    )
    base_unchanged = _boolean_field(
        fit["qualified_base_unchanged"], label=f"{label}.base_unchanged"
    )
    initial_cid = _cid_field(
        fit["initial_binding_cid"], label=f"{label}.initial_binding_cid"
    )
    fitted_cid = _cid_field(
        fit["fitted_binding_cid"], label=f"{label}.fitted_binding_cid"
    )
    schedule_cid = _cid_field(
        fit["batch_schedule_cid"], label=f"{label}.batch_schedule_cid"
    )
    if (
        required != TRAINABLE_PARAMETERS
        or seen > required
        or coverage is not (seen == required)
        or schedule_cid != _batch_schedule_cid(PROBE_DIRECTIONS, updates)
    ):
        raise ValueError(f"cached {label} fit ledger differs")
    return {
        "updates": updates,
        "elapsed_seconds": elapsed,
        "gradient_values_seen": seen,
        "gradient_values_required": required,
        "coverage": coverage,
        "base_unchanged": base_unchanged,
        "initial_binding_cid": initial_cid,
        "fitted_binding_cid": fitted_cid,
        "batch_schedule_cid": schedule_cid,
    }


_MECHANICS_KEYS = (
    "all_frame_identity_maximum_delta",
    "all_frame_step_connection_maximum_delta",
    "transported_matrix_read_covariance_maximum_delta",
    "full_delta_strict_causal_prefix_maximum_logits_delta",
    "full_delta_unobserved_target_mutation_maximum_prefix_delta",
    "additive_strict_causal_prefix_maximum_logits_delta",
    "additive_unobserved_target_mutation_maximum_prefix_delta",
    "state_off_v1_maximum_logits_delta",
    "full_delta_artifact_replay_maximum_logits_delta",
    "additive_artifact_replay_maximum_logits_delta",
    "transport_permutation_head_effect",
    "full_delta_binding_observable_maximum_head_logits",
    "additive_binding_observable_maximum_head_logits",
    "equal_runtime_intervention_work",
    "equal_probe_work",
    "forbidden_reads",
    "probe_forbidden_reads",
    "initial_binding_values_byte_identical",
    "initial_qualified_base_byte_identical",
    "equal_optimizer_batch_update_work",
    "full_delta_complete_gradient_coverage",
    "additive_complete_gradient_coverage",
    "both_qualified_bases_unchanged",
    "full_delta_replay_values_destroyed",
    "additive_replay_values_destroyed",
    "passed",
)


def _validate_historical_v2_implementation(value: object) -> None:
    """Validate the stored V2 tree while allowing later campaign modules."""

    implementation = _exact_mapping(
        value, keys=("files", "tree_cid"), label="V2 implementation"
    )
    files = implementation["files"]
    if not isinstance(files, list) or not files:
        raise ValueError("cached V2 implementation file ledger is invalid")
    paths: list[str] = []
    for offset, item in enumerate(files):
        record = _exact_mapping(
            item,
            keys=("bytes", "cid", "path"),
            label=f"V2 implementation.files[{offset}]",
        )
        if not isinstance(record["path"], str) or not record["path"]:
            raise ValueError("cached V2 implementation path is invalid")
        _integer_field(
            record["bytes"],
            label=f"V2 implementation.files[{offset}].bytes",
            minimum=1,
        )
        _cid_field(
            record["cid"], label=f"V2 implementation.files[{offset}].cid"
        )
        paths.append(record["path"])
    observed = _cid_field(
        implementation["tree_cid"], label="V2 implementation.tree_cid"
    )
    if paths != sorted(set(paths)) or tree_cid(list(files)) != observed:
        raise ValueError("cached predictive V2 implementation binding differs")
    if observed == V2_IMPLEMENTATION_TREE_CID:
        return
    if implementation != trainer_implementation_contract():
        raise ValueError("cached predictive V2 implementation binding differs")


def _validate_cached_result(value: Mapping[str, Any]) -> None:
    """Fail closed while reproducing all V2 decision-bearing fields."""

    _verify_self_cid(value, "result_cid")
    result = _exact_mapping(
        value,
        keys=(
            "schema",
            "issue",
            "policy",
            "model_policy",
            "implementation",
            "execution",
            "inputs",
            "dose",
            "mechanics",
            "fits",
            "scores",
            "native_capacity",
            "additive_attribution",
            "verdict",
            "admitted",
            "disposable_weights",
            "production_v5",
            "writer_process_id",
            "result_cid",
        ),
        label="V2 result",
    )
    if (
        result["schema"] != RESULT_SCHEMA
        or result["issue"] != ISSUE
        or result["policy"] != POLICY
        or result["model_policy"] != MODEL_POLICY
    ):
        raise ValueError("cached predictive V2 identity differs")
    _integer_field(result["issue"], label="issue", minimum=1)
    _integer_field(result["writer_process_id"], label="writer_process_id", minimum=1)
    _validate_historical_v2_implementation(result["implementation"])

    execution = _exact_mapping(
        result["execution"],
        keys=(
            "device",
            "torch_intraop_threads",
            "torch_interop_threads",
            "total_elapsed_seconds",
        ),
        label="execution",
    )
    intraop = _integer_field(
        execution["torch_intraop_threads"], label="execution.intraop", minimum=1
    )
    interop = _integer_field(
        execution["torch_interop_threads"], label="execution.interop", minimum=1
    )
    total_elapsed = _float_field(
        execution["total_elapsed_seconds"],
        label="execution.total_elapsed_seconds",
        minimum=0.0,
    )
    if (
        execution["device"] != "cpu"
        or intraop != CPU_THREADS
        or interop > 1_024
        or total_elapsed > HARD_WALL_SECONDS
    ):
        raise ValueError("cached predictive V2 CPU8 execution differs")

    inputs = _exact_mapping(
        result["inputs"],
        keys=(
            "predictive_v1",
            "predecessor",
            "revealed_v4",
            "selector",
            "h4_spin_frames",
        ),
        label="inputs",
    )
    if inputs["predictive_v1"] != {
        "result_cid": V1_RESULT_CID,
        "verdict": V1_VERDICT,
        "admitted": False,
        "implementation_tree_cid": V1_IMPLEMENTATION_TREE_CID,
    }:
        raise ValueError("cached predictive V1 historical binding differs")
    if inputs["predecessor"] != {
        "policy": PREDECESSOR_POLICY,
        "result_cid": PREDECESSOR_RESULT_CID,
        "artifact_cid": PREDECESSOR_ARTIFACT_CID,
        "artifact_bytes": PREDECESSOR_ARTIFACT_BYTES,
    }:
        raise ValueError("cached V2 predecessor binding differs")
    if inputs["revealed_v4"] != {
        "population_cid": V4_POPULATION_CID,
        "commitment_cid": V4_COMMITMENT_CID,
        "reveal_cid": V4_REVEAL_CID,
        "pair_start": PAIR_START,
        "pair_stop_exclusive": PAIR_STOP,
        "pairs": PROBE_PAIRS,
        "directions": PROBE_DIRECTIONS,
        "targets": PROBE_TARGETS,
        "slice_records_cid": SLICE_RECORDS_CID,
        "ordered_identities_cid": SLICE_IDENTITIES_CID,
    }:
        raise ValueError("cached V2 revealed-slice binding differs")
    _validate_selector(inputs["selector"])
    if inputs["h4_spin_frames"] != {
        "artifact_cid": H4_FRAME_ARTIFACT_CID,
        "file_cid": H4_FRAME_FILE_CID,
        "root_table_kappa": ROOT_TABLE_KAPPA,
        "multiplication_table_kappa": PRODUCT_TABLE_KAPPA,
    }:
        raise ValueError("cached V2 H4 binding differs")

    dose = _exact_mapping(
        result["dose"],
        keys=(
            "pairs",
            "directions",
            "targets",
            "maximum_updates_per_arm",
            "batch_directions",
            "optimizer",
            "cuda",
        ),
        label="dose",
    )
    if dict(dose) != {
        "pairs": PROBE_PAIRS,
        "directions": PROBE_DIRECTIONS,
        "targets": PROBE_TARGETS,
        "maximum_updates_per_arm": MAXIMUM_UPDATES,
        "batch_directions": BATCH_DIRECTIONS,
        "optimizer": {
            "name": "AdamW",
            "learning_rate": LEARNING_RATE,
            "betas": list(ADAM_BETAS),
            "epsilon": ADAM_EPSILON,
            "weight_decay": 0.0,
            "gradient_clip": GRADIENT_CLIP,
        },
        "cuda": "FORBIDDEN",
    }:
        raise ValueError("cached predictive V2 dose differs")

    fits = _exact_mapping(
        result["fits"],
        keys=("full_delta", "additive_no_overwrite"),
        label="fits",
    )
    full_fit = _fit_from_record(
        fits["full_delta"], expected_intervention="native", label="fits.full_delta"
    )
    additive_fit = _fit_from_record(
        fits["additive_no_overwrite"],
        expected_intervention="no_delta",
        label="fits.additive_no_overwrite",
    )
    if (
        full_fit["updates"] != additive_fit["updates"]
        or full_fit["batch_schedule_cid"] != additive_fit["batch_schedule_cid"]
        or full_fit["initial_binding_cid"] != additive_fit["initial_binding_cid"]
        or full_fit["elapsed_seconds"] + additive_fit["elapsed_seconds"]
        > total_elapsed + 1e-9
    ):
        raise ValueError("cached independent V2 fits are not matched")

    mechanics = _exact_mapping(
        result["mechanics"], keys=_MECHANICS_KEYS, label="mechanics"
    )
    float_fields = _MECHANICS_KEYS[:13]
    for field in float_fields:
        _float_field(mechanics[field], label=f"mechanics.{field}", minimum=0.0)
    boolean_fields = (
        "equal_runtime_intervention_work",
        "equal_probe_work",
        "initial_binding_values_byte_identical",
        "initial_qualified_base_byte_identical",
        "equal_optimizer_batch_update_work",
        "full_delta_complete_gradient_coverage",
        "additive_complete_gradient_coverage",
        "both_qualified_bases_unchanged",
        "passed",
    )
    for field in boolean_fields:
        _boolean_field(mechanics[field], label=f"mechanics.{field}")
    for field in (
        "forbidden_reads",
        "probe_forbidden_reads",
        "full_delta_replay_values_destroyed",
        "additive_replay_values_destroyed",
    ):
        _integer_field(mechanics[field], label=f"mechanics.{field}")
    if (
        mechanics["full_delta_replay_values_destroyed"] != TRAINABLE_PARAMETERS
        or mechanics["additive_replay_values_destroyed"] != TRAINABLE_PARAMETERS
        or mechanics["full_delta_complete_gradient_coverage"]
        is not full_fit["coverage"]
        or mechanics["additive_complete_gradient_coverage"]
        is not additive_fit["coverage"]
        or mechanics["both_qualified_bases_unchanged"]
        is not (full_fit["base_unchanged"] and additive_fit["base_unchanged"])
        or mechanics["initial_binding_values_byte_identical"]
        is not (full_fit["initial_binding_cid"] == additive_fit["initial_binding_cid"])
        or mechanics["equal_optimizer_batch_update_work"]
        is not (
            full_fit["updates"] == additive_fit["updates"]
            and full_fit["batch_schedule_cid"] == additive_fit["batch_schedule_cid"]
        )
    ):
        raise ValueError("cached V2 mechanics/fit binding differs")

    scores = _exact_mapping(
        result["scores"],
        keys=("full_delta", "additive_no_overwrite", "state_off"),
        label="scores",
    )
    full = _probe_score_from_record(
        scores["full_delta"], expected_intervention="native", label="scores.full_delta"
    )
    additive = _probe_score_from_record(
        scores["additive_no_overwrite"],
        expected_intervention="no_delta",
        label="scores.additive_no_overwrite",
    )
    state_off = _probe_score_from_record(
        scores["state_off"],
        expected_intervention="state_off",
        label="scores.state_off",
    )
    expected_equal_probe_work = (
        full.work_signature == additive.work_signature == state_off.work_signature
    )
    expected_probe_forbidden = (
        full.forbidden_reads + additive.forbidden_reads + state_off.forbidden_reads
    )
    if (
        mechanics["equal_probe_work"] is not expected_equal_probe_work
        or mechanics["probe_forbidden_reads"] != expected_probe_forbidden
        or mechanics["passed"] is not _mechanics_verdict(mechanics)
    ):
        raise ValueError("cached V2 mechanics verdict does not reproduce")

    expected_native = native_capacity_decision(
        full=full, state_off=state_off, mechanics=mechanics
    )
    expected_additive = additive_attribution_decision(
        full=full, additive=additive, state_off=state_off
    )
    if result["native_capacity"] != expected_native:
        raise ValueError("cached V2 native-capacity decision does not reproduce")
    if result["additive_attribution"] != expected_additive:
        raise ValueError("cached V2 additive attribution does not reproduce")
    admitted = _boolean_field(result["admitted"], label="admitted")
    if (
        result["verdict"] != expected_native["verdict"]
        or admitted is not expected_native["admitted"]
    ):
        raise ValueError("cached V2 top-level decision differs")

    disposal = _exact_mapping(
        result["disposable_weights"],
        keys=(
            "status",
            "full_delta_values",
            "additive_no_overwrite_values",
            "artifacts_written",
        ),
        label="disposable_weights",
    )
    if disposal != {
        "status": "DESTROYED_IN_MEMORY_NO_ARTIFACT",
        "full_delta_values": TRAINABLE_PARAMETERS,
        "additive_no_overwrite_values": TRAINABLE_PARAMETERS,
        "artifacts_written": 0,
    }:
        raise ValueError("cached V2 fitted-weight destruction differs")
    production = _exact_mapping(
        result["production_v5"],
        keys=("authorized", "created", "inspected", "selector"),
        label="production_v5",
    )
    if production != {
        "authorized": admitted,
        "created": False,
        "inspected": False,
        "selector": "NOT_IMPLEMENTED_IN_V2_PREFLIGHT_MODULE",
    }:
        raise ValueError("cached V2 V5 authorization boundary differs")


def _default_model_factory(
    inputs: FrozenV2Inputs, arm: str, device: torch.device
) -> Any:
    from .predictive_block_delta_binding import R4PredictiveBlockDeltaBindingV1

    geometry = _exact_geometry(inputs.predecessor)
    model = R4PredictiveBlockDeltaBindingV1(
        geometry, inputs.frames, arm=arm
    ).to(device)
    model.load_qualified_base_artifact(inputs.predecessor_artifact_path.read_bytes())
    return model


def run_predictive_block_delta_v2_preflight(
    *,
    root: Path,
    predecessor_root: Path,
    revealed_v4_root: Path,
    frame_sidecar_path: Path,
    v1_result_path: Path,
    device: torch.device | str = "cpu",
    maximum_updates: int = MAXIMUM_UPDATES,
    model_factory: Callable[[FrozenV2Inputs, str, torch.device], Any] = (
        _default_model_factory
    ),
) -> dict[str, Any]:
    """Run the independent-arm V2 correction without selecting or opening V5."""

    if maximum_updates != MAXIMUM_UPDATES:
        raise ValueError(
            "predictive block-delta V2 requires exactly 256 updates per arm"
        )
    root = root.resolve()
    result_path = root / RESULT_RELATIVE_PATH
    if result_path.exists() or result_path.is_symlink():
        result = _read_canonical_json(result_path)
        _validate_cached_result(result)
        return result
    gate_started = time.monotonic()
    selected_device = torch.device(device)
    if selected_device.type != "cpu":
        raise ValueError("predictive block-delta V2 is frozen to CPU execution")
    if torch.get_num_threads() != CPU_THREADS:
        raise ValueError("predictive block-delta V2 requires an explicit CPU8 process")
    if not 1 <= torch.get_num_interop_threads() <= 1_024:
        raise ValueError("predictive block-delta V2 interop thread count is invalid")
    def remaining_gate_seconds() -> float:
        remaining = HARD_WALL_SECONDS - (time.monotonic() - gate_started)
        if remaining <= 0.0:
            raise TimeoutError("predictive block-delta V2 exceeded its five-minute wall")
        return remaining

    frozen = load_frozen_v2_inputs(
        predecessor_root=predecessor_root,
        revealed_v4_root=revealed_v4_root,
        frame_sidecar_path=frame_sidecar_path,
        v1_result_path=v1_result_path,
    )
    remaining_gate_seconds()

    def fresh_model() -> Any:
        torch.manual_seed(INITIALIZATION_SEED)
        return model_factory(frozen, "geometric", selected_device)

    full_delta = fresh_model()
    additive = fresh_model()
    initial_full_binding = full_delta.export_binding_artifact()
    initial_additive_binding = additive.export_binding_artifact()
    initial_full_base = full_delta.export_qualified_base_artifact()
    initial_additive_base = additive.export_qualified_base_artifact()
    initial_binding_equal = initial_full_binding == initial_additive_binding
    initial_base_equal = initial_full_base == initial_additive_base
    if not initial_binding_equal or not initial_base_equal:
        raise ValueError("independent V2 arms did not start byte-identically")

    full_fit = fit_independent_arm(
        full_delta,
        frozen.pairs,
        intervention="native",
        device=selected_device,
        maximum_updates=maximum_updates,
        hard_wall_seconds=remaining_gate_seconds(),
    )
    additive_fit = fit_independent_arm(
        additive,
        frozen.pairs,
        intervention="no_delta",
        device=selected_device,
        maximum_updates=maximum_updates,
        hard_wall_seconds=remaining_gate_seconds(),
    )
    mechanics = fitted_mechanics(
        full_delta,
        additive,
        frozen.pairs,
        replay_factory=fresh_model,
        device=selected_device,
        full_fit=full_fit,
        additive_fit=additive_fit,
        initial_binding_values_byte_identical=initial_binding_equal,
        initial_qualified_base_byte_identical=initial_base_equal,
    )
    remaining_gate_seconds()
    full_score = score_probe(
        full_delta, frozen.pairs, intervention="native", device=selected_device
    )
    remaining_gate_seconds()
    additive_score = score_probe(
        additive, frozen.pairs, intervention="no_delta", device=selected_device
    )
    remaining_gate_seconds()
    state_off_score = score_probe(
        full_delta, frozen.pairs, intervention="state_off", device=selected_device
    )
    _bind_probe_work(
        mechanics,
        full=full_score,
        additive=additive_score,
        state_off=state_off_score,
    )
    native = native_capacity_decision(
        full=full_score, state_off=state_off_score, mechanics=mechanics
    )
    attribution = additive_attribution_decision(
        full=full_score, additive=additive_score, state_off=state_off_score
    )
    full_destroyed = destroy_disposable_weights(full_delta)
    additive_destroyed = destroy_disposable_weights(additive)
    total_elapsed = time.monotonic() - gate_started
    if total_elapsed > HARD_WALL_SECONDS:
        raise TimeoutError("predictive block-delta V2 exceeded its five-minute wall")

    result = _with_self_cid(
        {
            "schema": RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "model_policy": MODEL_POLICY,
            "implementation": trainer_implementation_contract(),
            "execution": {
                "device": str(selected_device),
                "torch_intraop_threads": torch.get_num_threads(),
                "torch_interop_threads": torch.get_num_interop_threads(),
                "total_elapsed_seconds": total_elapsed,
            },
            "inputs": dict(frozen.records),
            "dose": {
                "pairs": PROBE_PAIRS,
                "directions": PROBE_DIRECTIONS,
                "targets": PROBE_TARGETS,
                "maximum_updates_per_arm": MAXIMUM_UPDATES,
                "batch_directions": BATCH_DIRECTIONS,
                "optimizer": {
                    "name": "AdamW",
                    "learning_rate": LEARNING_RATE,
                    "betas": list(ADAM_BETAS),
                    "epsilon": ADAM_EPSILON,
                    "weight_decay": 0.0,
                    "gradient_clip": GRADIENT_CLIP,
                },
                "cuda": "FORBIDDEN",
            },
            "mechanics": mechanics,
            "fits": {
                "full_delta": full_fit,
                "additive_no_overwrite": additive_fit,
            },
            "scores": {
                "full_delta": full_score.record(),
                "additive_no_overwrite": additive_score.record(),
                "state_off": state_off_score.record(),
            },
            "native_capacity": native,
            "additive_attribution": attribution,
            "verdict": native["verdict"],
            "admitted": native["admitted"],
            "disposable_weights": {
                "status": "DESTROYED_IN_MEMORY_NO_ARTIFACT",
                "full_delta_values": full_destroyed,
                "additive_no_overwrite_values": additive_destroyed,
                "artifacts_written": 0,
            },
            "production_v5": {
                "authorized": native["admitted"],
                "created": False,
                "inspected": False,
                "selector": "NOT_IMPLEMENTED_IN_V2_PREFLIGHT_MODULE",
            },
            "writer_process_id": os.getpid(),
        },
        "result_cid",
    )
    _validate_cached_result(result)
    result_path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(result_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(descriptor, "wb", closefd=True) as target:
            descriptor = -1
            target.write(canonical_json_bytes(result))
            target.flush()
            os.fsync(target.fileno())
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    return result
