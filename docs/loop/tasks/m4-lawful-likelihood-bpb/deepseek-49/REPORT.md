# REPORT: m4-lawful-likelihood-bpb, deepseek-49 (the pre-registration sheet)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item: `m4-lawful-likelihood-bpb` (#2861). Base: `775d6aad2`. `References #2861`.

## What did not work, first

- **The unit had to be re-scoped, not re-built.** The design (PR #2940) and the host-only PRLL implementation (PR #2945, `91a689f1b`) were already merged by deepseek yesterday; the item stayed `ready` for one missing step: the pre-registration sheet (item step 4). The earlier attempt was blocked because the item's scope did not list `docs/harness/prep/`, which the PM has since added (`docs/harness/prep/m4-lawful-likelihood-bpb*` is in the item frontmatter now), so the sheet is claimable.
- **The chat held-out stream is not in the repository.** It is the `chat-v0-p2` held-out token stream recorded on the pod in the archived criterion-2 lab (`heldout.u16`, sha256 `e5f400b0…`). The sheet therefore names and hashes it from that archive and adds a hard freeze rule: the scoring host re-verifies the hash and token count before scoring, and any mismatch halts.
- **The scoring run cannot happen yet.** No discrete model is exported through `UORLUT01` (`m4-export-uorlut01-cli` is open) and no D40 release exists. The sheet therefore registers only the instrument and the stream; the registered scoring arm stays HELD.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m4-lawful-likelihood-bpb-2026-10-23.md` | new one-page pre-registration: instrument (PRLL), frozen stream and hashes, window construction, arms A0/B0/C0/U, the owner's 0.90 bar (not set here), pending base artifact, the seven finite controls, cost/power, stop rule; `TEST FITNESS: PENDING` |
| `docs/labs/m4-lawful-likelihood-bpb/README.md` | pointer to the sheet and what it does and does not release |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- None claimed: this unit registers an instrument; no model, no GPU, no BPB number. The host-only controls already on main are cited, not re-run.

## Checks run at the head

`uor-harness prep-check` (sheet present, `PENDING`), `loop report --check`, `loop pr-check --local`, `loop gates --full`; a non-author FIT is requested before any registered scoring.

## Skills used

- `solution-integrity`: did not rebuild the merged instrument; verified what already existed on main and filled only the item's actual missing step.
- `scope-guard`: sheet and lab README only, both inside the item scope; no bar, stream re-derivation, or compute claim.

## Still open

- A **non-author FIT** on the sheet (Claude, Kimi or antigravity), then the PM/owner registration card: the two conditions in §5 (uncalibrated-likelihood reading of the 0.90 bar; stream confirmation).
- The scoring run itself: blocked on `m4-export-uorlut01-cli` plus the owner's approval recorded verbatim in the lab's own chat (CHARTER §5.1); GPU only, host checks only until then.
- Notebook row (`docs/research/notebook/m4-serving.md`) is outside the item's scope; recorded here for the owning line to backfill.

## Plain summary

- The lawful-likelihood instrument and its seven finite controls were already merged; this PR adds the one missing step of the item: a frozen, hashed, one-page pre-registration for the chat held-out stream, with the 0.90 bar left where it belongs (the owner's) and no compute released.