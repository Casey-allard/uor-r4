# Stage-A K-sweep — G-A FAIL, eval-time widening closed (2026-10-15)

Canonical record for the executed Stage-A kept-cells sweep pre-registered in
[`docs/harness/prep/m1-discrete-rung0-admission-2026-10-15.md`](../../harness/prep/m1-discrete-rung0-admission-2026-10-15.md),
run with the reviewed evaluation-only instrument of
[`docs/labs/stage-a-admission-instrument-2026-10-15/README.md`](../stage-a-admission-instrument-2026-10-15/README.md)
under the Stage-A-only owner GO ([#2029 comment 6192566334](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6192566334)).
The instrument re-scored the four preserved seed-1 quarter checkpoints at
`kept_cells` K ∈ {8..16} on the frozen 14,287-row dev panel — 36 zero-update
full-panel evaluations, no training, no checkpoint mutation.

**Outcome: G-A FAIL (`FAIL_EVAL_TIME_WIDENING_ONLY`), `smallest_passing_k: null`, no instrument fault.**
The gate required some K to reach ≥ 4,894 target-admitted rows at cursor 16,404
(bar (ii) 4,775 + one Wilson half-width 119); the maximum measured is **2,400 at
the ABI-max K=16** — 2,494 rows below the gate. All four K=8 stored-value
controls reproduced the sealed quarter counts **exactly** (`k8_qualification:
ALL_FOUR_EXACT`), so the miss is a real ceiling, not an instrument error.

Verdict as posted by Kimi PM on #2029:
[comment 6195261236](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6195261236).
Run, pod operation and record: Kimi (PM). Codex's INSTRUMENT_FAULT standby was
released with thanks — no fault occurred at any step, no sweep retry was needed,
and `sweep-1` was the only output directory created.

Line: starling read / exact addressed memory · count 1/3 (admission line: eval-time widening closed without headline movement; mechanism open) · M1 reply acceptable 37/232 → 116/232 unchanged (blocked D28).

## Result and G-A read

- **Gate G-A** ([sheet](../../harness/prep/m1-discrete-rung0-admission-2026-10-15.md#bar-pre-registered--the-reachability-gate)):
  at checkpoint 16,404, at least one K ∈ {9..16} shows target-admitted rows
  ≥ 4,894, and the ceiling trajectory across 4,101 → 8,202 → 12,303 → 16,404 at
  that K does not end below 4,894. **Measured: FAIL.**
- At cursor 16,404 the target-admitted ceiling rises monotonically with K —
  1,920 / 2,069 / 2,099 / 2,148 / 2,204 / 2,240 / 2,260 / 2,307 / **2,400** for
  K = 8..16 — but saturates far below the gate. At K=16 every row's full 16-slot
  budget is consumed (`admitted_cell_count_sum` = 14,287 × 16 = 228,592), so
  **K=16 is the ABI maximum and 2,400 is that checkpoint's hard ceiling**, not a
  budget artifact.
- Strict wins fall as K widens at 16,404 (105 → 99, K8 → K16), as the sheet
  anticipated: the best rival distance is a min over admitted cells.
- The **earlier checkpoints reach higher ceilings than the most-trained one**
  (K=16 target-admitted: 3,242 @ 4,101; 3,206 @ 8,202; 2,509 @ 12,303; 2,400 @
  16,404) — training progressed while per-K admission headroom for the frozen
  panel narrowed, the B3 address-objective trade-off the parent sheets named.
- **Read:** the K=8-trained codebook cannot be widened at eval time into enough
  admission headroom (the 4,894-row gate is out of reach at every K, with 2,494
  rows missing even at saturation). Capacity must be *trained in*, not re-scored
  in; that is the R1-narrowed conclusion this instrument was built to buy.

## Qualification: the four K=8 controls (`ALL_FOUR_EXACT`)

Stored-expected-value qualification (no CPU model reference): at K=8 the
instrument must reproduce the sealed quarter metrics exactly. Order below is
top-1 / top-8 / strict wins / admitted strict wins / target-admitted rows.

| Cursor | Expected (sealed) | Reproduced | Authority |
|---|---|---|---|
| 4,101 | 96 / 1090 / 177 / 41 / 2201 | 96 / 1090 / 177 / 41 / 2201 ✓ | [Q1 verification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6174365494) |
| 8,202 | 94 / 967 / 178 / 42 / 2131 | 94 / 967 / 178 / 42 / 2131 ✓ | [Q2 verification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6178914118) |
| 12,303 | 96 / 953 / 214 / 44 / 1954 | 96 / 953 / 214 / 44 / 1954 ✓ | [Q3 verification](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6182703265) |
| 16,404 | 149 / 747 / 105 / 65 / 1920 | 149 / 747 / 105 / 65 / 1920 ✓ | [stage-A sheet](../../harness/prep/m1-discrete-rung0-admission-2026-10-15.md#stage-a-design) |

All four are exact — the instrument is qualified and the instrument's own
`k8_control` field is `EXACT` at K=8 / `NOT_APPLICABLE` above.

## The complete 36-point table

From the sealed `report.json` (the three evaluation-record batches per pair are
row-lossless; `row_columns`/`rows` carry the rows). `strict` = strict-win rows,
`adm_str` = admitted strict wins, `tgt_adm` = target-admitted rows (the ceiling).

| cursor | K | top1 | top8 | strict | adm_str | tgt_adm |
|---|---|---|---|---|---|---|
| 4101 | 8 | 96 | 1090 | 177 | 41 | 2201 |
| 8202 | 8 | 94 | 967 | 178 | 42 | 2131 |
| 12303 | 8 | 96 | 953 | 214 | 44 | 1954 |
| 16404 | 8 | 149 | 747 | 105 | 65 | 1920 |
| 4101 | 9 | 96 | 1040 | 175 | 41 | 2224 |
| 8202 | 9 | 95 | 965 | 177 | 42 | 2186 |
| 12303 | 9 | 96 | 982 | 159 | 36 | 2175 |
| 16404 | 9 | 153 | 738 | 103 | 63 | 2069 |
| 4101 | 10 | 93 | 1022 | 175 | 41 | 2290 |
| 8202 | 10 | 94 | 945 | 176 | 42 | 2242 |
| 12303 | 10 | 97 | 970 | 158 | 36 | 2205 |
| 16404 | 10 | 152 | 686 | 100 | 62 | 2099 |
| 4101 | 11 | 91 | 1030 | 174 | 43 | 2417 |
| 8202 | 11 | 85 | 904 | 175 | 42 | 2285 |
| 12303 | 11 | 95 | 950 | 157 | 36 | 2237 |
| 16404 | 11 | 151 | 674 | 100 | 62 | 2148 |
| 4101 | 12 | 93 | 995 | 131 | 46 | 2463 |
| 8202 | 12 | 87 | 889 | 132 | 45 | 2345 |
| 12303 | 12 | 95 | 939 | 157 | 36 | 2270 |
| 16404 | 12 | 150 | 655 | 100 | 62 | 2204 |
| 4101 | 13 | 93 | 1006 | 119 | 46 | 2539 |
| 8202 | 13 | 87 | 899 | 132 | 45 | 2384 |
| 12303 | 13 | 95 | 909 | 157 | 36 | 2323 |
| 16404 | 13 | 148 | 631 | 100 | 62 | 2240 |
| 4101 | 14 | 126 | 1151 | 107 | 46 | 2938 |
| 8202 | 14 | 88 | 898 | 121 | 46 | 2533 |
| 12303 | 14 | 86 | 871 | 155 | 36 | 2367 |
| 16404 | 14 | 144 | 611 | 100 | 63 | 2260 |
| 4101 | 15 | 137 | 1164 | 101 | 59 | 3193 |
| 8202 | 15 | 119 | 1038 | 108 | 45 | 3023 |
| 12303 | 15 | 88 | 870 | 112 | 39 | 2407 |
| 16404 | 15 | 145 | 605 | 99 | 63 | 2307 |
| 4101 | 16 | 136 | 1160 | 100 | 58 | 3242 |
| 8202 | 16 | 138 | 1083 | 100 | 60 | 3206 |
| 12303 | 16 | 88 | 879 | 111 | 40 | 2509 |
| 16404 | 16 | 145 | 586 | 99 | 63 | **2400** |

## Checkpoints and data

All four inputs authenticated as `PINNED_CHECKPOINT` against the sheet pins
before any CUDA context was created (`cuda_context_created: false` on all four
`check` results). Checkpoints restored from
`icloud:UOR-R4/results/claude/rung0-completion-seed1-20261015.tar` (md5
`2ddcd977…`; inner tgz sha256
`300246ab16a1e0adaba547d43590e08264912f59ee29f35d7e024d77591e63a1` — exact pin)
to pod `/workspace/uor-r4/rung0-completion-20261014/seed1-attempt-1/`
(240 files); staged corpora verified against `data-pins.sha256` (5/5 OK).

| cursor | checkpoint sha256 | accepted updates | status |
|---|---|---|---|
| 4,101 | `a5a22dce8a174bbfe368cd8d166170a6c34bf59137b1418943a9d0422cae4cdd` | 315 | PINNED_CHECKPOINT ✓ |
| 8,202 | `dde889aa6137c0e28e18dd5ae99a823b6850342286a0d065bdaa82de49b05822` | 366 | PINNED_CHECKPOINT ✓ |
| 12,303 | `102beed4608f1ddd70d68039bdae8d9372b3c05de2dbc4bc98b96d7595e40982` | 384 | PINNED_CHECKPOINT ✓ |
| 16,404 | `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf` | 407 | PINNED_CHECKPOINT ✓ |

Each derived device kept the original `original_driver_source_sha256`
(`cb090e85…`, the existing operational difference disclosed in the D-A record);
parameters, seed, epoch, cursor, corpus bindings and pins were unchanged.

## Run, cost and toolchain

- Pod `vast-56109378`, Vast Czechia (CZ), **$1.352/h**, image
  `runpod/pytorch:1.0.2-cu1281-torch280-ubuntu2404`, 1× RTX 5090 (driver
  610.57.04, cc 12.0). **Elapsed 913 s (15m13s); est. spend $0.343** — under the
  recorded < $0.50 owner ceiling. Pod released and `down`ed, instance deleted;
  the `/workspace` auto-archive holds the job logs (84 kB).
- Toolchain confirmed exact on the pod: **nvcc V12.8.93** (release 12.8),
  **rustc 1.97.1**, `UOR_NVCC_ARCH=compute_120`; bootstrap D29 PASS in 104 s at
  source sha `abdf431edf80a80851d0d10a767886ae22502a6c` (the #2508 merge), clean
  tree. Pod build of the instrument: 20.04 s, no patch; new executable
  `712cd0cc…` — the host executable hash was not reused.
- Static pins all match the host record; the compiled identity exactly
  reproduces the checkpoint-bound PTX/source/geometry pins (`931a511b…`,
  `e1836149…`, `46ddbb76…`).

## Identity (verbatim, from the pod)

```json
{
  "compiled": {
    "geometry_sha256": "46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327",
    "kernel_sha256": "931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b",
    "source_sha256": "e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a"
  },
  "cuda_context_created": false,
  "cuda_enabled": true,
  "driver_source_sha256": "70b45863860c25cf3579e76367b785ea4830edbc50e3902ef4a5ca2335bf7676",
  "executable_sha256": "712cd0cc7e0d7934c136575b0f4bf6cc9bccd9b5709e8eb9b722e5d92b5efc02",
  "instrument_lock_sha256": "72276998293683e471cdce1529f5ba799759731967eea66d41b292022fc0d6ae",
  "instrument_manifest_sha256": "6cfc87a9d736c3767ba39a82730d0fef580959e81ea86672980459078b8c23c1",
  "instrument_source_sha256": "719f38d990ffad9053ee5d1b2c6bbfafabdb7cd3274943099395d3ffdf58bb84",
  "model_updates": 0,
  "report_output_source_sha256": "cda30f9ba4ef8ff3e245ae352cd28932f9a24ab78598f683607e6996598fa810",
  "schema": "uor-r4.stage-a-instrument/1",
  "training": "HELD"
}
```

## Verifications

- **Seal:** manifest `uor-r4.report-manifest/1` lists 38 files (`attempt.json` +
  36 evaluation records + `report.json`); the on-disk set of 39 is exactly those
  plus the manifest — nothing missing, nothing unlisted; **all 38 blake3 hashes
  and byte counts verified PASS** (independent Python blake3).
- **Per-row monotone admission: PASS** — independently re-checked from the 36
  records across 4 checkpoints × 14,287 rows × K=8..16 (a target admitted at K
  is admitted at K+1; the instrument also enforces this internally).
- **Zero-update flags:** `snapshot_unchanged: true`, `updates_applied: 0`,
  `updates_applied_by_evaluation: 0` on every record; `k8_control` EXACT at K=8 /
  NOT_APPLICABLE above.
- **Preservation integrity:** pod → laptop full-tree sha256 match on all 43 files
  (39 sealed + `identity.json` + 3 job logs); `derived_record_bytes`
  33,627,092 (< 49 MiB record cap) — the real output size, measured.
- No instrument fault at any step; no sweep retry was needed; `sweep-1` was the
  only output directory created.

## Archive and preservation

- **iCloud:** `icloud:UOR-R4/results/kimi/stage-a-sweep-1-20261015.tar` —
  33,802,240 bytes, md5 **`b274beb321ce9232bd3690659fee7890`**, MD5 round-trip
  verified by cloud-store; index entry written. Contains `sweep-1/` (the sealed
  root, unmodified) + `identity.json` + `logs/{build-identity,check,sweep}.log`.
- **Laptop copy (outside the repo):** `~/uor-r4-local/stage-a-sweep-1-20261015/`.
  The shared-input archive is staged at `~/uor-r4-local/stage-a-staging/` (outer
  tar + extracted inner tgz, checkpoint shas reverified).

## Scope (R1) and D22 outcome

- **R1 scope:** this closes **eval-time widening of the K=8-trained codebook
  only**. The sweep re-scores codebooks *trained at K=8*; a K>8-trained lineage
  sees a different objective (the address hinge ranks rivals among the K admitted
  cells), so its ceiling is not bounded by this instrument. K>8-trained lineages
  remain an untested, named successor (Stage-B territory).
- **D22:** a configuration closure, never a mechanism negative. Stopping the
  mechanism needs the owner. The named policy-level successors stay recorded, not
  adopted: address metric, selection rule, objective weights (current 8·token +
  32·address).
- No automatic closure; nothing in this result releases Stage B. Training, M2
  and the seed-2 line remain HELD.

## Next (pre-registered outcome path)

Per the sheet's G-A FAIL branch, the PM commissioned the redirect pre-registration
on the admission policy/objective — [#2028 comment 6195270941](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6195270941)
— co-designed by **Antigravity + Codex** (`prep new -m m1`), covering the three
named levers (address metric, selection rule, objective weights). Requirements
recorded there: design from the sweep's actual evidence (ceilings fall as
training progresses at fixed K=8; strict wins fall as K widens — any proposal
must explain both curves); if the design converges on training in capacity, size
the K>8-trained lineage (the Stage-B amendment skeleton exists in the current
sheet); source-only design work — no pod compute, no training; TEST FITNESS goes
to Claude when the sheet is ready. Owner holds: training, M2, Stage B.
