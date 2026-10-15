# #2249 — Why starling learning is a lottery, and a dense bootstrap that makes 3/5 seeds match softmax at 1536 (row 2) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T09:45:34Z
- **Updated:** 2026-10-12T09:46:58Z
- **Closed:** 2026-10-12T09:46:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2249
- **Merged:** 2026-10-12T09:46:39Z
- **Merge commit:** 503a6d0b522c5e2d4ff3b650426b0b273330ba0c
- **Base / Head:** main / UOR-Foundation:claude/starling-bootstrap
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · count 1/3 → 2/3 · headline "softmax at runtime: yes" → yes. References #2032.

**The owner-requested learning-reliability investigation, as one result PR** (four pre-registrations, each FIT before compute). [Record](docs/labs/starling-bootstrap-2026-10-12/README.md).

1. **Step 3c, the helper schedule:** A75 and S72 learn 0/3 each. Helper length is not the lever.
2. **The bootstrap diagnostic** (16 instrumented runs, 512 and 1536):
   - learning waits for a heavy-tailed escape from the chance plateau;
   - escape is predicted by how much the 1/t dense helper concentrates on the true source (P2), which halves from 512 to 1536 (P1);
   - the anneal is not the binding deadline (P3 fails, row D): recall tracks the budget left after escape.
3. **Fix 1, a dense teacher bootstrap:** no seed left at chance (mean 0.693 against 0.264 on the same seeds), but 1/5 learn (row 4).
4. **Fix 1b:**
   - **DBR** (L2-score teacher): the bootstrap escapes on 4/5 seeds, but the handover limits (row 3).
   - **DBRK** (plus the KL during the bootstrap): **3/5 seeds at 0.96–0.999, flat to d1000, softmax level, with no softmax at runtime.** Row 2: one seed collapsed (the early KL) and one reached 0.79.

**Code** (training only; the served read is unchanged):
- the `track_bootstrap` probe;
- `set_starling_teacher_helper`, `set_starling_teacher_read` and `set_starling_teacher_age` (implemented, used next);
- `mqar-bench starling_bootstrap / starling_handover / starling_teacher_read / starling_bootstrap_kl / starling_teacher_age`.

**Exact-head checks** at `a83efab121d20f7c7c80298e42c23f4be087b4da`, on pod qyslqnzs93z9ax's CPU after the rebase onto main:
- starling lib tests: **16/16**;
- mqar-bench tests: **18/18**;
- rustfmt on both files: clean;
- GPU parity: 37/37.

**Runs:** 39 GPU runs, all at their full steps with no early stop, all `device=cuda`. Outputs `claude-s3c`, `-diag`, `-fix1` and `-fix1b-20261012` are in the cloud store.

STATUS, ROADMAP and #2028 are updated.

**Next:** 10-seed DBRK confirmation, plus DBRK with the teacher age table, then 4096.

**Would the next step differ if this came out the other way?** Yes. Without a reliable bootstrap the line would pivot to the address and selection design.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5489029859 — 2026-10-12T09:45:36Z — Casey-allard — COMMENTED

Self-review at exact head a83efab121d20f7c7c80298e42c23f4be087b4da: the pod tests passed at this head after the rebase (16/16, 18/18, fmt clean, GPU parity 37/37); each of the four stages was pre-registered and FIT before compute, and is scored by its rows (3c row 3, diagnostic row D, fix 1 row 4, fix 1b row 2 for DBRK); observations beyond the bars are labelled as such; the training-only code leaves the served read unchanged. APPROVE (self).


---
_Comment/review ids in this file: 5489029859_

