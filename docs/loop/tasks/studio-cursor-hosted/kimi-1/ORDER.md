# ORDER: Stage 0 contract and protected baseline, kimi-1

Lab: kimi. Item: [studio-cursor-hosted](../../../../roadmap/items/studio-cursor-hosted.md) (#3917). Base: `993bfd36cc4ef4660d482d0f65238361d7126e12`. References #3569; leaves its status and acceptance unchanged.

## Owner approval and outcome

The owner said verbatim: **"I approve of this plan, please implement it end to end autonomously"** (2026-11-09). Official issue #3917 is synced. Execute Tasks 1–4 with red-first tests and independent reviews: Rust contract, generated TypeScript, browser client/reducer, and protected baseline guards. Fixtures are not model evidence.

## Cancel probe

Confirm no competing ORDER overlaps these paths, the contract crate is absent, and typed-shell/PTY evidence remains. Official issue must exist before product code. Cancel only a complete duplicate; missing hooks/tests are work.

## Scope

Only `Cargo.toml`, `Cargo.lock`, `crates/uor-r4-studio-agent/**`, `scripts/studio/generate-agent-contract.mjs`, `web/studio/{package.json,package-lock.json,playwright.config.ts,index.html,README.md}`, `web/studio/src/{bootstrap.ts,studio.css}`, `web/studio/src/agent/**`, `web/studio/src/state/agent.ts`, `web/studio/src/components/**`, `web/studio/tests/e2e/**`, `docs/roadmap/issues.json`, this task folder, and approved spec/plan corrections. Generated output uses the official command only. No model/API/engine/worker/tokenizer, service, Tauri, deployment, credential, `.github` or owner-checkout change.

## Binding contracts

- `AgentStore` alone owns task/run/event/cursor state; `LifecycleStore` remains legacy shell/session/model state. Hosts never own task state. ReplayModelPort is a labelled fixture, not R4 generation.
- Exact schemas/IDs/canonical limits follow the approved plan/item. No `task.created` or `run.retried`. `retry_of_run_id` is `string | null`; terminal events are authoritative and `run.status` only a projection.
- `run.resumed` carries `{ resumed_at_ms, previous_generation, snapshot_digest }`: non-negative safe integers and `sha256:<64 lowercase hex>`. After verified generation G/sequence N, resume uses G+1/N+1, never repeats start markers or duplicates output.
- Raw JSON parsing rejects nested duplicate keys in Rust and generated TS and is used for hostile ingestion. Canonical objects use sorted ASCII keys, safe non-negative integers, minimal escaping and matching SHA-256 digests independent of insertion order.
- Typed mode has one task list and one `AgentRuntime` send path. Retarget one skip link to `#harness-main` typed and `#mainColumn` legacy; scope visible-selector assertions to `#studio-typed-root`.
- Rail: `1023_defaults_closed`, `1024_defaults_open`; test close/reopen/manual close without changing legacy AppShell, query switches, selectors or IME.

## Execution and done checks

For each Task N: run `scripts/task-brief PLAN_FILE N`; record BASE; dispatch one fresh implementer with this ORDER/brief; capture RED before and GREEN after; generate `scripts/review-package PLAN_FILE BASE HEAD`; obtain spec and quality verdicts from a fresh reviewer; resolve Critical/Important findings before the next task. Serial writers only.

Stage acceptance: official issue number and generated metadata land through `roadmap sync`; `cargo test -p uor-r4-studio-agent`; focused generated/client/reducer Vitest; typecheck/build; `cursor-baseline.spec.ts` and existing shell/workflows suites pass. Generator second run has an empty diff. Commit after each reviewed task and write actual counts/log paths into the stage REPORT.

## Stop and re-scope if

A requirement needs a new dependency, host task store, model/API support, second runtime authority, renamed protected selectors, changed legacy behavior, or unsupported capability claim. Official sync created #3917 after separate PR #3916 repaired base validation; never manually create issues or hand-edit generated metadata. Stop on any unexpected sync write or unmet task acceptance.
