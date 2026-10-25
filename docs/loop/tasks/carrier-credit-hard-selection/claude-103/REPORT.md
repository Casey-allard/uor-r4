# REPORT: carrier-credit-hard-selection, claude-103

Lab: claude. Order: [ORDER.md](ORDER.md). Decision: [DECISION.md](DECISION.md).
References #2819.

## What did not work, first

- **This PR records no verdict.** It delivers the code and the design: the CUDA selector, the three schedules, the fresh slice, the runner, the grader, the sheet, the release and the plan. The eight registered arms are still running on GPUs; the controls finish about 18:30Z. A docs-only follow-up unit grades them and records the result. The item stays open, so this PR says References, not Closes.
- **The cost ran far past the sheet's estimate.** The sheet estimated about $6; the run will cost about $30–40.
  - Each arm's per-batch GPU time grew as it trained. ON went from about 300 batches/min in the probe to a full-dose average of 62–70 (66 and 75 targets/s in the records), and OFF from about 490 to 112–116 (119 and 124 targets/s). GPU memory stayed flat and the GPUs ran at 100%, so it is compute growth. OFF slowed too, so it is not the selector.
  - The A8 dose ran at a constant 0.12 s per batch from a pinned codebook. This sheet starts from the seeded-device codebook.
  - The 164-batch probe saw only the start, so it under-projected. The run went ahead under AGENTS.md's rule on self-set budgets, with the arms spread across three pods (owner log #2819 `6396426933`).
