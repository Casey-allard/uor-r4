# REPORT: m1-address-as-read, kimi-40 (Stage 0 copy and parity)

Lab: kimi. Order: [ORDER.md](ORDER.md). Decision: [DECISION.md](DECISION.md). Sheet: [`m1-address-as-read-stage0-2026-10-26.md`](../../../harness/prep/m1-address-as-read-stage0-2026-10-26.md).
References #2833; this unit does not close the item or claim an oracle gain.

## What did not work, first

- The old 54/80 oracle plan was unfalsifiable: panel-v2 contains answers in model-visible dialogue on 70/80 rows, its 19/40 baseline belongs to another model/panel, and no lawful BPB evaluator exists for this stream. The D68 review rejected that plan and the absolute bar was removed, not re-run.
- FIT initially returned NOT_FIT for an unspecified manifest, a host-reference parity path, missing identity/schema/cost details, and a stop path that lost aggregate deltas. The final sheet uses two GPU paths only, a sealed original receipt, a concrete 40-row manifest and aggregate-before-stop reporting. Fresh review then returned FIT.
- Three dispatch setup faults preceded scoring: nested store paths hid `SHA256SUMS`; an explicit-target build was needed for the known `cdylib` collision; and the staged config needed its nested input path normalized. No model result was read from those failed dispatches.
- **Copy gate failed:** exact-hint free-running recovered **0/40** exact values. Exact versus no-hint has `b = c = 0`; exact versus random-hint also has `b = c = 0`. Neither primary comparison meets `d > 24 && d^2 > 4(b+c)`. Panel-v3, BPB and training did not follow.

## What changed

- Added `address_stage0` manifest generation and a GPU-only runner behind `stage0-lab`: pinned artifact/config/receipt validation, corpus-leak scan, two D11 parity passes, sealed `final-dev.json` comparison, teacher-forced and free-running copy, identity and aggregate-preserving stop reports.
- Added the 40-row manifest (`ea360118…`), protocol and pure parity tests, and the root workspace exclude required by the nested D11 instrument.
- Recorded the scoped result in `docs/mechanisms/exact-memory.md`, the lab README and `docs/research/notebook/m3-durable-memory.md`.
- Size at the REPORT commit: 17 files, 8,214 additions and 8 deletions. Code and unit tests 2,750 lines, documentation 210, generated manifest 5,212, dependency/workspace metadata 42. No numeric estimate was set; most additions are the generated manifest and the GPU runner.

## Measurements

**Repaired GPU verification, one RTX 5090 (`vast-57818697`) at `11624e808cba656ba9b068f338f5ad71c5699f80`:**

- Parity reproduced: **14,287/14,287** rows, status 0, no mismatches, aggregate `455 / 1,484 / 5,009`, all position-strip deltas `0`. Both records are byte-identical (`2346306589f327a751d510b574147512178ef9334c7e45ff329648760fd38bec`). The new parity report names no ordinal check and explicitly records row-order join.
- Copy reproduced: exact/no-hint/random-hint **0/40 exact matches**, normalized emissions **12 / 0 / 9**, teacher-forced counts `3/15`, `4/17`, `5/18` out of 275, and `b = c = 0` on both primary comparisons. Planted-wrong held on 9/9 normalized rows; `COPY_GATE_FAIL` stands.
- Identity: **CUDA driver API 13.2** (`cuDriverGetVersion` raw `13020`), CUDA toolkit 12.8, original/staged/normalized config hashes `77d614… / 43a316… / e6f85c…`, executable `d43ff1f7082d585cb3a74018ce3550061fa885d51a8ce82e313265cf8752f46b`, cubin `17dd4184…`, PTX `e8ee1c12…`. The actual NVIDIA display-driver version is **UNVERIFIED**: it was not captured. The v2 receipt's `nvidia_driver_version` field is the CUDA driver API value, not the display-driver package; the source field is corrected to `cuda_driver_api_version` without altering the sealed receipt.
- Cost: ledger create-to-down uptime `511 s` at `$1.106/h` = **$0.157**. The receipt was pulled and its HF listing verified before release/down. The first down refused its local archive step; deletion used `--confirm-archived` only after the separate HF upload succeeded. No energy claim.
- Artifacts: `results/kimi/m1-address-stage0-2026-10-26/results-v2/stage0-result-v2.tar`, SHA-256 `cb72cdf4eef5316c56d3c732eb5bc523ff55918ddc4725252afe7dfc52108395`. The pre-repair tar below remains preserved unchanged.

**Pre-repair GPU receipt, one RTX 5090 (`vast-57806658`) at `97354145ecaf7ea70908ef25922ea3c57dec55fe`:**

