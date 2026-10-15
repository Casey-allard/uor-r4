# Route learning-slot receipt tooling — 15 October 2026

**Source/tooling ready; actual final analysis NOT_RUN.** Codex/Sol, base `a6cd1d32d`.
[PM assignment 6181283888](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6181283888)
asks for the learning-slot evidence at completed cursor **16,404**, after seed-1 seals.
This delivers a standalone Rust metadata binary and final-quarter runbook wiring,
not a model, GPU, training or live-pod action. Line: evaluation integrity; the M1
headline remains 37/232 and its research-line count is unchanged. Decision: KEEP
source tooling; no mechanism or learning-schedule adoption.

## Exact scope and inputs

`docs/labs/completion-scoring-readiness-2026-10-14/scoring-audit/src/bin/route-learning-slot.rs`
is a second binary in the existing standalone receipt-tool package, with only
`serde_json` and `sha2` dependencies. The audited scorer's `src/main.rs` is unchanged;
`default-run` preserves its existing `cargo run` invocation. The quarter runbook
builds both binaries and invokes the new one only at Q4, after scoring succeeds.

The command accepts a successful `score-quarter.sh 16404 ...` directory and a new
output path. It checks `audit.json`, `seed1-run-state/evaluation-16404.json`, its
`.pin.json`, and `seed1-attempt-1/checkpoint-16404.json`. The compiled-in canonical
`SEED1_PLAN.json` pins the source/configuration and plan-byte identity; the pilot
baseline hash remains `e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3`.
The runbook's existing pin-after-transfer comparison and complete row audit remain
prerequisites. The sidecar does not reimplement quality scoring or authenticate an
owner release. Its input hashes bind bytes, not signatures or third-party provenance.

Refusals cover nonfinal/pilot/seed-2 input, wrong evaluation/checkpoint/pin/admission,
wrong baseline/plan, wrong Route model/configuration binding, lineage or mutation
flags, cursor/epoch inconsistencies, missing or unexpected counter kinds and
impossible counter ordering. Counters must agree across audit, evaluation and
checkpoint. The parser limits **each input to 128 MiB**, checks length before reading
and caps the reader at limit+1 to catch concurrent growth. This is a receipt resource
limit, not a scientific acceptance bar. All validation precedes exclusive output
creation; existing output files are preserved.

## What the output can establish

The frozen schedule maps indices 0..16,384 to VectorBlock, then indices
16,384..16,404 to Route coordinates 0..20 (both upper bounds exclusive), as defined
in `training.rs::coordinate`. The registered completion executor handles VectorBlock
and Route and rejects other coordinate kinds (`completion.rs`, counter update match).
No live source or executable is changed by this tool.

- **VectorBlock:** observed cumulative attempted / candidate-cost-differs / accepted
  counters, including the inherited pilot. Final attempts must equal 16,384.
- **Route:** observed cumulative counters; all 20 attempts belong to the final read
  slots because no earlier Route proposals occur in this pass. Output explicitly
  answers `attempted_any`, `cost_differed_any`, and `accepted_any`.
- **Gate:** not admitted by this frozen Route executor; `observed_counters: null`.
  False attempted/accepted flags are source-derived exclusion, not a measured Gate
  control and not a claim that Gate learning fails.

The three possible Route summaries are `ROUTE_ATTEMPTED_NO_CANDIDATE_COST_DIFFERENCE`,
`ROUTE_COST_CHANGED_NO_UPDATE_ACCEPTED`, and `ROUTE_UPDATE_ACCEPTED`. These describe
stored observations only. No threshold closes or promotes a mechanism.

No step receipts are consumed: the list of Route coordinate indices is explicitly
**schedule-derived**, and accepted coordinate identities / parameter changes /
step-level losses remain unavailable here. Q3→Q4 is not an isolated Route-quality
delta because it also includes VectorBlock updates. A single seed supplies neither
fresh transfer nor the registered two-seed conclusion. The Relation learning sheet
still needs a declared interleaved schedule, its own FIT and the owner's release.

## Development controls and use after the final seal

Only synthetic JSON metadata fixtures were executed. Four Rust tests cover the three
outcomes; missing/noninteger/impossible counters; nonfinal, wrong pins, wrong seed,
wrong reader, lineage, epochs and unexpected Gate fields; file round-trip, byte
mutation, output preservation and oversized-input refusal. These are instrument
controls, not analysis of actual pilot/quarter receipts and not model computation.

```sh
export PATH="$HOME/.cargo/bin:$PATH" UOR_LAB=codex
CARGO_TARGET_DIR=local/scoring-target cargo build --release --locked --offline \
  --manifest-path docs/labs/completion-scoring-readiness-2026-10-14/scoring-audit/Cargo.toml
CARGO_TARGET_DIR=local/scoring-target cargo test --locked --offline \
  --manifest-path docs/labs/completion-scoring-readiness-2026-10-14/scoring-audit/Cargo.toml \
  --bin route-learning-slot
```

After Claude seals Q4, follow the [quarter runbook](../completion-quarter-runbook-2026-10-14/README.md).
The full script saves both audits and executable/input hashes. Reusing an already
verified Q4 directory is also documented there. Post final cursor and input/executable
pins, per-kind counters, the Route observation, Gate exclusion and limitations on
#2029; this is the evidence the later learning-sheet author consumes. No schedule
choice is made in advance of the observed counters.

Source-only authorization; no CUDA dependency, pod contact, paid resources, training
or hold change. Seed-1 is untouched; seed-2 and M2 remain HELD. Object-store #2487 is
a separate reviewed contract and is not implemented in this PR.

Validation on the delivered source: both receipt binaries build offline in release
mode; `cargo test --bin route-learning-slot` passes **4/4** synthetic tests
([log](synthetic-tests.txt)); focused `cargo clippy --bin route-learning-slot -- -D warnings`,
package `cargo fmt --check`, `bash -n score-quarter.sh`, claim-wording and `git diff
--check` pass. D23/D25 remain 1,641/93. The broader initial `clippy --all-targets`
command also selected the unchanged scorer and hit its pre-existing
`single_element_loop` at `src/main.rs:433`; that scorer is preserved byte-for-byte,
and no full-package Clippy pass is claimed. No tests consuming real pilot/final
receipts were run for this delivery.
