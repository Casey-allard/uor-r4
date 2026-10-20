# #2276 — Add multilab touch: liveness without overwriting intent (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T03:39:29Z
- **Updated:** 2026-10-13T03:41:43Z
- **Closed:** 2026-10-13T03:41:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2276
- **Merged:** 2026-10-13T03:41:24Z
- **Merge commit:** 06731f803e139d9d5abaa179f7c7ba64bc00016f
- **Base / Head:** main / UOR-Foundation:harness/touch-presence
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only: this touches the harness, which runs no model code. The figures below measure the
harness or a local hook, taken here.

## What and why

Continuous cross-lab presence needs a lab to appear live without a human running anything. A
per-prompt hook is the only mechanism available — and **Kimi is the only lab that has one**
(`UserPromptSubmit`, 5s timeout, registered in `~/.kimi-code/config.toml`).

Neither existing primitive fits that use: `heartbeat` writes both tiers (right for a periodic
check-in, too heavy per prompt), and `write_presence` with no `doing` would have to invent intent.

So `touch_presence` does one thing: **refreshes the timestamp, preserves intent, writes only the
uncommitted live tier.** A prompt cannot dirty the repository, and a touch says "alive" rather than
claiming to know what the lab is working on.

```
$ multilab touch
<lab>: live, updated <timestamp> (doing: <whatever the last heartbeat published>)
```

A touch never changes the parenthesised part — that is the whole point.

**Measured: import + touch is 0.061s** — what a `UserPromptSubmit` hook pays on every prompt,
against a 5-second timeout.

## Result

Decision: **KEEP** (tooling).

5 new tests: touch creates live presence with no heartbeat; it preserves what the lab is doing; it
writes no committed file; it survives an unwritable live tier; it requires a lab.

```
python3 -m unittest discover -s tests/harness -t .   -> 274 tests, OK
python3 scripts/harness/cli.py check                 -> 43 rules, 0 errors
```

## A second defect, in the hook itself

While wiring it I found the existing hook had **never published anything**, and for a reason worth
recording: its gate resolved `cwd` but compared it against an *unresolved* symlink
(`/Users/casey.allard/uor-r4-kimi` → `/Users/casey.allard/uor-r4`), so the containment test could
never match and the hook returned early for a session at the repo root.

Fixed by resolving both sides of the comparison. Verified three ways against the real invocation:

```
cwd=<the symlink>     -> banner printed, presence published
cwd=<resolved path>   -> banner printed, presence published
cwd=/tmp              -> silent, exit 0
```

The hook change lives outside this repository and is **not** in this PR. It is backed up at
`~/.kimi-code/bin/uor-context-hook.bak-20261012` and reverts with one `cp`.

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] `python3 scripts/check_claim_wording.py` passes
- [x] Record: docs/labs/multilab-harness-2026-10-12/README.md (the unit this extends)
- [x] Branch deleted after merge

### Stated rather than hidden

- **Still no foreign lab has run this.** The touch primitive removes the *mechanism* obstacle to
  continuous presence, but only Kimi has a hook, and I have not asked Kimi to reload its config.
  Until then this is tested by me, not by a peer.
- Clicking on a hook is not the same as a lab adopting the harness; a hook publishing presence is
  one lab's liveness, not four.
- The other three labs have no per-prompt hook, so their presence will remain per-session until
  they check in manually.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

