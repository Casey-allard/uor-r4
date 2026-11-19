# Requested-occurrence replay

D19 uses the ordered D18 artifact and 8,416-byte fitted bank for each seed. A new treatment artifact differs only by the packed requested-role policy and its bound protocol framing. The original D18 artifact must compare byte for byte with the rebuilt baseline. The archived assertion fence, agreement operator, ordered value scorer, exact writer and tokenizers come from the D18 stage and its pinned upstream chain.

## Stage

Restore the complete D18 stage at the original paths. Create a separate D19 stage alongside it with `sources/`, `panel/`, `bin/` and `build-receipt/`. Source and executable manifests have paths relative to the D19 stage. The required new binaries are `uor-chat`, `pack-all-starts`, `project-role`, `requested-role-device`, its nvcc executable `requested-role-sm89`, and the existing independent regrader `fresh-validity-check`. Build the regrader from the D17 `regrade/Cargo.toml`; its optional row-count arguments bind this panel to 70 total and 64 supported rows. The four-path artifact packer still constructs the baseline byte-for-byte.

With the two independent readers and panel manifest sealed before scoring, run `stage-inputs.sh ABS_STAGE` from the staged source tree. It retokenizes both seed panels from unmarked text, verifies the complete prompt tapes and requested spans, and writes `specs/seed{1,2}.json` and `inputs.sha256`. The stage step packs and checks inputs and does not replay held rows.

## Dispatch

After passing finite device controls, independent FIT, and pre-run review on the leased device, invoke

```bash
dispatch.sh ABS_STAGE ABS_GO GO_SHA ABS_FIT ABS_FINITE ABS_REVIEW STAGE_SHA ABS_NEW_OUT
```

The GO JSON uses schema `uor-r4.requested-occurrence-go/1`, decision `GO`, and four digest fields `inputs_sha256`, `non_author_fit_sha256`, `finite_controls_sha256` and `pre_run_review_sha256`. The finite receipt manifest contains absolute or stage-relative paths, verified before seed replay. Supply a new output path. Each primary and cold repeat starts a separate process and records occupancy, device UUID, timing, phase and raw hashes. The dispatcher compares the stable replay hashes across repeats.

## Observations

The first GPU stage receives only the frozen baseline or treatment artifact and blind token tape. It separately computes eight role words per row, checks exact agreement with the Rust role operator, and binds each row to its little-endian prefix digest. Treatment role words are applied to the legacy model to collect assertion witnesses. The original CUDA assertion fence, agreement and writer verify those witnesses before the frozen ordered value scorer runs. The supplied requested span and opposing real query span are diagnostic interventions. The archived writer must reproduce both forced full literals through EOS, and the wrong-key check must reject the unchanged forced-gold stream.

The ordinary client uses unmarked protocol-2 `--say` with the baseline or treatment artifact. The grader checks its role and source, every emitted ID and every raw cursor word against the CUDA replay. Read-disabled uses the treatment artifact. Labels and interventions are confined to the CUDA controls and post-selection grading. `summary.json` reports per-row role and source classes, expected and actual first mismatch, EOS, paired gains and regressions, negative categories and the held-factor cells. The 48 novel-literal and 16 overlapping-literal subgroups use the panel's pre-run novelty census. Raw device and scorer traces, client receipts, grades, and custody hashes remain in each seed output directory.

The treatment candidate census is in `capture/role-treatment.json`. The selection adapter preserves the baseline candidate metadata for legacy readers. Its new role words and assertion witnesses are bound to the device census.
