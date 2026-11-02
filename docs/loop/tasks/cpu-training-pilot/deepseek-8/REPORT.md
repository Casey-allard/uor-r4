# REPORT: cpu-training-pilot, full-path FBS audit coverage

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `942d0086a`. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The first reply-table keys did not fire.** They included the leading BOS
  token even though the memory turn anchor starts after the latest BOS/EOS
  boundary. The repaired keys start with `[42, 77]`; both reply rows now change
  their Q12 query relative to a reply-less control even though the downstream
  head happens to choose the same token.
- **A shared target first made the isolated CPU test look like a source
  failure.** Reusing that target reproduced the existing incompatible-feature
  artifact collision. Every accepted run below used a fresh
  `CARGO_TARGET_DIR`; no source workaround bypassed the build graph.
- **No readable trained full-path artifact is available.** This is synthetic
  mechanism coverage, not trained-model or language evidence.

## What changed

Added one deterministic served-forward audit test in
`crates/uor-r4-cpu-train/src/served_forward.rs`. Its compact three-layer,
64-coordinate artifact activates:

- four-head Hamming copy/type reads at stream depth 1 and readout depth 3;
- an exact opening lookup that deliberately misses;
- a one-row point table that names token 99 on that opening miss;
- two exact reply-table rows across the following reply steps; and
- the native vector writer head.

The control artifact removes only the point table. The test checks complete
parity, report determinism, mechanism manifests and non-zero stage costs, then
observes the point fallback's token-level change from the control. A second
control removes the reply table; both reply rows must change their complete Q12
query relative to it.

## Measurements

The five-token sequence is `[0, 42, 77, 11, 22]` with reply start at one-based
position 3. Conditions: synthetic seed 705, width 64, three layers, context 20,
and a fresh release target.

- parity checked 5 positions, 320 Q12 query coordinates, 5 token decisions and
  40 kept cells, with zero mismatches;
- CPU and serving decision hashes were equal and repeated reports serialized
  byte-identically;
- stream and readout Hamming stages each modelled 24 candidate comparisons and
  24 popcount words over the sequence, with non-zero typed Hamilton work;
- the memory stage modelled 3 exact key lookups and 1 point-table row scan on
  the missed opening;
- both reply rows changed their complete Q12 query when the reply table was
  present, proving that their exact keys fired rather than merely contributing
  to the declared lookup count;
- the point-table artifact returned token 99 at the opening row; the otherwise
  identical control without the point table did not;
- the native readout modelled 80 candidate-token visits over the sequence.

The public artifacts still carry neither Hamming nor turn memory. No trained
full-path artifact or credentialed panel was available.

## Checks run at the head

- Focused release test
  `full_path_audit_covers_hamming_memory_point_key_and_native_head`: passed in
  a fresh isolated target.
- Full `uor-r4-cpu-train` release library suite in a fresh isolated target:
  125 passed, 0 failed and 4 ignored release benchmarks.
- `cargo clippy -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`
  passed. Dependency-inclusive Clippy is blocked by 50 pre-existing findings in
  `uor-r4-integer` under the pinned newer toolchain; none is in this change.
- The inserted source region has no rustfmt diff. File-wide
  `rustfmt --check` still reports pre-existing formatting in unrelated lines.
  `git diff --check` and `uor-harness loop report --check --order` passed.
- Loop report/pr checks and exact-head review are recorded in the PR review
  record.

## Skills used

No repository skill was invoked. The unit follows the existing
served-forward audit and synthetic FBS fixture patterns.

## Still open

- A readable trained full-path artifact must replace the synthetic fixture
  before this can support a trained-model claim.
- The audit models memory key lookups but does not currently expose per-step
  hit counters; reply and point activity are established through their
  token/query controls and exact parity rather than a direct hit bit.
- Training through the complete served graph, the bounded surrogate comparison
  and Claude's end-to-end local-chat qualification remain tracked by the active
  program sequence.

## Plain summary

The CPU audit previously had separate evidence for Hamming reads and turn
memory. This unit runs both mechanisms in one deterministic synthetic artifact
and compares every served decision between the independent CPU implementation
and the integer serving session. It also forces an opening-key miss so the point
table changes the predicted token, then switches the point table off to prove
that the fallback caused the difference. The evidence proves that the audit can
cover the composed path; it does not claim that a trained full-path model has
been tested.
