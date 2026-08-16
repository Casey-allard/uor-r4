# GPT-2 certified dense source execution (#704)

- **Date:** 2026-08-15
- **Role:** Empirical Criterion and implementation record
- **Scope:** offline GPT-2 source-teacher execution only
- **Issue:** [UOR-Foundation/uor-r4#704](https://github.com/UOR-Foundation/uor-r4/issues/704)

This file is append-only evidence for the Choice-A dense migration. It does not
change or weaken the deployed R4 inference operation contract: deployed graph
inference remains allocation-free and multiplication-free. The floating-point
work described here runs only while an offline teacher produces observations.

## Decision history

The first exact-owner prototype was about `220x` slower than conventional GPT-2
and was not shipped. Immutable prepared weights did not rescue it: the real
layer-0 `c_attn` consumer measured `1357.803945x`, and a later lookup/grade
reachability study still left an optimistic `45.6–57x` gap. A first
certified-native row design matched exact output bits but measured
`16.903285271x` despite certifying 20,676 of 20,736 lanes. Those negatives rule
out allocation cleanup, prepacking alone, and the original certified bound.

The maintainer then selected **Choice A**: compiler arithmetic may use a native
lane only when a mechanical witness establishes the declared binary32 result;
uncertain lanes use the pinned exact owner. Output semantics remain exact and
the deployed runtime remains unchanged.

The replacement factorization passed its strict layer-0 cheap gate before any
whole-model clock:

| metric | result | rule |
|---|---:|---:|
| paired median, real layer-0 `c_attn` | `1.838843890x` | descriptive |
| exact one-sided 95% bootstrap upper | `1.847055652x` | `<4.0x` |
| lane census | 20,673 fast + 63 refined + 0 fallback | complete |
| hot allocations | 0 | 0 |
| pinned-exact output parity | bit-for-bit | required |

That positive cheap gate activated, but did not prejudge, the whole-model run.

## Pre-declared whole-model contract

- **Metric:** paired candidate/conventional elapsed time for three independently
  reset real GPT-2 stories, 11 recurrent steps total.
- **Reachability instrument:** all 49 fixed matrices must be reached with
  exactly 539 calls and 1,465,211 output lanes per suite; fast and refined arms
  must both be nonzero and exact fallback must be zero before timing.
- **Correctness:** production/default equals explicit conventional state;
  candidate equals the stored pinned-exact recurrent state after every step;
  every poisoned output is overwritten and every lane receives one verdict.
- **Fairness:** one loaded immutable model, one prepared caller-owned workspace,
  two complete warmup suites per arm, then nine adjacent pairs with alternating
  first arm. Load, preparation, reset, parity, census, and allocation accounting
  are outside timers.
- **Estimator:** median of the nine paired ratios. The binding statistic is the
  deterministic exact empirical-bootstrap one-sided 95% upper percentile; for
  nine distinct samples it is the seventh ordered ratio.
- **Exit rule:** every hard precondition passes and the upper bound is
  `<=3.0x`.
- **If positive:** implement current production dispatch plus a separate dense
  execution record, preserve source κ and historical identities, then run the
  repository gates and review.
- **If negative:** ship no dispatch/provenance/pin/API and run no additional
  dense clock.

## Frozen measurement identity

- Base: `6fbf718b4115859df6544545a5c43d7638a6ad0a`
- Candidate `gpt2.rs` SHA-256:
  `1a945487fb9ec350fd8f670b8c04dacaf6b66e2339ee96b6b3883082e4de4bf8`
- Harness SHA-256:
  `144fd32d35292a6fd3e949bc7040b7e14c58563bdc31924b66d38bc1db547d89`
- Exact fallback owner: `uor-matmul`
  `b13c98449948174f590e337c4dc25dfc394a07d0`
- Source: `openai-community/gpt2@607a30d783dfa663caf39e06633721c8d4cfcd7e`
- Model: 548,105,171 bytes,
  `blake3:3bca1b7f6c327daecafc16e52d1319375299354e35413fb4e18d24e59b77ce06`
- Config: 665 bytes,
  `blake3:23e4471d412e06128072b559c031207de920b8a56d7108879d4b487c079a310c`
- Host/toolchain: Apple M1, aarch64 macOS, rustc 1.97.1 / LLVM 22.1.6,
  release build, no external codegen/profile overrides.

The later production-integration compatibility repair changes opaque scratch
equality but not model outputs or the frozen measurement. Its final reviewed
`gpt2.rs` SHA-256 is
`d29e209af69e2d79a7aee5ef027029a9d8921645779cd17b97e9dcaae536c384`.

## Whole-model result

Each time is the sum of only the 11 forward calls.

| pair | first | candidate ns | conventional ns | paired ratio |
|---:|---|---:|---:|---:|
| 0 | candidate | 310,361,916 | 438,655,001 | 0.707530782 |
| 1 | conventional | 305,372,418 | 439,392,877 | 0.694987183 |
| 2 | candidate | 306,299,583 | 441,852,623 | 0.693216623 |
| 3 | conventional | 303,427,415 | 439,566,250 | 0.690288244 |
| 4 | candidate | 309,243,207 | 439,152,917 | 0.704181152 |
| 5 | conventional | 302,627,915 | 439,511,791 | 0.688554713 |
| 6 | candidate | 306,728,040 | 440,322,877 | 0.696598010 |
| 7 | conventional | 303,182,542 | 439,220,832 | 0.690273593 |
| 8 | candidate | 308,994,624 | 438,990,000 | 0.703876225 |

- Paired median: **`0.694987183x`**.
- Exact empirical-bootstrap one-sided 95% upper: **`0.703876225x`**.
- Candidate median: `306,299,583 ns/suite` (`27,845,417 ns/token`).
- Conventional median: `439,392,877 ns/suite` (`39,944,807 ns/token`).
- Ratio of arm medians: `0.697097288x` (descriptive, not the gate).
- Structural/preflight census per suite: 539 calls / 1,465,211 lanes;
  1,450,907 fast + 14,304 refined + 0 fallback.
- Timed census: 4,851 calls / 13,186,899 lanes; 13,058,163 fast +
  128,736 refined + 0 fallback.
- Prepared workspace: 157,203,831 bytes; preparation remained outside timing.
- Every preflight/warmup/timed candidate state matched the pinned exact route;
  every timed candidate and conventional call allocated zero bytes.

**Verdict: PASS.** The binding upper `0.703876225x` is below `3.0x`, and every
hard identity, parity, census, overwrite, fallback, and allocation precondition
passed.

## Adopted execution provenance

`DenseOperatorSpec` is model-source execution provenance, not source-file
provenance and not a runtime configuration knob. Current GPT-2 declares
`gpt2-source-dense/2`; the historical sequential binary32 folds remain
`gpt2-source-dense/1`. The final canonical declared digests are:

- dense/1:
  `blake3:b16a2a7f14828f854a7784d33cea9b49631136dbda77491899f2171cec011033`
- dense/2:
  `blake3:3a61d92e61b2a322e086162767173aca8439dffd1ddc7443f1d8b44ee1b1eaf6`

Only learned-absolute attention/1+dense/1 and learned-absolute
attention/2+dense/2 are registered explicit GPT-2 pairs. Dense absence remains
valid for Llama and historical records. Observation identity bundle `/1` bytes
are unchanged when dense is absent; a present dense record selects `/2` between
the attention and trace components. Source and snapshot κs remain addresses of
source bytes and do not move for this executor-only change. Observation,
corpus, report, and artifact CIDs move only when their own bytes move.

Production integration preserves serial, trace, and batched dispatch parity;
validates the pair before mutation at observation, recorded-corpus, cover,
evaluation, certification, API, completion/recovery, and serving boundaries;
and places current dense/2 bundles only in the resolver-owned
`<name>-attention-v2-dense-v2` root. This does not assert that a manifestless
source-tree digest is a `source_manifest.json` κ; that pre-existing provenance
type limitation is tracked separately from #704.
