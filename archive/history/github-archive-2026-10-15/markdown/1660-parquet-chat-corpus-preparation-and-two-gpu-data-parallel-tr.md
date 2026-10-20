# #1660 — Parquet chat-corpus preparation and two-GPU data-parallel training for the 100M rung

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T21:37:54Z
- **Updated:** 2026-10-09T13:08:46Z
- **Closed:** 2026-10-03T21:39:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1660
- **Merged:** 2026-10-03T21:39:11Z
- **Merge commit:** b88ba1666de22f5cbb25fa5776273d965eea863e
- **Base / Head:** main / UOR-Foundation:codex/prepare-chat-parquet
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Two training-side tools the 100M rung needs. Both are opt-in, and default builds and default `train` runs are unchanged.

## 1. `prepare-chat-parquet` (feature `parquet-input`)
- **What it does:** builds a protocol-2 `uor-r4-chat-corpus/v1` split directly from Hugging Face parquet exports.
  - Each `source=LABEL:LICENCE:SKIP:FILES` skips its first SKIP rows, past chat-v0's fetched prefix, so the new rows are disjoint from chat-v0's train and held-out splits and from the chat panel drawn from that split.
  - Rows that are empty, have no response, are oversized or contain special-token text are dropped and counted.
  - Encoding runs on all cores in fixed batches; output is independent of the thread count.
- **Real output** (pod, 3 October): `chat-v1`, built from the full SmolTalk smol-magpie-ultra train set (Apache-2.0, skip 13,900) and UltraChat 200k train_sft (MIT, skip 10,200).

  | Rows used | Tokens | Response tokens | Oversized rows dropped |
  |---|---|---|---|
  | 589,939 | 1,388,814,239 | 1,160,356,227 (83.6%) | 3,363 |

  - `DialogueSplit::load` and `train` read it, and the 100M run is training on it.
- **Dependency:** `parquet` 60 is optional (snap and zstd only, no arrow) and is compiled only with the feature.

## 2. `train data_parallel=2` (CUDA)
- **How it works:**
  1. Each step draws the same batch as a single-GPU run.
  2. Equal halves run in parallel on GPU 0 and a replica on GPU 1.
  3. The replica's gradients are averaged into GPU 0's (`average_replica_gradients`).
  4. AdamW updates on GPU 0, and the weights are copied back (`StackModel::copy_variables_from`).
- **Equivalence:** the mean of the two half-batch means equals the full-batch mean. Rounding differs from a single-device run.
- **Refused combinations:** non-CUDA devices, an odd batch, `qat` and `transport_snap`.
- **The report** records `data_parallel`.

**Measured on Runpod 2× RTX 4090** (WIP build of the same code, 180 s each):

| Model | Single GPU, batch 32 | `data_parallel=2`, batch 32 | Speed-up |
|---|---|---|---|
| 29M | 58.1k tok/s | 86.8k tok/s | 1.49× |
| 77.5M | 29.0k tok/s | 42.6k tok/s | 1.47× |

**Same-seed 29M curves**, single GPU vs `data_parallel=2`:
- train loss agrees within 0.005 at every logged step (100–800);
- dev loss differences are within ±0.02 at 8 evaluation windows.

## Checks at `74d8189d`
Run on the pod, Linux x86, rustc 1.97.1, from `git archive`, with `UOR_BUILD_SOURCE_COMMIT` set:
- `cargo fmt --all --check`: exit 0.
- `cargo test --release -p uor-r4-training --lib replica_copy`: 1 passed. This is a new CPU test: the replica copy is exact, and the averaged half-batch gradient equals the full-batch gradient within 1e-5 + 1e-4·|g|.
- `cargo test --release -p uor-r4-training --features parquet-input --bin prepare-chat-parquet`: 1 passed.
- `cargo build --release -p uor-r4-training --features cuda --example geometric-stack`: built.
  - binary sha256 prefix `1513c8cd`;
  - this binary runs the 100M rung (`/root/run-100m.sh`).

Not run: the full workspace suite, and Metal (no Mac GPU in this path).

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5402859856 — 2026-10-03T21:38:05Z — Casey-allard — COMMENTED

**Exact-head review at `74d8189da3c63b011ebd10912917a75e7a741723` (Claude lab; self-review under the owner's 2026-09-30 rule).**

**Code read**
- Both new paths are opt-in: the `parquet-input` feature, and `data_parallel=2` validated to CUDA with an even batch and no qat or snap.
- The single-GPU branch of the training step is unchanged.
- The gradient average fails loudly when only one replica has a gradient.

**Checks at this head** (the PR body's list, run on the pod from `git archive`): fmt 0; the new replica test passes; the converter test passes; the CUDA example builds. That binary is running the 100M rung.

**APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5402859856_

