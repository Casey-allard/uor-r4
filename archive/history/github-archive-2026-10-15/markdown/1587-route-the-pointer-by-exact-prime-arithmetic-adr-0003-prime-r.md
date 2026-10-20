# #1587 — Route the pointer by exact prime arithmetic (ADR-0003 prime router)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T22:32:03Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-10-01T22:45:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1587
- **Merged:** 2026-10-01T22:45:36Z
- **Merge commit:** 70d8a223c2a304fff1de7df513d6340f9b1eb0b0
- **Base / Head:** main / UOR-Foundation:codex/prime-route-attention
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1512, #1552.

**What it adds.** The owner chose (1 October) to build the prime-router attention of ADR-0003 inside the model. This PR puts it in the pointer-copy head as an exact route. It is offered alongside the learned scores, not in place of them.

**How the route works.**
- Each token has its registered prime: the relation store's `PrimeRegistry`, where token t maps to the (t+1)-th prime.
- A position's key is the product of the distinct primes of its last W tokens, W = 1–6.
- At position t, a source j is admitted when the key ending at j−1 lies wholly before the query's own tokens and shares a factor with the query's key (`gcd > 1`). The pointer then copies the token at j: the value that followed the matching key.
- An admitted source is scored by `ln gcd` plus a recency term below `ln 2`, then softmaxed (sharpness 4). A shared rare atom (large prime) outranks a shared common one, and recency orders sources with the same shared atoms. With no admitted source the attention is zero, so the row is the generated distribution alone.
- The gate stays learned. The route has no parameters, so the query and key get no gradient.

**Interfaces.**
- `PointerConfig.route: Option<PrimeRoute>`. It is serialized only when set, so earlier configurations are unchanged.
- `StackModel::set_pointer_route`.
- `geometric-stack dialogue-train pointer_route=none|prime:W`, for new heads and `init=` heads.
- `m-world evaluate | evaluate-cells | probe pointer_route=…`, applied post hoc to saved weights.
- A route excludes `pointer_select=`, and the refusal is explicit.

**Checks run locally** (`a62fc993`, plus the merge of main at `9d5c0761`).
- `cargo fmt --check` and clippy on the new code: clean (the lib's existing warnings are unchanged).
- New library tests:
  - the route copies the value after the matching key;
  - with no shared atom, or only an overlapping key, it attends nowhere;
  - rarer shared atoms outrank later common ones, and the recency ratio is exact;
  - validation, and exclusion of a selection;
  - a configuration round trip;
  - the routed loss, the 24 scored rows and `next_scores` match an f64 reference of the mixture;
  - the gate's gradient matches finite differences, and the query and key get no gradient and no effect;
  - clearing the route restores the soft loss bit for bit.
- Existing pointer tests: 34 library and 8 CLI (`geometric-stack`) tests pass, plus the `m-world` selection parse test.
- GitHub evidence: branch `codex/ci/prime-route` (workflow v3, head plus one commit) runs fmt, check of all targets, `uor-r4-training` lib tests and the two example test targets.

**Not claimed.** No language or recall result yet. R0 is queued behind the internal model slot: emit-1 re-routed post hoc at W = 1–4 against the same binary's soft pointer on M-world v2 (`recall=off`). It decides whether, and with which W, a routed fine-tune follows. Copy is 0/33 in every current arm, so it is the first target. The pointer has no D11 served form (unchanged): `qat=true` and `export` still refuse a pointer model.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5942173359 — 2026-10-01T22:44:42Z — Casey-allard

**Exact-head review: `9d5c07615b65a1c7f74fb5f3e209c2931322f7e5`** (self-review, under the owner rule of 30 September on #820).

**Scope checked.**
- **The route arithmetic:**
  - `token_prime` reuses `PrimeRegistry`: token t maps to the (t+1)-th prime; ids ≥ 65,536 are not atoms.
  - Keys are products of distinct primes. With W ≤ 6 and primes < 2^20, a key stays below 2^120 and fits a u128.
  - A source's key must end before the query's first token, which excludes self-overlap.
  - Scores are ln gcd plus recency below ln 2, then a softmax. No admitted source means zero attention, so the row is the generated distribution alone.
- **Gradient path:** the exact route forms no score gradient, so query, key and beta get nothing; the gate and logits gradients are unchanged.
- **Inference:** `next_scores` and `score_targets` use the same attention as the loss.
- **Configuration:**
  - `route` is serialized only when set, so earlier configurations parse unchanged.
  - A route and a pointer selection are mutually exclusive, enforced both in `validate` and in the two setters.
  - `init=` applies a route to a saved head without changing weights, and a resume carries it through the lineage configuration.
- **Unchanged:** D11, export and qat still refuse a pointer model. Nothing changes for models without a route.

**Evidence at this head.** Run https://github.com/UOR-Foundation/uor-r4/actions/runs/36935608640, on evidence commit `d7c2e98f` whose parent is this head:
- `cargo fmt --check`: success.
- `cargo check --workspace --all-targets` (workbench excluded): success.
- `uor-r4-training` lib: 479 passed, 0 failed, 6 ignored.
- Example `geometric-stack`: 9 passed.
- Example `m-world`: 5 passed.

**Not claimed.** No language or recall result. R0, R1 and R2 are queued on #1512. The failing `Validate delivery evidence` / `mission-delivery-gate` check is the advisory gate (owner rule, #820 comment 5912382942).

Verdict: approve for merge at this head.


### comment 5942701596 — 2026-10-01T23:33:14Z — Casey-allard

**Merged** as `70d8a223` through the queue at head `9d5c0761`. Main's tree at `70d8a223` is identical to the PR head (`git diff 9d5c0761 70d8a223` is empty). The first measurements are in: on emit-1's recipe the exact route alone is worse than the learned pointer (details on #1512). The route stays opt-in (`route: None` by default), so no existing model changes.


---
_Comment/review ids in this file: 5942173359, 5942701596_

