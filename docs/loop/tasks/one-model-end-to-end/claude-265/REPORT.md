# REPORT: one-model-end-to-end, claude-265 (trained and served without the icosian snap, I-3 misses fall to 32 and 14 of 310; partial, not yet the bar of 8)

Lab: claude. [ORDER](./ORDER.md). PR #3600, item #3407.
Sheet: [one-model-end-to-end-i3-unsnapped-2026-11-03](../../../../harness/prep/one-model-end-to-end-i3-unsnapped-2026-11-03.md).

**In one paragraph.** claude-263 and claude-264 found that the integer first token misses the float one on 100–137 of DEV's 310 prompts, against a bar of 8. The served bridge snapped both Hamilton gates and the recurrence transition to the 120 icosians, and the project's own R6 work (claude-180 to claude-182) had found the snap to be the main source of that kind of gap. This unit made the served bridge train and score on the unsnapped D11 contract: the gates pass through and the transition is unit-normalized, while the Q12 query, the Q16 residual, the i16 activations and the projection rounding stay on. It trained claude-262's 256-step served recipe at seeds 1 and 2 under that contract on one RTX 5090, scored each (seed 1 twice), exported raw and served on the laptop. First tokens now differ on 32 and 14 of 310, from claude-264's 100 and 131. The float scorer gave identical output twice, so these are real parity misses, not run-to-run noise. By the sheet's bar this is partial: the snap was the main part, and the rest is still above the bar.

## Tokens per second first

- **The integer model, laptop, `fbs-toks`, prefill included:** 759.0 tok/s (seed 1, 18,203 reply tokens in 24.0 s) and 767.7 tok/s (seed 2, 17,581 in 22.9 s).
- **Training, GPU, served:** 74 s wall per 256-step run, and the trainer's own clock read 30 s. That is 256 × 16 windows × 192 tokens in 30 s, about 26,000 context tokens/s (the clock prints whole seconds). claude-262's served runs, snapped, took 93–94 s wall and 18,282 / 18,243 context tokens/s.
- **The float scorer, GPU, serial decode** (the `decode` records `score_rows` prints): 74.0–75.2 tok/s over the three scores. claude-263 read 88–89.5 and claude-264 30.7–42.5, on other hosts. These time the instrument, not the served model.

## What did not work

1. **I-3 is not closed.** `serve-265.sh` printed:

   | Seed | First tokens differ (bar 8) | claude-264 | Whole replies differ | Order mismatches | Reading |
   |---:|---:|---:|---:|---:|---|
   | 1 | 32 / 310 | 100 | 287 | 0 | PARTIAL (≤ 50) |
   | 2 | 14 / 310 | 131 | 240 | 0 | PARTIAL (≤ 65) |

   Both seeds are above 8, so neither is a pass. The full-length served run, which the sheet opens only on a pass, did not start.
2. **Whole replies still differ on 287 and 240 of 310.** The first token matches more often, but later tokens still drift apart. Some float-versus-integer difference other than the snap compounds over a reply. This unit does not say which.
3. **The models are new, so the comparison is not paired.** Both seeds were trained under the changed contract. claude-264's 100 and 131 came from snap-trained models served snapped. So the fall from 100 / 131 to 32 / 14 mixes two things: the snap's removal from serving, and different trained weights. The smoke (item 3 under Measurements) shows the second can matter: snap-trained weights served raw miss on 307 and 298.
4. **A panic gap outside the served bridge.** Done-when 1 is met under the served bridge: its `Restore` guard (`hamilton_sheet.rs:774-794`) puts the residual flags, `BRIDGE_ACTIVATION_I16` and the whole `GATE_HOOK` back on `Ok`, `Err` and a panic, `bridge_d11` included, and the existing test pins it with `catch_unwind` (`:3300-3307`). Called on its own, `with_d11_training_snaps` restores `bridge_d11` on `Ok` and `Err` but not on a panic. `with_d11_training` at the base had the same gap, so it is not new.
5. **No guard in `fbs-export`.** The exporter still defaults to snapped and does not read how a run trained. `crates/uor-r4-integer/src/bin/` is outside the item's scope, so the I-3 serve step does the check instead (I-6 below). A user who exports one of these checkpoints without `--raw-gates --raw-transition` gets a snapped artifact the model was not trained for.
6. **The step-256 DEV lines differ from claude-262's**, which is descriptive only (the contract changed): seed 1 `[46, 362, 48, 459]` and seed 2 `[61, 335, 19, 515]` (top-1, nearest, openings, admitted), against `[53, 305, 9, 491]` and `[67, 357, 17, 490]`. Top-1 is lower at both seeds, and seed 2's nearest is lower; seed 1's nearest and both openings counts are higher. 256 steps at two seeds cannot say whether the contract costs quality.

