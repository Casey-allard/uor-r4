# REPORT: m1-address-as-read, kimi-42 (object-store oracle)

Lab: kimi. Order: [ORDER.md](ORDER.md). Prep: [`m1-address-as-read-oracle-2026-10-26.md`](../../../harness/prep/m1-address-as-read-oracle-2026-10-26.md). **Closes #2833 at the tested exact-read/oracle scope only.** The mechanism remains PARKED; no learned-memory, default-serving or M3 qualification claim is made.

## What did not work, first

- A provider host exited before SSH/bootstrap and produced no evidence; its cost was **$0.190**. The first v2 replacement then ran before a fresh FIT/D67 and before prep-sheet shape repair. That registration process fault was not repaired by relabelling the run. Final v3 was separately pre-registered, reviewed and dispatched at `590ace9ed`.
- Three model-control rows emitted invalid UTF-8 (`current-023`, `current-024`, `previous-017`). The first v2 attempt aborted; the final runner decodes lossily and marks `emission_utf8=false` instead of treating a model miss as an instrument abort. The true-address/store path is unaffected.
- The earlier v2 replacement cost **$0.606**, exceeding its $0.50 projection because of live pricing and instrument repairs. Final pre-registered v3 cost **$0.135** and met the projection. Total pod spend across all three is **$0.932**; earlier cost is not hidden inside the v3 figure.
- The initial pure layer allowed pooled contrasts, ambiguous sieve-address semantics and weak identity/resource claims. These were corrected before the final measurement. Both prep sheets initially failed `prep.sheet_shape`; `uor-harness check` now passes.
- No-memory generation recovered 0/80 exact values. This is not a success for the model writer and does not explain Stage 0's copy failure. Hard store emission is not learned address formation or a model-mediated read.

## What changed

- Added a versioned 80-row manifest and pure object-store composition using `AddressDirectory`: 40 current, 20 previous, 20 abstention (10 Absent + 10 Evicted), with deterministic sieve and present/value-different random bindings.
- Added five matched arms and exact true-address gates, per-family contrasts, formed-query versus selected-slot distinctions, ceiling/gain/no-gain classification and one composed gate requiring parity, leaks, identity, baseline identity, random control and hard gates.
- Added parity-only reuse of the Stage 0 GPU scorer, real candidate/fetch/byte/resource accounting, actual-emission token/context rates, clean-tree and runtime-identity checks, invalid-UTF8 marking, and a single-tar/checksum/readback evidence path.
- Updated the item's measurement records and removed the stale trunk dependency: this T16_fz oracle does not require the M2 trunk checkpoint. Item scope and mechanism status remain bounded.
- Size at the REPORT commit: 20 files, 11,381 additions and 21 deletions. Code and unit tests 4,627 lines (including 5 dependency/workspace metadata lines), documentation 227, generated manifest 6,527. No numeric estimate was set; most additions are the generated manifest, pure controls and GPU runner.

## Measurements

**Final pre-registered v3, one RTX 5090 (`vast-57846936`) at `590ace9ed9a745c1246b7f9e9797aaecf6cf58e6`:**

