# W0 stored-checkpoint ALIAS CENSUS — 18 October 2026

**Scope:** host-only, read-only analysis of preserved discrete-model checkpoints.
No GPU, no pod, no model execution, no GitHub post, no repository mutation.
Scratch work lives only in
`/Users/casey.allard/uor-r4/.worktrees/kimi-pm/local/alias-census-20261018/`.

---

## 1. Verdict

**The non-injectivity is real and, if anything, worse than projected. The
inference drawn from it — that Bar (i) = 1,941 is arithmetically out of reach —
is not.**

Two separate quantities were conflated in the architect's estimate:

1. **Distinct full vectors (token identities).** Measured: **16 at checkpoint 0**,
   **242 treatment / 251 control at Q1 (4,096 batches)**, and **264 at the only
   actual full-dose checkpoint (Q4, 16,404 batches)**. That run accepted **391
   VectorBlock moves out of 16,384 attempts (2.4 %)** and produced 0.63 new
   vectors per accepted move, so its hard upper bound was 407. The rows_v1 Q1
   acceptance rate, projected arithmetically to 16,404 batches, would bound
   ≈ 1,602 vectors — still below 1,941. No rows_v1 Q4 checkpoint exists.
2. **Bar (i) = 1,941.** The sheet defines it as **top-1 correct over 14,287
   frozen DEV rows**, not as a count of tokens. A single predictable token can
   carry many rows, and this panel is extremely concentrated: 674 distinct
   target tokens, with token 1 appearing 1,226 times and token 16 appearing
   1,018 times.

**Alias ceiling on DEV rows** (rows whose target is the lowest-ID member of its
alias group — the only member that can ever win the evaluation tie-break):

| checkpoint | alias ceiling | ≥ 1,941? |
|---|---:|---|
| checkpoint 0 (16 reps, 4 distinct target tokens) | **1,986 / 14,287** | yes, by 45 rows |
| Q1 treatment (242 reps, 131 target tokens) | **8,908 / 14,287** | yes, 4.6× |
| Q1 control (251 reps, 133 target tokens) | **8,879 / 14,287** | yes, 4.6× |
| Q4 (264 reps, 143 target tokens) | **8,948 / 14,287** | yes, 4.6× |

So, per the task's condition: **yes, ≤ ~1,820 distinct vectors at Q4 is
confirmed (264)** — the distinct-vector ceiling is real and sharper than the
architect projected. But that ceiling **does not bound Bar (i)**, because Bar (i)
counts rows. Bar (i) is arithmetically reachable under aliasing at every located
checkpoint. The token count being below 1,941 does not bound a row-count bar:
the same 264 tokens cover 8,948 DEV rows.

**What is actually binding.** The separate admitted-cell ceiling reported for the
same Q4 artifact is **1,920 target-admitted rows < 1,941** (K8 completion run).
That is 21 rows short, is independent of aliasing, and is the already-documented
arithmetic unreachability. Under the registered K16 redirect the admission gate
targets ≥ 4,894 rows, so that ceiling is designed to lift. Aliasing gives no
reason to stop the K16 spend.

**What aliasing does cost.** Only **143 of the 674 distinct DEV target tokens**
are predictable at Q4 (131 at Q1, 4 at checkpoint 0). Any claim of broad token
coverage is capped at **264 token identities** in this configuration. On a
uniform-target panel the Q4 alias ceiling would be only ≈ 921 rows
(264 × 14,287 / 4,096) — below the bar. The bar survives here because the frozen
panel is concentrated, not because the codebook is expressive.

---

## 2. Per-checkpoint measurements

`N` = distinct full vectors (4 blocks × 8 trits). Cell = `root(first_block)·16 +
root(last_block)`. "Tie mass" = tokens sharing a cell with ≥ 1 other token;
"pairs" = Σ n(n−1)/2 over cells.

| Checkpoint | Batches | Accepted vector moves | N | Alias-group histogram (size:groups) | Unique-vector tokens | Occupied cells | Tokens in tied cells | Cell pairs | Max cell | Multi-vector cells | DEV alias ceiling |
|---|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| rows treatment cp0 | 0 | 0 | **16** | 256:16 | 0 | **16** | 4,096 | 522,240 | 256 | 0 | 1,986 |
| rows control cp0 | 0 | 0 | **16** | 256:16 | 0 | **16** | 4,096 | 522,240 | 256 | 0 | 1,986 |
| rows treatment Q1 | 4,096 | 396 | **242** | 1:216, 2:9, 3:1, 237:2, 239:3, 240:2, 241:2, 242:1, 243:1, 244:4, 245:1 | 216 | **169** | 3,985 | 469,372 | 247 | 50 | 8,908 |
| rows control Q1 | 4,096 | 411 | **251** | 1:225, 2:10, 236:2, 237:1, 238:2, 239:1, 240:3, 241:1, 243:3, 244:1, 245:1, 248:1 | 225 | **173** | 3,988 | 467,252 | 250 | 57 | 8,879 |
| rung0 completion Q4 (seed 1) | 16,404 | 391 | **264** | 1:237, 2:9, 3:2, 230:1, 234:1, 235:1, 237:1, 238:1, 239:3, 241:1, 242:3, 243:2, 244:1, 247:1 | 237 | **189** | 3,958 | 467,269 | 248 | 45 | 8,948 |
| K16 restart-5 cp0 (seed 1) | 0 | 0 | **16** | 256:16 | 0 | **16** | 4,096 | 522,240 | 256 | 0 | 1,986 |

