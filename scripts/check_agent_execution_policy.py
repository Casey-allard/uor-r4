#!/usr/bin/env python3
"""Check the small build-first policy contract without executing project code."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "docs/integration/agent-execution-policy.json"
TRACK_FRAGMENTS = (
    "fixed recurrent",
    "sparse geometric attention",
    "nonlinear geometric",
    "scale",
    "retrieval",
    "product alpha",
    "rust/table lowering",
    "release proof",
)
TRACK_PATHS = (
    "AGENTS.md",
    "CONTRIBUTING.md",
    "README.md",
    "ROADMAP.md",
    "docs/RESEARCH.md",
    "docs/r4_intelligence_completion_plan.md",
    "docs/geometric_intelligence_programme.md",
    "docs/uor_productization_integration_plan.md",
    "docs/integration/project-track.md",
    "docs/integration/agent-execution-policy.md",
    "docs/integration/current-state.md",
    "docs/integration/CONTINUE.md",
)


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
        require(
            fragment not in text,
            f"{path.relative_to(ROOT)} still contains {fragment!r}",
        )


def require_ordered_track(path: Path) -> None:
    text = re.sub(r"[^a-z0-9/]+", " ", path.read_text(encoding="utf-8").casefold())
    cursor = 0
    for fragment in TRACK_FRAGMENTS:
        offset = text.find(fragment, cursor)
        require(
            offset >= 0,
            f"{path.relative_to(ROOT)} omits or reorders track stage {fragment!r}",
        )
        cursor = offset + len(fragment)


def main() -> int:
    try:
        raw_policy = POLICY_PATH.read_text(encoding="utf-8")
        policy = json.loads(raw_policy)
        require(
            raw_policy == json.dumps(policy, indent=2, sort_keys=True) + "\n",
            "policy JSON is not canonically serialized",
        )
        require(
            policy.get("schema") == "uor-r4.agent-execution-policy/3",
            "wrong schema",
        )
        require(
            policy.get("mode") == "build_first_architectural_alpha",
            "wrong mode",
        )
        require(
            policy["execution"]["automatic_retries"] == 0,
            "retries must remain zero",
        )
        require(policy["delivery"]["active_task_limit"] == 1, "one-task limit changed")
        require(
            policy["delivery"]["protected_pull_request"] is True,
            "protected PR disabled",
        )
        require(
            "broad_formal_proof_work"
            in policy["routine_artifacts"]["not_default_before_release_candidate"],
            "proof work is no longer deferred",
        )
        require(
            "model_training_fitting_and_evaluation" in policy["execution"]["allowed"],
            "model work is not allowed",
        )
        require(
            policy["project_track"]["ordered_stages"]
            == [
                "fixed_recurrent_geometric_memory",
                "sparse_geometric_attention",
                "nonlinear_geometric_block",
                "scale_data_and_instruction_behavior",
                "retrieval_and_tools",
                "representative_product_alpha",
                "rust_table_lowering_and_optimization",
                "release_proof_evidence_and_qa",
            ],
            "project-track order changed",
        )
        require(
            policy["project_track"]["current_stage"]
            == "scale_data_and_instruction_behavior",
            "current project-track stage changed",
        )
        require(
            policy["evidence"]["unavailable_is_model_evidence"] is False,
            "UNAVAILABLE was promoted to model evidence",
        )
        require(
            policy["execution"]["final_heldout_evaluation"]
            == "after_design_selection",
            "held-out evaluation moved into open development",
        )

        agents = require_fragments(
            ROOT / "AGENTS.md",
            (
                "<!-- agent-execution-policy:start -->",
                "build_first_architectural_alpha",
                "Research campaigns are on-demand donor reservoirs",
                "Bounded open-data development iteration is allowed",
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
            (
                "Build-first architectural-alpha policy",
                "project track",
                "agent-execution-policy.json",
            ),
        )
        require_fragments(
            ROOT / "docs/integration/agent-execution-policy.md",
            (
                "# Build-first architectural-alpha execution",
                "Ordered authority",
                "versioned successor may re-enter",
            ),
        )
        require_fragments(
            ROOT / "docs/integration/README.md",
            (
                "[build-first architectural-alpha policy](agent-execution-policy.md)",
                "[Active project track](project-track.md)",
            ),
        )
        require_fragments(
            ROOT / "docs/integration/project-track.md",
            (
                "## Ordered build sequence",
                "Sparse geometric attention",
                "## Research reservoirs",
                "## Evidence and iteration rules",
            ),
        )
        require_fragments(
            ROOT / "tools/skills/uor-project-workflow/SKILL.md",
            (
                "build_first_architectural_alpha",
                "Bounded iteration on open development data is allowed",
                "`UNAVAILABLE` is not model evidence",
            ),
        )
        for relative_path in TRACK_PATHS:
            require_ordered_track(ROOT / relative_path)
        forbid_fragments(
            ROOT / "README.md",
            ("## Current roadmap", "The active research step is #1082"),
        )
        forbid_fragments(ROOT / "ROADMAP.md", ("\n## Active\n",))
        forbid_fragments(
            ROOT / "docs/uor_productization_integration_plan.md",
            ("**Immediate scientific next action:**",),
        )
        forbid_fragments(
            ROOT / "docs/integration/current-state.md",
            ("Routine pre-alpha pull requests",),
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
                    "owner decides whether to authorize a separate manual "
                    "qualification",
                ),
            )
    except (KeyError, OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"agent execution policy: FAIL: {exc}", file=sys.stderr)
        return 1

    print("agent execution policy: OK (build-first architectural alpha)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
