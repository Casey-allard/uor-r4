# #2228 — The address primitive and its null-control permutation (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T01:57:45Z
- **Updated:** 2026-10-12T02:22:07Z
- **Closed:** 2026-10-12T02:03:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2228
- **Merged:** 2026-10-12T02:03:44Z
- **Merge commit:** 6e64a8734d9d77e2f313e30d81658015f15a77b6
- **Base / Head:** main / UOR-Foundation:deepseek/address-primitive
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**First slice of the preparation for the [invariant-addressed read](docs/mechanisms/invariant-addressed-read.md)** — the deterministic half, landed and tested before any router touches the model.

| | |
|---|---|
| `position_address(token, back, buckets, space)` | **O(1), parameter-free** address: the token identity scattered by a fixed prime, combined with the position's **family key** (its distance bucket). One token at different ages is a different address; every occurrence at the same age agrees **exactly** — which is what makes an address invariable, comparable and permutable |
| `permute_address` + `permutation_from_seed` | the **null control's** fixed bijection: affine with an odd step, hence coprime to a power-of-two space, **recorded rather than shuffled** |
| `route(query, addresses, k, out)` | exact matches first, then leading-bit near neighbours, ties to the earlier position, at most `k` indices |

**Four focused tests pass** (`cargo test -p uor-r4-training --lib stack_address`): determinism and range; ages separate and equal ages agree; the null map is a **bijection** over the whole space (injective *and* surjective — asserted, not assumed); routing takes exact matches in order and honours `k = 0`; and the null map **moves more than nine tenths of the space**, without which the null arm would silently be the same arm as the real one.

**Nothing in the model changes** — this module is not yet called by the read, so the "off state is today's model bit for bit" requirement is met trivially at this slice. The router, its configuration gate and the bit-exact off-state test are the next slice, before any compute.

Line: invariant-addressed read · count 0/3 · headline **unchanged** (preparation; the nine-fit experiment stays pre-registered and unstarted)

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

