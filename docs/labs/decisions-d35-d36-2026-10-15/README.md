# DECISIONS.md D35 + D36 entries (2026-10-15)

Canonical lab record for the PM records PR adding two owner decisions to
[`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md).

## D35 — Seed-2 packet re-issue required before any seed-2 GO; registered seed-2 held for the admission iteration

Records the owner decision relayed via Claude (#2028 comment 6181407533; PM record
6181460874): any seed-2 GO requires a re-issued packet with the checkpoint-zero
guard in the binary — newly qualified CUDA build from a named `main` commit, new
executable/driver-source/lineage pins, `completion-check-plan` to
`PLAN_METADATA_VALID`, rebuilt quarter scorer, owner-personal release. Section 4
records the same-day (~07:55Z) owner decision (#2029 comment 6187197075) holding
the registered seed-2 for the admission-objective iteration: the re-issue applies
only if the owner later GOs the registered seed-2; the active line is
`docs/harness/prep/m1-discrete-rung0-admission-2026-10-15.md`.

## D36 — GitHub Pages: keep the last good deploy live; deploy job skipped until a D11 served wasm engine lands

Records the owner decision delivered in Claude's session (#2028 comment
6185876280), superseding the same-morning "drop R4G1 from wasm" relay
(#2028 comment 6184702464): the live site stays as the last good deploy; the
deploy job is skipped (`if: ${{ false }}`, reason and re-enable condition
in-file) until M4 lands a D11 served wasm engine. R4G1 source preserved; router
engine behind the non-default `legacy-engine` feature. Out-of-scope breakage
(native `graph-compiler`/`graph-certify`/`api`/`proof-model`/root crate on the
retired `transformerless::{compiler, runtime}`) is filed for its own owner
decision.

Status: records only. No compute, no hold changes.