- **Run 1 of phase 2 stopped itself.** Four arms at once on one GPU projected 203 minutes per arm, past the 90-minute stop. Several CUDA contexts time-slicing one GPU ran slower in total than a single context.
- **The first launch on two new pods did nothing.** `uor-pod run` joins its arguments with spaces and so dropped the quoting of `bash -c '…'`, which meant phase 1 never ran. And a cache-hit bootstrap leaves no source tree. Both were repaired: a shallow fetch of `4aac77aa6`, one chain script per GPU, and new output tags. The faulted folders are kept.
- **A stray instance.** Two concurrent `uor-pod up` calls each landed on a 2-GPU offer. One failed to create and still billed disk; it held no data and was deleted.
- **This unit locked the discrete crate for others.** claude-104 put `crates/uor-r4-discrete/**` in the item's scope. Because `loop claim` reads an open order's item scope, every discrete item was unclaimable by any lab for about six hours; nine were ready, five of them on the M1 path.
  - claude-105 (#3222) removed the glob.
  - claude-106 (#3224) restored it briefly for this PR's review fixes.
  - This PR lands now so the lock ends.
- **The exact-head review found one Required defect.** `CarrierRelease::validate_for` counted and de-duplicated a release's arms without checking that each was registered. It is fixed, with a negative test. The live release lists exactly the eight registered arms, so no run was affected.
- **The "harness rules and tests" check stays red** on this PR. It is not one of `main`'s five required checks. Its only two errors are two antigravity sheets on `main` that fail `prep.sheet_shape`: `r-secant-surrogate-descent` (no Stop rule block) and `read-binding-on-lawful-read` (five blocks missing). Both were noted on #2028 (`6396324125`, `6399607831`), and antigravity has claimed the r-secant item again (#3225). This PR does not edit another lab's sheet. Its own sheet was fixed to pass the rule.
- **Not established here:**
  - D11 integer export: no milestone claim rests on this run.
  - Reopen condition 2 of carrier credit, and the "all 4,096 choices with physical copy aliases" part of condition 4.

## What changed

**Code** (`crates/uor-r4-discrete`):
- `discrete.cu`: the `discrete_carrier_select` kernel, integer only and one block. ON's credit is the token-plus-address hinge (unweighted, `out[0]+out[1]`), given to the target and, when top-1 is wrong, to the strongest rival. The argmax ties to the lowest id, with a lowest-target fallback. The controls draw by SplitMix64.
- `device.rs`: `Device::carrier_select` and `carrier_select_rows`. They share the evaluate launch, and the host reads only the 2-word result and the 1-word status.
- `carrier.rs`: the modes, the schedule key, SplitMix64, and a source-scan gate on the downloads.
- `training.rs`:
  - the schedules `carrier_credit_v1`, `carrier_uniform_target_v1` and `carrier_uniform_row_v1`, with reads placed as in `interleaved_read_every8_v1`;
  - `StepReceipt.carrier`;
  - a default-bytes gate, with literals computed on `origin/main` `14fc2cbe4`;
  - `issue_carrier_arm_admission`;
  - `evaluate_transfer` now accepts a `Fresh` split.
- `carrier_fresh.rs` and the `carrier-fresh-slice` bin: the sealed fresh slice.
- `carrier_arms.rs` and the `carrier-arms` bin: `plan`, `run` and `grade`; the D67 release record; the W2-rule count tables over visited TRAIN only; the exact integer two-sigma grader with its seed-noise floor; and the selection audit.
- `tests/carrier_select_controls.rs`: 10 CUDA controls.
- The driver-source pin moves to `cb686691…`. `carrier.rs` is a driver source.

**Docs:**
- the D68 card (approved in round 4);
- the prep sheet (FIT in round 2; its TEST FITNESS and heading sections were added after the release pinned it, and change nothing in the test);
- in `docs/labs/carrier-credit-hard-selection/`: the base config, the fresh-slice manifest, the plan, the release and the README.

## Measurements

These are instrument and throughput measurements only. The scored results are graded in the follow-up unit.

- **Phase 1, on each of three RTX 5090 pods:**
  - the 9 selector controls (run before c10 was added) and the CUDA lib suite (322 tests) pass;
  - the plan's sheet digest is `b2561a85…`, matching the reviewed bytes;
  - the release file is `695f4205…`. Each arm record's `release_sha256` is `3296f7a1…`, the sha256 of the release's compact JSON, which is what `run_arm` hashes.
- **Solo probe throughput, batches 0–164:** ON 318–319, CONTROL-U 326, CONTROL-F 326–327, OFF 521–532 targets/s. OFF matches A8's measured 533, and the selector costs about 39%.
- **The fresh slice:**
  - documents after TRAIN position 3,251,164;
  - 68,773 candidates scanned, 3,237 exact repeats dropped and 65,536 kept;
  - 211,846 tokens;
  - tokens `4d9ead83…`, mask `557131bc…`, sealed in the HF store before any arm ran.
- **Outputs:** `results/claude/carrier-credit-2026-10-25/` in the HF store. `podA/` (the ON arms and run 1) is verified by a full checksum round trip. `podB/` and `podC/` follow when those pods finish.

## Checks run at the head

- The discrete host suite: 362 tests pass, 0 fail (316 lib tests, plus the binaries and integration tests).
- The stub-nvcc `--features cuda` check: clean. `ratchet d23` and `d25`: at baseline. `loop pr-check --local`: PASS. Its banned-presence rule first caught `rows[row as usize].0` in c7, which the D32 scan reads as a float literal; c7 now destructures the row.
- **Negative gates shown failing:**
  - the source-scan gate, with an injected row download;
  - the bytes gate, with an injected rename;
  - the release gate, with the `registered` check set aside (fails at `carrier_arms.rs:735`);
  - the grade's vacuous-label pin, with the vacuous branch of `credit_label` set aside (fails at `carrier_arms.rs:679`).
- **The exact-head code review, in four slices:**
  - slices 1, 2 and 3 APPROVE;
  - slice 4 returned CHANGES (the Required fix above);
  - round 2, a fresh review of the fix delta, APPROVE. Its advisories are listed under Still open, except the vacuous-label pin, which is taken.
- **A fresh claims check of this REPORT and the lab README:** APPROVE. It flagged one causal overclaim in the README (the codebook as the cause of the slowdown), now worded as untested. The pod A figures were then checked against the archived records and logs: the probe rates, run 1's 86 targets/s and 203 minutes, and 9 controls and 322 lib tests on each of the three pods.

## Skills used

- holding-the-goal: the archived sheet could not be replayed lawfully, so the unit re-registered its design on the live learner rather than closing the item.

## Still open

- **The results unit** grades the eight arms, records the notebook row, updates the mechanism page's tried table and the lab README, and closes #2819 if the done-when is met.
- **Review advisories not taken here:**
  - Gate `Fresh` behind the carrier release, instead of accepting it in every caller of `evaluate_transfer`.
  - Pin 65,536 and the slice hashes as constants.
  - Tie each audit to its receipts' lineage.
  - Use `ROW_WORDS` in place of the kernel's literal `14u`.
  - Note that the credit is the unweighted `out[0]+out[1]`.
  - Sync the fresh slice's `tokens.uort` and `mask.u8` as the receipts are (the manifest hashes catch a short file at load).
  - Make the source-scan gate find a function's end by brace depth, not by the first closing brace at its indent.
  - On a GPU, show that c10 fails with the kernel's within-chunk comparison flipped to `>=`.
- **A candidate harness change, for its owner:** `loop claim` should not hold a lock on code paths an open order has finished with.

## Plain summary

I built a way for the learner to choose which word's vector to adjust next, from the errors it has just made, plus two blind comparison rules and the old fixed order. I set all four running at two seeds on rented GPUs. The code is done and reviewed, and it merges now. The scores come in a separate report once the last runs finish this evening.
