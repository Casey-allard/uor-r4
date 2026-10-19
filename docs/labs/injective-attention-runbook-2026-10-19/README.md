# Injective-codebook learned-attention test: staged runbook for owner release — 2026-10-19

- **Lab:** Claude (self-directed; #2029 intent `6257406680`).
- **Sheet:** `docs/harness/prep/m1-attention-injective-init-DRAFT-2026-10-18.md`, **TEST FITNESS: FIT** (#2712). The header was corrected in this PR, and the sheet SHA is re-pinned to `e47ce5afb3966bda13cd04bd9b99849b401c028e078de958fa98b5e8b7ed4efb` in `attention_learning.rs`.
- **§Sequencing:** met by the K16 outcome call (#2029 `6257335075`, record `docs/labs/k16-outcome-call-2026-10-19/`). The call selected arm A with no K16 admission advantage, so the sheet's K8 / objective-A configuration stands.
- **Status: STAGED. Nothing has run.** Training is HELD. Running needs the explicit PM dispatch, then the owner's release per arm. Under D40, Casey types the phrase himself in a terminal with `UOR_LAB` unset; no lab types it.

## What the test is

It is one variable against the parent's latest `rows_v1` attempt (`docs/labs/attention-rows-q1-diagnosis-2026-10-18/`). The initial codebook changes from the seeded device default (16 distinct vectors, alias ceiling 1,986 DEV rows) to the **PrimeZeta V4096 B4 injective table**: 4,096 distinct vectors, 256 of 256 writer cells, alias ceiling 14,287.

There are two arms, run in parallel, one GPU each:
- **Treatment:** `Relation{8,1}` LearnV1.
- **Control:** the matched FrozenZeroV1, which locks the relation trits. Vector and combine updates stay active.

Both use seed 1, K = 8, objective A (`NearestAddressMarginV1`), `active_leaf_v1`, `interleaved_read_every8_v1`, `rows_v1` and the Dose 1 pins. The contrast is inside the sheet: Relation against FrozenZero on the same initialization.

## Pinned inputs (in `evidence/`, with `SHA256SUMS`)

| File | What | SHA-256 |
|---|---|---|
| `prime-zeta-v4096-b4-c192-k8.json` | `uor-r4.codebook-table/1`, mode `prime_zeta`, seed 1, spec V4096 / B4 / C192 / K8 | file `83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b` (`table_sha256` `e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804`, the registered digest; 4,096 distinct vectors) |
| `parent-rows-relation-seed1.json` | Parent `rows_v1` treatment config; equals the recorded raw config hash (rows-Q1 `evidence.json`) | `52b32dc8747148e0cd385d309dbfb1c3cc750569d11ff3932745313e134cc670` |
| `parent-rows-frozen-zero-seed1.json` | Parent `rows_v1` matched control config | `6a035717130a5e7106e1…` (see `SHA256SUMS`) |
| `injective-relation-seed1.json` | Parent treatment plus `codebook_initialization = pinned_table_v1` | see `SHA256SUMS` |
| `injective-frozen-zero-seed1.json` | Parent control plus the same pin | see `SHA256SUMS` |

The two injective configs differ from their parents **only** in `codebook_initialization`, which is checked by `jq` diff. They use the parent's canonical pod data paths (`/workspace/uor-r4/local/driver/data/...`). The table pin's path is `/workspace/uor-r4/local/driver/data/injective/prime-zeta-v4096-b4-c192-k8.json`.

The table was built on the host with `prime-zeta-codebook evidence/spec.json 1 OUT prime_zeta`, which is offline table construction, not model compute. It is deterministic, and its seed 1/2 bit-exactness was qualified in #2692. Step 2 below rebuilds it on the pod and must reproduce the same file SHA.

**Scopes**, as computed by `attention_learning::scope` from the table's file SHA prefix `83c1380a8f50`:
- treatment: `m1-attention-learning-injective-83c1380a8f50-relation-k8-seed1-batches0..16404`
- control: `m1-attention-learning-injective-83c1380a8f50-relation-frozen-zero-k8-seed1-batches0..16404`

## Runbook

**Lab side, after PM dispatch.** Use one Vast 2 × RTX 5090 pod via `uor-pod` at the merged head of this PR, and record the head SHA. `discrete-stack` is the `uor-r4-discrete` CLI binary.

1. **Build.**

   ```
   cargo build --release --locked -p uor-r4-discrete --features cuda
   ```

   Record `driver_source_sha256`. Every plan below binds it, so build nothing else in between.
2. **Table.**

   ```
   prime-zeta-codebook evidence/spec.json 1 /workspace/uor-r4/local/driver/data/injective/prime-zeta-v4096-b4-c192-k8.json prime_zeta
   ```

   Required output: `file_sha256 = 83c1380a…` and `table_sha256 = e9e03bcd…`. Any other value is an instrument blocker; stop.
3. **Configs.** Copy `evidence/injective-{relation,frozen-zero}-seed1.json`, then run `discrete-stack attention-build-identity CONFIG` on each. That validates the config and returns the identities the plan binds, including the scope above.
4. **Pre-flight battery** (sheet §3; GPU, zero updates):
   - **G-Init and its negative controls:**
     - an edited table file with stale hashes is refused on the host;
     - a re-hashed and re-pinned non-root word is refused with GPU status 6;
     - the loaded table equals the pin word for word, and routes, gates and epoch equal the unpinned initializer's.
   - **G-Occ:** 256 occupied cells at checkpoint 0.
   - **G-Cap:** run `coverage-census --token-cells` on the checkpoint-0 export from `attention-export-token-cells`. The gate is **net G-Cap > max(500, top1_control(cp0) + 24)**.
   - **`B3_init(P_succ)` from `attention-dev-b3`:** must be ≥ 500, or the run is not dispatched.

   Post the battery receipt on #2029. Any FAIL stops the runbook for diagnosis.
5. **Plans and review.**
   - Write one `uor-r4.attention-learning-owner-plan/2` per arm from the build identities. Follow the parent `rows_v1` plan templates in `codex-attention-dose2-rows-final-static-20261018/packet/admission/prepared-inputs/`.
   - Set `run_state_directory` to a fresh absolute path, separate from the release directory.
   - Set `transfer` to the sheet's fresh V2 source: tokens `0b7e754c…`, mask `a1cda529…`, seal `df5d074a…`.
   - Get one `APPROVE_ATTENTION_LEARNING_ADMISSION_DIFF` review per arm. Per the owner direction (#2029 `6255848646`), a fresh-context adversarial Claude reviewer subagent may serve. Name the seat and post its findings.
   - `discrete-stack attention-check-plan PLAN` must print `PLAN_METADATA_VALID` and `owner_release: STILL_REQUIRED`.

**Owner step (Casey, personally; about two minutes per arm).** In a terminal on the pod, with `UOR_LAB` unset, for each arm:

```
env -u UOR_LAB <ABS>/target/release/discrete-stack attention-owner-release <ABS>/injective-relation-seed1.plan.json <ABS>/release-relation-seed1
```

The tool prints the plan, then `Type exactly:` with the phrase. For the treatment arm it is:

```
I am Casey; release D4 for m1-attention-learning-injective-83c1380a8f50-relation-k8-seed1-batches0..16404 only through batch 16404 with the pinned DEV transfer and relation-off evaluations
```

Repeat with `injective-frozen-zero-seed1.plan.json` and `release-frozen-zero-seed1`. That phrase is:

```
I am Casey; release D4 for m1-attention-learning-injective-83c1380a8f50-relation-frozen-zero-k8-seed1-batches0..16404 only through batch 16404 with the pinned DEV transfer and relation-off evaluations
```

Each run writes `owner.json` and `admission.json` into its release directory. The release directory and the plan's `run_state_directory` must both not exist yet.

**Lab side, run and transfer:**

6. Run `discrete-stack attention CONFIG <release>/admission.json NEW_REPORT_DIR`, one arm per GPU (`uor-pod run … --gpu 0` and `--gpu 1`). Use `attention-resume` with the same arguments after any interruption. The Q1 gate at batch 4,096 and the DEV evaluations run inside the job. The sheet's §7.9 / Q1 stop rule halts a failing run at Q1. The phrase cannot narrow the scope; it always says "through batch 16404".
7. After the sealed paired DEV decision, run `discrete-stack attention-transfer PAIRED_DEV_DECISION NEW_REPORT_DIR`. This opens the fresh V2 slice once.
8. Post the receipts, archive to the HF store and iCloud, and delete the pod.

## Cost (sheet §Cost; assumptions)

About 1.31 s per batch. The full dose of 16,404 batches is about 6.0 h, roughly $11.4 at about $1.89/h for 2 × 5090. A Q1 stop costs about 1.5 h, roughly $2.8. The pre-flight battery adds under 30 min. The owner caps (≤ 4 pods, ≤ $8/h) and the 30 GiB disk floor are hard stops.

## Not covered by this runbook

- **Seed 2** is reproducibility only (sheet §D22), with its own later release.
- **The plan JSONs, reviews, release directories and run receipts** are produced in steps 3–7 on the pod. They are not committed here, because the plans bind the exact build's `driver_source_sha256`.
