from __future__ import annotations

from cli.main import _SCENARIO_PRESETS
from evaluation.benchmark_runner.runner import (
    BenchmarkRunnerConfig,
    build_observation_for_actor,
    build_playable_benchmark_session,
    run_benchmark_lifecycle,
)
from agents.gateway.action_parser import parse_action_command
from agents.gateway.action_parser import parse_model_action_output
from agents.protocol.observation import Observation
from agents.protocol.serialization import canonical_json_dumps
from agents.llm_runtime import (
    _build_mode_instructions,
    _build_routed_loop_layers,
    build_angular_canonical_prompt_plan,
    build_benchmark_prompt,
    build_benchmark_prompt_from_payload,
    build_canonical_model_facing_observation_payload,
    build_expected_action_output_contract,
    build_fail_closed_action_submission,
    build_geometric_routed_prompt_plan,
    build_invalid_output_repair_prompt,
    build_legacy_router_backed_prompt_plan,
    build_persistent_session_model_facing_observation_payload,
    build_persistent_session_prompt,
    build_persistent_session_prompt_from_payload,
    build_runtime_telemetry,
    run_benchmark_llm_turn,
)


def _sample_observation() -> Observation:
    return Observation.from_dict(
        {
            "run_id": "tiny-fetch-quest-run",
            "step": 2,
            "location": "corridor",
            "description": "A narrow stone corridor.",
            "exits": ["east", "west"],
            "entities": [
                {"type": "npc", "name": "guard-1"},
                {"type": "item", "name": "golden-key"},
            ],
            "inventory": [],
            "health": 100,
            "messages": ["The guard blocks part of the hall."],
            "action_space": [
                "move east",
                "move west",
                "attack guard-1",
                "take golden-key",
                "look",
                "wait",
            ],
            "remaining_steps": 3,
            "protocol_version": "1.0",
        }
    )


def _event_to_action(event: dict[str, object]) -> str | None:
    event_type = event.get("event_type")
    actor_id = event.get("actor_id")
    payload = event.get("payload")
    if not isinstance(event_type, str) or not event_type.startswith("action_"):
        return None
    if actor_id != "agent-a" or not isinstance(payload, dict):
        return None
    verb = event_type.removeprefix("action_")
    if verb in {"wait", "look"}:
        return verb
    if verb == "move":
        return f"move {payload['direction']}"
    if verb in {"take", "drop", "use"}:
        return f"{verb} {payload['item_id']}"
    if verb == "attack":
        return f"attack {payload['target_id']}"
    if verb == "give":
        return f"give {payload['item_id']} {payload['target_id']}"
    raise AssertionError(f"unsupported event_type: {event_type}")


def _representative_slice_observation(scenario_name: str, *, target_step: int) -> Observation:
    result = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="test-audit",
            benchmark_id="test-audit",
            scenario=_SCENARIO_PRESETS[scenario_name],
            actor_ids=("agent-a",),
        )
    )
    actions = [
        action
        for action in (
            _event_to_action(event)
            for event in result.to_dict()["replay_artifact"]["events"]
        )
        if action is not None
    ]
    session = build_playable_benchmark_session(
        scenario=_SCENARIO_PRESETS[scenario_name],
        actor_ids=("agent-a",),
        run_id="test-audit",
    )
    for step, action in enumerate(actions):
        observation = build_observation_for_actor(
            session.world_state.get_snapshot(),
            actor_id="agent-a",
            run_id="test-audit",
            step=step,
            max_steps=session.scenario_initialization.max_steps,
        )
        if step == target_step:
            return observation
        action_request = parse_action_command(actor_id="agent-a", action=action)
        assert action_request.action_request is not None
        session.controller.step([action_request.action_request])
    raise AssertionError(f"target step {target_step} not reached for {scenario_name}")


