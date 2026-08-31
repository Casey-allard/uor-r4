"""Command line for the bounded #1014, #1017, #1019, and #954 campaigns."""

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
from .data import download_source, load_dataset_manifest, prepare_dataset
from .finalize import finalize_continuation
from .grounding import train_grounding
from .paths import (
    default_capacity_root,
    default_continuation_root,
    default_grounding_predecessor_root,
    default_grounding_root,
    default_research_root,
)
from .provenance import verify_bound_manifest
from .train import TrainConfig, reveal_sealed_test, run_overfit_smoke, train_main


def _root(value: str) -> Path:
    return Path(value).expanduser().resolve()


def _print_result(value: dict[str, Any]) -> None:
    print(json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2))


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
            "or issue-1019 according to the selected lifecycle)"
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
    if arguments.root:
        root = arguments.root
    elif arguments.command in capacity_commands:
        root = default_capacity_root()
    elif arguments.command in continuation_commands:
        root = default_continuation_root()
    elif arguments.command in grounding_commands:
        root = default_grounding_root()
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
    raise AssertionError(f"unhandled command: {arguments.command}")
