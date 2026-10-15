# #41 — Add HF `evaluate-report` command for D3 fidelity certificates on compiled SmolLM2 bundles

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-22T00:27:49Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-07-22T05:53:17Z
- **Assignees:** auser, afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/41
- **Merged:** 2026-07-22T05:53:17Z
- **Merge commit:** 2653ebc707355f0db4f08db673d1cac6dfa17ec6
- **Base / Head:** main / UOR-Foundation:copilot/evaluate-hf-compiled-models
- **Reviews / inline review comments:** 1 / 5
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Gate C / Phase 0 lacked an end-to-end command to produce a fidelity certificate for the current default HF-compiled model path (`.uor-models/compiled/smollm2-135m-instruct`). This PR adds that missing workflow by emitting a machine-readable instruction evaluation report over the held-out D3 split with required fidelity metrics and CIDs.

- **New CLI surface: `evaluate-report`**
  - Adds `r4 evaluate-report` in the top-level CLI.
  - Supports explicit paths while defaulting to the current SmolLM2 source/compiled locations.
  - Output defaults to `<compiled>/instruction-eval.json`.

- **Core evaluation + certificate emission**
  - Implements `transformerless evaluate-report` in `crates/uor-r4-core/src/transformerless/command.rs`.
  - Loads:
    - HF teacher from `--source`
    - compiled artifacts/store/corpus from `--compiled`
  - Evaluates held-out D3 (`compiler::train_cut` story-based 80/20 split).
  - Emits report fields for:
    - `top1_accuracy_pct`
    - `teacher_argmax_agreement_pct`
    - `bits_per_token`
    - `teacher_floor_bits_per_token`
    - `bits_over_teacher_floor`
  - Includes source/artifact/store/tokenizer/corpus CIDs and report CID.

- **Documentation update**
  - Replaces README’s “evaluation tooling gap” note with concrete `evaluate-report` usage for the SmolLM2 compiled path and expected report semantics.

- **Focused parser/CLI coverage**
  - Adds command-option parsing tests for `evaluate-report` in core command tests.
  - Adds CLI parse/help coverage for the new subcommand in `src/main.rs` tests.

```bash
cargo run --release -- evaluate-report \
  --source .uor-models/sources/smollm2-135m-instruct \
  --compiled .uor-models/compiled/smollm2-135m-instruct \
  --report .uor-models/compiled/smollm2-135m-instruct/instruction-eval.json
```

## Timeline (6 entries; chronological: conversation comments, reviews, review comments)

### review-comment 3626849496 — 2026-07-22T01:49:24Z — Copilot

_on crates/uor-r4-core/src/transformerless/command.rs:31_

`BehaviorSource` and `TeacherOracle` are imported but not used in this module, which will fail CI if warnings are denied (the repo runs clippy with `-D warnings`). Remove the unused imports.

### review-comment 3626849512 — 2026-07-22T01:49:24Z — Copilot

_on crates/uor-r4-core/src/transformerless/command.rs:239_

`file_cid` reads the entire file into memory (`std::fs::read`). For HF model files (e.g., multi-GB safetensors), this can cause very high memory usage or OOM. Stream the file into a Blake3 hasher instead of buffering the whole file.

### review 4750300659 — 2026-07-22T01:49:25Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

Adds an end-to-end “evaluate HF-compiled bundle and emit a fidelity certificate” workflow to the R4 CLI, targeting the default SmolLM2 compiled bundle path and producing a machine-readable held-out D3 report.

**Changes:**
- Adds a new top-level CLI subcommand `evaluate-report` and argument plumbing in `src/main.rs`, plus CLI parse/help tests.
- Implements `evaluate-report` in the transformerless core command runner, including CID collection and held-out D3 metrics emission.
- Updates README evaluation instructions to use the new command and describe report semantics.

### Reviewed changes

Copilot reviewed 3 out of 3 changed files in this pull request and generated 5 comments.

| File | Description |
| ---- | ----------- |
| src/main.rs | Adds `evaluate-report` CLI surface, forwards args to core, and adds parsing/help tests. |
| README.md | Replaces “tooling gap” note with concrete `evaluate-report` usage and expected report contents. |
| crates/uor-r4-core/src/transformerless/command.rs | Implements core evaluation logic, report JSON schema, CID computation, and command dispatch/help text. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3626849527 — 2026-07-22T01:49:25Z — Copilot

_on crates/uor-r4-core/src/transformerless/command.rs:307_

Avoid duplicating the vocabulary size constant in Witten–Bell backoff. The codebase already defines `compiler::V` for vocab size, so using it here prevents silent drift if the constant changes.

### review-comment 3626849535 — 2026-07-22T01:49:25Z — Copilot

_on crates/uor-r4-core/src/transformerless/command.rs:373_

The teacher oracle is stepped for *all* corpus tokens, but 80% of them are immediately skipped by the held-out check. Since the split is by story id, you can skip teacher inference entirely for training stories to reduce evaluation runtime substantially.

### review-comment 3626849545 — 2026-07-22T01:49:25Z — Copilot

_on crates/uor-r4-core/src/transformerless/command.rs:454_

The PR description says the machine-readable report includes the report CID, but the JSON written to disk does not contain it (it’s only printed to stdout). If downstream tooling needs the CID inside the report artifact, consider writing an envelope (e.g., `{ report: ..., report_cid_of_report_bytes: ... }`) or clarify the intended semantics in the PR description/docs.


---
_Comment/review ids in this file: 4750300659, 3626849496, 3626849512, 3626849527, 3626849535, 3626849545_

