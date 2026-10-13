#!/usr/bin/env python3
"""Research-only wrapper for guarded routing trials on real MUDBench replay signal."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


ROOT = Path(__file__).resolve().parents[2]
AI_RESEARCH_ROOT = ROOT.parents[1]
DEFAULT_REPLAY = (
    AI_RESEARCH_ROOT
    / "MUDBench"
    / "tmp"
    / "timing_mode_eval_v1"
    / "off_baseline.json.replay.json"
)

SIGNAL_METRICS = (
    "actions.count",
    "coverage.rooms",
    "objective.progress",
    "quest.completed",
    "social.give.completed",
    "social.trade.completed",
    "combat.damage_dealt",
    "combat.damage_taken",
    "survival.steps_alive",
)


@dataclass(frozen=True)
class RealSignalTrace:
    trace_source: str
    scenario_id: str
    run_id: str
    trace_length: int
    base_phases: list[int]
    phi_tuples: list[tuple[int, int]]
    next_return_gaps: list[int]
    route_full_keys: list[str]
    salient_bits: list[int]
    tracker_totals: list[int]
    mean_signal_pressure: float


def _aggregate_metrics(state: dict[str, object]) -> dict[str, float]:
    totals = {metric_name: 0.0 for metric_name in SIGNAL_METRICS}
    for agent in state.get("agent_states", ()):
        for metric in agent.get("metrics", ()):
            metric_name = str(metric.get("metric_name", ""))
            if metric_name in totals:
                totals[metric_name] += float(metric.get("last_value", 0.0))
    return totals


def _metric_delta_series(values: list[dict[str, float]], metric_name: str) -> list[float]:
    out: list[float] = []
    previous = 0.0
    for row in values:
        current = float(row.get(metric_name, 0.0))
        out.append(current - previous)
        previous = current
    return out


def _future_signal_keys(signal_words: list[tuple[int, ...]], horizon: int) -> list[str]:
    out: list[str] = []
    length = len(signal_words)
    for index in range(length):
        future = tuple(signal_words[(index + step) % length] for step in range(horizon))
        out.append(f"full:future={future}")
    return out


def _next_return_gaps(bits: list[int]) -> list[int]:
    length = len(bits)
    gaps: list[int] = []
    for index in range(length):
        gap = 0
        while index + gap < length and bits[index + gap] == 0:
            gap += 1
        gaps.append(gap)
    return gaps


def build_real_signal_traces(
    replay_path: Path = DEFAULT_REPLAY,
    *,
    horizon: int = 3,
    replay_paths: Iterable[Path] | None = None,
) -> list[RealSignalTrace]:
    traces: list[RealSignalTrace] = []
    active_replays = tuple(replay_paths) if replay_paths is not None else (replay_path,)
    for active_replay in active_replays:
        replay = json.loads(active_replay.read_text())
        for run in replay["runs"]:
            snapshots = []
            step_domain_counts: dict[int, int] = {}
            for event in run["events"]:
                if event["event_type"] == "step_completed":
                    step_domain_counts[int(event["step_index"])] = int(event["payload"].get("domain_event_count", 0))
                if event["event_type"] == "state_snapshot":
                    snapshots.append(json.loads(event["payload"]["state_json"]))

            aggregate_rows = [_aggregate_metrics(snapshot) for snapshot in snapshots]
            tracker_totals = [int(snapshot.get("tracker_total_signals", 0)) for snapshot in snapshots]

            action_delta = _metric_delta_series(aggregate_rows, "actions.count")
            coverage_delta = _metric_delta_series(aggregate_rows, "coverage.rooms")
            objective_delta = _metric_delta_series(aggregate_rows, "objective.progress")
            quest_delta = _metric_delta_series(aggregate_rows, "quest.completed")
            give_delta = _metric_delta_series(aggregate_rows, "social.give.completed")
            trade_delta = _metric_delta_series(aggregate_rows, "social.trade.completed")
            combat_out_delta = _metric_delta_series(aggregate_rows, "combat.damage_dealt")
            combat_in_delta = _metric_delta_series(aggregate_rows, "combat.damage_taken")
            survival_delta = _metric_delta_series(aggregate_rows, "survival.steps_alive")

            signal_words: list[tuple[int, ...]] = []
            salient_bits: list[int] = []
            phi_tuples: list[tuple[int, int]] = []
            for index, snapshot in enumerate(snapshots):
                step_index = int(snapshot["step_index"])
                objective_event = int(
                    objective_delta[index] > 0
                    or quest_delta[index] > 0
                    or give_delta[index] > 0
                    or trade_delta[index] > 0
                )
                world_event = int(coverage_delta[index] > 0 or step_domain_counts.get(step_index, 0) > 2)
                combat_event = int(combat_out_delta[index] > 0 or combat_in_delta[index] > 0)
                pressure_event = int(
                    survival_delta[index] > 0 and tracker_totals[index] > (tracker_totals[index - 1] if index > 0 else 0)
                )
                signal_words.append((objective_event, world_event, combat_event, pressure_event))
                salient_bits.append(int(objective_event or world_event or combat_event))
                phi_tuples.append((objective_event + (2 * world_event), combat_event + (2 * pressure_event)))

            traces.append(
                RealSignalTrace(
                    trace_source=active_replay.name,
                    scenario_id=str(run["scenario_id"]),
                    run_id=str(run["run_id"]),
                    trace_length=len(snapshots),
                    base_phases=[index % 35 for index in range(len(snapshots))],
                    phi_tuples=phi_tuples,
                    next_return_gaps=_next_return_gaps(salient_bits),
                    route_full_keys=_future_signal_keys(signal_words, horizon),
                    salient_bits=salient_bits,
                    tracker_totals=tracker_totals,
                    mean_signal_pressure=sum(tracker_totals) / len(tracker_totals) if tracker_totals else 0.0,
                )
            )
    return traces
