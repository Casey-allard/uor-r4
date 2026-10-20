# #2482 — Independent adversarial review of SpiralCore reconstruction-native attention: REVISE, per-item verdicts (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T18:31:49Z
- **Updated:** 2026-10-14T19:13:00Z
- **Closed:** 2026-10-14T19:11:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2482
- **Merged:** 2026-10-14T19:11:27Z
- **Merge commit:** 618e422ce44d62985935b5dd58fe8e406100d5e3
- **Base / Head:** main / UOR-Foundation:claude/spiralcore-attention-review-20261014
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

Casey asked for an independent adversarial review (a different lab from Codex) before deciding whether to migrate toward reconstruction-native relational attention (#2029 comment 6174925713). This PR delivers that review as a dated record with its evidence.

Reviewer: Claude Fable 5.1 (`claude-fable-5-1`) at max reasoning effort, fresh authenticated Desktop Code-tab session, no credit purchase, provider substitution or access limit.

Target: #2480 at `bbbbdf7fbb28c5c8f6f5cd83ab7aecf48e7d4889` (record, eight specialist reviews, three evidence directories), the first assessment (#2478, `67c57781e`), the six original attachments (all SHA-256 verified against the manifest), and the local migration decision brief. The attention note is treated as a proposal. The discrete crate has no source drift between Codex's snapshot `88c413599` and `ae3aa4dc3`.

## Result

**Verdict on the migration as posed: REVISE — it is four separable items.**

| Item | Verdict |
|---|---|
| Exact compressed-data reads | PROCEED TO PROTOTYPE as a Rust object store on the D19 durable-memory line (the model path has no transfer step; unique content expands 2.17× in records, the benefit is warm/edit reuse) |
| Root packing and the ≤16K Flock index | INSUFFICIENT EVIDENCE for any change (correct derivations; no payoff at C192/B4; context capped at 243; the index is per frame block and payload blocks 1–3 already share one selection) |
| Relational attention | REVISE, then PROCEED TO PROTOTYPE of a bounded exact-relation reranker as a third `ReadSpec` with Gate-kind trits; the energy controller is unconsumed machinery; the source shows neither Route nor Flock can express a copy-successor read, and 3,847 of 14,287 DEV targets are recoverable by one (data statistic, no model) |
| Shared writer | No change; admission (Q1 2,201/14,287) is the measured ceiling and is reported separately in every arm |

Parallel versus replacement: neither. A third read law under the existing dispatch is the whole migration; rollback is `read_spec: route`; nothing proposes replacing transport, binder or writer. Learning-credit correction: the registered schedule proposes read parameters only in the last 20 of 16,404 batches, so a trained-in comparison needs an interleaved schedule on both arms and should wait for seed-1's Route rows at cursor 16,404.

Fresh executions: the three retained byte/data-structure reproductions on the original attachments, byte-identical to Codex's retained outputs; a standalone Rust data statistic on the pinned DEV corpus (pins verified). No tokenizer, model, reference, attention simulation, CUDA, pod, rental, training, active-run change, adoption or hold release. Not FIT. The seed-1 completion executor and Q2 scoring are untouched.

Line: owner-requested independent architecture review · count 0/3 (no model line advanced) · headline unchanged. This PR creates no waiting dependency; the next step is the owner's decision on the revised sequence.

## Validation at this exact head (`eb06478a36ffb931a28955ab53d183397fda92fd`)

- `uor-harness ratchet d23`: OK, 1641 remaining (baseline 1641)
- `uor-harness ratchet d25`: OK, 93 remaining (baseline 93)
- `python3 scripts/check_claim_wording.py`: passed
- `git diff --cached --check`: clean
- `rustfmt --edition 2021 --check evidence/tools/induction_stat.rs`: clean
- `shasum -a 256 -c evidence/SHA256SUMS`: all OK
- `uor-harness shard-check`: 6 shards, 441 receipts, none lost
- Reproductions: exit 0 ×5, outputs byte-identical to retained (timings excluded)
- `904a150eb` merges `origin/main` (#2483) into the branch; the only conflict was the newest-first `current-state.md` ordering, resolved by keeping both entries (this review above #2483's); `eb06478a3` does the same for #2484; no change to any other file
- Fresh-context citation audit (delegate reviewer, read-only) of every `file:line` citation and derived number: 2 line-range mismatches, 1 imprecise citation, 4 arithmetic/label nits and 1 substantive point (a mode-2 Gate apply would be refused by the current Flock-only checks) — all corrected in `01c61d684`; verdicts and numbers unchanged

Files: `docs/labs/spiralcore-attention-independent-review-2026-10-14/` (README, evidence with SHA256SUMS), newest-first `docs/integration/current-state.md` entry, harness claim and skill receipt. STATUS/ROADMAP/tracker headline and the curated README are unchanged because no capability, milestone or active mechanism changed.

References #2029

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

