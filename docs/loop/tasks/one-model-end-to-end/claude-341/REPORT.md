# REPORT: one-model-end-to-end, claude-341

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Code head `bcfad349e87d552bdff5bfb384fb91ad51242047`, on base `d4154488fcd41c3541ccf0eaa2b92caaeddd84c8`.

**In one line:** the red test held an out-of-date claim, not a scorer fault, so it was rewritten to pin what is now true: both engines read the ask heads and decode the asking fixture alike.

## What did not work, first

- **This test was red on `main` and nothing noticed.** The instrument crate has its own `[workspace]`, so no required check runs it. #3898 merged with this test failing and the crate still has no required check. This unit does not add one.
- **Engine agreement is shown on one synthetic fixture only:** four prompts that decode, up to 12 tokens each, plus one past the context that stops at once, on one artifact with an ask on both Hamming reads and a 3-row carry on the readout. No real trained artifact was decoded through both engines here. DeepSeek's own parity tests (deepseek-261) are the wider evidence.
- **The refusal is now a choice with no measured reason behind it.** See "Decision" below.
- **The first negative check of the refusal text proved nothing.** My edit broke the build, so no test ran. It was discarded and redone; the result below is from the run that compiled.
- **The bench was not read.** `local/bench/INBOX.md` does not exist in the main checkout.

## Classification

**Instrument, not mechanism.** No model, no scored field and no bar is involved. The failing assertion was a claim about the CPU port that #3898 made false on purpose.

## Why the test changed

The specified behaviour really did change. The evidence, in the order the ORDER asked:

1. **#3898 meant to do this.** Its ORDER's goal is that CPU "training, evaluation and audit paths no longer silently omit the asking model's query". The old assertion, `cpu(&asking) == cpu(&plain)`, says the port does omit it.
2. **The engines agree.** On the fixture, `CpuFbsForward` and `IntegerStackSession::serve_step` give the same tokens and stops, on both the plain and the asking artifact. So there is no fault in the CPU crate for DeepSeek.
3. **The scorer did not start reading asks silently.** `refuse_engine` runs before the forward is built, and it still refuses `--engine cpu` on an asking artifact. That covers the teacher-forced and paired-first ranks too, since they need the CPU engine. So `--engine cpu` and the rank modes score nothing they did not score before.

The only part of the scorer that was wrong is its text. The header comment and the refusal message both said the port does not read asks.

## What changed

All in `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`:

- `only_the_integer_session_reads_the_ask_heads` becomes `both_engines_read_the_ask_heads`. It asserts the two engines match on the plain artifact, match on the asking artifact, and that the asking artifact decodes differently from the plain one, so neither engine can pass by ignoring the asks.
- The refusal message now says the panel scores such an artifact only on the product path. The header comment and `refuse_engine`'s doc comment say the same.
- `each_engine_refuses_what_it_would_score_silently_wrong` is renamed `each_engine_refuses_what_it_does_not_score`, since the ask case is no longer silent mis-scoring. It also now checks the new message.

## Decision

**The `--engine cpu` refusal on asking artifacts stays.** Removing it would let the default engine and the rank modes score asking artifacts, a path nobody has run or reviewed. That is a change to what the scorer does, which this repair does not need. It is easy to reverse: delete the check and its test line. Whoever wants CPU ranks on an asking artifact should take that as its own unit.

## Measurements

Host, `--offline --locked --features cpu-parity`:

| Check | Tree | Result |
|---|---|---|
| Full `served-reply-panel` suite, before | `d4154488f` | 27 passed, 1 failed, at `served-reply-panel.rs:1612` |
| Full suite, after | this head's code | **28 passed, 0 failed** |
| `both_engines_read_the_ask_heads` | CPU crate from `a8e51c0f4`, the parent of #3898 | **fails** at the asking-artifact parity assertion |
| `both_engines_read_the_ask_heads` | current CPU crate with only the readout carry switched off | **fails** at the asking-artifact parity assertion |
| `each_engine_refuses_what_it_does_not_score` | old refusal text put back | **fails**, printing the old message |

All three failures compiled and ran the named test. The two CPU-crate runs used throwaway detached checkouts under `local/`, since removed; no tracked file in `crates/uor-r4-cpu-train` was edited. After the restore the source was byte-identical and the suite was rebuilt: 28 passed.

`rustfmt --check` is clean. Clippy on the binary and its tests reports nothing in this file.

## Checks run at the head

- The table above.
- **The in-unit fresh review** is the counted record on the PR.

## Skills used

- None beyond the loop itself: the ORDER, the host tests with the negative checks and the fresh exact-head review.

## Still open

- **The crate still has no required check,** so the next change to `CpuFbsForward` can turn this suite red unseen in the same way.
- **claude-338's REPORT** names the old test and its old claim. It stays as written, as a record of its time.

Closes nothing: #3407 stays open.

## Plain summary

One test in the grading program failed because it was out of date. It said only the main serving engine can read a model's question-asking tables. Earlier today another lab taught the CPU engine to read them too, on purpose, so the test's claim stopped being true.

The two engines now produce identical output on the test model, and the test checks that instead. The CPU engine is still not allowed to grade models that use those tables. Its error message used to give the old reason, and now gives the real one. No grade changes.

## Records

- Notebook: one row in `docs/research/notebook/float-training.md`.
- No GPU, no pod and no artifact were used. claude-339's run and its pinned scorer were not touched.