def _build_pre_selection_geometric_prompt(observation: Observation) -> str:
    payload = build_canonical_model_facing_observation_payload(observation)
    routing_plan = build_geometric_routed_prompt_plan(payload)
    loop_layers = _build_routed_loop_layers(
        payload["observation"],
        payload["allowed_actions"],
        payload["allowed_targets"],
    )
    section_lines: list[str] = []
    for layer_name in routing_plan["prompt_section_order"]:
        layer_payload = loop_layers[layer_name]
        section_label = layer_name
        if layer_name in routing_plan["compressed_sections"]:
            section_label = f"{layer_name} (compressed)"
        section_lines.extend(
            (
                f"Loop layer: {section_label}",
                canonical_json_dumps(dict(layer_payload)),
                "",
            )
        )
    return "\n".join(
        (
            _build_mode_instructions("benchmark_single_turn"),
            "",
            "Invariant runtime guardrails:",
            "- Return exactly one JSON object and nothing else.",
            '- The JSON object may contain only one field: "action".',
            "- The action value must be a string copied exactly from allowed_actions.",
            "- Do not output prose, markdown, code fences, commentary, or extra keys.",
            "- If an action references a direction, item, or target, use only the allowed_targets listed below.",
            "- If your output is invalid, MUDBench will allow at most one repair attempt and then fail closed.",
            "",
            "Routing plan:",
            canonical_json_dumps(routing_plan),
            "",
            "Session frame:",
            canonical_json_dumps(dict(payload["session_frame"])),
            "",
            "Expected output contract:",
            canonical_json_dumps(dict(payload["response_format"])),
            "",
            "Allowed actions:",
            canonical_json_dumps({"allowed_actions": list(payload["allowed_actions"])}),
            "",
            "Allowed targets:",
            canonical_json_dumps({"allowed_targets": dict(payload["allowed_targets"])}),
            "",
            "Routed observation layers:",
            *section_lines,
            "Observation anchor:",
            canonical_json_dumps(payload["observation"]),
            "",
            'Return JSON now in the exact form {"action":"..."}',
        )
    )


def test_build_canonical_model_facing_observation_payload_is_deterministic() -> None:
    observation = _sample_observation()

    first = build_canonical_model_facing_observation_payload(observation, actor_id="agent-a")
    second = build_canonical_model_facing_observation_payload(observation, actor_id="agent-a")

    assert canonical_json_dumps(first) == canonical_json_dumps(second)
    assert first["mode"] == "benchmark_single_turn"
    assert first["actor_id"] == "agent-a"
    assert first["response_format"] == build_expected_action_output_contract()
    assert first["allowed_actions"] == list(observation.action_space)
    assert first["allowed_targets"] == {
        "move": ["east", "west"],
        "take": ["golden-key"],
        "attack": ["guard-1"],
    }
    assert first["observation"] == observation.to_dict()


def test_build_benchmark_prompt_is_deterministic() -> None:
    observation = _sample_observation()

    first = build_benchmark_prompt(observation)
    second = build_benchmark_prompt(observation)

    assert first == second
    assert "one isolated benchmark step" in first
    assert "Invariant runtime guardrails:" in first
    assert "Allowed actions:" in first
    assert "Allowed targets:" in first
    assert "fail closed" in first
    assert '"action"' in first
    assert canonical_json_dumps(observation.to_dict()) in first
    assert "Routing plan:" not in first


def test_context_pressure_baseline_prompt_keeps_full_observation_path() -> None:
    observation = _representative_slice_observation(
        "tiny-context-pressure",
        target_step=9,
    )

    prompt = build_benchmark_prompt(observation)

    assert canonical_json_dumps(observation.to_dict()) in prompt
    assert "Routing plan:" not in prompt
    assert "Angular canonical plan:" not in prompt
    assert "Legacy router-backed proxy plan:" not in prompt
    assert "Observation summary:" not in prompt


def test_build_benchmark_prompt_from_payload_matches_observation_helper() -> None:
    observation = _sample_observation()
    payload = build_canonical_model_facing_observation_payload(observation)

    assert build_benchmark_prompt_from_payload(payload) == build_benchmark_prompt(observation)


def test_build_invalid_output_repair_prompt_is_deterministic_for_invalid_case() -> None:
    observation = _sample_observation()
    parse_result = parse_model_action_output(
        raw_output="I would attack the guard first.",
        action_space=observation.action_space,
    )

    assert parse_result.accepted is False
    assert parse_result.reason == "invalid_json"

    first = build_invalid_output_repair_prompt(
        failure_reason=parse_result.reason,
        action_space=observation.action_space,
        allowed_targets={"attack": ["guard-1"]},
    )
    second = build_invalid_output_repair_prompt(
        failure_reason=parse_result.reason,
        action_space=observation.action_space,
        allowed_targets={"attack": ["guard-1"]},
    )

    assert first == second
    assert "Your previous response was invalid for MUDBench." in first
    assert "- invalid_json" in first
    assert canonical_json_dumps({"action_space": list(observation.action_space)}) in first
    assert canonical_json_dumps({"allowed_targets": {"attack": ["guard-1"]}}) in first


