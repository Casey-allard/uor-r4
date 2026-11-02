# REPORT: CPU trained-writer full-path audit, deepseek-10

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base:
`3063b20c7d1f8a6a75d52cf56675a86193b20c1e`.

## What did not work, first

- **This is not a trained Hamming, memory or point-key model.** The fixture's
  structural records are synthetic. Only its writer table is trained. It proves
  the trainer and audit preserve those records, not that the records were
  learned.
- **A public trained full-path artifact is still unavailable locally.** The
  published `/7` and `/8` artifacts are in the private HF store, and this host
  has no readable HF credential or local copy. The public dual-c131 artifacts
  have no Hamming or memory records.
- **Reusing a Cargo target after a focused test hit the repository's known
  mixed-feature artifact collision.** The failing rerun did not reach a source
  test. A fresh release target passed the focused test and then the complete
  release library suite.
- **No quality, corpus-gain, complete-reply or fresh-phrasing result is made.**
  Eight constructed admitted samples exercise exact update acceptance; they are
  not a language-training comparison.

## What changed

Added `a_trained_writer_preserves_the_full_served_path_and_exact_audit` in
`crates/uor-r4-cpu-train/src/train.rs`.

The deterministic fixture is a synthetic `/8` artifact with:

- Hamming copy/type reads at stream and readout depths;
- one opening-memory row and two reply-memory rows;
- a point-key fallback row; and
- the native cell-writer head.

The existing `CpuRankMarginTrainer` finds eight admitted samples and accepts all
eight updates. The test requires:

- bytes outside `writer.table` remain identical;
- the trained artifact parses;
- the trained session still takes point-key token 99 at the missed opening;
- the trained complete served decision differs from the untrained control;
- Hamming, opening/reply memory, point-key and native-head manifests remain
  active;
- Hamming, memory and readout work remain non-zero;
- query-coordinate, token and kept-cell parity is exact for both artifacts;
- repeated audit JSON is byte-identical; and
- checkpoint resume reproduces the trained artifact and checkpoint bytes.

## Measurements

- Training: **8/8** admitted steps accepted.
- Untrained and trained audits: 5 positions, **320** Q12 query coordinates,
  **5** token decisions and **40** kept cells each, with zero mismatches.
- CPU and serving decision hashes matched in both audits.
- The trained artifact's complete decision hash differs from the untrained
  artifact.
- The trained artifact returned token **99** at the point-key miss.
- Checkpoint resume preserved the trained artifact and checkpoint bytes.

The test also asserts non-zero modelled Hamming and turn-memory work. This is a
component/instrument result. It closes the train-to-audit preservation gap for
this fixture, not trained full-path weights or a capability claim.

## Checks run at the head

- Focused release test in a fresh target: 1 passed.
- `cargo test --locked -p uor-r4-cpu-train --release --lib` in a fresh target:
  126 passed, 0 failed, 4 ignored.
- Targeted `rustfmt --check` on `train.rs`: passed.
- `git diff --check`: passed.

## Skills used

No available Codex skill applied. The existing CPU trainer, full-path audit,
artifact formats and repository rules were read directly.

## Still open

- A genuinely trained full-path artifact, including learned Hamming, memory and
  point-key weights, still needs a readable source and a separate validation.
- Two-seed or corpus-scale train-to-audit controls, complete replies and fresh
  phrasing remain outside this host fixture.
- Candidate export scripts still need to adopt the served-forward artifact gate
  universally.

## Plain summary

The CPU trainer can now update a full-path artifact and immediately prove that
the trained bytes still execute exactly through Hamming reads, turn memory, the
point-key fallback and the native head. The attached mechanisms are structural
test records, so this shows preservation and auditability, not learned memory.
