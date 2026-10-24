# ORDER: m1-a8-registration-and-runbook-staging, claude-80 (fold Codex's 19 Oct audited-dose runbook into the staging runbook)

Lab: claude. Item: [m1-a8-registration-and-runbook-staging](../../../../roadmap/items/m1-a8-registration-and-runbook-staging.md) (#2832). Base: `204cb0c9f`.

## Goal

Codex wrote an audited A8 dose runbook on 19 October 2026 ("Audited A8 dose runbook — 19 October 2026", 197 lines, SHA-256 `59953debf2f855a10a9d18c4c42b68104a054384e1fecab49ebc8ed8de9630d3`). It never reached main. Casey's direction is that every piece of local work is understood and finished into main, and nothing is left as unexplained archive.

This unit:

1. adds Codex's runbook byte-identically as an appendix in the staging folder, so the source is kept and credited;
2. compares it section by section with main's staging runbook (#2976, with #3090's identity-pin corrections and the claude-70 erratum of 24 Oct);
3. folds only the genuine gaps into the staging README, in a short section that credits Codex and links the appendix.

Codex's file pins a sheet hash `fcb770e6…` that #2784 later changed to `059037ea…`, and cites an outside folder that no longer exists. Those pins are superseded; none is copied as current. Nothing runs in this unit, so it adds no claim about a run.

## Cancel probe (run first)

Does main already contain Codex's runbook? Check: the file's git blob (`git hash-object` = `4e5154f7…`) does not appear in `git ls-tree -r origin/main`, and `git grep 'Audited A8 dose runbook' origin/main` finds nothing. Result at the base: 0 blob matches, 0 title matches. The task stands. Class: instrument.

## Files in scope

- `docs/labs/m1-a8-registration-and-runbook-staging/**`
- `docs/loop/tasks/m1-a8-registration-and-runbook-staging/claude-80/**`

## Done when

1. The appendix `docs/labs/m1-a8-registration-and-runbook-staging/codex-audited-dose-runbook-2026-10-19.md` is byte-identical to Codex's file (SHA-256 above).
2. Each Codex section is classified as covered by main, superseded, or a genuine gap, with the main section or the superseding record cited.
3. The staging README points to the appendix near the top and gains a section "From Codex's 19 Oct audited-dose runbook" holding only the genuine gaps, Codex credited, every pin re-checked against main's current identities or plainly marked as not re-checked.
4. Checks pass: `loop pr-check --local`, `git diff --check`, and `cargo xtask links` with no broken link in the staging folder.

## Stop and ask if

- A gap can only be filled by stating a pin that cannot be re-checked against main, and the gap would be misleading without it.
- A file outside scope must change.
- Codex's runbook turns out to be on main already under another name.