def test_build_persistent_session_prompt_explicitly_differs_from_benchmark_prompt() -> None:
    observation = _sample_observation()

    benchmark_prompt = build_benchmark_prompt(observation, actor_id="agent-a")
    persistent_prompt = build_persistent_session_prompt(observation, actor_id="agent-a")
    persistent_payload = build_persistent_session_model_facing_observation_payload(
        observation,
        actor_id="agent-a",
    )

    assert persistent_prompt != benchmark_prompt
    assert "persistent local-process session" in persistent_prompt
    assert "caller_managed_session_history" in persistent_prompt
    assert (
        build_persistent_session_prompt_from_payload(persistent_payload) == persistent_prompt
    )


def test_build_geometric_routed_prompt_plan_is_deterministic() -> None:
    observation = _sample_observation()
    payload = build_canonical_model_facing_observation_payload(observation)

    first = build_geometric_routed_prompt_plan(payload)
    second = build_geometric_routed_prompt_plan(payload)

    assert first == second
    assert first["engine"] == "geometric-routed"
    assert first["dominant_loop"] == "immediate_action"
    assert first["secondary_loops"] == [
        "temporal_world",
        "local_objective",
        "multi_agent",
        "persistence",
    ]
    assert first["prompt_section_order"] == [
        "immediate_action",
        "temporal_world",
        "local_objective",
        "multi_agent",
        "persistence",
    ]
    assert first["compressed_sections"] == ["persistence"]
    assert first["reasoning_pressure_tag"] == "execution_pressure"
    assert first["routing_context"]["pressure_features"]["actionable_hazard_pressure"] > 0


def test_build_geometric_routed_benchmark_prompt_differs_in_controlled_way() -> None:
    observation = _sample_observation()

    baseline = build_benchmark_prompt(observation)
    routed = build_benchmark_prompt(observation, prompt_engine="geometric-routed")

    assert routed != baseline
    assert "Invariant runtime guardrails:" in routed
    assert "Routing plan:" in routed
    assert "Loop layer: immediate_action" in routed
    assert "Loop layer: temporal_world (summary)" in routed
    assert "Loop layer: local_objective" not in routed
    assert "Loop layer: persistence" not in routed
    assert "Loop layer: multi_agent" not in routed
    assert "Observation summary:" in routed
    assert "Observation anchor:" not in routed
    assert "Allowed actions:" in routed
    assert '"action"' in routed


def test_build_angular_canonical_prompt_plan_is_deterministic() -> None:
    observation = _sample_observation()
    payload = build_canonical_model_facing_observation_payload(observation)

    first = build_angular_canonical_prompt_plan(payload, router_variant="angular-hopf-trans")
    second = build_angular_canonical_prompt_plan(payload, router_variant="angular-hopf-trans")

    assert first == second
    assert first["engine"] == "angular-canonical"
    assert first["router_variant"] == "angular-hopf-trans"
    assert first["router_family"] == "canonical_single_shell_angular_hopf_router"
    assert first["single_shell"] is True
    assert first["coordinate_adapter_contract"] == "mudbench_prompt_state_v2"
    assert first["router_k"] == 8
    assert first["prompt_section_order"][0] == "immediate_action"
    assert first["reasoning_pressure_tag"].startswith("angular_hopf_trans:")
    assert first["routing_context"]["pressure_features"]["hostile_actor_pressure"] > 0
    assert (
        first["routing_coordinate_adapter"]["pressure_features"]["actionable_hazard_pressure"]
        > 0
    )


def test_build_angular_canonical_variants_are_distinct_and_wired() -> None:
    observation = _sample_observation()
    payload = build_canonical_model_facing_observation_payload(observation)

    base_plan = build_angular_canonical_prompt_plan(payload, router_variant="angular-hopf-base")
    trans_plan = build_angular_canonical_prompt_plan(payload, router_variant="angular-hopf-trans")

    assert base_plan["router_variant"] == "angular-hopf-base"
    assert trans_plan["router_variant"] == "angular-hopf-trans"
    assert base_plan["sector_bins"] != trans_plan["sector_bins"]
    assert base_plan["compressed_sections"] != trans_plan["compressed_sections"]
    assert base_plan["routing_coordinate_adapter"] == trans_plan["routing_coordinate_adapter"]


