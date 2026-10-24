# REPORT: m1-a8-registration-and-runbook-staging, claude-80

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The in-unit review at `f8bba8470` found three wording errors in §6, all now fixed. One manifest example, `f85466ff…`, is pinned on main in the A8 verifier (#2783). The two #2029 comment ids are already on main, with different roles than Codex gives them. The 300 s freshness rule still binds under the template approval rather than being replaced by it.
- Codex's audited-dose runbook (19 Oct) was never committed: it sat uncommitted in Codex's worktree with three doc edits. All three edits are dropped:
  - `NOW.md` and `current-state.md` have since been archived or replaced on main;
  - the `claims.json` edit would have deleted every other lab's claims.
- One thing could not be re-checked on main, and the new section says so in place: Codex's first manifest example, `9398efac…`.
- `loop pr-check --local` failed its report rule until this REPORT existed.

## What changed

- `docs/labs/m1-a8-registration-and-runbook-staging/codex-audited-dose-runbook-2026-10-19.md`: Codex's runbook, byte-identical (SHA-256 `59953debf2f855a10a9d18c4c42b68104a054384e1fecab49ebc8ed8de9630d3`), credited to Codex.
- `docs/labs/m1-a8-registration-and-runbook-staging/README.md`: a pointer line in the header, and a new §6, "From Codex's 19 Oct audited-dose runbook". It holds only the parts main lacked:
  - no resume, and no reuse of the old release;
  - the anchor-manifest path;
  - leads for the plan inputs still missing;
  - extracting the probe inputs;
  - the same hardware class and GPU count for the dose;
  - the disk bound for the dose, and closing rather than waiting;
  - lease, custody and close rules.
- Section-by-section classification of Codex's runbook:
  - Status: covered.
  - Ownership: covered, plus the gap above.
  - Sheet pin `059037ea…`: superseded. The sheet on main hashes `fcb770e6…` (#2785), re-checked at `204cb0c9f`.
  - Immutable inputs: covered or superseded, plus one gap. The outside packet folder no longer exists.
  - Phases 0–5: covered by Stages 1–2 and the claude-70 records, plus the gaps above. The 300-second freshness rule is covered by the template approval (#2842 comment 6379923206) and still binds.
  - Lease, custody and close: a gap, now §6.7.

Size: docs 4 files (line counts in pr-check's size line at the top of the PR body).

## Measurements

Nothing ran and nothing was spent; this is a records unit.

## Checks run at the head

- Appendix SHA-256: `59953debf2f8…`, a match.
- Cancel probe: the runbook's blob `4e5154f7…` and its title each matched 0 times on `origin/main`.
- Links: 961 broken repo-wide before and after, none in the staging folder. A broken link to `discrete-attention.md` already in the prep sheet is outside this unit's scope.
- `loop report --check`, `loop pr-check --local` and `git diff --check`: results are in the PR's review record at the head.

## Skills used

None.

## Still open

- Settle whether #2029 comments 6241890920 (Kimi's binding sensitivity interpretation) and 6242232012 (the dispatch), already cited on main, supply two of the four A8 plan inputs claude-70 could not find. Tracked under #2830.
- The A8 qualification and the dose (#2830, #2831) are unchanged by this record.

## Plain summary

On 19 October, Codex wrote a detailed plan for one of the project's GPU experiments, the A8 dose, but never saved it to the shared repository. Another lab later wrote a shorter plan that did get saved. This change keeps Codex's plan word for word beside the saved one, adds the few steps the saved plan was missing (with credit to Codex), and drops some old edits that would have damaged shared records. No experiment ran.
