from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

from cli.main import main


DETERMINISTIC_WRAPPER_SCRIPT = """
import json
import sys

line = sys.stdin.readline()
observation = json.loads(line)
action_space = tuple(observation.get("action_space", ()))
action = "wait"
for candidate in action_space:
    if candidate.startswith("take "):
        action = candidate
        break
else:
    for candidate in action_space:
        if candidate.startswith("move "):
            action = candidate
            break
    else:
        for candidate in action_space:
            if candidate.startswith("attack "):
                action = candidate
                break
        else:
            action = "look" if "look" in action_space else "wait"
print(json.dumps({"action": action}, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
"""

PERSISTENT_WRAPPER_SCRIPT = """
import json
import sys
from pathlib import Path

turn = 0
trace_path = Path("__TRACE_FILE__")
for line in sys.stdin:
    observation = json.loads(line)
    action_space = tuple(observation.get("action_space", ()))
    step = int(observation.get("step", -1))
    trace_path.write_text(
        trace_path.read_text(encoding="utf-8") + f"{turn}:{step}\\n" if trace_path.exists() else f"{turn}:{step}\\n",
        encoding="utf-8",
    )
    if turn == 0 and step != 0:
        print(json.dumps({"bad": "unexpected_restart"}))
        sys.stdout.flush()
        continue
    if turn > 0 and step == 0:
        print(json.dumps({"bad": "unexpected_restart"}))
        sys.stdout.flush()
        continue

    action = "wait"
    if turn == 0:
        action = "move east" if "move east" in action_space else "wait"
    elif turn == 1:
        action = "move east" if "move east" in action_space else "wait"
    elif turn == 2:
        action = "take golden-key" if "take golden-key" in action_space else "wait"
    elif "wait" in action_space:
        action = "wait"
    elif action_space:
        action = action_space[0]

    print(json.dumps({"action": action}, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
    sys.stdout.flush()
    turn += 1
"""


def _read_json_output(output: str) -> dict[str, object]:
    return json.loads(output.strip())


def _write_wrapper_script(tmp_path: Path) -> Path:
    script_path = tmp_path / "deterministic_wrapper.py"
    script_path.write_text(DETERMINISTIC_WRAPPER_SCRIPT, encoding="utf-8")
    return script_path


def _write_persistent_wrapper_script(tmp_path: Path) -> Path:
    script_path = tmp_path / "persistent_wrapper.py"
    trace_path = tmp_path / "persistent_wrapper.trace"
    script_path.write_text(
        PERSISTENT_WRAPPER_SCRIPT.replace("__TRACE_FILE__", str(trace_path)),
        encoding="utf-8",
    )
    return script_path


def test_cli_run_external_local_agent_hook_executes_through_real_path(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_wrapper_script(tmp_path)

    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} {script_path}",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-fetch-quest"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["scorecard"]["aggregate_score"] > 0.0
    assert payload["replay"]["event_count"] > 0


def test_cli_run_external_local_agent_persistent_session_executes_through_real_path(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    script_path = _write_persistent_wrapper_script(tmp_path)
    trace_path = tmp_path / "persistent_wrapper.trace"

    exit_code = main(
        [
            "run",
            "--scenario",
            "tiny-fetch-quest",
            "--actor-id",
            "agent-a",
            "--agent-command",
            f"{sys.executable} {script_path}",
            "--persistent-agent-session",
        ]
    )
    captured = capsys.readouterr()

    assert exit_code == 0
    payload = _read_json_output(captured.out)
    assert payload["accepted"] is True
    assert payload["scenario_id"] == "tiny-fetch-quest"
    assert payload["lifecycle"]["status"] == "finalized"
    assert payload["lifecycle"]["step_count"] > 1
    assert payload["scorecard"]["aggregate_score"] > 0.0
    assert payload["replay"]["event_count"] > 0
    trace_lines = trace_path.read_text(encoding="utf-8").strip().splitlines()
    assert len(trace_lines) > 1
    assert trace_lines[0].startswith("0:0")
