# REPORT: m3-policy-precedence-and-sieve-tests, deepseek-31 (owner-directed close-out)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `b00d9edad`. Item PR, docs-only.

## What did not work, first

- **The item kept being offered although its code merged a day-earlier cycle.** #2947 delivered the helper extraction, the two precedence tests, the store-miss/sieve test and descriptive docs; #2947's own REPORT left the item `ready` on a PM decision card (`#2028` 6335479675), and the owner then directed the lab to proceed without waiting (quoted in the ORDER).
- **A first attempt to claim the umbrella item for the notebook-field edit was refused:** `loop claim` blocks an order whose scope overlaps an open PR's item scope, and Kimi's PR 8 (#2982) holds `docs/roadmap/items/*.md`. So this PR cannot add the item `notebook:` field (scope is read at the merge base). The PM card's Option A (row in the in-scope lab README) is what this close-out uses; the real-page row plus `notebook:` field is a PM/infra backfill after #2982 lands, and it cannot be picked up under this item once it is `done` (claims require `ready`).
- **The coupling is recorded as current behaviour, not as a settled mechanism contract.** That is the PM card's Option A and the conservative choice: a future change to make `StoreRead::Absent` fall through to `log_recall` is a mechanism decision with its own tests, not a bookkeeping edit.
- **No code was changed.** `lib.rs` already gates both modules under `#[cfg(test)]` (verified by reading it, with the exclusion comment), so PM card point 1 needs no further work.
- **No receipts file in this PR on purpose.** It is a docs-only change; per D46 a doc-only PR may carry a same-lab fresh-context record, and adding a `receipts.jsonl` would make it not doc-only. The verification evidence is in this REPORT and the gate logs.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/roadmap/items/m3-policy-precedence-and-sieve-tests.md` | docs | 1 | `status: ready` → `status: done` |
| `docs/labs/m3-policy-precedence-and-sieve-tests/README.md` | docs | +~12 | the PM-card resolution under the owner direction: module gating verified, coupling kept descriptive, notebook row in-README with the real-page follow-up |
| `docs/roadmap/README.md` | generated | render | counts and row move to done |
| `docs/loop/tasks/m3-policy-precedence-and-sieve-tests/deepseek-31/{ORDER,REPORT}.md` | docs | small | this order and report |

**Size:** about 60 changed lines.

## Measurements

- **Done-when evidence (already on `main` from #2947):** `op_policy_precedences_and_derived_id_guard_resolve_disagreements` pins the **extracted helper** for both `OpPolicy` arms including the derived-id guard, with two mutation gates recorded in #2947's REPORT; `store_miss_returns_absent_without_calling_log_sieve_while_unresolved_calls_it` pins the store-miss/sieve rule; both are documented descriptively. **Honesty caveat (adversarial analysis):** `SavedCompiler::action` and `GroundedSession::emitter_suffix` remain inside `#[cfg(any())]` impls and the modules are `#[cfg(test)]`, so these are specification pins, not live-path regression tests; no non-test qualification path consumes these rules, and deleting a caller's one-line delegation would not fail a test.
- **Comment-gating evidence:** `crates/uor-r4-training/src/lib.rs` lines around the two modules show `#[cfg(test)]` plus "Compiled only under `cargo test`; excluded from the non-test native qualification graph."
- **No out-of-scope path:** item status-only edit (allowed), lab README inside the item scope, roadmap render, and the task folder.

## Checks run at the head

- `cargo run -p xtask -- roadmap render` then `roadmap render --check`: current.
- `uor-harness loop report --check` and `loop pr-check --local`: PASS at the head.
- `loop gates --full`: GREEN; `--break all`: CAUGHT.

## Skills used

- **solution-integrity**: the close-out names exactly what is settled (tests, documented descriptive rule) and what is not (the mechanism contract, the real notebook page), instead of treating a bookkeeping flip as the work.
- **holding-the-goal**: the item only closes because the done-when is already delivered; the owner direction resolves the pending coordination, not the science.

## Adversarial analysis (fresh-context subagent, 2026-10-22)

A fresh-context subagent was told to attack the three PM-card points and the close-out. Its corrections, all applied above:
1. **Helper-level, not live-path:** keeping `#[cfg(test)]` is right (re-admission would drag in the retired float graph), but the tests pin the extracted helper; both callers remain `#[cfg(any())]` and a caller-level test is a separate item if the native session returns.
2. **Coupling is intended code with a measured limitation, not a settled contract:** `with_log_recall`'s doc makes the skip deliberate for resolved reads, while #1738's 62→0 and the address-formation finding show the cost; a fallback is an open mechanism decision.
3. **Close-out:** `done` is justified by the done-when plus PM Option A, but the real notebook row becomes a **PM/infra backfill** (a done item cannot be claimed), so the REPORT must name that instead of a routine follow-up.

The analysis also confirmed no code change is needed and that no out-of-scope path may be touched in this PR.

## Still open

- **PM/infra backfill:** add `notebook: docs/research/notebook/m3-durable-memory.md` to the item and move/duplicate the row to the real page. This cannot be a routine m3 follow-up: the field needs an ORDER whose item scope covers `docs/roadmap/items/*.md` (only the umbrella/PR-8 scopes do, and #2982 currently holds them), and a `done` item cannot be claimed normally.
- **The store-miss/sieve coupling as a settled mechanism contract** remains a decision if anyone wants it declared; today it is the documented current behaviour.

## Plain summary

- **The item's work was already delivered; this closes the bookkeeping under the owner direction and the PM card's Option A.** The two precedence tests, the store-miss/sieve test and the descriptive docs are on `main`, the module gating is already `#[cfg(test)]`, and the notebook row is recorded in the in-scope lab README.
- **Nothing scientific was changed or claimed settled.** The coupling is recorded as a deliberate code contract **with a measured limitation** (the #1738 62→0 result and the address-formation finding); a fallback remains an open mechanism decision. The real notebook page row plus `notebook:` field is a PM/infra backfill after #2982.