# ORDER: r-prime-zeta-vs-nulls, kimi-25 (non-author TEST FITNESS audit)

Lab: kimi. Item: [r-prime-zeta-vs-nulls](../../../../roadmap/items/r-prime-zeta-vs-nulls.md) (#2881).
Base: `259f9b9b2`.

## Goal

The sheet `docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md` (antigravity-27, PR #3014) is at `TEST FITNESS: PENDING` and its step 2 plus the #2881 card (comment `6353762758`) request a non-author FIT review from Claude or Kimi. This unit performs that audit against the sheet's own content, the source, the ratchets and the archive, and records the verdict in the sheet, following the #3062 pattern. It registers no bar and approves no spend — both are the owner's calls (CHARTER §5.1/§5.2).

## Cancel probe (run first)

Class instrument / pre-registration audit (host-only, no GPU). At `259f9b9b2`: the sheet reads `TEST FITNESS: PENDING` (`grep -c "TEST FITNESS: PENDING" docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md` = 2), no non-author FIT is recorded on it, and the author (antigravity) cannot review it. Task stands.

## Files in scope

- `docs/harness/prep/r-prime-zeta-vs-nulls*`
- `docs/labs/r-prime-zeta-vs-nulls/**`
- this task folder

## Done when

- The audit is recorded in the sheet's TEST FITNESS line (`uor-harness prep fit -m m1 --verdict <verdict> --sheet docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md --mechanism prime-zeta-codebook`), with each check the audit ran named so another lab can repeat it.
- `uor-harness prep-check`, `loop report --check`, `loop pr-check --local` and `loop gates --full` pass.
- No bar, hold, spend or FIT-gated compute is set or released by this PR; the item stays `status: ready` (`References #2881`).

## Stop and ask if

- The comparison bar or the pod spend would have to be registered by the reviewer (owner call, CHARTER §5.2) — they are recorded as prerequisites instead.
- A file outside the item scope would need to change.
