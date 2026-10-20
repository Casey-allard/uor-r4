# #751 — byte-BPE prompt encoder assumes GPT2 Ġ-remap convention; breaks for tokenizer.bin exports using literal raw bytes

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-17T01:54:16Z
- **Updated:** 2026-08-17T08:41:54Z
- **Closed:** 2026-08-17T08:41:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/751
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Found while investigating #745 (root-causing the degenerate small-vocabulary generation on the best-available compiled bundle). Not #745's root cause (see that issue's latest comment for why), but a real, independently-worth-fixing bug found and empirically verified along the way.

# Problem

`uor_r4_core::transformerless::scenarios::Tokenizer::encode_into` (`crates/uor-r4-core/src/transformerless/scenarios.rs`) detects a byte-BPE-family vocabulary (`vocab.len() > 32000 || vocab[1] == "<|im_start|>" || vocab[1] == "\xC4\xA0"`) and, when detected, unconditionally rewrites the input text before matching against the vocab map: every literal space becomes the GPT2 byte-level remap character `'Ġ'` (UTF-8 bytes `0xC4 0xA0`), `\n`→`'Ċ'`, `\r`→`'Ĉ'`, `\t`→`'ĉ'`. This assumes the loaded vocab's own byte content was built using that same GPT2 remap convention (i.e., a vocab entry for the word " how" is literally stored as the bytes for `"Ġhow"`).

That assumption is false for at least one real local tokenizer.bin: `.uor-models/compiled/smollm2-1-7b-instruct/tokenizer.bin` (49152 entries, confirmed via direct inspection) stores **literal raw bytes** — a real `0x20` space, not the two-byte `Ġ` remap. Zero vocab entries contain the byte sequence `0xC4 0xA0` anywhere; the longest vocab entries (e.g. id 40672, `" ================================...\"`) start with a literal space byte.

Consequence: for this tokenizer, `encode_into` can never match any merged word-initial token (there are none in the Ġ-remapped byte space it searches), and falls back to splitting the (wrongly-inserted) `Ġ` character into two spurious single-byte tokens at every space boundary. Confirmed via a probe tool:

```
"Hello, how are you?" encodes to:
  [19556, 28, 143, 250, 6778, 143, 250, 522, 143, 250, 8320, 47]  (12 tokens, unpatched)
```
where id 143 = raw byte 0xC4, id 250 = raw byte 0xA0 — i.e. every space becomes two junk tokens instead of being absorbed into (or matched against) a real merged token. `decode()` happens to round-trip this correctly by accident (concatenating raw bytes 0xC4 0xA0 reconstitutes the same 'Ġ' character, which `decode()`'s `.replace('Ġ', " ")` then turns back into a space) — which is why a naive encode-then-decode round-trip test does **not** catch this bug. It only surfaces when inspecting the actual token ids/counts, or comparing against what a real BPE tokenizer would produce for the same text.

# Fix implemented (verified, not yet landed — see linked PR)

Detect which convention the loaded vocab actually uses before applying the GPT2 remap: scan for at least one vocab entry starting with the two-byte `Ġ` sequence (`0xC4 0xA0`). If none exists, skip the space/`\n`/`\r`/`\t` substitution and match the literal raw bytes directly.

Verified with the same probe: after the fix, `"Hello, how are you?"` encodes to `[19556, 28, 638, 359, 346, 47]` — 6 tokens, using real merged word tokens (e.g. id 638 = `" how"`, id 359 = `" are"`, id 346 = `" you"`) instead of injecting junk byte pairs at every space. Round-trip still exact.

# Scope

- Land the auto-detection fix in `scenarios::Tokenizer::encode_into` (cache the detected convention at parse time rather than rescanning the vocab on every `encode_into` call, for efficiency).
- Add a regression test using a synthetic literal-byte-convention vocab fixture (so this doesn't require a real multi-hundred-KB tokenizer.bin fixture) asserting that a word appearing after a space returns the same token id as the same word at the start of the string, and that spaces don't produce dead single-byte-pair tokens when a real merge exists.
- Note in a code comment (already partially done in the experimental patch) why round-trip testing alone does not catch this class of bug, so it isn't reintroduced with only a round-trip regression test.
- Do not change the existing GPT2-remap behavior for tokenizer.bin files that genuinely use that convention (must remain a no-op for those).

# Non-goals

- This is not #745's root cause; #745's word-salad output was empirically unaffected by this fix (predictions still resolve to the same tokens, since the compiled tables were built/queried on a per-token-id basis regardless of tokenization efficiency — see #745's latest comment for the full trace). Do not close #745 by pointing at this fix.
- Does not address the R4G1-path testing gap tracked in #750.

# Acceptance criteria

- [ ] `encode_into` auto-detects the vocab's actual byte convention (GPT2-remap vs literal-byte) instead of assuming GPT2-remap whenever `is_bpe` is true.
- [ ] A regression test using a synthetic vocab fixture demonstrates correct merged-token encoding under the literal-byte convention.
- [ ] Existing GPT2-remap-convention tests/bundles are unaffected (no behavior change for vocabs that do use the remap).
- [ ] Workspace fmt/clippy/tests pass.

# Verification

- Re-run the probe against `.uor-models/compiled/smollm2-1-7b-instruct/tokenizer.bin` and confirm token counts drop and no spurious 0xC4/0xA0-byte-pair tokens appear at word boundaries for ordinary prompts.

## Comments (2)

### comment 5310932696 — 2026-08-17T01:59:02Z — Casey-allard

Starting on this now. Implementing the fix as scoped: cache the detected byte-convention (GPT2 Ġ-remap vs literal raw bytes) at \`Tokenizer\` construction time rather than rescanning the vocab on every \`encode_into\` call, add a synthetic-fixture regression test, and a code comment on why round-trip-only testing misses this class of bug.

Working in \`.worktrees/i751-tokenizer-byte-convention\` off \`main@20001570\` (post-#749).

### comment 5313784347 — 2026-08-17T08:41:53Z — Casey-allard

Closing the loop here -- the fix landed in PR #754 (merged) but that PR
used "References #751" rather than "Closes #751" (deliberately, since it
was written as a standalone, independently-useful fix rather than framed
as resolving this specific tracking issue), so it didn't auto-close.
Confirming against this issue's own acceptance criteria:

- [x] `encode_into` auto-detects the vocab's actual byte convention
      (scans for the 2-byte `Ġ` sequence at parse time, caches the result
      on `Tokenizer`, consulted before applying the remap) instead of
      assuming GPT2-remap whenever `is_bpe` is true.
- [x] Regression test using a synthetic literal-byte-convention vocab
      fixture demonstrates correct merged-token encoding.
- [x] A second regression test using a synthetic GPT2-remap-convention
      vocab confirms the remap still applies where it should -- no
      behavior change for tokenizer.bin files that genuinely use that
      convention.
- [x] `cargo fmt --check`, `cargo clippy --workspace --all-targets
      --all-features --offline -- -D warnings`, `cargo run -p xtask --
      audit-limits`, `cargo test --workspace --offline` all clean (PR
      #754's own verification section).
- [x] Real-bundle probe re-run against
      `.uor-models/compiled/smollm2-1-7b-instruct/tokenizer.bin`, done
      before the PR was written (documented in this issue's own prior
      comment): `"Hello, how are you?"` went from 12 tokens (junk
      `0xC4`/`0xA0` byte-pairs at every word boundary) to 6, using real
      merged word tokens.

All acceptance criteria satisfied. Closing.



---
_Comment/review ids in this file: 5310932696, 5313784347_

