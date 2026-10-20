# #1938 — Mask reconstruction proven bit-identical; no source can teach short replies (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:23:50Z
- **Updated:** 2026-10-09T13:21:09Z
- **Closed:** 2026-10-08T17:26:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1938
- **Merged:** 2026-10-08T17:26:05Z
- **Merge commit:** c52a4e5de2618e89a8567843714edebf966a7264
- **Base / Head:** main / UOR-Foundation:deepseek/mask-identical
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The reconstructed mask is byte-identical to the builder's; no source can teach short replies

Two results, both exact.

FIRST: the response mask reconstructed by scanning the corpus for its literal role markers is
PROVEN BIT-IDENTICAL to the builder's own record. The manifest's mask_sha256
(28b7e98bb8a85368596dd7d1b8aa8aa16a277879a8368925dadbd37f20c3b3c1) equals the hash of the
mask emitted this session, at 82,248,461 bytes, and the manifest's mask_rule states the same
rule that was implemented. So rounds 50-55 achieved their goal without a rebuild, a re-fetch
or a licence decision, and the result is now proven against the builder's own hash rather than
argued. It also validates the two mask rules the validator rejected before the third passed.

SECOND: every source in the corpus is prose-dominant. Per-source response medians are 701, 174,
225, 103, 438 and 45 tokens, and no source reaches 0.3 percent of responses at ten tokens or
fewer. The shortest, everyday-conversations, has a median of 45 tokens. So no re-weighting or
re-selection of the existing sources can produce an answer-form tail either - the constraint is
the data, not the mixture.

That leaves two routes, both far cheaper than the 1.39B rebuild rounds 36-48 correctly argued
against: synthetic short-form instruction data derived from the corpus the project already
holds and may already use, with the now-proven mask marking the short answer; or a new
licence-clean short-form source, which is an owner decision. The evidence supports trying the
first.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

