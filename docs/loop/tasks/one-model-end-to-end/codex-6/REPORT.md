# REPORT: one-model-end-to-end, codex-6

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Order: [ORDER.md](ORDER.md). Result: [PARITY-RESULT.md](PARITY-RESULT.md).

## What did not work, first

- The first Route trace attempt returned 32 zero words per row: `discrete_forward` still passed
  the trace pointer only for active-leaf modes. Passing the same pointer in Route mode fixed it.
- The first implementation changed the independently qualified bucket-writer source hash even
  though the trace was a read-only observation, then briefly compiled two Route bodies. The final
  version has one trace-capable production Route read; the bucket generator strips marked
  trace-only blocks, checks the remaining qualified shape and pins its existing source hash.
- The full CPU-training suite reports two failures in FBS ask/carry parity. Claude-371 already
  recorded both failing at the parent base with its change set aside. They are outside this
  trace unit; all other CPU-training tests pass.
- The exact-head review at `1c925165e` found two P2 issues: an untraced Relation read still
  allocated and copied trace-only relation scores, and the active-slot control asserted
  non-vacuity without pinning the measured 126 words.
- The next exact-head review found two more P2 issues: Relation trace layouts differed between
  CUDA and serving, and the temporary Route trace helper duplicated the production Route body.
- The following review found the bucket generator still carried a frozen Route body, CUDA's
  `relation_scores()` retained padded rather than compact semantics, Relation fault controls were
  missing, and the ORDER's file scope was stale. All four are fixed at the head recorded in Git
  history and are submitted for delta re-review.
- Its final pass found that untraced Relation forwards still built row-node metadata and that the
  ORDER omitted the CPU re-export file. The metadata is now trace-only, the public regression
  observation is pinned, the padded layout has a structural guard and the ORDER lists `lib.rs`.

## What changed

- `IntegerDiscreteModel::evaluate_prefix_traced` exposes the exact Route and active-leaf words
  produced by the same serving read. `evaluate_prefix` still takes the untraced path.
- `CpuDiscreteProposalScorer::query_traced` and the public Route/Flock/Relation readers expose the
  CPU oracle words.
- `Device::forward_traced` now supports Route as well as Flock/Relation.
  `Forward::read_trace` returns the mode-specific raw words; ordinary `forward` allocates none.
- Route header and stage words pin the configured frame and selected port. Active-leaf levels
  trace selection even at gate zero, matching CUDA.
- Relation keeps compact scores in `evaluation.relation_scores` and exposes separately padded
  `relation_trace_words`, matching the CUDA batch tail without mixing row boundaries. CUDA's
  `relation_scores()` compacts each block back to its row's actual node count.
- CUDA retains Relation row-node counts only for traced forwards; ordinary Relation `forward()`
  keeps both the trace and its metadata absent.
- The synthetic provenance loaders and exact apply alias exist only behind the non-default
  `qualification` feature. Ordinary builds cannot reach the private training update hold, and the
  focused CUDA contract runs with `--features cuda,qualification`.
- CUDA compiles one `route_vector` with an optional trace sink. The bucket generator's explicit
  checked un-instrumentation strips marked trace blocks from that same body and preserves the
  already-qualified bucket source; a mutation of a decision line reaches generation and fails the
  receipt.
- The 39-prefix contract compares every trace word through CUDA, CPU and serving. Fault controls
  retain the output-only counts and add direct trace counts.

## Measurements

Device and build:

- RTX 4090, compute capability 8.9, driver 595.99.02.
- CUDA 13.1, AOT `compute_89`.
- 39 prefixes; Route, Flock and Relation fixtures; CPU debug test build.

Parity:

- Route, Flock and Relation: every read vector, admitted cell, top-8, token, base trace word and
  padded Relation-tail word matched across CUDA, CPU and serving.
- Route, Flock and Relation: all 1,872 carry words and all 48 counterfactual losses matched.
- Route candidate 7, Flock candidate 1 and Relation candidate 2 applied, epoch `0 -> 1`.
- Untraced and traced forward returned identical public words; the untraced result held no trace.

Fault visibility:

- Route frames: direct trace `252 / 252`; output-only K1 remains `108 seen / 144 missed`.
- Flock gates: direct trace `8 / 8`; output-only K1 remains `3 seen / 5 missed`.
- Relation level gates: direct trace `12 / 12`.
- Active trace: 126 kept-node words and 126 transport-action words on each of the Flock and
  Relation fixtures. Recomputed table-root mutations changed 54 kept / 90 action decisions on
  Flock and 12 kept / 90 action decisions on Relation.
- Relation: 258 compact score words and 1,014 padded trace words over the 39 prefixes.
- One planted carry word and stale proposal reuse remained detected.

No training run, capability, language result, throughput or performance claim is made.

## Checks run at the head

- `cargo test --locked -p uor-r4-cpu-train
  native_space_cuda_parity_contract_runs_the_same_words --features cuda,qualification --lib` -
  passed on the RTX 4090.
- `cargo test --locked -p uor-r4-cpu-train
  native_space_parity_contract_catches_one_planted_fault_per_category --lib` - passed.
- `cargo test --locked -p uor-r4-cpu-train
  native_space_trace_contract_exposes_active_leaf_kept_slots --lib` - passed.
- `cargo test --locked -p uor-r4-cpu-train
  native_space_relation_trace_keeps_its_scores_separate --lib` - passed.
- Full `uor-r4-discrete --lib`: **356 passed, 0 failed, 1 ignored**.
- Full `uor-r4-cpu-train --lib`: **509 passed, 2 failed, 5 ignored**. Both failures are the
  parent-base FBS ask/carry tests named above. With those two named tests filtered:
  **509 passed, 0 failed, 5 ignored**.
- `bucket_source::tests::committed_gpu_qualification_receipts_pass_b1_b2_b3_and_match_across_builds`
  - passed; the bucket helper/writer source and its committed receipt are unchanged.
- `rustfmt --check` on the changed Rust files - passed.
- `cargo clippy --no-deps` for `uor-r4-cpu-train` (with `cuda`), `uor-r4-discrete` and
  `uor-r4-integer` - passed; repository warnings remain.
- `UOR_LAB=codex uor-harness check` - **0 errors, 314 warnings**.
- `loop pr-check --local --base e978e53ef` - **PASS**, with the repository's scope, commit-order
  and FILEMAP advisories.
- Fresh-context exact-head review: the first pass returned CHANGES with the two P2 findings above;
  the repaired head is submitted for delta re-review. External PR publication requires owner
  approval under the repository AGENTS instruction.

## Skills used

- `uor-research-architect`: kept the unit finite, artifact-bound and separate from language
  qualification.
- `working-discipline`: read the existing state first, compared the shipped paths, retained row
  words and reported the parent-base failures separately.
- `compaction`: recovered the interrupted CUDA-parity state without reconstructing measurements
  from memory.

## Still open

- Loader symmetry (roadmap step 2).
- Shared typed operator definitions (step 3).
- Table/Route/Gate update-status parity (step 4).
- Generation and stop state (step 5).
- Exact copy and turn-memory carry (step 6).

## Plain summary

The model now shows which Route frame and port it chose, and which nodes, transport actions and
Relation scores each active read used, as the same raw words from the RTX 4090, the CPU oracle and
integer serving. The previously invisible 144 route faults and five gate faults are now directly
observable. This is still finite construction parity, not training or language capability.