- **Parity:** 14,287/14,287 rows, status 0 and zero mismatches; prediction/top-8/admitted-count agree with the sealed receipt. Aggregate `455 / 1,484 / 5,009`, all position-strip deltas `0`, repeat records byte-identical (`2346306589…`).
- **True address:** **40/40 current, 20/20 previous, 20/20 typed abstention** (10 Absent + 10 Evicted), zero wrong substitutions. All carry the `ExactStore` source marker; abstentions have no content to whole-object verify. `wrong_substitutions: 0` is the true-arm hard gate only; sieve/random wrong values are expected controls. Evicted-address serialization is post-run hardening with a wrong-key RED→GREEN test; `verify_built_store` already checked the sealed v3 addresses.
- **Model baselines:** no-memory and no-address each recovered **0/80** exact values. Their generated token vectors and arm-excluded emission records were identical; complete serialized rows still differ in `arm` and `slots` by design. `slots=0` for no-memory is a logical arm label while the process-level store remains populated for other arms.
- **Sieve:** formed-query match **60/80** (current 20/40, previous 20/20, abstention 20/20); selected-address match **40/80**; exact value **20/80** (current 20/40, all other families 0). Previous current-value fallback is descriptive and excluded from sieve contrasts.
- **Contrasts:** true-vs-no-memory `b/c = 40/0` current, `20/0` previous, `20/0` abstention. True-vs-sieve `20/0` current and `20/0` abstention. Sieve-vs-no-memory current is also `20/0`: that is the deterministic baseline's gain, not a treatment gain. Every contrast is family-specific and `c = 0` throughout.
- **Controls:** random-address binding held on 80/80 rows with 0 expected-value hits. Panel/TRAIN/DEV leaks were empty. The composed gate is `ORACLE_INSTRUMENT_PASS`. Exact `candidates: 1` means one exact key was considered, not that content was fetched or admitted; absent rows still have zero backend fetches.
- **Identity:** display driver `595.80`, CUDA driver API `13020`, full Stage 0 runtime identity `f7aaf7687f9fab7add1835cc01870d4c6abb1f0a4984eb7e66c8c244abfaaefc`. Serialized store `8,100` bytes and 150 slots; resident heap is `UNVERIFIED`, not inferred. No store-reload claim is made from the byte count alone.
- **Rates:** store-emission rates are not model decode or an assisted-model speedup. True-store emission was **1.480M tokens/s**, sieve **0.590M**, no-memory model decode **952 tokens/s**, no-address **952 tokens/s**. The raw JSON fields are milli-tokens/s and were divided by 1,000 for these values.
- **Cost:** final create-to-down `439 s` at `$1.106/h` = **$0.135**; build 172 s, run 10 s, upload 8 s. Earlier failed host `$0.190` and unregistered replacement `$0.606` are separate. All values are rounded from the ledger: exact final `$0.134869`, failed `$0.190313`, replacement `$0.606463`, total `$0.931645` (displayed as `$0.932`). The final sidecar is uploaded and readback-verified.
- **Evidence:** HF `results/kimi/m1-address-oracle-2026-10-26/results-v3/address-oracle-result-v3.tar.gz`, SHA-256 `3682dd7a80a8d994955b5b64131fceecc29be7ecdf49034276f744807154bb55`.

## Checks run at the head

- `uor-harness check`: PASS after both prep-sheet shape repairs.
- Default object-store tests: **104 passed, 0 failed, 1 ignored**; with `stage0-lab`: **133 passed, 0 failed, 1 ignored** at corrected test head `4365d3dd7`.
- `NVCC=/Users/casey.allard/uor-r4/local/fake-nvcc/nvcc UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo check -p uor-r4-object-store --features cuda`: PASS.
- Production manifest SHA-256 `a4aa548f3e38478fdc8cd4ad077d9886cefa3aaa8e2712abe8f38abe0dba4103`, reproduced and `cmp`-verified on the pod.
- FIT at `94330a3c` (agent-40) and D67 PASS at the same configuration (agent-34): [#2833 comment 6422745537](https://github.com/UOR-Foundation/uor-r4/issues/2833#issuecomment-6422745537).
- D68 measurement/scope review (agent-41): [#2833 comment 6423077214](https://github.com/UOR-Foundation/uor-r4/issues/2833#issuecomment-6423077214). The exact-read/oracle done-when is met at bounded scope; do not promote the mechanism or infer learned capability.
- `uor-harness loop gates --full`: all 11 gates GREEN at `ac1e8ebf2`.
- `uor-harness loop report --check --order docs/loop/tasks/m1-address-as-read/kimi-42/ORDER.md`: PASS at `ac1e8ebf2`.
- `uor-harness loop pr-check --local`: PASS at `ac1e8ebf2`; only advisory scope and rule-source warnings.

## Skills used

- `uor-research-architect`: exact claim boundaries, causal controls and artifact identity.
- `superpowers:systematic-debugging`, `test-driven-development`: RED→GREEN protocol, candidate and emission fixes, and provider/dispatch fault classification.
- `scope-guard`, `solution-integrity`, `holding-the-goal`: kept the delivered object-store path and refused to call hard emission learned memory.

## Still open

- The next question is address formation on the 20 current rows where deterministic sieve fails while true address succeeds. This is a separate item with its own FIT if GPU work is needed.
- Resident heap bytes remain UNVERIFIED. Store-emission rates do not establish product-serving throughput or energy use.
- Hidden-fact panels, BPB, training, model-mediated reads and default-serving promotion are outside this close. M3 durable-memory qualification is not established.

## Plain summary

Supplying the true address reads every stored value exactly. The deterministic sieve forms the right query on only 60 of 80 rows and recovers only 20 values, while the model alone recovers none. That is a real oracle gain at the item's tested scope, not proof that the model has learned memory.
