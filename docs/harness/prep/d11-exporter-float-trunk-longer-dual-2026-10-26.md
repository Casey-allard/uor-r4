# Pre-registration: longer dual-forward training (`d11-exporter-float-trunk`, kimi-46), 2026-10-26

- **Item:** #2820; ORDER `docs/loop/tasks/d11-exporter-float-trunk/kimi-46/ORDER.md`.
- **Prior decision:** claude-136's bound proves exporter fidelity cannot flip any of the six existing verdicts. The remaining pivot is one decisive longer run, not more residual inspection. claude-131's DEV losses were still falling at step 1,024.
- **Intervention:** **2,048 steps per seed instead of 1,024**, with the same per-step learning rates (`0.002` base, `20` decay multiplier), weight decay `0.1`, clipping, corpus, dual 1:1 loss, bridges, exporter and GPU evaluator. Evaluation points are exactly `0, 256, 512, 1,024, 2,048`. The first 1,024 batches are claude-131's deterministic document-order batches; the next 1,024 are new windows from the same corpus, so this is a dose-plus-coverage intervention, not a pure optimizer-step isolation.
- **D68 decision:** `agent-53` PASS at `e77a1f91f0f198de9aee58b7bc20d65a6c12b298`, owner log #2820 comment `6428815813`; full card `docs/loop/tasks/d11-exporter-float-trunk/kimi-46/DECISION.md`. It approves dose-plus-coverage, the exact D counterfactual and the registered floors, with cancellation/marginal readings required.
TEST FITNESS: FIT — agent-50 at `5345b554038954a3d46fdedcf3c30daeeb9d429f`. No compute before D67.

## 1. Question

Does the 2,048-step `Fbs_1Atom_Active` dual-forward recipe bring the exported artifact inside Stage 4b's export-gap band on both seeds, without sacrificing claude-129's artifact top-1 floors? This tests the configuration only; it does not show that dual training beats a 2,048-step plain arm, closes M1, or supplies sealed-fresh acceptance. #2820 may close only at its DEV export-gap contract.

## 2. Base artifact

- **F:** this run's plain-float model, trained from step 0 with `TrainForward::Dual` for 2,048 steps on seeds 1 and 2, scored on 14,287 `ft/dev` rows.
- **A:** each saved model's UORLUT01 artifact, scored through `fbs-export`, `d11-fbs-eval run --verify` and the existing artifact-gap scorer.
- **D:** each saved model's D11 forward, with D predictions and `loss_d11` captured. D is the perfect-exporter counterfactual: D passes only if `|net(F,D)| <= F_I(F,D)` and `|L_D-L_F| <= Δ_noise,L`.
- Each seed's model, snapshot, F/D/A predictions, artifact, evaluator output and gap/residual receipts are sealed with source, binary, kernel, geometry, data and sheet identities.

## 3. Bar and outcomes

`b` is F-hit/A-miss and `c` the reverse on 14,287 rows; `F_I = max(24, ceil(2*sqrt(b+c)))`. `Δ_noise,L = 0.045679637781173676`, read from claude-126's `d11-gap.json` (SHA-256 `88b6a5f988fe144671365a258a100e5cf3935f98e917040b38f90df2637a1e8a`). Apply this precedence per seed; never average mixed seeds.

1. **Instrument stop** if any instrument, batch/prefix, save/load, verification or finiteness check fails. Repair it; no model result.
2. **Unflagged PASS** if both A seeds pass top-1 and loss, all instruments are green, and artifact top-1 is at least 1,907 (seed 1) and 2,294 (seed 2). Measure product decode/context tokens/s, zero-multiply audits and integer FBS serving tests, then close #2820 at DEV scope only.
3. **Flagged PASS** if both A seeds pass but either artifact is below its floor. D68 review; do not close #2820 automatically.
4. **Cancellation PASS** if both A seeds pass but D fails on either seed. The registered A result stands, but report that A and D diverge; do not claim D11 fidelity or exporter clearance. This goes to D68 before any #2820 close and is not an unflagged PASS.
5. **Exporter unresolved** if at least one A seed fails and every failing A seed has D passing top-1 and loss, including mixed pass/unresolved. Do not call FAIL; open a separately FIT'd localisation/repair unit with its own cost.
6. **FAIL** if any seed has both A and D failing, including mixed pass/clean-fail. Close only this 2,048-step dual configuration and record a lab stop decision; the mechanism stays open and #2820 is not closed. Reopen only with new evidence that the D11 forward can meet its own band; otherwise #2820 remains parked for this recipe family. A D failure whose excess over its paired F_I exceeds the model's own `|net(D,A)|` is clean; within that envelope it is marginal: still a categorical FAIL, with separately FIT'd localisation as a reading, not a reclassification, and never attributed to the D11 forward alone.

