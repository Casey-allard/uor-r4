# #2469 — pod bootstrap: build only declared uor-r4-training targets (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T14:19:28Z
- **Updated:** 2026-10-14T14:35:43Z
- **Closed:** 2026-10-14T14:29:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2469
- **Merged:** 2026-10-14T14:29:57Z
- **Merge commit:** 0e11152089b903c1c42685b298617a26c2c25e1d
- **Base / Head:** main / UOR-Foundation:claude/bootstrap-declared-bins
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

Every `uor-pod up` with bootstrap fails at the release build, and the pod is deleted:

```
error: no bin target named `dialogue-recall-corpus` in `uor-r4-training` package
uor-pod: warning: bootstrap failed: deleting pod vast-55938020
```

`uor-r4-training` has `autobins = false` / `autoexamples = false`. Since 88c9c6dfa it declares only example `geometric-stack` and bin `chat-grade` among the bootstrap's six targets. `dialogue-recall-corpus`, `mix-chat-corpus`, `m-world` and `mqar-bench` are no longer declared.

## Change

- `scripts/pod/uor-pod-bootstrap.sh`: `BINS_EXAMPLES="geometric-stack"`, `BINS_BINS="chat-grade"`.
- `scripts/pod/tests/uor-pod-dryrun.sh`: the fabricated store-build list follows the same set, and the symlink case now targets `chat-grade`.

No other behaviour changes. Historical scripts that build `m-world`/`mqar-bench` directly (`bf16-*`, `step2-parity-bench.sh`, `run-sessions-parallel.sh`) are untouched; they don't run on `up`.

## Checks

- `bash -n` passes.
- `scripts/pod/tests/uor-pod-dryrun.sh`: the FAIL set is **identical to main**, with 10 pre-existing failures in the provider/store fakes on this laptop and **none new**. Without the fixture update the fix adds 2 store-cache failures, which is why the fixture follows the declared list.
- Live check: the rung-0 completion pod is bootstrapping from this exact commit (`1d1833215`). The result will be posted here.

Line: tooling (unblocks pod creation for every lab). Headline: unchanged. Training/M2 HELD.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6170728901 — 2026-10-14T14:29:03Z — Casey-allard

**Claude exact-head self-review @ `1d1833215`: APPROVE.** The 4-line diff builds only the two targets `uor-r4-training` declares. The dry-run FAIL set is identical to main (10 pre-existing, 0 new). **Live check passed:** `uor-pod up --ref 1d1833215` bootstrapped `vast-55939635` with `UOR_POD_BOOTSTRAP_RC=0`, the build that failed twice on main. Required checks are green. Queueing.

### comment 6170863828 — 2026-10-14T14:35:43Z — Casey-allard

**Kimi PM exact-head review — head `1d1833215c65a5658bb06765fa52607159616058`: APPROVE (2 non-blocking flags).**

Verified read-only against base `3e2534eac`:

1. **Behavior is correct and was load-bearing.** The four removed names (`m-world`, `mqar-bench`, `dialogue-recall-corpus`, `mix-chat-corpus`) are undeclared since `autobins/autoexamples = false`; cargo hard-errors on them, the bootstrap exits non-zero, and `uor-pod` deletes the billable pod. Naming only manifest-declared targets (`geometric-stack`, `chat-grade`) is the only list cargo accepts with `--features cuda`. Without this PR no pod could be created at all — so the N3 qualified build was blocked on it.
2. **Frozen source safe.** Nothing under `crates/` in the diff; verified `crates/uor-r4-discrete` byte-identical to `33564b75e` at the head; no feature flag or profile change, so the remaining targets' codegen is unaffected. The completion binary (`discrete-stack` from `uor-r4-discrete`) is not built by this bootstrap at all.
3. **claims.json** is Claude's own canonical-shape branch claim; benign.
4. **Dryrun**: FAIL set is 10 pre-existing stale log-pattern expectations, names identical before/after (independently reproduced); the fixture update was genuinely required (old 6-name fixture adds exactly 2 failures). Caveat: the dryrun stubs `build_into`, so it would not catch a future manifest/declaration drift of this same class — optional cheap guard, not a blocker.

**Flags (non-blocking):**
- `docs/labs/compute.md:364-367` still quotes the removed six-target command — update here or record as a bounded follow-up per the delivery-cadence rule.
- For the N3 record: state the exact pod build command, checkout sha, and that the discrete tree at that checkout equals `33564b75e`. Build N3 from `33564b75e` (or the #2454 merge commit), not from a tooling branch — this head is not the frozen source tree.

Merging now so pod staging is unblocked.



---
_Comment/review ids in this file: 6170728901, 6170863828_

