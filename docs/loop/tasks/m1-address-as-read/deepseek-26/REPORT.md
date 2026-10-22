# REPORT: m1-address-as-read, deepseek-26 (non-author FIT review of the serving-time address-as-read oracle)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `683e182ba`. Item PR (not a loop setup PR); the other-vendor exact-head review follows the normal §5.5 path.

## What did not work, first

- **The verdict is `NOT_FIT`, and deliberately so.** The sheet's shape is complete — all 8 blocks, clear arms, stop rules, bounded cost — and the delivered controls pass (`address_read`: 6/6 finite tests). But a FIT verdict clears compute only for **the exact configuration in the sheet** (D22), and that configuration is not yet pinned:
  1. **The frozen, hashed panel does not exist.** `docs/harness/panels/m3-memory.json` gives the milestone panel as `>= 200` rows for a decisive run and carries no content SHA-256 for an 80-row evaluation panel; the sheet defers the pin to a future `#2853` freeze. State the exact composition and SHA-256, and reconcile 80 vs 200 rows against D22 §3.
  2. **The candidate checkpoint is not pinned.** The base artifact is `m1-trunk-assembly-and-training` (`#2845`), which is `held`, with no SHA-256 in the sheet.
  3. **The bar is a proposal.** Section 3's numbers need the owner's registered bar and ISO-UTC time (CHARTER §5.2); the `19/40` vs `20/40` baseline wording must settle to one baseline for the `+8` gain.
  4. **The consumer is not commit-pinned.** "Landed in `#2950`" should name the delivered commit SHA (and the 6 finite tests) so the configuration is immutable.
- **I did not run or plan any GPU work.** The evaluation run is a separate unit behind the owner's bar and typed release; this review clears nothing (D22).
- **I did not add the notebook row** from the item's step 6: rows record the evaluated result, and `docs/research/notebook/**` is outside the item's scope. That belongs to the run unit.
- **The `prep fit` receipt is a live-tier coordination write**, so the committed record is the sheet's `## Verdict` (which rides this PR); the receipt line is carried by `ledger-sync` at delivery if the gate asks for it.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` | docs | +~22/−1 | top status now says FIT-reviewed `NOT_FIT` by deepseek and that no compute is cleared; a `## Verdict` section carries the rendered `prep fit` record and the four pinned prerequisites |
| `docs/loop/tasks/m1-address-as-read/deepseek-26/{ORDER,REPORT}.md` | docs | small | this order and report, plus receipts |

**Size:** about 75 changed lines (ORDER estimate about 50, 1.5).

## Measurements

- **Completeness:** `uor-harness prep-check` reports the sheet `complete: true` with all 8 blocks (`bar, base, split, steps, stop, cost, power, fit`) before this change; after the verdict it reads `NOT_FIT` and `may_compute: false`.
- **Controls:** `cargo test -p uor-r4-object-store --test address_read` = 6 passed (exact hit, exact miss, wrong-address/tampered, overwrite/version/evicted, sieve, byte-identical round trip).
- **Configuration checks:** panel has no content SHA-256 and says `>= 200` rows for a decisive run; `m1-trunk-assembly-and-training` is `held`; no owner bar registration found for `#2833`; the sheet's base consumer reference names a PR, not a commit.

## Checks run at the head

- `UOR_LAB=deepseek uor-harness prep-check`: sheet `NOT_FIT`, `may_compute: false`.
- `UOR_LAB=deepseek uor-harness loop report --check`, `loop pr-check --local`, `loop gates --full`, `loop gates --break all`: run before the review request.
- `cargo test -p uor-r4-object-store --test address_read`: 6/6.

## Skills used

- **solution-integrity**: FIT is treated as a clearance of a pinned configuration, not a compliment about a document; every unpinned prerequisite is named instead of waved through.
- **holding-the-goal**: the item's own unit boundary was kept — FIT review is host-only work, the GPU run stays behind the owner bar and typed release.

## Still open

- **Pinning the four prerequisites** is antigravity's side of the DRAFT sheet (panel freeze/SHA, checkpoint SHA, owner bar registration, consumer commit). Once pinned, re-request FIT.
- **The GPU evaluation** is a separate unit: needs the owner's registered bar, PM dispatch and Casey's typed release (CHARTER §5.1, D40).

## Plain summary

- **The sheet gets an honest non-author FIT review, and it refuses clearing compute for now.** The shapes and controls are good, but the panel, checkpoint, bar and consumer commit are not yet pinned — so a FIT would clear a configuration nobody can reproduce.
- **Four concrete prerequisites are recorded in the sheet and on #2833**, each with what must be added (composition + SHA-256, checkpoint SHA-256, owner bar + ISO-UTC time, consumer commit).
- **No GPU, no training, no spend, no bar decision** was made; re-requesting FIT once the pins are in is a small follow-up.