def test_build_angular_canonical_benchmark_prompt_differs_in_controlled_way() -> None:
    observation = _sample_observation()

    baseline = build_benchmark_prompt(observation)
    angular = build_benchmark_prompt(
        observation,
        prompt_engine="angular-canonical",
        router_variant="angular-hopf-trans",
    )

    assert angular != baseline
    assert "Invariant runtime guardrails:" in angular
    assert "Angular canonical plan:" in angular
    assert '"router_variant":"angular-hopf-trans"' in angular
    assert "Angular layer: immediate_action" in angular
    assert "Angular layer: temporal_world (summary)" in angular
    assert "Angular layer: persistence" not in angular
    assert "Angular layer: multi_agent" not in angular
    assert "Angular layer: local_objective" not in angular
    assert "Observation summary:" in angular
    assert "Observation anchor:" not in angular
    assert '"action"' in angular


def test_build_legacy_router_backed_prompt_plan_is_deterministic() -> None:
    observation = _sample_observation()
    payload = build_canonical_model_facing_observation_payload(observation)

    first = build_legacy_router_backed_prompt_plan(
        payload,
        router_variant="legacy-phase4d_hopf_transport",
    )
    second = build_legacy_router_backed_prompt_plan(
        payload,
        router_variant="legacy-phase4d_hopf_transport",
    )

    assert first == second
    assert first["engine"] == "legacy-router-backed"
    assert first["router_variant"] == "legacy-phase4d_hopf_transport"
    assert first["router_family"] == "legacy_phi_fibonacci_prompt_router"
    assert first["routing_context"]["pressure_features"]["hostile_actor_pressure"] > 0


def test_build_legacy_router_backed_benchmark_prompt_differs_in_controlled_way() -> None:
    observation = _sample_observation()

    baseline = build_benchmark_prompt(observation)
    legacy_router = build_benchmark_prompt(
        observation,
        prompt_engine="legacy-router-backed",
        router_variant="legacy-phase4d_hopf_product_phase",
    )

    assert legacy_router != baseline
    assert "Invariant runtime guardrails:" in legacy_router
    assert "Legacy router-backed proxy plan:" in legacy_router
    assert '"router_variant":"legacy-phase4d_hopf_product_phase"' in legacy_router
    assert "Legacy router layer: immediate_action" in legacy_router
    assert "Legacy router layer: local_objective (summary)" in legacy_router
    assert "Legacy router layer: persistence" not in legacy_router
    assert "Legacy router layer: temporal_world" not in legacy_router
    assert "Observation summary:" in legacy_router
    assert "Observation anchor:" not in legacy_router
    assert '"action"' in legacy_router


def test_context_pressure_enriched_adapter_distinguishes_entry_and_gate_states() -> None:
    entry_observation = _representative_slice_observation(
        "tiny-context-pressure",
        target_step=0,
    )
    gate_observation = _representative_slice_observation(
        "tiny-context-pressure",
        target_step=9,
    )

    entry_plan = build_angular_canonical_prompt_plan(
        build_canonical_model_facing_observation_payload(entry_observation),
        router_variant="angular-hopf-base",
    )
    gate_plan = build_angular_canonical_prompt_plan(
        build_canonical_model_facing_observation_payload(gate_observation),
        router_variant="angular-hopf-base",
    )

    entry_features = entry_plan["routing_context"]["pressure_features"]
    gate_features = gate_plan["routing_context"]["pressure_features"]

    assert gate_features["dependency_gate_pressure"] > entry_features["dependency_gate_pressure"]
    assert gate_features["location_sensitive_use_pressure"] > entry_features["location_sensitive_use_pressure"]
    assert gate_features["resource_scarcity_pressure"] > entry_features["resource_scarcity_pressure"]
    assert (
        gate_plan["routing_coordinate_adapter"]["bundle_scores"]
        != entry_plan["routing_coordinate_adapter"]["bundle_scores"]
    )


