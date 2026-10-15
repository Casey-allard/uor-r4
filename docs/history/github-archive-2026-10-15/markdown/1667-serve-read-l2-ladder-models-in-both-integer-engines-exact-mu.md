# #1667 — Serve read=l2 ladder models in both integer engines (exact multiplier-free L2 read)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T02:39:46Z
- **Updated:** 2026-10-09T13:08:44Z
- **Closed:** 2026-10-04T02:40:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1667
- **Merged:** 2026-10-04T02:40:55Z
- **Merge commit:** d87de5e6271d6fc3f177e209756693fae9009c1a
- **Base / Head:** main / UOR-Foundation:codex/d11-l2-read
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Every ladder model since 8M (20M, 29M, ~96M) uses `read=l2`, the flat Euclidean read that beat Lorentz in the 8M controls. `stack_export` refused it as "a float ablation control with no integer export or engine", so none of these models could be served in integer form. This PR adds an integer L2 read to both integer engines and to the export.

## Integer definition
- q and k are i32 at exponent −16. The squared distance s = Σ(q_i − k_i)² is computed exactly as a u128 at exponent −32.
- The floor is max(s, 429), the same floor code as Lorentz (≈1e-7·2³²).
- The distance is d = isqrt(s ≪ 16), a u64 at exponent −24.
- The score then follows the existing Lorentz path: `grid_apply(d − offset[h], beta[h])`, negated, shifted by SCORE_EXP − DISTANCE_EXP, plus age.
- No new table. L2 needs no key lifts and no arcosh table.
- **D11:** a new `#[inline(never)] stack_l2_distance` built from `stack_square` and `stack_isqrt`, inside the multiplier-free contract, and added to `STACK_MANDATORY_SYMBOLS`.
- **D10:** the same integer definition.

## Changes
- Formats (`uor-r4-lut`, `uor-r4-integer`): accept `"l2"`, with `l2()` and `scaled()` (Lorentz or L2). beta and offset are required for both. The arcosh table and the lift cache stay Lorentz-only.
- Engines: the L2 branch in each read.
- `stack_export` and `load-candidate-forward-report`: the export writes `"l2"` with beta and offset.
- The QAT `served_plan` still refuses L2. Pointer heads are still refused; that is a separate port.

## Evidence
**Tests at `594b663f`** (macOS, all exit 0):

| Command | Result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo test -p uor-r4-lut --lib` | 23 passed |
| `cargo test -p uor-r4-integer --lib` | 203 passed |
| `cargo test -p uor-r4-training --lib stack_export` | 14 passed |
| `cargo test -p uor-r4-training --test stack_d11_oracle` | 7 passed |
| `cargo check --workspace --all-targets --exclude uor-r4-workbench` | ok |

New tests:
- an exact-kernel test against an integer reference, covering edge cases, i32 limits and 5,000 random vectors;
- `l2_distances_match_f64`;
- L2 rows in the D10↔D11 oracle, for random and amplified stacks;
- the L2 export carries beta and offset and no arcosh table;
- format refusals.

**Multiplier-free audit:** `scripts/audit_zero_matmul_serving.py --stack` on the release `uor-r4-stack` binary gives FULL PASS: 0 multiply, divide or floating-point instructions across 34 symbol ranges and 37 reachable functions, including `stack_l2_distance`.

**A real trained L2 model:** geo-20m (512/8/8 `rrarrarr`, quaternion transport, `read=l2`; float TinyStories validation NLL 1.3882), round-to-nearest export, with no calibration.

| Measure | Value |
|---|---|
| Artifact | `model.lut`, 11,891,396 bytes; float weights are 79,726,120 bytes |
| `lut-evaluate` (D10 vs float, 64 windows, 24,576 targets) | integer NLL 1.4163 vs float 1.3872, **+0.0291 nats**; top-1 agreement 0.927 |
| `d11-evaluate` (D11 vs D10, 8 windows, 3,072 targets) | **bit-identical**: max logit difference 0, 0 positions with a difference, NLL 1.45954 for both |
| Speed (M1 CPU, 4 threads) | D10 ~299 tok/s, D11 ~61 tok/s |

A calibrated (GPTQ) export of the same model is running as follow-up evidence.

## Limits
- Pointer-head models (the chat fine-tunes) still cannot export. That is the next port.
- Not established: J/token energy, or serving speed on larger models.

References #820, #964

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5403959933 — 2026-10-04T02:40:01Z — Casey-allard — COMMENTED

**Exact-head review at `594b663f9417bf05c51607455d0b89275d1f80a7` (Claude lab; self-review under the owner's 2026-09-30 rule).**

**Kernel read:** `stack_l2_distance` sums `stack_square` of i64 differences into a u128 (at most 256·(2³²)² < 2⁷³), floors at 429, shifts left by 16 (< 2⁸⁹) and takes `stack_isqrt` into a u64 (< 2⁴⁵). It is inside the D11 contract and identical to D10's definition.

**Checks at this head** (listed in the PR body): fmt 0; lut 23, integer 203, stack_export 14, D11 oracle 7, all passed; workspace check ok; multiplier-free audit FULL PASS.

**Real-model export** at this head: geo-20m gives +0.0291 nats for integer vs float, and D11 = D10 bit for bit (0 differing logits over 3,072 positions).

**APPROVE**; queueing with `--match-head-commit`.

### comment 5975877554 — 2026-10-04T02:42:51Z — Casey-allard

**Merged as `d87de5e6`.** Verified: the squash patch equals the PR diff.

**Follow-up evidence: calibrated (GPTQ) export** of the same geo-20m model, `calibration=` TinyStories train with 64 windows. `lut-evaluate` on the same 64 TinyStories validation windows (24,576 targets):

| Export | Integer NLL | vs float (1.3872) | Top-1 agreement |
|---|---|---|---|
| Round-to-nearest | 1.4163 | +0.0291 | 0.927 |
| GPTQ | 1.4080 | **+0.0208** | **0.938** |


---
_Comment/review ids in this file: 5975877554, 5403959933_

