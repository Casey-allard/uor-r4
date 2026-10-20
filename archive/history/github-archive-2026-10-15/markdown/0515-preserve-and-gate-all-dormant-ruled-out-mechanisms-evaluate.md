# #515 — Preserve-and-gate all dormant/ruled-out mechanisms; evaluate feature-flag modularization

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-08T23:41:53Z
- **Updated:** 2026-08-10T23:54:22Z
- **Closed:** 2026-08-10T23:54:22Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/515
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Direction (maintainer)
Keep **every** mechanism available - including ruled-out, measured-negative, and never-wired ones - so a later run can return to them. Preserve by **registration**, not deletion, and align each with the gates. Evaluate a modular / feature-flagged layout so dormant mechanisms can be toggled in and out cleanly.

## Current dormant `open` claims (model/ledger.toml)
`cd-space-dormant`, `lie-jordan-dormant`, `bott-fock-dormant`, `quantum-cover-dormant` (from #273), plus `packed-routing-dormant` and `graph-construction-dormant` (from #510). R4's carve-out already sanctions a marker whose line cites an `open` claim.

## Work
1. **Audit for un-registered ruled-out branches.** Sweep the negative-result exploration (e.g. #64 Sigma-over-cloud, #456 reconstructability null, #457 IPF-consistency-at-floor, CD-space #400, and any never-wired modules) and ensure each surviving-but-dormant mechanism is either registered `open` with an activation gate or is genuinely gone-by-design (documented). Nothing ruled-out should sit outside the register.
2. **Feature-flag modularization (evaluate).** Put dormant mechanisms behind cargo features (default-off) so they compile-toggle without bit-rotting: they stay buildable (a CI job builds `--all-features`), can be activated for a run by flipping the flag, and R4/R5 see them under the same discipline. Decide granularity (per-mechanism vs per-track).
3. **Activation-gate registry.** Each dormant claim already states its gate ("a measured lift clearing the pre-declared bar"); make the gates runnable so "is this worth reviving?" is a command, not a memory.

## Why
The substrate program is a search; branches ruled out at one scale/teacher may revive at another (the #509 broad-corpus result is exactly a mechanism that only works with the right teacher). Deleting them loses the option; registering + flagging them keeps the option at near-zero cost.


## Comments (2)

### comment 5246514209 — 2026-08-10T21:59:15Z — auser

Open-weight plan audit (2026-08-10): the current `uor-r4-model-source::Llama::forward` experimental branch still computes the same scalar dot product as standard attention, but groups coordinates in chunks of four (`crates/uor-r4-model-source/src/lib.rs`, attention branch around lines 677-710), divides by `sqrt(head_size)`, and then calls `softmax_with_mode` in both branches. With `head_size % 4 != 0`, the experimental branch drops the remainder coordinates. The public CLI/API/docs still call it “Spin(4)” and “softmax-free”; #425/#426 recorded deferral but did not correct this technical mismatch. The path is therefore a dormant/source-teacher experiment, not an R4-native operator.

The open-weight plan will not create a duplicate modularization issue: #515/PRs #593/#594/#596 own feature/ledger/activation-gate discipline. Proposed next action after the gate registry lands: register this path’s truthful operator and a pinned standard-vs-experimental A/B as its activation evidence; correct the stale names/claims and add the remainder-dimension regression. No promotion or deletion is implied.


### comment 5247380075 — 2026-08-10T23:54:21Z — Casey-allard

Completed across three parts.

## 1. Audit — preserve by registration

Every mechanism retained off the serving path is now registered in `model/ledger.toml` as an `open` claim with a pre-declared activation gate (seeded in #590). Registration was then reconciled against the build: each candidate was feature-gated and the full workspace built and test-compiled. Six registrations #590 introduced were disproven — the module is actually wired, so feature-gating it broke the default build or the default test build — and were corrected:

- `region_store` — used by the root model layer, `src/model.rs` (#592).
- `future_state_planner` — used by `semantic_emission_decoupling.rs`; `stage_dag` — used by `uor-r4-proof-model`'s `verify_compiler_stage_dag_compliance` (#593).
- `long_context` — the final call in the `certify()` entrypoint (`graph-certify/src/certify.rs`); `compiler_scaling` — used by `uor-r4-proof-model`'s `verify_compiler_scaling_certificate_compliance`; and the `patch-overlay` claim wording, which wrongly listed `patch_chain` (wired via `R4G1Runtime`) as off-path (#614).

Mechanisms that are genuinely off the *serving* path but carry a live validation/diagnostic harness are intentionally kept **registered-but-compiled** rather than gated, because gating them default-off would delete real coverage from the shipped build: `bott_fock` / `cd_space` (reachable via the `graph-cli` `cd-compile` / `quantum-eval` subcommands) and `quantum_cover`, `rate_distortion_compression`, `monograph`, `reference_compiler_ir`, `behavioral_probes`, `lie_jordan`, `endomorphism`, `packed_kernels` (exercised by the root BDD suite `tests/bdd.rs`). The register now holds 17 accurate open claims.

## 2. Feature-flag modularization

Per-mechanism **default-off** cargo features were added for every dormant module with zero default-build reference (code or tests):

- `uor-r4-graph-certify`: `anti_degeneracy`, `fairness_provenance`, `holographic_encoding`, `patch_lifecycle`, `predictive_sufficiency`, `shortlist_evaluator` (#591).
- `uor-r4-graph-compiler`: `graph` (build_graph construction), `perturbation`, `patch_induction` (#593).

The shipped default build is byte-identical whether or not these are enabled. Dormancy is proven by the build: the default build excludes them; `--all-features` compiles all of them and runs their gated tests. A new CI job (#595) does a real `--all-features` build + gated-test run on every merge so the dormant code cannot bit-rot. Finding: feature-gating is only correct for a dormant module with no live harness; the many that do have a BDD or CLI harness stay registered-but-compiled (see part 1).

## 3. Runnable activation-gate registry

`cargo xtask gate <claim-id>` and `cargo xtask gates` (#596) read `model/ledger.toml`, enumerate every open claim, extract and classify its activation gate (mechanism-comparison vs metric-adoption), and report status — every open claim is NOT CLEARED by definition (an open claim is dormant until its gate clears), with the exact pre-declared bar printed so a future run knows precisely what to measure. Per-claim measurement runners are the wired extension point, keyed by claim id, attaching against the Gate C fixtures.

Note: promoting the `register conformance` and the new `all-features` CI jobs to **required** status checks is a repo-admin branch-protection toggle.


---
_Comment/review ids in this file: 5246514209, 5247380075_