- Parity: **14,287/14,287** rows have status 0 and agree with the sealed original GPU receipt on target position/token, prediction, top-8 and admitted count. The receipt has no ordinal field; row order was used only to join corresponding records. Repeat records are byte-identical (`7f373b3b…`). Aggregate top-1 `455`, top-8 `1,484`, admitted `5,009`; all position-strip deltas are `0`. `parity-report.json` PASS.
- Copy: exact free-running **0/40 exact matches**, no-hint **0/40**, random-hint **0/40**. Normalized emissions were **12 / 0 / 9** respectively; these are not exact matches. Both primary comparisons have `b = c = 0`, hence `COPY_GATE_FAIL`. Random-hint planted-wrong held on all 9 normalized rows.
- Teacher-forced (descriptive only): exact `3/275` token top-1 and `15/275` top-8; no-hint `4/275` and `17/275`; random-hint `5/275` and `18/275`. No row has all value tokens top-1 or top-8. These are not generation capability claims.
- Config provenance: retained original Stage T config `77d61451125b04827821216e221d548885af188bad6d9d301234d126fefa3b7d`; staged input config `43a316bc67bb45fe5b50bd2ceee57f6b919164c02fe45ee9e677f8819fea5d15`; actual normalized config `e6f85ca9ddaf2d1e471d6e15d385f4bce6411ef7514c412f32ac44776d838569`. A local `sed` rewrite produced the staged input's absolute paths; a second `sed` path correction on the pod produced the normalized file.
- Driver provenance: the measured run's identity receipt did not include the actual NVIDIA driver version, so it is **UNVERIFIED** for this run. The repaired runner now queries `cuDriverGetVersion`; no driver version is inferred for the old receipt.
- Cost: pod uptime `1,163 s` at `$0.563/h` = **$0.182** (not an inferred energy measurement). One pod, released and deleted after upload. Host preparation/review cost was lab time only.
- Artifacts: `results/kimi/m1-address-stage0-2026-10-26/results/stage0-result.tar`, local tar SHA-256 `2be4f5ef8880e6c7a53505bb6974ee06f063b49108f98b6bc35ce3f73e4c4a07`. Binary `6aa0709ddbf2b78275f95e9a8005328c37b4951fb69814098eb74abaffc7e530`; cubin `17dd4184d1e49dbb07b7f1d7bb236859c38b715edd968255dd41e61196a10465`; PTX `e8ee1c1292a46f9a5eb24779f163e771fdce94dc10644f9ce89050f5dec333be`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-object-store --no-fail-fast`: **83 passed, 0 failed, 1 ignored** after the repair.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test -p uor-r4-object-store --features stage0-lab --no-fail-fast`: **112 passed, 0 failed, 1 ignored** after the repair.
- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo check -p uor-r4-object-store` and the same check with `--features stage0-lab`: PASS after the repair.
- `NVCC=/Users/casey.allard/uor-r4/local/fake-nvcc/nvcc UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo check -p uor-r4-object-store --features cuda`: PASS.
- `uor-harness loop gates --full`: all 11 gates GREEN at `ec46ec2b6`.
- `uor-harness loop report --check --order docs/loop/tasks/m1-address-as-read/kimi-40/ORDER.md`: PASS at `ec46ec2b6`.
- `uor-harness loop pr-check --local`: PASS at `ec46ec2b6`; only advisory scope and rule-source warnings.
- D68 review: REJECT old plan; Stage 0 accepted as the required next measurement.
- FIT: FIT at `97354145e` and confirmed again for the repaired instrument (agent-32). D67: PASS at `97354145e`; delta PASS at `11624e808` after the staging and repeat-registration corrections (agent-34).

## Skills used

- `uor-research-architect`: whole-project mechanism/claim boundaries and instrument-first review.
- `superpowers:systematic-debugging`, `test-driven-development`: failing protocol tests before implementation and each dispatch fault classified before any result.
- `scope-guard`, `solution-integrity`, `holding-the-goal`: no larger panel or training after the copy gate failed.

## Still open

- This artifact/copy layout cannot support a hidden-fact oracle measurement. Reassess the model/read instrument before any v3; no current mechanism claim follows.
- The address consumer's 6 finite controls remain valid at store scope. Level 3 trunk behavior is not in the D11 and was not measured here.

## Plain summary

The integer model reproduces the original GPU result exactly on every development row. It still cannot copy a short supplied value exactly once in 40 tries. That means a bigger memory-oracle panel would not measure anything useful on this artifact, so I stopped instead of calling it a memory result.
