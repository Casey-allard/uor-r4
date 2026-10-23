# REPORT: r-prime-zeta-vs-nulls, kimi-25 (non-author TEST FITNESS audit)

Lab: kimi. Order: [ORDER.md](ORDER.md). Item #2881. Base: `259f9b9b2`. `References #2881`.

## What did not work, first

- **The sheet cannot be stamped `FIT` as it stands, for two reasons that are not the author's to fix alone.** (a) The comparison bar is `PROPOSED, NOT REGISTERED` and the ~$0.20 pod spend is unapproved; both are owner/PM calls already pending on #2881 (comment `6353762758`), and CHARTER §5.2 forbids a reviewer registering them. (b) The outcome rule is not total: positive attribution needs `b - c > F` against all six nulls and the null reading covers `|b - c| <= F` against `Q0..Q2`, leaving every mixed pattern (e.g. clearing `Q0..Q2` but not all of `R0..R2`) unnamed and deciding nothing. One sentence recording the residual as descriptive-only (D22) closes it.
- **A first verification pass nearly cleared the wrong test target.** The six unit tests live in `prime_zeta.rs`, which is compiled into the `prime-zeta-codebook` **binary** (`mod prime_zeta;` at `prime-zeta-codebook.rs:14`), so `cargo test -p uor-r4-discrete --lib` does not include them; the correct command is `--bin prime-zeta-codebook` (`7 / 7` = 6 unit tests + the CLI parser test). The audit summary in the sheet records this so the next reviewer does not repeat the detour.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md` | `TEST FITNESS: NOT_FIT - M1 #2029 prime-zeta-codebook` recorded (kimi, non-author), with the seven verified checks and the two prerequisites |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- Host-only audit (`$0`, no GPU). All seven pinned codebook tables regenerated on host and every `file_sha256` matches the sheet's pin table exactly; ratchets `d32`/`d23`/`d25` show zero added sites; archive numbers (`427/14,287`, `127/3,847`, `38/14,287`), the `F = max(24, 2·sqrt(b+c))` floor and the DEV pins all match their sources. Details are in the sheet's audit summary.

## Checks run at the head

- `uor-harness prep fit -m m1 --verdict NOT_FIT --sheet docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md --mechanism prime-zeta-codebook`
- `uor-harness prep-check` (sheet reads `NOT_FIT`)
- `uor-harness loop report --check`, `uor-harness loop pr-check --local`, `uor-harness loop gates --full`

## Skills used

- `solution-integrity`: every sheet claim re-verified against source, archive or a host re-run rather than trusting the design note; the one gap found (non-total outcome rule) is recorded as a prerequisite instead of being waved through.
- `scope-guard`: sheet and this task folder only; no bar, spend or compute set or released.

## Still open

- Owner/PM registration of the comparison bar and the ~$0.20 pod spend on #2881, and the one-sentence outcome-rule completion; then a fresh non-author review can stamp `FIT`. The battery itself stays HELD until both land. Item `r-prime-zeta-vs-nulls` stays `status: ready`.

## Plain summary

- The r-prime-zeta sheet is mechanically sound — every pin, hash, formula and archive number reproduced exactly on host — but it is `NOT_FIT` for compute until the owner registers the bar/spend (already his pending card) and the outcome rule names its residual case.
