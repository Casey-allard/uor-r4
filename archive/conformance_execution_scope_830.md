# #830 — execution scope, serving reachability, and non-vacuous empirical verdicts

Item B of the S0 truth-and-inference-closure tracker (#821). Per-issue record of
the register schema migration and the register/ledger reconciliation. Historical
records are append-only; the register and ledger are the corrected live
summaries.

## Problem

`model/ids.toml` and `model/ledger.toml` generated a consistent register, but the
register could not distinguish reference-only, offline-compiler,
certifier/instrument, dormant portable-runtime, normative-runtime, and
deployed-serving evidence. That permitted over-reading a structural harness as a
production result. Two ledger summaries were also stale: the route-fit real run
was recorded as UNAVAILABLE after its #804/#605 S1 verdict had already returned
FAIL, and the patch-overlay entry described `R4G1Runtime` holding a patch chain as
serving-runtime wiring.

## What changed

### Register schema (`crates/repo-model`)

Every `model/ids.toml` row now carries three new fields, rendered into
`CONFORMANCE.md`:

- `scope` — execution scope: one of `reference-only`, `offline-compiler`,
  `certifier-instrument`, `dormant-portable-runtime`, `normative-runtime`,
  `deployed-production`.
- `reachability` — serving reachability: one of `deployed-serving`,
  `off-serving-path`, `dormant-gated`.
- `evidence` — a pointer to the harness/suite/source that validates the `build`
  claim. Required non-empty for a `build` row (`Model::check`).

The fields carry deterministic migration defaults (`reference-only`,
`off-serving-path`, empty) so a row that omits them understates rather than
over-reads; all 29 existing rows set them explicitly and every `RF-*` ID is
stable.

Two model-check guards were added:

1. A `build` claim with no `evidence` pointer is rejected.
2. A row claiming `deployed-production` scope without a `deployed-serving`
   reachability assertion is rejected — a non-production test cannot be cited as
   production evidence without a dedicated reachability assertion.

### Empirical verdict axis (`crates/repo-model/src/empirical.rs`)

`build` is *harness-built* (structural) status. An empirical verdict is a
separate axis with exactly three values — `PASS`, `FAIL`, `UNAVAILABLE` —
modeled by `EmpiricalStatus`. The only path to `PASS` requires a present,
CID-bound fixture *and* a run that met its criterion; an absent fixture is
`UNAVAILABLE` before the run outcome is even consulted, so a missing fixture can
never mint a `PASS` (the RF-29 "vacuous pass when fixtures absent" hazard).

### Audited rows (narrowed wording)

- **RF-08** future_state_planner → `reference-only` / `off-serving-path`
  (owned-string reference planner).
- **RF-22** r4g1_quality → `certifier-instrument` / `off-serving-path`; statement
  narrowed from "generated-response quality" to a synthetic-input pathology
  rejection filter that does not establish generation quality.
- **RF-27** semantic_state_space and **RF-28** separate_semantic_emission →
  `reference-only` / `off-serving-path` (reference/f32 models, per the `S` / `T`
  compiler/reference-only binding in `docs/formal_vocabulary.md`).
- **RF-29** teacher_parity_benchmarks → `certifier-instrument` /
  `off-serving-path`; statement notes it is a fixture-gated empirical instrument
  whose absent-fixture verdict is `UNAVAILABLE`, never `PASS`.

Rows genuinely on the served `R4G1Runtime` / `R4Engine` call graph
(RF-09/10/11/13/23/26, verified via `src/tless_uor.rs` and `src/chat.rs`) are
`normative-runtime` / `deployed-serving`; the single-normative-scorer designation
itself is #831 (item C of #821) and is named explicitly rather than assumed. No
row claims `deployed-production` scope, which remains reserved for a deployed-path
reachability audit.

### Ledger reconciliation (`model/ledger.toml`)

- `route-fit-dormant` and `target-operator-certificate-dormant` — the
  real-teacher stage is no longer UNAVAILABLE. The #804/#605 S1 run against a
  traced SmolLM2-360M teacher (62,875 records, 6.6M eligible steps) returned FAIL
  — instrument vacuous: fitted support overlap 0.396 over the permuted-code N1
  null 0.192, but the pre-registered anti-vacuity N2 null reached 0.292
  (temporally-smooth teacher supports), with 119/120 heads individually vacuous.
  The operator stays dormant behind its unchanged gate; the real-corpus stage
  remains UNAVAILABLE (#531 corpus not produced). Full record: `docs/RESEARCH.md`,
  #605 (corpus issuecomment-5337863280 / verdict issuecomment-5337903765), #804.
- `patch-overlay-dormant` — corrected to distinguish `R4G1Runtime` holding a
  patch-chain field (portable-runtime reachability) from normative `R4Engine`
  deployed serving. No deployed serving path induces or emits patch epochs today;
  `patch_induction` / `patch_lifecycle` remain the dormant overlay-update surface.

## Negative controls

- `crates/repo-conformance/tests/empirical_absence.rs` — an absent fixture
  resolves to `UNAVAILABLE` (both would-be outcomes) and never `PASS`; a present
  fixture reflects the run outcome (positive control, so the negative is not
  vacuous).
- `crates/repo-model/src/lib.rs` tests — a `deployed-production` row without a
  `deployed-serving` assertion is rejected; a `build` row with no evidence pointer
  is rejected; the valid forms are accepted.

## Verification

`cargo xtask check-model` (29 ids, CM-01); `cargo test -p repo-conformance -p
repo-model`; `cargo xtask audit-deferral` (R4); `cargo xtask audit-limits` (R5);
`python3 scripts/check_claim_wording.py`; `cargo fmt --check`; `cargo clippy
--workspace --all-targets --all-features -D warnings`; the no_std ladder; and the
wasm router lib build.

## Scope boundary

Structural / harness status only. `CONFORMANCE.md` is generated (never
hand-edited). No dormant route/patch/planner/reference model is promoted.
Empirical `PASS` remains reserved for a fixture-present, CID-bound run.
