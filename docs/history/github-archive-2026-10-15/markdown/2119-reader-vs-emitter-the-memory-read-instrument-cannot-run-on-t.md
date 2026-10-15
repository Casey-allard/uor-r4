# #2119 — Reader vs emitter: the memory-read instrument cannot run on this artifact, and the pointer trace can (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T01:07:16Z
- **Updated:** 2026-10-10T03:23:30Z
- **Closed:** 2026-10-10T01:08:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2119
- **Merged:** 2026-10-10T01:08:40Z
- **Merge commit:** da28a91dd884326ddcf273dd42f50598a07822c2
- **Base / Head:** main / UOR-Foundation:numeric-reader-emitter
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The question

For the 13 numeric v5 rows: **does the learned read path SELECT the right token before emission?** Selected-but-not-emitted is the **EMITTER**; never selected is the **READER**.

**CPU only, no pod, no training, no model run, $0.** No v5 re-run.

## Finding 1 — the instrument exists and is public
`Session::memory_read_diagnostic(model, target)` (`memory_training.rs:53`) returns `predicted_token` — exactly "what is selected before emission" — plus `target_routes`, the count of admitted routes carrying the expected token.

## Finding 2 — IT CANNOT RUN ON THE ARTIFACT THAT ANSWERED v5, and the reason is structural
The diagnostic returns `None` unless the session carries a memory operator of schema `/3`, `/4` or `/5`. **The v5-answering artifact carries no memory operator at all** — its whole config is `arch`, `vocab_size`, `width`, `heads`, `mlp_hidden`, `context`, `pattern`, `read`, `rotation`, `seed` and a **`pointer`**, with **no memory reader**.

So the diagnostic is `None` **by construction** on every row — not because the model failed to select anything, but because **there is no memory read to diagnose**. **The split is therefore NOT MEASURABLE on the sealed v5 artifact with this instrument, and this PR reports that rather than a suggestive reading.**

## Finding 3 — the instrument that CAN separate them, and it is cheap
The artifact **does** carry a copy pointer, and the pointer's per-step selection is observable: `StackModel::next_scores_with_pointer` returns the emitted logits **and the selection** per step (`geometric_stack.rs:3544`), `pointer_copy_trace()` (`:3494`) turns the record on, and `greedy_reply_with_copy_stop` already records those steps.

**So: generate the 13 replies with the pointer trace on, record what the pointer selected and what was emitted, and classify each row** — stored digit selected but not emitted → **EMITTER**; never selected → **READER**. Untrained, CPU-only, one run. **Specified here and NOT run** — this PR did not build the trace binary and reports no partial reading in its place.

## Designed to fail loudly
Following the addressability probe's standard (inequalities as well as equalities):
1. **Both classes must be non-empty or the result is INCONCLUSIVE, not confirmatory.**
2. **`mem-040`** — the one numeric row that emits the stored value exactly — is the **positive control**; its trace must show the stored digit selected or the trace is broken.
3. **EMITTER is assigned only when the selected token IS the stored digit and the reply lacks it**, compared as **token ids, not strings**.
4. **The twelve failures must not all land in one class** without the per-step record shown for at least two of them, so a class assignment can be audited.

## The ledger — four readings, three dead, one survivor this piece NARROWS
| reading | status | what killed it |
|---|---|---|
| Token count explains the cluster | **REFUTED** | numbers are **2 tokens against 3.74** for words |
| Minimal pairs / digit order | **REFUTED** | `mem-040`'s distractor `74` is a **transposition** of its `84` and it passes; **0 of 12** name the distractor |
| Value addressability | **REFUTED** | the codec recovers a two-token run as **one ordered value with an interval** |
| **The learned read/emit path** | **OPEN — split** | the read half is not measurable on this artifact; the **pointer-selection-versus-emission half is measurable and unmeasured** |

A fifth dead reading would have been welcome; instead the question is now **runnable rather than hypothetical**.

## Delivery
Record [`docs/labs/numeric-reader-vs-emitter-2026-10-09/README.md`](docs/labs/numeric-reader-vs-emitter-2026-10-09/README.md); newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; bundle on cloud-store: `numeric-reader-vs-emitter-2026-10-09.tar`, 19,456 bytes, md5 `7489ab276668d30da50e8480326abd81`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch.

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged; v5 not re-run. References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

