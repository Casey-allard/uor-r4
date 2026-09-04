#!/usr/bin/env python3
"""Fail closed when the deterministic source-only agent policy drifts.

This script uses the Python standard library, reads tracked text, and never
imports or executes project code.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "docs/integration/agent-execution-policy.json"
AGENTS_PATH = ROOT / "AGENTS.md"
CONTRIBUTING_PATH = ROOT / "CONTRIBUTING.md"
WORKFLOW_PATH = ROOT / ".github/workflows/ci.yml"
POLICY_DOC_PATH = ROOT / "docs/integration/agent-execution-policy.md"
INTEGRATION_INDEX_PATH = ROOT / "docs/integration/README.md"

EXPECTED_POLICY: dict[str, Any] = {
    "agent_scope": {
        "applies_to": ["automated_repository_agents"],
        "policy_change_requires": [
            "explicit_owner_instruction",
            "protected_pull_request",
        ],
    },
    "failure_budget": {
        "automatic_retries": 0,
        "environment_probe_attempts": 0,
        "source_corrections_after_concrete_remote_failure": 1,
        "terminal_action_after_next_failure": "park_and_report",
    },
    "local_execution": {
        "allowed": [
            "read_repository_and_authoritative_sources",
            "edit_declared_source_paths",
            "git_status_diff_log_and_named_path_staging",
            "run_static_agent_policy_guard",
        ],
        "forbidden": [
            "cargo_rustc_rustup_and_wasm_pack",
            "test_benchmark_fuzz_lint_and_build_runners",
            "model_teacher_training_fitting_and_evaluation",
            "browser_service_and_product_probes",
            "sandbox_syscall_entitlement_and_environment_probes",
            "custom_supervisors_watchdogs_and_retry_wrappers",
        ],
    },
    "mode": "deterministic_source_only",
    "remote_execution": {
        "agent_may_dispatch_qa": False,
        "owner_manual_qa_workflow": "workflow_dispatch_only",
        "pull_request_and_merge_group": (
            "static_policy_guard_plus_ruleset_transport_only"
        ),
    },
    "reporting": {
        "final_fields": [
            "delivered_result",
            "source_review_limits",
            "closure_state",
            "one_concrete_next_action",
        ],
        "suppress": [
            "individual_build_progress",
            "individual_test_progress",
            "individual_probe_progress",
            "unchanged_polling",
        ],
    },
    "schema": "uor-r4.agent-execution-policy/1",
    "workspace": {
        "base": "refreshed_origin_main",
        "preserve_user_and_unique_evidence": True,
        "sparse_pruned_or_hand_copied_workspaces_forbidden": True,
        "type": "full_git_worktree",
    },
}

REQUIRED_TRANSPORT_JOBS = {
    "required-core-transport",
    "required-audit-transport",
    "required-fuzz-transport",
    "required-wasm-transport",
    "required-gate-c-transport",
}

# These tokens are prohibited only in automatic PR/merge transport job `run:`
# commands. Manual workflow_dispatch jobs intentionally retain release QA.
PROHIBITED_AUTOMATIC_RUN_TOKENS = (
    "cargo ",
    "rustc ",
    "rustup ",
    "wasm-pack ",
    "nextest ",
    "pytest ",
    "cargo-fuzz ",
    "npm ",
    "pnpm ",
    "yarn ",
    "scripts/gate_c",
)


def fail(message: str) -> None:
    raise ValueError(message)


def load_policy() -> dict[str, Any]:
    try:
        policy = json.loads(POLICY_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"cannot read canonical policy: {exc}")
    if policy != EXPECTED_POLICY:
        fail("agent execution policy differs from the guarded version")
    canonical = json.dumps(policy, indent=2, sort_keys=True) + "\n"
    if POLICY_PATH.read_text(encoding="utf-8") != canonical:
        fail("agent execution policy JSON is not canonically serialized")
    return policy


def require_text(path: Path, fragments: tuple[str, ...]) -> str:
    text = path.read_text(encoding="utf-8")
    for fragment in fragments:
        if fragment not in text:
            fail(f"{path.relative_to(ROOT)} is missing required text: {fragment!r}")
    return text


def job_blocks(workflow: str) -> dict[str, str]:
    try:
        jobs = workflow.split("\njobs:\n", maxsplit=1)[1]
    except IndexError:
        fail("workflow has no jobs section")
    matches = list(re.finditer(r"(?m)^  ([a-zA-Z0-9_-]+):\n", jobs))
    blocks: dict[str, str] = {}
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(jobs)
        blocks[match.group(1)] = jobs[match.start() : end]
    return blocks


def run_commands(block: str) -> list[str]:
    commands: list[str] = []
    lines = block.splitlines()
    index = 0
    while index < len(lines):
        line = lines[index]
        single = re.match(r"^\s+-?\s*run:\s*(?![>|]\s*$)(.*)$", line)
        if single:
            commands.append(single.group(1).strip())
            index += 1
            continue
        multiline = re.match(r"^(\s*)-?\s*run:\s*[>|]\s*$", line)
        if multiline:
            base_indent = len(multiline.group(1))
            index += 1
            body: list[str] = []
            while index < len(lines):
                candidate = lines[index]
                if candidate.strip() and len(candidate) - len(candidate.lstrip()) <= base_indent:
                    break
                body.append(candidate.strip())
                index += 1
            commands.append("\n".join(body))
            continue
        index += 1
    return commands


def check_workflow() -> None:
    workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
    event_section = workflow.split("concurrency:", maxsplit=1)[0]
    for forbidden_event in ("  push:\n", "  schedule:\n", "  workflow_run:\n"):
        if forbidden_event in event_section:
            fail(f"automatic CI trigger is forbidden: {forbidden_event.strip()}")

    blocks = job_blocks(workflow)
    if not REQUIRED_TRANSPORT_JOBS.issubset(blocks):
        missing = sorted(REQUIRED_TRANSPORT_JOBS - blocks.keys())
        fail(f"required transport jobs are missing: {missing}")

    required_if = (
        "github.event_name == 'pull_request' || "
        "github.event_name == 'merge_group'"
    )
    for name in sorted(REQUIRED_TRANSPORT_JOBS):
        block = blocks[name]
        if required_if not in block:
            fail(f"{name} no longer has the bounded PR/merge trigger")
        commands = run_commands(block)
        allowed_commands = {
            command
            for command in commands
            if command == "python3 scripts/check_agent_execution_policy.py"
            or command.startswith('echo "Ruleset transport only;')
        }
        if len(allowed_commands) != len(commands):
            fail(f"{name} contains a command outside the static/transport allowlist")
        for command in commands:
            lowered = command.lower()
            for token in PROHIBITED_AUTOMATIC_RUN_TOKENS:
                if token in lowered:
                    fail(f"{name} contains prohibited automatic command token {token!r}")

        uses = re.findall(r"(?m)^\s+(?:-\s+)?uses:\s*(\S+)\s*$", block)
        expected_uses = ["actions/checkout@v4"] if name == "required-core-transport" else []
        if uses != expected_uses:
            fail(f"{name} contains actions outside the static/transport allowlist")

    core = blocks["required-core-transport"]
    if "uses: actions/checkout@v4" not in core:
        fail("required-core-transport must use the complete repository checkout")
    if "\n        with:" in core:
        fail("required-core-transport checkout must not be sparse or filtered")
    if "run: python3 scripts/check_agent_execution_policy.py" not in core:
        fail("required-core-transport must execute the static policy guard")

    for name, block in blocks.items():
        if name in REQUIRED_TRANSPORT_JOBS:
            continue
        has_executable_qa = bool(run_commands(block)) or "uses:" in block
        if has_executable_qa:
            dispatch_if = re.search(
                r"(?m)^    if: github\.event_name == 'workflow_dispatch'(?: && [^|\n]+)?$",
                block,
            )
            if dispatch_if is None:
                fail(f"executable job {name} is not restricted to workflow_dispatch")


def main() -> int:
    try:
        load_policy()
        agents = require_text(
            AGENTS_PATH,
            (
                "<!-- agent-execution-policy:start -->",
                "mode is `deterministic_source_only`",
                "Sparse, pruned, filtered, or hand-copied workspace capsules",
                "Agents do not run or dispatch builds, tests,",
                "probes, model work, or QA.",
                "<!-- agent-execution-policy:end -->",
            ),
        )
        if agents.count("<!-- agent-execution-policy:start -->") != 1:
            fail("AGENTS.md must contain exactly one policy start marker")
        if agents.count("<!-- agent-execution-policy:end -->") != 1:
            fail("AGENTS.md must contain exactly one policy end marker")
        require_text(
            CONTRIBUTING_PATH,
            (
                "Deterministic source-only agent policy",
                "docs/integration/agent-execution-policy.json",
                "Agents do not run or dispatch builds, tests,",
                "probes, model work, or QA.",
            ),
        )
        require_text(
            POLICY_DOC_PATH,
            (
                "# Deterministic source-only agent execution",
                "Environment-probe and automatic-retry budgets are both zero.",
                "project builds, tests, probes, model runs, and product",
                "`NOT_RUN_BY_POLICY`",
            ),
        )
        require_text(
            INTEGRATION_INDEX_PATH,
            (
                "[Deterministic source-only policy](agent-execution-policy.md)",
                "[machine contract](agent-execution-policy.json)",
            ),
        )
        check_workflow()
    except (OSError, ValueError) as exc:
        print(f"agent execution policy: FAIL: {exc}", file=sys.stderr)
        return 1

    print("agent execution policy: OK (static text only; no project code executed)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
