# Prep sheet: bounded public CPU rank-margin pilot, 2 November 2026

Lab: deepseek. Item: [cpu-training-pilot](../../roadmap/items/cpu-training-pilot.md) (#3506). Order: [deepseek-1](../../loop/tasks/cpu-training-pilot/deepseek-1/ORDER.md). Source base: `02767ac73`.

TEST FITNESS: FIT

## The question

On the Ryzen 9 9950X, does a bounded deterministic rank-margin pass over the public response-masked corpus improve document-disjoint internal next-token ranking on both public trained dual-c131 artifact seeds without losing target admission or exact CPU/serving parity, and at what measured cost?

This is a public internal-development pilot. It does not substitute for complete-reply or genuinely fresh-phrasing qualification.

## Arms

- **Baseline:** the published artifact with zero updates, evaluated over the fixed public development panel.
- **Training:** exactly 256 rank-margin update attempts from the same published artifact, with the same corpus order and schedule.
- Both arms run separately from artifact seed 1 and artifact seed 2. The artifact seed selects the base model; the trainer's deterministic order seed is fixed at `1` for both so model-seed differences are not confounded by a second shuffle.

## Base artifact

- Seed 1: `dual-c131-seed-1-raw_both.uorlut`, SHA-256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`, 2,144,960 bytes.
- Seed 2: `dual-c131-seed-2-raw_both.uorlut`, SHA-256 `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`, 2,144,960 bytes.
- Both are public `uor-r4.lut-stack/5` artifacts: vocabulary 4,096, width 256, MLP 384, six recurrence layers, context 192, raw Hamilton gates and raw transition, native writer, no Hamming or turn memory.
- The run pins the built binary, source commit, Rust 1.97.1 toolchain, artifact hash, checkpoint hash and final artifact hash before any outcome is interpreted.

## Data split

- Corpus: `caseyallard/uor-r4-data` `dev/tokens.u16`, SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, and `dev/response_mask.u8`, SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- The manifest pins 42,856 payload tokens, 14,287 response targets, tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, BOS 0 and a 256-token horizon.
- Split whole BOS-delimited documents by ordinal with evaluation stride 2: odd ordinals are evaluation, even ordinals are training. No document crosses the split.
- The split contains 200 training documents and 7,080 training targets. The public manifest identifies this corpus as `split=train`; the base models' historical exposure is not verified. The evaluation side is therefore a document-disjoint internal development set, not held-out data, and is deterministically capped at 1,024 rows sampled evenly from the reserved documents.
- Inputs use at most the artifact's complete 192-token causal context. Targets and masks are represented identically in training and evaluation.

## Steps

- Baseline: `0` update attempts.
- Training: `256` update attempts.
- Checkpoints: every `64` attempts and at final; the final checkpoint is independently reloaded by the driver and compared to the live trainer artifact.
- Command template:

```sh
cargo run --release --locked -p uor-r4-cpu-train --bin cpu-corpus-driver -- \
  <artifact> <tokens.u16> <response_mask.u8> <manifest.json> \
  1 <steps> 2 1024 64 <run-directory>
```

The `<steps>` value is `0` for the baseline and `256` for training. The final artifact is written as `<run-directory>/final.uorlut` and must parse through the integer serving engine.

## Bar (pre-registered)

For each artifact seed independently, compare the 256-step result with that seed's zero-update baseline on the same 1,024 rows.

- **Export:** CPU versus `IntegerStackSession` query-coordinate, token and kept-cell mismatches are exactly zero for baseline and final.
- **Admission:** final `target_admitted` is at least baseline `target_admitted`.
- **Primary ranking gain:** either top-1 or top-8 increases by at least `ceil(3 * sqrt(N * p * (1 - p)))`, where `N` is the evaluated rows and `p` is that seed's baseline rate for the same metric.
- **No trade:** the non-improving ranking metric does not decrease.
- **Checkpoint:** reloading the final checkpoint reproduces the artifact and checkpoint bytes.
- **Development positive:** every bullet above passes on both artifact seeds.
- A one-seed pass, no-change result or loss-only improvement is a configuration negative for this bounded public pilot. It is not relabelled as a language result.

The already observed cap-256 baseline readings are retained as context only, not as the bar and not as run results:

| Artifact seed | Top-1 | Top-8 | Admitted | Margin sum |
|---|---:|---:|---:|---:|
| 1 | 66 / 256 | 93 / 256 | 97 / 256 | 2,634,836 |
| 2 | 53 / 256 | 72 / 256 | 73 / 256 | 8,161,930 |

The actual run computes its own cap-1,024 baseline before the trained arm.

## Instrument checks

- Source and artifact hashes match this sheet.
- The public corpus manifest, UORT header, mask count and tokenizer identity pass the driver checks.
- Every accepted trainer update changes at most the selected trackable coordinates and re-exports a parseable exact artifact.
- A 16-step timing probe runs on each seed in a separate fresh run directory from the untouched pinned artifact. The 256-step arm starts from another fresh load of that same artifact and does not inherit probe state.
- Trainer checkpoint/resume equality has finite component coverage from the merged corpus-driver tests. `cpu-corpus-driver` has no public-run resume argument, so public-corpus restart from an interval checkpoint is `NOT_RUN` and remains a #3506 gap.
- Peak RSS comes from process high-water mark. No run occurs outside the local workstation or through a pod.
- Before either arm, capture source commit, built-binary SHA-256, Rust and Cargo versions, OEM hardware identity, `uname -a`, `lscpu`, `free -h` and `df -h` into the run summary directory:

```sh
mkdir -p "$run/identity"
git rev-parse HEAD > "$run/identity/source.txt"
sha256sum target/release/cpu-corpus-driver > "$run/identity/binary.txt"
rustc --version > "$run/identity/rustc.txt"
cargo --version > "$run/identity/cargo.txt"
uname -a > "$run/identity/uname.txt"
lscpu > "$run/identity/lscpu.txt"
free -h > "$run/identity/free.txt"
df -h . > "$run/identity/df.txt"
```

## Minimum test power

- Two independently published artifact seeds are tested.
- Each seed evaluates 1,024 fixed, document-disjoint rows from the same pinned public corpus.
- The three-standard-deviation per-seed bar above is a screening threshold for this development pilot. The 1,024 rows exceed the project's 200-row minimum, but they are not a sealed fresh draw, base-model exposure is unverified and they are not claimed as statistically independent qualification evidence.
- A miss is retained as a negative for this 256-step configuration; it does not reject the rank-margin mechanism or authorize more seeds mid-run.

## Cost

- Local Ryzen 9 9950X, 16 physical cores / 32 hardware threads, 60 GiB RAM: `$0` external compute.
- Two seeds times baseline plus 256 update attempts, four checkpoints and panel evaluation. The 16-step timing probe caps the full pilot at one hour per seed; projected total wall time is under two hours.
- No GPU, pod, rental, corpus upload or paid API call is part of this pilot.

## Stop rule

- Stop a seed before its 256-step arm if the 16-step probe projects more than one CPU hour, if any artifact/corpus identity differs, if the CPU/serving export is not exact, if admission is lost, or if checkpoint reload is not identical.
- A malformed or non-finite trainer result stops the pilot and retains the run root as an instrument failure.
- A healthy arm that starts is not shortened because its preliminary loss is unimpressive. It runs to 256 or a declared stop condition.
- Complete-reply and genuinely fresh-phrasing outcomes remain `NOT_RUN`: the sealed `m2-fresh256` and `m2-reply512` inputs/labels return HTTP 401 without credentials. A public prompt panel can be emitted only as ungraded qualitative output.

## Claim boundary

A pass supports only the narrow statement that the tested 256 subsequent CPU updates improved public document-disjoint internal next-token ranking while preserving admission and exact serving parity. It does not establish that the published base models were unexposed to those documents. It is development research under D84, not GPU qualification, language capability, complete-reply success, fresh-phrasing transfer, public-run restart or evidence that CPU training should replace a qualification path.

## Verdict

TEST FITNESS: FIT - fresh non-author adversarial review of sheet SHA-256 `8751892a7c124872870c48b98914e4dd8b015c70f0e7fd6618015aed40f20c62`.

The reviewer confirmed the document-disjoint/internal-exposure boundary, exact fixture identities, zero/256-step arms, fresh 16-step probe, per-seed three-standard-deviation rules, no-trade/admission/export gates, identity capture, cost and stop rule. It also confirmed public-run restart and complete-reply/fresh-phrasing remain `NOT_RUN`. This FIT clears only the bounded public development pilot in this sheet; it does not clear a full #3506 closure claim.
