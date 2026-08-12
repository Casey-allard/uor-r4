"""Minimal direct-provider benchmark gameplay runner."""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from collections.abc import Callable, Mapping, Sequence
from dataclasses import replace
from dataclasses import dataclass
from pathlib import Path
from time import monotonic
from typing import TYPE_CHECKING, Any

_SRC_PATH = Path(__file__).resolve().parents[1]
if str(_SRC_PATH) not in sys.path:
    sys.path.insert(0, str(_SRC_PATH))

if TYPE_CHECKING:
    from agents.llm_runtime import BenchmarkLLMTurnResult
    from agents.protocol.observation import Observation

_SUPPORTED_DIRECT_PROVIDER = "openai-chat-completions"
_DEFAULT_OPENAI_BASE_URL = "https://api.openai.com/v1/chat/completions"
_DEFAULT_TIMEOUT_SECONDS = 30.0
_OPENAI_API_KEY_ENV = "MUDBENCH_OPENAI_API_KEY"
_OPENAI_MODEL_ENV = "MUDBENCH_OPENAI_MODEL"
_OPENAI_BASE_URL_ENV = "MUDBENCH_OPENAI_BASE_URL"
_RUNTIME_TELEMETRY_FILENAME_TEMPLATE = "{run_id}__step_{step:06d}__{actor_id}.llm_runtime_telemetry.json"
_PROMPT_DUMP_FILENAME_TEMPLATE = "{run_id}__step_{step:06d}__{actor_id}.direct_provider_prompt_dump.json"


@dataclass(frozen=True, slots=True)
class DirectProviderConfig:
    """Bounded direct-provider configuration."""

    provider: str
    model: str
    api_key: str
    base_url: str
    timeout_seconds: float = _DEFAULT_TIMEOUT_SECONDS


def resolve_direct_provider_config(
    *,
    provider: str,
    model: str | None = None,
    env: Mapping[str, str] | None = None,
) -> DirectProviderConfig:
    """Resolve one bounded direct-provider configuration from env plus explicit args."""
    resolved_env = dict(os.environ if env is None else env)
    if provider != _SUPPORTED_DIRECT_PROVIDER:
        raise ValueError(f"unsupported_direct_provider:{provider}")

    api_key = resolved_env.get(_OPENAI_API_KEY_ENV, "").strip()
    if not api_key:
        raise ValueError(f"direct_provider_missing_api_key:{_OPENAI_API_KEY_ENV}")

    resolved_model = (model or resolved_env.get(_OPENAI_MODEL_ENV, "")).strip()
    if not resolved_model:
        raise ValueError(f"direct_provider_missing_model:{_OPENAI_MODEL_ENV}")

    resolved_base_url = (
        resolved_env.get(_OPENAI_BASE_URL_ENV, _DEFAULT_OPENAI_BASE_URL).strip()
        or _DEFAULT_OPENAI_BASE_URL
    )

    return DirectProviderConfig(
        provider=provider,
        model=resolved_model,
        api_key=api_key,
        base_url=resolved_base_url,
    )


def build_direct_provider_command(
    config: DirectProviderConfig,
    *,
    python_executable: str,
    prompt_engine: str = "baseline",
    router_variant: str | None = None,
) -> tuple[str, ...]:
    """Build the deterministic subprocess command for the direct-provider runner."""
    if not isinstance(python_executable, str) or not python_executable:
        raise ValueError("python_executable must be a non-empty string")
    command = [
        python_executable,
        str(Path(__file__).resolve()),
        "--provider",
        config.provider,
        "--model",
        config.model,
        "--base-url",
        config.base_url,
    ]
    if prompt_engine != "baseline":
        command.extend(("--prompt-engine", prompt_engine))
    if prompt_engine in {"angular-canonical", "legacy-router-backed"} and router_variant is not None:
        command.extend(("--router-variant", router_variant))
    return tuple(command)


