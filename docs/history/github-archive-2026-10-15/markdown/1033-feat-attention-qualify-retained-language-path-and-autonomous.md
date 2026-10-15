# #1033 — feat(attention): qualify retained language path and autonomous decoding

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-01T10:34:36Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-09-01T10:35:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1033
- **Merged:** 2026-09-01T10:35:50Z
- **Merge commit:** 0e098248f867fee21ce1cdadf982f46d602d59b0
- **Base / Head:** main / UOR-Foundation:issue-973-language-path-generalization
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Delivers #973's independently frozen `R4RetainedLanguagePathV1` campaign and its predeclared retained-only autonomous-generation successor.

The matched language-path result is terminal `RETAINED_LANGUAGE_PATH_PASS`:

- retained NLL `8.326806644 -> 3.899861931`;
- ordinary causal-softmax control NLL `8.328084567 -> 3.903394426`;
- retained state-off NLL `4.234849487`, a `0.334987556`-nat penalty;
- state-off loses 16,660 correct decisions across 247,920 validation positions;
- retained versus ordinary is `-0.003532495` nats and `-0.073814` percentage points; and
- zero forbidden reads, exact artifact reload, deterministic replay, and direct/stationary retained parity pass.

This establishes a generalizing, causally load-bearing geometric retained-attention language path competitive with the matched ordinary control. H4-specific superiority remains `NOT_EVALUATED`.

## Autonomous local decoding

Adds the frozen `generate-language-path` command and exact result validation. The create-once retained-only smoke is terminal `AUTONOMOUS_GENERATION_SMOKE_COMPLETE`:

- `5/5` prompts emit 64 local tokens with valid raw ByteLevel UTF-8;
- `5/5` fresh-artifact/fresh-zero-state replays are exact across token IDs, stops, decoded bytes/text, audit CIDs, and logits-trace CIDs;
- zero forbidden, future, source-data, target, teacher, or provider reads/calls; and
- zero training or optimizer steps.

Result CID: `blake3:723f37764a4785aa7b2aa40864621865f0b7f9ccaf30d43206bf7b12f11ce2fa`.

The exact 51,865-byte executed runner is archived under CID `blake3:522862195c01c83ee59dcd849db7824b3345c20bcbce170ce86e89a23b57cf64`. Validator-only hardening landed after execution; the immutable result was never modified or rerun. The current verifier requires the historical source archive and independently rehashes all common model dependencies.

## Direction and claim boundary

All authoritative repository entry points, roadmap/completion-plan/research records, programme root #820, #973, and milestone 21 now agree: do not reopen attention-existence testing. Preserve this cell and move #973 to one independently frozen, data-supported representation/capacity successor with a prompt-conditioned behavior criterion.

The five outputs are TinyStories-like but all drift from their prompt subjects or scenes and contain grammatical errors. This does not establish prompt-responsive coherence, reasoning, H4 superiority, exact/table-native runtime, browser readiness, or release readiness. #973 remains open and continues to block #954.

## Focused verification

- `47/47` language-path model/data/campaign/generation tests
- full existing-result revalidation through the public CLI
- exact byte comparison for six raw evidence/source archives
- `cargo fmt --check`
- `python3 scripts/check_claim_wording.py`
- `git diff --check`
- independent adversarial generator and evidence audits: GO

The expensive workspace/wasm/audit/fuzz integration ladder is intentionally left to the protected merge queue.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