## What changed

- **`hamilton_sheet.rs`** (`d2a85b9bb`, doc fix `d4bb64ce2` = `C`):
  - `SERVED_SNAPS = D11Snaps { gates: false, transition: false }`, the unsnapped contract.
  - `with_served_surrogate_training` calls `with_d11_training_snaps(seed, step, SERVED_SNAPS, f)` in place of `with_d11_training`.
  - `with_d11_mode` takes `SERVED_SNAPS` while the served bridge's `BRIDGE_ACTIVATION_I16` is set, and `D11Snaps::ALL` otherwise. So the served checkpoint's DEV panel (`evaluate_dev_panel_d11`) and loss (`panel_loss_d11`) follow the contract with no edit of their own.
  - The new test `served_bridge_takes_the_unsnapped_d11_contract` fails at named assertions with either line set aside (`:3331` "the served bridge", `:3336` "the D11 mode under the served bridge") and passes once restored.
- **`in_space_long.rs`:** one assertion of `the_training_forward_selects_and_restores_the_served_bridges`, the transition flag the served contract turns off.
- **Unchanged:** the D11 trainer, and the plain and Export forwards. `git diff --stat C HEAD -- crates/uor-r4-float-train Cargo.lock` is empty: every later commit of this branch is documents only, and the merge of main brings one crate file, `crates/uor-r4-cpu-train/src/train.rs` (#3603), which the float trainer does not build.

## Measurements

1. **I-0, on the pod at `C`:** the CUDA library suite gave 299 passed, 0 failed, 7 ignored. All four named tests passed: the new test, `the_training_forward_selects_and_restores_the_served_bridges`, `a_step_as_served_reaches_every_projection_after_a_requantized_activation` and `served_surrogate_gradients_are_exact_on_cuda`.
2. **The run:** `vast-58968294`, one RTX 5090, 13:45–14:31Z, about $0.43 of the sheet's $2.00. Binaries `train-in-space` `d0d69ea6…3d95` and `fbs-snapshot` `a83dab66…3b82`. Training 74 s per seed, scores 705 s, 647 s and 704 s, snapshots ok, `EXIT 0` with `DONE`.
3. **The laptop smoke (sheet step 0), before the pod:** claude-262's snap-trained snapshots, served raw, passed the header check and served, with 307 and 298 first-token misses (155 and 137 distinct first tokens, so not degenerate). The comparison script reproduced claude-263's 112 and 137 from its stored files.
4. **I-3** (above), with:
   - **I-4:** 310 samples in each of the three float files.
   - **I-5:** no `copy_key` in either header.
   - **I-6:** both headers are `uor-r4.lut-stack/7`, with `shape.raw_gates: true`, `shape.raw_transition: true` and no `transport_snap`.
   - **R2R:** seed 1's two float scores differ on 0 of 310 first tokens and 0 whole replies. The scorer gave identical output twice on one host, so the sheet's "not decidable" band does not apply. One pair of runs shows no spread; it does not bound one.

## The scripts

All three are in `~/uor-r4/local/claude/i3-265/`, named with their SHA-256 in the reviewed sheet:
- `job-265.sh` (`07f24d5f…0230`), the pod job: source pin, I-0, build, data pins, train, score, snapshot, seal;
- `watch-265.sh` (`294e493f…561a`), the lease and stall watcher;
- `serve-265.sh` (`205f0af4…1ef6`), the laptop export, header checks, serve and comparison.

The comparison at the heart of `serve-265.sh`, for each pair of sample files A and B (`differ KIND A B`):

```sh
case "${kind}" in
  first) sel='$a[0][.].reply[0] != $b[0][.].reply[0]' ;;
  whole) sel='$a[0][.].reply != $b[0][.].reply' ;;
  order) sel='$a[0][.].position != $b[0][.].position or $a[0][.].prompt != $b[0][.].prompt' ;;
esac
jq -n --slurpfile a "${a}" --slurpfile b "${b}" "[range(0; \$a[0]|length) | select(${sel})] | length"
```

## Evidence

HF store `results/claude/one-model-end-to-end-i3-unsnapped-2026-11-03/`, read back on 3 Nov:
- `claude-265.tar` SHA-256 `92203f2775993e1b31127ab3d4077453268c61adb62128161d7eba5b4a5ae4ac`. It is the pod's `/workspace/i3u` without the fetched inputs: the job log, every gate's output, the binaries' hashes, both seeds' run folders, the three scored folders and both snapshots. Its `SHA256SUMS.files` checks on the laptop.
- `MEMBERS.sha256` (`344e22b2…d5`) and `SHA256SUMS`; `shasum -a 256 -c` on the read-back copy gives OK.

The raw exports and the integer replies are not stored: `serve-265.sh` rebuilds them from the snapshots in about a minute.

## Checks run at the head

- **On the pod**, at `C`: I-0, as in Measurements.
- **On the laptop:**
  - the four named tests' host parts, and the crate's 214 host lib tests, at `C`;
  - the new test with each line of the fix set aside (fails at the named assertions), then restored (passes);
  - `git diff --stat C HEAD -- crates/uor-r4-float-train Cargo.lock` is empty (the merge of main adds only `crates/uor-r4-cpu-train/src/train.rs`);
  - `cargo test --locked -p uor-r4-harness the_real_sheets`;
  - `uor-harness loop report --check` and `uor-harness loop pr-check --local` on this REPORT.
- **Before every push**, the no-overlap check against #3583 and #3599.

## Skills used

None applied. The three scripts above are local and not committed as files.

## Still open

1. **The remaining parity sites: the next unit.** With the snap gone and the scorer giving identical output twice, the sites the float surrogate may still round differently from the integer session are:
   - the Hamming reads and the turn memory;
   - the writer's Q12 query;
   - the convolution;
   - the embedding;
   - the RMS norm and its shift, and the recurrence state and its decay;
   - `q16_residual`'s f32 edge on odd grid points with |x| in [128, 256).

   claude-263's still open 1 named `fbs-snapshot --dev` and `fbs-compare`, but they would mismeasure these models. `fbs-snapshot` builds its D11 readout through `with_d11_mode` outside the served bridge (`fbs-snapshot.rs:266`, `:585-592`; `fbs_export.rs:1108`, `:1136`), so for a raw-trained model that readout is snapped (`D11Snaps::ALL`). Comparing a raw export against it is the same mismatch that gave the smoke's 307 and 298. A readout on the served contract comes first. That is a change to an instrument, so it goes to Casey before it is built. The whole-reply count (287 and 240) says the gap compounds, so a value read per layer says more than a first-token count.
2. **A guard in `fbs-export`** that refuses a snapped export of a checkpoint trained unsnapped. It needs `crates/uor-r4-integer/src/bin/` in the item's scope.
3. **`with_d11_training_snaps` on a panic, outside the served bridge** (item 4): a drop guard like the served bridge's `Restore`. `with_d11_snaps` has none either (`flock_read_hamming.rs:1048` notes that a panic skips its restore).
4. **The full-length served run.** Its sheet (claude-263's part B) is FIT. It needs re-pinning to `C` and the raw export. Whether it waits for a pass at 8, or runs with I-3 reported beside it, is a decision for the next unit's own decision review.

## Plain summary

The model trained on the GPU and the integer copy served on the laptop now agree on the first word of a reply for 278 and 296 of 310 test prompts, up from 210 and 179. The change was to stop rounding the model's rotation gates to the 120 fixed icosian points, in training and in the served copy alike. Training took 74 s for the same 256 steps that took 93 s before, though on a different machine, so the change is not shown to be the cause. The agreement is still short of the bar of 302, and whole replies still drift apart, so other, smaller rounding differences remain. The next unit looks for them layer by layer.

References #3407.
