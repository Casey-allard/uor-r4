"""Command line for the bounded #1014/#1017/#1019/#954/#973 campaigns."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from .admission import admit_rust_smoke_qualification
from .capacity import (
    admit_capacity_prefix_parity,
    admit_capacity_smoke,
    reveal_capacity,
    run_capacity_hardware_probe,
    run_capacity_overfit_smoke,
    train_capacity,
)
from .capacity_data import (
    load_capacity_training_view_manifest,
    prepare_capacity_dataset,
)
from .capacity_finalize import (
    finalize_capacity,
    verify_capacity_generation_ready,
    write_capacity_rubric_template,
)
from .continuation import (
    admit_enabled_prefix_parity,
    reveal_continuation,
    train_continuation,
)
from .continuation_data import (
    load_continuation_training_view_manifest,
    prepare_continuation_dataset,
)
from .contextual_retained_fit import (
    fit_contextual_key_value_address_read,
    fit_contextual_key_value,
    fit_contextual_retained,
    fit_contextual_retained_full,
)
from .contextual_retained_generation import generate_contextual_retained
from .data import download_source, load_dataset_manifest, prepare_dataset
from .direct_retained_readout_campaign import (
    prepare_direct_retained_readout,
    probe_direct_retained_readout,
    run_direct_retained_readout,
    verify_direct_retained_readout_result,
)
from .finalize import finalize_continuation
from .grounding import train_grounding
from .group_retention_campaign import (
    prepare_group_retention_data,
    run_group_retention_preflight,
)
from .group_retention_decoder_campaign import (
    prepare_group_retention_decoder_data,
    run_group_retention_decoder_preflight,
)
from .group_retention_decoder_cpu_recovery_campaign import (
    prepare_group_retention_decoder_cpu_recovery_data,
    run_group_retention_decoder_cpu_recovery_preflight,
)
from .joint_candidate_margin_campaign import (
    prepare_joint_candidate_margin_data,
    run_joint_candidate_margin_preflight,
)
from .language_path_generalization_campaign import (
    probe_language_path_execution,
    run_language_path_generalization,
)
from .language_path_generalization_data import prepare_language_path_data
from .language_path_generation import run_language_path_generation
from .ordinary_language_generation import generate_ordinary_language_path
from .layerwise_normalized_retained_readout_campaign import (
    prepare_layerwise_normalized_retained_readout,
    probe_layerwise_normalized_retained_readout,
    run_layerwise_normalized_retained_readout,
    verify_layerwise_normalized_retained_readout_result,
)
from .learned_associative_readout_campaign import (
    prepare_learned_associative_readout,
    probe_learned_associative_readout,
    run_learned_associative_readout,
    verify_learned_associative_readout_result,
)
from .paired_h4_prompt_capacity_campaign import (
    prepare_paired_h4_prompt_capacity,
    probe_paired_h4_prompt_capacity,
    run_paired_h4_prompt_capacity,
)
from .paired_query_binding_campaign import (
    prepare_paired_query_binding_data,
    run_paired_query_binding_preflight,
)
from .predictive_block_delta_campaign import (
    MAXIMUM_UPDATES as PREDICTIVE_BLOCK_DELTA_MAXIMUM_UPDATES,
    run_predictive_block_delta_preflight,
)
from .predictive_block_delta_campaign_v2 import (
    run_predictive_block_delta_v2_preflight,
)
from .predictive_block_delta_terminal_campaign import (
    prepare_predictive_block_delta_terminal,
    probe_predictive_block_delta_terminal,
    run_predictive_block_delta_terminal,
    verify_predictive_block_delta_terminal,
)
from .position_kv_binding_campaign import (
    collect_position_kv_story_exclusions,
    preflight_position_kv_binding_campaign,
    prepare_position_kv_binding_campaign,
    run_position_kv_binding_campaign,
    validate_position_kv_binding_result,
)
from .position_r4_language_generation import generate_position_r4_language_path
from .fixed_recurrent_r4_language_generation import (
    generate_fixed_recurrent_r4_language_path,
)
from .sparse_geometric_r4_language_generation import (
    generate_sparse_geometric_r4_language_path,
)
from .role_tagged_associative_development import (
    preflight_role_tagged_associative_development,
    prepare_role_tagged_associative_development,
    run_role_tagged_associative_development,
    verify_role_tagged_associative_development,
)
from .paths import (
    default_attended_relation_adapter_root,
    default_capacity_root,
    default_continuation_root,
    default_direct_retained_readout_predecessor,
    default_direct_retained_readout_raw_source,
    default_direct_retained_readout_root,
    default_direct_retained_readout_source_train,
    default_direct_retained_readout_source_train_index,
    default_grounding_predecessor_root,
    default_grounding_root,
    default_group_retention_decoder_cpu_recovery_root,
    default_group_retention_decoder_root,
    default_group_retention_root,
    default_group_retention_source_root,
    default_joint_candidate_margin_root,
    default_language_path_geometry,
    default_language_path_root,
    default_language_path_source_root,
    default_learned_associative_readout_predecessor,
    default_learned_associative_readout_raw_source,
    default_learned_associative_readout_root,
    default_learned_associative_readout_source_train,
    default_learned_associative_readout_source_train_index,
    default_learned_associative_readout_v1_population,
    default_learned_associative_readout_v2_population,
    default_learned_associative_readout_v3_population,
    default_layerwise_normalized_readout_predecessor,
    default_layerwise_normalized_readout_raw_source,
    default_layerwise_normalized_readout_root,
    default_layerwise_normalized_readout_source_train,
    default_layerwise_normalized_readout_source_train_index,
    default_layerwise_normalized_readout_v1_population,
    default_layerwise_normalized_readout_v2_population,
    default_paired_h4_prompt_capacity_predecessor,
    default_paired_h4_prompt_capacity_raw_source,
    default_paired_h4_prompt_capacity_root,
    default_paired_h4_prompt_capacity_source_train,
    default_paired_query_binding_root,
    default_predictive_block_delta_frame_sidecar,
    default_predictive_block_delta_predecessor,
    default_predictive_block_delta_revealed_v4_root,
    default_predictive_block_delta_root,
    default_predictive_block_delta_v1_result,
    default_predictive_block_delta_v2_root,
    default_predictive_block_delta_terminal_prior_populations,
    default_predictive_block_delta_terminal_root,
    default_predictive_block_delta_terminal_v2_result,
    default_research_root,
    default_source_relation_head_root,
    default_source_span_pointer_root,
    model_store_root,
)
from .provenance import verify_bound_manifest
from .source_relation_adapter_campaign import (
    prepare_attended_relation_data,
    run_attended_relation_preflight,
)
from .source_relation_head import train_source_relation_head
from .source_span_pointer import train_source_span_pointer
from .train import TrainConfig, reveal_sealed_test, run_overfit_smoke, train_main


def _root(value: str) -> Path:
    return Path(value).expanduser().resolve()


def _predictive_block_delta_updates(value: str) -> int:
    try:
        updates = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("maximum updates must be an integer") from error
    if not 1 <= updates <= PREDICTIVE_BLOCK_DELTA_MAXIMUM_UPDATES:
        raise argparse.ArgumentTypeError(
            "maximum updates must be between 1 and "
            f"{PREDICTIVE_BLOCK_DELTA_MAXIMUM_UPDATES}"
        )
    return updates


def _print_result(value: dict[str, Any]) -> None:
    print(json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2))


def _default_position_kv_binding_root() -> Path:
    return model_store_root() / "research" / "issue-1043-position-kv-binding"


def _default_role_tagged_associative_root() -> Path:
    return model_store_root() / "research" / "issue-1045-role-tagged-associative"


def parser() -> argparse.ArgumentParser:
    command = argparse.ArgumentParser(
        prog="r4-softmax-trainer",
        description="Frozen causal-softmax language-model campaigns for UOR-R4",
    )
    command.add_argument(
        "--root",
        type=_root,
        help=(
            "untracked data/checkpoint root (defaults to issue-1014, issue-1017, "
            "issue-1019, issue-973, or the selected issue-954 grounding mechanism "
            "according to the lifecycle)"
        ),
    )
    subcommands = command.add_subparsers(dest="command", required=True)

    subcommands.add_parser("download", help="download and verify the exact pinned TinyStories file")
    prepare = subcommands.add_parser("prepare", help="split, train BPE, and build capped token stores")
    prepare.add_argument("--source", type=_root, help="pre-existing exact pinned source file")
    prepare.add_argument("--force", action="store_true", help="rebuild known derived files")

    smoke = subcommands.add_parser("smoke", help="run the 64-sequence overfit admission gate")
    smoke.add_argument("--max-seconds", type=float, default=300.0)

    admit = subcommands.add_parser(
        "admit",
        help="verify and freeze the passed smoke export plus real Rust two-arm parity",
    )
    admit.add_argument(
        "--rust-qualification",
        type=_root,
        required=True,
        help="JSON emitted by r4-softmax-local-qualify against smoke/export",
    )

    train = subcommands.add_parser("train", help="run or resume the one frozen main campaign")
    train.add_argument("--resume", action="store_true", help="resume checkpoints/latest.pt exactly")

    subcommands.add_parser(
        "reveal",
        help="after selection CID freeze, evaluate enabled/off sealed NLL and emit parity fixtures",
    )

    verify = subcommands.add_parser("verify", help="reproduce dataset and optional export CIDs")
    verify.add_argument("--export", action="store_true", help="also verify export/export-manifest.json")

    continuation = subcommands.add_parser(
        "prepare-continuation",
        help="build #1017's fresh story-aligned population without opening #1014 sealed data",
    )
    continuation.add_argument(
        "--predecessor-root",
        type=_root,
        default=default_research_root(),
        help="immutable #1014 research root",
    )
    continuation.add_argument("--source", type=_root, help="exact pinned TinyStories source")
    continuation.add_argument("--force", action="store_true")

    continue_run = subcommands.add_parser(
        "continue", help="run or resume the one frozen #1017 continuation"
    )
    continue_run.add_argument("--resume", action="store_true")

    enabled = subcommands.add_parser(
        "admit-enabled-parity",
        help="bind the sole enabled-only Rust 32-token qualification",
    )
    enabled.add_argument("--rust-qualification", type=_root, required=True)
    subcommands.add_parser(
        "reveal-continuation",
        help="irreversibly open and score #1017's fresh enabled-only confirmation",
    )
    subcommands.add_parser(
        "verify-continuation-training",
        help="verify #1017's nonsealed training view while confirmation remains denied",
    )
    finalize = subcommands.add_parser(
        "finalize-continuation",
        help="bind the opened reveal, ten existing Rust generations, and human rubric once",
    )
    finalize.add_argument(
        "--rubric",
        type=_root,
        required=True,
        help="independently prepared exact five-record human rubric JSON",
    )

    capacity = subcommands.add_parser(
        "prepare-capacity",
        help="build #1019's canonical train stream and fresh sealed populations",
    )
    capacity.add_argument(
        "--predecessor-root",
        type=_root,
        default=default_continuation_root(),
        help="immutable #1017 research root",
    )
    capacity.add_argument("--source", type=_root, help="exact pinned TinyStories source")

    smoke_capacity = subcommands.add_parser(
        "smoke-capacity", help="run #1019's create-once 64-sequence overfit smoke"
    )
    smoke_capacity.add_argument("--backend", choices=["mps"], required=True)
    smoke_capacity.add_argument("--max-seconds", type=float, default=600.0)

    admit_capacity = subcommands.add_parser(
        "admit-capacity-smoke",
        help="bind #1019's passed overfit export and enabled-only Rust parity",
    )
    admit_capacity.add_argument("--rust-qualification", type=_root, required=True)

    probe_capacity = subcommands.add_parser(
        "probe-capacity",
        help="run #1019's exact 200-step accelerator time/memory admission",
    )
    probe_capacity.add_argument("--backend", choices=["mps"], required=True)

    train_capacity_parser = subcommands.add_parser(
        "train-capacity", help="run or resume the one frozen #1019 campaign"
    )
    train_capacity_parser.add_argument("--backend", choices=["mps"], required=True)
    train_capacity_parser.add_argument("--resume", action="store_true")

    admit_capacity_parity = subcommands.add_parser(
        "admit-capacity-parity",
        help="bind the selected #1019 checkpoint's enabled-only Rust parity",
    )
    admit_capacity_parity.add_argument("--rust-qualification", type=_root, required=True)

    reveal_capacity_parser = subcommands.add_parser(
        "reveal-capacity",
        help="irreversibly score #1019 and frozen #1017 on the fresh confirmation",
    )
    reveal_capacity_parser.add_argument(
        "--baseline-1017-root",
        type=_root,
        default=default_continuation_root(),
        help="immutable completed #1017 research root",
    )
    subcommands.add_parser(
        "verify-capacity-training",
        help="verify #1019's nonsealed view while confirmation remains denied",
    )
    capacity_rubric = subcommands.add_parser(
        "prepare-capacity-rubric",
        help="validate #1019 generation pairs and write a review-only rubric template",
    )
    capacity_rubric.add_argument("--output", type=_root, required=True)
    subcommands.add_parser(
        "verify-capacity-generation-ready",
        help="read-only validation before #1019's irreversible generation stage",
    )
    finalize_capacity_parser = subcommands.add_parser(
        "finalize-capacity",
        help="bind #1019's reveal, ten Rust generation reports, and human rubric",
    )
    finalize_capacity_parser.add_argument("--rubric", type=_root, required=True)

    grounding = subcommands.add_parser(
        "finetune-grounding",
        help="run the fixed 384-step MPS context-grounding SFT over #1017",
    )
    grounding.add_argument(
        "--predecessor",
        type=_root,
        default=default_grounding_predecessor_root(),
        help="completed #1017 Hugging Face export",
    )
    grounding.add_argument(
        "--resume",
        action="store_true",
        help="resume the identical fixed run from checkpoints/latest.pt",
    )
    pointer = subcommands.add_parser(
        "train-source-span-pointer",
        help=(
            "extract immutable #1017 states, gate the C1-SB1 pointer, and run its "
            "sole 256-step fit only after Rust score parity"
        ),
    )
    pointer.add_argument(
        "--predecessor",
        type=_root,
        default=default_grounding_predecessor_root(),
        help="immutable completed #1017 Hugging Face export",
    )
    pointer.add_argument(
        "--rust-score-parity",
        type=_root,
        help=(
            "uor-r4.grounded-answer/2 JSON from r4 answer using the emitted "
            "preflight head/source; required only on the second invocation"
        ),
    )
    relation = subcommands.add_parser(
        "train-source-relation-head",
        help=(
            "run the C1-SB2 matched-transfer gate, admit three Rust parity "
            "reports, and only then perform its sole 512-step relation-head fit"
        ),
    )
    relation.add_argument(
        "--predecessor",
        type=_root,
        default=default_grounding_predecessor_root(),
        help="immutable completed #1017 Hugging Face export",
    )
    relation.add_argument(
        "--rust-score-parity",
        type=_root,
        nargs=3,
        metavar=("ANSWER_JSON", "ABSTAIN_JSON", "CONFLICT_JSON"),
        help=(
            "three uor-r4.grounded-answer/3 reports emitted from the preflight "
            "head; required only on the second invocation"
        ),
    )
    prepare_adapter = subcommands.add_parser(
        "prepare-attended-relation",
        help=(
            "commit C1-SB3 independent relation data and sealed product CIDs "
            "without starting optimization"
        ),
    )
    prepare_adapter.add_argument(
        "--predecessor",
        type=_root,
        default=default_grounding_predecessor_root(),
        help="immutable completed #1017 Hugging Face export",
    )
    train_adapter = subcommands.add_parser(
        "train-attended-relation-preflight",
        help=(
            "run the sole <=10 minute C1-SB3 representation-transfer preflight; "
            "never opens product text"
        ),
    )
    train_adapter.add_argument(
        "--predecessor",
        type=_root,
        default=default_grounding_predecessor_root(),
        help="immutable completed #1017 Hugging Face export",
    )
    prepare_joint = subcommands.add_parser(
        "prepare-joint-candidate-margin",
        help=(
            "commit the C1-SB4 fresh joint-candidate data and sealed product "
            "CIDs without starting optimization"
        ),
    )
    prepare_joint.add_argument(
        "--predecessor",
        type=_root,
        default=default_grounding_predecessor_root(),
        help="immutable completed #1017 Hugging Face export",
    )
    train_joint = subcommands.add_parser(
        "train-joint-candidate-margin-preflight",
        help=(
            "run the sole <=10 minute C1-SB4 complete-record structured-margin "
            "preflight; never opens product text"
        ),
    )
    train_joint.add_argument(
        "--predecessor",
        type=_root,
        default=default_grounding_predecessor_root(),
        help="immutable completed #1017 Hugging Face export",
    )
    prepare_paired = subcommands.add_parser(
        "prepare-paired-query-binding",
        help=(
            "commit the C1-SB5 paired-query population and sealed product CIDs "
            "without starting optimization"
        ),
    )
    prepare_paired.add_argument(
        "--predecessor",
        type=_root,
        default=default_grounding_predecessor_root(),
        help="immutable completed #1017 Hugging Face export",
    )
    train_paired = subcommands.add_parser(
        "train-paired-query-binding-preflight",
        help=(
            "run the sole C1-SB5 optimizer under its 300-second ceiling, then "
            "the mandatory controls; never opens product text"
        ),
    )
    train_paired.add_argument(
        "--predecessor",
        type=_root,
        default=default_grounding_predecessor_root(),
        help="immutable completed #1017 Hugging Face export",
    )
    prepare_retention = subcommands.add_parser(
        "prepare-group-retention",
        help=(
            "freeze #973's source-free geometry and physically sealed "
            "256/64 natural-language population without training"
        ),
    )
    prepare_retention.add_argument(
        "--source-root",
        type=_root,
        default=default_group_retention_source_root(),
        help="immutable completed #1017 research root; weights and traces are forbidden",
    )
    prepare_retention.add_argument(
        "--geometry",
        type=_root,
        required=True,
        help="canonical JSON emitted by r4-group-geometry-export",
    )
    retention_preflight = subcommands.add_parser(
        "preflight-group-retention",
        help=(
            "run #973's sole structural, MPS timing, gradient, and disposable "
            "overfit gate; held-out bytes remain sealed"
        ),
    )
    retention_preflight.add_argument("--backend", choices=["mps"], required=True)
    prepare_retention_decoder = subcommands.add_parser(
        "prepare-group-retention-decoder",
        help=(
            "freeze #973's independent fit-only fuller-decoder construction "
            "population and inherited geometry without training"
        ),
    )
    prepare_retention_decoder.add_argument(
        "--predecessor",
        type=_root,
        required=True,
        help="immutable completed issue-973-group-retention root",
    )
    retention_decoder_preflight = subcommands.add_parser(
        "preflight-group-retention-decoder",
        help=(
            "run #973's sole two-arm fuller-decoder construction terminal; "
            "no model-held-out or main command exists"
        ),
    )
    retention_decoder_preflight.add_argument(
        "--backend", choices=["mps"], required=True
    )
    prepare_retention_decoder_cpu = subcommands.add_parser(
        "prepare-group-retention-decoder-cpu-recovery",
        help=(
            "freeze #973's resource-only Apple CPU recovery in a distinct "
            "create-once root without touching the terminal MPS attempt"
        ),
    )
    prepare_retention_decoder_cpu.add_argument(
        "--predecessor",
        type=_root,
        required=True,
        help="immutable completed issue-973-group-retention root",
    )
    retention_decoder_cpu_preflight = subcommands.add_parser(
        "preflight-group-retention-decoder-cpu-recovery",
        help=(
            "run #973's single-process four-thread Apple Accelerate recovery; "
            "timing is telemetry and no reveal or main command exists"
        ),
    )
    retention_decoder_cpu_preflight.add_argument(
        "--backend", choices=["cpu"], required=True
    )
    prepare_language_path = subcommands.add_parser(
        "prepare-language-path",
        help=(
            "freeze #973's compact matched language population and exact-H4 "
            "geometry without reading sealed data or training"
        ),
    )
    prepare_language_path.add_argument(
        "--source-root",
        type=_root,
        default=default_language_path_source_root(),
        help="immutable nonsealed #1019 training-view root",
    )
    prepare_language_path.add_argument(
        "--geometry",
        type=_root,
        default=default_language_path_geometry(),
        help="canonical exact-H4 group geometry inherited from #973",
    )
    subcommands.add_parser(
        "probe-language-path",
        help=(
            "measure the eligible deterministic Apple execution plans and "
            "bind the fastest admitted plan"
        ),
    )
    run_language_path = subcommands.add_parser(
        "run-language-path",
        help="run or resume #973's one frozen matched language-path experiment",
    )
    run_language_path.add_argument(
        "--resume",
        action="store_true",
        help="resume the same frozen trajectory from its latest checkpoints",
    )
    subcommands.add_parser(
        "generate-language-path",
        help=(
            "run or verify #973's create-once retained-only five-prompt "
            "autonomous generation smoke"
        ),
    )
    fit_contextual = subcommands.add_parser(
        "fit-contextual-retained",
        help=(
            "adapt the existing retained artifact through #973's contextual "
            "value write using open training bytes only"
        ),
    )
    fit_contextual.add_argument("--updates", type=int, default=128)
    fit_contextual.add_argument("--threads", type=int, choices=[4], default=4)
    fit_contextual.add_argument("--max-seconds", type=float, default=840.0)
    subcommands.add_parser(
        "fit-contextual-retained-full",
        help=(
            "fit one complete deterministic epoch through #973's contextual "
            "value write using open training bytes only"
        ),
    )
    subcommands.add_parser(
        "fit-contextual-key-value",
        help=(
            "run #973's fixed bounded fit with one strict-prior context "
            "supplying both retained key and value writes"
        ),
    )
    subcommands.add_parser(
        "fit-contextual-key-value-address-read",
        help=(
            "run #973's fixed bounded fit with relative exact-H4 address "
            "participating in the retained attention score"
        ),
    )
    generate_contextual = subcommands.add_parser(
        "generate-contextual-retained",
        help=(
            "continue one arbitrary prompt through #973's contextual retained "
            "value-write successor"
        ),
    )
    generate_contextual.add_argument("--prompt", required=True)
    generate_contextual.add_argument(
        "--artifact",
        type=_root,
        help="retained artifact to load; defaults to the historical V1 artifact",
    )
    generate_contextual.add_argument(
        "--geometry",
        type=_root,
        default=default_language_path_geometry(),
        help="canonical exact-H4 group geometry",
    )
    generate_contextual.add_argument("--max-new-tokens", type=int, default=32)
    generate_contextual.add_argument("--seed", type=int, default=9_738)
    generate_contextual.add_argument(
        "--json",
        action="store_true",
        help="emit the artifact identities, token IDs, and execution details",
    )
    generate_ordinary = subcommands.add_parser(
        "generate-ordinary-language-path",
        help=(
            "continue one prompt through #973's already-fitted ordinary "
            "causal-softmax control without fitting"
        ),
    )
    generate_ordinary.add_argument("--prompt", required=True)
    generate_ordinary.add_argument("--max-new-tokens", type=int, default=16)
    generate_ordinary.add_argument("--seed", type=int, default=9_738)
    generate_ordinary.add_argument(
        "--json",
        action="store_true",
        help="emit the artifact identities, token IDs, and execution details",
    )
    generate_position_r4 = subcommands.add_parser(
        "generate-position-r4-language-path",
        help=(
            "continue one prompt through #973's ordinary weights with the "
            "position-preserving R4 K/V path"
        ),
    )
    generate_position_r4.add_argument("--prompt", required=True)
    generate_position_r4.add_argument(
        "--geometry",
        type=_root,
        default=default_language_path_geometry(),
        help="canonical exact-H4 group geometry",
    )
    generate_position_r4.add_argument(
        "--h4-sidecar",
        type=_root,
        default=default_predictive_block_delta_frame_sidecar(),
        help="validated H4 frame sidecar used for key/value transport",
    )
    generate_position_r4.add_argument("--max-new-tokens", type=int, default=16)
    generate_position_r4.add_argument("--seed", type=int, default=9_738)
    generate_position_r4.add_argument(
        "--json",
        action="store_true",
        help="emit the artifact identities, token IDs, and execution details",
    )
    generate_fixed_recurrent = subcommands.add_parser(
        "generate-fixed-recurrent-r4-language-path",
        help=(
            "continue one prompt through #973's ordinary weights with the "
            "fixed-size recurrent R4/H4 K/V path"
        ),
    )
    generate_fixed_recurrent.add_argument("--prompt", required=True)
    generate_fixed_recurrent.add_argument(
        "--geometry",
        type=_root,
        default=default_language_path_geometry(),
        help="canonical exact-H4 group geometry",
    )
    generate_fixed_recurrent.add_argument(
        "--h4-sidecar",
        type=_root,
        default=default_predictive_block_delta_frame_sidecar(),
        help="validated H4 frame sidecar used for recurrent transport",
    )
    generate_fixed_recurrent.add_argument(
        "--max-new-tokens", type=int, default=16
    )
    generate_fixed_recurrent.add_argument("--seed", type=int, default=9_738)
    generate_fixed_recurrent.add_argument(
        "--json",
        action="store_true",
        help="emit artifact, recurrent-state, token, and execution details",
    )
    generate_sparse_geometric = subcommands.add_parser(
        "generate-sparse-geometric-r4-language-path",
        help=(
            "continue one prompt through #973's ordinary weights with bounded "
            "H4 candidate admission over fixed recurrent memory"
        ),
    )
    generate_sparse_geometric.add_argument("--prompt", required=True)
    generate_sparse_geometric.add_argument(
        "--geometry",
        type=_root,
        default=default_language_path_geometry(),
        help="canonical exact-H4 group geometry",
    )
    generate_sparse_geometric.add_argument(
        "--h4-sidecar",
        type=_root,
        default=default_predictive_block_delta_frame_sidecar(),
        help="validated H4 frame sidecar used for candidate admission and transport",
    )
    generate_sparse_geometric.add_argument(
        "--max-new-tokens", type=int, default=16
    )
    generate_sparse_geometric.add_argument("--seed", type=int, default=9_738)
    generate_sparse_geometric.add_argument(
        "--json",
        action="store_true",
        help="emit artifact, candidate-trace, token, and execution details",
    )
    prepare_paired_h4 = subcommands.add_parser(
        "prepare-paired-h4-prompt-capacity",
        help=(
            "freeze #973's one paired-H4 successor, fresh heldout slice, and "
            "independent prompt-conditioning population"
        ),
    )
    prepare_paired_h4.add_argument(
        "--predecessor-root",
        type=_root,
        default=default_paired_h4_prompt_capacity_predecessor(),
        help="immutable qualified retained-language-path root",
    )
    prepare_paired_h4.add_argument(
        "--source-train",
        type=_root,
        default=default_paired_h4_prompt_capacity_source_train(),
        help="verified nonsealed #1019 train-token store",
    )
    prepare_paired_h4.add_argument(
        "--raw-source",
        type=_root,
        default=default_paired_h4_prompt_capacity_raw_source(),
        help="pinned raw TinyStories source for the sealed prompt population",
    )
    subcommands.add_parser(
        "probe-paired-h4-prompt-capacity",
        help="run #973's five-step Apple Accelerate CPU4 admission probe",
    )
    run_paired_h4 = subcommands.add_parser(
        "run-paired-h4-prompt-capacity",
        help="run or resume #973's single paired-H4 fit and frozen evaluation",
    )
    run_paired_h4.add_argument(
        "--resume",
        action="store_true",
        help="resume the identical frozen candidate trajectory",
    )
    prepare_readout = subcommands.add_parser(
        "prepare-direct-retained-readout",
        help=(
            "freeze #973's readout-only successor, disjoint heldout slice, "
            "and independently sealed prompt-contrast V2 population"
        ),
    )
    prepare_readout.add_argument(
        "--predecessor-root",
        type=_root,
        default=default_direct_retained_readout_predecessor(),
        help="immutable qualified retained-language-path root",
    )
    prepare_readout.add_argument(
        "--source-train",
        type=_root,
        default=default_direct_retained_readout_source_train(),
        help="verified nonsealed #1019 train-token store",
    )
    prepare_readout.add_argument(
        "--source-train-index",
        type=_root,
        default=default_direct_retained_readout_source_train_index(),
        help="canonical #1019 train-story index binding the heldout slice",
    )
    prepare_readout.add_argument(
        "--raw-source",
        type=_root,
        default=default_direct_retained_readout_raw_source(),
        help="pinned raw TinyStories source for prompt-contrast V2",
    )
    subcommands.add_parser(
        "probe-direct-retained-readout",
        help="run #973's sole five-step Apple Accelerate CPU4 admission probe",
    )
    run_readout = subcommands.add_parser(
        "run-direct-retained-readout",
        help="run or resume #973's sole readout-only fit and frozen evaluation",
    )
    run_readout.add_argument(
        "--resume",
        action="store_true",
        help="resume the identical frozen candidate trajectory",
    )
    subcommands.add_parser(
        "verify-direct-retained-readout",
        help="fresh-process exact re-score of terminal prompt and heldout evidence",
    )
    prepare_layerwise_readout = subcommands.add_parser(
        "prepare-layerwise-normalized-readout",
        help=(
            "freeze #973's layerwise-normalized readout successor, disjoint "
            "heldout slice, and sealed prompt-contrast V3 population"
        ),
    )
    prepare_layerwise_readout.add_argument(
        "--predecessor-root",
        type=_root,
        default=default_layerwise_normalized_readout_predecessor(),
        help="immutable qualified retained-language-path root",
    )
    prepare_layerwise_readout.add_argument(
        "--source-train",
        type=_root,
        default=default_layerwise_normalized_readout_source_train(),
        help="verified nonsealed #1019 train-token store",
    )
    prepare_layerwise_readout.add_argument(
        "--source-train-index",
        type=_root,
        default=default_layerwise_normalized_readout_source_train_index(),
        help="canonical #1019 train-story index binding the heldout slice",
    )
    prepare_layerwise_readout.add_argument(
        "--raw-source",
        type=_root,
        default=default_layerwise_normalized_readout_raw_source(),
        help="pinned raw TinyStories source for prompt-contrast V3",
    )
    prepare_layerwise_readout.add_argument(
        "--v1-population",
        type=_root,
        default=default_layerwise_normalized_readout_v1_population(),
        help="revealed V1 prompt population used only for story-CID exclusion",
    )
    prepare_layerwise_readout.add_argument(
        "--v2-population",
        type=_root,
        default=default_layerwise_normalized_readout_v2_population(),
        help="revealed V2 prompt population used only for story-CID exclusion",
    )
    subcommands.add_parser(
        "probe-layerwise-normalized-readout",
        help="run #973's sole five-step Apple Accelerate CPU4 admission probe",
    )
    run_layerwise_readout = subcommands.add_parser(
        "run-layerwise-normalized-readout",
        help="run or resume #973's sole layerwise-readout fit and frozen evaluation",
    )
    run_layerwise_readout.add_argument(
        "--resume",
        action="store_true",
        help="resume the identical frozen candidate trajectory",
    )
    subcommands.add_parser(
        "verify-layerwise-normalized-readout",
        help="fresh-process exact re-score of terminal V3 prompt and heldout evidence",
    )
    prepare_learned_readout = subcommands.add_parser(
        "prepare-learned-associative-readout",
        help=(
            "seal #973's frozen V4 prompt and fresh-language populations for "
            "the learned geometric and pooled associative readouts"
        ),
    )
    prepare_learned_readout.add_argument(
        "--predecessor-root",
        type=_root,
        default=default_learned_associative_readout_predecessor(),
        help="immutable qualified retained-language-path root",
    )
    prepare_learned_readout.add_argument(
        "--source-train",
        type=_root,
        default=default_learned_associative_readout_source_train(),
        help="verified nonsealed #1019 train-token store",
    )
    prepare_learned_readout.add_argument(
        "--source-train-index",
        type=_root,
        default=default_learned_associative_readout_source_train_index(),
        help="canonical #1019 train-story index binding the heldout slice",
    )
    prepare_learned_readout.add_argument(
        "--raw-source",
        type=_root,
        default=default_learned_associative_readout_raw_source(),
        help="pinned raw TinyStories source used only for the sealed V4 population",
    )
    prepare_learned_readout.add_argument(
        "--v1-population",
        type=_root,
        default=default_learned_associative_readout_v1_population(),
        help="revealed V1 prompt population used only for story-CID exclusion",
    )
    prepare_learned_readout.add_argument(
        "--v2-population",
        type=_root,
        default=default_learned_associative_readout_v2_population(),
        help="revealed V2 prompt population used only for story-CID exclusion",
    )
    prepare_learned_readout.add_argument(
        "--v3-population",
        type=_root,
        default=default_learned_associative_readout_v3_population(),
        help="revealed V3 prompt population used only for story-CID exclusion",
    )
    subcommands.add_parser(
        "probe-learned-associative-readout",
        help="benchmark eligible local backends using training data only",
    )
    run_learned_readout = subcommands.add_parser(
        "run-learned-associative-readout",
        help="run or resume the one frozen two-head fit and one-time evaluation",
    )
    run_learned_readout.add_argument(
        "--resume",
        action="store_true",
        help="resume only the identical frozen pre-reveal trajectory",
    )
    subcommands.add_parser(
        "verify-learned-associative-readout",
        help="fresh-process exact re-score of terminal V4 and heldout evidence",
    )
    predictive_delta = subcommands.add_parser(
        "preflight-predictive-block-delta",
        help=(
            "run #973's sole disposable predictive block-delta expressivity gate "
            "against the already revealed V4 population"
        ),
    )
    predictive_delta.add_argument(
        "--predecessor-root",
        type=_root,
        default=default_predictive_block_delta_predecessor(),
        help="immutable qualified retained-language-path root",
    )
    predictive_delta.add_argument(
        "--revealed-v4-root",
        type=_root,
        default=default_predictive_block_delta_revealed_v4_root(),
        help="completed learned-associative root containing the revealed V4 population",
    )
    predictive_delta.add_argument(
        "--frame-sidecar",
        type=_root,
        default=default_predictive_block_delta_frame_sidecar(),
        help="canonical JSON emitted by r4-h4-spin-frame-export",
    )
    predictive_delta.add_argument(
        "--maximum-updates",
        type=_predictive_block_delta_updates,
        default=PREDICTIVE_BLOCK_DELTA_MAXIMUM_UPDATES,
        help=(
            "disposable optimizer ceiling, bounded to the frozen campaign maximum "
            f"of {PREDICTIVE_BLOCK_DELTA_MAXIMUM_UPDATES}"
        ),
    )
    predictive_delta_v2 = subcommands.add_parser(
        "preflight-predictive-block-delta-v2",
        help=(
            "run #973's frozen independent native/additive correction on "
            "revealed V4 pairs 32 through 63"
        ),
    )
    predictive_delta_v2.add_argument(
        "--predecessor-root",
        type=_root,
        default=default_predictive_block_delta_predecessor(),
        help="immutable qualified retained-language-path root",
    )
    predictive_delta_v2.add_argument(
        "--revealed-v4-root",
        type=_root,
        default=default_predictive_block_delta_revealed_v4_root(),
        help="completed learned-associative root containing the revealed V4 population",
    )
    predictive_delta_v2.add_argument(
        "--frame-sidecar",
        type=_root,
        default=default_predictive_block_delta_frame_sidecar(),
        help="canonical JSON emitted by r4-h4-spin-frame-export",
    )
    predictive_delta_v2.add_argument(
        "--v1-result",
        type=_root,
        default=default_predictive_block_delta_v1_result(),
        help="immutable create-once V1 result required by the V2 correction",
    )
    prepare_predictive_terminal = subcommands.add_parser(
        "prepare-predictive-block-delta-terminal",
        help=(
            "create and mode-000 seal #973's authorized V5 prompt and fresh-language "
            "populations without returning either payload"
        ),
    )
    prepare_predictive_terminal.add_argument(
        "--predecessor-root",
        type=_root,
        default=default_predictive_block_delta_predecessor(),
        help="immutable qualified retained-language-path root",
    )
    prepare_predictive_terminal.add_argument(
        "--source-train",
        type=_root,
        default=default_learned_associative_readout_source_train(),
        help="verified nonsealed #1019 train-token store",
    )
    prepare_predictive_terminal.add_argument(
        "--source-train-index",
        type=_root,
        default=default_learned_associative_readout_source_train_index(),
        help="canonical #1019 train-story index binding the V5 fresh slice",
    )
    prepare_predictive_terminal.add_argument(
        "--raw-source",
        type=_root,
        default=default_learned_associative_readout_raw_source(),
        help="pinned raw TinyStories source used only during create-once V5 selection",
    )
    for version, default in enumerate(
        default_predictive_block_delta_terminal_prior_populations(), start=1
    ):
        prepare_predictive_terminal.add_argument(
            f"--v{version}-population",
            type=_root,
            default=default,
            help=f"exact revealed V{version} prompt population used only for CID exclusion",
        )
    prepare_predictive_terminal.add_argument(
        "--frame-sidecar",
        type=_root,
        default=default_predictive_block_delta_frame_sidecar(),
        help="canonical Rust-exported H4 spin-frame sidecar",
    )
    prepare_predictive_terminal.add_argument(
        "--v2-result",
        type=_root,
        default=default_predictive_block_delta_terminal_v2_result(),
        help="exact admitted V2 authorization read before any V5 access",
    )
    prepare_predictive_terminal.add_argument(
        "--pooled-comparator-root",
        type=_root,
        default=default_learned_associative_readout_root(),
        help="completed V4 campaign holding the immutable pooled comparator",
    )
    subcommands.add_parser(
        "probe-predictive-block-delta-terminal",
        help="benchmark CPU4, CPU8, and two CPU4 workers using construction data only",
    )
    run_predictive_terminal = subcommands.add_parser(
        "run-predictive-block-delta-terminal",
        help="fit three frozen arms, reveal V5 once, and write the terminal decision",
    )
    run_predictive_terminal.add_argument(
        "--resume",
        action="store_true",
        help="resume only exact pre-reveal checkpoints under the frozen plan",
    )
    subcommands.add_parser(
        "verify-predictive-block-delta-terminal",
        help="independently reproduce V5 prompt, fresh-language, and decision evidence",
    )
    prepare_position_kv = subcommands.add_parser(
        "prepare-position-kv-binding",
        help=(
            "freeze #1043's complete story-disjoint construction and sealed "
            "terminal populations without fitting"
        ),
    )
    prepare_position_kv.add_argument(
        "--retained-language-root",
        type=_root,
        default=default_language_path_root(),
        help="immutable #973 ordinary/retained language-path root",
    )
    prepare_position_kv.add_argument(
        "--source-root",
        type=_root,
        default=default_language_path_source_root(),
        help="immutable #1019 token stores and canonical story indexes",
    )
    prepare_position_kv.add_argument(
        "--tokenizer",
        type=_root,
        default=default_language_path_root() / "tokenizer" / "tokenizer.json",
        help="exact inherited 4096-token tokenizer",
    )
    prepare_position_kv.add_argument(
        "--geometry",
        type=_root,
        default=default_language_path_geometry(),
        help="canonical exact-H4 group geometry",
    )
    prepare_position_kv.add_argument(
        "--h4-sidecar",
        type=_root,
        default=default_predictive_block_delta_frame_sidecar(),
        help="canonical Rust-exported H4 spin-frame sidecar",
    )
    prepare_position_kv.add_argument(
        "--v5-root",
        type=_root,
        default=default_predictive_block_delta_terminal_root(),
        help="immutable revealed V5 root used only to bind prior story exclusions",
    )
    subcommands.add_parser(
        "preflight-position-kv-binding",
        help="run #1043's oracle, mechanics, and Apple CPU 1/4/8 admission",
    )
    subcommands.add_parser(
        "run-position-kv-binding",
        help="execute #1043's one frozen fit, reveal, score, and terminal decision",
    )
    subcommands.add_parser(
        "verify-position-kv-binding",
        help="fresh-process validation of #1043's create-once terminal result",
    )
    prepare_role_tagged = subcommands.add_parser(
        "prepare-role-tagged-associative",
        help="bind #1045's open role-tagged ladder to #1043 construction only",
    )
    prepare_role_tagged.add_argument(
        "--source-root",
        type=_root,
        default=_default_position_kv_binding_root(),
        help="immutable #1043 root; only ordinary inputs and construction are read",
    )
    subcommands.add_parser(
        "preflight-role-tagged-associative",
        help="run #1045's role oracle, overfit, and Apple CPU plan probe",
    )
    subcommands.add_parser(
        "run-role-tagged-associative",
        help="run #1045's open MQAR rung until its first decision",
    )
    subcommands.add_parser(
        "verify-role-tagged-associative",
        help="validate #1045's open result and learned artifact",
    )
    return command


def main() -> None:
    arguments = parser().parse_args()
    continuation_commands = {
        "prepare-continuation",
        "continue",
        "admit-enabled-parity",
        "reveal-continuation",
        "verify-continuation-training",
        "finalize-continuation",
    }
    capacity_commands = {
        "prepare-capacity",
        "smoke-capacity",
        "admit-capacity-smoke",
        "probe-capacity",
        "train-capacity",
        "admit-capacity-parity",
        "reveal-capacity",
        "verify-capacity-training",
        "prepare-capacity-rubric",
        "verify-capacity-generation-ready",
        "finalize-capacity",
    }
    grounding_commands = {"finetune-grounding"}
    pointer_commands = {"train-source-span-pointer"}
    relation_commands = {"train-source-relation-head"}
    attended_relation_commands = {
        "prepare-attended-relation",
        "train-attended-relation-preflight",
    }
    joint_candidate_commands = {
        "prepare-joint-candidate-margin",
        "train-joint-candidate-margin-preflight",
    }
    paired_query_commands = {
        "prepare-paired-query-binding",
        "train-paired-query-binding-preflight",
    }
    group_retention_commands = {
        "prepare-group-retention",
        "preflight-group-retention",
    }
    group_retention_decoder_commands = {
        "prepare-group-retention-decoder",
        "preflight-group-retention-decoder",
    }
    group_retention_decoder_cpu_recovery_commands = {
        "prepare-group-retention-decoder-cpu-recovery",
        "preflight-group-retention-decoder-cpu-recovery",
    }
    language_path_commands = {
        "prepare-language-path",
        "probe-language-path",
        "run-language-path",
        "generate-language-path",
        "fit-contextual-retained",
        "fit-contextual-retained-full",
        "fit-contextual-key-value",
        "fit-contextual-key-value-address-read",
        "generate-contextual-retained",
        "generate-ordinary-language-path",
        "generate-position-r4-language-path",
        "generate-fixed-recurrent-r4-language-path",
        "generate-sparse-geometric-r4-language-path",
    }
    paired_h4_prompt_capacity_commands = {
        "prepare-paired-h4-prompt-capacity",
        "probe-paired-h4-prompt-capacity",
        "run-paired-h4-prompt-capacity",
    }
    direct_retained_readout_commands = {
        "prepare-direct-retained-readout",
        "probe-direct-retained-readout",
        "run-direct-retained-readout",
        "verify-direct-retained-readout",
    }
    layerwise_normalized_readout_commands = {
        "prepare-layerwise-normalized-readout",
        "probe-layerwise-normalized-readout",
        "run-layerwise-normalized-readout",
        "verify-layerwise-normalized-readout",
    }
    learned_associative_readout_commands = {
        "prepare-learned-associative-readout",
        "probe-learned-associative-readout",
        "run-learned-associative-readout",
        "verify-learned-associative-readout",
    }
    predictive_block_delta_commands = {"preflight-predictive-block-delta"}
    predictive_block_delta_v2_commands = {"preflight-predictive-block-delta-v2"}
    predictive_block_delta_terminal_commands = {
        "prepare-predictive-block-delta-terminal",
        "probe-predictive-block-delta-terminal",
        "run-predictive-block-delta-terminal",
        "verify-predictive-block-delta-terminal",
    }
    position_kv_binding_commands = {
        "prepare-position-kv-binding",
        "preflight-position-kv-binding",
        "run-position-kv-binding",
        "verify-position-kv-binding",
    }
    role_tagged_associative_commands = {
        "prepare-role-tagged-associative",
        "preflight-role-tagged-associative",
        "run-role-tagged-associative",
        "verify-role-tagged-associative",
    }
    if arguments.root:
        root = arguments.root
    elif arguments.command in capacity_commands:
        root = default_capacity_root()
    elif arguments.command in continuation_commands:
        root = default_continuation_root()
    elif arguments.command in grounding_commands:
        root = default_grounding_root()
    elif arguments.command in pointer_commands:
        root = default_source_span_pointer_root()
    elif arguments.command in relation_commands:
        root = default_source_relation_head_root()
    elif arguments.command in attended_relation_commands:
        root = default_attended_relation_adapter_root()
    elif arguments.command in joint_candidate_commands:
        root = default_joint_candidate_margin_root()
    elif arguments.command in paired_query_commands:
        root = default_paired_query_binding_root()
    elif arguments.command in group_retention_commands:
        root = default_group_retention_root()
    elif arguments.command in group_retention_decoder_commands:
        root = default_group_retention_decoder_root()
    elif arguments.command in group_retention_decoder_cpu_recovery_commands:
        root = default_group_retention_decoder_cpu_recovery_root()
    elif arguments.command in language_path_commands:
        root = default_language_path_root()
    elif arguments.command in paired_h4_prompt_capacity_commands:
        root = default_paired_h4_prompt_capacity_root()
    elif arguments.command in direct_retained_readout_commands:
        root = default_direct_retained_readout_root()
    elif arguments.command in layerwise_normalized_readout_commands:
        root = default_layerwise_normalized_readout_root()
    elif arguments.command in learned_associative_readout_commands:
        root = default_learned_associative_readout_root()
    elif arguments.command in predictive_block_delta_commands:
        root = default_predictive_block_delta_root()
    elif arguments.command in predictive_block_delta_v2_commands:
        root = default_predictive_block_delta_v2_root()
    elif arguments.command in predictive_block_delta_terminal_commands:
        root = default_predictive_block_delta_terminal_root()
    elif arguments.command in position_kv_binding_commands:
        root = _default_position_kv_binding_root()
    elif arguments.command in role_tagged_associative_commands:
        root = _default_role_tagged_associative_root()
    else:
        root = default_research_root()
    if arguments.command == "download":
        path = download_source(root)
        _print_result({"path": str(path), "bytes": path.stat().st_size, "status": "VERIFIED"})
        return
    if arguments.command == "prepare":
        _print_result(prepare_dataset(root, source=arguments.source, force=arguments.force))
        return
    if arguments.command == "smoke":
        _print_result(run_overfit_smoke(root, max_seconds=arguments.max_seconds))
        return
    if arguments.command == "admit":
        _print_result(admit_rust_smoke_qualification(root, arguments.rust_qualification))
        return
    if arguments.command == "train":
        _print_result(train_main(root, config=TrainConfig(), resume=arguments.resume))
        return
    if arguments.command == "reveal":
        _print_result(reveal_sealed_test(root))
        return
    if arguments.command == "verify":
        result: dict[str, Any] = {"dataset": load_dataset_manifest(root)}
        if arguments.export:
            result["export"] = verify_bound_manifest(
                root / "export" / "export-manifest.json",
                artifact_root=root / "export",
            )
        _print_result(result)
        return
    if arguments.command == "prepare-continuation":
        _print_result(
            prepare_continuation_dataset(
                root,
                predecessor_root=arguments.predecessor_root,
                source=arguments.source,
            )
        )
        return
    if arguments.command == "continue":
        _print_result(train_continuation(root, resume=arguments.resume))
        return
    if arguments.command == "admit-enabled-parity":
        _print_result(admit_enabled_prefix_parity(root, arguments.rust_qualification))
        return
    if arguments.command == "reveal-continuation":
        _print_result(reveal_continuation(root))
        return
    if arguments.command == "verify-continuation-training":
        _print_result(load_continuation_training_view_manifest(root))
        return
    if arguments.command == "finalize-continuation":
        _print_result(finalize_continuation(root, arguments.rubric))
        return
    if arguments.command == "prepare-capacity":
        _print_result(
            prepare_capacity_dataset(
                root,
                predecessor_root=arguments.predecessor_root,
                source=arguments.source,
                force=False,
            )
        )
        return
    if arguments.command == "smoke-capacity":
        _print_result(
            run_capacity_overfit_smoke(
                root,
                backend=arguments.backend,
                max_seconds=arguments.max_seconds,
            )
        )
        return
    if arguments.command == "admit-capacity-smoke":
        _print_result(admit_capacity_smoke(root, arguments.rust_qualification))
        return
    if arguments.command == "probe-capacity":
        _print_result(run_capacity_hardware_probe(root, backend=arguments.backend))
        return
    if arguments.command == "train-capacity":
        _print_result(
            train_capacity(
                root,
                backend=arguments.backend,
                resume=arguments.resume,
            )
        )
        return
    if arguments.command == "admit-capacity-parity":
        _print_result(admit_capacity_prefix_parity(root, arguments.rust_qualification))
        return
    if arguments.command == "reveal-capacity":
        _print_result(
            reveal_capacity(root, baseline_1017_root=arguments.baseline_1017_root)
        )
        return
    if arguments.command == "verify-capacity-training":
        _print_result(load_capacity_training_view_manifest(root))
        return
    if arguments.command == "prepare-capacity-rubric":
        _print_result(write_capacity_rubric_template(root, arguments.output))
        return
    if arguments.command == "verify-capacity-generation-ready":
        _print_result(verify_capacity_generation_ready(root))
        return
    if arguments.command == "finalize-capacity":
        _print_result(finalize_capacity(root, arguments.rubric))
        return
    if arguments.command == "finetune-grounding":
        _print_result(
            train_grounding(
                root,
                predecessor=arguments.predecessor,
                resume=arguments.resume,
            )
        )
        return
    if arguments.command == "train-source-span-pointer":
        _print_result(
            train_source_span_pointer(
                root,
                predecessor=arguments.predecessor,
                rust_score_parity=arguments.rust_score_parity,
            )
        )
        return
    if arguments.command == "train-source-relation-head":
        _print_result(
            train_source_relation_head(
                root,
                predecessor=arguments.predecessor,
                rust_score_parity=arguments.rust_score_parity,
            )
        )
        return
    if arguments.command == "prepare-attended-relation":
        _print_result(
            prepare_attended_relation_data(root, predecessor=arguments.predecessor)
        )
        return
    if arguments.command == "train-attended-relation-preflight":
        _print_result(
            run_attended_relation_preflight(root, predecessor=arguments.predecessor)
        )
        return
    if arguments.command == "prepare-joint-candidate-margin":
        _print_result(
            prepare_joint_candidate_margin_data(
                root, predecessor=arguments.predecessor
            )
        )
        return
    if arguments.command == "train-joint-candidate-margin-preflight":
        _print_result(
            run_joint_candidate_margin_preflight(
                root, predecessor=arguments.predecessor
            )
        )
        return
    if arguments.command == "prepare-paired-query-binding":
        _print_result(
            prepare_paired_query_binding_data(
                root, predecessor=arguments.predecessor
            )
        )
        return
    if arguments.command == "train-paired-query-binding-preflight":
        _print_result(
            run_paired_query_binding_preflight(
                root, predecessor=arguments.predecessor
            )
        )
        return
    if arguments.command == "prepare-group-retention":
        _print_result(
            prepare_group_retention_data(
                root,
                source_root=arguments.source_root,
                geometry_path=arguments.geometry,
            )
        )
        return
    if arguments.command == "preflight-group-retention":
        _print_result(run_group_retention_preflight(root, backend=arguments.backend))
        return
    if arguments.command == "prepare-group-retention-decoder":
        _print_result(
            prepare_group_retention_decoder_data(
                root,
                predecessor=arguments.predecessor,
            )
        )
        return
    if arguments.command == "preflight-group-retention-decoder":
        _print_result(
            run_group_retention_decoder_preflight(root, backend=arguments.backend)
        )
        return
    if arguments.command == "prepare-group-retention-decoder-cpu-recovery":
        _print_result(
            prepare_group_retention_decoder_cpu_recovery_data(
                root,
                predecessor=arguments.predecessor,
            )
        )
        return
    if arguments.command == "preflight-group-retention-decoder-cpu-recovery":
        _print_result(
            run_group_retention_decoder_cpu_recovery_preflight(
                root, backend=arguments.backend
            )
        )
        return
    if arguments.command == "prepare-language-path":
        prepared = prepare_language_path_data(
            source_root=arguments.source_root,
            output_root=root,
            geometry_path=arguments.geometry,
        )
        _print_result(prepared.manifest)
        return
    if arguments.command == "probe-language-path":
        _print_result(probe_language_path_execution(root))
        return
    if arguments.command == "run-language-path":
        _print_result(
            run_language_path_generalization(root, resume=arguments.resume)
        )
        return
    if arguments.command == "generate-language-path":
        _print_result(run_language_path_generation(root))
        return
    if arguments.command == "fit-contextual-retained":
        _print_result(
            fit_contextual_retained(
                root,
                updates=arguments.updates,
                threads=arguments.threads,
                max_seconds=arguments.max_seconds,
            )
        )
        return
    if arguments.command == "fit-contextual-retained-full":
        _print_result(fit_contextual_retained_full(root))
        return
    if arguments.command == "fit-contextual-key-value":
        _print_result(fit_contextual_key_value(root))
        return
    if arguments.command == "fit-contextual-key-value-address-read":
        _print_result(fit_contextual_key_value_address_read(root))
        return
    if arguments.command == "generate-contextual-retained":
        result = generate_contextual_retained(
            root,
            geometry_path=arguments.geometry,
            prompt=arguments.prompt,
            artifact_path=arguments.artifact,
            max_new_tokens=arguments.max_new_tokens,
            seed=arguments.seed,
        )
        if arguments.json:
            _print_result(result)
        else:
            print(result["text"])
        return
    if arguments.command == "generate-ordinary-language-path":
        result = generate_ordinary_language_path(
            root,
            prompt=arguments.prompt,
            max_new_tokens=arguments.max_new_tokens,
            seed=arguments.seed,
        )
        if arguments.json:
            _print_result(result)
        else:
            print(result["text"])
        return
    if arguments.command == "generate-position-r4-language-path":
        result = generate_position_r4_language_path(
            root,
            geometry_path=arguments.geometry,
            frame_path=arguments.h4_sidecar,
            prompt=arguments.prompt,
            max_new_tokens=arguments.max_new_tokens,
            seed=arguments.seed,
        )
        if arguments.json:
            _print_result(result)
        else:
            print(result["text"])
        return
    if arguments.command == "generate-fixed-recurrent-r4-language-path":
        result = generate_fixed_recurrent_r4_language_path(
            root,
            geometry_path=arguments.geometry,
            frame_path=arguments.h4_sidecar,
            prompt=arguments.prompt,
            max_new_tokens=arguments.max_new_tokens,
            seed=arguments.seed,
        )
        if arguments.json:
            _print_result(result)
        else:
            print(result["text"])
        return
    if arguments.command == "generate-sparse-geometric-r4-language-path":
        result = generate_sparse_geometric_r4_language_path(
            root,
            geometry_path=arguments.geometry,
            frame_path=arguments.h4_sidecar,
            prompt=arguments.prompt,
            max_new_tokens=arguments.max_new_tokens,
            seed=arguments.seed,
        )
        if arguments.json:
            _print_result(result)
        else:
            print(result["text"])
        return
    if arguments.command == "prepare-paired-h4-prompt-capacity":
        prepared = prepare_paired_h4_prompt_capacity(
            root=root,
            predecessor_root=arguments.predecessor_root,
            source_train_path=arguments.source_train,
            raw_source_path=arguments.raw_source,
        )
        _print_result(prepared.manifest)
        return
    if arguments.command == "probe-paired-h4-prompt-capacity":
        _print_result(probe_paired_h4_prompt_capacity(root))
        return
    if arguments.command == "run-paired-h4-prompt-capacity":
        _print_result(
            run_paired_h4_prompt_capacity(root, resume=arguments.resume)
        )
        return
    if arguments.command == "prepare-direct-retained-readout":
        prepared = prepare_direct_retained_readout(
            root=root,
            predecessor_root=arguments.predecessor_root,
            source_train_path=arguments.source_train,
            source_train_index_path=arguments.source_train_index,
            raw_source_path=arguments.raw_source,
        )
        _print_result(prepared.manifest)
        return
    if arguments.command == "probe-direct-retained-readout":
        _print_result(probe_direct_retained_readout(root))
        return
    if arguments.command == "run-direct-retained-readout":
        _print_result(run_direct_retained_readout(root, resume=arguments.resume))
        return
    if arguments.command == "verify-direct-retained-readout":
        _print_result(verify_direct_retained_readout_result(root))
        return
    if arguments.command == "prepare-layerwise-normalized-readout":
        prepared = prepare_layerwise_normalized_retained_readout(
            root=root,
            predecessor_root=arguments.predecessor_root,
            source_train_path=arguments.source_train,
            source_train_index_path=arguments.source_train_index,
            raw_source_path=arguments.raw_source,
            prior_v1_prompt_population_path=arguments.v1_population,
            prior_v2_prompt_population_path=arguments.v2_population,
        )
        _print_result(prepared.manifest)
        return
    if arguments.command == "probe-layerwise-normalized-readout":
        _print_result(probe_layerwise_normalized_retained_readout(root))
        return
    if arguments.command == "run-layerwise-normalized-readout":
        _print_result(
            run_layerwise_normalized_retained_readout(
                root,
                resume=arguments.resume,
            )
        )
        return
    if arguments.command == "verify-layerwise-normalized-readout":
        _print_result(verify_layerwise_normalized_retained_readout_result(root))
        return
    if arguments.command == "prepare-learned-associative-readout":
        prepared = prepare_learned_associative_readout(
            root=root,
            predecessor_root=arguments.predecessor_root,
            source_train_path=arguments.source_train,
            source_train_index_path=arguments.source_train_index,
            raw_source_path=arguments.raw_source,
            prior_v1_prompt_population_path=arguments.v1_population,
            prior_v2_prompt_population_path=arguments.v2_population,
            prior_v3_prompt_population_path=arguments.v3_population,
        )
        _print_result(prepared.manifest)
        return
    if arguments.command == "probe-learned-associative-readout":
        _print_result(probe_learned_associative_readout(root))
        return
    if arguments.command == "run-learned-associative-readout":
        _print_result(
            run_learned_associative_readout(root, resume=arguments.resume)
        )
        return
    if arguments.command == "verify-learned-associative-readout":
        _print_result(verify_learned_associative_readout_result(root))
        return
    if arguments.command == "preflight-predictive-block-delta":
        _print_result(
            run_predictive_block_delta_preflight(
                root=root,
                predecessor_root=arguments.predecessor_root,
                revealed_v4_root=arguments.revealed_v4_root,
                frame_sidecar_path=arguments.frame_sidecar,
                maximum_updates=arguments.maximum_updates,
            )
        )
        return
    if arguments.command == "preflight-predictive-block-delta-v2":
        _print_result(
            run_predictive_block_delta_v2_preflight(
                root=root,
                predecessor_root=arguments.predecessor_root,
                revealed_v4_root=arguments.revealed_v4_root,
                frame_sidecar_path=arguments.frame_sidecar,
                v1_result_path=arguments.v1_result,
            )
        )
        return
    if arguments.command == "prepare-predictive-block-delta-terminal":
        prepared = prepare_predictive_block_delta_terminal(
            root=root,
            predecessor_root=arguments.predecessor_root,
            source_train_path=arguments.source_train,
            source_train_index_path=arguments.source_train_index,
            raw_source_path=arguments.raw_source,
            prior_population_paths=(
                arguments.v1_population,
                arguments.v2_population,
                arguments.v3_population,
                arguments.v4_population,
            ),
            frame_sidecar_path=arguments.frame_sidecar,
            v2_result_path=arguments.v2_result,
            pooled_comparator_root=arguments.pooled_comparator_root,
        )
        _print_result(prepared.manifest)
        return
    if arguments.command == "probe-predictive-block-delta-terminal":
        _print_result(probe_predictive_block_delta_terminal(root))
        return
    if arguments.command == "run-predictive-block-delta-terminal":
        _print_result(
            run_predictive_block_delta_terminal(root, resume=arguments.resume)
        )
        return
    if arguments.command == "verify-predictive-block-delta-terminal":
        _print_result(verify_predictive_block_delta_terminal(root))
        return
    if arguments.command == "prepare-position-kv-binding":
        exclusions = collect_position_kv_story_exclusions(
            source_root=arguments.source_root,
            v5_root=arguments.v5_root,
        )
        _print_result(
            prepare_position_kv_binding_campaign(
                root,
                retained_language_root=arguments.retained_language_root,
                source_root=arguments.source_root,
                tokenizer_path=arguments.tokenizer,
                geometry_path=arguments.geometry,
                h4_sidecar_path=arguments.h4_sidecar,
                excluded_story_cids=exclusions,
            )
        )
        return
    if arguments.command == "preflight-position-kv-binding":
        _print_result(preflight_position_kv_binding_campaign(root))
        return
    if arguments.command == "run-position-kv-binding":
        _print_result(run_position_kv_binding_campaign(root))
        return
    if arguments.command == "verify-position-kv-binding":
        _print_result(validate_position_kv_binding_result(root))
        return
    if arguments.command == "prepare-role-tagged-associative":
        _print_result(
            prepare_role_tagged_associative_development(
                root,
                source_root=arguments.source_root,
            )
        )
        return
    if arguments.command == "preflight-role-tagged-associative":
        _print_result(preflight_role_tagged_associative_development(root))
        return
    if arguments.command == "run-role-tagged-associative":
        _print_result(run_role_tagged_associative_development(root))
        return
    if arguments.command == "verify-role-tagged-associative":
        _print_result(verify_role_tagged_associative_development(root))
        return
    raise AssertionError(f"unhandled command: {arguments.command}")