At claude-131's discordance, a pass requires roughly 79% (seed 1) and 70% (seed 2) reductions in absolute net gap. A FAIL therefore rejects this configuration, not every longer-training benefit. Report plain/export-mode top-1 and admitted rates, trajectories, `net(D,A)`, full-precision two-seed loss band against `0.0230`, plain-float top-1 spread against `655`, artifact top-1 against claude-131's `3,492 / 2,897` as a regression reading, and paired b/c/F_I with a 95% paired interval as a reading. The paired F_I reading is operative; the two-seed spread is reported beside it and never used to re-score old receipts.

## 4. Required instrument checks

1. **I-Bridges:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes, including dual gradient sum, tangent bridge, Q12 clamp and all prior D11 tests.
2. **I-Probe:** `--mode d11-train-probe --train-forward dual --steps 2048` has finite losses, falling training/DEV loss and `under_one_hour_per_seed=true`; its receipt distinguishes 32 probe steps from the 2,048 target and projected evaluation points.
3. **I-Batch-Preflight:** the real loader returns exactly 2,048 batches and `used_windows=8,192`, with `usable_windows>=8,192`, plus total windows, train tokens, loaded-batch scored targets, peak host RSS, and exact projected eval points `[0,256,512,1024,2048]`. `train.json` repeats steps, batch count and eval points.
4. **I-Prefix:** at step 1,024, both seeds reproduce claude-131's stored top-1/export-mode/admitted counts exactly and full-precision losses within `1e-6`. A mismatch is source/data/device drift, not a dose result; stop and diagnose before interpreting step 2,048.
5. **I-Save-Load:** each reloaded model reproduces its plain predictions and loss exactly at step 2,048.
6. **I-Verify-All:** `d11-fbs-eval run --verify` gives G = H = R at all 82,176 positions per seed.
7. **I-D:** D is deterministic, D predictions and loss are saved, and the F-vs-D counterfactual is scored with paired F_I and `Δ_noise,L`.
8. **I-Inputs:** source commit, all three binary SHAs, four data pins, `d11-gap.json`, `train.json`, model/config/snapshot/artifact/windows/evaluator/gap/residual SHAs, exact commands and exit codes are recorded and verified. `d11-artifact-gap` checks `--model-sha256` and `--steps 2048`; every residual receipt binds the same inputs.
9. **I-Negative:** one deliberately wrong artifact or windows hash yields `i_inputs=false` and `gap_inside_band="not counted"`.
10. **I-DEV-Denominator:** 14,287 rows over 428 windows; do not conflate 1,572,864 training positions with DEV rows.
11. **I-Finite:** every command exits 0; stdout/stderr/exit codes and SHA256SUMS are saved and read back.

## 5. Data split

Use `w2-inputs/2026-10-25/driver-data/` with the existing pins:

