# REPORT: generated-state admission headroom, deepseek-134

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `70fad10ad`.

## What did not work

- **The existing K=16 admission ceiling did not unlock writer training.**
  Seed 1 exposed only **17** positive-margin training decisions, down from 30
  at K=8. Seed 2 exposed **52**, up from 32. The readiness bar required 640.
- **Same-cell control coverage remained insufficient.** K=16 produced **6**
  same-cell pairs on seed 1 and **17** on seed 2, against the required 64.
- **Held-out exposure was too small.** K=16 added 32 admitted held-out targets
  on seed 1 and 49 on seed 2, against the required 128.
- **K=16 did not beat the equal-budget random-cell control on target
  diversity.** It admitted fewer held-out targets than random extras on seed 1
  (85 vs 88) and did not exceed their positive-margin target-cell count on
  either seed.
- **Seed 1 teacher-forced top-1 fell.** K=8/K=16 read 438/420 of 3,765. Seed 1
  generated depth rose from 0 to 51, but its K=16 replies collapsed to one
  distinct non-empty text with 103/103 repeated 4-grams.
- **The registered bar failed on both seeds.** This closes the fixed
  existing 8→16 admission-budget configuration; it does not reject writer
  geometry or intentionally widened cell sets generally.

## What changed

Added a CPU-only admission-headroom path:

1. ranks the reference target cell among all 256 writer cells;
2. reports admission, positive margin and writer top-1 at
   K=1,2,4,8,12,16;
3. compares against eight deterministic non-top cells added to K=8;
4. builds a K=16 artifact by changing only the shape header and preserving
   every table payload byte;
5. runs complete K=8 and K=16 generated trajectories independently; and
6. audits both generated token sequences against integer serving and repeats
   seed 1 exactly.

The K=8/K=16 admissions are not estimated from one another's queries.

## Measurements

| Seed | K=8 eligible | K=16 eligible | K=16 pairs | Held-out add | Random add |
|---:|---:|---:|---:|---:|---:|
| 1 | 30 | **17** | 6 | 32 | 88 |
| 2 | 32 | **52** | 17 | 49 | 81 |

K=16 generated depth was 51/973 decisions on seed 1 and 4/973 on seed 2.
Teacher-forced top-1 was 438/420 and 385/415 for K=8/K=16.

Every arm and seed retained exact CPU/integer parity:

- seed 1 and its repeat: 4,328 positions, 1,107,968 Q12 coordinates, 4,328
  token decisions, and 34,624/69,248 kept cells at K=8/K=16;
- seed 2: the same positions, coordinates and decisions, with the same kept
  cell counts.

All mismatches were zero. Artifact hashes:

| Seed | K=8 | K=16 |
|---:|---|---|
| 1 | `ed06a88b…d7daffc` | `c198c5ec…97aa32e1` |
| 2 | `2c071845…eaf7e842` | `60fc6c43…4ea1ddbb` |

The K=16 artifacts differ from K=8 only in shape/header alignment; their
table payload bytes are identical.

The fresh build took **18 s**, peaked at **1,203,564 KiB** and occupied
**263,422,235 bytes**. Each 32-worker census took **10.90 s** at
**292,952-294,376 KiB**. The binary SHA-256 is
`2190b2b27632bc69873e65f7ec7e0b23ad2617fc722b159424d3bff9995aa9a6`.
The retained pipeline is **14,065,758 bytes**.

Compact evidence hashes:

- seed-1 report:
  `b6822932ef6b3b580713a2f3e901053138e8211f0b65e9a2478c8775d6c3c0b9`;
- seed-2 report:
  `6e457c06bed758df52003f926cf28cbd2bf656f6025e574274c2b22a5f13008a`;
- pipeline `SHA256SUMS`:
  `af6e2f077e38948d583f369f78a5d375805f510742099601b6c765e700938ba0`.

## Checks run at the head

- First FIT and code reviews rejected measuring K=16 on K=8-generated queries;
  the driver now uses complete independent K=16 trajectories and includes a
  regression where the two trajectories differ.
- A fresh review then returned FIT at `d5d742c19`, and a separate fresh
  pre-run review returned GO at `a2c611773`.
- Full `uor-r4-cpu-train` release suite: **229 library and 56 binary tests
  passed, 0 failed; 4 library tests were expected ignores**.
- Focused self-prefix tests: **16/16**; admission driver tests: **2/2**.
- Strict no-deps all-target CPU Clippy with `-D warnings`, formatting,
  prep-shape and `git diff --check`: passed.
- All five public input hashes matched. The implementation tree matches pin
  `5565bf9b432b4937fb4de8d532321f20966bee1d`.
- Both seeds and the repeat exited 0, normalized JSON and artifacts matched,
  and recursive pipeline checksums passed.

## Skills used

No system-provided Codex skill applied. Read-only subagents performed the
next-mechanism audits, two FIT/code reviews and the pre-run review.

## Still open

- Direct generated-query/admission training is now the next plausible
  mechanism; another post-admission writer retune would not address the
  measured fault.
- Complete replies, fresh phrasing, useful local chat, serving integration
  and GPU qualification remain open.
- Exact-head delivery review, protected merge and fresh-main verification.

## Plain summary

The existing K=16 ceiling does not expose enough admitted training states.
Seed 1 falls from 30 eligible rows at K=8 to 17; seed 2 rises from 32 to 52,
both far below 640, with only 6/17 same-cell pairs. K=16 preserves exact
serving parity and table payloads, but fails the registered readiness and
non-loss bar. The next move is direct generated-query/admission training.
