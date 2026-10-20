# #1557 — Let dialogue-train read 384 positions: extend a model's context and its episodes (D18 A2)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T00:53:30Z
- **Updated:** 2026-10-09T13:08:45Z
- **Closed:** 2026-10-01T01:03:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1557
- **Merged:** 2026-10-01T01:03:32Z
- **Merge commit:** c190c7c2fa23570cc7cb1bb053a41def6a27a161
- **Base / Head:** main / UOR-Foundation:codex/d18-context-384
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Why

The owner ran `s8-panel-count` on the sealed §8 panel (#1552): it needs up to **317 positions** with a 32-token reply budget (285 with empty replies). That is more than the stack's 256. The owner kept D18 and re-entered A2 as source work. This PR is the training side of a 384-position window.

## What it changes

**`StackModel::extend_context(N)`**
- Each read's age table (`read.age`, `[heads, context]`, indexed by age) keeps every learned entry.
- Past the saved context, each head continues along its initial slope (`-2^(-8(h+1)/heads)` per position) from its last learned value.
- So **the model scores sequences up to the saved context bit for bit as before**.
- The transformer control has no age table, so only its context changes. A smaller context is refused, and so is a model with a served representation set.

**`dialogue-train context=N`** (default 256, at least 256)
- It is a shape option for a new model. After `init=`, it may only grow the saved context.
- Training episodes fill the model's context. Episode contracts now admit any context from 2 to 4,096.
- **The development panel stays the 256-ID panel**, so development NLL stays comparable with earlier runs. `evaluate_development_nll` now accepts a model context of at least the panel's, not only an equal one.

**Bare `truncated_prefix`** now means as many prefix IDs as fit at any context (KEEP 4,094). At 256 it keeps exactly what KEEP 254 kept (tested). `truncated_prefix:KEEP` accepts 1 to 4,094.

**Unchanged:**
- A1's M-world v2.1 instrument and its 256-ID episodes;
- every FullPrefix and RoleOnly record at 256.

The D11 runtime already sizes everything from the artifact header. `stack_d11_snap` exports and serves a context-8 model.

## Checks run

- `cargo fmt --check`.
- `cargo test -p uor-r4-training --lib -- dialogue_episodes stack_dialogue extending_the_context`: **14 passed**. They include:
  - `extending_the_context_keeps_every_learned_age_and_reads_further`: Lorentz, Dot and transformer; logits within the old context are bit-identical; ages follow the rule; save and load at the new context;
  - `a_longer_context_admits_longer_documents_and_bare_truncation_fits_any`.
- **End-to-end release smoke:**
  - `dialogue-train init=arm-C-s2/model context=384 policy=truncated_prefix steps=2` on chat-v0 (2 threads, 6 s).
  - The saved model has `context: 384` and 2,107,913 parameters, exactly +2 reads × 4 heads × 128 ages.
  - It trained on **68,038** eligible responses, against 50,592 at 256, of which 36,600 had a cut prefix.
  - The 256 development panel scored normally.
- The exact-head GitHub run on `codex/ci/v3/context-384` covers the `uor-r4-training` lib, the `geometric-stack` example and the `stack_d11_snap` integration test, with workflow v3.

**No training result.** Training at 384, and serving the pointer under D11, remain to do. The pointer has no D11 port yet (D18 §5).

References #1552

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5922603436 — 2026-10-01T01:02:39Z — Casey-allard

**Claude lab, exact-head self-review at `9e3d2317`: APPROVE** (owner rule, #820). I read the whole diff.
- `extend_context` touches only `*.read.age` tensors. It keeps rows verbatim, and refuses shrinking and served models. The test shows logits bit-identical within the old context for Lorentz, Dot and transformer, and save/load at the new context.
- `dialogue-train` now builds the configuration before the episode index. The init path extends before adding select or pointer, and `model.config == config` still guards it. The development panel stays at the 256 contract, with `evaluate_development_nll` relaxed from `==` to `>=`.
- Bare `truncated_prefix` (4,094) is tested equal to KEEP 254 at 256. No other caller depended on the 254 value.

Exact-head run [36798399840](https://github.com/UOR-Foundation/uor-r4/actions/runs/36798399840), workflow v3:
- fmt, all-targets check: success;
- `uor-r4-training` lib **409/0**;
- `geometric-stack` example 8/0;
- `stack_d11_snap` 2/0.

### comment 5922615927 — 2026-10-01T01:03:54Z — Casey-allard

Merged as c190c7c2fa23570cc7cb1bb053a41def6a27a161. Verified: `git merge-tree --write-tree 4ec11d7d 9e3d2317` equals the squash tree (e51d2a5d). `dialogue-train context=N` and `StackModel::extend_context` are on main. No model has been trained at 384 yet.


---
_Comment/review ids in this file: 5922603436, 5922615927_