def build_openai_chat_completions_request(
    *,
    prompt: str,
    config: DirectProviderConfig,
) -> dict[str, Any]:
    """Build one deterministic OpenAI chat-completions request payload."""
    if not isinstance(prompt, str) or not prompt:
        raise ValueError("prompt must be a non-empty string")
    if config.provider != _SUPPORTED_DIRECT_PROVIDER:
        raise ValueError(f"unsupported_direct_provider:{config.provider}")

    body = {
        "model": config.model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0,
    }
    return {
        "url": config.base_url,
        "headers": {
            "Authorization": f"Bearer {config.api_key}",
            "Content-Type": "application/json",
        },
        "body": body,
        "timeout_seconds": config.timeout_seconds,
    }


def request_openai_chat_completions(
    *,
    prompt: str,
    config: DirectProviderConfig,
    transport: Callable[[dict[str, Any]], dict[str, Any]] | None = None,
) -> str:
    """Request one completion from the bounded direct-provider path."""
    request_payload = build_openai_chat_completions_request(prompt=prompt, config=config)
    response_payload = (
        _default_http_transport(request_payload) if transport is None else transport(request_payload)
    )
    if not isinstance(response_payload, dict):
        raise ValueError("direct_provider_invalid_response:not_object")

    choices = response_payload.get("choices")
    if not isinstance(choices, list) or len(choices) == 0:
        raise ValueError("direct_provider_invalid_response:missing_choices")
    first_choice = choices[0]
    if not isinstance(first_choice, dict):
        raise ValueError("direct_provider_invalid_response:invalid_choice")
    message = first_choice.get("message")
    if not isinstance(message, dict):
        raise ValueError("direct_provider_invalid_response:missing_message")
    content = message.get("content")
    if not isinstance(content, str) or not content:
        raise ValueError("direct_provider_invalid_response:missing_content")
    return content


def run_direct_provider_benchmark_turn(
    observation: "Observation",
    *,
    config: DirectProviderConfig,
    completion_request: Callable[[str, DirectProviderConfig], str] | None = None,
    prompt_engine: str = "baseline",
    router_variant: str | None = None,
) -> "BenchmarkLLMTurnResult":
    """Execute one benchmark turn through the direct-provider completion path."""
    from agents.llm_runtime import run_benchmark_llm_turn
    from agents.protocol.observation import Observation

    if not isinstance(observation, Observation):
        raise ValueError("observation must be an Observation")

    provider_request_count = 0
    provider_latency_ms = 0.0

    def _complete(prompt: str) -> str:
        nonlocal provider_request_count
        nonlocal provider_latency_ms
        started_at = monotonic()
        if completion_request is not None:
            content = completion_request(prompt, config)
        else:
            content = request_openai_chat_completions(prompt=prompt, config=config)
        provider_request_count += 1
        provider_latency_ms += (monotonic() - started_at) * 1000.0
        return content

    result = run_benchmark_llm_turn(
        observation,
        model_completion=_complete,
        prompt_engine=prompt_engine,
        router_variant=router_variant,
    )
    return replace(
        result,
        runtime_telemetry=_build_direct_provider_runtime_telemetry(
            base_runtime_telemetry=result.runtime_telemetry,
            provider=config.provider,
            provider_request_count=provider_request_count,
            provider_latency_ms=provider_latency_ms,
        ),
    )


def _build_direct_provider_runtime_telemetry(
    *,
    base_runtime_telemetry: Mapping[str, Any] | None,
    provider: str,
    provider_request_count: int,
    provider_latency_ms: float,
) -> dict[str, Any]:
    if base_runtime_telemetry is None:
        raise ValueError("base_runtime_telemetry must be present")
    if not isinstance(provider, str) or not provider:
        raise ValueError("provider must be a non-empty string")
    if not isinstance(provider_request_count, int) or provider_request_count < 0:
        raise ValueError("provider_request_count must be a non-negative integer")
    if not isinstance(provider_latency_ms, (int, float)) or provider_latency_ms < 0:
        raise ValueError("provider_latency_ms must be a non-negative number")

    telemetry = dict(base_runtime_telemetry)
    telemetry["provider_name"] = provider
    telemetry["provider_request_count"] = provider_request_count
    telemetry["provider_latency_ms"] = round(float(provider_latency_ms), 3)
    return telemetry