Additional facts that hold at every located checkpoint:

- **Every block is still a root vector.** Distinct values per block position are
  `[16, 16, 16, 16]`; no block has left the 16-root set. The vector space is
  therefore the 4-tuple space `16⁴ = 65,536`; observed occupancy is 264 at Q4.
- **No token has an absent cell** (`root_id` never returns ABSENT). Cell counts
  are exact, not lower bounds.
- **Checkpoint 0 is 16 alias groups of exactly 256 tokens**, each cell also
  exactly 256 tokens. No token has a unique vector, so at initialization no token
  has an individual address identity.
- The alias structure has two regimes at Q1/Q4: ~216–237 tokens with unique
  vectors, and 16 large residual groups of 230–248 tokens (the un-moved init
  groups). Growth is driven by the small unique population, not by splitting the
  big groups.

### Growth curve (distinct vectors / occupied cells)

| Batches | 0 | 256 | 512 | 768 | 1024 | 1280 | 1536 | 1792 | 2048 | 2304 | 2560 | 2816 | 3072 | 3328 | 3584 | 3840 | 4096 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| treatment | 16/16 | 40/37 | 73/61 | 93/79 | 116/99 | 132/109 | 158/123 | 174/134 | 186/141 | 200/147 | 205/151 | 215/154 | 225/158 | 226/159 | 232/162 | 235/163 | 242/169 |
| control | 16/16 | 40/37 | 72/60 | 96/83 | 112/93 | 131/105 | 162/122 | 176/132 | 190/139 | 202/144 | 209/149 | 220/156 | 229/161 | 232/162 | 240/167 | 247/171 | 251/173 |

The curve is sub-linear and saturating: +226 vectors over the first 4,096
batches for 396 accepted moves (0.571 new vectors per accepted move); Q4 reaches
264 after 391 accepted moves (0.634 per move). The architect's "one accepted
move = one new vector, ~1,800 moves at full dose" projection therefore
overstates the count by ~6.8× at the only actual Q4 artifact; even its own
arithmetic bound (`16 + 391 = 407`) is 4.4× below the projection.

---

## 3. Artifact identities and lineage

All file hashes were recomputed on the host before parsing. The Q1 hashes match
the values published in `docs/labs/attention-rows-q1-diagnosis-2026-10-18/README.md`;
the Q4 hash matches the value published in the runbook and the 18 October
domain diagnosis; the checkpoint-0 hash matches the propose-subphase diagnostic
pin.

In the rows paths, `…/` is
`/Users/casey.allard/uor-r4/.worktrees/kimi-pm/local/alias-census-20261018/extract/workspace/attention-dose2-rows/reports`.

| Artifact | Path (host) | SHA-256 | Bytes | Inner `model_sha256` |
|---|---|---|---:|---|
| rows treatment cp0 | `…/seed1-treatment/attempt-0001/checkpoint-00000.json` | `e702bd13295766ffe49f48b74fab834fa5718e78dfd5117f04ddd1af85b68dec` | 6,879,215 | `c536de05930d98576b016b56fc58cf181ca7888bef57b5ddefcb250d8b92953a` |
| rows control cp0 | `…/seed1-frozen-zero/attempt-0001/checkpoint-00000.json` | `1c01fed3a62bcad4679159648fab5f176af2ad987ff00c64d43b001f4b3d508c` | 6,879,303 | `c536de05930d98576b016b56fc58cf181ca7888bef57b5ddefcb250d8b92953a` |
| rows treatment Q1 | `…/seed1-treatment/attempt-0001/checkpoint-04096.json` | `e0549a767d64f5d1a100c2993d7914665ae737dc7685b327512f76c0d70bd640` | 6,879,427 | `7360d2d26a1bf88814d5d8ccce1dce78d51504041ba3cd032490d37f07f73f45` |
| rows control Q1 | `…/seed1-frozen-zero/attempt-0001/checkpoint-04096.json` | `1d1d47aa1fbec6f89fa5fc4f5cd3bab5832fd77093d6ca96a745e17b2a340943` | 6,879,540 | `25a8bd301b281fc349b707d7546cf8659afa65542564c3b4c7eb9cb3db529ef8` |
| rung0 completion Q4 | `/Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/q4-original-authenticated/checkpoint-16404.json` | `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf` | 3,138,786 | `49fa448cb3349ac5711966082a9c53918a693d45e11f282221d240cbb463fa73` |
| K16 restart-5 cp0 | `/Users/casey.allard/uor-r4-local/codex-retained-checkpoint-inputs-20261017/checkpoint0-timing-snapshot.json` | `9a76743e401759b7d3aabc708ab0a26a9470ea474dc140f328c7b1aab59e49e4` | 934,895 | — (bare `Snapshot`) |
| source archive | `~/uor-r4-local/attention-dose2-rows-compressed-data1/payload.tar.gz` | `5db64397ccb1ccdba1ab9e8b851229a7f0ecaf53c2a8541aa104658ecd571560` | 90,781,172 | — (302 verified members, 940,029,685 uncompressed bytes) |
| DEV tokens | `/Users/casey.allard/uor-r4/.worktrees/kimi-pm/local/sheet-check/ft/dev/tokens.u16` | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` | 85,776 | — |
| DEV mask | `…/ft/dev/response_mask.u8` | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` | 42,856 | — |

