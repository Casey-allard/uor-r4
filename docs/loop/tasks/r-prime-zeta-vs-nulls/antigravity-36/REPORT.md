# REPORT: r-prime-zeta-vs-nulls, antigravity-36

Lab: antigravity. Item: [`r-prime-zeta-vs-nulls`](../../../../roadmap/items/r-prime-zeta-vs-nulls.md) (#2881). Milestone: M1 (#2029).

## What did not work, first

- **In `kimi-25` (PR `#3067`), Kimi's non-author `TEST FITNESS` audit verified all mechanical pins and tests on host (`7 / 7` in `--bin prime-zeta-codebook`, all 7 table SHAs bit-exact) and recorded two prerequisites before `FIT` (`TEST FITNESS: NOT_FIT`, configuration / pre-registration gap):**
  1. **Prerequisite `(b)` (author fix, resolved in this PR):** The `## Bar` outcome rule covered positive attribution ($b - c > F$ against all six nulls `Q0..Q2, R0..R2`) and the quota-equivalence reading ($|b - c| \le F$ against `Q0..Q2`), leaving mixed discordance patterns (e.g., clearing `Q0..Q2` while missing one or more `R0..R2` seeds) unnamed. Also, `## Steps` step 1 did not name `--bin prime-zeta-codebook` as the cargo test target.
  2. **Prerequisite `(a)` (owner/PM call, still open):** The comparison bar (`PROPOSED, NOT REGISTERED`) and `~$0.20` pod spend await owner/PM registration on `#2881` (`6353762758`; CHARTER §5.1, §5.2).

## What changed

- Updated [`docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`](../../../../harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md):
  - Added the explicit `Mixed / residual discordance pattern (descriptive-only under D22)` clause in `## Bar` so the outcome partition is total across all six null arms.
  - Updated `## Steps` step 1 to name `~/.cargo/bin/cargo test -p uor-r4-discrete --bin prime-zeta-codebook` (`7 / 7` passed).
  - Added `### Post-review author update (antigravity-36, 2026-10-23)` recording that prerequisite `(b)` is resolved and only prerequisite `(a)` (owner/PM bar and spend registration on `#2881`) remains.
- Updated [`docs/labs/r-prime-zeta-vs-nulls/README.md`](../../../../labs/r-prime-zeta-vs-nulls/README.md) to reflect the `kimi-25` audit and `antigravity-36` update.

## Measurements

- `~/.cargo/bin/cargo test -p uor-r4-discrete --bin prime-zeta-codebook`: `7 passed; 0 failed` (`6` unit tests in `prime_zeta.rs` + `1` CLI mode parser test).
- `uor-harness prep-check` on `docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`: 8/8 required blocks present, `TEST FITNESS: NOT_FIT` preserved until prerequisite `(a)` lands.

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-discrete --bin prime-zeta-codebook`
- `uor-harness loop report --check`
- `uor-harness loop pr-check --local`
- `uor-harness loop gates --full`

## Skills used

- `uor-research-architect`, `solution-integrity`, `scope-guard`.

## Still open

- Prerequisite `(a)`: owner/PM registration of the comparison bar and `~$0.20` pod spend on `#2881` (`6353762758`), followed by the non-author `TEST FITNESS: FIT` stamp and execution of the 7-arm zero-update GPU battery. Item `r-prime-zeta-vs-nulls` stays `status: ready`. References #2881, #2029.

## Plain summary

I updated the `r-prime-zeta-vs-nulls` pre-registration sheet (`#2881`) to resolve Kimi's non-author `TEST FITNESS` review prerequisite `(b)` — adding the mixed-pattern residual clause that makes the `## Bar` outcome rule total under D22 and naming the exact `--bin prime-zeta-codebook` test command (`7 / 7` passed) — so that only owner/PM registration of the bar and `~$0.20` pod spend remains before `FIT`.
