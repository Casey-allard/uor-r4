# #938 — research(#931): record RF-31 content calibrator re-entry verdict

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-25T18:00:55Z
- **Updated:** 2026-10-09T14:17:18Z
- **Closed:** 2026-08-25T18:28:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/938
- **Merged:** 2026-08-25T18:28:56Z
- **Merge commit:** 9fe85b6e59d117f6319f841591a867cc2606eebd
- **Base / Head:** main / UOR-Foundation:issue-931-content-evidence-calibrator
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Executes the single S2 RF-31 content-evidence calibrator re-entry under the frozen #838 gates. The binding FIT instrument qualified, but no selectable arm met the release or research gate on CAL.

Final verdict: **`NO CALIBRATOR ESTABLISHED`**. TEST remained sealed, the real powered semantic fixture remains `UNAVAILABLE`, semantic abstention remains `NOT ESTABLISHED`, and #839 stays legacy-only.

## Scope

- add the issue-numbered offline integration harness using the deployed packed SKMX/PSIB row semantics
- preserve FIT/CAL/TEST isolation and single-use TEST admission
- exercise all selectable arms, controls, shuffles, corruption/leakage teeth, packed/reference differentials, and deterministic result generation
- add the CID-bound JSON result and readable evidence record
- append the #931 result to the #837 history and reconcile the current research/sequence summaries

No runtime behavior, artifact format, protocol, migration policy, model registry row, or production gate changes.

## Evidence

- binding instrument: `PROCEED`, 20.5 s, FIT only
- full teacher-free CAL study: 38.4 s
- result CID: `blake3:8372e7a1171fe0e841f3d5b29541db16f386c70a04885c5b245027cf43267496`
- result SHA-256: `838b4591f126f6ab0d2f767a976f24c07782b9932910831e6c9dff3a019d7ade`
- null controls: label, feature, and content-key shuffles all non-vacuous and non-qualifying
- TEST: `test_opened=false`

## Verification

- `cargo test -p uor-r4-api --offline` — PASS
- `cargo clippy -p uor-r4-api --all-targets --all-features --offline -- -D warnings` — PASS
- `cargo fmt --check` — PASS
- graph-format no-default/no-std and alloc rungs — PASS
- `python3 scripts/check_claim_wording.py` — PASS
- `cargo run -q -p xtask -- check-model` — PASS, generated conformance unchanged at 33 IDs
- `cargo test -q -p repo-conformance` — PASS
- `cargo run -q -p xtask -- audit-deferral` — PASS
- `cargo run -q -p xtask -- audit-limits` — PASS
- ignored #931 instrument and full study run explicitly against the canonical local bundle — PASS and byte-identical result regeneration

The previously aborted fixture-present workspace run was not repeated locally as an opaque multi-hour gate. The protected merge queue remains the binding full-workspace, κ, wasm, fuzz, audit, and Gate C verdict.

## Excluded local content

`crates/uor-r4-core/tests/.uor-r4-recorded-corpus-producers/` is generated residue from earlier testing and is intentionally untracked and excluded from this PR.

Closes #931


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

