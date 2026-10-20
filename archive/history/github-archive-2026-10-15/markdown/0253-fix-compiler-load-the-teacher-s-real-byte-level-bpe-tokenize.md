# #253 — fix(compiler): load the teacher's real byte-level BPE tokenizer for HF-path observation (#242)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-28T04:52:37Z
- **Updated:** 2026-10-09T13:09:11Z
- **Closed:** 2026-07-28T05:03:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/253
- **Merged:** 2026-07-28T05:03:23Z
- **Merge commit:** c97e82c66579492511d8e1d618cfc33a8c8be2c7
- **Base / Head:** main / UOR-Foundation:issue-242-hf-bpe-tokenizer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #242.

## What

Replaces the bespoke tokenizer on the **HF observation path** with a correct byte-level BPE implementation loaded from the model's own `tokenizer.json`:

- New `hf_bpe.rs` (`uor-r4-core/transformerless`): vocab + ordered merges (string and array formats), `added_tokens` matched atomically leftmost-longest before pre-tokenization, GPT-2 `bytes_to_unicode` byte-level mapping, `add_prefix_space` per HF `ByteLevel` semantics, and merge-**rank**-ordered BPE (the bespoke tokenizer merged by lowest merged-token-*id* — a different algorithm producing segmentations the teacher never saw). SmolLM2's actual pre-tokenizer is `Sequence[Digits{individual_digits}, ByteLevel]` (verified against the real published `tokenizer.json`) — both steps implemented; unknown pre-tokenizer steps error loudly instead of being silently approximated.
- `TokenizerKind { Legacy, HfBpe }` wrapper introduced at the HF call sites only: `observe_text_command` (`graph-cli/src/lib.rs:317-334`) selects `HfBpeTokenizer::from_dir(source)` when `tokenizer.json` exists (the same snapshot dir the oracle reads `config.json`/`model.safetensors` from), legacy fallback otherwise. **The legacy llama2.c path is untouched** — κ-pinned baselines unaffected.
- Real tokenizer provenance: `address()` = `blake3:<hex>` of the tokenizer.json bytes, replacing placeholder-string territory.
- 9 unit tests, including the decisive one: a constructed case where lowest-token-id greedy segments **differently** from rank-ordered BPE, asserting the correct result.

## Not in this PR (flagged in-code)

Serving-side teacher prompting (`generate_attention_text`) still uses `tless_tokenize` — fixing it requires threading `TokenizerKind` through the no-alloc `tless_tokenize/detokenize` surface (~10 call sites). `// TODO(#242 follow-up)` markers at `src/server.rs:712` and `src/tless_uor.rs:542`; follow-up issue to be filed on merge.

## Measurement follow-through

After merge: recompile the HF artifact and post before/after `evaluate-report` numbers on #242 — the decisive metric is teacher argmax-vs-next on held-out ≫ chance (vs the ≈uniform 14.84 bits floor that motivated this issue).

## Gates

All four green locally (macOS arm64, pinned 1.97.1, offline); 9/9 new tokenizer tests pass. Compiler/observation-side only — P-4 kernel scan unaffected.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

