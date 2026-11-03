# Prep sheet: does training and serving on the unsnapped D11 contract close I-3? (one-model-end-to-end, claude-265, 3 Nov 2026)

TEST FITNESS: FIT - a fresh non-author adversarial review of this sheet (SHA-256 `1a278234…1bd2` at `6a3e98081`), the crates diff from `74e6cf369` to `C`, and the three scripts. No blocker.
- It confirmed the change: `BRIDGE_ACTIVATION_I16` is written only inside the served bridge and its guard's restore, so `with_d11_mode`'s dispatch cannot reach a non-served path, and it is restored on `Err` and on panic. The D11 trainer, plain and Export forwards are unchanged. Training and scoring both take the served forward.
- Its one MAJOR is fixed: the bands now have a stated order, so a seed at 8 or fewer misses with R2R above 8 reads Not decidable.
- Its two MINORs are fixed in `serve-265.sh`: a failed `fbs-toks` is refused per seed, and the integer file must hold 310 rows. The script's hash below is the fixed one, and the smoke reran with the same counts.
- Its note on the watcher's $15 stop is answered under "Cost".
- One existing gap is noted: `with_d11_training_snaps` restores `bridge_d11` on `Ok` and `Err`, not on a panic, as `with_d11_training` did before.

PRE-RUN: PASS - the same fresh D67 review, conditional on three items it could not read, all checked by the session at `C`:
- every flag of the train and score commands is parsed by `train-in-space.rs`, which is unchanged since claude-264's `5ce220ebc`, where the score command ran. `fbs-snapshot MODEL_DIR OUT_JSON` is its first usage line;
- all four named tests are in each file's one `mod tests`: `hamilton_sheet.rs:3317` and `:3459`, `in_space_long.rs:14587` and `:14694`;
- a default `hamming_copy_key` is left out of `config.json` (`skip_serializing_if = "is_one"`), so the job's `null` check holds.

The raw export's fidelity rests on claude-182, claude-259 and step 0's smoke. If it falls short, a miss reads as an instrument fault (§4.2).
- *Chain:* plan step 4, with I-3 as the door to the full-length served run.
- *Outcomes:* a pass opens that run; partial and not the part each name the next sites; not decidable fixes the scorer first.
- *Caps:* no pod was running at registration; one 5090, 2.5 h, at most $2.

## Question

claude-263 and claude-264 found that a model trained as served, exported and served in integers, gives a different first token from its float model on 100–137 of DEV's 310 prompts, against a bar of 8.

- **The served bridge snapped.** At main `74e6cf369`, `with_served_surrogate_training` calls `with_d11_training`, which runs `with_d11_mode`: both Hamilton gates take the icosian-hard arm and the recurrence transition is snapped to the 120 icosians. So these runs had to be exported snapped (an icosian `transport_snap` in the header).
- **R6's result is that the snap is the gap:**
  - claude-180: the gate snap carries most of the D11 forward's gap from float.
  - claude-181: with the same snaps, an exact reference and the integer session split on 33 of 2,048 positions at snap near-ties; unsnapped, on none.
  - claude-182: the unsnapped contract closes the export gap, and PLAN.md records R6 done on it.
  - claude-259: trained with only the export's table rounding and exported raw, its first tokens differ from float on 0–1 of 310.

This unit's commit `C` makes the served bridge train and evaluate on the unsnapped contract: the gates pass through (`HamiltonActive`), and the transition is unit-normalized, not snapped. The Q12 query bridge, the Q16 residual, the i16 activations and the projection rounding stay on. It then trains claude-262's recipe at seeds 1 and 2, exports raw, and serves on the laptop.

The question: **at both seeds, does the integer session give the float model's first token on all but at most 8 of 310 DEV prompts?**

This is a parity reading on two 256-step models. It does not establish language capability, a milestone bar or a training law.

## Base artifact

