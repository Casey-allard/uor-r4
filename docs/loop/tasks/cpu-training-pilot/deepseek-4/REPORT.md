# REPORT: cpu-training-pilot, deepseek-4

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The existing checkpoint driver refused every trained artifact with a nonzero epoch.** The
  public four-table artifacts are at epoch 420, while the driver required epoch zero. The lineage
  now records the source epoch independently of the accepted-update cursor, so continuation starts
  at the artifact's actual epoch without rewriting its ancestry.
- **The existing path had no artifact writeback.** A checkpoint preserved the model but could not
  produce a serving artifact. The driver now rewrites the four base tables and advances the header
  epoch, then reloads the result through `IntegerDiscreteModel` and checks exact serving parity.
- **The first four-step smoke exposed a final-checkpoint collision.** When the final step also
  matched the interval, the driver tried to create the same checkpoint twice. The final record now
  reuses the already-written interval checkpoint.
- **Independent review caught a final-batch semantic bug.** The first implementation wrapped the
  last batch into the next corpus pass. It now takes the short final batch exactly as the GPU
  `PreparedCorpus::batch` does, and rejects a target cursor outside the eligible rows.
- **The public 19-table M1 trunk is not supported by this path.** The CPU proposal scorer implements
  the four-table Route/Flock/Relation family, not the extra 15 trunk tables. The driver refuses it
  instead of silently ignoring the trunk.

## What changed

- Added `cpu-discrete-corpus-driver`, a bounded CPU corpus loop over the existing exact proposal,
  chooser, apply, BVG-K and checkpoint components.
- Added nonzero source-epoch continuation and exact four-table `UORLUT01` artifact reconstruction.
- Added matched panel readings and CPU-versus-`IntegerDiscreteModel` export parity: query vectors,
  admitted cells, top-8 and top-1 token.
- Added hash-bound sidecars for the source artifact, tokens, mask, manifest, seed, split controls,
  checkpoint bytes and emitted artifact.
- Resume now requires the checkpoint filename, binding step and restored cursor step to agree, and
  checks the sibling emitted artifact against its bound hash.
- Reports separate cumulative cursor state from the resumed invocation's accepted/refused updates,
  context tokens and timing.
- Added focused tests for exact writeback and serving reload, epoch-420 continuation, the short
  final batch, resume binding, the interleaved read boundary, and the existing typed-mutation,
  K-block and process-restart controls.

## Measurements

Public inputs:

- Source commit: `dc2c0e4474bd1b9f377e5ed35d94c81ec072246c`.
- Release binary: `9e5696c9a53a4e46808f7f7f5f50cd037a003fb2f5b724edd74cfb873e97b248`.
- Seed 1 artifact: `fc31bcc7a88250f3758d07a8b3d12480a5332acb5437b0c0f0e5c71ae88939ca`.
- Seed 2 artifact: `a75ea7f5907275fd4c1e853ee76135a7a2615ba8302aaadf362bfa7eb96ae12a`.
- DEV token stream: `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`.
- Response mask: `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.

The bounded implementation smoke was four exact updates on a deterministic 200-document training
split and a 128-row document-disjoint internal panel:

| Seed | Accepted / refused | Epoch | Panel top-1 | Panel top-8 | Admitted |
|---|---:|---:|---:|---:|---:|
| 1 | 2 / 2 | 420 → 422 | 2 → 4 | 9 → 10 | 46 → 46 |
| 2 | 2 / 2 | 420 → 422 | 2 → 4 | 9 → 10 | 46 → 46 |

Both seeds reduced the panel margin-loss sum by 32 and target-rank sum by 135. The rank sum is over
tokens in admitted cells, not all 4,096 vocabulary entries. This is a four-step probe of the
component path, not a capability result: the public artifact's TRAIN exposure is not verified and
the panel is internal to the public DEV input.

- Step-only throughput was 38,842 / 39,484 context-tokens/s and 436 / 443 target visits/s on
  seeds 1 / 2. Whole process wall time was 0.906 / 0.879 s; peak RSS was about 19.0 MiB.
- Exact CPU/serving export parity held on both source and rewritten artifacts: 128 rows, 4,096 Q12
  query coordinates, zero query, admitted-cell, top-8 or token mismatches.
- A separate process resumed seed 1 from step 2 to step 4. Its final artifact
  `ad92bb38315b94ed6b84e68d0231990be2ea88054eabd33c2b6d238dbbe77828` and checkpoint
  `6b36a989d23c15d53ff988089433d488fa661d19f4ab69b3cefa1953c0c6dbdd` were byte-identical to the
  uninterrupted run. The resumed invocation executed two batches, accepted zero and refused two;
  its report names those as invocation-local counts while retaining the cumulative `2 accepted /
  2 refused` cursor.

## Checks run at the head

- `cargo test --locked -p uor-r4-cpu-train` in a fresh isolated target directory: 114 library
  tests, 3 existing FBS-driver tests and 4 new discrete-driver tests passed; four release
  benchmarks remained ignored. The `cpu_discrete` library subset was 12/12.
- Exact cargo clippy: `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`:
  passed.
- Targeted `rustfmt --check` on all changed Rust files: passed.
- New discrete-driver unit tests: 4/4. Existing resumable FBS driver tests: 3/3.
- `cargo run --locked -p xtask -- roadmap check`: 134 items OK.
- `uor-harness loop gates --lab deepseek --full`: 11/11 green on retry. The first run hit the
  existing loop-drive mock-`gh` `ETXTBSY` race; the identical retry passed.
- `uor-harness check`: zero errors and 40 pre-existing advisory warnings.
- Public zero-update and four-step process runs on both four-table seeds: completed.
- Public process resume from step 2 to step 4: final artifact and checkpoint bytes matched.
- Evidence `sha256sum -c SHA256SUMS`: passed.

## Skills used

None.

## Still open

- More than K=1 coordinate blocks are not yet driven by this CLI. The underlying K-block and BVG-K
  checkpoint components remain available and tested.
- The 19-table M1 trunk needs a CPU writer for its 15 additional tables before this driver can
  continue that model rather than only its four-table base.
- The four-step smoke does not reach the last batch or an interleaved Gate read. Those are covered
  by focused tests, not by a public corpus capability run.
- A pre-registered, FIT-reviewed configuration must establish whether these exact CPU proposals
  improve held-out ranking or complete replies.
- GPU qualification, sealed fresh panels and credential-gated complete-reply data remain outside
  this CPU component result.

## Plain summary

The CPU can now run the exact discrete four-table training decisions over a guarded corpus, save
and resume their complete state, and emit an integer artifact that the serving reader reloads
exactly. A two-seed four-step probe worked, but it proves the path and not that the model learned
anything useful.