Lineage:

| | rows cp0 / Q1 (both arms) | rung0 completion Q4 | K16 restart-5 cp0 |
|---|---|---|---|
| Wrapper schema | `uor-r4.attention-learning-checkpoint/2` | `uor-r4.completion-checkpoint/1` | bare `Snapshot` |
| Snapshot schema | `uor-r4.discrete-spiralcore-fbs/2` | same | same |
| Seed / epoch | 1 / 0 (cp0), 1 / 631 & 558 (Q1) | 1 / 407 | 1 / 0 |
| Cursor | 0 and 4,096 batches; accepted 0 and 631 / 558 | 16,404 batches; 407 accepted; next target 1,049,856 | — |
| Read | Relation, kept 8, sweeps 1, `active_leaf_v1` | Route, K8 | Route, K8 |
| Schedule | `interleaved_read_every8_v1` | `canonical_vector_then_read_v1` | n/a |
| Objective | `nearest_address_margin_v1` (default) | same | same |
| Driver source | `9a1b121b0d8896bdba85baed40d95834f01bdb9b22d48167c374e34de8fa069e` | `cb090e85e78805acd2b2347373340168a9e2f87aaf72a1dcb42f091e9ff1f7b4` | — |
| Model source | `030367eb3605d181e087305986a13f9137fc0acb376bfd8576f95aa59f91f9c9` | `e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a` | `f6c4cc9f7a842ecb7a58fb569dcb7af61f84eb533cb4a059ee614be9c0f18fa7` |
| Kernel | `5cd0e73241fd895f2dd0004c83af3d1eb9f89e00ec2e933c1acc44fc7127bfd6` | `931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b` | `1a21d6c3e2773a427c14ebff9499e03386147ab41b3ce413b7b5c9123925fd28` |
| Geometry | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` | same | same |
| Counters | vector 3,584 attempted / 396 or 411 accepted | vector 16,384 / 391; route 20 / 16 | — |

The Q4 file's stored `model_sha256` is `49fa448c…`, not the runbook's
`29ba5517…`; the 18 October domain diagnosis in the same directory establishes
that the runbook value is a hash-domain error and that `49fa448c…` is the correct
Domain-A value. The file hash `4477eb5c…` is the binding identity here.

---

## 4. Method and exact commands

### Stored format (inspected, not assumed)

- Q1/Q4 files are JSON checkpoint wrappers; the model is
  `.checkpoint.model` (`DriverCheckpoint` → `model`).
- `model.table` is a flat `i32` array of `vocabulary × blocks × 8` =
  **131,072 values** = 4,096 tokens × 4 blocks × 8 trits in `{−2, 0, 2}`.
- A token's full vector is the **32-int tuple** `table[(t·4+b)·8 .. +8]` for
  `b = 0..3` — not 4 ints, and not a nibble code. Documented from
  `discrete.cu:25–31` (`table_block`) and `device.rs:386–408` (`Snapshot.table`).
- `root_id` matches a block against the 16 fixed 8-int roots
  (`discrete.cu:283–285`); `token_cell` returns `root(first)·16 + root(last)`
  (`discrete.cu:286–289`). `address_distance` uses the same first/last roots
  (`discrete.cu:274–275`). 256 possible cells.
- The 16 roots were read from the generated `geometry.rs` (`build.rs:330`) and
  independently validated two ways: they are exactly the 16 distinct block rows
  observed at checkpoint 0, and the computed cell counts reproduce the recorded
  values (**16** at checkpoint 0, **189** at Q4).
- Initialization (`discrete.cu:348–356`): `state = seed ^ block ^ 0x9e3779b9`,
  xorshift, `table[i] = ROOT + (state & 15)·8 + (i & 7)`. One 4-bit root draw per
  8-int block, so each block is exactly one root; this is the source of the
  non-injectivity the architect identified.
- Evaluation tie-break (`discrete.cu:480–487`): only tokens in the ≤ 16 admitted
  cells are ranked; equal `token_distance` is broken by **smaller token ID**.
  Therefore only the smallest-ID member of an alias group can ever be top-1;
  all other members are permanently unrankable.
- DEV eligibility (`corpus.rs:206–213`): positions `p > 0` with `token ≠ BOS` and
  `mask = 1` → 14,287 rows over 674 distinct target tokens.

### Commands

```sh
# artifact identity (examples; all files hashed this way)
shasum -a 256 payload.tar.gz \
  seed1-treatment/attempt-0001/checkpoint-04096.json \
  seed1-frozen-zero/attempt-0001/checkpoint-04096.json
