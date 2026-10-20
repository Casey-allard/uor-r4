# #2235 — mqar-bench: a d3000 bucket for context 4096 and reachability=false (prep 1 for starling step 2) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T02:55:33Z
- **Updated:** 2026-10-12T02:57:03Z
- **Closed:** 2026-10-12T02:56:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2235
- **Merged:** 2026-10-12T02:56:34Z
- **Merge commit:** 0c61858eea12ee98db82f78aae988c41c0ea1c4e
- **Base / Head:** main / UOR-Foundation:claude/starling-context
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · count 0/3 (after the #2233 KEEP) · headline "softmax at runtime: yes" → yes. References #2032.

**Preparation PR 1 for ladder step 2**, the starling read against the L2 softmax read at equal budget across context 512 / 1536 / 4096 ([pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6116807513), its amendment and the probe note; FIT). The run is going now, on pod o7hbm7x6u0wu60 at this head's binary.

**Changes, `mqar-bench` only:**
- **A d3000 distance bucket** (positions 3000–3600 back) for context 4096. Shorter windows filter it out, so the 512 and 1536 draws are unchanged. The new test `long_context_buckets_leave_short_windows_unchanged` pins the bucket sets.
- **`reachability=false`** skips the two observation-only D20 reachability counters after the final evaluation. Each puts the whole final panel into one forward (384 items of 4,096 tokens) and ran out of memory on a 24 GB 4090 at every batch size. The default stays `true`, and the report records `"reachability": "not requested"`.

**Exact-head checks** at `c04b2f31f`, on the pod's CPU in a fresh checkout:
- `cargo test --release -p uor-r4-training --example mqar-bench`: **18 passed**, including the new test;
- `rustfmt --check` on `mqar-bench.rs`: clean;
- GPU probe at this binary: L, W03L and M16 complete end to end at context 4096, batch 4.

**Would the next step differ without this?** Yes. Without it, step 2 cannot run at 4096.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5486170763 — 2026-10-12T02:55:35Z — Casey-allard — COMMENTED

Self-review at exact head c04b2f31f2cb8468a41e6eab4de169550e1cc13c: the bucket is filtered by `high < context`, so the 512 and 1536 draws are unchanged (pinned by the test); `reachability=false` only skips observation counters, the default is unchanged, and both test literals are updated; tests 18/18 and rustfmt clean on the pod at this head. APPROVE (self).


---
_Comment/review ids in this file: 5486170763_