def test_context_pressure_routed_families_share_enriched_pressure_features() -> None:
    observation = _representative_slice_observation(
        "tiny-context-pressure",
        target_step=9,
    )
    payload = build_canonical_model_facing_observation_payload(observation)

    geometric_plan = build_geometric_routed_prompt_plan(payload)
    angular_plan = build_angular_canonical_prompt_plan(
        payload,
        router_variant="angular-hopf-base",
    )
    legacy_plan = build_legacy_router_backed_prompt_plan(
        payload,
        router_variant="legacy-phase4d_hopf_transport",
    )

    assert geometric_plan["routing_context"]["pressure_features"] == angular_plan["routing_context"]["pressure_features"]
    assert geometric_plan["routing_context"]["pressure_features"] == legacy_plan["routing_context"]["pressure_features"]
    assert geometric_plan["engine"] == "geometric-routed"
    assert angular_plan["engine"] == "angular-canonical"
    assert legacy_plan["engine"] == "legacy-router-backed"
    assert angular_plan["coordinate_adapter_contract"] == "mudbench_prompt_state_v2"


def test_context_pressure_routed_prompt_surfaces_enriched_pressure_features() -> None:
    observation = _representative_slice_observation(
        "tiny-context-pressure",
        target_step=9,
    )

    angular_prompt = build_benchmark_prompt(
        observation,
        prompt_engine="angular-canonical",
        router_variant="angular-hopf-base",
    )

    assert '"dependency_gate_pressure"' in angular_prompt
    assert '"location_sensitive_use_pressure"' in angular_prompt
    assert '"resource_scarcity_pressure"' in angular_prompt
    assert '"adapter_schema":"mudbench_prompt_state_v2"' in angular_prompt


def test_routed_prompt_length_is_materially_reduced_on_representative_slice() -> None:
    observation = _representative_slice_observation(
        "tiny-hazard-route",
        target_step=5,
    )

    pre_selection_routed = _build_pre_selection_geometric_prompt(observation)
    routed = build_benchmark_prompt(observation, prompt_engine="geometric-routed")

    assert len(routed) <= len(pre_selection_routed) - 300


def test_run_benchmark_llm_turn_accepts_valid_first_output() -> None:
    observation = _sample_observation()
    prompts: list[str] = []

    def _model_completion(prompt: str) -> str:
        prompts.append(prompt)
        return '{"action":"take golden-key"}'

    result = run_benchmark_llm_turn(observation, model_completion=_model_completion)

    assert len(prompts) == 1
    assert result.prompt == prompts[0]
    assert result.prompt_engine == "baseline"
    assert result.routing_plan is None
    assert result.repair_prompt is None
    assert result.initial_parse_result.accepted is True
    assert result.action_submission.action == "take golden-key"
    assert result.used_fail_closed_fallback is False
    assert result.fail_closed_reason is None
    assert result.runtime_telemetry == {
        "repair_used": False,
        "fail_closed_used": False,
        "final_parse_status": "accepted_initial",
        "failure_reason": None,
    }
    assert result.model_facing_observation_payload["observation"] == observation.to_dict()


def test_run_benchmark_llm_turn_repairs_one_invalid_first_output() -> None:
    observation = _sample_observation()
    prompts: list[str] = []
    responses = iter(("not-json", '{"action":"move east"}'))

    def _model_completion(prompt: str) -> str:
        prompts.append(prompt)
        return next(responses)

    result = run_benchmark_llm_turn(observation, model_completion=_model_completion)

    assert len(prompts) == 2
    assert result.prompt_engine == "baseline"
    assert result.initial_parse_result.accepted is False
    assert result.initial_parse_result.reason == "invalid_json"
    assert result.repair_prompt == prompts[1]
    assert result.repaired_parse_result is not None
    assert result.repaired_parse_result.accepted is True
    assert result.action_submission.action == "move east"
    assert result.used_fail_closed_fallback is False
    assert result.runtime_telemetry == {
        "repair_used": True,
        "fail_closed_used": False,
        "final_parse_status": "accepted_after_repair",
        "failure_reason": "invalid_json",
    }