- train tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`; train mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- DEV tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; DEV mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- No `fresh_v2` evaluation and no base checkpoint. Models train from step 0 at `e77a1f91f0f198de9aee58b7bc20d65a6c12b298` (or a descendant whose only changes are documentation). `--steps` accepts only 1,024 or 2,048 and is validated before opening CUDA; other longer doses are refused.
- Prefix control: `results/claude/d11-fbs-dual-train-2026-10-26/dual/train.json`, SHA-256 `5a290fbe47d8f10495b6d89fa8384110061902001ad9ebdf6b365935e220d9bc`. Compare both seeds at step 1,024 for exact top-1/top-8/admitted counts and losses within `1e-6`.

The two seeds are **initialization seeds on a common fixed batch stream and common DEV panel**, not independent data-order replicates. Each seed's paired row statistics are reported separately; no cross-seed confidence interval is inferred.

## 6. Steps and minimum power

1. Lease one RTX 5090 inside the fleet cap and build/copy `spiralcore-fbs-sheet`, `fbs-export`, and `d11-fbs-eval` with SHA-256 pins. Record the source commit and binary hashes in a run manifest.
2. Fetch/check section 5 inputs, run I-Bridges, I-Probe and I-Batch-Preflight, then stop on any failure.
3. Run `spiralcore-fbs-sheet --mode d11-train-export --train-forward dual --steps 2048 --data-root <driver-data> --save-dir save --stage4b d11-gap.json --out train.json` and apply I-Prefix to its step-1,024 evaluations against claude-131's sealed receipt.
4. Generate DEV windows. For each seed, export with `fbs-export`, verify with `d11-fbs-eval`, and run both:
   - `--mode d11-artifact-gap --steps 2048 --model-sha256 <model> --stage4b train.json --preds-artifact-out preds-artifact-seed-<s>.json` with model/evaluator/artifact/window/prediction pins;
   - `--mode d11-residual --steps 2048 --model-sha256 <model> --stage4b train.json --preds-d11-out preds-d11-seed-<s>.json` with the same inputs and label `longer-dual`.
5. Save every model, snapshot, F/D/A prediction, artifact, evaluator/gap/residual receipt, manifest, command, exit code and SHA256SUMS to the attempt-specific `$ARCHIVE` in section 9, read back, then release/down the pod.
6. Only for an unflagged PASS, run laptop `fbs-toks` decode/context, both zero-multiply audits and integer FBS serving tests.

**Power:** two initialization seeds, 14,287 DEV rows, 2,048 steps per seed (1,572,864 training positions), paired discordance and no seed averaging. The decision is the large registered gap reduction above, not a precise modest-effect estimate. No 2,048-step plain arm is run, so no dual-versus-plain causal comparison is licensed.

## 7. Cost and stop

- Base run: about 1.75 hours—10 minutes builds, 12 minutes dual training, 20 minutes verified evaluation per seed, residual scoring and archival. At `$1.11/h`, about `$1.95`; lease 2.5 hours and record complete cost. Healthy runs are renewed, not cancelled for estimates.
- Exporter-unresolved localisation is a separate GPU unit with its own FIT and cost, not included in this lease or authorized here.
- Stop before full training on failed I-Bridges/I-Probe/I-Batch. Stop after training on prefix, identity, verification or finiteness failure and preserve evidence. Classify instrument versus configuration before any conclusion.

## 8. Review

`TEST FITNESS: FIT` is recorded in section 10 at `5345b554038954a3d46fdedcf3c30daeeb9d429f`. D67 is still required before dispatch.

## 9. Exact dispatch and serving manifest

Source `e77a1f91f0f198de9aee58b7bc20d65a6c12b298`. Build and copy all three binaries, record their SHA-256s, and run them from the session-unique run directory. Inputs are fetched with the store client, checked against the pins, and no host model/parity fallback is allowed.

Attempt 1 stopped before training because the pin manifest used two `printf` placeholders for single hash-and-path arguments. This was an instrument failure, archived and read back at `results/kimi/d11-fbs-longer-dual-2026-10-26/`; it is not a model result. Attempt 2 uses one placeholder per line and attempt-specific pod/store paths so no retry overwrites failure evidence.

**Laptop lease boundary:** run `scripts/pod/uor-pod status` first and record the exact free GPU index as `GPU`. If no suitable free 5090 exists, run `scripts/pod/uor-pod up --lab kimi --session longer-dual --purpose "2,048-step dual export-gap FIT run" --hours 2.5 --count 1 --gpu 5090 --ref e77a1f91f0f198de9aee58b7bc20d65a6c12b298 --card https://github.com/UOR-Foundation/uor-r4/issues/2820`, assign `POD` to the exact id it prints, and set `GPU=0`. If a free 5090 already exists, replace `up` with `scripts/pod/uor-pod lease "$POD" --lab kimi --session longer-dual --gpus "$GPU" --purpose "2,048-step dual export-gap FIT run" --hours 2.5` and then `scripts/pod/uor-pod bootstrap "$POD" --ref e77a1f91f0f198de9aee58b7bc20d65a6c12b298`. Set `ATTEMPT=attempt-2` and `ROOT=/workspace/kimi-longer-dual-$POD-$ATTEMPT`, verify it is empty, and ship the supplied `local/kimi/d11-float-trunk/job.sh` with `scripts/pod/uor-pod ssh "$POD" "cat > '$ROOT/job.sh'" < local/kimi/d11-float-trunk/job.sh`. Run `scripts/pod/uor-pod run "$POD" --lab kimi --session longer-dual --gpu "$GPU" -- env POD="$POD" ATTEMPT="$ATTEMPT" bash "$ROOT/job.sh"`.

The pod job starts with `set -euo pipefail` and this receipt wrapper; every model command and instrument step uses it:

