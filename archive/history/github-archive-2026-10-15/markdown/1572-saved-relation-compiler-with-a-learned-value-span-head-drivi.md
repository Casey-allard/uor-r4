# #1572 — Saved relation compiler with a learned value-span head, driving the grounded session

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T09:29:51Z
- **Updated:** 2026-10-09T13:08:48Z
- **Closed:** 2026-10-01T09:54:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1572
- **Merged:** 2026-10-01T09:54:18Z
- **Merge commit:** e8f3cb1d8e9a38215fcdeaaf561b72c31749296e
- **Base / Head:** main / UOR-Foundation:codex/saved-compiler
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552, #962. This is the D19 integrated deliverable's compiler side: saved learned relation/act/span compiler → existing exact `StackStore` → real `emit-1` emitter, inside Codex's `GroundedSession`, with predicted inputs and generated history. `stack_grounded_session.rs` is not edited; this PR implements its `TurnCompiler` trait.

**Library (`relation_compiler.rs`):**
- **`word_spans`:** the world's `words`, with byte spans into the unchanged source. A test checks equality with `words` and slicing, including curly apostrophes and non-ASCII text.
- **`SpanHead`:** a learned value interval with no-write credit.
  - **Supervision:** template and paraphrase `{v}` slots. Paraphrase examples now keep their pattern as `template`, which changes nothing else.
  - **Representation:** words are seen only through a frame vocabulary (training words never inside a value), so the head cannot memorise values.
  - **Decoding:** the best contiguous run of at most 4 words with positive summed margin.
- **`SavedCompiler`:** one canonical JSON artifact.
  - **Contents:** the parameters stored by f64 bits, labels, vocabularies, settings, tokenizer digest and a training-provenance record (integers and strings only).
  - **Loading:** only canonical bytes load.
  - **Behaviour:** it implements `TurnCompiler`, with relation IDs 1…10 and `encoder: None`.
- **`ActRule`:**
  - `table` (the default, v18) lets the act head name statement or query.
  - `span` (v19) makes a decoded value a statement, with the act head choosing only Assert vs Correct; no value makes a query.
  - Table-rule artifacts carry no `act_rule` field, so v18 bytes still load unchanged.

**CLI (`examples/m-world.rs`):**
- **`compiler-save`:** fits, writes `compiler.json` and scores relation, act, span exact, no-write and exact action on development phrasings. `values=train` (the default) uses training values only; `values=both` is E4's draw.
- **`session`:** the actual `GroundedSession` over M-world development conversations.
  - **Emitter:** the model is saved as a sealed inference checkpoint with an empty store, and data identities come from its sealed training report.
  - **Arms:** `default`, `no_read` and `no_write`, plus save/load continuity.

**Measured (#1552 cards v18–v19; one draw; development phrasings × development values; a diagnostic, not a gate):**
- **Span head:** exact value span 1,493/1,503.
- **v19 session (default arm):**
  - open relation 34/52, closed 13/17, open abstention 7/8;
  - controls: `no_read` 8/52, `no_write` 1/52;
  - 5/5 save/load continuations equal; 0 turn errors.
- **Comparators:**
  - v18 (`act_rule=table`): 23/52 open.
  - The reference-history E4 route: 36/52 open, 2/17 closed.
- **Remaining open misses:** 14 of 18 are relation naming of development wordings.

**Limits:**
- MQAR has no channel in the one-entity session.
- The raw paraphrase union carries audited label errors.
- No ordinary-text CLI yet.
- Not D11-served; no capability claim.

**Checks (local):**
- `cargo fmt`;
- `cargo clippy -p uor-r4-training --lib --example m-world --bin grounded-session -- -A clippy::approx_constant`. The allow is needed because `b3_e8_codecs.rs` on main fails `approx_constant`, which stops Clippy before it reaches the examples. An earlier run without it did not lint the example. The touched code has no new warnings; the one warning, `type_complexity` in `fit_route`, comes from #1570;
- `cargo test -p uor-r4-training --lib relation_compiler`: 9 passed;
- release build and the runs above;
- `git diff --check`; `check_claim_wording.py` passed.

Exact-head GitHub evidence follows.

**Ordinary-text CLI (`src/bin/grounded-session.rs`):**
- **Commands:** `init` (sealed checkpoint + tokenizer + compiler → an empty envelope), `turn` (one turn, JSON outcome), `chat` (stdin lines) and `show`.
- **Process model:** each command loads a sealed envelope in a fresh process, with the compiler taken from the envelope's `compiler.bin`, and saves a new envelope. Loading replays the transcript against the store.
- **Fresh-process witness:** 3 development conversations of `session-2`, one process per turn, match the in-process run on **14/14** turns (reply, action and recall). Roots: `claude-log-sieve/cli-witness-1`.
- **Chat check:** a short session (name, dog, correction, an absent relation) stores, corrects and recalls as expected. Its sentences are close to M-world templates, so this is not general-conversation evidence.

Evidence: `codex/ci/v3/saved-compiler` at `b6e6f4ec` (before the CLI) and `codex/ci/v3/saved-compiler-cli` at the current head `6f137819`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5377715548 — 2026-10-01T09:53:24Z — Casey-allard — COMMENTED

Exact-head review at `6f1378190055b36ef3e08cd78fed1eef780ef216` (self-review under the owner's rule).

**Source read (relation_compiler.rs, examples/m-world.rs, bin/grounded-session.rs, docs):**
- `word_spans` is tested equal to `words`, with char-boundary spans.
- The span head's frame vocabulary excludes every word ever seen inside a value, so value identity cannot be learned.
- **Artifact:**
  - f64 parameters are stored by bits;
  - only canonical bytes load;
  - training provenance holds integers and strings only;
  - table-rule artifacts carry no `act_rule` field, so v18 bytes are unchanged (tested).
- **`TurnCompiler`:** relation IDs are 1-based, with `encoder: None`; under the span rule, value ⇒ statement and none ⇒ query.
- **`m-world session`:** the checkpoint is built from the model's sealed training report (manifest-bound data identities). Arms reuse one set of drawn conversations; reload compares whole `TurnOutcome`s.
- **CLI:** each command loads the envelope with its own `compiler.bin` and saves to a new envelope; failed turns are not saved.
- `stack_grounded_session.rs` (#962, Codex) is not modified.

**Executed:**
- GitHub evidence `codex/ci/v3/saved-compiler-cli` (run 36843468030), commit `5bacb9d1`, parent = this head:
  - fmt success;
  - `cargo check --workspace --all-targets` (workbench excluded) success;
  - `uor-r4-training` lib **427 passed, 0 failed, 5 ignored**;
  - `m-world` example 5 passed.
- **Local:**
  - `relation_compiler` tests 9/9;
  - Clippy with `-A clippy::approx_constant` (pre-existing on main) shows no new warnings in touched code;
  - release runs v18 and v19;
  - fresh-process CLI witness 14/14.

Approve for the queue.

### comment 5929036855 — 2026-10-01T09:54:42Z — Casey-allard

Merged as `e8f3cb1d`. The squash tree `27ab5bc80ccd5bf4dab172786521d790b58d4e9a` equals `git merge-tree --write-tree` of the parent `7050f0d3` and the reviewed head `6f137819`.


---
_Comment/review ids in this file: 5929036855, 5377715548_

