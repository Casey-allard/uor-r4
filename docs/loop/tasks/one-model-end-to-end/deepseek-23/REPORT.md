# REPORT: one-model-end-to-end, deepseek-23

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Laptop only in this preparation; no
training, pod or spend yet.

## What did not work, first

- **The sheet's registered grade command could not run.** It used
  `cpu-checkpoint-grade --max-replies 0`, but the bin refuses zero reply
  examples. The amended command uses one one-token reply to exercise export
  while the continuation gate still reads the deterministic 512-row nearest
  grade.
- **Training cannot be stopped safely from outside at step 512.** A plain
  checkpoint poll would let the next 256 steps continue while scoring. The job
  sends `SIGSTOP` as soon as each checkpoint appears, exports and scores while
  frozen, then resumes only when no registered stop matches.
- **The chain is not runnable until its two code dependencies merge.** The SFT
  branch must be rebased onto the merged `--init-from` and per-turn EOS-rank
  changes before the final pin and exact-head D67 review.
- **`m2-reply512` is still unavailable.** No substitute was introduced.

## What changed

- Added the pod-side `job.sh`. It:
  - requires a clean source checkout at the exact 40-hex pin;
  - fetches and re-hashes the mixed SFT store, OASST2 held-out guard, base
    archive, `m2-fresh256`, tokenizer and DEV files;
  - rebuilds the checked base artifact and reproduces base panel controls;
  - runs one 1,024-step schedule from the exact `(b64L)` seed-1 checkpoint;
  - freezes the trainer at each 256-step checkpoint and runs
    `cpu-checkpoint-grade`, per-turn teacher-forced ranks, per-turn free
    generation and one-user free generation;
  - enforces the continuation floor and the step-512 no-ranking-movement stop;
  - writes full hashes, uploads to a new store root, downloads it again and
    verifies the readback.
- Added `run-s1.sh`. It checks the launch pin, creates only one DeepSeek session
  pod, uploads the exact job by hash, renews the lease, watches through a
  terminal state, verifies the store's `READBACK`, and releases/downs the pod.
- Added `gate.sh` and `test-mutants.sh` for the registered stop, hash, lease,
  readback and teardown rules.
- Amended the prep sheet's grade command to `--max-replies 1
  --reply-tokens 1` and recorded why.

## Measurements

No model or GPU step ran. The job's fixed runtime envelope remains the sheet's:
one RTX 5090, about 11–12 training minutes, about 36–39 minutes planned wall
time, and about $0.69–0.75 at $1.15/h.

The score roots are new per controller run:

- pod run: `/workspace/uor-r4/runs/deepseek-sft-s1-<run-id>`;
- store:
  `results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1-<run-id>`.

## Checks run at the head

- `gate.sh`: 68 complete-line checks passed. Hash, stop, seed, lease,
  readback, upload and teardown predicates are pinned as whole lines rather
  than substrings.
- `test-mutants.sh`: unmutated gate passed first; 21 of 21 semantic mutations
  were then caught, including disabled/weakened stops, seed 2, hidden input-hash
  changes, a forged hash comparison, directory-only or true-valued readback and
  omitting either `SIGCONT` that delivers termination to a frozen trainer,
  changing only the checkpoint-grade flags, or pointing the panel binary away
  from the actual build target. The unmodified job also rejected seed 2 with
  the registered message before touching the store or filesystem.
- `bash -n` and `shellcheck` on the operational scripts: passed.
- The job was reviewed against the actual `uor-store`, `uor-pod`,
  `train-in-space`, `cpu-checkpoint-grade` and `served-reply-panel` command
  interfaces.

Not run: GPU build, input download, model training, pod launch or store write.

## Skills used

No system-provided Codex skill applied. The repository's loop, compute, geometry
and sealing rules were used directly.

## Still open

- Rebase after the trainer and diagnostic PRs merge, then record their pin.
- Fresh FIT and D67 reviews of the exact merged scripts and data packet.
- Execute seed 1 and apply the outcome map.

## Plain summary

The SFT plan now has an executable guarded path. One DeepSeek-owned GPU pod
rechecks every input, restores the existing model, trains the fixed schedule,
freezes at each 256-step checkpoint to score EOS and answer continuation, stops
at 512 when the registered no-movement condition holds, seals all evidence,
reads it back and is taken down. No run has started yet.
