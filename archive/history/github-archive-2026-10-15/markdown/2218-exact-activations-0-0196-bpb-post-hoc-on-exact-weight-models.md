# #2218 — Exact activations: +0.0196 BPB post-hoc on exact-weight models, trained-in no better (row 3); evaluate on GPU (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T00:07:39Z
- **Updated:** 2026-10-12T00:08:53Z
- **Closed:** 2026-10-12T00:08:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2218
- **Merged:** 2026-10-12T00:08:41Z
- **Merge commit:** 4cdb0f36c649d0a754896fd8f6cab6d29043d10b
- **Base / Head:** main / UOR-Foundation:claude/activation-snap
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** geometric weights · count 1/3 · headline "softmax at runtime: yes" → yes (0.886838 BPB served, unchanged)

**Result: row 3.** Exact activations cost +0.0196 BPB when snapped after training. Training the snap in does not help (+0.0231).
- **Reading:** #2032 [6114831893](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6114831893).
- **Pre-registration:** [6115101545](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115101545).
- **Fitness:** [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115118695).

| Arm (3 seeds, all on CUDA) | BPB | vs QM2g |
|---|---:|---:|
| QM2g, exact weights | 1.5148 | — |
| **QM2g-post**, activations snapped after training | **1.5343** | **+0.0196** |
| QA, activation snap trained in | 1.5379 | +0.0231 |

**Usable now:** fully exact quaternion maps (weights and inputs) at +0.021 against dense at equal parameters.

**What it adds:**
- \`ActivationSnap::IcosianPair/IcosianPairF32\`: the weights' exact set on every quaternion map input, straight-through, on the device snap.
- \`train activation_snap=\`, saved in the model.
- \`evaluate device=cuda\` (and \`tf32\`); the JSON records the device and the snap actually applied.
- Snap tables cached per device, and shared inputs snapped once. This fixed a throughput-gate blocker: 32.9k → 42.4k tok/s.

**Compute:** all training, evaluation and unit tests ran on the pod; the laptop did no model or build work.

Record: `docs/labs/activation-snap-2026-10-11/README.md`. References #2032.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6115423654 — 2026-10-12T00:07:41Z — Casey-allard

**Exact-head checks for `7d2809bef899ecda0c61d99082773ab32b9d4c6b`.** Code identical to the pod-tested 866b13d2b: `git diff 866b13d2b 7d2809bef899ecda0c61d99082773ab32b9d4c6b -- crates` is empty; the rebase changed docs only. All checks below ran on the pod, per the owner's direction against laptop builds.
- **Pod bootstrap at 866b13d2b:** release build of `geometric-stack`, `mqar-bench` and the others, plus `cuda_stack_ops_parity`: **37 passed**.
- **Pod CPU, fresh checkout of 866b13d2b:** `cargo test --release -p uor-r4-training --lib -- icosian snap activation_snap mantissa quaternion`: **36 passed**. This includes the device-snap bit-identity test and the activation round-trip test.
- **Negative gates** (shown on #2032): not saving the activation mode fails the round trip; a wrong exponent rounding fails bit identity.
- **Runs:** all 6 training runs and 9 evaluations on CUDA at this code. Throughput gate passed (42.4k ≥ 39k).
- `check_claim_wording.py`: PASS.

### review 5485616037 — 2026-10-12T00:07:42Z — Casey-allard — COMMENTED

**Self-review at exact head `7d2809bef899ecda0c61d99082773ab32b9d4c6b`.**
- The activation snap is opt-in; the probe modes are unchanged.
- The table cache and the one-entry activation cache are thread-local. Tensor ids are never reused, so a cache hit is the same tensor.
- The scoring follows the pre-registered rows on a single device.
- The record states row 3 plainly and names the usable post-hoc configuration.

APPROVE (self-review is allowed under the owner's exact-head rule).


---
_Comment/review ids in this file: 6115423654, 5485616037_