def _build_runtime_telemetry_output_path(
    *,
    telemetry_dir: Path,
    run_id: str,
    step: int,
    actor_id: str,
) -> Path:
    return telemetry_dir / _RUNTIME_TELEMETRY_FILENAME_TEMPLATE.format(
        run_id=run_id,
        step=step,
        actor_id=actor_id,
    )


def _build_prompt_dump_output_path(
    *,
    prompt_dump_dir: Path,
    run_id: str,
    step: int,
    actor_id: str,
) -> Path:
    return prompt_dump_dir / _PROMPT_DUMP_FILENAME_TEMPLATE.format(
        run_id=run_id,
        step=step,
        actor_id=actor_id,
    )


def _write_runtime_telemetry_if_configured(
    *,
    telemetry_dir: str | None,
    telemetry_actor_id: str | None,
    observation: "Observation",
    runtime_telemetry: Mapping[str, Any] | None,
) -> None:
    if telemetry_dir is None or telemetry_actor_id is None or runtime_telemetry is None:
        return
    if not isinstance(telemetry_dir, str) or not telemetry_dir:
        raise ValueError("telemetry_dir must be a non-empty string when provided")
    if not isinstance(telemetry_actor_id, str) or not telemetry_actor_id:
        raise ValueError("telemetry_actor_id must be a non-empty string when provided")

    output_dir = Path(telemetry_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    payload = {
        "telemetry_schema": "llm_runtime_turn_v1",
        "actor_id": telemetry_actor_id,
        "run_id": observation.run_id,
        "step": observation.step,
        "repair_used": bool(runtime_telemetry.get("repair_used")),
        "fail_closed_used": bool(runtime_telemetry.get("fail_closed_used")),
        "final_parse_status": str(runtime_telemetry.get("final_parse_status", "")),
    }
    failure_reason = runtime_telemetry.get("failure_reason")
    if isinstance(failure_reason, str) and failure_reason:
        payload["failure_reason"] = failure_reason
    provider_name = runtime_telemetry.get("provider_name")
    if isinstance(provider_name, str) and provider_name:
        payload["provider_name"] = provider_name
    provider_request_count = runtime_telemetry.get("provider_request_count")
    if isinstance(provider_request_count, int):
        payload["provider_request_count"] = provider_request_count
    provider_latency_ms = runtime_telemetry.get("provider_latency_ms")
    if isinstance(provider_latency_ms, (int, float)):
        payload["provider_latency_ms"] = round(float(provider_latency_ms), 3)

    output_path = _build_runtime_telemetry_output_path(
        telemetry_dir=output_dir,
        run_id=observation.run_id,
        step=observation.step,
        actor_id=telemetry_actor_id,
    )
    output_path.write_text(
        json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n",
        encoding="utf-8",
    )


def _serialize_parse_result(parse_result: Any) -> dict[str, Any]:
    if parse_result is None:
        return {}
    payload: dict[str, Any] = {"accepted": bool(parse_result.accepted)}
    if parse_result.reason is not None:
        payload["reason"] = parse_result.reason
    if parse_result.action_submission is not None:
        payload["action_submission"] = parse_result.action_submission.to_dict()
    return payload


def _write_prompt_dump_if_configured(
    *,
    prompt_dump_dir: str | None,
    prompt_dump_actor_id: str | None,
    prompt_dump_scenario_id: str | None,
    observation: "Observation",
    result: "BenchmarkLLMTurnResult",
    provider_turns: Sequence[Mapping[str, Any]],
) -> None:
    if prompt_dump_dir is None and prompt_dump_actor_id is None and prompt_dump_scenario_id is None:
        return
    if prompt_dump_dir is None:
        raise ValueError("prompt_dump_actor_id_requires_prompt_dump_dir")
    if prompt_dump_actor_id is None:
        raise ValueError("prompt_dump_dir_requires_prompt_dump_actor_id")
    if not isinstance(prompt_dump_dir, str) or not prompt_dump_dir:
        raise ValueError("prompt_dump_dir must be a non-empty string when provided")
    if not isinstance(prompt_dump_actor_id, str) or not prompt_dump_actor_id:
        raise ValueError("prompt_dump_actor_id must be a non-empty string when provided")
    if prompt_dump_scenario_id is not None and (
        not isinstance(prompt_dump_scenario_id, str) or not prompt_dump_scenario_id
    ):
        raise ValueError("prompt_dump_scenario_id must be a non-empty string when provided")

    output_dir = Path(prompt_dump_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    normalized_turns = [dict(turn) for turn in provider_turns]
    payload: dict[str, Any] = {
        "dump_schema": "direct_provider_prompt_dump_v1",
        "actor_id": prompt_dump_actor_id,
        "run_id": observation.run_id,
        "step": observation.step,
        "prompt_engine": result.prompt_engine,
        "router_variant": result.router_variant,
        "prompt_text": result.prompt,
        "repair_prompt_text": result.repair_prompt,
        "provider_turns": normalized_turns,
        "initial_parse_result": _serialize_parse_result(result.initial_parse_result),
        "repaired_parse_result": _serialize_parse_result(result.repaired_parse_result),
        "final_action": result.action_submission.to_dict(),
        "used_fail_closed_fallback": result.used_fail_closed_fallback,
        "fail_closed_reason": result.fail_closed_reason,
    }
    if prompt_dump_scenario_id is not None:
        payload["scenario_id"] = prompt_dump_scenario_id
    if len(normalized_turns) >= 1:
        payload["raw_provider_response_text"] = normalized_turns[0].get("raw_provider_response_text")
    if len(normalized_turns) >= 2:
        payload["repair_raw_provider_response_text"] = normalized_turns[1].get(
            "raw_provider_response_text"
        )
    if result.runtime_telemetry is not None:
        provider_name = result.runtime_telemetry.get("provider_name")
        if isinstance(provider_name, str) and provider_name:
            payload["provider_name"] = provider_name
        provider_request_count = result.runtime_telemetry.get("provider_request_count")
        if isinstance(provider_request_count, int):
            payload["provider_request_count"] = provider_request_count
        provider_latency_ms = result.runtime_telemetry.get("provider_latency_ms")
        if isinstance(provider_latency_ms, (int, float)):
            payload["provider_latency_ms"] = round(float(provider_latency_ms), 3)
        final_parse_status = result.runtime_telemetry.get("final_parse_status")
        if isinstance(final_parse_status, str) and final_parse_status:
            payload["final_parse_status"] = final_parse_status

    output_path = _build_prompt_dump_output_path(
        prompt_dump_dir=output_dir,
        run_id=observation.run_id,
        step=observation.step,
        actor_id=prompt_dump_actor_id,
    )
    output_path.write_text(
        json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n",
        encoding="utf-8",
    )


def _default_http_transport(request_payload: dict[str, Any]) -> dict[str, Any]:
    from agents.protocol.serialization import canonical_json_dumps

    data = canonical_json_dumps(request_payload["body"]).encode("utf-8")
    request = urllib.request.Request(
        request_payload["url"],
        data=data,
        headers=request_payload["headers"],
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=float(request_payload["timeout_seconds"])) as response:
            response_bytes = response.read()
    except urllib.error.HTTPError as exc:
        raise ValueError(f"direct_provider_http_error:{exc.code}") from exc
    except urllib.error.URLError as exc:
        raise ValueError(f"direct_provider_connection_error:{exc.reason}") from exc

    try:
        payload = json.loads(response_bytes.decode("utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError("direct_provider_invalid_response:invalid_json") from exc
    if not isinstance(payload, dict):
        raise ValueError("direct_provider_invalid_response:not_object")
    return payload


def _read_observation() -> "Observation":
    from agents.protocol.observation import Observation

    raw_line = sys.stdin.readline()
    if not raw_line:
        raise ValueError("missing_observation_payload")
    payload = json.loads(raw_line)
    if not isinstance(payload, dict):
        raise ValueError("observation_payload_must_be_object")
    return Observation.from_dict(payload)


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="direct_provider_runner", add_help=False)
    parser.add_argument(
        "--provider",
        choices=(_SUPPORTED_DIRECT_PROVIDER,),
        required=True,
        help="Bounded direct-provider mode to execute.",
    )
    parser.add_argument(
        "--model",
        required=True,
        help="Provider model identifier.",
    )
    parser.add_argument(
        "--base-url",
        default=_DEFAULT_OPENAI_BASE_URL,
        help="Provider HTTP endpoint.",
    )
    parser.add_argument(
        "--prompt-engine",
        choices=("baseline", "geometric-routed", "angular-canonical", "legacy-router-backed"),
        default="baseline",
        help="Prompt assembly mode for the bounded provider-backed turn.",
    )
    parser.add_argument(
        "--router-variant",
        choices=(
            "angular-hopf-base",
            "angular-hopf-trans",
            "legacy-phase4d_hopf_base",
            "legacy-phase4d_hopf_transport",
            "legacy-phase4d_hopf_product_phase",
        ),
        default=None,
        help="Router variant to use when a variant-aware prompt engine is selected.",
    )
    parser.add_argument(
        "--telemetry-dir",
        default=None,
        help="Optional directory for writing deterministic runtime telemetry sidecar records.",
    )
    parser.add_argument(
        "--telemetry-actor-id",
        default=None,
        help="Optional actor identifier to include in runtime telemetry sidecar records.",
    )
    parser.add_argument(
        "--prompt-dump-dir",
        default=None,
        help="Optional directory for writing deterministic prompt/raw-response dump sidecar records.",
    )
    parser.add_argument(
        "--prompt-dump-actor-id",
        default=None,
        help="Optional actor identifier to include in prompt/raw-response dump sidecar records.",
    )
    parser.add_argument(
        "--prompt-dump-scenario-id",
        default=None,
        help="Optional scenario identifier to include in prompt/raw-response dump sidecar records.",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    try:
        args = _build_parser().parse_args(argv)
        if args.prompt_dump_dir is None and args.prompt_dump_actor_id is not None:
            raise ValueError("prompt_dump_actor_id_requires_prompt_dump_dir")
        if args.prompt_dump_dir is None and args.prompt_dump_scenario_id is not None:
            raise ValueError("prompt_dump_scenario_id_requires_prompt_dump_dir")
        if args.prompt_dump_dir is not None and args.prompt_dump_actor_id is None:
            raise ValueError("prompt_dump_dir_requires_prompt_dump_actor_id")
        observation = _read_observation()
        config = resolve_direct_provider_config(
            provider=args.provider,
            model=args.model,
        )
        if args.base_url != _DEFAULT_OPENAI_BASE_URL:
            config = DirectProviderConfig(
                provider=config.provider,
                model=config.model,
                api_key=config.api_key,
                base_url=args.base_url,
                timeout_seconds=config.timeout_seconds,
            )
        provider_turns: list[dict[str, Any]] = []

        def _capturing_completion_request(prompt: str, runtime_config: DirectProviderConfig) -> str:
            raw_provider_response_text = request_openai_chat_completions(
                prompt=prompt,
                config=runtime_config,
            )
            provider_turns.append(
                {
                    "phase": "initial" if len(provider_turns) == 0 else "repair",
                    "prompt_text": prompt,
                    "raw_provider_response_text": raw_provider_response_text,
                }
            )
            return raw_provider_response_text

        result = run_direct_provider_benchmark_turn(
            observation,
            config=config,
            completion_request=_capturing_completion_request,
            prompt_engine=args.prompt_engine,
            router_variant=args.router_variant,
        )
        _write_runtime_telemetry_if_configured(
            telemetry_dir=args.telemetry_dir,
            telemetry_actor_id=args.telemetry_actor_id,
            observation=observation,
            runtime_telemetry=result.runtime_telemetry,
        )
        _write_prompt_dump_if_configured(
            prompt_dump_dir=args.prompt_dump_dir,
            prompt_dump_actor_id=args.prompt_dump_actor_id,
            prompt_dump_scenario_id=args.prompt_dump_scenario_id,
            observation=observation,
            result=result,
            provider_turns=provider_turns,
        )
        sys.stdout.write(
            json.dumps(
                result.action_submission.to_dict(),
                sort_keys=True,
                separators=(",", ":"),
                ensure_ascii=True,
            )
        )
        sys.stdout.write("\n")
        sys.stdout.flush()
        return 0
    except (ValueError, TypeError, json.JSONDecodeError) as exc:
        print(f"direct_provider_runner error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
