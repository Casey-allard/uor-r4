# #866 — fix(#865): deterministic hierarchical_codes + re-pin 360M source-of-record (#833)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-20T07:11:53Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-08-20T07:33:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/866
- **Merged:** 2026-08-20T07:33:32Z
- **Merge commit:** 19465441e21c5e8777e7d0346beb1c40b8f94a2b
- **Base / Head:** main / UOR-Foundation:issue-865-hierarchical-codes-determinism
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Clears the first two HELD preconditions of the #833 attested broad-bundle rebuild,
and reduces the third.

**(1) Determinism defect (#865).** `induce_hierarchical_codes`
(`crates/uor-r4-core/src/transformerless/compiler.rs`) sorted `frequent_paths` by
transition count alone. Count is not a total order, so tied paths kept `HashMap`
iteration order (`RandomState` is seeded per process) and `take(100)` selected a
run-dependent subset — a byte-reproducibility violation isolated to
`hierarchical_codes.json` (the #833 preflight measured it). Fixed with a total
tie-break: count descending, then path bytes ascending.

**(2) Source re-pin (#833).** `models/smollm2-360m-instruct.json` now records the
local snapshot as the content-addressed source-of-record so reconstruction no longer
depends on a dead HF revision.

## Scope and non-goals

- **Scope:** one compiler sort + a regression test; one descriptor file; the #833
  record update.
- **Non-goals:** the flagship rebuild itself; any numeric/quality change (the *set*
  of frequent paths is unchanged — only the previously-ambiguous order among ties
  becomes fixed).

## Acceptance-criteria status

- Determinism — **met**. Two fresh sequential `compile-recorded` runs (isolated
  caches) now produce a byte-identical `hierarchical_codes.json` (previously
  differed); `tless_artifacts.bin` (κ `blake3:0b85c43d…`), `tless_store.bin`,
  `hamming_calibration.json` were already byte-identical and remain so.
- Re-pin — **met**. `source_kappa blake3:eb23c3e8…` (loader-emitted, verified
  against the local `model.safetensors`), `source_bytes 723674912`,
  `source_kappa_scope model.safetensors`, `revision a10cc1512…` (byte-identical,
  previously-fetchable release; the old `2366112999…` 404s upstream).

## Tests and verification

- New unit test `hierarchical_codes_determinism_865` (pins the total tie-break) — pass.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -D
  warnings`, `cargo build --target wasm32-unknown-unknown --lib`,
  `cargo run -p xtask -- validate` (check-model / audit-limits / audit-deferral /
  gnaf), `python3 scripts/check_claim_wording.py` — all pass.
- End-to-end determinism re-verified with the rebuilt release binary (two sequential
  `compile-recorded` runs, `cmp` byte-identical).

## Evidence artifacts, compatibility, conformance, claim boundary

`docs/attested_broad_baseline_833.md` updated (append) with the resolution. No
format/ABI change; no numeric change; no conformance-row change (R1 unaffected — the
descriptor is `models/*.json`, not `model/*.toml`); no claim-boundary change.

## Negative / unavailable results

None regressed. The re-pin's `source_kappa` is the loader-emitted value, verified by
loading the local weights (it differs from a differently-labeled hash in the P3
record; the verified loader value is authoritative).

## Intentionally excluded

The flagship rebuild remains #833's own work — reduced to a ~2–3 h compute window
because the raw teacher observation (`.uor-models/obs-wiki-360m`, 256 shards) is
reusable, so the ~6 h re-observe is largely avoidable.

Closes #865
References #833


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

