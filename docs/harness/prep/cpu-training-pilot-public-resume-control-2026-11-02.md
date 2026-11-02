# Prep sheet: public CPU corpus restart equivalence, 2 November 2026

Lab: deepseek. Item: [cpu-training-pilot](../../roadmap/items/cpu-training-pilot.md) (#3506). Order: [deepseek-3](../../loop/tasks/cpu-training-pilot/deepseek-3/ORDER.md). Source base for the implementation: `636f5e1ed`.

TEST FITNESS: FIT

## The question

On the named Ryzen 9 9950X, does a hash-bound step-64 checkpoint from the public masked corpus resume in a fresh process to total step 256 and produce exactly the same final artifact, final checkpoint and final panel as an uninterrupted 256-step process, independently on both public trained FBS artifact seeds?

This is a host determinism and state-continuity control. It evaluates the checkpoint/resume instrument, not a new training recipe or language capability.

## Arms

- **Uninterrupted:** one fresh process runs 256 update attempts from the pinned source artifact.
- **Interrupted and resumed:** one fresh process runs 64 update attempts and exits; a second fresh process loads the step-64 checkpoint and runs exactly 192 further attempts to total step 256.

Each arm uses trainer seed `1`, evaluation stride `2`, evaluation cap `1024`, checkpoint interval `64`, the same deterministic corpus order and the same evaluation panel. The pair is repeated independently for artifact seeds 1 and 2.

## Base artifact

- Seed 1: `dual-c131-seed-1-raw_both.uorlut`, SHA-256 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`.
- Seed 2: `dual-c131-seed-2-raw_both.uorlut`, SHA-256 `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.
- Both are public `uor-r4.lut-stack/5` artifacts with vocabulary 4,096, width 256, context 192, six recurrence layers, raw Hamilton gates/transition and native writer.

The run binds the built `cpu-corpus-driver` binary, source commit, Rust/Cargo 1.97.1, both artifacts, corpus, mask, manifest, trainer seed, split controls and every checkpoint hash.

## Data split

- Tokens: `dev/tokens.u16`, SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`.
- Response mask: `dev/response_mask.u8`, SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- Manifest: `dev-manifest.json`, SHA-256 `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab`.
- The manifest pins 42,856 payload tokens, 14,287 response targets, tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, BOS 0 and a 256-token source horizon.
- Whole BOS-delimited documents are split by ordinal: even documents train, odd documents evaluate. The training side has 200 documents and 7,080 targets; the fixed evaluation side is capped at 1,024 rows.
- The public manifest says `split=train`, so the evaluation side is a document-disjoint internal development set. Base-model exposure is not claimed as fresh.

## Steps

- Uninterrupted arm: `0` to `256` update attempts in one process.
- Interrupted source arm: `0` to `64` update attempts, then clean process exit.
- Resumed arm: hash-bound step `64` to total step `256`, executing exactly `192` attempts.
- Every arm writes interval and final checkpoints. The resumed invocation must verify the sidecar before constructing or advancing the trainer.

Command shape:

```sh
cpu-corpus-driver <artifact> <tokens> <mask> <manifest> \
  1 <total-steps> 2 1024 64 <fresh-run-directory> [step-64-checkpoint]
```

## Bar (pre-registered)

For each artifact seed independently, the interrupted/resumed arm must match the uninterrupted arm exactly:

- `resumed_from_step = 64` and `executed_steps = 192`;
- final artifact bytes and SHA-256 are identical;
- final checkpoint bytes and SHA-256 are identical;
- final panel admission, top-1, top-8, rank sums, margin sums and export counters are identical;
- CPU/serving query-coordinate, token and kept-cell mismatches are exactly zero in both arms;
- every resumed input and the checkpoint binding verify before the first resumed update.

A deterministic mismatch, admission loss or nonzero export mismatch stops the comparison and is classified as an instrument/continuity failure before any training interpretation. The control passes only when every item above holds on both artifact seeds. There is no statistical tolerance because the claimed outcome is byte and reading equality for the same deterministic process.

## Instrument checks

- Finite tests compare uninterrupted and interval-resumed runs and mutate every sidecar identity field. Source artifact, token, mask, manifest, tokenizer, trainer seed, evaluation stride, evaluation cap, completed step, schema and checkpoint hash must all be refused on mismatch.
- The resumed run must use a separate output directory and may not inherit live trainer state from the source process.
- The final artifact must parse through `IntegerStackModel`, and the final checkpoint must reproduce the live trainer artifact before success.
- The step-64 checkpoint remains in the ignored local run root because the charter forbids committing binaries. Its sidecar, exact checkpoint hash and the resumed receipt are copied into evidence. The public runner captures source commit, binary hash, Rust/Cargo versions, kernel, CPU, RAM and filesystem identity.
- A pre-FIT comparison retained in this worktree is diagnostic only. It establishes that the commands are executable and is not the registered outcome.

## Minimum test power

- Two independently published trained artifact seeds are exercised.
- The outcome is exact state equality, so the relevant power is two independent process boundaries rather than a statistical row count.
- One step-64 boundary per seed is tested. Crash consistency during checkpoint writes, multiple boundary positions, power loss and filesystem damage remain outside this claim.

## Cost

- Local Ryzen 9 9950X, 16 physical cores / 32 threads, 60 GiB RAM: `$0` external compute.
- Six runs total: three per seed (64-step source, uninterrupted 256-step reference and 192-step resume), each under two minutes at the measured public rate.
- Preparation/build, FIT review, checks, evidence and delivery are local. No GPU, pod, rental, upload or paid API call is used.

## Stop rule

- Stop before a resumed arm if the source checkpoint or binding is missing, any pinned input differs, the binary/source identity differs, any tamper control is accepted, or the checkpoint reports a step other than 64.
- Stop the comparison on artifact, checkpoint, panel, admission or export inequality.
- Preserve a failed run as an instrument or determinism result; do not call it a training or capability negative and do not alter the bar after observing bytes.
- A healthy run is not shortened because of elapsed wall time or an unimpressive training margin.

## Claim boundary

A pass supports only hash-bound, process-separated continuation from this step-64 boundary on these two public CPU development runs to byte-identical final state. The sidecar is an integrity binding, not a signature or external trust anchor. It does not establish crash consistency, arbitrary checkpoint corruption recovery, a positive training configuration, complete-reply or fresh-phrasing capability, GPU qualification or useful language training.

## Verdict

The independent non-author review is recorded separately in
[FIT_REVIEW.md](../../loop/tasks/cpu-training-pilot/deepseek-3/FIT_REVIEW.md).
The sheet must remain byte-identical to the reviewed hash.
