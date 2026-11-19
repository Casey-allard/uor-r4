# [codex] Bounded #4106 deciding runner

This joins authorized captures to the native CUDA ranker, frozen writer and independently reviewed validity and grading. The portable Rust crate has no embedded research data. Protected dispatch requires the top session's signed spec receipt after non-author FIT. This directory supplies no approval.

## Build and prepare the two specs

Use the central Wick stage `~/uor-codex/20261119-value-ranking`. The binder depends on the same workspace-relative integer crate on Wick and the workstation. The build runs finite Rust tests and does not open any data path.

```bash
./build-for-wick.sh "$reviewed_source_root" "$reviewed_git_sha" "$stage/study-build"
binder="$stage/study-build/target/release/uor-rank-study-bind"
```

For an authenticated source snapshot without Git metadata, append its absolute source inventory and externally pinned SHA-256. The helper verifies all inventory entries before building.

Prepare drafts with actual deployed tool, stage and data paths. Drafts intentionally contain null capture, validity and reducer pins. They cannot pass the launcher.

```bash
./stage-spec-inputs.sh "$stage" 1 "$binder" "$stage/specs/study-draft-seed1.json"
./stage-spec-inputs.sh "$stage" 2 "$binder" "$stage/specs/study-draft-seed2.json"
```

After authorization has produced the four raw capture archives per seed, and independent input validity is complete, generate the deciding specs. Each common fit/held spec has already been pinned by `launch.sha256`. The assembler records the deployed binary digests for review, including both learned normal tools. Both data splits use the independently sealed endpoint files from their own seed.

```bash
./stage-spec-inputs.sh "$stage" 1 "$binder" "$stage/specs/study-seed1.json" \
  "$validity1" "$validity1_sha" "$reducer" "$reducer_sha"
./stage-spec-inputs.sh "$stage" 2 "$binder" "$stage/specs/study-seed2.json" \
  "$validity2" "$validity2_sha" "$reducer" "$reducer_sha"
```

The top-session GO receipt is a separately SHA-pinned JSON with `schema:uor-r4.value-rank-pre-run/1`, `decision:GO`, `spec_sha256`, `non_author_fit_sha256`, and `finite_controls_sha256`. Use the exact same seed spec and GO receipt for its primary and repeat runs.

```bash
./run-wick.sh "$seed1_spec" "$seed1_spec_sha" "$seed1_go" "$seed1_go_sha" primary "$stage/study-1-primary"
./run-wick.sh "$seed2_spec" "$seed2_spec_sha" "$seed2_go" "$seed2_go_sha" primary "$stage/study-2-primary"
./run-wick.sh "$seed1_spec" "$seed1_spec_sha" "$seed1_go" "$seed1_go_sha" repeat "$stage/study-1-repeat"
./run-wick.sh "$seed2_spec" "$seed2_spec_sha" "$seed2_go" "$seed2_go_sha" repeat "$stage/study-2-repeat"
for seed in 1 2; do
    cmp "$stage/study-$seed-primary/study-cold.sha256" "$stage/study-$seed-repeat/study-cold.sha256"
done
```

## Reuse, qualification and receipts

- The study never calls the capture launcher. Each spec binds both fit and held common specs, both primary and repeat capture roots for each split, and each corresponding full raw inventory. The reuse checker recomputes every inventory directly from a nonsymlinked tree and compares bytes, verifies each canonical cold manifest and its equality across the pair, then links the chosen read-only original directories. The raw manifests allow an archived path-specific receipt to differ across cold roots. The actual files must match their own pins.
- Before fitting, the independently generated admissibility receipt must bind the requested captures by SHA-256 and establish 94 fit rows, 88 fit positives, 42 held rows, 36 held positives and 24 matched positives. Grade-only checks stay out of scorer inputs. The fit binder joins supervision to the exact retained assertion and original address on FIT only. Non-supported FIT rows supply no examples.
- All emitted modes have 8,416 allocated bank bytes and equal used-record counts. The matched gate receives only structural membership and blind captured shape plus fitted class support metadata. It calculates no held distance. Whole-bank fit refusal or absent matched structural support is a configuration negative. Individual unseen/radius, tie and nonpositive decisions remain outcomes.
- The frozen writer must complete the 24 matched gold controls and two actual rivals before native scoring. The controlled semantic regrade and later learned-output regrades bind the original GPU writer and generated-ID tapes. Post-seal gold reach is accounted independently of nomination.
- A native selected source must match a reachable original ordinal and assertion index. The binder applies the exact cursor and its delimiter to each candidate separately. It takes the first original source only when all retained assertions complete to the same original ID sequence. A missing completion or conflict supplies no source.
- Diagnostic public/requested learned and WordStart tracks preserve full 42-row writer tapes. Their independent grade views contain all 36 supported rows. Original frozen normal `--say` transcripts and learned normal transcripts each use 42 rows including six controls. The latter use the new packed artifact in the normal client and must match the GPU writer IDs, eight cursor words and role/source trace before grading. Requested-span assistance remains diagnostic.
- The reducer is the independently pinned D17 validity script. Its input includes independent grade receipts, source audits, and native observations where available. Run timings, memory, and refusal logs remain in the raw inventory. The semantic cold manifest omits run-root-dependent audit filenames. Status and raw receipts remain on failure.

## Finite validation

The independent input audit can call `uor-rank-study-bind verify-input-rows BLIND CAPTURE GRADES SUPPLEMENT_OR_DASH`. It validates the original blind file, hashes its actual u32 little-endian tapes, checks the requested capture and grade bindings, and verifies the sealed positive/control roster. Its only output is the exit status and a validation error on refusal. The caller pins the input files and keeps the grade key outside ranking.

Run `reuse-finite.sh` with a new absolute synthetic directory. It exercises both phase selections, cold equality despite a raw-only receipt difference, then changes a raw-only file and requires refusal before linking. The Rust unit checks native first-source agreement, disagreement, missing completion and invalid addresses. Released public capture tests in the original scratch verified the rewriter and rejection of a shifted native ordinal. The assembled deciding launcher has not run protected fitting or evaluation.

The exact source desk review at 459027bca found the held and normal-client records compatible with the predecessor CUDA writer and independent grader, conditional on custody and the deciding run. The final launcher cross-checks the original three immutable semantic keys and both blind-file digests against the captures before any fit. The Wick release binder e4d571cbfa008d9369de8812549ea6d38e7a8f1049b212db0c988b82aee989bc passed the strict input checker, complete synthetic metadata join and paired reducer. The two seed drafts and primary/repeat capture index remain distinct from a dispatch authorization.
