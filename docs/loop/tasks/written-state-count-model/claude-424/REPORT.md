# REPORT: written-state-count-model, claude-424 (the extension census)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **A query filter barely thins the low orders' extension work** [claude]. At orders 0–2, 85.2–100% of the order-k+1 grams back off to a context order k's query keeps, so a filtered count must still build them. At order 3 the share is 50.95%. Only from order 6 up does a filter remove more than 92%.
- **The assembler still needs every own gram.** The census changes nothing in `NgramAssembler::push`, which hashes each whole order and checks its counts of counts. Unless push moves to the device, every own-order gram still has to be copied to the host. Push took 241.203 s in claude-423, and that part of the cost stays.
- **Nothing here is a measured saving.** No filter was built. Every saving below is an estimate that scales claude-423's measured phase times by the census fractions. It assumes cost grows linearly with grams.
- **The census costs a full count.** The run took 1,929 s wall, close to claude-423's 1,910 s, because it counts every order in full to tally the extensions. It is a one-off measurement, not a faster path.

## What changed

- **`KnQuery::extension_census(k)`** (`crates/uor-r4-discrete/src/kn_filter.rs`, k below the top order). It is fed order k+1's partitions in canonical order. For each triple it counts:
  - the gram;
  - whether `back_off(key)` is one of order k's query contexts (a context extension);
  - whether (`back_off(key)`, next) is one of order k's query pairs (a pair extension).
  
  It uses integer keys, sorted vectors and binary search. It refuses the top order, a partition of the wrong order and keys that fall.
- **`kn-reference --extension-census`.** The sink feeds each order to that order's census. The flag writes the census JSON and returns after the count, with no scoring. It refuses `--check-host` and `--sealed-run`. Without the flag the instrument does what it did before.
- **Tests:**
  - `kn_extension_census_counts_as_brute_force` checks the census against suffixes rebuilt from the oracle's tokens, on one TRAIN seed (0x424a) and one query seed (0x424b), N = 4. It also checks the three refusals.
  - `the_extension_census_scores_nothing` checks the instrument path.

## Measurements

The run is wick's 4090 at `f3bf5dadb`, budget 2^24, all of TRAIN (2,091,560,001 tokens) against all of DEV-tune at N = 10, with `--extension-census` (job `c424-census`, [`run1/c424/`](run1/c424/)). It took 1,929 s wall with a peak resident set of 8,934,376 KiB. `census-10.json` has sha256 `c31995c5a51cdfbb44318dfb2c088de49467c2276931ee8a5a50242ddada271f`.

Per order k, the share of order-k+1 grams a filtered count must still build:

| k | Order-k+1 grams | Context extensions | Share | Pair extensions | Share |
|---:|---:|---:|---:|---:|---:|
| 0 | 3,287,899 | 3,287,899 | 100.00% | 3,276,487 | 99.65% |
| 1 | 52,997,638 | 52,962,650 | 99.93% | 35,538,318 | 67.06% |
| 2 | 201,064,161 | 171,319,317 | 85.21% | 64,165,321 | 31.91% |
| 3 | 411,773,607 | 209,782,069 | 50.95% | 60,970,168 | 14.81% |
| 4 | 633,254,327 | 171,205,591 | 27.04% | 47,148,084 | 7.45% |
| 5 | 829,909,716 | 117,985,058 | 14.22% | 32,892,255 | 3.96% |
| 6 | 988,919,928 | 75,577,667 | 7.64% | 22,609,172 | 2.29% |
| 7 | 1,113,001,553 | 48,397,859 | 4.35% | 15,985,509 | 1.44% |
| 8 | 1,209,995,770 | 32,644,979 | 2.70% | 11,791,960 | 0.97% |
| 9 | 1,286,910,428 | 23,134,389 | 1.80% | 9,150,742 | 0.71% |
| **all** | **6,731,115,027** | **906,297,478** | **13.46%** | **303,528,016** | **4.51%** |

- A kept context record's continuation counts read every context extension. A kept triple reads only its pair extensions. So the context share is the one that bounds a filter.
- The order-0 row is 100% by construction, because every order-1 gram backs off to the empty context.
- The census's extension grams plus order 0's 4,079 own grams equal claude-423's 6,731,119,106 own grams.

## What the numbers say

All figures here are estimates from measured counts and claude-423's measured phases [claude].

1. **Extension runs.** claude-423 measured 261.207 s of host `extension_runs` over orders 1–9. Scaling each order's time by its context share gives about **32.7 s**. Orders 1–3 account for about 14.5 s of that, because their shares are 51–100%. The rest of the 261 s goes to extensions no query reads.
2. **The query filter `keep`** (378.940 s) runs on the host over every adjusted record. Done on the device before the copy, it would touch only the kept records and their extensions. The census does not measure that kernel's cost.
3. **Copies and record building** (90.640 s and 185.150 s) cover both the own count and the extension count. claude-423 did not record that split.
   - The extension half could shrink by about the 13.46% share.
   - The own half cannot shrink while push needs every own gram on the host.
4. **What fix 1 alone leaves.** Push (241.203 s), the own-order copies and records, selection (260.940 s) and sort (198.145 s) all stay. So fix 1 alone cannot reach the 3–5 minute hypothesis.
5. **What claude-425 changes.** Fixes 2 and 3 remove the repeated stream selects and the separate extension count (374.996 s of outer interval in claude-423). After 425, order k's runs are built from order k+1's raw counts, which are already counted. The census shares then say how much of that raw data a device-side filter must still send to the host: 13.46% of extension grams in all, but 51–100% at orders 0–3.

## Checks run at the head

- **G1, laptop, at `f3bf5dadb`:**
  - the stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the instrument's `cargo check --features cuda` are clean;
  - the host `kn_` and `ngram_count` tests pass, 32 / 32;
  - the instrument's tests pass, 6 / 6.
- **G2, negative gate:** with the suffix set to the triple's own key instead of `back_off(key)`, the brute-force test fails (0 against 2,863).
- **G3, wick:** the census completed at the head. For every k in 0..9, its order-k total equals the order-k+1 `grams` of claude-423's `sweep-10.log` (10 / 10, [`head.txt`](run1/c424/head.txt)). Its per-order count records are identical to claude-423's in all 10 shared fields across all 11 orders: `jq -S '.orders'` of `census-10.json` is `cmp`-identical to claude-423's `sweep-10.json` orders restricted to those fields. The query sha matches DEV-tune's.
- **G4:** this REPORT.
- **Not run:** the CUDA test suite on wick. The new code is host-only, and the census run exercised the GPU counter end to end.

## Skills used

working-discipline.

## Still open

- **Fix 1's design.** A device-side filter has to keep the whole-order counts of counts and both digests, either by moving push to the device or by copying every own gram. Decide which after claude-425 lands, and measure it.
- **The split of copies and records** between own and extension counts. claude-425's sweep removes the separate extension count, which settles this.
- **DEV-test was not read.** Its census is unmeasured. claude-422 kept about 0.27% of DEV-test's top-order triples against 0.143% for DEV-tune, so its shares are likely higher.

## Plain summary

If the count kept only what DEV-tune's queries read, it would still have to build about one in seven of the grams above order 0 (906,297,478 of 6,731,115,027). These are the grams that feed the lower orders' continuation counts. About half of them (48.3%) are order-1 to order-4 grams backing off to orders 0–3, where 51–100% of grams are needed. From order 6 up, under 8% are needed. Filtering alone therefore removes most of the wasted high-order work but none of the integrity hashing, which still needs every gram. The next unit, claude-425, removes the repeated passes first.
