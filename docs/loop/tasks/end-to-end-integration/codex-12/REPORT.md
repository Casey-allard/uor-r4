# REPORT: end-to-end-integration, codex-12

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic context-slide` was refused because
  `crates/uor-r4-integer/src/stack/**` overlaps claude-384's PR #3983. The unit changes the session
  and a test, adds no dependency and does not change `Cargo.lock`.
- **The eviction is `O(context)`.** `tokens` is a `Vec<u32>`, so dropping the oldest token moves the
  rest. That is fine for the read-free `cpu-training-pilot` artifacts (context 192), and it is paid
  only at the bound; a long-context read-free model would want a ring, which is a separate
  data-structure unit.
- **The policy is not serialized.** A restored session starts with it off; the token window is
  restored, the switch is not.
- **The dense path still refuses.** Only the two FBS step paths slide; the dense path has
  position-indexed caches whose eviction is not wired. The refusal is structural
  (`can_slide_tokens` is false when `fbs` is `None`) and is not fixture-tested: this crate has no
  dense artifact.
- **The Q31 read's own history still grows.** Sliding the token window does not touch
  `Q31ReadHistory`; a model with that read is allowed the policy because its history is separate,
  but its own bound is not addressed.
- **No served-token comparison to an unbounded run.** A read-free model has no larger-context twin
  with identical weights in the test; the safety is structural (no reader names positions), and the
  test proves completion, window contents, determinism and the refusal.
- No model or quality claim.

## What changed

- `crates/uor-r4-integer/src/stack/session.rs`:
  - `IntegerStackModel::can_slide_tokens` (FBS, no Hamming/place read, no turn memory).
  - `IntegerStackSession::set_context_slide` and the session field.
  - the two FBS step paths accept `position == context` under the policy and drop the oldest token
    before the push; the dense path is unchanged.
- `crates/uor-r4-integer/tests/context_slide.rs` (new): the read-free 16-step test, the refusal
  without the policy, determinism, and the Hamming refusal.

## Measurements

- Read-free context-8 trunk: 16 steps served; the window is the last 8 ids; two runs identical.
- Without the policy: the ninth step returns `ContextFull`.
- Hamming trunk: `can_slide_tokens` false and `set_context_slide(true)` refused.
- Library: **674 passed**, 1 ignored.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --test context_slide`
  — 1 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib` — 674 passed.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib
  --tests` — no warning in the changed files.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: the policy is allowed only where no reader names positions, and the cost and
  unsaved-switch limits are stated rather than hidden.
- `scope-guard`: the session and one test; the dense path, ring and schema are untouched.
- `test-freeze`: a new test; no existing test changed.

## Still open

- A ring-backed token window for long-context read-free models (the `O(context)` eviction).
- Serializing the policy with the session.
- The Q31 read's own history bound.
- The read-free policy on the dense path, if a dense artefact ever needs it.

## Plain summary

A read-free FBS model now runs past its context by dropping the oldest token, which nothing it
computes reads; the window is the last `context` ids, two runs agree, and a model with a Hamming read
refuses the switch. The eviction is `O(context)` and the switch is not saved yet.
