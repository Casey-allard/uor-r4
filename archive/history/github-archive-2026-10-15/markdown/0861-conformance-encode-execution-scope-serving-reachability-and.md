# #861 — conformance: encode execution scope, serving reachability, and non-vacuous empirical verdicts (#830)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T01:08:35Z
- **Updated:** 2026-10-09T13:08:54Z
- **Closed:** 2026-08-20T01:30:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/861
- **Merged:** 2026-08-20T01:30:25Z
- **Merge commit:** bc5a2a92cf066367c519b559cf8a28813f3e94c3
- **Base / Head:** main / UOR-Foundation:issue-830-conformance-scope
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

The conformance register (`model/ids.toml` + `model/ledger.toml` → `CONFORMANCE.md`) could not distinguish reference-only, offline-compiler, certifier/instrument, dormant portable-runtime, normative-runtime, and deployed-serving evidence, which permitted over-reading a structural harness as a production result. Two ledger summaries were also stale. This PR makes execution scope and serving reachability machine-visible, reconciles the stale ledger truth, and makes fixture-dependent empirical status impossible to report as `PASS` when inputs are absent (item B of the S0 tracker #821).

## Scope

- **Register schema** (`crates/repo-model/src/registry.rs`): new `scope` (`reference-only` / `offline-compiler` / `certifier-instrument` / `dormant-portable-runtime` / `normative-runtime` / `deployed-production`) and `reachability` (`deployed-serving` / `off-serving-path` / `dormant-gated`) enums, plus an `evidence` pointer, on every `IdRow`. Rendered into `CONFORMANCE.md` (`codegen.rs`).
- **Model checks** (`lib.rs`): a `build` claim must carry an evidence pointer; a `deployed-production` scope requires a `deployed-serving` reachability assertion.
- **Empirical axis** (`crates/repo-model/src/empirical.rs`): `EmpiricalStatus` (`PASS` / `FAIL` / `UNAVAILABLE`), separate from the harness-built `build` level; a missing fixture is `UNAVAILABLE`, never `PASS`.
- **Row audit** (`model/ids.toml`): all 29 `RF-*` rows classified; RF-08 (owned-string reference planner), RF-22 (synthetic pathology filter), RF-27/28 (reference/f32 models), RF-29 (fixture-gated instrument) narrowed to implemented reachability.
- **Ledger reconciliation** (`model/ledger.toml`): `route-fit-dormant` + `target-operator-certificate-dormant` updated to the #804/#605 S1 **FAIL — instrument vacuous** verdict; `patch-overlay-dormant` corrected so `R4G1Runtime` patch-chain reachability is not called normative `R4Engine` serving.
- **Docs**: `formal_vocabulary.md` version header aligned to changelog (new 0.1.14 entry); `features/suites/README.md` refreshed; per-issue record `docs/conformance_execution_scope_830.md`.
- **`CONFORMANCE.md`** regenerated from the model.

## Non-goals (respected)

No dormant route/patch/planner/reference model promoted; no one-RF-ID-per-issue allocation; `CONFORMANCE.md` not hand-edited (generated).

## Acceptance criteria

- [x] Every capability row renders an execution scope and explicit deployed-serving reachability.
- [x] A non-production test cannot be cited as production evidence without a dedicated reachability assertion (enforced by `Model::check`; planted negative in `repo-model`).
- [x] Empirical status is exactly `PASS`/`FAIL`/`UNAVAILABLE` and missing fixtures cannot produce `PASS` (`EmpiricalStatus`; planted negative in `repo-conformance`).
- [x] The route-fit and patch-overlay ledger entries match live evidence and remain dormant behind activation gates.
- [x] Generated files match their TOML/test sources; all existing `RF-*` IDs remain stable.

## Tests and verification

`cargo xtask check-model` (29 ids, CM-01) · `cargo test -p repo-conformance -p repo-model` (incl. the new schema-guard and empirical-absence tests) · `cargo xtask audit-deferral` (R4) · `cargo xtask audit-limits` (R5) · `python3 scripts/check_claim_wording.py` · `cargo fmt --check` · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `cargo check -p uor-r4-graph-format --no-default-features [--features alloc]` · wasm router lib build. Broader `cargo test --workspace` was green across all binaries; the pinned live-teacher BDD runs in slow mode and skips graph scenarios on a pre-existing bundle-quality condition unrelated to this change (no runtime/serving/format bytes are touched here).

## Negative / unavailable results

- Absent-fixture planted negative (`crates/repo-conformance/tests/empirical_absence.rs`): a missing fixture resolves to `UNAVAILABLE` for both would-be outcomes and never `PASS`; a present fixture reflects the run outcome (positive control).
- Schema guards (`crates/repo-model/src/lib.rs`): `deployed-production` without `deployed-serving`, and a `build` row without evidence, are both rejected; valid forms accepted.
- The #804/#605 real-teacher route-fit run is recorded as a **FAIL (instrument vacuous)** — a first-class negative result now reflected in the ledger; the real-corpus stage stays `UNAVAILABLE` (#531 corpus not produced).

## Compatibility and migration

New fields carry deterministic migration defaults (`reference-only`, `off-serving-path`, empty), so a row that omits them understates rather than over-reads; all 29 rows set them explicitly and every `RF-*` ID and link is stable. No shipped crate depends on `repo-model`; no artifact bytes change.

## Conformance effects and claim boundary

Register/harness status is `build` (structural). Empirical `PASS` is a distinct axis reserved for a fixture-present, CID-bound run. `deployed-production` scope is representable but claimed by no row yet — reserved for a deployed-path reachability audit; the single-normative-scorer designation is #831 and is named explicitly.

## Intentionally excluded

Untracked `obs-text/` and `tf1-pkg/` (user-owned) are not staged.

Closes #830

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

