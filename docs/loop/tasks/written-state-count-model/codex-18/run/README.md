# Wick dispatch for the fixed-writer factorial

Use the two seed carriers from the completed D17 run at `20261119-value-ranking/live-rank-d4688456ef77`. The ordered, ranked, veto and combined policies embed the same byte-identical ordered bank for each seed. The positional comparator embeds its own frozen D17 bank and has the same allocated and used record counts. Packaging checks that the repacked ordered and positional artifacts reproduce their D17 artifacts exactly, and that the five artifacts for each seed have distinct digests and equal size.

## Stage and authorize

Use a new Wick stage with empty `bin/`, `sources/` and `panel/`. The source snapshot includes the native Rust/CUDA scorer, the minimal integer Cargo workspace, and its original D17 validity tools under `docs/loop/tasks/written-state-count-model/codex-17/validity/`. The panel directory holds the frozen seed paths, the panel custody manifest, both independent reader files, its adjudication, the tracked grader adapter and its exact output, and the independent names-only endpoint seal.

```bash
./build-wick.sh "$stage"
./pack-wick.sh "$stage"
./stage-wick.sh "$stage"
```

The build records source, executable and CUDA instruction manifests. Packaging uses the archived fit packet, labels, bank and profile with no optimizer update. Staging retokenizes from the sealed unmarked text with both pinned tokenizers and compares against the frozen tapes. It binds the staged artifacts, old writer chain and original D17 fit inputs, runner and grading files to `inputs.sha256`.

Run exact-stage finite controls before authorization. The top session supplies the absolute GO receipt, non-author FIT, finite receipt manifest, and pre-run review. The GO JSON uses schema `uor-r4.value-start-factorial-go/1`, decision `GO`, and the digest fields `inputs_sha256`, `non_author_fit_sha256`, `finite_controls_sha256` and `pre_run_review_sha256`. The stage and receipt digests below come from the approved review.

```bash
cd "$stage"
sha256sum --check --status finite-controls.sha256
"$stage/run/run-wick.sh" "$stage" "$go" "$go_sha" "$fit" \
    "$stage/finite-controls.sha256" "$review" "$stage_sha" "$new_output"
```

The dispatcher checks for an empty compute-app list before each seed/repetition and records the observed occupancy. Each CUDA stage uses the existing Wick advisory lock. Individual stages require at least 4 GiB of free device memory and 8 GiB of available host memory. A failure stops the run with phase status and raw outputs preserved.

## Observations

Each cold process regenerates the public and independently supplied requested-endpoint captures using the original role chain and exact writer. Before protected selector scoring, the unchanged D17 validity tool checks the captured matched signatures, three distinct reachable original writes, the 8/24 ceiling, all 36 held gold sources, and the 88 original FIT rows. A structural availability check confirms the frozen ordered and positional banks have the required classes and radii. The original CUDA writer then independently executes gold and both real rivals. The wrong-key grader must reject the unchanged gold outputs.

The ordinary client receives unmarked text and the selected artifact through protocol-2 `--say`. For each of the five policies, CUDA/Rust comparison covers the scorer's raw words. The original writer receipt must then match the ordinary client's role/source, emitted IDs through EOS and cursor words. The read-disabled control opens the combined-policy artifact. The requested-endpoint intervention uses the same five policies and writer. Its full original-ID tapes are retained and its supported rows are graded in a separate summary.

The seed directory exposes `summary.json` for the ordinary and control results, `requested-summary.json` for the supplied-endpoint diagnostic, `grades/` for observed tokens and first mismatches, and `modes/` for raw and decoded native traces. `raw.sha256` preserves run-specific logs and receipts. `cold.sha256` covers stable model decisions, exact writer tapes, client records and grades for byte equality across repetitions.
