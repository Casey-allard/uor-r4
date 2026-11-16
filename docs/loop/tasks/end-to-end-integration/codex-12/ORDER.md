# ORDER: end-to-end-integration, codex-12

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-11](../codex-11/ORDER.md). Reviewed head: `2d7014a53`.

## Why this unit exists

codex-11 deferred the read-free overflow policy because the session's `Vec<u32>` makes eviction
`O(context)`. That cost is fine for the read-free `cpu-training-pilot` artifacts (context 192), and
those models currently stop at `ContextFull` for no reason: their reads do not name token positions,
so the recurrence and the native Q31 state carry everything the next step needs.

This unit adds the policy: a full context drops its oldest token **only** on a model whose reads
ignore positions, and refuses the switch on a read-bearing model.

## Claim record

`loop claim end-to-end-integration --lab-topic context-slide` was refused because
`crates/uor-r4-integer/src/stack/**` overlaps claude-384's PR #3983. The unit changes the session, its
policy and a test, adds no dependency and does not change `Cargo.lock`.

## Method

- `IntegerStackModel::can_slide_tokens`: an FBS model with no Hamming read (and so no place read)
  and no turn memory.
- `IntegerStackSession::set_context_slide(bool)`: refuses a read-bearing or dense model.
- The two FBS step paths allow `position == context` when the policy is on and drop the oldest token
  before the push; the dense path still refuses.
- A test: a read-free context-8 trunk steps 16 ids, the window is the last 8, two runs agree, a
  ninth step without the policy refuses, and a Hamming trunk refuses the switch.

## Files in scope

- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/tests/context_slide.rs` (new)
- `docs/loop/tasks/end-to-end-integration/codex-12/**`

Out of scope: a ring buffer, serializing the policy, the dense path, the Q31 read's own history, a
schema, a dependency or a GPU run.

## Done when

1. A read-free model steps past its context with the policy on; the window holds the last `context`
   ids and two runs agree.
2. Without the policy, the same model refuses at the bound.
3. A model with a Hamming read refuses the switch; a dense model is refused structurally by
   `can_slide_tokens` (this crate has no dense artifact to fixture-test it).
4. No existing test is weakened; the library suite passes.
5. `git diff --check`, the harness contract rule, the REPORT check and the exact-head review pass.

## Cancel

Stop if the slide changes any served token on a read-free model compared with an unbounded run, or
if it needs a data-structure change. Do not build the ring here.

STOP and report when done.
