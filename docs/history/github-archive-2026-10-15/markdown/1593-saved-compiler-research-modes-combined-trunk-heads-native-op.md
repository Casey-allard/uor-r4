# #1593 — Saved compiler research modes: combined trunk heads, native op model, Ollama teacher, parallel evaluation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T01:58:23Z
- **Updated:** 2026-10-09T13:08:46Z
- **Closed:** 2026-10-02T03:00:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1593
- **Merged:** 2026-10-02T03:00:56Z
- **Merge commit:** 35482908f9bfd799b53ee150bb8098eb7db90f93
- **Base / Head:** main / UOR-Foundation:codex/native-op-compiler
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552 (D19: grounded conversation and durable memory first).

This delivers the saved-compiler research branches that ran cards v20–v25. They were unmerged until now. Every mode is opt-in; the merged table/span compiler (#1572) is unchanged by default.

**Contents** (3 files against main: `relation_compiler.rs`, `examples/m-world.rs`, `src/bin/teacher-paraphrase.rs`).
- **`relation_mode=combined | combined_relation`.** Dense heads over a bound frozen trunk (`Trunk`, bound by file digests), beside the word table.
  - Measured NEGATIVE for the session, v20/v22: relation naming 0.922, but `Correct` exact only 41/313 and session 26/52 vs v19's 34/52.
  - Retained per D12.
- **`relation_mode=op_model`.** A geometric stack fine-tuned on `compile-corpus` documents (`System: Compile. / User: … / Assistant: Op: assert|update <rel> <value> | query <rel> | none`) generates each turn's memory op. `parse_op` locates the value in the source text.
  - The artifact binds the op model's identity.
  - **New in this PR:** an assert/update whose value does not occur in the turn is read as a query of its relation. The turn holds no value to store.
- **`m-world compile-corpus`** writes those documents. **`compiler-save` and `session`** gain trunk and op-model support and a parallel evaluation that compiles each turn once (verified identical to sequential, counts and artifact hash).
- **`teacher-paraphrase ollama=MODEL`.** A local Ollama backend (`/api/chat`, Metal) as an offline paraphrase teacher; it never serves.
  - About 30 tokens/s on the M1 GPU, against 0.0/s for SmolLM2-1.7B on GitHub runner CPUs (measured: 177 tokens in 4.5 h).
  - The report records the engine, model and digest.

**Results so far** (#1552; M-world v2, development phrasings × development values unless noted):

| Card | Teacher data | Op-model relation | Session relation category | Open relation |
|---|---|---|---|---|
| v19/v21 (table compiler, baseline) | | | | **34/52** |
| v23 | raw | 0.398 | 33/83 | |
| v24 | + teacher batches | 0.694 | 27/83 | 15/52 |
| v25 | + 16 Ollama batches | **0.872** | **41/83** | 24/52 |

**Diagnosis of v25 vs the table compiler**, per action inside the session:

| Action | Op model (v25) | Table compiler |
|---|---|---|
| query | 37/83 | 77/83 |
| assert | 123/166 | 128/166 |
| correct | 39/49 | 40/49 |
| unresolved | 661/777 | 723/777 |

34 of the 46 missed queries were imperative questions ("Remind me what my job is.") generated as assert ops with an invented value. That is what the new parse rule targets.

**Checks run locally** (head `bdd02351`): fmt; `relation_compiler` and `temporal_compiler` tests (17 passed, 1 ignored), including op formatting and parsing, the new valueless-op query rule, op-model binding and the split mode; `m-world` example tests. GitHub evidence: `codex/ci/native-op-compiler` (workflow v3).

**Queued** on the internal slot (#1552): v25b (v25's op model plus the parse rule, paired with v25) and v26 (another 16 Ollama batches, about 5,950 paraphrases, then retrain). Results will be added here. No capability claim beyond the measured counts above.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5944532323 — 2026-10-02T02:33:32Z — Casey-allard

**Exact-head review: `bdd023512dfaa1945e1b97a775557b36a88b1d93`** (self-review, under the owner rule of 30 September on #820).

**Scope checked.**
- Every new mode is opt-in (`relation_mode=combined|combined_relation|op_model`, `teacher-paraphrase ollama=`); the default table/span compiler path of #1572 is unchanged.
- `parse_op` locates statement values in the source text (exact, then ASCII case-insensitive).
- **New rule:** an assert/update whose value is absent becomes `QueryCurrent` of its relation (tested).
- The op-model and trunk identities are bound into the saved artifact by file digests.
- Parallel evaluation was verified identical to sequential, counts and artifact hash.
- The merge of main at `86188622` is clean. The combined tree with #1591 (`ea183faf`) compiles in every target.

**Evidence at this head.** Run https://github.com/UOR-Foundation/uor-r4/actions/runs/36953414096: fmt and the check of all targets succeeded; `uor-r4-training` lib 506 passed, 0 failed; examples pass.

**Measured:** v25b (paired with v25) shows compiler relation 0.872 → 0.959 and session open relation 24 → 30/52. That is still below the table compiler's 34/52; v26 and the emit-2 sessions are queued.

Verdict: approve for merge at this head.


---
_Comment/review ids in this file: 5944532323_

