# ORDER — written-state-count-model (claude-424)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-424, the extension census, the measurement that fix 1 waits on. It follows
claude-423 (PR #4075), which profiled R\*'s count.

## Why

claude-423 found that most of the count's host time goes to building, adjusting, checking and
filtering records for every gram, while the query keeps few of them. At N = 10 on DEV-tune the
kept triples per order are:

| Order | Grams | Kept triples | Kept |
|---|---|---|---|
| 1 | 3,287,899 | 401,191 | 12.2% |
| 4 | 411,773,607 | 3,295,997 | 0.80% |
| 7 | 988,919,928 | 2,588,113 | 0.26% |
| 10 | 1,286,910,428 | 1,842,098 | 0.143% |

Fix 1 would filter on the GPU before the copy. Codex's review of #4075 showed why those
fractions do not bound it. Below the top order, a kept record's adjusted counts are
continuation counts. Those are built from every order-k+1 gram whose suffix
(`PrimeRouteKey::back_off` of its key, then its next token) falls in the kept set. The query's
contexts are mostly frequent ones, and a frequent context has many left extensions. No one has
measured how many order-k+1 grams a filtered count would still have to build and copy.

## What it does

1. **`KnQuery::extension_census(k)`** in `kn_filter.rs`, for k < N. It is fed order k+1's
   partitions in canonical order, and for every order-k+1 triple it counts:
   - the triple itself (all grams);
   - whether `back_off(key)` is one of order k's query contexts. That is the extension a kept
     context record needs;
   - whether (`back_off(key)`, next) is one of order k's query pairs. That is the extension a
     kept triple needs.

   Integer keys, sorted vectors and binary search only, as in the filter.
2. **`kn-reference --extension-census`.** At each order k+1 the sink also feeds that census.
   The flag writes the census JSON and stops after the count: no scoring and no host check.
   Without the flag, nothing the instrument does changes.
3. **One N = 10 count on wick:** all of TRAIN against DEV-tune's keys, budget 2^24.
   - It is a count measurement, not an evaluation. No target is scored, no BPB is computed,
     and no bar or milestone claim rests on it. So under §4.5 it needs no pre-registration
     sheet.
   - DEV-test stays sealed. It is not read.

## Gates

- G1, laptop:
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the
    instrument's `cargo check --features cuda` are clean;
  - the host `kn_` and `ngram_count` tests pass;
  - a new test checks the census against a brute-force count over the oracle's order-k+1
    triples.
- G2, negative gate: with the `back_off` lookup set aside, the new test fails.
- G3, wick: the census run completes at the head. Its order-k total equals the order-k+1
  `grams` of claude-423's `sweep-10.log` for every k.
- G4: the REPORT gives, per order:
  - the fraction of order-k+1 grams a filtered count must still build;
  - what that implies for fix 1's host and copy cost against claude-423's phases.

  These are estimates from the measured counts, not a measured fix.

## Out of scope

No filter is built and no count changes. Fix 1 itself, the whole-order count-of-counts and
digests on the device, and fixes 2–4 are later units, chosen from this census.
