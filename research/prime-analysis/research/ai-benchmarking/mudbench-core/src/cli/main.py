"""MUDBench CLI entrypoint with deterministic benchmark run wiring."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import sys
from dataclasses import asdict, is_dataclass
from pathlib import Path
from typing import Any, Mapping
from typing import Sequence

from agents.direct_provider_runner import build_direct_provider_command, resolve_direct_provider_config
from human_console_client import run_human_console_session, run_human_shared_shard_session
from evaluation.benchmark_runner.runner import (
    BenchmarkRunnerConfig,
    build_playable_slice_comparison_entry,
    build_tiny_suite_baseline_report,
    build_tiny_suite_comparison_report,
    build_tiny_suite_external_comparison_report,
    build_tiny_suite_external_profile_comparison_report,
    build_tiny_suite_mixed_external_profile_comparison_report,
    build_tiny_suite_mixed_external_comparison_report,
    resolve_timing_mode_cadence_config,
    run_benchmark_lifecycle,
)
from world.state.world_persistence import (
    WORLD_SAVE_DIR_DEFAULT,
    list_world_slots,
    validate_slot_name,
)

_DEFAULT_RUN_ID = "cli-run"
_DEFAULT_BENCHMARK_ID = "mudbench-cli"
_DEFAULT_ACTOR_IDS = ("agent-a", "agent-b")
_DEFAULT_SUITE_ID = "tiny"
_BUILTIN_COMPARISON_AGENT_IDS = ("agent-a", "agent-b")
_EXTERNAL_COMPARISON_AGENT_ID = "external-local-agent"
_DIRECT_PROVIDER_PROMPT_ENGINES = (
    "baseline",
    "geometric-routed",
    "angular-canonical",
    "legacy-router-backed",
)
_ANGULAR_ROUTER_VARIANTS = (
    "angular-hopf-base",
    "angular-hopf-trans",
)
_LEGACY_ROUTER_VARIANTS = (
    "legacy-phase4d_hopf_base",
    "legacy-phase4d_hopf_transport",
    "legacy-phase4d_hopf_product_phase",
)
_TIMING_MODE_CHOICES = (
    "off",
    "equal-cadence",
    "human-parity",
    "native-speed",
)
_PLAYABLE_SLICE_SCENARIOS = (
    "tiny-guarded-relic",
    "tiny-hazard-route",
    "tiny-delayed-cost",
    "tiny-context-pressure",
)
_SUITE_REPLAY_DRILLDOWN_SCHEMA_VERSION = "suite_replay_drilldown_v1"
_TINY_SUITE_SCENARIOS = (
    "tiny-delayed-retrieval",
    "tiny-fetch-quest",
    "tiny-hidden-key",
    "tiny-locked-path",
    "tiny-social-trade",
)

_CANONICAL_SCENARIO_DIR = Path(__file__).resolve().parent.parent.parent / "scenarios" / "canonical"


def _load_canonical_scenarios() -> dict[str, dict[str, Any]]:
    """Load all canonical scenario JSON files keyed by scenario_id.

    Called once at module-load time and merged into ``_SCENARIO_PRESETS``.
    Any ``*.json`` file under ``scenarios/canonical/`` with a valid
    ``scenario_id`` field is included automatically, eliminating the need to
    manually maintain inline copies of canonical scenarios in this file.
    """
    presets: dict[str, dict[str, Any]] = {}
    if not _CANONICAL_SCENARIO_DIR.is_dir():
        return presets
    for path in sorted(_CANONICAL_SCENARIO_DIR.glob("*.json")):
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError):
            continue
        scenario_id = payload.get("scenario_id")
        if isinstance(scenario_id, str) and scenario_id:
            presets[scenario_id] = payload
    return presets


_SCENARIO_PRESETS: dict[str, dict[str, Any]] = {
    "minimal": {
        "scenario_id": "cli-minimal-scenario",
        "title": "CLI Minimal Scenario",
        "description": "Deterministic minimal scenario for mudbench run.",
        "start_room_id": "room-start",
        "max_steps": 3,
        "seed": 7,
        "version": "1.0",
        "scenario_vars": {"mode": "cli-minimal"},
        "objectives": [
            {
                "objective_id": "obj-a",
                "objective_type": "collect_item",
                "target_id": "item-key",
                "required_count": 1,
            }
        ],
    },
    "phase4-runtime-replay": {
        "scenario_id": "phase4-runtime-replay-scenario",
        "title": "Phase 4 Runtime Replay Wiring Scenario",
        "description": "Scenario preset that matches runtime replay gate expectations.",
        "start_room_id": "room-start",
        "max_steps": 3,
        "seed": 51,
        "version": "1.0",
        "scenario_vars": {"mode": "runtime-replay"},
        "objectives": [
            {
                "objective_id": "obj-a",
                "objective_type": "collect_item",
                "target_id": "item-key",
                "required_count": 1,
            }
        ],
    },
    # Canonical tiny-* scenarios auto-loaded from scenarios/canonical/*.json
    **_load_canonical_scenarios(),
}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="mudbench")
    subcommands = parser.add_subparsers(dest="command", required=True)

    run_parser = subcommands.add_parser("run", help="Execute deterministic benchmark runtime path")
    run_parser.add_argument(
        "--scenario",
        choices=tuple(sorted(_SCENARIO_PRESETS.keys())),
        default="minimal",
        help="Built-in scenario preset to execute",
    )
    run_parser.add_argument(
        "--scenario-file",
        default=None,
        help="Path to a scenario definition JSON file on disk",
    )
    run_parser.add_argument("--run-id", default=_DEFAULT_RUN_ID, help="Run identifier")
    run_parser.add_argument("--benchmark-id", default=_DEFAULT_BENCHMARK_ID, help="Benchmark identifier")
    run_parser.add_argument("--run-seed", type=int, default=None, help="Optional run seed override")
    run_parser.add_argument("--max-steps", type=int, default=None, help="Optional max step override")
    run_parser.add_argument(
        "--actor-id",
        action="append",
        default=[],
        help="Actor identifier (repeatable). Defaults to built-in deterministic actor set.",
    )
    run_parser.add_argument(
        "--output",
        choices=("json", "pretty"),
        default="json",
        help="CLI output format",
    )
    run_parser.add_argument(
        "--agent-command",
        default=None,
        help="Single-shot local-process agent command string; MUDBench sends one observation on stdin and expects one action JSON on stdout",
    )
    run_parser.add_argument(
        "--agent-profile",
        default=None,
        help="Path to an external local-process agent profile JSON file",
    )
    run_parser.add_argument(
        "--agent-label",
        default=None,
        help="Optional label to use for an external local-process agent in CLI output; requires --agent-command",
    )
    run_parser.add_argument(
        "--persistent-agent-session",
        action="store_true",
        help="Keep the external local-process agent command alive across turns using the same stdin/stdout JSON contract; requires --agent-command",
    )
    run_parser.add_argument(
        "--direct-provider",
        choices=("openai-chat-completions",),
        default=None,
        help="Execute one bounded direct-provider benchmark path using the built-in canonical LLM runtime helpers.",
    )
    run_parser.add_argument(
        "--direct-provider-model",
        default=None,
        help="Direct-provider model identifier. Falls back to MUDBENCH_OPENAI_MODEL for the supported provider path.",
    )
    run_parser.add_argument(
        "--direct-provider-timeout-seconds",
        type=float,
        default=None,
        help="Optional external-agent timeout override for live direct-provider run turns only.",
    )
    run_parser.add_argument(
        "--provider-min-turn-delay-seconds",
        type=float,
        default=None,
        help="Optional minimum wall-clock delay (in seconds) between provider-backed actor turns.",
    )
    run_parser.add_argument(
        "--provider-max-actions",
        type=int,
        default=None,
        help="Optional cap on total accepted actions from provider-backed actors; run finalizes after reaching the limit.",
    )
    run_parser.add_argument(
        "--world-save-path",
        default=None,
        help="Optional file path to save a world snapshot after the run completes.",
    )
    run_parser.add_argument(
        "--world-load-path",
        default=None,
        help="Optional file path to load a world snapshot before the run begins.",
    )
    run_parser.add_argument(
        "--world-save-slot",
        default=None,
        help="Optional named save slot to store world state after the run (stored in --save-dir).",
    )
    run_parser.add_argument(
        "--world-load-slot",
        default=None,
        help="Optional named save slot to load world state from before the run (from --save-dir).",
    )
    run_parser.add_argument(
        "--save-dir",
        default=None,
        help=f"Directory for named save slots (default: {WORLD_SAVE_DIR_DEFAULT}).",
    )
    run_parser.add_argument(
        "--direct-provider-prompt-dump-dir",
        default=None,
        help="Optional directory for writing direct-provider prompt/raw-response dump sidecars.",
    )
    run_parser.add_argument(
        "--prompt-engine",
        choices=_DIRECT_PROVIDER_PROMPT_ENGINES,
        default="baseline",
        help="Prompt assembly mode for the optional direct-provider benchmark path.",
    )
    run_parser.add_argument(
        "--router-variant",
        choices=_ANGULAR_ROUTER_VARIANTS + _LEGACY_ROUTER_VARIANTS,
        default=None,
        help="Router variant to use when a variant-aware prompt engine is selected.",
    )
    run_parser.add_argument(
        "--timing-mode",
        choices=_TIMING_MODE_CHOICES,
        default=None,
        help="Optional explicit benchmark timing regime that resolves into cadence settings.",
    )
    run_parser.add_argument(
        "--action-cadence-interval",
        type=int,
        default=None,
        help="Optional positive world-tick cadence interval for benchmark actor actions.",
    )
    run_parser.add_argument(
        "--actor-action-cadence",
        action="append",
        default=[],
        help="Optional per-actor cadence override in actor_id=interval form; requires --action-cadence-interval.",
    )

    play_parser = subcommands.add_parser("play", help="Launch a minimal local human-play console client")
    play_parser.add_argument(
        "--scenario",
        choices=tuple(sorted(_SCENARIO_PRESETS.keys())),
        default="minimal",
        help="Built-in scenario preset to execute",
    )
    play_parser.add_argument(
        "--scenario-file",
        default=None,
        help="Path to a scenario definition JSON file on disk",
    )
    play_parser.add_argument("--run-id", default="human-console", help="Run identifier")
    play_parser.add_argument("--run-seed", type=int, default=None, help="Optional run seed override")
    play_parser.add_argument("--max-steps", type=int, default=None, help="Optional max step override")
    play_parser.add_argument(
        "--actor-id",
        default="human-player",
        help="Human-controlled actor identifier",
    )

    shared_play_parser = subcommands.add_parser(
        "play-shared-shard",
        help="Launch a minimal local shared-shard console loop for multiple participants",
    )
    shared_play_parser.add_argument(
        "--scenario",
        choices=tuple(sorted(_SCENARIO_PRESETS.keys())),
        default="tiny-fetch-quest",
        help="Built-in scenario preset to execute",
    )
    shared_play_parser.add_argument(
        "--scenario-file",
        default=None,
        help="Path to a scenario definition JSON file on disk",
    )
    shared_play_parser.add_argument("--run-id", default="human-shared-shard", help="Run identifier")
    shared_play_parser.add_argument("--shard-id", default="shared-shard-local", help="Shard identifier")
    shared_play_parser.add_argument("--run-seed", type=int, default=None, help="Optional run seed override")
    shared_play_parser.add_argument("--max-steps", type=int, default=None, help="Optional max step override")
    shared_play_parser.add_argument(
        "--actor-id",
        action="append",
        default=[],
        help="Participant actor identifier (repeatable). Defaults to two local participants.",
    )
    shared_play_parser.add_argument(
        "--mock-agent-actor-id",
        action="append",
        default=[],
        help="Participant actor identifier to run through the local mock LLM path (repeatable).",
    )
    shared_play_parser.add_argument(
        "--agent-command",
        default=None,
        help="Local-process agent command string for one external shared participant.",
    )
    shared_play_parser.add_argument(
        "--external-agent-actor-id",
        default=None,
        help="Participant actor identifier to run through the external local-process wrapper path.",
    )
    shared_play_parser.add_argument(
        "--persistent-agent-session",
        action="store_true",
        help="Keep the external shared participant alive across turns using the same stdin/stdout JSON contract; requires --agent-command.",
    )
    shared_play_parser.add_argument(
        "--direct-provider",
        choices=("openai-chat-completions",),
        default=None,
        help="Execute one bounded direct-provider participant path in shared shard play.",
    )
    shared_play_parser.add_argument(
        "--direct-provider-model",
        default=None,
        help="Direct-provider model identifier. Falls back to MUDBENCH_OPENAI_MODEL for the supported shared path.",
    )
    shared_play_parser.add_argument(
        "--timing-mode",
        choices=_TIMING_MODE_CHOICES,
        default=None,
        help="Optional explicit benchmark/session timing regime that resolves into shared cadence settings.",
    )
    shared_play_parser.add_argument(
        "--action-cadence-interval",
        type=int,
        default=None,
        help="Optional positive world-tick cadence interval applied to shared-shard actor actions.",
    )
    shared_play_parser.add_argument(
        "--actor-action-cadence",
        action="append",
        default=[],
        help="Optional per-actor cadence override in actor_id=interval form; requires --action-cadence-interval.",
    )
    shared_play_parser.add_argument(
        "--shared-external-agent-timeout-seconds",
        type=float,
        default=None,
        help="Optional positive timeout override for shared external-agent or direct-provider turns.",
    )
    shared_play_parser.add_argument(
        "--world-save-path",
        default=None,
        help="Optional file path to save a world snapshot when the shared-shard session ends.",
    )
    shared_play_parser.add_argument(
        "--world-load-path",
        default=None,
        help="Optional file path to load a world snapshot before the shared-shard session begins.",
    )
    shared_play_parser.add_argument(
        "--world-save-slot",
        default=None,
        help="Optional named save slot to store world state when the session ends (in --save-dir).",
    )
    shared_play_parser.add_argument(
        "--world-load-slot",
        default=None,
        help="Optional named save slot to reconnect to an existing world state (from --save-dir).",
    )
    shared_play_parser.add_argument(
        "--save-dir",
        default=None,
        help=f"Directory for named save slots (default: {WORLD_SAVE_DIR_DEFAULT}).",
    )

    list_saves_parser = subcommands.add_parser(
        "list-saves",
        help="List named world save slots in the save directory",
    )
    list_saves_parser.add_argument(
        "--save-dir",
        default=None,
        help=f"Directory containing named save slots (default: {WORLD_SAVE_DIR_DEFAULT}).",
    )
    list_saves_parser.add_argument(
        "--output",
        choices=("json", "pretty"),
        default="json",
        help="CLI output format",
    )

    compare_parser = subcommands.add_parser(
        "compare-playable-slices",
        help="Execute a compact comparison pass across the richer playable slices",
    )
    compare_parser.add_argument("--benchmark-id", default=_DEFAULT_BENCHMARK_ID, help="Benchmark identifier")
    compare_parser.add_argument(
        "--actor-id",
        default="agent-a",
        help="Single actor identifier used across comparison runs",
    )
    compare_parser.add_argument(
        "--output",
        choices=("json", "pretty"),
        default="json",
        help="CLI output format",
    )
    compare_parser.add_argument(
        "--direct-provider",
        choices=("openai-chat-completions",),
        default=None,
        help="Optionally include one bounded direct-provider comparison mode.",
    )
    compare_parser.add_argument(
        "--direct-provider-model",
        default=None,
        help="Direct-provider model identifier. Falls back to MUDBENCH_OPENAI_MODEL for the supported provider path.",
    )
    compare_parser.add_argument(
        "--include-routed-prompt-engine",
        action="store_true",
        help="Also compare the optional geometric-routed prompt engine for the direct-provider mode.",
    )
    compare_parser.add_argument(
        "--include-angular-canonical-prompt-engine",
        action="store_true",
        help="Also compare the optional canonical angular prompt engine for the direct-provider mode.",
    )
    compare_parser.add_argument(
        "--angular-router-variant",
        choices=_ANGULAR_ROUTER_VARIANTS,
        action="append",
        default=None,
        help=(
            "Canonical angular router variant to use when "
            "--include-angular-canonical-prompt-engine is selected. Repeat to compare multiple "
            "angular variants in one invocation."
        ),
    )
    compare_parser.add_argument(
        "--include-legacy-router-backed-prompt-engine",
        action="store_true",
        help="Also compare the explicit legacy router-backed proxy prompt engine for the direct-provider mode.",
    )
    compare_parser.add_argument(
        "--legacy-router-variant",
        choices=_LEGACY_ROUTER_VARIANTS,
        default=None,
        help="Legacy proxy router variant to use when --include-legacy-router-backed-prompt-engine is selected.",
    )
    compare_parser.add_argument(
        "--direct-provider-comparison-timeout-seconds",
        type=float,
        default=None,
        help="Optional external-agent timeout override for live direct-provider comparison turns only.",
    )
    compare_parser.add_argument(
        "--direct-provider-comparison-prompt-dump-dir",
        default=None,
        help="Optional directory for writing direct-provider comparison prompt/raw-response dump sidecars.",
    )
    compare_parser.add_argument(
        "--provider-min-turn-delay-seconds",
        type=float,
        default=None,
        help="Optional minimum wall-clock delay (in seconds) between provider-backed actor turns in each compare row.",
    )
    compare_parser.add_argument(
        "--provider-max-actions",
        type=int,
        default=None,
        help="Optional cap on total accepted provider-backed actor actions per compare row; each row finalizes after hitting the limit.",
    )

    suite_parser = subcommands.add_parser("suite", help="Execute deterministic tiny-suite baseline reporting")
    suite_parser.add_argument(
        "--suite",
        choices=("tiny",),
        default=_DEFAULT_SUITE_ID,
        help="Built-in suite preset to execute",
    )
    suite_parser.add_argument("--benchmark-id", default=_DEFAULT_BENCHMARK_ID, help="Benchmark identifier")
    suite_parser.add_argument(
        "--baseline-agent",
        default=None,
        help="Built-in actor profile to use as the baseline comparison side",
    )
    suite_parser.add_argument(
        "--candidate-agent",
        default=None,
        help="Built-in actor profile to use as the candidate comparison side",
    )
    suite_parser.add_argument(
        "--agent-command",
        default=None,
        help="External local-process agent command string for suite comparison candidate side",
    )
    suite_parser.add_argument(
        "--agent-profile",
        default=None,
        help="Path to an external local-process agent profile JSON file for suite comparison candidate side",
    )
    suite_parser.add_argument(
        "--baseline-agent-profile",
        default=None,
        help="Path to an external local-process agent profile JSON file for the suite comparison baseline side",
    )
    suite_parser.add_argument(
        "--candidate-agent-profile",
        default=None,
        help="Path to an external local-process agent profile JSON file for the suite comparison candidate side",
    )
    suite_parser.add_argument(
        "--agent-label",
        default=None,
        help="Optional label to use for the external local-process suite candidate; requires --agent-command",
    )
    suite_parser.add_argument(
        "--external-agent-actor",
        default=None,
        help="Built-in actor slot to replace with the external local-process candidate in shared-run suite comparison",
    )
    suite_parser.add_argument(
        "--persistent-agent-session",
        action="store_true",
        help="Keep the external local-process suite candidate alive across turns using the same stdin/stdout JSON contract; requires --agent-command",
    )
    suite_parser.add_argument(
        "--actor-id",
        action="append",
        default=[],
        help="Actor identifier (repeatable). Defaults to built-in deterministic actor set.",
    )
    suite_parser.add_argument(
        "--output",
        choices=("json", "pretty"),
        default="json",
        help="CLI output format",
    )
    suite_parser.add_argument(
        "--output-file",
        default=None,
        help="Optional path to write the emitted suite report JSON",
    )
    suite_parser.add_argument(
        "--timing-mode",
        choices=_TIMING_MODE_CHOICES,
        default=None,
        help="Optional explicit benchmark timing regime that resolves into cadence settings for all suite runs.",
    )
    suite_parser.add_argument(
        "--action-cadence-interval",
        type=int,
        default=None,
        help="Optional positive world-tick cadence interval for suite benchmark actor actions.",
    )
    suite_parser.add_argument(
        "--actor-action-cadence",
        action="append",
        default=[],
        help="Optional per-actor cadence override in actor_id=interval form; requires --action-cadence-interval.",
    )
    suite_parser.add_argument(
        "--direct-provider-timeout-seconds",
        type=float,
        default=None,
        help="Optional explicit timeout in seconds for external provider-backed suite rows; overrides the built-in default.",
    )
    suite_parser.add_argument(
        "--provider-min-turn-delay-seconds",
        type=float,
        default=None,
        help="Optional minimum wall-clock delay (in seconds) between provider-backed actor turns in each suite row.",
    )
    suite_parser.add_argument(
        "--provider-max-actions",
        type=int,
        default=None,
        help="Optional cap on total accepted provider-backed actor actions per suite row; each row finalizes after hitting the limit.",
    )

    reports_parser = subcommands.add_parser("reports", help="Inspect saved suite report artifacts")
    reports_subcommands = reports_parser.add_subparsers(dest="reports_command", required=True)

    reports_list_parser = reports_subcommands.add_parser("list", help="List saved suite report artifacts")
    reports_list_parser.add_argument("--dir", required=True, help="Directory containing saved report manifests")
    reports_list_parser.add_argument(
        "--output",
        choices=("json", "pretty"),
        default="json",
        help="CLI output format",
    )

    reports_history_parser = reports_subcommands.add_parser(
        "history", help="Summarize saved suite report artifacts as deterministic history"
    )
    reports_history_parser.add_argument("--dir", required=True, help="Directory containing saved report manifests")
    reports_history_parser.add_argument(
        "--output",
        choices=("json", "pretty"),
        default="json",
        help="CLI output format",
    )

    reports_export_parser = reports_subcommands.add_parser(
        "export", help="Export a stable saved-report viewmodel for downstream tooling"
    )
    reports_export_parser.add_argument("--dir", required=True, help="Directory containing saved report manifests")
    reports_export_parser.add_argument(
        "--output",
        choices=("json", "pretty"),
        default="json",
        help="CLI output format",
    )

    reports_show_parser = reports_subcommands.add_parser("show", help="Show a saved suite report artifact")
    reports_show_parser.add_argument("--manifest", required=True, help="Path to a saved suite report manifest")
    reports_show_parser.add_argument(
        "--output",
        choices=("json", "pretty"),
        default="json",
        help="CLI output format",
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.command == "run":
        actor_ids = tuple(args.actor_id) if len(args.actor_id) > 0 else _DEFAULT_ACTOR_IDS
        try:
            actor_action_cadence_overrides = _parse_actor_action_cadence_overrides(
                args.actor_action_cadence
            )
            if args.action_cadence_interval is not None and args.action_cadence_interval <= 0:
                raise ValueError("action_cadence_interval_must_be_positive")
            if (
                args.timing_mode is None
                and len(actor_action_cadence_overrides) > 0
                and args.action_cadence_interval is None
            ):
                raise ValueError("actor_action_cadence_requires_action_cadence_interval")
            if args.timing_mode is None and any(
                actor_id not in actor_ids for actor_id in actor_action_cadence_overrides
            ):
                raise ValueError("actor_action_cadence_actor_ids_must_be_subset_of_actor_ids")
            (
                resolved_timing_mode,
                resolved_action_cadence_interval,
                resolved_actor_action_cadence_overrides,
            ) = resolve_timing_mode_cadence_config(
                timing_mode=args.timing_mode,
                action_cadence_interval=args.action_cadence_interval,
                actor_action_cadence_overrides=actor_action_cadence_overrides,
                actor_ids=actor_ids,
            )
            if args.direct_provider is None and args.direct_provider_model is not None:
                raise ValueError("direct_provider_model_requires_direct_provider")
            if args.direct_provider is None and args.direct_provider_timeout_seconds is not None:
                raise ValueError("direct_provider_timeout_requires_direct_provider")
            if args.direct_provider is None and args.direct_provider_prompt_dump_dir is not None:
                raise ValueError("direct_provider_prompt_dump_dir_requires_direct_provider")
            scenario_payload = _resolve_run_scenario_payload(
                scenario_name=args.scenario,
                scenario_file=args.scenario_file,
            )
            external_agent_config = _resolve_external_agent_config(
                agent_command=args.agent_command,
                agent_label=args.agent_label,
                agent_profile=args.agent_profile,
                persistent_agent_session=args.persistent_agent_session,
            )
            external_agent_command = external_agent_config["command"]
            external_agent_label = external_agent_config["label"]
            external_agent_profile_id = external_agent_config["profile_id"]
            persistent_agent_session = external_agent_config["persistent_agent_session"]
            if args.direct_provider is not None:
                if external_agent_command is not None:
                    raise ValueError("direct_provider_conflicts_with_agent_command")
                if args.agent_profile is not None:
                    raise ValueError("direct_provider_conflicts_with_agent_profile")
                if args.agent_label is not None:
                    raise ValueError("direct_provider_conflicts_with_agent_label")
                if persistent_agent_session:
                    raise ValueError("direct_provider_conflicts_with_persistent_agent_session")
                if len(actor_ids) != 1:
                    raise ValueError("direct_provider_requires_single_actor")
                if args.direct_provider_timeout_seconds is not None and args.direct_provider_timeout_seconds <= 0.0:
                    raise ValueError("direct_provider_timeout_seconds_must_be_positive")
                if args.router_variant is not None and not _prompt_engine_supports_router_variant(args.prompt_engine):
                    raise ValueError("router_variant_requires_variant_prompt_engine")
                if (
                    args.router_variant is not None
                    and not _router_variant_matches_prompt_engine(args.prompt_engine, args.router_variant)
                ):
                    raise ValueError("router_variant_mismatch_for_prompt_engine")
                direct_provider_config = resolve_direct_provider_config(
                    provider=args.direct_provider,
                    model=args.direct_provider_model,
                    env=os.environ,
                )
                external_agent_command = build_direct_provider_command(
                    direct_provider_config,
                    python_executable=sys.executable,
                    prompt_engine=args.prompt_engine,
                    router_variant=args.router_variant,
                )
                external_agent_command = _augment_direct_provider_prompt_dump_command(
                    external_agent_command,
                    prompt_dump_dir=args.direct_provider_prompt_dump_dir,
                    actor_id=actor_ids[0],
                    scenario_id=_scenario_payload_id(scenario_payload),
                )
                external_agent_label = f"direct-provider:{direct_provider_config.provider}"
                external_agent_profile_id = None
            elif args.prompt_engine != "baseline":
                raise ValueError("prompt_engine_requires_direct_provider")
            elif args.router_variant is not None:
                raise ValueError("router_variant_requires_variant_prompt_engine")
            if persistent_agent_session and external_agent_command is None:
                raise ValueError("persistent_agent_session_requires_agent_command")
            if external_agent_label is not None and external_agent_command is None:
                raise ValueError("agent_label_requires_agent_command")
            if args.provider_min_turn_delay_seconds is not None and args.provider_min_turn_delay_seconds < 0.0:
                raise ValueError("provider_min_turn_delay_seconds_must_be_non_negative")
            if args.provider_max_actions is not None and args.provider_max_actions <= 0:
                raise ValueError("provider_max_actions_must_be_positive")
            if args.world_load_path is not None and not os.path.isfile(args.world_load_path):
                raise ValueError(f"world_load_path_not_found:{args.world_load_path}")
            if args.world_save_path is not None and args.world_save_slot is not None:
                raise ValueError("world_save_path and world_save_slot cannot both be provided")
            if args.world_load_path is not None and args.world_load_slot is not None:
                raise ValueError("world_load_path and world_load_slot cannot both be provided")
            if args.world_save_slot is not None:
                _slot_err = validate_slot_name(args.world_save_slot)
                if _slot_err:
                    raise ValueError(f"world_save_slot:{_slot_err}")
            if args.world_load_slot is not None:
                _slot_err = validate_slot_name(args.world_load_slot)
                if _slot_err:
                    raise ValueError(f"world_load_slot:{_slot_err}")
        except ValueError as exc:
            error_payload = {
                "accepted": False,
                "error_type": "run_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        config = BenchmarkRunnerConfig(
            run_id=args.run_id,
            benchmark_id=args.benchmark_id,
            scenario=scenario_payload,
            actor_ids=actor_ids,
            run_seed=args.run_seed,
            max_steps_override=args.max_steps,
            external_agent_command=external_agent_command,
            persistent_agent_session=persistent_agent_session,
            external_agent_timeout_seconds=args.direct_provider_timeout_seconds,
            timing_mode=args.timing_mode,
            action_cadence_interval=args.action_cadence_interval,
            actor_action_cadence_overrides=actor_action_cadence_overrides,
            provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
            provider_max_actions=args.provider_max_actions,
            world_save_path=args.world_save_path,
            world_load_path=args.world_load_path,
            world_save_slot=args.world_save_slot,
            world_load_slot=args.world_load_slot,
            save_dir=args.save_dir,
        )

        try:
            result = run_benchmark_lifecycle(config)
        except (ValueError, RuntimeError) as exc:
            error_payload = {
                "accepted": False,
                "error_type": "run_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        response_payload = _build_run_response(
            result.to_dict(),
            external_agent_label=external_agent_label,
            external_agent_profile_id=external_agent_profile_id,
        )
        if args.output == "pretty":
            print(json.dumps(response_payload, sort_keys=True, indent=2, ensure_ascii=True))
        else:
            print(json.dumps(response_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
        return 0
    if args.command == "play":
        try:
            scenario_payload = _resolve_run_scenario_payload(
                scenario_name=args.scenario,
                scenario_file=args.scenario_file,
            )
        except ValueError as exc:
            error_payload = {
                "accepted": False,
                "error_type": "play_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        try:
            result = run_human_console_session(
                scenario=scenario_payload,
                actor_id=args.actor_id,
                run_id=args.run_id,
                run_seed=args.run_seed,
                max_steps_override=args.max_steps,
            )
        except (ValueError, RuntimeError) as exc:
            error_payload = {
                "accepted": False,
                "error_type": "play_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        print(_render_human_console_summary(result))
        return 0
    if args.command == "play-shared-shard":
        mock_agent_actor_ids = tuple(args.mock_agent_actor_id)
        try:
            external_agent_config = {"command": None, "persistent_agent_session": False}
            external_agent_actor_id = args.external_agent_actor_id
            if args.direct_provider is None and args.direct_provider_model is not None:
                raise ValueError("direct_provider_model_requires_direct_provider")
            if args.direct_provider is not None:
                if args.agent_command is not None:
                    raise ValueError("direct_provider_conflicts_with_agent_command")
                if args.persistent_agent_session:
                    raise ValueError("direct_provider_conflicts_with_persistent_agent_session")
                direct_provider_config = resolve_direct_provider_config(
                    provider=args.direct_provider,
                    model=args.direct_provider_model,
                    env=os.environ,
                )
                external_agent_config = {
                    "command": build_direct_provider_command(
                        direct_provider_config,
                        python_executable=sys.executable,
                    ),
                    "persistent_agent_session": False,
                }
                if external_agent_actor_id is None:
                    external_agent_actor_id = "direct-provider"
            if args.agent_command is not None:
                external_agent_config = _resolve_external_agent_config(
                    agent_command=args.agent_command,
                    agent_label=None,
                    agent_profile=None,
                    persistent_agent_session=args.persistent_agent_session,
                )
                if external_agent_actor_id is None:
                    external_agent_actor_id = "external-agent"
            persistent_agent_session = external_agent_config["persistent_agent_session"]
            if len(args.actor_id) > 0:
                actor_ids = tuple(args.actor_id)
            elif external_agent_actor_id is not None:
                actor_ids = ("human-a", external_agent_actor_id)
            elif len(mock_agent_actor_ids) > 0:
                actor_ids = ("human-a",) + mock_agent_actor_ids
            else:
                actor_ids = ("human-a", "human-b")
            scenario_payload = _resolve_run_scenario_payload(
                scenario_name=args.scenario,
                scenario_file=args.scenario_file,
            )
            if len(actor_ids) < 2:
                raise ValueError("shared_shard_loop_requires_at_least_two_actors")
            if len(set(mock_agent_actor_ids)) != len(mock_agent_actor_ids):
                raise ValueError("shared_shard_loop_agent_actor_ids_must_be_unique")
            if any(actor_id not in actor_ids for actor_id in mock_agent_actor_ids):
                raise ValueError("shared_shard_loop_agent_actor_ids_must_be_subset_of_actor_ids")
            if len(mock_agent_actor_ids) >= len(actor_ids):
                raise ValueError("shared_shard_loop_requires_at_least_one_human_actor")
            if args.agent_command is None and args.direct_provider is None and args.external_agent_actor_id is not None:
                raise ValueError("external_agent_actor_id_requires_agent_command")
            if args.persistent_agent_session and args.agent_command is None:
                raise ValueError("persistent_agent_session_requires_agent_command")
            if args.agent_command is not None and external_agent_actor_id not in actor_ids:
                raise ValueError("shared_shard_loop_external_agent_actor_ids_must_be_subset_of_actor_ids")
            if external_agent_actor_id is not None and external_agent_actor_id in mock_agent_actor_ids:
                raise ValueError("shared_shard_loop_actor_id_conflicts_between_mock_and_external_agents")
            actor_action_cadence_overrides = _parse_actor_action_cadence_overrides(
                args.actor_action_cadence
            )
            if args.action_cadence_interval is not None and args.action_cadence_interval <= 0:
                raise ValueError("action_cadence_interval_must_be_positive")
            if (
                args.timing_mode is None
                and len(actor_action_cadence_overrides) > 0
                and args.action_cadence_interval is None
            ):
                raise ValueError("actor_action_cadence_requires_action_cadence_interval")
            if args.timing_mode is None and any(
                actor_id not in actor_ids for actor_id in actor_action_cadence_overrides
            ):
                raise ValueError("actor_action_cadence_actor_ids_must_be_subset_of_actor_ids")
            resolve_timing_mode_cadence_config(
                timing_mode=args.timing_mode,
                action_cadence_interval=args.action_cadence_interval,
                actor_action_cadence_overrides=actor_action_cadence_overrides,
                actor_ids=actor_ids,
            )
            if (
                args.shared_external_agent_timeout_seconds is not None
                and args.shared_external_agent_timeout_seconds <= 0.0
            ):
                raise ValueError("shared_external_agent_timeout_seconds_must_be_positive")
            if args.world_load_path is not None and not os.path.isfile(args.world_load_path):
                raise ValueError(f"world_load_path_not_found:{args.world_load_path}")
            if args.world_save_path is not None and args.world_save_slot is not None:
                raise ValueError("world_save_path and world_save_slot cannot both be provided")
            if args.world_load_path is not None and args.world_load_slot is not None:
                raise ValueError("world_load_path and world_load_slot cannot both be provided")
            if args.world_save_slot is not None:
                _slot_err = validate_slot_name(args.world_save_slot)
                if _slot_err:
                    raise ValueError(f"world_save_slot:{_slot_err}")
            if args.world_load_slot is not None:
                _slot_err = validate_slot_name(args.world_load_slot)
                if _slot_err:
                    raise ValueError(f"world_load_slot:{_slot_err}")
        except ValueError as exc:
            error_payload = {
                "accepted": False,
                "error_type": "play_shared_shard_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        try:
            result = run_human_shared_shard_session(
                scenario=scenario_payload,
                actor_ids=actor_ids,
                mock_agent_actor_ids=mock_agent_actor_ids,
                external_agent_commands_by_actor=(
                    {external_agent_actor_id: external_agent_config["command"]}
                    if external_agent_actor_id is not None and external_agent_config["command"] is not None
                    else None
                ),
                persistent_agent_session=persistent_agent_session,
                run_id=args.run_id,
                shard_id=args.shard_id,
                run_seed=args.run_seed,
                max_steps_override=args.max_steps,
                timing_mode=args.timing_mode,
                action_cadence_interval=args.action_cadence_interval,
                actor_action_cadence_overrides=actor_action_cadence_overrides,
                external_agent_timeout_seconds=args.shared_external_agent_timeout_seconds,
                world_load_path=args.world_load_path,
                world_save_path=args.world_save_path,
                world_load_slot=args.world_load_slot,
                world_save_slot=args.world_save_slot,
                save_dir=args.save_dir,
            )
        except (ValueError, RuntimeError) as exc:
            error_payload = {
                "accepted": False,
                "error_type": "play_shared_shard_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        print(_render_human_console_summary(result))
        return 0
    if args.command == "list-saves":
        effective_save_dir = args.save_dir or WORLD_SAVE_DIR_DEFAULT
        list_result = list_world_slots(effective_save_dir)
        response_payload: dict[str, Any] = list_result.to_dict()
        response_payload["accepted"] = list_result.accepted
        if args.output == "pretty":
            if not list_result.slots:
                print(f"No save slots found in: {list_result.save_dir}")
            else:
                print(f"Save slots in: {list_result.save_dir}")
                for slot in list_result.slots:
                    print(
                        f"  {slot.slot_name:20s}  scenario={slot.scenario_id}  "
                        f"tick={slot.world_tick}  version={slot.scenario_version}"
                    )
        else:
            print(json.dumps(response_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
        return 0
    if args.command == "compare-playable-slices":
        try:
            if args.direct_provider is None and args.direct_provider_model is not None:
                raise ValueError("direct_provider_model_requires_direct_provider")
            if args.direct_provider is None and args.direct_provider_comparison_timeout_seconds is not None:
                raise ValueError("direct_provider_comparison_timeout_requires_direct_provider")
            if args.direct_provider is None and args.direct_provider_comparison_prompt_dump_dir is not None:
                raise ValueError("direct_provider_comparison_prompt_dump_dir_requires_direct_provider")
            if args.direct_provider is None and args.include_routed_prompt_engine:
                raise ValueError("include_routed_prompt_engine_requires_direct_provider")
            if args.direct_provider is None and args.include_angular_canonical_prompt_engine:
                raise ValueError("include_angular_canonical_prompt_engine_requires_direct_provider")
            if args.direct_provider is None and args.include_legacy_router_backed_prompt_engine:
                raise ValueError("include_legacy_router_backed_prompt_engine_requires_direct_provider")
            if args.direct_provider is None and args.angular_router_variant is not None:
                raise ValueError("angular_router_variant_requires_direct_provider")
            if args.direct_provider is None and args.legacy_router_variant is not None:
                raise ValueError("legacy_router_variant_requires_direct_provider")
            if args.direct_provider_comparison_timeout_seconds is not None and (
                args.direct_provider_comparison_timeout_seconds <= 0.0
            ):
                raise ValueError("direct_provider_comparison_timeout_seconds_must_be_positive")
            if args.provider_min_turn_delay_seconds is not None and args.provider_min_turn_delay_seconds < 0.0:
                raise ValueError("provider_min_turn_delay_seconds_must_be_non_negative")
            if args.provider_max_actions is not None and args.provider_max_actions <= 0:
                raise ValueError("provider_max_actions_must_be_positive")
            if (
                args.direct_provider is not None
                and args.angular_router_variant is not None
                and not args.include_angular_canonical_prompt_engine
            ):
                raise ValueError("angular_router_variant_requires_angular_canonical_prompt_engine")
            if (
                args.direct_provider is not None
                and args.legacy_router_variant is not None
                and not args.include_legacy_router_backed_prompt_engine
            ):
                raise ValueError("legacy_router_variant_requires_legacy_router_backed_prompt_engine")

            direct_provider_command: tuple[str, ...] | None = None
            direct_provider_identity: str | None = None
            direct_provider_routed_command: tuple[str, ...] | None = None
            direct_provider_angular_canonical_commands: tuple[tuple[str, tuple[str, ...]], ...] = ()
            direct_provider_legacy_router_backed_command: tuple[str, ...] | None = None
            if args.direct_provider is not None:
                direct_provider_config = resolve_direct_provider_config(
                    provider=args.direct_provider,
                    model=args.direct_provider_model,
                    env=os.environ,
                )
                direct_provider_command = build_direct_provider_command(
                    direct_provider_config,
                    python_executable=sys.executable,
                )
                direct_provider_identity = f"direct-provider:{direct_provider_config.provider}"
                if args.include_routed_prompt_engine:
                    direct_provider_routed_command = build_direct_provider_command(
                        direct_provider_config,
                        python_executable=sys.executable,
                        prompt_engine="geometric-routed",
                    )
                if args.include_angular_canonical_prompt_engine:
                    direct_provider_angular_canonical_commands = tuple(
                        (
                            angular_router_variant,
                            build_direct_provider_command(
                                direct_provider_config,
                                python_executable=sys.executable,
                                prompt_engine="angular-canonical",
                                router_variant=angular_router_variant,
                            ),
                        )
                        for angular_router_variant in _resolve_compare_angular_router_variants(
                            args.angular_router_variant
                        )
                    )
                if args.include_legacy_router_backed_prompt_engine:
                    direct_provider_legacy_router_backed_command = build_direct_provider_command(
                        direct_provider_config,
                        python_executable=sys.executable,
                        prompt_engine="legacy-router-backed",
                        router_variant=args.legacy_router_variant,
                    )

            entries: list[dict[str, Any]] = []
            mode_ids = ["built_in", "mock_wrapper"]
            if direct_provider_command is not None:
                mode_ids.append("direct_provider")
            if direct_provider_routed_command is not None:
                mode_ids.append("direct_provider_routed")
            if direct_provider_angular_canonical_commands:
                mode_ids.append("direct_provider_angular_canonical")
            if direct_provider_legacy_router_backed_command is not None:
                mode_ids.append("direct_provider_legacy_router_backed")

            for scenario_name in _PLAYABLE_SLICE_SCENARIOS:
                scenario_payload = _SCENARIO_PRESETS[scenario_name]
                built_in_result = run_benchmark_lifecycle(
                    BenchmarkRunnerConfig(
                        run_id=f"cli-compare-built-{scenario_name}",
                        benchmark_id=args.benchmark_id,
                        scenario=scenario_payload,
                        actor_ids=(args.actor_id,),
                    )
                )
                entries.append(
                    build_playable_slice_comparison_entry(
                        built_in_result,
                        mode="built_in",
                        agent_identity="built-in",
                        actor_id=args.actor_id,
                    )
                )

                mock_wrapper_result = run_benchmark_lifecycle(
                    BenchmarkRunnerConfig(
                        run_id=f"cli-compare-mock-{scenario_name}",
                        benchmark_id=args.benchmark_id,
                        scenario=scenario_payload,
                        actor_ids=(args.actor_id,),
                        external_agent_command=(
                            sys.executable,
                            "examples/agents/mock_llm_wrapper.py",
                        ),
                    )
                )
                entries.append(
                    build_playable_slice_comparison_entry(
                        mock_wrapper_result,
                        mode="mock_wrapper",
                        agent_identity="mock-llm-wrapper",
                        actor_id=args.actor_id,
                    )
                )

                if direct_provider_command is not None and direct_provider_identity is not None:
                    direct_provider_result = run_benchmark_lifecycle(
                        BenchmarkRunnerConfig(
                            run_id=f"cli-compare-direct-{scenario_name}",
                            benchmark_id=args.benchmark_id,
                            scenario=scenario_payload,
                            actor_ids=(args.actor_id,),
                            external_agent_command=_augment_direct_provider_prompt_dump_command(
                                direct_provider_command,
                                prompt_dump_dir=args.direct_provider_comparison_prompt_dump_dir,
                                actor_id=args.actor_id,
                                scenario_id=_scenario_payload_id(scenario_payload),
                            ),
                            external_agent_timeout_seconds=args.direct_provider_comparison_timeout_seconds,
                            provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                            provider_max_actions=args.provider_max_actions,
                        )
                    )
                    entries.append(
                        _with_prompt_engine_metadata(
                            build_playable_slice_comparison_entry(
                                direct_provider_result,
                                mode="direct_provider",
                                agent_identity=direct_provider_identity,
                                actor_id=args.actor_id,
                            ),
                            prompt_engine="baseline",
                        )
                    )

                if direct_provider_routed_command is not None and direct_provider_identity is not None:
                    direct_provider_routed_result = run_benchmark_lifecycle(
                        BenchmarkRunnerConfig(
                            run_id=f"cli-compare-direct-routed-{scenario_name}",
                            benchmark_id=args.benchmark_id,
                            scenario=scenario_payload,
                            actor_ids=(args.actor_id,),
                            external_agent_command=_augment_direct_provider_prompt_dump_command(
                                direct_provider_routed_command,
                                prompt_dump_dir=args.direct_provider_comparison_prompt_dump_dir,
                                actor_id=args.actor_id,
                                scenario_id=_scenario_payload_id(scenario_payload),
                            ),
                            external_agent_timeout_seconds=args.direct_provider_comparison_timeout_seconds,
                            provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                            provider_max_actions=args.provider_max_actions,
                        )
                    )
                    entries.append(
                        _with_prompt_engine_metadata(
                            build_playable_slice_comparison_entry(
                                direct_provider_routed_result,
                                mode="direct_provider_routed",
                                agent_identity=f"{direct_provider_identity}:geometric-routed",
                                actor_id=args.actor_id,
                            ),
                            prompt_engine="geometric-routed",
                        )
                    )

                if direct_provider_angular_canonical_commands and direct_provider_identity is not None:
                    for (
                        angular_router_variant,
                        direct_provider_angular_canonical_command,
                    ) in direct_provider_angular_canonical_commands:
                        direct_provider_angular_canonical_result = run_benchmark_lifecycle(
                            BenchmarkRunnerConfig(
                                run_id=(
                                    f"cli-compare-direct-angular-canonical-"
                                    f"{angular_router_variant}-{scenario_name}"
                                ),
                                benchmark_id=args.benchmark_id,
                                scenario=scenario_payload,
                                actor_ids=(args.actor_id,),
                                external_agent_command=_augment_direct_provider_prompt_dump_command(
                                    direct_provider_angular_canonical_command,
                                    prompt_dump_dir=args.direct_provider_comparison_prompt_dump_dir,
                                    actor_id=args.actor_id,
                                    scenario_id=_scenario_payload_id(scenario_payload),
                                ),
                                external_agent_timeout_seconds=args.direct_provider_comparison_timeout_seconds,
                                provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                                provider_max_actions=args.provider_max_actions,
                            )
                        )
                        entries.append(
                            _with_prompt_engine_metadata(
                                build_playable_slice_comparison_entry(
                                    direct_provider_angular_canonical_result,
                                    mode="direct_provider_angular_canonical",
                                    agent_identity=(
                                        f"{direct_provider_identity}:angular-canonical:"
                                        f"{angular_router_variant}"
                                    ),
                                    actor_id=args.actor_id,
                                ),
                                prompt_engine="angular-canonical",
                                router_variant=angular_router_variant,
                            )
                        )

                if (
                    direct_provider_legacy_router_backed_command is not None
                    and direct_provider_identity is not None
                ):
                    direct_provider_legacy_router_backed_result = run_benchmark_lifecycle(
                        BenchmarkRunnerConfig(
                            run_id=f"cli-compare-direct-legacy-router-backed-{scenario_name}",
                            benchmark_id=args.benchmark_id,
                            scenario=scenario_payload,
                            actor_ids=(args.actor_id,),
                            external_agent_command=_augment_direct_provider_prompt_dump_command(
                                direct_provider_legacy_router_backed_command,
                                prompt_dump_dir=args.direct_provider_comparison_prompt_dump_dir,
                                actor_id=args.actor_id,
                                scenario_id=_scenario_payload_id(scenario_payload),
                            ),
                            external_agent_timeout_seconds=args.direct_provider_comparison_timeout_seconds,
                            provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                            provider_max_actions=args.provider_max_actions,
                        )
                    )
                    entries.append(
                        _with_prompt_engine_metadata(
                            build_playable_slice_comparison_entry(
                                direct_provider_legacy_router_backed_result,
                                mode="direct_provider_legacy_router_backed",
                                agent_identity=(
                                    f"{direct_provider_identity}:legacy-router-backed:"
                                    f"{args.legacy_router_variant or 'legacy-phase4d_hopf_base'}"
                                ),
                                actor_id=args.actor_id,
                            ),
                            prompt_engine="legacy-router-backed",
                            router_variant=args.legacy_router_variant or "legacy-phase4d_hopf_base",
                        )
                    )
        except (ValueError, RuntimeError) as exc:
            error_payload = {
                "accepted": False,
                "error_type": "playable_slice_comparison_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        response_payload = {
            "accepted": True,
            "comparison_schema": "playable_slice_comparison_v1",
            "benchmark_id": args.benchmark_id,
            "actor_id": args.actor_id,
            "scenario_ids": list(_PLAYABLE_SLICE_SCENARIOS),
            "mode_ids": mode_ids,
            "entry_count": len(entries),
            "entries": entries,
        }
        if (
            args.provider_min_turn_delay_seconds is not None
            or args.provider_max_actions is not None
        ):
            response_payload["provider_budget"] = {
                "provider_min_turn_delay_seconds": args.provider_min_turn_delay_seconds,
                "provider_max_actions": args.provider_max_actions,
            }
        print(_render_cli_output(response_payload, output_format=args.output))
        return 0
    if args.command == "suite":
        actor_ids = tuple(args.actor_id) if len(args.actor_id) > 0 else _DEFAULT_ACTOR_IDS
        try:
            actor_action_cadence_overrides = _parse_actor_action_cadence_overrides(
                args.actor_action_cadence
            )
            if args.action_cadence_interval is not None and args.action_cadence_interval <= 0:
                raise ValueError("action_cadence_interval_must_be_positive")
            if args.direct_provider_timeout_seconds is not None and args.direct_provider_timeout_seconds <= 0.0:
                raise ValueError("direct_provider_timeout_seconds_must_be_positive")
            if args.provider_min_turn_delay_seconds is not None and args.provider_min_turn_delay_seconds < 0.0:
                raise ValueError("provider_min_turn_delay_seconds_must_be_non_negative")
            if args.provider_max_actions is not None and args.provider_max_actions <= 0:
                raise ValueError("provider_max_actions_must_be_positive")
            if (
                args.timing_mode is None
                and len(actor_action_cadence_overrides) > 0
                and args.action_cadence_interval is None
            ):
                raise ValueError("actor_action_cadence_requires_action_cadence_interval")
            if args.timing_mode is None and any(
                actor_id not in actor_ids for actor_id in actor_action_cadence_overrides
            ):
                raise ValueError("actor_action_cadence_actor_ids_must_be_subset_of_actor_ids")
            (
                resolved_timing_mode,
                resolved_action_cadence_interval,
                resolved_actor_action_cadence_overrides,
            ) = resolve_timing_mode_cadence_config(
                timing_mode=args.timing_mode,
                action_cadence_interval=args.action_cadence_interval,
                actor_action_cadence_overrides=actor_action_cadence_overrides,
                actor_ids=actor_ids,
            )
            suite_results_for_saved_replay: tuple[tuple[str, Any], ...] = ()
            baseline_external_agent_config = _resolve_external_agent_config(
                agent_command=None,
                agent_label=None,
                agent_profile=args.baseline_agent_profile,
                persistent_agent_session=False,
            )
            candidate_external_agent_config = _resolve_external_agent_config(
                agent_command=None,
                agent_label=None,
                agent_profile=args.candidate_agent_profile,
                persistent_agent_session=False,
            )
            external_agent_config = _resolve_external_agent_config(
                agent_command=args.agent_command,
                agent_label=args.agent_label,
                agent_profile=args.agent_profile,
                persistent_agent_session=args.persistent_agent_session,
            )
            external_agent_command = external_agent_config["command"]
            external_agent_label = external_agent_config["label"]
            external_agent_profile_id = external_agent_config["profile_id"]
            persistent_agent_session = external_agent_config["persistent_agent_session"]
            if persistent_agent_session and external_agent_command is None:
                raise ValueError("persistent_agent_session_requires_agent_command")
            if external_agent_label is not None and external_agent_command is None:
                raise ValueError("agent_label_requires_agent_command")
            _validate_suite_comparison_args(
                baseline_agent=args.baseline_agent,
                candidate_agent=args.candidate_agent,
                actor_ids=actor_ids,
                external_agent_command=external_agent_command,
                external_agent_actor=args.external_agent_actor,
                baseline_agent_profile=args.baseline_agent_profile,
                candidate_agent_profile=args.candidate_agent_profile,
            )
            resolved_external_agent_id = (
                external_agent_profile_id or external_agent_label or _EXTERNAL_COMPARISON_AGENT_ID
            )
            resolved_external_agent_label = external_agent_label
            suite_scenario_names = _resolve_suite_scenarios(args.suite)
            if args.baseline_agent_profile is not None or args.candidate_agent_profile is not None:
                baseline_profile_id = baseline_external_agent_config["profile_id"]
                candidate_profile_id = candidate_external_agent_config["profile_id"]
                if args.external_agent_actor is not None:
                    shared_profile_result_bundle = tuple(
                        run_benchmark_lifecycle(
                            BenchmarkRunnerConfig(
                                run_id=f"cli-suite-profile-shared-{scenario_name}",
                                benchmark_id=args.benchmark_id,
                                scenario=_SCENARIO_PRESETS[scenario_name],
                                actor_ids=actor_ids,
                                external_agent_commands_by_actor={
                                    args.baseline_agent: baseline_external_agent_config["command"],
                                    args.external_agent_actor: candidate_external_agent_config["command"],
                                },
                                persistent_agent_session=(
                                    baseline_external_agent_config["persistent_agent_session"]
                                    or candidate_external_agent_config["persistent_agent_session"]
                                ),
                                timing_mode=args.timing_mode,
                                action_cadence_interval=args.action_cadence_interval,
                                actor_action_cadence_overrides=actor_action_cadence_overrides,
                                external_agent_timeout_seconds=args.direct_provider_timeout_seconds,
                                provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                                provider_max_actions=args.provider_max_actions,
                            )
                        )
                        for scenario_name in suite_scenario_names
                    )
                    report_payload = build_tiny_suite_mixed_external_profile_comparison_report(
                        shared_profile_result_bundle,
                        baseline_actor_id=args.baseline_agent,
                        candidate_actor_id=args.external_agent_actor,
                        baseline_external_agent_id=baseline_profile_id,
                        candidate_external_agent_id=candidate_profile_id,
                    )
                    suite_results_for_saved_replay = (("shared", shared_profile_result_bundle),)
                else:
                    baseline_profile_result_bundle = tuple(
                        run_benchmark_lifecycle(
                            BenchmarkRunnerConfig(
                                run_id=f"cli-suite-profile-baseline-{scenario_name}",
                                benchmark_id=args.benchmark_id,
                                scenario=_SCENARIO_PRESETS[scenario_name],
                                actor_ids=actor_ids,
                                external_agent_command=baseline_external_agent_config["command"],
                                persistent_agent_session=baseline_external_agent_config["persistent_agent_session"],
                                timing_mode=args.timing_mode,
                                action_cadence_interval=args.action_cadence_interval,
                                actor_action_cadence_overrides=actor_action_cadence_overrides,
                                external_agent_timeout_seconds=args.direct_provider_timeout_seconds,
                                provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                                provider_max_actions=args.provider_max_actions,
                            )
                        )
                        for scenario_name in suite_scenario_names
                    )
                    candidate_profile_result_bundle = tuple(
                        run_benchmark_lifecycle(
                            BenchmarkRunnerConfig(
                                run_id=f"cli-suite-profile-candidate-{scenario_name}",
                                benchmark_id=args.benchmark_id,
                                scenario=_SCENARIO_PRESETS[scenario_name],
                                actor_ids=actor_ids,
                                external_agent_command=candidate_external_agent_config["command"],
                                persistent_agent_session=candidate_external_agent_config["persistent_agent_session"],
                                timing_mode=args.timing_mode,
                                action_cadence_interval=args.action_cadence_interval,
                                actor_action_cadence_overrides=actor_action_cadence_overrides,
                                external_agent_timeout_seconds=args.direct_provider_timeout_seconds,
                                provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                                provider_max_actions=args.provider_max_actions,
                            )
                        )
                        for scenario_name in suite_scenario_names
                    )
                    report_payload = build_tiny_suite_external_profile_comparison_report(
                        baseline_profile_result_bundle,
                        candidate_profile_result_bundle,
                        compared_actor_id=args.baseline_agent,
                        baseline_external_agent_id=baseline_profile_id,
                        candidate_external_agent_id=candidate_profile_id,
                    )
                    suite_results_for_saved_replay = (
                        ("baseline", baseline_profile_result_bundle),
                        ("candidate", candidate_profile_result_bundle),
                    )
                report_payload = dict(report_payload)
                report_payload["baseline_external_agent_profile_id"] = baseline_profile_id
                report_payload["candidate_external_agent_profile_id"] = candidate_profile_id
                if baseline_external_agent_config["label"] is not None:
                    report_payload["baseline_external_agent_label"] = baseline_external_agent_config["label"]
                if candidate_external_agent_config["label"] is not None:
                    report_payload["candidate_external_agent_label"] = candidate_external_agent_config["label"]
                response_actor_ids = (baseline_profile_id, candidate_profile_id)
            elif args.baseline_agent is None and args.candidate_agent is None and external_agent_command is None:
                baseline_result_bundle = tuple(
                    run_benchmark_lifecycle(
                        BenchmarkRunnerConfig(
                            run_id=f"cli-suite-{scenario_name}",
                            benchmark_id=args.benchmark_id,
                            scenario=_SCENARIO_PRESETS[scenario_name],
                            actor_ids=actor_ids,
                            timing_mode=args.timing_mode,
                            action_cadence_interval=args.action_cadence_interval,
                            actor_action_cadence_overrides=actor_action_cadence_overrides,
                            external_agent_timeout_seconds=args.direct_provider_timeout_seconds,
                            provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                            provider_max_actions=args.provider_max_actions,
                        )
                    )
                    for scenario_name in suite_scenario_names
                )
                report_payload = build_tiny_suite_baseline_report(baseline_result_bundle)
                response_actor_ids = actor_ids
                suite_results_for_saved_replay = (("baseline", baseline_result_bundle),)
            elif external_agent_command is None:
                baseline_result_bundle = tuple(
                    run_benchmark_lifecycle(
                        BenchmarkRunnerConfig(
                            run_id=f"cli-suite-{scenario_name}",
                            benchmark_id=args.benchmark_id,
                            scenario=_SCENARIO_PRESETS[scenario_name],
                            actor_ids=actor_ids,
                            timing_mode=args.timing_mode,
                            action_cadence_interval=args.action_cadence_interval,
                            actor_action_cadence_overrides=actor_action_cadence_overrides,
                            external_agent_timeout_seconds=args.direct_provider_timeout_seconds,
                            provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                            provider_max_actions=args.provider_max_actions,
                        )
                    )
                    for scenario_name in suite_scenario_names
                )
                report_payload = build_tiny_suite_comparison_report(
                    baseline_result_bundle,
                    baseline_agent_id=args.baseline_agent,
                    candidate_agent_id=args.candidate_agent,
                )
                response_actor_ids = actor_ids
                suite_results_for_saved_replay = (("shared", baseline_result_bundle),)
            elif args.external_agent_actor is not None:
                baseline_result_bundle = tuple(
                    run_benchmark_lifecycle(
                        BenchmarkRunnerConfig(
                            run_id=f"cli-suite-{scenario_name}",
                            benchmark_id=args.benchmark_id,
                            scenario=_SCENARIO_PRESETS[scenario_name],
                            actor_ids=actor_ids,
                            timing_mode=args.timing_mode,
                            action_cadence_interval=args.action_cadence_interval,
                            actor_action_cadence_overrides=actor_action_cadence_overrides,
                            external_agent_timeout_seconds=args.direct_provider_timeout_seconds,
                            provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                            provider_max_actions=args.provider_max_actions,
                        )
                    )
                    for scenario_name in suite_scenario_names
                )
                mixed_result_bundle = tuple(
                    run_benchmark_lifecycle(
                        BenchmarkRunnerConfig(
                            run_id=f"cli-suite-mixed-{scenario_name}",
                            benchmark_id=args.benchmark_id,
                            scenario=_SCENARIO_PRESETS[scenario_name],
                            actor_ids=actor_ids,
                            external_agent_command=external_agent_command,
                            external_agent_actor_id=args.external_agent_actor,
                            persistent_agent_session=persistent_agent_session,
                            timing_mode=args.timing_mode,
                            action_cadence_interval=args.action_cadence_interval,
                            actor_action_cadence_overrides=actor_action_cadence_overrides,
                            external_agent_timeout_seconds=args.direct_provider_timeout_seconds,
                            provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                            provider_max_actions=args.provider_max_actions,
                        )
                    )
                    for scenario_name in suite_scenario_names
                )
                report_payload = build_tiny_suite_mixed_external_comparison_report(
                    mixed_result_bundle,
                    baseline_agent_id=args.baseline_agent,
                    external_actor_id=args.external_agent_actor,
                    external_agent_id=resolved_external_agent_id,
                )
                report_payload = dict(report_payload)
                if resolved_external_agent_label is not None:
                    report_payload["external_agent_label"] = resolved_external_agent_label
                if external_agent_profile_id is not None:
                    report_payload["external_agent_profile_id"] = external_agent_profile_id
                response_actor_ids = (args.baseline_agent, resolved_external_agent_id)
                suite_results_for_saved_replay = (("shared", mixed_result_bundle),)
            else:
                baseline_result_bundle = tuple(
                    run_benchmark_lifecycle(
                        BenchmarkRunnerConfig(
                            run_id=f"cli-suite-{scenario_name}",
                            benchmark_id=args.benchmark_id,
                            scenario=_SCENARIO_PRESETS[scenario_name],
                            actor_ids=actor_ids,
                            timing_mode=args.timing_mode,
                            action_cadence_interval=args.action_cadence_interval,
                            actor_action_cadence_overrides=actor_action_cadence_overrides,
                            external_agent_timeout_seconds=args.direct_provider_timeout_seconds,
                            provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                            provider_max_actions=args.provider_max_actions,
                        )
                    )
                    for scenario_name in suite_scenario_names
                )
                external_result_bundle = tuple(
                    run_benchmark_lifecycle(
                        BenchmarkRunnerConfig(
                            run_id=f"cli-suite-external-{scenario_name}",
                            benchmark_id=args.benchmark_id,
                            scenario=_SCENARIO_PRESETS[scenario_name],
                            actor_ids=actor_ids,
                            external_agent_command=external_agent_command,
                            persistent_agent_session=persistent_agent_session,
                            timing_mode=args.timing_mode,
                            action_cadence_interval=args.action_cadence_interval,
                            actor_action_cadence_overrides=actor_action_cadence_overrides,
                            external_agent_timeout_seconds=args.direct_provider_timeout_seconds,
                            provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
                            provider_max_actions=args.provider_max_actions,
                        )
                    )
                    for scenario_name in suite_scenario_names
                )
                report_payload = build_tiny_suite_external_comparison_report(
                    baseline_result_bundle,
                    external_result_bundle,
                    compared_actor_id=args.baseline_agent,
                    external_agent_id=resolved_external_agent_id,
                )
                report_payload = dict(report_payload)
                if resolved_external_agent_label is not None:
                    report_payload["external_agent_label"] = resolved_external_agent_label
                if external_agent_profile_id is not None:
                    report_payload["external_agent_profile_id"] = external_agent_profile_id
                response_actor_ids = (args.baseline_agent, resolved_external_agent_id)
                suite_results_for_saved_replay = (
                    ("baseline", baseline_result_bundle),
                    ("candidate", external_result_bundle),
                )
        except (ValueError, RuntimeError) as exc:
            error_payload = {
                "accepted": False,
                "error_type": "suite_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        response_payload = _build_suite_response(
            suite_id=args.suite,
            benchmark_id=args.benchmark_id,
            actor_ids=response_actor_ids,
            report_payload=report_payload,
            timing_mode=resolved_timing_mode,
            action_cadence_interval=resolved_action_cadence_interval,
            actor_action_cadence_overrides=resolved_actor_action_cadence_overrides,
            provider_min_turn_delay_seconds=args.provider_min_turn_delay_seconds,
            provider_max_actions=args.provider_max_actions,
        )
        rendered_output = _render_cli_output(response_payload, output_format=args.output)
        if args.output_file is not None:
            output_path = Path(args.output_file)
            try:
                output_path.write_text(rendered_output + "\n", encoding="utf-8")
            except OSError as exc:
                error_payload = {
                    "accepted": False,
                    "error_type": "suite_rejected",
                    "reason": f"output_file_write_failed:{args.output_file}:{exc.strerror or 'unknown_error'}",
                }
                print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
                return 1
            manifest_payload = _build_suite_output_manifest(
                response_payload=response_payload,
                artifact_path=output_path,
            )
            replay_drilldown_payload = _build_suite_replay_drilldown_payload(
                response_payload=response_payload,
                result_groups=suite_results_for_saved_replay,
                artifact_path=output_path,
            )
            replay_drilldown_path = _resolve_suite_replay_drilldown_path(output_path)
            manifest_path = _resolve_suite_manifest_path(output_path)
            try:
                manifest_path.write_text(
                    json.dumps(manifest_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n",
                    encoding="utf-8",
                )
            except OSError as exc:
                error_payload = {
                    "accepted": False,
                    "error_type": "suite_rejected",
                    "reason": (
                        f"output_manifest_write_failed:{manifest_path}:{exc.strerror or 'unknown_error'}"
                    ),
                }
                print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
                return 1
            try:
                replay_drilldown_path.write_text(
                    json.dumps(replay_drilldown_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n",
                    encoding="utf-8",
                )
            except OSError as exc:
                error_payload = {
                    "accepted": False,
                    "error_type": "suite_rejected",
                    "reason": (
                        f"output_replay_drilldown_write_failed:{replay_drilldown_path}:{exc.strerror or 'unknown_error'}"
                    ),
                }
                print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
                return 1
        print(rendered_output)
        return 0
    if args.command == "reports":
        try:
            if args.reports_command == "list":
                response_payload = _build_reports_list_response(Path(args.dir))
            elif args.reports_command == "history":
                response_payload = _build_reports_history_response(Path(args.dir))
            elif args.reports_command == "export":
                response_payload = _build_reports_export_response(Path(args.dir))
            elif args.reports_command == "show":
                response_payload = _build_reports_show_response(Path(args.manifest))
            else:
                raise ValueError(f"unsupported reports command: {args.reports_command}")
        except ValueError as exc:
            error_payload = {
                "accepted": False,
                "error_type": "reports_rejected",
                "reason": str(exc),
            }
            print(json.dumps(error_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
            return 1

        print(_render_cli_output(response_payload, output_format=args.output))
        return 0
    parser.error("Unsupported command")
    return 2


def _build_run_response(
    result_payload: Mapping[str, Any],
    *,
    external_agent_label: str | None = None,
    external_agent_profile_id: str | None = None,
) -> dict[str, Any]:
    lifecycle_payload = result_payload["lifecycle_state"]
    scorecard_payload = result_payload["scorecard"]
    replay_payload = result_payload["replay_artifact"]
    parity_payload = result_payload["replay_parity_artifact"]
    response_payload = {
        "accepted": True,
        "run_id": lifecycle_payload["run_id"],
        "benchmark_id": scorecard_payload["metadata"]["benchmark_id"],
        "scenario_id": lifecycle_payload["scenario_id"],
        "lifecycle": {
            "status": lifecycle_payload["status"],
            "step_count": lifecycle_payload["step_index"],
            "max_steps": lifecycle_payload["max_steps"],
            "seed": lifecycle_payload["seed"],
        },
        "scorecard": {
            "aggregate_score": scorecard_payload["aggregate_score"],
            "metadata": scorecard_payload["metadata"],
        },
        "replay": {
            "artifact_refs": result_payload["replay_artifact_refs"],
            "event_count": len(replay_payload["events"]),
            "schema_version": replay_payload["envelope"]["schema_version"],
            "parity": {
                "terminal_step": parity_payload["terminal_step"],
                "step_count": parity_payload["step_count"],
                "terminal_state_hash": parity_payload["terminal_state_hash"],
                "applied_steps_hash": parity_payload["applied_steps_hash"],
                "score_summary_hash": parity_payload["score_summary_hash"],
            },
        },
    }
    if external_agent_label is not None:
        response_payload["external_agent_label"] = external_agent_label
    if external_agent_profile_id is not None:
        response_payload["external_agent_profile_id"] = external_agent_profile_id
    timing_payload = result_payload.get("timing")
    if isinstance(timing_payload, Mapping):
        timing_mode = timing_payload.get("timing_mode")
        action_cadence_interval = timing_payload.get("action_cadence_interval")
        actor_action_cadence_overrides = list(timing_payload.get("actor_action_cadence_overrides", ()))
        actor_next_action_eligible_at = list(timing_payload.get("actor_next_action_eligible_at", ()))
        if timing_mode is not None:
            response_payload["timing_mode"] = timing_mode
        if action_cadence_interval is not None:
            response_payload["action_cadence_interval"] = action_cadence_interval
            response_payload["actor_action_cadence_overrides"] = actor_action_cadence_overrides
            response_payload["actor_next_action_eligible_at"] = actor_next_action_eligible_at
    provider_budget_payload = result_payload.get("provider_budget")
    if isinstance(provider_budget_payload, Mapping):
        response_payload["provider_budget"] = dict(provider_budget_payload)
    return response_payload


def _build_suite_response(
    *,
    suite_id: str,
    benchmark_id: str,
    actor_ids: Sequence[str],
    report_payload: Mapping[str, Any],
    timing_mode: str | None = None,
    action_cadence_interval: int | None = None,
    actor_action_cadence_overrides: Mapping[str, int] | None = None,
    provider_min_turn_delay_seconds: float | None = None,
    provider_max_actions: int | None = None,
) -> dict[str, Any]:
    response = {
        "accepted": True,
        "suite_id": suite_id,
        "benchmark_id": benchmark_id,
        "actor_ids": list(actor_ids),
        "report": report_payload,
    }
    if timing_mode is not None:
        response["timing_mode"] = timing_mode
    if action_cadence_interval is not None:
        response["action_cadence_interval"] = action_cadence_interval
        response["actor_action_cadence_overrides"] = [
            {"actor_id": actor_id, "cadence_interval": cadence_interval}
            for actor_id, cadence_interval in (actor_action_cadence_overrides or {}).items()
        ]
    if provider_min_turn_delay_seconds is not None or provider_max_actions is not None:
        response["provider_budget"] = {
            "provider_min_turn_delay_seconds": provider_min_turn_delay_seconds,
            "provider_max_actions": provider_max_actions,
        }
    return response


def _build_suite_output_manifest(
    *,
    response_payload: Mapping[str, Any],
    artifact_path: Path,
) -> dict[str, Any]:
    report_payload = response_payload["report"]
    report_schema_version = str(report_payload["schema_version"])
    if report_schema_version == "tiny_suite_baseline_report_v1":
        entries = report_payload["entries"]
        scenario_ids = sorted({str(entry["scenario_id"]) for entry in entries})
        actor_ids = sorted({str(entry["agent_id"]) for entry in entries})
        replay_present = all(bool(entry.get("replay_ref")) for entry in entries)
        parity_present = all(bool(entry.get("parity_ref")) for entry in entries)
        command_mode = "suite_baseline"
    elif report_schema_version == "tiny_suite_comparison_report_v1":
        comparisons = report_payload["comparisons"]
        scenario_ids = [str(entry["scenario_id"]) for entry in comparisons]
        actor_ids = [
            str(report_payload["baseline_agent_id"]),
            str(report_payload["candidate_agent_id"]),
        ]
        replay_present = all(
            bool(side.get("replay_ref"))
            for entry in comparisons
            for side in (entry["baseline"], entry["candidate"])
        )
        parity_present = all(
            bool(side.get("parity_ref"))
            for entry in comparisons
            for side in (entry["baseline"], entry["candidate"])
        )
        command_mode = "suite_comparison"
    else:
        raise ValueError(f"unsupported suite report schema_version: {report_schema_version}")

    return {
        "artifact_type": "suite_report_manifest_v1",
        "command_mode": command_mode,
        "artifact_path": str(artifact_path),
        "benchmark_id": response_payload["benchmark_id"],
        "suite_id": response_payload["suite_id"],
        "scenario_ids": scenario_ids,
        "actor_ids": actor_ids,
        "has_replay_refs": replay_present,
        "has_parity_refs": parity_present,
        "report_schema_version": report_schema_version,
        **(
            {"timing_mode": str(response_payload["timing_mode"])}
            if isinstance(response_payload.get("timing_mode"), str)
            and str(response_payload["timing_mode"]) != ""
            else {}
        ),
        **(
            {"action_cadence_interval": int(response_payload["action_cadence_interval"])}
            if isinstance(response_payload.get("action_cadence_interval"), int)
            and not isinstance(response_payload.get("action_cadence_interval"), bool)
            else {}
        ),
        **(
            {
                "actor_action_cadence_overrides": list(
                    response_payload.get("actor_action_cadence_overrides", ())
                )
            }
            if isinstance(response_payload.get("action_cadence_interval"), int)
            and not isinstance(response_payload.get("action_cadence_interval"), bool)
            else {}
        ),
        **(
            {"external_agent_label": str(report_payload["external_agent_label"])}
            if isinstance(report_payload.get("external_agent_label"), str)
            and str(report_payload["external_agent_label"]) != ""
            else {}
        ),
        **(
            {"external_agent_profile_id": str(report_payload["external_agent_profile_id"])}
            if isinstance(report_payload.get("external_agent_profile_id"), str)
            and str(report_payload["external_agent_profile_id"]) != ""
            else {}
        ),
        **(
            {"baseline_external_agent_profile_id": str(report_payload["baseline_external_agent_profile_id"])}
            if isinstance(report_payload.get("baseline_external_agent_profile_id"), str)
            and str(report_payload["baseline_external_agent_profile_id"]) != ""
            else {}
        ),
        **(
            {"candidate_external_agent_profile_id": str(report_payload["candidate_external_agent_profile_id"])}
            if isinstance(report_payload.get("candidate_external_agent_profile_id"), str)
            and str(report_payload["candidate_external_agent_profile_id"]) != ""
            else {}
        ),
        **(
            {"baseline_external_agent_label": str(report_payload["baseline_external_agent_label"])}
            if isinstance(report_payload.get("baseline_external_agent_label"), str)
            and str(report_payload["baseline_external_agent_label"]) != ""
            else {}
        ),
        **(
            {"candidate_external_agent_label": str(report_payload["candidate_external_agent_label"])}
            if isinstance(report_payload.get("candidate_external_agent_label"), str)
            and str(report_payload["candidate_external_agent_label"]) != ""
            else {}
        ),
    }


def _resolve_suite_manifest_path(output_path: Path) -> Path:
    if output_path.suffix:
        return output_path.with_suffix(output_path.suffix + ".manifest.json")
    return Path(str(output_path) + ".manifest.json")


def _resolve_suite_replay_drilldown_path(output_path: Path) -> Path:
    if output_path.suffix:
        return output_path.with_suffix(output_path.suffix + ".replay.json")
    return Path(str(output_path) + ".replay.json")


def _build_reports_list_response(directory_path: Path) -> dict[str, Any]:
    if not directory_path.exists():
        raise ValueError(f"reports_dir_not_found:{directory_path}")
    if not directory_path.is_dir():
        raise ValueError(f"reports_dir_not_directory:{directory_path}")

    artifact_summaries = [
        _load_saved_suite_report_artifact(manifest_path)["artifact"]
        for manifest_path in sorted(directory_path.glob("*.manifest.json"))
    ]
    return {
        "accepted": True,
        "command": "reports_list",
        "directory": str(directory_path),
        "artifact_count": len(artifact_summaries),
        "artifacts": artifact_summaries,
    }


def _build_reports_show_response(manifest_path: Path) -> dict[str, Any]:
    loaded_artifact = _load_saved_suite_report_artifact(manifest_path)
    return {
        "accepted": True,
        "command": "reports_show",
        "artifact": loaded_artifact["artifact"],
        "manifest": loaded_artifact["manifest"],
        "report": loaded_artifact["report"],
    }


def _build_reports_history_response(directory_path: Path) -> dict[str, Any]:
    if not directory_path.exists():
        raise ValueError(f"reports_dir_not_found:{directory_path}")
    if not directory_path.is_dir():
        raise ValueError(f"reports_dir_not_directory:{directory_path}")

    loaded_artifacts = [
        _load_saved_suite_report_artifact(manifest_path)
        for manifest_path in sorted(directory_path.glob("*.manifest.json"))
    ]
    history_entries = [_build_reports_history_entry(loaded_artifact) for loaded_artifact in loaded_artifacts]
    leaderboard = _build_reports_history_leaderboard(history_entries)
    identity_rollups = _build_reports_identity_rollups(history_entries)
    return {
        "accepted": True,
        "command": "reports_history",
        "directory": str(directory_path),
        "artifact_count": len(history_entries),
        "history": history_entries,
        "leaderboard": leaderboard,
        "identity_rollups": identity_rollups,
    }


def _build_reports_export_response(directory_path: Path) -> dict[str, Any]:
    history_payload = _build_reports_history_response(directory_path)
    history_entries = history_payload["history"]
    scenario_ids = sorted(
        {
            str(scenario_id)
            for entry in history_entries
            for scenario_id in entry.get("scenario_ids", ())
            if isinstance(scenario_id, str) and scenario_id
        }
    )
    actor_ids = sorted(
        {
            str(actor_id)
            for entry in history_entries
            for actor_id in entry.get("actor_ids", ())
            if isinstance(actor_id, str) and actor_id
        }
    )
    external_agent_labels = sorted(
        {
            str(entry["external_agent_label"])
            for entry in history_entries
            if isinstance(entry.get("external_agent_label"), str) and str(entry["external_agent_label"]) != ""
        }
        | {
            str(entry["baseline_external_agent_label"])
            for entry in history_entries
            if isinstance(entry.get("baseline_external_agent_label"), str)
            and str(entry["baseline_external_agent_label"]) != ""
        }
        | {
            str(entry["candidate_external_agent_label"])
            for entry in history_entries
            if isinstance(entry.get("candidate_external_agent_label"), str)
            and str(entry["candidate_external_agent_label"]) != ""
        }
    )
    external_agent_profile_ids = sorted(
        {
            str(entry["external_agent_profile_id"])
            for entry in history_entries
            if isinstance(entry.get("external_agent_profile_id"), str)
            and str(entry["external_agent_profile_id"]) != ""
        }
        | {
            str(entry["baseline_external_agent_profile_id"])
            for entry in history_entries
            if isinstance(entry.get("baseline_external_agent_profile_id"), str)
            and str(entry["baseline_external_agent_profile_id"]) != ""
        }
        | {
            str(entry["candidate_external_agent_profile_id"])
            for entry in history_entries
            if isinstance(entry.get("candidate_external_agent_profile_id"), str)
            and str(entry["candidate_external_agent_profile_id"]) != ""
        }
    )
    artifacts = [
        {
            "report_path": entry["report_path"],
            "manifest_path": entry["manifest_path"],
            "artifact_type": entry["artifact_type"],
            "command_mode": entry["command_mode"],
            "suite_id": entry["suite_id"],
            "benchmark_id": entry["benchmark_id"],
        }
        for entry in history_entries
    ]
    replay_drilldowns = [
        _build_reports_export_replay_drilldown_entry(
            entry,
            _load_suite_replay_drilldown_for_report(Path(entry["report_path"])),
        )
        for entry in history_entries
    ]
    identity_rollups = _build_reports_identity_rollups(history_entries, replay_drilldowns=replay_drilldowns)
    return {
        "accepted": True,
        "command": "reports_export",
        "directory": str(directory_path),
        "viewmodel_version": "reports_export_viewmodel_v1",
        "artifact_count": history_payload["artifact_count"],
        "coverage": {
            "scenario_ids": scenario_ids,
            "actor_ids": actor_ids,
            "external_agent_labels": external_agent_labels,
            "external_agent_profile_ids": external_agent_profile_ids,
        },
        "artifacts": artifacts,
        "history": history_entries,
        "leaderboard": history_payload["leaderboard"],
        "identity_rollups": identity_rollups,
        "replay_drilldowns": replay_drilldowns,
    }


def _load_saved_suite_report_artifact(manifest_path: Path) -> dict[str, Any]:
    manifest_payload = _read_json_mapping(manifest_path, missing_prefix="report_manifest_not_found")
    artifact_type = manifest_payload.get("artifact_type")
    if artifact_type != "suite_report_manifest_v1":
        raise ValueError(f"report_manifest_unsupported_artifact_type:{manifest_path}:{artifact_type}")

    artifact_path_value = manifest_payload.get("artifact_path")
    if not isinstance(artifact_path_value, str) or artifact_path_value == "":
        raise ValueError(f"report_manifest_missing_artifact_path:{manifest_path}")
    report_path = Path(artifact_path_value)
    report_payload = _read_json_mapping(report_path, missing_prefix="report_file_not_found")
    if report_payload.get("accepted") is not True:
        raise ValueError(f"report_file_invalid_payload:{report_path}")
    return {
        "artifact": {
            "report_path": str(report_path),
            "manifest_path": str(manifest_path),
            "artifact_type": artifact_type,
            "command_mode": manifest_payload.get("command_mode"),
            "suite_id": manifest_payload.get("suite_id"),
            "benchmark_id": manifest_payload.get("benchmark_id"),
        },
        "manifest": manifest_payload,
        "report": report_payload,
    }


def _build_reports_history_entry(loaded_artifact: Mapping[str, Any]) -> dict[str, Any]:
    artifact_payload = loaded_artifact["artifact"]
    manifest_payload = loaded_artifact["manifest"]
    report_payload = loaded_artifact["report"]
    history_entry = {
        "report_path": artifact_payload["report_path"],
        "manifest_path": artifact_payload["manifest_path"],
        "artifact_type": artifact_payload["artifact_type"],
        "command_mode": artifact_payload["command_mode"],
        "suite_id": artifact_payload["suite_id"],
        "benchmark_id": artifact_payload["benchmark_id"],
        "scenario_ids": list(manifest_payload.get("scenario_ids", ())),
        "actor_ids": list(manifest_payload.get("actor_ids", ())),
        "score_summary": _build_reports_history_score_summary(report_payload),
    }
    external_agent_label = manifest_payload.get("external_agent_label")
    if isinstance(external_agent_label, str) and external_agent_label:
        history_entry["external_agent_label"] = external_agent_label
    external_agent_profile_id = manifest_payload.get("external_agent_profile_id")
    if isinstance(external_agent_profile_id, str) and external_agent_profile_id:
        history_entry["external_agent_profile_id"] = external_agent_profile_id
    baseline_external_agent_profile_id = manifest_payload.get("baseline_external_agent_profile_id")
    if isinstance(baseline_external_agent_profile_id, str) and baseline_external_agent_profile_id:
        history_entry["baseline_external_agent_profile_id"] = baseline_external_agent_profile_id
    candidate_external_agent_profile_id = manifest_payload.get("candidate_external_agent_profile_id")
    if isinstance(candidate_external_agent_profile_id, str) and candidate_external_agent_profile_id:
        history_entry["candidate_external_agent_profile_id"] = candidate_external_agent_profile_id
    baseline_external_agent_label = manifest_payload.get("baseline_external_agent_label")
    if isinstance(baseline_external_agent_label, str) and baseline_external_agent_label:
        history_entry["baseline_external_agent_label"] = baseline_external_agent_label
    candidate_external_agent_label = manifest_payload.get("candidate_external_agent_label")
    if isinstance(candidate_external_agent_label, str) and candidate_external_agent_label:
        history_entry["candidate_external_agent_label"] = candidate_external_agent_label
    history_entry["identity_summary"] = _build_reports_history_identity_summary(history_entry)
    return history_entry


def _build_reports_history_identity_summary(history_entry: Mapping[str, Any]) -> list[dict[str, Any]]:
    actor_ids = history_entry.get("actor_ids", ())
    if not isinstance(actor_ids, Sequence) or isinstance(actor_ids, (str, bytes)):
        return []
    identities = [
        _build_reports_identity_metadata(history_entry, actor_id)
        for actor_id in actor_ids
        if isinstance(actor_id, str) and actor_id
    ]
    identities.sort(
        key=lambda entry: (
            str(entry["identity_type"]),
            str(entry.get("external_agent_profile_id", "")),
            str(entry.get("external_agent_label", "")),
            str(entry["actor_id"]),
        )
    )
    return identities


def _build_reports_identity_metadata(
    history_entry: Mapping[str, Any],
    actor_id: str,
) -> dict[str, Any]:
    external_agent_profile_id = history_entry.get("external_agent_profile_id")
    external_agent_label = history_entry.get("external_agent_label")
    baseline_external_agent_profile_id = history_entry.get("baseline_external_agent_profile_id")
    candidate_external_agent_profile_id = history_entry.get("candidate_external_agent_profile_id")
    baseline_external_agent_label = history_entry.get("baseline_external_agent_label")
    candidate_external_agent_label = history_entry.get("candidate_external_agent_label")
    if isinstance(baseline_external_agent_profile_id, str) and baseline_external_agent_profile_id == actor_id:
        identity_metadata = {
            "identity_type": "external_agent_profile",
            "actor_id": actor_id,
            "external_agent_profile_id": baseline_external_agent_profile_id,
        }
        if isinstance(baseline_external_agent_label, str) and baseline_external_agent_label:
            identity_metadata["external_agent_label"] = baseline_external_agent_label
        return identity_metadata
    if isinstance(candidate_external_agent_profile_id, str) and candidate_external_agent_profile_id == actor_id:
        identity_metadata = {
            "identity_type": "external_agent_profile",
            "actor_id": actor_id,
            "external_agent_profile_id": candidate_external_agent_profile_id,
        }
        if isinstance(candidate_external_agent_label, str) and candidate_external_agent_label:
            identity_metadata["external_agent_label"] = candidate_external_agent_label
        return identity_metadata
    if isinstance(external_agent_profile_id, str) and external_agent_profile_id == actor_id:
        identity_metadata = {
            "identity_type": "external_agent_profile",
            "actor_id": actor_id,
            "external_agent_profile_id": external_agent_profile_id,
        }
        if isinstance(external_agent_label, str) and external_agent_label:
            identity_metadata["external_agent_label"] = external_agent_label
        return identity_metadata
    if isinstance(external_agent_label, str) and external_agent_label == actor_id:
        return {
            "identity_type": "external_agent_label",
            "actor_id": actor_id,
            "external_agent_label": external_agent_label,
        }
    if actor_id == _EXTERNAL_COMPARISON_AGENT_ID:
        return {
            "identity_type": "external_agent_command",
            "actor_id": actor_id,
        }
    if actor_id in _BUILTIN_COMPARISON_AGENT_IDS:
        return {
            "identity_type": "built_in_actor",
            "actor_id": actor_id,
        }
    return {
        "identity_type": "actor_id",
        "actor_id": actor_id,
    }


def _build_reports_history_score_summary(report_payload: Mapping[str, Any]) -> dict[str, Any]:
    report = report_payload.get("report")
    if not isinstance(report, Mapping):
        raise ValueError("report_file_missing_report_payload")
    schema_version = report.get("schema_version")
    if schema_version == "tiny_suite_baseline_report_v1":
        entries = report.get("entries")
        if not isinstance(entries, list):
            raise ValueError("report_file_invalid_baseline_entries")
        aggregate_scores = [float(entry["aggregate_score"]) for entry in entries]
        composite_scores = [float(entry["composite_score"]) for entry in entries]
        return {
            "report_schema_version": schema_version,
            "entry_count": len(entries),
            "aggregate_score_max": max(aggregate_scores) if aggregate_scores else 0.0,
            "aggregate_score_min": min(aggregate_scores) if aggregate_scores else 0.0,
            "composite_score_max": max(composite_scores) if composite_scores else 0.0,
            "composite_score_min": min(composite_scores) if composite_scores else 0.0,
        }
    if schema_version == "tiny_suite_comparison_report_v1":
        summary = report.get("summary")
        if not isinstance(summary, Mapping):
            raise ValueError("report_file_invalid_comparison_summary")
        return {
            "report_schema_version": schema_version,
            "scenario_count": int(report.get("scenario_count", 0)),
            "baseline_composite_score_total": float(summary["baseline_composite_score_total"]),
            "candidate_composite_score_total": float(summary["candidate_composite_score_total"]),
            "composite_score_difference_total": float(summary["composite_score_difference_total"]),
            "composite_score_difference_average": float(summary["composite_score_difference_average"]),
        }
    raise ValueError(f"report_file_unsupported_schema_version:{schema_version}")


def _build_reports_history_leaderboard(
    history_entries: Sequence[Mapping[str, Any]],
) -> list[dict[str, Any]]:
    totals: dict[tuple[str, str, str, str], dict[str, Any]] = {}
    for entry in history_entries:
        actor_ids = entry.get("actor_ids", ())
        if not isinstance(actor_ids, Sequence) or isinstance(actor_ids, (str, bytes)):
            continue
        score_summary = entry.get("score_summary")
        if not isinstance(score_summary, Mapping):
            continue
        contribution_value = _resolve_reports_identity_score_contribution(score_summary)

        for actor_id in actor_ids:
            if not isinstance(actor_id, str) or not actor_id:
                continue
            identity_metadata = _build_reports_identity_metadata(entry, actor_id)
            identity_key = (
                str(identity_metadata["identity_type"]),
                str(identity_metadata["actor_id"]),
                str(identity_metadata.get("external_agent_profile_id", "")),
                str(identity_metadata.get("external_agent_label", "")),
            )
            actor_totals = totals.setdefault(
                identity_key,
                {
                    "actor_id": actor_id,
                    "identity_type": identity_metadata["identity_type"],
                    "artifact_count": 0,
                    "suite_ids": set(),
                    "benchmark_ids": set(),
                    "score_total": 0.0,
                    **(
                        {"external_agent_profile_id": identity_metadata["external_agent_profile_id"]}
                        if isinstance(identity_metadata.get("external_agent_profile_id"), str)
                        else {}
                    ),
                    **(
                        {"external_agent_label": identity_metadata["external_agent_label"]}
                        if isinstance(identity_metadata.get("external_agent_label"), str)
                        else {}
                    ),
                },
            )
            actor_totals["artifact_count"] += 1
            actor_totals["score_total"] += contribution_value
            suite_id = entry.get("suite_id")
            benchmark_id = entry.get("benchmark_id")
            if isinstance(suite_id, str) and suite_id:
                actor_totals["suite_ids"].add(suite_id)
            if isinstance(benchmark_id, str) and benchmark_id:
                actor_totals["benchmark_ids"].add(benchmark_id)

    leaderboard = []
    for identity_key in sorted(totals):
        actor_totals = totals[identity_key]
        leaderboard.append(
            {
                "actor_id": actor_totals["actor_id"],
                "identity_type": actor_totals["identity_type"],
                "artifact_count": int(actor_totals["artifact_count"]),
                "suite_ids": sorted(actor_totals["suite_ids"]),
                "benchmark_ids": sorted(actor_totals["benchmark_ids"]),
                "score_total": float(actor_totals["score_total"]),
                **(
                    {"external_agent_profile_id": actor_totals["external_agent_profile_id"]}
                    if isinstance(actor_totals.get("external_agent_profile_id"), str)
                    else {}
                ),
                **(
                    {"external_agent_label": actor_totals["external_agent_label"]}
                    if isinstance(actor_totals.get("external_agent_label"), str)
                    else {}
                ),
            }
        )
    leaderboard.sort(
        key=lambda entry: (
            -entry["score_total"],
            str(entry["identity_type"]),
            str(entry.get("external_agent_profile_id", "")),
            str(entry["actor_id"]),
        )
    )
    return leaderboard


def _resolve_reports_identity_score_contribution(score_summary: Mapping[str, Any]) -> float:
    report_schema_version = score_summary.get("report_schema_version")
    if report_schema_version == "tiny_suite_baseline_report_v1":
        return float(score_summary.get("composite_score_max", 0.0))
    if report_schema_version == "tiny_suite_comparison_report_v1":
        return float(score_summary.get("candidate_composite_score_total", 0.0))
    return 0.0


def _build_reports_identity_rollups(
    history_entries: Sequence[Mapping[str, Any]],
    *,
    replay_drilldowns: Sequence[Mapping[str, Any]] | None = None,
) -> list[dict[str, Any]]:
    replay_by_manifest_path = {
        str(entry.get("manifest_path")): entry
        for entry in (replay_drilldowns or ())
        if isinstance(entry, Mapping) and isinstance(entry.get("manifest_path"), str)
    }
    rollups: dict[tuple[str, str, str, str], dict[str, Any]] = {}

    for history_entry in history_entries:
        identities = history_entry.get("identity_summary")
        if not isinstance(identities, Sequence) or isinstance(identities, (str, bytes)):
            continue
        score_summary = history_entry.get("score_summary")
        if not isinstance(score_summary, Mapping):
            continue
        scenario_ids = sorted(
            {
                str(scenario_id)
                for scenario_id in history_entry.get("scenario_ids", ())
                if isinstance(scenario_id, str) and scenario_id
            }
        )
        suite_id = history_entry.get("suite_id")
        benchmark_id = history_entry.get("benchmark_id")
        is_comparison_artifact = history_entry.get("command_mode") == "suite_comparison"
        replay_entry = replay_by_manifest_path.get(str(history_entry.get("manifest_path", "")))
        has_shared_run_arena_artifact = False
        if isinstance(replay_entry, Mapping):
            runs = replay_entry.get("runs")
            has_shared_run_arena_artifact = (
                isinstance(runs, Sequence)
                and not isinstance(runs, (str, bytes))
                and any(
                    isinstance(run, Mapping) and run.get("source") == "shared"
                    for run in runs
                )
            )
        score_contribution = _resolve_reports_identity_score_contribution(score_summary)

        for identity in identities:
            if not isinstance(identity, Mapping):
                continue
            actor_id = identity.get("actor_id")
            identity_type = identity.get("identity_type")
            if not isinstance(actor_id, str) or actor_id == "":
                continue
            if not isinstance(identity_type, str) or identity_type == "":
                continue
            external_agent_profile_id = identity.get("external_agent_profile_id")
            external_agent_label = identity.get("external_agent_label")
            identity_key = (
                identity_type,
                actor_id,
                str(external_agent_profile_id or ""),
                str(external_agent_label or ""),
            )
            rollup = rollups.setdefault(
                identity_key,
                {
                    "identity_type": identity_type,
                    "identity_value": actor_id,
                    "actor_id": actor_id,
                    "artifact_count": 0,
                    "run_count": 0,
                    "scenario_ids": set(),
                    "suite_ids": set(),
                    "benchmark_ids": set(),
                    "score_total": 0.0,
                    "comparison_artifact_count": 0,
                    **(
                        {"external_agent_profile_id": external_agent_profile_id}
                        if isinstance(external_agent_profile_id, str) and external_agent_profile_id
                        else {}
                    ),
                    **(
                        {"external_agent_label": external_agent_label}
                        if isinstance(external_agent_label, str) and external_agent_label
                        else {}
                    ),
                },
            )
            rollup["artifact_count"] += 1
            rollup["run_count"] += len(scenario_ids)
            rollup["score_total"] += score_contribution
            if is_comparison_artifact:
                rollup["comparison_artifact_count"] += 1
            if isinstance(suite_id, str) and suite_id:
                rollup["suite_ids"].add(suite_id)
            if isinstance(benchmark_id, str) and benchmark_id:
                rollup["benchmark_ids"].add(benchmark_id)
            for scenario_id in scenario_ids:
                rollup["scenario_ids"].add(scenario_id)
            if has_shared_run_arena_artifact:
                rollup["shared_run_arena_artifact_count"] = int(
                    rollup.get("shared_run_arena_artifact_count", 0)
                ) + 1

    identity_rollups: list[dict[str, Any]] = []
    for identity_key in sorted(rollups):
        rollup = rollups[identity_key]
        artifact_count = int(rollup["artifact_count"])
        score_total = float(rollup["score_total"])
        built_rollup = {
            "identity_type": rollup["identity_type"],
            "identity_value": rollup["identity_value"],
            "actor_id": rollup["actor_id"],
            "scenario_ids": sorted(rollup["scenario_ids"]),
            "scenario_coverage_count": len(rollup["scenario_ids"]),
            "artifact_count": artifact_count,
            "run_count": int(rollup["run_count"]),
            "suite_ids": sorted(rollup["suite_ids"]),
            "benchmark_ids": sorted(rollup["benchmark_ids"]),
            "score_total": score_total,
            "score_average_per_artifact": (score_total / artifact_count) if artifact_count > 0 else 0.0,
            "comparison_artifact_count": int(rollup["comparison_artifact_count"]),
            "has_comparison_artifacts": bool(rollup["comparison_artifact_count"]),
            **(
                {"external_agent_profile_id": rollup["external_agent_profile_id"]}
                if isinstance(rollup.get("external_agent_profile_id"), str)
                else {}
            ),
            **(
                {"external_agent_label": rollup["external_agent_label"]}
                if isinstance(rollup.get("external_agent_label"), str)
                else {}
            ),
        }
        if "shared_run_arena_artifact_count" in rollup:
            built_rollup["shared_run_arena_artifact_count"] = int(rollup["shared_run_arena_artifact_count"])
            built_rollup["has_shared_run_arena_artifacts"] = bool(rollup["shared_run_arena_artifact_count"])
        identity_rollups.append(built_rollup)

    identity_rollups.sort(
        key=lambda entry: (
            -float(entry["score_total"]),
            str(entry["identity_type"]),
            str(entry.get("external_agent_profile_id", "")),
            str(entry.get("external_agent_label", "")),
            str(entry["identity_value"]),
        )
    )
    return identity_rollups


def _build_suite_replay_drilldown_payload(
    *,
    response_payload: Mapping[str, Any],
    result_groups: Sequence[tuple[str, Sequence[Any]]],
    artifact_path: Path,
) -> dict[str, Any]:
    runs: list[dict[str, Any]] = []
    for source_label, results in result_groups:
        for result in results:
            result_payload = result.to_dict()
            replay_payload = result_payload["replay_artifact"]
            scorecard_payload = result_payload["scorecard"]
            replay_ref = _find_named_ref(result_payload["replay_artifact_refs"], name="replay_artifact")
            final_state_summary = _extract_final_state_summary(replay_payload["events"])
            runs.append(
                {
                    "source": source_label,
                    "run_id": replay_payload["envelope"]["run_id"],
                    "scenario_id": replay_payload["envelope"]["scenario_id"],
                    "actor_ids": list(replay_payload["envelope"]["actor_ids"]),
                    "max_steps": int(replay_payload["envelope"]["max_steps"]),
                    "event_count": len(replay_payload["events"]),
                    "event_types": sorted({str(event["event_type"]) for event in replay_payload["events"]}),
                    "replay_ref": replay_ref,
                    "parity_ref": {
                        "terminal_state_hash": result_payload["replay_parity_artifact"]["terminal_state_hash"],
                        "applied_steps_hash": result_payload["replay_parity_artifact"]["applied_steps_hash"],
                        "score_summary_hash": result_payload["replay_parity_artifact"]["score_summary_hash"],
                    },
                    "final_state_summary": final_state_summary,
                    "score_summary": _build_replay_drilldown_score_summary(scorecard_payload),
                    "events": replay_payload["events"],
                }
            )
    runs.sort(key=lambda run: (str(run["scenario_id"]), str(run["source"]), str(run["run_id"])))
    return {
        "schema_version": _SUITE_REPLAY_DRILLDOWN_SCHEMA_VERSION,
        "artifact_type": _SUITE_REPLAY_DRILLDOWN_SCHEMA_VERSION,
        "artifact_path": str(_resolve_suite_replay_drilldown_path(artifact_path)),
        "report_path": str(artifact_path),
        "suite_id": response_payload["suite_id"],
        "benchmark_id": response_payload["benchmark_id"],
        "run_count": len(runs),
        "runs": runs,
    }


def _build_replay_drilldown_score_summary(scorecard_payload: Mapping[str, Any]) -> dict[str, Any]:
    actors = scorecard_payload.get("actors")
    if not isinstance(actors, list):
        raise ValueError("scorecard actors must be a list")
    return {
        "aggregate_score": float(scorecard_payload["aggregate_score"]),
        "actors": [
            {
                "actor_id": str(actor["actor_id"]),
                "composite_score": float(actor["composite_score"]),
                "normalized_metrics": actor["normalized_metrics"],
                "contributions": actor["contributions"],
            }
            for actor in actors
        ],
    }


def _find_named_ref(refs: Sequence[Mapping[str, Any]], *, name: str) -> str:
    for ref in refs:
        if ref.get("name") == name:
            value = ref.get("ref")
            if isinstance(value, str) and value:
                return value
    raise ValueError(f"missing_named_ref:{name}")


def _extract_final_state_summary(events: Sequence[Mapping[str, Any]]) -> dict[str, Any] | None:
    for event in reversed(tuple(events)):
        if event.get("event_type") != "state_snapshot":
            continue
        payload = event.get("payload")
        if not isinstance(payload, Mapping):
            continue
        state_json = payload.get("state_json")
        if not isinstance(state_json, str) or state_json == "":
            continue
        state_payload = json.loads(state_json)
        if not isinstance(state_payload, Mapping):
            continue
        return {
            "step_index": int(state_payload.get("step_index", 0)),
            "tracker_total_signals": state_payload.get("tracker_total_signals", {}),
            "agent_states": state_payload.get("agent_states", []),
        }
    return None


def _load_suite_replay_drilldown_for_report(report_path: Path) -> Mapping[str, Any]:
    replay_drilldown_path = _resolve_suite_replay_drilldown_path(report_path)
    replay_drilldown_payload = _read_json_mapping(
        replay_drilldown_path,
        missing_prefix="replay_drilldown_file_not_found",
    )
    replay_schema_version = replay_drilldown_payload.get("schema_version")
    if replay_schema_version != _SUITE_REPLAY_DRILLDOWN_SCHEMA_VERSION:
        raise ValueError(
            f"replay_drilldown_unsupported_schema_version:{replay_drilldown_path}:{replay_schema_version}"
        )
    if not isinstance(replay_drilldown_payload.get("runs"), list):
        raise ValueError(f"replay_drilldown_invalid_runs:{replay_drilldown_path}")
    return replay_drilldown_payload


def _build_reports_export_replay_drilldown_entry(
    history_entry: Mapping[str, Any],
    replay_drilldown_payload: Mapping[str, Any],
) -> dict[str, Any]:
    return {
        "manifest_path": history_entry["manifest_path"],
        "report_path": history_entry["report_path"],
        "replay_drilldown_path": replay_drilldown_payload["artifact_path"],
        "command_mode": history_entry["command_mode"],
        "suite_id": history_entry["suite_id"],
        "benchmark_id": history_entry["benchmark_id"],
        "scenario_ids": history_entry["scenario_ids"],
        "actor_ids": history_entry["actor_ids"],
        **(
            {"external_agent_label": history_entry["external_agent_label"]}
            if isinstance(history_entry.get("external_agent_label"), str)
            else {}
        ),
        **(
            {"external_agent_profile_id": history_entry["external_agent_profile_id"]}
            if isinstance(history_entry.get("external_agent_profile_id"), str)
            else {}
        ),
        **(
            {"baseline_external_agent_profile_id": history_entry["baseline_external_agent_profile_id"]}
            if isinstance(history_entry.get("baseline_external_agent_profile_id"), str)
            else {}
        ),
        **(
            {"candidate_external_agent_profile_id": history_entry["candidate_external_agent_profile_id"]}
            if isinstance(history_entry.get("candidate_external_agent_profile_id"), str)
            else {}
        ),
        **(
            {"baseline_external_agent_label": history_entry["baseline_external_agent_label"]}
            if isinstance(history_entry.get("baseline_external_agent_label"), str)
            else {}
        ),
        **(
            {"candidate_external_agent_label": history_entry["candidate_external_agent_label"]}
            if isinstance(history_entry.get("candidate_external_agent_label"), str)
            else {}
        ),
        "replay_run_count": int(replay_drilldown_payload.get("run_count", 0)),
        "runs": replay_drilldown_payload["runs"],
    }


def _read_json_mapping(path: Path, *, missing_prefix: str) -> Mapping[str, Any]:
    try:
        raw_payload = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise ValueError(f"{missing_prefix}:{path}:{exc.strerror or 'unknown_error'}") from exc

    try:
        parsed_payload = json.loads(raw_payload)
    except json.JSONDecodeError as exc:
        raise ValueError(f"json_payload_invalid:{path}:{exc.msg}") from exc

    if not isinstance(parsed_payload, Mapping):
        raise ValueError(f"json_payload_not_mapping:{path}")
    return parsed_payload


def _resolve_suite_scenarios(suite_id: str) -> tuple[str, ...]:
    if suite_id == "tiny":
        return _TINY_SUITE_SCENARIOS
    raise ValueError(f"unsupported suite: {suite_id}")


def _validate_suite_comparison_args(
    *,
    baseline_agent: str | None,
    candidate_agent: str | None,
    actor_ids: Sequence[str],
    external_agent_command: Sequence[str] | None,
    external_agent_actor: str | None,
    baseline_agent_profile: str | None,
    candidate_agent_profile: str | None,
) -> None:
    if baseline_agent_profile is not None or candidate_agent_profile is not None:
        if baseline_agent_profile is None or candidate_agent_profile is None:
            raise ValueError("baseline_agent_profile and candidate_agent_profile must both be provided")
        if external_agent_command is not None:
            raise ValueError("agent_command is not supported with dual external profile comparison")
        if candidate_agent is not None:
            raise ValueError("candidate_agent is not supported with dual external profile comparison")
        if baseline_agent is None:
            raise ValueError("baseline_agent must be provided for dual external profile comparison")
        if baseline_agent not in _BUILTIN_COMPARISON_AGENT_IDS:
            raise ValueError(f"unsupported baseline_agent: {baseline_agent}")
        configured_actor_ids = tuple(sorted(actor_ids))
        if baseline_agent not in configured_actor_ids:
            raise ValueError("baseline_agent must be present in configured actor_ids")
        if external_agent_actor is not None:
            if external_agent_actor not in _BUILTIN_COMPARISON_AGENT_IDS:
                raise ValueError(f"unsupported external_agent_actor: {external_agent_actor}")
            if external_agent_actor not in configured_actor_ids:
                raise ValueError("external_agent_actor must be present in configured actor_ids")
            if external_agent_actor == baseline_agent:
                raise ValueError("external_agent_actor must differ from baseline_agent")
        return
    if external_agent_command is None and baseline_agent is None and candidate_agent is None:
        return
    if external_agent_command is not None:
        if baseline_agent is None:
            raise ValueError("baseline_agent must be provided for suite external comparison")
        configured_actor_ids = tuple(sorted(actor_ids))
        if baseline_agent not in _BUILTIN_COMPARISON_AGENT_IDS:
            raise ValueError(f"unsupported baseline_agent: {baseline_agent}")
        if baseline_agent not in configured_actor_ids:
            raise ValueError("baseline_agent must be present in configured actor_ids")
        if external_agent_actor is not None:
            if candidate_agent is not None:
                raise ValueError("candidate_agent is not supported with mixed suite external comparison")
            if external_agent_actor not in _BUILTIN_COMPARISON_AGENT_IDS:
                raise ValueError(f"unsupported external_agent_actor: {external_agent_actor}")
            if external_agent_actor not in configured_actor_ids:
                raise ValueError("external_agent_actor must be present in configured actor_ids")
            if external_agent_actor == baseline_agent:
                raise ValueError("external_agent_actor must differ from baseline_agent")
            return
        if candidate_agent is not None:
            raise ValueError("candidate_agent is not supported with suite external comparison")
        return
    if external_agent_actor is not None:
        raise ValueError("external_agent_actor requires agent_command")
    if baseline_agent is None or candidate_agent is None:
        raise ValueError("baseline_agent and candidate_agent must both be provided for suite comparison")
    if baseline_agent not in _BUILTIN_COMPARISON_AGENT_IDS:
        raise ValueError(f"unsupported baseline_agent: {baseline_agent}")
    if candidate_agent not in _BUILTIN_COMPARISON_AGENT_IDS:
        raise ValueError(f"unsupported candidate_agent: {candidate_agent}")
    if baseline_agent == candidate_agent:
        raise ValueError("baseline_agent and candidate_agent must differ")
    configured_actor_ids = tuple(sorted(actor_ids))
    if baseline_agent not in configured_actor_ids or candidate_agent not in configured_actor_ids:
        raise ValueError("baseline_agent and candidate_agent must be present in configured actor_ids")


def _resolve_run_scenario_payload(
    *,
    scenario_name: str,
    scenario_file: str | None,
) -> Mapping[str, Any]:
    if scenario_file is None:
        return _SCENARIO_PRESETS[scenario_name]

    file_path = Path(scenario_file)
    try:
        raw_payload = file_path.read_text(encoding="utf-8")
    except OSError as exc:
        raise ValueError(f"scenario_file_read_failed:{file_path}:{exc.strerror or 'unknown_error'}") from exc

    try:
        parsed_payload = json.loads(raw_payload)
    except json.JSONDecodeError as exc:
        raise ValueError(f"scenario_file_invalid_json:{file_path}:{exc.msg}") from exc

    if not isinstance(parsed_payload, Mapping):
        raise ValueError(f"scenario_file_payload_not_mapping:{file_path}")
    return parsed_payload


def _resolve_external_agent_command(agent_command: str | None) -> tuple[str, ...] | None:
    if agent_command is None:
        return None
    try:
        parsed_command = tuple(shlex.split(agent_command))
    except ValueError as exc:
        raise ValueError(f"external_agent_command_invalid:{exc}") from exc
    if len(parsed_command) == 0:
        raise ValueError("external_agent_command_empty")
    return parsed_command


def _resolve_external_agent_config(
    *,
    agent_command: str | None,
    agent_label: str | None,
    agent_profile: str | None,
    persistent_agent_session: bool,
) -> dict[str, Any]:
    if agent_profile is not None and agent_command is not None:
        raise ValueError("agent_profile_conflicts_with_agent_command")
    if agent_profile is not None and agent_label is not None:
        raise ValueError("agent_profile_conflicts_with_agent_label")
    if agent_profile is None:
        return {
            "command": _resolve_external_agent_command(agent_command),
            "label": _resolve_external_agent_label(agent_label),
            "profile_id": None,
            "persistent_agent_session": persistent_agent_session,
        }

    profile_path = Path(agent_profile)
    profile_payload = _read_json_mapping(profile_path, missing_prefix="agent_profile_read_failed")
    agent_id = profile_payload.get("agent_id")
    if not isinstance(agent_id, str) or agent_id.strip() == "":
        raise ValueError(f"agent_profile_missing_agent_id:{profile_path}")
    command_value = profile_payload.get("command")
    if not isinstance(command_value, str):
        raise ValueError(f"agent_profile_missing_command:{profile_path}")
    display_name = profile_payload.get("display_name")
    if display_name is not None and (not isinstance(display_name, str) or display_name.strip() == ""):
        raise ValueError(f"agent_profile_invalid_display_name:{profile_path}")
    profile_persistent = profile_payload.get("persistent_agent_session", False)
    if not isinstance(profile_persistent, bool):
        raise ValueError(f"agent_profile_invalid_persistent_agent_session:{profile_path}")
    return {
        "command": _resolve_external_agent_command(command_value),
        "label": display_name.strip() if isinstance(display_name, str) else None,
        "profile_id": agent_id.strip(),
        "persistent_agent_session": persistent_agent_session or profile_persistent,
    }


def _parse_actor_action_cadence_overrides(
    raw_values: Sequence[str],
) -> dict[str, int]:
    normalized_overrides: dict[str, int] = {}
    for raw_value in raw_values:
        if not isinstance(raw_value, str) or raw_value == "":
            raise ValueError("actor_action_cadence_entries_must_be_non_empty_strings")
        actor_id, separator, raw_interval = raw_value.partition("=")
        if separator == "" or actor_id == "" or raw_interval == "":
            raise ValueError("actor_action_cadence_entries_must_use_actor_id_equals_interval")
        try:
            cadence_interval = int(raw_interval)
        except ValueError as exc:
            raise ValueError("actor_action_cadence_interval_must_be_integer") from exc
        if cadence_interval <= 0:
            raise ValueError("actor_action_cadence_interval_must_be_positive")
        normalized_overrides[actor_id] = cadence_interval
    return {actor_id: normalized_overrides[actor_id] for actor_id in sorted(normalized_overrides)}


def _resolve_external_agent_label(agent_label: str | None) -> str | None:
    if agent_label is None:
        return None
    normalized_label = agent_label.strip()
    if normalized_label == "":
        raise ValueError("external_agent_label_empty")
    return normalized_label


def _scenario_payload_id(scenario_payload: Mapping[str, Any]) -> str | None:
    scenario_id = scenario_payload.get("scenario_id")
    if scenario_id is None:
        return None
    if not isinstance(scenario_id, str) or not scenario_id:
        raise ValueError("scenario_payload_missing_valid_scenario_id")
    return scenario_id


def _augment_direct_provider_prompt_dump_command(
    command: Sequence[str],
    *,
    prompt_dump_dir: str | None,
    actor_id: str,
    scenario_id: str | None,
) -> tuple[str, ...]:
    normalized_command = tuple(command)
    if prompt_dump_dir is None:
        return normalized_command
    if not isinstance(prompt_dump_dir, str) or not prompt_dump_dir:
        raise ValueError("direct_provider_prompt_dump_dir_must_be_non_empty")
    if not isinstance(actor_id, str) or not actor_id:
        raise ValueError("prompt_dump_actor_id_must_be_non_empty")

    augmented_command = normalized_command + (
        "--prompt-dump-dir",
        prompt_dump_dir,
        "--prompt-dump-actor-id",
        actor_id,
    )
    if scenario_id is not None:
        augmented_command += ("--prompt-dump-scenario-id", scenario_id)
    return augmented_command


def _prompt_engine_supports_router_variant(prompt_engine: str) -> bool:
    return prompt_engine in {"angular-canonical", "legacy-router-backed"}


def _router_variant_matches_prompt_engine(prompt_engine: str, router_variant: str) -> bool:
    if prompt_engine == "angular-canonical":
        return router_variant in _ANGULAR_ROUTER_VARIANTS
    if prompt_engine == "legacy-router-backed":
        return router_variant in _LEGACY_ROUTER_VARIANTS
    return False


def _resolve_compare_angular_router_variants(raw_variants: Sequence[str] | None) -> tuple[str, ...]:
    if not raw_variants:
        return ("angular-hopf-trans",)

    ordered_variants: list[str] = []
    for router_variant in raw_variants:
        if router_variant not in ordered_variants:
            ordered_variants.append(router_variant)
    return tuple(ordered_variants)


def _with_prompt_engine_metadata(
    entry: Mapping[str, Any],
    *,
    prompt_engine: str,
    router_variant: str | None = None,
) -> dict[str, Any]:
    enriched_entry = dict(entry)
    enriched_entry["prompt_engine"] = prompt_engine
    if router_variant is not None:
        enriched_entry["router_variant"] = router_variant
    return enriched_entry


def _render_cli_output(payload: Mapping[str, Any], *, output_format: str) -> str:
    if output_format == "pretty":
        return json.dumps(payload, sort_keys=True, indent=2, ensure_ascii=True)
    return json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _render_human_console_summary(result: Mapping[str, Any] | object) -> str:
    if isinstance(result, Mapping):
        payload = result
    elif is_dataclass(result):
        payload = asdict(result)
    else:
        payload = dict(result.__dict__)
    inventory = payload.get("final_inventory", ())
    rendered_inventory = ", ".join(str(item) for item in inventory) if isinstance(inventory, Sequence) else ""
    if rendered_inventory == "":
        rendered_inventory = "empty"
    if "shard_id" in payload:
        actor_ids = payload.get("actor_ids", ())
        rendered_actor_ids = ", ".join(str(actor_id) for actor_id in actor_ids) if isinstance(
            actor_ids, Sequence
        ) else ""
        if rendered_actor_ids == "":
            rendered_actor_ids = "none"
        completed_actor_ids = payload.get("completed_actor_ids", ())
        rendered_completed_actor_ids = ", ".join(
            str(actor_id) for actor_id in completed_actor_ids
        ) if isinstance(completed_actor_ids, Sequence) else ""
        if rendered_completed_actor_ids == "":
            rendered_completed_actor_ids = "none"
        lines = [
            "Session Summary",
            f"Scenario: {payload['scenario_id']}",
            f"Shard: {payload['shard_id']}",
            f"Actors: {rendered_actor_ids}",
            f"Steps Played: {payload['step_count']}/{payload['max_steps']}",
            f"Completed Actors: {rendered_completed_actor_ids}",
            f"Quit Requested: {payload['quit_requested']}",
            f"Shard Mutation Generation: {payload['shard_mutation_generation']}",
            f"World Tick Count: {payload.get('world_tick_count', 0)}",
            f"Last World Tick Heartbeat: {payload.get('last_world_tick_heartbeat', 'not_started')}",
            f"World NPC Stance Phase: {payload.get('world_npc_stance_phase', 'dormant')}",
        ]
        timing_mode = payload.get("timing_mode")
        if timing_mode is not None:
            lines.append(f"Timing Mode: {timing_mode}")
        action_cadence_interval = payload.get("action_cadence_interval")
        if action_cadence_interval is not None:
            cadence_overrides = payload.get("actor_action_cadence_overrides", ())
            next_eligible = payload.get("actor_next_action_eligible_at", ())
            rendered_overrides = ", ".join(
                f"{actor_id}={cadence_interval}"
                for actor_id, cadence_interval in cadence_overrides
            ) if isinstance(cadence_overrides, Sequence) else ""
            rendered_next_eligible = ", ".join(
                f"{actor_id}={tick_value}"
                for actor_id, tick_value in next_eligible
            ) if isinstance(next_eligible, Sequence) else ""
            lines.extend(
                (
                    f"Action Cadence Interval: {action_cadence_interval}",
                    f"Actor Action Cadence Overrides: {rendered_overrides or 'none'}",
                    f"Next Action Eligible At: {rendered_next_eligible or 'none'}",
                )
            )
        return "\n".join(lines)
    return "\n".join(
        (
            "Session Summary",
            f"Scenario: {payload['scenario_id']}",
            f"Actor: {payload['actor_id']}",
            f"Steps Played: {payload['step_count']}/{payload['max_steps']}",
            f"Objective Completed: {payload['objective_completed']}",
            f"Quit Requested: {payload['quit_requested']}",
            f"Final Location: {payload['final_location']}",
            f"Final Inventory: {rendered_inventory}",
        )
    )


if __name__ == "__main__":
    raise SystemExit(main())
