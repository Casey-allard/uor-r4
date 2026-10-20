# #1669 — Serve the pointer-copy head in both integer engines (chat models export; pointer schema /2)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T04:01:40Z
- **Updated:** 2026-10-09T13:08:44Z
- **Closed:** 2026-10-04T04:06:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1669
- **Merged:** 2026-10-04T04:06:07Z
- **Merge commit:** cacb54c41078c24493166c752302257dbf03e663
- **Base / Head:** main / UOR-Foundation:codex/d11-pointer
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Every chat fine-tune (`dialogue-train pointer=32`) carries the pointer-copy head, and `stack_export` refused it ("no D11 port yet"). After #1667 (integer L2 read), this was the last blocker to serving the ladder's chat models in integer form.

## What it does
**Export.** The pointer query, key and gate maps are written (4-bit, after the final norm), plus the gate bias and, for a Lorentz pointer, beta. Selected or routed pointers, and QAT, are still refused.

**Engines.** Both D10 (`uor-r4-lut`) and multiplier-free D11 (`uor-r4-integer`) serve the mixture, bit-identically. Per position:
- the pointer key is cached;
- Dot or Lorentz scores are computed over sources 0..=t;
- two Q31 softmaxes come from the sealed exp table: the copy mass per input id, and the generate distribution;
- the gate goes through the sealed sigmoid;
- the Q30 mixture is p_v = (u_v·G + m_v·K)·2⁻⁶³, with G and K the gate-weighted reciprocals.

D11 does this in `#[inline(never)] stack_pointer` and `stack_pointer_mixture`, using long division and multiples-table products. Both kernels are added to the audit's mandatory symbols.

**Serving surface.**
- `step` still returns logits. The mixture is available through `mixture()` and `next_token_scores()`.
- `uor-r4-stack generate`, `s2-behavior-codec` and greedy decoding argmax the mixture, and `digest` hashes it.
- `d11-evaluate` compares mixtures as integers and scores NLL from them. A zero probability is floored at 2⁻³⁰.
- Modes that read raw logits refuse pointer artifacts: `lut-evaluate`, the samplers and the grid references.
- D11 session save and restore carry the pointer state.

**Schema.** Pointer artifacts are written as `uor-r4.lut-stack/2`; plain artifacts stay `/1`, byte-identical. Both engines tie the schema to whether a pointer is present, and refuse either schema under the other's shape. An engine built before this port ignored the unknown `pointer` field and would have served the plain distribution; it now refuses a `/2` artifact.

## Evidence at `b451e55a`
Run on macOS, current main plus these two commits, with `UOR_BUILD_SOURCE_COMMIT` set.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo test -p uor-r4-lut --lib` | 25 passed |
| `cargo test -p uor-r4-integer --lib` | 204 passed |
| `cargo test -p uor-r4-training --lib stack_export` | 16 passed |
| `cargo test --release -p uor-r4-training --test stack_d11_oracle` | 11 passed |
| `cargo test -p uor-r4-training --example geometric-stack pointer` | 4 passed |
| `cargo check --workspace --all-targets --exclude uor-r4-workbench` | ok |

New tests cover:
- D10 and D11 mixtures identical on 6 random pointer configurations (Dot and Lorentz pointers, dim 1–32, with Dot, Lorentz and L2 reads), including ones amplified up to 65,536×;
- schema binding in both directions, plus format refusals;
- the D11 snapshot round-trip;
- the kernel against its plain definition;
- parity of the integer mixture with the float model (greedy agreement and probability gap bounds stated in the tests).

**Multiplier-free audit** (`audit_zero_matmul_serving.py --stack` on the release `uor-r4-stack`): FULL PASS, 0 forbidden instructions across 36 symbol ranges and 39 reachable functions. While the port was being written, the audit caught a vectorized maximum that compiled to `smaxv`/`fmov`; it is fixed with per-element `black_box`.

**The real chat model.** chat-29m-B-lr5e-4 (`read=l2`, pointer=32, the best 29M chat fine-tune; #820 issuecomment-5974333259), exported from the stack's parent `1094dd9f` with code identical to this PR's:
- **Artifact:** 16,871,364 bytes, `uor-r4.lut-stack/2`.
- **`d11-evaluate`** (TinyStories validation, 8 windows, 3,072 targets): D11 and D10 mixtures are **bit-identical**, with max difference 0 and 0 differing positions. The NLL from the mixture is 2.7508 in both.
- **Speed on the M1:** D10 at 4 threads 194 tok/s; D11 at 1 thread 42 tok/s.
- **Float reference** (`evaluate`, consecutive 256-blocks, full validation set): 2.6526. That puts the integer cost at roughly +0.1 nats. The protocols and windows differ, so this is not a matched gap.
- **The chat fine-tune is off-domain on TinyStories:** the base 29M scores 1.2857.

## Limits
- No matched float-vs-integer gap for pointer models yet, because `lut-evaluate` reads raw logits.
- No integer chat-panel run yet.
- Selected and routed pointers, and QAT, are not ported.

References #820, #964

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5404255512 — 2026-10-04T04:05:16Z — Casey-allard — COMMENTED

**Exact-head review at `b451e55a8a9c34821f88110daf1b259fd36b2a61` (Claude lab; self-review under the owner's 2026-09-30 rule, plus an independent read-only code review).**

**Verified**
- **Float semantics match:** sources `0..=t`; copy weight sigmoid(g+bias), generate weight 2³¹−C; Dot scale `round(2³⁰/√dim)`; Lorentz −β·d with no offset; copy mass keyed by input id; final-norm gain folded into the query, key and gate maps.
- **Fixed-point bounds:** T ≥ 2³¹ ⇒ reciprocal ≤ 2⁶²; products < 2⁹⁵; no wrap on real values; division guarded.
- **D10 = D11** rounding is identical, and the zero floor exists only in evaluation.
- **D11 contract:** `#[inline(never)]`, multiples tables and long division only; audit FULL PASS.
- **Schema:** `/2` is bound to the pointer in both engines; old engines refuse it.
- **Plain models:** `/1` artifacts and logits are unchanged.

**Tests at this head** (listed in the PR body): all passed. **Real chat model:** D10 and D11 mixtures bit-identical over 3,072 positions.

**Follow-ups (non-blocking):**
1. D11's `set_copy_scale` should refuse a pointer model, since it would break D10 = D11.
2. `digest` should hash the mixture under a separate `mixture_sha256`.
3. Share `MAX_POINTER_DIM` between the two engines.

**APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5404255512_

