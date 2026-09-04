#!/usr/bin/env python3
"""Check the small build-first policy contract without executing project code."""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "docs/integration/agent-execution-policy.json"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def require_fragments(path: Path, fragments: tuple[str, ...]) -> str:
    text = path.read_text(encoding="utf-8")
    for fragment in fragments:
        require(fragment in text, f"{path.relative_to(ROOT)} is missing {fragment!r}")
    return text


def forbid_fragments(path: Path, fragments: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for fragment in fragments:
        require(fragment not in text, f"{path.relative_to(ROOT)} still contains {fragment!r}")


def main() -> int:
    try:
        raw_policy = POLICY_PATH.read_text(encoding="utf-8")
        policy = json.loads(raw_policy)
        require(
            raw_policy == json.dumps(policy, indent=2, sort_keys=True) + "\n",
            "policy JSON is not canonically serialized",
        )
        require(policy.get("schema") == "uor-r4.agent-execution-policy/2", "wrong schema")
        require(policy.get("mode") == "build_first_pre_alpha", "wrong mode")
        require(policy["execution"]["automatic_retries"] == 0, "retries must remain zero")
        require(policy["delivery"]["active_task_limit"] == 1, "one-task limit changed")
        require(policy["delivery"]["protected_pull_request"] is True, "protected PR disabled")
        require(
            "formal_proof_work" in policy["routine_artifacts"]["deferred_until_alpha"],
            "proof work is no longer deferred",
        )
        require(
            "model_training_fitting_and_evaluation" in policy["execution"]["allowed"],
            "model work is not allowed",
        )
        require(
            policy["routine_artifacts"]["early_activation"]
            == "explicit_owner_instruction_only",
            "deferred process can be reactivated without the owner",
        )

        agents = require_fragments(
            ROOT / "AGENTS.md",
            (
                "<!-- agent-execution-policy:start -->",
                "mode is `build_first_pre_alpha`",
                "Formal proof work, claim-ledger maintenance",
                "Agents may build, test, train, evaluate, and run",
                "<!-- agent-execution-policy:end -->",
            ),
        )
        require(
            agents.count("<!-- agent-execution-policy:start -->") == 1,
            "duplicate policy start marker",
        )
        require(
            agents.count("<!-- agent-execution-policy:end -->") == 1,
            "duplicate policy end marker",
        )
        require_fragments(
            ROOT / "CONTRIBUTING.md",
            ("Build-first pre-alpha policy", "agent-execution-policy.json"),
        )
        require_fragments(
            ROOT / "docs/integration/agent-execution-policy.md",
            ("# Build-first pre-alpha execution", "Formal proof work"),
        )
        require_fragments(
            ROOT / "docs/integration/README.md",
            ("[Build-first pre-alpha policy](agent-execution-policy.md)",),
        )
        require_fragments(
            ROOT / "tools/skills/uor-project-workflow/SKILL.md",
            (
                "build_first_pre_alpha",
                "Agents may edit, compile, lint, test, train, fit, evaluate",
                "Only an explicit owner instruction can activate one earlier",
            ),
        )
        for current_path in (
            ROOT / "docs/integration/current-state.md",
            ROOT / "docs/integration/frontend-port-plan.md",
            ROOT / "docs/uor_productization_integration_plan.md",
        ):
            forbid_fragments(
                current_path,
                (
                    "Automated agents do not dispatch or run builds",
                    "owner decides whether to authorize a separate manual qualification",
                ),
            )
    except (KeyError, OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"agent execution policy: FAIL: {exc}", file=sys.stderr)
        return 1

    print("agent execution policy: OK (build-first pre-alpha)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
