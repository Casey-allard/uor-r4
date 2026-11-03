# REPORT: public M1 CPU trunk evidence, deepseek-11

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base:
`0c76760e8a99e13c2a5f2ee92db03d871c5ddb90`.

## What did not work

- **The evidence instrument still stopped at writer-only parity.** The merged
  CPU trunk reader existed, but `m2-evidence` continued to classify the
  19-table artifact as unavailable and compared only the writer from the
  serving Q12 query.
- **The first public rerun reused a Cargo target after a test build.** This
  reproduced the repository's known mixed-feature artifact collision before
  the evidence process started. A separate target passed; this was a build
  instrument fault, not a model mismatch.
- **This is not a capability gain.** The public M1 teacher-forced panel remains
  an evaluation instrument. Generated complete replies, generated-reply
  novelty, fresh phrasing, training and GPU qualification remain unmeasured.
- **The reference index is a generated control, not canonical TRAIN.** Its
  overlap strata remain contextual smoke readings rather than a model-quality
  verdict.

## What changed

- `m2-evidence` now dispatches four-table artifacts to
  `CpuDiscreteProposalScorer` and 19-table artifacts to
  `CpuDiscreteTrunkReader`.
- For a supported 19-table artifact, `cpu_engine_parity` now compares every
  complete 32-coordinate CPU query, admitted-cell vector, top-8 and token with
  `IntegerDiscreteModel`. Unsupported schemas remain explicitly unavailable.
- The CLI summary prints `cpu_engine_parity`, and the writer-only scope text
  now names the independent query check rather than calling it unavailable.
- Updated the instrument and line READMEs, roadmap item and notebook row with
  the new CPU boundary.

## Measurements

Public seed-1 19-table artifact:
`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.

| Reading | Result |
|---|---:|
| Panel response targets measured | 616 |
| CPU decision rows | 616 |
| Complete Q12 query coordinates checked | 19,712 |
| Query-coordinate mismatches | 0 |
| Admitted-cell mismatches | 0 |
| Top-8 mismatches | 0 |
| Token mismatches | 0 |
| Admitted targets | 365 |
| Top-1 | 293 |
| Top-8 | 306 |
| Writer-only parity rows | 616 / 616 exact |

The `independent_cpu_trunk_query` entry is no longer in the report's
`unavailable` map. The full evidence summary is
`docs/labs/one-model-end-to-end/evidence-deepseek-11-trunk-parity.json`
(SHA-256 `d93d61bf0ee25825812522f49f7d82552d95c4f9601f54f0baa4cf49a843c12c`).
The complete generated report SHA-256 is
`91bcc770b26493ada06150585dbf2701a796ad1dda86bda9d2f8458532d07ebf`.

## Checks run at the head

- `cargo test --offline --features cpu-parity --lib --bin m2-evidence -j1`:
  14 library tests and 3 CLI tests passed.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`: passed.
- The public `m2-evidence` run over the pinned 616-target panel passed with the
  measurements above.
- Targeted `rustfmt --check` passed.
- `git diff --check` passed.

## Skills used

No available Codex skill applied. The existing evidence lane, CPU trunk reader,
public fixture and repository delivery rules were read directly.

## Still open

- Generated complete-reply correctness and novelty.
- Fresh-reply phrasing and an honest canonical-TRAIN reference index.
- Count-table, memory-off and no-read arm artifacts.
- A trained artifact carrying learned Hamming, turn-memory and point-key
  records, followed by the relevant GPU qualification.

## Plain summary

The existing public 19-table M1 evidence report no longer has to trust the
serving query before checking the writer. It now recomputes the whole CPU
decision itself and matches all 616 measured targets across 19,712 query
coordinates, admitted cells, top-8 choices and tokens. This removes an
evaluation-instrument gap; it does not prove that the model can answer complete
conversations.
