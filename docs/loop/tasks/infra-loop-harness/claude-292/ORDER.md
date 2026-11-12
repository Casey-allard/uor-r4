# ORDER: infra-loop-harness, claude-292 (bootstrap Codex registration)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md)
(#2904). Setup row: 1 (the lab registry and its no-identity hint).

## Goal

Restore Codex to the five-lab roster while the base checker still requires an ORDER
from an already registered lab. This is the first half of a two-PR delivery.

## Owner approval

Casey approved the first registration of lab `codex` with entry
`{"vendor": "OpenAI", "client": "Codex", "headless": "codex exec", "open_pr_limit": 3}`
in `docs/harness/labs.json` at `2026-11-11T12:20Z` quoting:
> "what is wrong here? please register yourself then"

On 12 November 2026, Casey then said `proceed` after being shown that the policy job
builds the base checker, authorizing this bootstrap PR followed by a checker PR.

## Files in scope

- `docs/harness/labs.json`
- `crates/uor-r4-harness/src/{labs,loopguard,onboard,receipt}.rs`
- `crates/uor-r4-harness/tests/{lab_registry_cli,loop_prcheck}.rs`
- `docs/harness/README.md`
- `docs/loop/README.md`
- `docs/roadmap/items/infra-loop-harness.md`
- this task folder

Nothing else. If another file is needed, stop and report it.

## Size

The final measured size is recorded in REPORT.md; this task folder is excluded from that count.

## Cancel probe

Read `docs/harness/labs.json` before editing. Cancel if `codex` is already registered or
the registry already lists five labs; report the existing state rather than duplicating it.

## Done when

- The registry lists exactly `antigravity`, `claude`, `codex`, `deepseek`, and `kimi`,
  with Codex at vendor `OpenAI`, client `Codex`, headless `codex exec`, and open-PR limit 3.
- The registration-facing harness tests pass with all five receipts verified.
- `loop pr-check --local` passes this bootstrap branch under the base checker.
- The code-only first-registration hardening follows from fresh `main` in a separate PR.

## Stop and report if

- the registry is already at five labs;
- a file outside this ORDER is required;
- the same check fails twice;
- the bootstrap would require changing the policy job to run PR code.

## Rules

No model, GPU, pod, training, or spend is part of this unit. Do not weaken the refusal of a
genuinely unregistered lab. Stop after this bootstrap PR; the checker hardening is a separate
unit in a new session.