```sh
ATTEMPT=${ATTEMPT:?attempt id is required}
REF=e77a1f91f0f198de9aee58b7bc20d65a6c12b298
ROOT=/workspace/kimi-longer-dual-$POD-$ATTEMPT
RUN=$ROOT/run
DATA=$ROOT/driver-data
STAGING=$ROOT/staging
ARCHIVE=results/kimi/d11-fbs-longer-dual-2026-10-26/$ATTEMPT
run_step() {
  local name="$1"
  shift
  set +e
  "$@" >"$RUN/$name.out" 2>"$RUN/$name.err"
  local code=$?
  set -e
  printf '%s\n' "$code" >"$RUN/$name.exit"
  if [ "$code" -ne 0 ]; then
    printf 'step failed: %s exit %s\n' "$name" "$code"
    exit "$code"
  fi
}
mkdir -p "$RUN/bin" "$STAGING" "$RUN/store"
trap '(cd "$RUN" && find . -type f ! -name SHA256SUMS ! -name job.log -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS && sha256sum -c SHA256SUMS && /root/uor-store put "$RUN" "$ARCHIVE" && /root/uor-store ls "$ARCHIVE")' EXIT
run_step job-copy cp "$ROOT/job.sh" "$RUN/job.sh"
exec > >(tee -a "$RUN/job.log") 2>&1
SRC=/root/build/src-$REF
export UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR=/root/build/target-sm120
run_step build-sheet cargo build --release --locked --manifest-path "$SRC/Cargo.toml" -p uor-r4-float-train --features cuda --bin spiralcore-fbs-sheet
run_step build-export cargo build --release --locked --manifest-path "$SRC/Cargo.toml" -p uor-r4-integer --features serving-wasm --bin fbs-export
run_step build-eval env CARGO_TARGET_DIR=/root/build/eval-target-sm120 cargo build --release --locked --target x86_64-unknown-linux-gnu --features cuda --bin d11-fbs-eval --manifest-path "$SRC/docs/labs/d11-exporter-float-trunk/instrument/Cargo.toml"
run_step copy-binaries cp /root/build/target-sm120/release/spiralcore-fbs-sheet /root/build/target-sm120/release/fbs-export /root/build/eval-target-sm120/x86_64-unknown-linux-gnu/release/d11-fbs-eval "$RUN/bin/"
run_step binary-pins bash -c 'sha256sum "$1"/* > "$1/binary-SHA256SUMS"' _ "$RUN/bin"
run_step source-pin bash -c 'test "$(git -C "$1" rev-parse HEAD)" = "$2"' _ "$SRC" "$REF"
if ! command -v jq >/dev/null 2>&1; then
  run_step install-jq bash -c 'apt-get update && apt-get install -y jq'
fi
run_step disk-gate bash -c 'avail=$(df -k /workspace | tail -1 | tr -s " " | cut -d" " -f4); test "$avail" -ge 31457280'
run_step get-inputs /root/uor-store get w2-inputs/2026-10-25/ "$STAGING"
run_step link-data ln -s "$STAGING/w2-inputs/2026-10-25/driver-data" "$DATA"
run_step pins bash -c "printf '%s\n' \
  '3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8  mw-balp/train/tokens.u16' \
  'b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3  mw-balp/train/response_mask.u8' \
  '0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b  ft/dev/tokens.u16' \
  'a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078  ft/dev/response_mask.u8' \
  'd36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89  tokenizer.json' | (cd \"$DATA\" && sha256sum -c -)"
run_step get-gap /root/uor-store get results/claude/d11-fbs-export-gap-2026-10-26/d11gap/d11-gap.json "$RUN/store"
run_step stage-gap mv "$RUN/store/results/claude/d11-fbs-export-gap-2026-10-26/d11gap/d11-gap.json" "$RUN/d11-gap.json"
run_step get-prefix /root/uor-store get results/claude/d11-fbs-dual-train-2026-10-26/dual/train.json "$RUN/store"
run_step stage-prefix mv "$RUN/store/results/claude/d11-fbs-dual-train-2026-10-26/dual/train.json" "$RUN/train-1024.json"
run_step reference-pins bash -c "printf '%s\n' \
  '88b6a5f988fe144671365a258a100e5cf3935f98e917040b38f90df2637a1e8a  d11-gap.json' \
  '5a290fbe47d8f10495b6d89fa8384110061902001ad9ebdf6b365935e220d9bc  train-1024.json' | (cd \"$RUN\" && sha256sum -c -)"
run_step bridges cargo test --release --locked --manifest-path "$SRC/Cargo.toml" -p uor-r4-float-train --features cuda --lib
run_step probe "$RUN/bin/spiralcore-fbs-sheet" --mode d11-train-probe --train-forward dual --steps 2048 --data-root "$DATA" --out "$RUN/probe.json"
run_step probe-gate jq -e '.train_learns and .learns and .under_one_hour_per_seed and .probe_steps == 32 and .target_steps == 2048 and .projected_eval_steps == [0,256,512,1024,2048] and .inputs.batch_count == 2048 and .inputs.used_windows == 8192 and .inputs.usable_windows >= 8192 and .inputs.total_windows >= 8192 and .inputs.train_tokens == 7512797 and .inputs.scored_targets > 0 and .inputs.peak_host_rss_bytes > 0' "$RUN/probe.json"
run_step train "$RUN/bin/spiralcore-fbs-sheet" --mode d11-train-export --train-forward dual --steps 2048 --data-root "$DATA" --save-dir "$RUN/save" --stage4b "$RUN/d11-gap.json" --out "$RUN/train.json"
run_step train-gate jq -e '.instruments_pass and .steps == 2048 and .train_batches == 2048 and .eval_steps == [0,256,512,1024,2048] and (.seeds | length == 2) and all(.seeds[]; .i_save_load and (.run.eval | map(.step) == [0,256,512,1024,2048]))' "$RUN/train.json"
run_step prefix jq -e -n --slurpfile new "$RUN/train.json" --slurpfile old "$RUN/train-1024.json" '
  def point: .run.eval[] | select(.step == 1024);
  def close($a; $b): ($a - $b) | abs <= 1e-6;
  def valid($doc): ([$doc.seeds[].seed] | sort) == [1,2] and all($doc.seeds[]; (.run.eval | map(select(.step == 1024)) | length == 1));
  valid($new[0]) and valid($old[0]) and
  all($new[0].seeds[]; . as $n | ($old[0].seeds[] | select(.seed == $n.seed)) as $o |
    ($n | point) as $np | ($o | point) as $op |
    $np.dev_float.top1_hits == $op.dev_float.top1_hits and
    $np.dev_int.top1_hits == $op.dev_int.top1_hits and
    $np.dev_float.top8_hits == $op.dev_float.top8_hits and
    $np.dev_int.top8_hits == $op.dev_int.top8_hits and
    $np.dev_float.admitted_target_hits == $op.dev_float.admitted_target_hits and
    $np.dev_int.admitted_target_hits == $op.dev_int.admitted_target_hits and
    close($np.dev_loss_float; $op.dev_loss_float) and
    close($np.dev_loss_int; $op.dev_loss_int))'
run_step windows "$RUN/bin/spiralcore-fbs-sheet" --mode d11-artifact-windows --data-root "$DATA" --out "$RUN/windows.json"
for s in 1 2; do
  run_step "export-$s" "$RUN/bin/fbs-export" "$RUN/save/seed-$s/snapshot.json" "$RUN/artifact-seed-$s.uorlut"
  run_step "eval-$s" "$RUN/bin/d11-fbs-eval" run "$RUN/artifact-seed-$s.uorlut" "$RUN/windows.json" "$RUN/run-seed-$s.json" --verify
  run_step "hash-artifact-$s" bash -c 'sha256sum "$1" | cut -d" " -f1' _ "$RUN/artifact-seed-$s.uorlut"
  ART=$(cat "$RUN/hash-artifact-$s.out")
  run_step "hash-model-$s" bash -c 'sha256sum "$1" | cut -d" " -f1' _ "$RUN/save/seed-$s/model/model.safetensors"
  MODEL=$(cat "$RUN/hash-model-$s.out")
  run_step "hash-windows-$s" bash -c 'sha256sum "$1" | cut -d" " -f1' _ "$RUN/windows.json"
  WINDOWS=$(cat "$RUN/hash-windows-$s.out")
  run_step "gap-$s" "$RUN/bin/spiralcore-fbs-sheet" --mode d11-artifact-gap --steps 2048 --data-root "$DATA" \
    --model-dir "$RUN/save/seed-$s/model" --evaluator "$RUN/run-seed-$s.json" \
    --windows-sha256 "$WINDOWS" --artifact-sha256 "$ART" --model-sha256 "$MODEL" \
    --stage4b "$RUN/train.json" --stored-preds "$RUN/save/seed-$s/preds-float.json" --seed "$s" \
    --out "$RUN/gap-seed-$s.json" --preds-artifact-out "$RUN/preds-artifact-seed-$s.json"
  run_step "residual-$s" "$RUN/bin/spiralcore-fbs-sheet" --mode d11-residual --steps 2048 --data-root "$DATA" \
    --model-dir "$RUN/save/seed-$s/model" --evaluator "$RUN/run-seed-$s.json" \
    --windows-sha256 "$WINDOWS" --artifact-sha256 "$ART" --model-sha256 "$MODEL" \
    --stage4b "$RUN/train.json" --stored-preds "$RUN/save/seed-$s/preds-float.json" \
    --seed "$s" --label longer-dual --out "$RUN/residual-seed-$s.json" \
    --preds-d11-out "$RUN/preds-d11-seed-$s.json"
  run_step "gap-gate-$s" jq -e '.i_save_load and .i_inputs and .i_verify_all and .i_dev_denominator and .instruments_pass and (.gap_inside_band != "not counted: an instrument check failed")' "$RUN/gap-seed-$s.json"
  run_step "residual-gate-$s" jq -e '.i_save_load and .i_inputs and .i_verify_all and .i_dev_denominator and .i_d_determinism and .i_head and .instruments_pass and (.counterfactual_within_band != null)' "$RUN/residual-seed-$s.json"
done
# I-Negative: only the artifact pin is wrong; the seed-1 model pin is correct.
run_step hash-model-1-negative bash -c 'sha256sum "$1" | cut -d" " -f1' _ "$RUN/save/seed-1/model/model.safetensors"
MODEL1=$(cat "$RUN/hash-model-1-negative.out")
run_step negative "$RUN/bin/spiralcore-fbs-sheet" --mode d11-artifact-gap --steps 2048 --data-root "$DATA" \
  --model-dir "$RUN/save/seed-1/model" --evaluator "$RUN/run-seed-1.json" \
  --windows-sha256 "$WINDOWS" --artifact-sha256 0000000000000000000000000000000000000000000000000000000000000000 \
  --model-sha256 "$MODEL1" --stage4b "$RUN/train.json" \
  --stored-preds "$RUN/save/seed-1/preds-float.json" --seed 1 --out "$RUN/negative.json"
run_step negative-check jq -e '.i_inputs == false and (.gap_inside_band | contains("not counted"))' "$RUN/negative.json"
```