- **The training commit `C`** is `d4bb64ce26cba75491b1aaef3cf51f7ff42f5be0` on PR #3600's branch `claude/i3-unsnapped`: main `74e6cf369` plus `hamilton_sheet.rs` (the fix and its host test) and one assertion in `in_space_long.rs`.
  - `SERVED_SNAPS = D11Snaps { gates: false, transition: false }`.
  - `with_served_surrogate_training` now ends in `with_d11_training_snaps(seed, step, SERVED_SNAPS, f)`.
  - `with_d11_mode` takes `SERVED_SNAPS` while `BRIDGE_ACTIVATION_I16` is set (the served bridge's own flag), else `D11Snaps::ALL`. So the served checkpoint's DEV panel and loss (`evaluate_dev_panel_d11`, `spiralcore_fbs_sheet::panel_loss_d11`) follow the contract without an edit.
  - Every later commit on the PR touches documents only; the REPORT shows `git diff --stat C H -- crates Cargo.lock` empty at the PR head `H`.
- **No checkpoint input.** Both models are trained in this unit.
- **The serving tree** is this worktree at `C`. Its `crates/uor-r4-integer`, `crates/uor-r4-tokenizer` and `Cargo.lock` equal main `74e6cf369`'s (`serve-265.sh` refuses otherwise). `fbs-export` and `fbs-toks` are built with `--features serving-wasm`.

## Data split

DEV, TRAIN and the tokenizer by claude-261's pins, as in claude-262's and claude-263's sheets:

| File | SHA-256 |
|---|---|
| `tokenizer.json` | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| `mw-balp/train/tokens.u16` | `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` |
| `mw-balp/train/response_mask.u8` | `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` |
| `ft/dev/tokens.u16` | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| `ft/dev/response_mask.u8` | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |

They are staged from the HF store at `w2-inputs/2026-10-25/driver-data`. A pin mismatch stops the job. No panel is read; DEV is diagnostic.

## Steps

**0. Laptop, done before the pod (at `C` unless named):**
- The full `uor-r4-float-train` host suite passes, 214 of 214 lib tests (at `d2a85b9bb`, whose crates differ from `C` in two comments only).
- **Negative gates** for `hamilton_sheet::tests::served_bridge_takes_the_unsnapped_d11_contract`, each a one-line set-aside with the Edit tool, then the test rerun:
  - **A:** the bridge's last line back to `with_d11_training(seed, step, f)`. It fails at `hamilton_sheet.rs:3331`, "the served bridge": left `(Some(HamiltonIcosianHard), false, true, true)`, right `(Some(HamiltonActive), false, false, true)`.
  - **B:** `with_d11_mode`'s served branch back to `D11Snaps::ALL`. It fails at `:3336`, "the D11 mode under the served bridge", with the same left and right.
  - Restored, the test passes and the tree is clean at `C`.
- A stub-nvcc `cargo check --features cuda --all-targets` of the crate passes.
- **The serve smoke.** `serve-265.sh` ran on claude-262's existing served snapshots, which were trained with snaps.
  - The raw export ran, and I-6 read `raw_gates: true`, `raw_transition: true` and no `transport_snap`, under schema `uor-r4.lut-stack/7`.
  - `fbs-toks` exited 0 and served about 780 tok/s.
  - First tokens differed on 307 and 298 of 310. That is expected: those models were trained on snaps, so serving them unsnapped is a different model. Their raw replies are not degenerate: 155 and 137 distinct first tokens.
- **The comparison's instrument check.** On claude-263's own files, the script's `differ first` gives 112 and 137, claude-263's counts.

**1. The pod.** Bring up one RTX 5090 (step "Cost"). Push `job-265.sh` with `uor-pod ssh POD 'cat > /workspace/job.sh' < job-265.sh`, and check its SHA-256 on the pod. Run `uor-pod run POD --lab claude --session claude-265 --gpu 0 -- bash /workspace/job.sh C`, and arm `watch-265.sh` on it. The job does steps 2–5. Every exit seals `SHA256SUMS.files`, then `DONE` (exit 0 only), then `EXIT`.

**2. Pin, I-0 and build.**
- The job refuses any SHA but `C`, and checks the tree is clean at it.
- `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes, with these four tests named:
  - `hamilton_sheet::tests::served_bridge_takes_the_unsnapped_d11_contract`;
  - `in_space_long::tests::the_training_forward_selects_and_restores_the_served_bridges`;
  - `in_space_long::tests::a_step_as_served_reaches_every_projection_after_a_requantized_activation`, the served/plain gradient-norm ratio check, now unsnapped;
  - `hamilton_sheet::tests::served_surrogate_gradients_are_exact_on_cuda`.
- Build `train-in-space` and `fbs-snapshot` with `--features cuda`, and record both SHA-256.

**3. Data.** Fetch the five files and check every pin.

**4. Train, in turn.** Seed 1, then seed 2, with claude-262's step R recipe unchanged:

```text
train-in-space --data-root DATA --out ROOT/served-sN --objective cell --hamming-read 4 \
  --retain-rates 1 --decay-mult 1 --batch 16 --steps 256 --eval-every 64 \
  --lr 0.001 --lr-end 0 --turn-memory 8 --turn-memory-reply 1 \
  --turn-memory-dropout 500 --train-as-served 1 --seed N
```

A non-zero exit, a missing `models/step-000256`, or a config that is not `train_as_served: true` with rates retained and no copy key stops the job.

**5. Score and snapshot, in turn.** Score with claude-263's command (`--score-rows CK --samples 1536 --reply-tokens 64 --retain-rates 1 --seed N`):
- seed 1 into `served-s1-scored`;
- seed 2 into `served-s2-scored`;
- seed 1 again into `served-s1-rescored`, on the same host, for the run-to-run reading.

The scorer reads `train_as_served` from the run's `config.json`, so it scores through the served bridge, now unsnapped. Then `fbs-snapshot CK ROOT/served-sN-snapshot.json` for both seeds. Each `samples-memory.json` must hold 310 samples (I-4).

**6. Seal.**
- Pull with `uor-pod ssh POD 'tar -C /workspace --exclude i3u/in -cf - i3u' > claude-265.tar`, unpack, and check `SHA256SUMS.files`.
- Put the tar with a `SHA256SUMS` at `results/claude/one-model-end-to-end-i3-unsnapped-2026-11-03/`, and read it back.
- Then release the lease and take the pod down.

**7. Laptop serve and compare.** Run `zsh serve-265.sh ROOT TREE OUT`, where `ROOT` is the unpacked `i3u/out`. For each seed it:
- exports with `fbs-export --raw-gates --raw-transition --run-config ROOT/served-sN/config.json`;
- runs I-6 and I-5;
- serves DEV's 310 prompts with `fbs-toks --replies --reply-tokens 64`, timed;
- compares with seed N's first float score.

It then compares seed 1's two float scores. The script's output goes into the REPORT.

## Readings

For each seed, in order:
- laptop integer tokens/s, and each training run's context tokens/s and wall time (the REPORT leads with these);
- order and prompt mismatches;
- prompts whose first reply token differs between float and integer;
- prompts whose whole reply differs.

Beside them:
- **R2R:** seed 1's two float scores, first tokens and whole replies.
- **Descriptive only:** each run's step-256 DEV line (top-1, nearest hits, openings, admitted). claude-262's served runs read `[53, 305, 9, 491]` and `[67, 357, 17, 490]`. The training contract changed, so this is no comparison of quality.

## Bar

The readings are taken in this order, and the first that applies is the unit's reading: **Not read**, then **Not decidable**, then **Pass**, then **Partial**, then **Not the part**. So a seed at 8 or fewer misses with R2R above 8 reads Not decidable, not Pass.

- **Pass, at both seeds:**
  - order and prompt mismatches 0;
  - first-token misses at most 8 of 310 (`310 · 29 / 1000`, claude-261's bar);
  - I-5 and I-6 hold, and `fbs-toks` exits 0.

  I-3 is then closed on the unsnapped contract. The full-length served run (claude-263's part B sheet) runs next, re-pinned to `C` and to the raw export.
- **Partial:** not a pass, but misses at most 50 at seed 1 and 65 at seed 2, half of claude-264's 100 and 131. The snap was the main part. The next unit reads the remaining sites (the Hamming reads, the turn memory, the writer's Q12 query, the convolution, the embedding) with the R2R count in hand.
- **Not the part:** misses above half of claude-264's at either seed. The snap was not the main gap. The next unit reads those sites first.
- **Not decidable at this bar:** R2R's first-token count is above 8. The float model then differs from itself by more than the bar allows, so a miss count against one score cannot tell parity from the float's own spread, whether it is above or below 8. The REPORT gives every count anyway. The next unit makes the scorer deterministic, or registers the bar against R2R, before the full-length run.
- **Not read:** any order or prompt mismatch, a sample count other than 310 in a float or an integer file, an I-6 refusal, a failed `fbs-toks`, or a failed train, score or snapshot.

A miss is classed instrument (§4.2), as in claude-263 and claude-264. The two models here are new, trained under the changed contract, so claude-264's counts are a reference, not a paired control.

## Instrument checks

- **I-0** (pod): above.
- **I-4:** 310 samples in each of the three float files.
- **I-5:** no `copy_key` in either header.
- **I-6:** each header's shape has `raw_gates: true` and `raw_transition: true`, with no `transport_snap`. The schema is no guide: a turn-memory stack is `uor-r4.lut-stack/7` (`STACK_FBS_MEMORY_SCHEMA`) raw or snapped, and claude-263's snapped exports were `/7` with an icosian `transport_snap`. The rule that `rec_gates` of `geometric-retention` needs an icosian `transport_snap` applies only to a snapped transition (`stack/format.rs`).
- **The comparison:** step 0's reproduction of claude-263's 112 and 137.
- **The scripts:**

  | Script | SHA-256 |
  |---|---|
  | `job-265.sh` | `07f24d5f720d06634bc92df913b851b57658959265b47f677247823295900230` |
  | `watch-265.sh` | `294e493fa3bf1458a98e11549deb7ce74db4f197aceaf3434a793db0c804561a` |
  | `serve-265.sh` | `205f0af47459c627d2766f3e892e97d3f423a499ef4c203685e88b0aa4081ef6` |

  All three are in `~/uor-r4/local/claude/i3-265/`. The REPORT quotes them.

## Minimum test power

There are 310 paired prompts per seed. The pass bar (≤ 8) and claude-264's counts (100 and 131) are more than an order of magnitude apart, and the partial band's edge (50 and 65) sits halfway. Two seeds show whether the reading holds across initializations. They do not bound a rate.

## Cost and stop rule

- **Estimate.** In claude-262, a served 256-step run took 93–94 s of wall time, and the CUDA suite and build took about 1 minute on a bootstrapped pod. Scoring took 532–568 s a seed in claude-263 and 1,372–1,560 s in claude-264. So three scores take at most about 80 minutes, and the job about 1 h 45 min with bootstrap.
- **Lease.** 2.5 h on one RTX 5090 at under $0.80/h: at most $2.00. At registration no pod was running in any lab, and the cheapest verified 5090 offer was $0.563/h. This is inside the 4-pod and $8/h caps.

  ```text
  scripts/pod/uor-pod up --lab claude --session claude-265 --count 1 --gpu 5090 --hours 2.5 \
    --purpose "claude-265 I-3 on the unsnapped contract: train 2 seeds served, score, snapshot" \
    --card https://github.com/UOR-Foundation/uor-r4/pull/3600
  ```

- **The watcher.** `watch-265.sh` checks every minute. It renews the lease every 25 minutes, and stops on a lost pod, a 90-minute stall, $15 spent or 4.5 h. Its $15 stop is a backstop for the whole project, not this sheet's ceiling: the session enforces the $2.00 and 2.5 h below, and the watcher only wakes it.
- **Stop rule.**
  - A build, I-0, training, scoring or snapshot failure stops that step, and is classed instrument against mechanism before any retry.
  - A resource fault is rerun once, on a new pod.
  - A healthy run that outlasts the lease is extended, never cancelled.
  - The pod goes down only after step 6's read-back matches.
  - Work past one pod, 2.5 GPU-hours or $2.00 is a new ORDER.

## Run result

Recorded after the run, 3 Nov 2026. The sections above are unchanged from the FIT text.

- **The pod:** `vast-58968294`, one RTX 5090, 13:45–14:31Z, about $0.43. The job ran at `C` and exited 0 with `DONE`.
- **I-0:** the CUDA library suite gave 299 passed, 0 failed, 7 ignored. All four named tests passed.
- **Training:** 74 s wall per seed, and the trainer's own clock read 30 s for 256 steps. Step-256 DEV lines `[top-1, nearest, openings, admitted]`: seed 1 `[46, 362, 48, 459]`, seed 2 `[61, 335, 19, 515]`.
- **Scoring:** 705 s, 647 s and 704 s (seed 1 rescored). The float scorer decoded at 74.0–75.2 tok/s.
- **The seal:** `claude-265.tar` SHA-256 `92203f2775993e1b31127ab3d4077453268c61adb62128161d7eba5b4a5ae4ac`, stored at `results/claude/one-model-end-to-end-i3-unsnapped-2026-11-03/` and read back OK. The pod is down.
- **The serve (`serve-265.sh`, laptop):** from this worktree at `99d2d4bd3`, not `C` as "Base artifact" says. The commits between are documents only, and the script confirmed the integer and tokenizer crates and `Cargo.lock` equal `74e6cf369`'s.

  | Seed | I-6 header | I-5 | Laptop tok/s | Order mismatches | First tokens differ (bar 8) | Whole replies differ |
  |---:|---|---|---:|---:|---:|---:|
  | 1 | `/7`, raw gates, raw transition, no `transport_snap` | `[]` | 759.0 | 0 | 32 / 310 | 287 |
  | 2 | the same | `[]` | 767.7 | 0 | 14 / 310 | 240 |

- **R2R:** seed 1's two float scores differ on 0 of 310 first tokens and 0 whole replies.
- **The reading, by the bar's order:** read, and decidable (R2R 0). Not a pass at either seed. **Partial**: 32 is at most 50, and 14 is at most 65. The snap was the main part of the gap. The next unit reads the remaining sites.
