# Starling read pivot: selection works and the value path works; the read does not learn to pick the source by content (11 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032).
**Line:** starling read · count 3/3 (pivot) · headline "softmax at runtime: yes" → yes.
**Pivot card:** [6112716462](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6112716462).
**Owner decision requested:** [6114147704](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6114147704).

After rung 3 ([record](../starling-content-2026-10-11/README.md)), the pivot card ran cheap diagnostics to choose the configuration of one decisive run. Every GPU step was pre-registered on #2032 and passed an adversarial TEST FITNESS review before compute, per the owner's rule for targeted GPU runs. The total GPU cost was under $0.50.

## What each step showed

| Step | Question | Result |
|---|---|---|
| **D2, velocity** (laptop, [6113389082](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6113389082)) | Does a recurrence lane's per-token turn rₜ predict the next token beyond position? | **No.** A random 120-way hash of the current token carries 1.72 bits; the best lane's turn carries 1.65. The turn is mostly token identity, so velocity routing was dropped. |
| **D1, oracle** ([6113580742](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6113580742)) | Can the read recall when the source is in every head's kept set (oracle on in training)? | Recall **1.000** on 2 seeds. Correction: the oracle used a fixed slot, so this shows the **value path** works, not content selection ([6113795870](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6113795870)). |
| **Shared kept set** ([6113795870](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6113795870)) | Every head reads the union of all heads' kept positions | The per-head hit rate is **0.61–0.72**; recall is **0.012**. Availability is not the limit. |
| **Content-ranked** ([6114147704](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6114147704)) | Kept slots compete by sorted q·k through the rank table | Recall is 0.007–0.049. The source is ranked first when kept only **0.03–0.12** (shared) or **0.28–0.29** (unshared), against chance of about 0.05 and 0.14. |

## Conclusion

- **The geometric selection works:** the 600-cell neighbour lookup keeps the true source 36–81% of the time.
- **The value path works.**
- **What fails is learning q·k to pick the source among the kept candidates.**
  - Here q and k learn only through gate′ × rank weight on a few kept slots.
  - The L2 read learns q·k matching because its softmax gives every position a competitive gradient.
  - The flock rank reads used a **training-time softmax surrogate** (sorted forward, softmax-over-kept backward) and came within +0.0185 BPB of softmax ([brief](../../mechanisms/flock-rank-reads.md)).
- **Recommended decisive run** (owner decision, D22): the same, on starling candidates. The runtime read stays softmax-free (acceptance item 0); the softmax is used in training only.

## Code delivered (all opt-in, tested)

**`read_route=starling:M:K[:content][:shared][:ranked]`:**
- **`shared`:** every head reads the union of all heads' kept positions. The rank table starts uniform.
- **`ranked`:** kept slots are weighted by their content rank (`content_order`), with a harmonic initialisation.

**Diagnostics (thread-local, so tests and models never mix them):**
- the evaluation and training oracle (`starling_oracle_set`, `mqar-bench starling_oracle_train=`);
- the top-content-match record (`starling_top_start`/`take`);
- mqar-bench's oracle recall, per-head hit rate and "source ranked first when kept";
- positions-scored now counts the content gate's slots.

**Instrument:** `examples/velocity-info.rs`, the D2 replay. It matches the model's core output to within 1e-5.

**Tests** (10 starling, plus `content_order`): union equality, ranked against address order, finite-difference gradients, parse rules, and the oracle placing the source in every head. Each fix has a negative gate: the union step skipped, the re-rank skipped, the sort direction flipped.

## Files

Run logs are in cloud-store `claude/starling-oracle-20261011`, `starling-shared-20261011` and `starling-ranked-20261011`. The D2 model is in `quaternion-study-20261011`.