After every exit, the job's trap runs `(cd "$RUN" && find . -type f ! -name SHA256SUMS ! -name job.log -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS && sha256sum -c SHA256SUMS && /root/uor-store put "$RUN" "$ARCHIVE" && /root/uor-store ls "$ARCHIVE")` so failure evidence is preserved too.

On the laptop after the pod job exits, require `df -k ~ | tail -1 | tr -s " " | cut -d" " -f4` to report at least `31457280` KiB before and after readback. Run `scripts/pod/uor-store get "$ARCHIVE/" local/kimi/d11-float-trunk/readback`, then `(cd "local/kimi/d11-float-trunk/readback/$ARCHIVE" && shasum -a 256 -c SHA256SUMS)`. Only after every hash matches and the 30 GiB margin still holds, run `scripts/pod/uor-pod release "$POD" --lab kimi --session longer-dual` and `scripts/pod/uor-pod down "$POD" --lab kimi --session longer-dual`. Never down before the readback succeeds.

PASS-only serving: MacBookPro17,1, Apple M1 (4 performance + 4 efficiency cores), 16 GiB, AC power; `fbs-toks` records the OS and its binary SHA. Build with `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo build --release --locked -p uor-r4-integer --features serving-wasm --bin fbs-toks`. For each artifact run `local/target/release/fbs-toks <artifact>`: one warmup and 20 timed runs, 128-token prefill and 64-token greedy decode, median and p95 for context and decode separately. Run `python3 scripts/audit_zero_matmul_serving.py <release-rlib> --stack` and the same command on `local/target/release/uor-r4-stack`, plus `~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --no-fail-fast`.

## 10. FIT record

Agent-50 returned **TEST FITNESS: FIT** at `5345b554038954a3d46fdedcf3c30daeeb9d429f`, after the counterfactual, batch, scorer, ORDER, D68 and full pod-job/readback reviews. The final hardening commit `e77a1f91f0f198de9aee58b7bc20d65a6c12b298` adds missing-seed-loss refusal and passed the earlier host suite (60 lib, 3 CLI, 2 dependency, 12 kernel controls). The archived CUDA library suite at that source separately reports 82 passed and 5 ignored. D68 owner log #2820 comment `6428815813` and the executable job at `local/kimi/d11-float-trunk/job.sh` are part of this packet. Fresh D67 is still required before dispatch.