def test_run_benchmark_llm_turn_fails_closed_after_one_invalid_repair_attempt() -> None:
    observation = _sample_observation()
    prompts: list[str] = []

    def _model_completion(prompt: str) -> str:
        prompts.append(prompt)
        return "still-not-json"

    result = run_benchmark_llm_turn(observation, model_completion=_model_completion)

    assert len(prompts) == 2
    assert result.prompt_engine == "baseline"
    assert result.initial_parse_result.reason == "invalid_json"
    assert result.repaired_parse_result is not None
    assert result.repaired_parse_result.reason == "invalid_json"
    assert result.used_fail_closed_fallback is True
    assert result.fail_closed_reason == "model_output_rejected_after_repair:invalid_json"
    assert result.action_submission.action == "wait"
    assert result.runtime_telemetry == {
        "repair_used": True,
        "fail_closed_used": True,
        "final_parse_status": "fail_closed",
        "failure_reason": "model_output_rejected_after_repair:invalid_json",
    }


def test_build_fail_closed_action_submission_prefers_wait_then_look_then_first_action() -> None:
    assert build_fail_closed_action_submission(("move east", "wait")).action == "wait"
    assert build_fail_closed_action_submission(("move east", "look")).action == "look"
    assert build_fail_closed_action_submission(("move east", "take relic")).action == "move east"


def test_build_runtime_telemetry_includes_provider_fields_when_present() -> None:
    telemetry = build_runtime_telemetry(
        initial_parse_result=parse_model_action_output(
            raw_output='{"action":"wait"}',
            action_space=("wait",),
        ),
        provider_runtime={
            "provider_name": "openai-chat-completions",
            "provider_request_count": 1,
            "provider_latency_ms": 12.5,
        },
    )

    assert telemetry == {
        "repair_used": False,
        "fail_closed_used": False,
        "final_parse_status": "accepted_initial",
        "failure_reason": None,
        "provider_name": "openai-chat-completions",
        "provider_request_count": 1,
        "provider_latency_ms": 12.5,
    }


def test_run_benchmark_llm_turn_supports_geometric_routed_prompt_engine() -> None:
    observation = _sample_observation()
    prompts: list[str] = []

    def _model_completion(prompt: str) -> str:
        prompts.append(prompt)
        return '{"action":"move east"}'

    result = run_benchmark_llm_turn(
        observation,
        model_completion=_model_completion,
        prompt_engine="geometric-routed",
    )

    assert len(prompts) == 1
    assert result.prompt_engine == "geometric-routed"
    assert result.routing_plan is not None
    assert result.routing_plan["engine"] == "geometric-routed"
    assert "Routing plan:" in prompts[0]
    assert result.action_submission.action == "move east"


def test_run_benchmark_llm_turn_supports_angular_canonical_prompt_engine() -> None:
    observation = _sample_observation()
    prompts: list[str] = []

    def _model_completion(prompt: str) -> str:
        prompts.append(prompt)
        return '{"action":"move east"}'

    result = run_benchmark_llm_turn(
        observation,
        model_completion=_model_completion,
        prompt_engine="angular-canonical",
        router_variant="angular-hopf-trans",
    )

    assert len(prompts) == 1
    assert result.prompt_engine == "angular-canonical"
    assert result.router_variant == "angular-hopf-trans"
    assert result.routing_plan is not None
    assert result.routing_plan["engine"] == "angular-canonical"
    assert result.routing_plan["router_variant"] == "angular-hopf-trans"
    assert "Angular canonical plan:" in prompts[0]
    assert '"router_variant":"angular-hopf-trans"' in prompts[0]
    assert result.action_submission.action == "move east"


def test_run_benchmark_llm_turn_supports_legacy_router_backed_prompt_engine() -> None:
    observation = _sample_observation()
    prompts: list[str] = []

    def _model_completion(prompt: str) -> str:
        prompts.append(prompt)
        return '{"action":"move east"}'

    result = run_benchmark_llm_turn(
        observation,
        model_completion=_model_completion,
        prompt_engine="legacy-router-backed",
        router_variant="legacy-phase4d_hopf_product_phase",
    )

    assert len(prompts) == 1
    assert result.prompt_engine == "legacy-router-backed"
    assert result.router_variant == "legacy-phase4d_hopf_product_phase"
    assert result.routing_plan is not None
    assert result.routing_plan["engine"] == "legacy-router-backed"
    assert result.routing_plan["router_variant"] == "legacy-phase4d_hopf_product_phase"
    assert "Legacy router-backed proxy plan:" in prompts[0]
    assert '"router_variant":"legacy-phase4d_hopf_product_phase"' in prompts[0]
    assert result.action_submission.action == "move east"