shasum -a 256 /Users/casey.allard/uor-r4-local/codex-cf1-k16-rows-fastpath/q4-original-authenticated/checkpoint-16404.json
shasum -a 256 /Users/casey.allard/uor-r4-local/codex-retained-checkpoint-inputs-20261017/checkpoint0-timing-snapshot.json
shasum -a 256 .../sheet-check/ft/dev/tokens.u16 .../sheet-check/ft/dev/response_mask.u8

# extract only checkpoint members to scratch (originals untouched)
tar -xzf payload.tar.gz -C $SCRATCH/extract \
  'workspace/attention-dose2-rows/reports/*/attempt-0001/checkpoint-*.json'

# census over 130 rows checkpoints (0..4096 step 64, two arms) + Q4 + K16 cp0
python3 census.py inputs.json results.json     # -> results.json, census.csv

# alias ceiling against the frozen DEV panel
python3 dev_ceiling.py dev-inputs.json dev-ceiling.json
```

Scratch outputs: `results.json` (full per-checkpoint records), `census.csv`
(132 rows), `dev-ceiling.json`, `inputs.json`, `dev-inputs.json`, plus
`extract/` (853 MB of extracted checkpoint JSON; removable).

### Cross-checks that passed

- Q1 treatment/control files hash to the exact values published in the Q1 record.
- Q4 file hashes to the exact value published in the runbook and domain diagnosis.
- Checkpoint-0 snapshot hashes to the exact value pinned by the propose-subphase
  diagnostic (`docs/labs/propose-subphase-diagnostic-run-2026-10-16`).
- Computed occupied cells = 16 at checkpoint 0 (restart-5 record: "checkpoint 0
  occupies exactly 16 cells") and = 189 at Q4 (runbook: "189 occupied cells").
- Archive extraction reproduces the published Q1 member hashes, proving the
  extraction path is byte-faithful.

---

## 5. Limitations

1. **Q4 is a different run.** The only full-dose checkpoint obtainable host-side
   is the seed-1 rung0 completion artifact (Route read, K8, source `e1836149…`,
   407 accepted updates). The `rows_v1`/K16-R configuration has **no Q4
   checkpoint**: its run stopped at 4,096 batches (Q1) and no K16 training has
   executed. Q4 numbers therefore characterize the closest actual full-dose
   artifact, not the redirect arm.
2. **Seed 1 only.** Seed-2 configs exist in
   `codex-retained-checkpoint-inputs-20261017/configs/`, but no seed-2 stored
   checkpoint or snapshot was found host-side → **NOT_OBTAINABLE** for seed 2.
3. **Dose-1 partial checkpoints not obtained.** The 904/1,065-batch strategic-stop
   partials were not present as raw host-side files; the later rows Q1 (4,096)
   supersedes them for this question. Reference:
   `docs/labs/attention-dose1-strategic-stop-2026-10-18/`.
4. **The alias ceiling is an upper bound.** It ignores cell admission and
   query-specific distance ties. It says what aliasing alone forbids, not what
   the model achieves. The recorded top-1 at Q4 is 149/14,287, far under the
   8,948 alias ceiling — ranking quality, not aliasing, is the observed gap.
5. **The verdict is panel-specific.** The frozen DEV panel has 674 distinct
   targets and two tokens carry 2,244 rows. On a uniform-target panel, the Q4
   alias ceiling would be ≈ 921 rows (< 1,941). The bar remains reachable on the
   registered panel, not in general.
6. **No model execution.** The 1,920 target-admitted figure is quoted from the
   published seed-1 completion record, not recomputed here (it needs query-level
   admission).
7. **No cleanup performed.** The extracted 853 MB scratch copy is left in place;
   the archive and every checkpoint original are unmodified. Nothing was deleted.
