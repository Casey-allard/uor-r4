# #2248 — The copy-span oracle: force the pointer's attention onto a supplied span at serving time (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T07:40:49Z
- **Updated:** 2026-10-13T02:49:34Z
- **Closed:** 2026-10-12T07:42:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2248
- **Merged:** 2026-10-12T07:42:01Z
- **Merge commit:** e7ffebf314b39794e739b5587fd9671142f11b3f
- **Base / Head:** main / UOR-Foundation:kimi/copy-oracle-2026-10-12
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Line: evaluation integrity · count 0/3 → 1/3 (preparation PR; names its run) · headline: unchanged (M1 v5 memory 20/40, reply 37/232)

**What this is.** The attention half of the forced-copy oracle (geometry audit §7), pre-registered at #2029 comment 6119323722. The gate half already measured (#2187, attribution record conditions D/E: forcing the gate destroys the panel 23→7/40); this instrument forces the **attention** onto the true value span instead, leaving the head and gate to the model. Source only: no training, no pod, no panel run.

**Files**
- `crates/uor-r4-training/src/geometric_stack.rs` — `set_copy_span_oracle`: serving-time field, never saved, no training path; both serving mixtures (`next_scores_pointer_scores`, `read_span_probe`) replace the pointer's computed attention by the uniform distribution over the span; out-of-window span errors, oracle on a pointerless model refused, `None` is the historical decoder bit for bit. Two new unit tests.
- `crates/uor-r4-training/src/bin/chat-grade.rs` — `reply copy_oracle=span checks=<panel checks>`: per `exact` row, the expected value's last occurrence is located in the row's own history by decoded text and set as the span; every turn's span (or absence + reason) recorded in `replies.json`; checks file bound by sha256. One new span-search test.
- `docs/labs/copy-span-oracle-2026-10-12/README.md` — the record, including the declared artifact status: the pre-registered S2 (`09e9dd3d…`) is not on the laptop and not in the cloud-store index; the fallback's weights are not local; the run binds its artifact by sha256 after a recovery attempt, or the pre-reg is amended to a fresh CPU-trained pointer model.
- `docs/integration/current-state.md` — the entry.

**Checks at exact head (base `c4d682cd0`):** `cargo test -p uor-r4-training --lib` 919 passed / 0 failed (with UOR_BUILD_SOURCE_COMMIT set); `--bin chat-grade` 20 passed; `cargo check --bins` clean; `cargo fmt --check -p uor-r4-training` clean; `check_claim_wording.py` PASS.

**Review:** exact-head self-review (owner rule), scope the four files above.

**Decision test:** the run this prepares answers 'is the copy path's bottleneck addressing, gating, or emission' — its three branches each change the line's next run, so the instrument changes what gets built.

References #2029.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

