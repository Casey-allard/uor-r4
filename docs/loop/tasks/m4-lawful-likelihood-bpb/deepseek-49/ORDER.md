# ORDER: m4-lawful-likelihood-bpb, deepseek-49 (pre-registration sheet)

Lab: deepseek. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861). Base: `775d6aad2`. `References #2861`.

## Goal

The design (PR #2940) and the host-only lawful-likelihood implementation (PR #2945, `91a689f1b`, `crates/uor-r4-integer/src/lawful_likelihood.rs`) are on main. The item's one missing lab step is the one-page pre-registration (item step 4) that names and hashes the chat held-out stream, fixes the scoring formula, carries the owner's 0.90 bar, and goes to a non-author FIT before any GPU run. The PM has added `docs/harness/prep/m4-lawful-likelihood-bpb*` to the item scope, so the sheet is claimable.

## Cancel probe

Class instrument (host-only; no GPU). At the base: the module and its nine controls are on main; `docs/harness/prep/` has no `m4-lawful-likelihood-bpb*` sheet (`git ls-tree -r origin/main docs/harness/prep`), and the item is `ready`. Task stands.

## Files in scope

- `docs/harness/prep/m4-lawful-likelihood-bpb*` (the sheet)
- `docs/labs/m4-lawful-likelihood-bpb/**` (lab record)
- this task folder

## Done when

- The sheet names and hashes the chat held-out stream (from the archived criterion-2 record; the file itself is not in-repo, so the sheet states the re-verify-on-host rule), fixes the formula and window construction, carries the owner's bar without setting it, lists arms and controls, cost/power and stop rule, and ends `TEST FITNESS: PENDING`.
- `uor-harness prep-check` reads the sheet as present and `PENDING`; `loop report --check`, `loop pr-check --local`, `loop gates --full` pass; a non-author FIT is requested from Claude/Kimi/antigravity.
- No GPU, no model export, no bar change; the item stays `ready`.

## Stop and ask if

- The 0.90 bar, a hold or a spend would have to be set (none is; only the owner's existing bar is referenced).
- A file outside the item scope is needed (for example the notebook row; it is recorded as an out-of-scope follow-up instead).