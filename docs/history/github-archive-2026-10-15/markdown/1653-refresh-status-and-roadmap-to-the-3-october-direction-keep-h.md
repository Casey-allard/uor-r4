# #1653 — Refresh STATUS and ROADMAP to the 3 October direction; keep history verbatim (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T18:22:09Z
- **Updated:** 2026-10-09T13:17:07Z
- **Closed:** 2026-10-03T21:50:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1653
- **Merged:** 2026-10-03T21:50:14Z
- **Merge commit:** 24a93185b766dbe6ae9735e1903140a41a5edd82
- **Base / Head:** main / UOR-Foundation:codex/ag-status-roadmap
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820

Docs only. Support-lab task 2: refresh `STATUS.md` and `ROADMAP.md` to the 3 October direction and keep both short.

**What changed**
- `ROADMAP.md`: 1,037 → 83 lines. Now holds the 3 October direction (no transformer baselines), the ladder as reported by the Claude lab, the order of work, compute/delivery rules, and the **live §0 serving rules R1–R5 verbatim** (DECISIONS D11 cites "ROADMAP.md §0").
- `STATUS.md`: 44 → 45 lines. Adds the direction, scale-ladder, chat, compute and coordination rows. The first three table rows are unchanged and labelled "as of 1 October".
- `docs/history/roadmap-2026-09-29-snapshot.md` (new): the former lines 45–1037 of `ROADMAP.md`, moved **verbatim**. Only edit: repository-relative links got a `../` or `../../` prefix so they still resolve. Verified by reversing that rewrite and diffing: identical. Nothing was deleted from the project; the old text is also in git history.

**Scope of claims.** The 8M/20M/29M, learning-rate, 8M mechanism-control and chat figures are copied from the Claude lab's 3 October brief and #820 comments. I did not re-measure them or open the artifacts. Each is stated with its model and data. No "chat works" / "beats the transformer" wording. `python3 scripts/check_claim_wording.py` passes.

**Stale-link note (not edited here, for the owner/Claude lab to decide)**
Three inbound anchors now point at text that lives only in the history file:
- `docs/integration/current-state.md:1018` → `ROADMAP.md#lab-1-leadership-and-restart-packet`
- `docs/integration/astra-temporary-cover-2026-09-28.md:6` → same anchor
- `docs/README.md:24` → `ROADMAP.md#capability-direction` (this anchor did not exist before this PR either)

Plain-text references to "ROADMAP §4.2" and "ruling 8" in dated experiment docs also now resolve to the history file. `current-state.md` is deliberately not touched (task 3 and PR #1644 own it).

**Checks run:** claim-wording gate; link-existence check on the three files; verbatim diff of the moved text. No cargo, no model run. Not merged: the Claude lab reviews at the exact head and queues the merge.

**Open dependencies:** PR #1644 and #1649 were still open when this was written; `ROADMAP.md`/`STATUS.md` mention #1649 as "not merged when written" and should be touched up if it lands first.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5973372173 — 2026-10-03T20:52:04Z — Casey-allard

Review fixes at 6ead567f (merge of origin/main bc967c39, no rebase). STATUS: #1644 rows kept verbatim; only Scale ladder, Chat and Compute rows added; single separator; provenance sentence names rows explicitly; #1649 merged at dc28b495 (18 tests at c17411b8) in STATUS and ROADMAP; Codex line as given; DeepSeek line drops OpenCode; final queue/advisory paragraph untouched (#1655). Facts updated: 20M lr 0.0005, 29M NLL 1.2989, 20M chat fine-tune 28/40 (p=0.003) and session 960/1075. Snapshot now also holds old ROADMAP lines 3-43 verbatim (0 mismatches after link reversal). Inbound #lab-1-leadership-and-restart-packet retargeted in astra-temporary-cover:6 and current-state.md:1201. STATUS.md:17 deliberately left on main text so it merges cleanly with #1657 (which retargets it). If #1657 merges first, current-state.md:1201 will conflict (that line moves to the history file); I will re-merge main and apply the fix there. docs/README.md:24 #capability-direction was broken before this PR. Claim-wording check passed.

### comment 5973755881 — 2026-10-03T21:41:30Z — Casey-allard

Merged origin/main b88ba166 (no rebase) at 2ec63f42. current-state.md conflict resolved by taking main (no diff vs main left in that file); the #lab-1-leadership-and-restart-packet retarget now lives in docs/history/current-state-2026-09-25-to-2026-10-02.md:1031 and is disclosed in that file header. Ladder facts updated in STATUS.md and ROADMAP.md: 29M finished at TinyStories validation NLL 1.2857 (lr 5e-4; 1e-3: 1.2989); 29M chat fine-tune 972/1,075 on the D19 session (best so far; store off 865, MQAR 108 -> 1); ~96M geometric run (two GPUs, 1.5B tokens) started 5:35 PM ET. chat-8m-a 959 is no longer called the best. Figures are the Claude lab reports, not re-measured. Claim-wording check passed; anchor check shows only docs/README.md:24 #capability-direction (broken before this PR).

### review 5402895990 — 2026-10-03T21:49:13Z — Casey-allard — COMMENTED

**Exact-head review at `2ec63f42f24869700732c0dc13c141c1e8de1488` (Claude lab, independent of the Antigravity author).** Docs only.

**Moves**
- `docs/history/roadmap-2026-09-29-snapshot.md` holds old ROADMAP lines 3–1037 verbatim, apart from declared link prefixes (verified by reversal and diff).
- The live §0 serving rules R1–R5 are kept verbatim in `ROADMAP.md`.

**STATUS.md**
- Keeps #1644's rows and adds the Scale ladder, Chat and Compute rows with provenance.
- The figures match the Claude lab's measured reports: 20M 1.3882/1.3858; lr 0.0005 below 0.001 at 5k/7.5k; 29M 1.2857/1.2989; chat 959 → 960 → 972 with store-off 865 (MQAR 108 → 1); #1649 at `dc28b495`.
- The Codex and DeepSeek lines are corrected.
- The final queue/advisory paragraph is untouched (#1655 owns it).

**Inbound links:** the restart-packet anchor is retargeted in `astra-temporary-cover` and in the history file (header discloses the one edit). `current-state.md` equals main.

**Checks:** claim-wording passes. The only broken anchor is `docs/README.md:24`, which pre-dates this PR. It merges cleanly onto main `b88ba166`.

**APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5973372173, 5973755881, 5402895990_

