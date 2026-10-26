# Prep sheet: the residual check, the D11 panel against the artifact (d11-exporter-float-trunk, claude-132), 2026-10-26

- Item: [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). ORDER: [claude-132](../../loop/tasks/d11-exporter-float-trunk/claude-132/ORDER.md).
- This is the first step of claude-131's §4.4 pivot note, set by its [decision review](https://github.com/UOR-Foundation/uor-r4/issues/2820#issuecomment-6419387874).
- Run: one job on one leased RTX 5090, `spiralcore-fbs-sheet --mode d11-residual` over six saved models. No training, and no evaluator re-run.

## 1. Question

**Does each exported artifact compute its own D11 forward?**
- Three configurations each passed the loss half and missed the top-1 half:
  - claude-129, the plain-trained trunk of claude-126, exported directly;
  - claude-130, trained in the D11 mode;
  - claude-131, dual-forward training.
- On claude-131's seed 2, the export-mode panel (E) is 126 top-1 rows from the artifact (A). E does not snap the transition or round the tables, so it is not the artifact's emulation.
- The emulation is the D11 panel (D, `evaluate_dev_panel_d11`). None of the three runs paired D with A.
- **If A departs from D, the exporter, or the evaluator it was read by, is at fault.** That would be an instrument defect bearing on all three results (§4.2, L-007), not a mechanism result.

## 2. Arms

There is one model per (configuration, seed), six in all, each scored five ways on the 14,287-row `ft/dev` panel:
- **F:** the plain-float panel. It must reproduce the stored predictions.
- **E:** the export-mode panel, on the tables as trained.
- **D:** the D11 panel. The tables are rounded as exported; the gates and the transition snap to the 120 icosians; the query and the writer are at Q12.
- **H:** the D11 writer head at Q12, applied to the artifact's own Q12 queries.
- **A:** the artifact's token at each scored offset, read from the stored `d11-fbs-eval run --verify` output. G = H = R held there at all 82,176 positions.

D and H share one head (`write_q12_rows`), so where they differ, it is because the queries differ. D's queries and A's are compared code by code, as the Q12 drift.

## 3. Bar and decision table

**The question is identity: does A compute the same forward as D?** It is not whether A scores as well as D. A hit-count test alone would miss a departure that changes many tokens on rows that both D and A get wrong, which is about 86% of the rows. So the rule is row by row.

**The classification rule, per model** (`classify_residual`; every instrument check of section 4 green first). Let r = 14,287 rows, d_H = the rows where H and A differ, and d_D = the rows where D and A differ.
- **Faithful:** d_H = 0, **and** d_D ≤ 3% of r (428 rows), **and** D against A passes the paired test, |b − c| ≤ F_I = max(24, ⌈2√(b + c)⌉). Here b counts the rows D hits and A misses, and c the reverse.
- **Departure:** d_H > 0, **or** d_D > 10% of r (1,429 rows or more), **or** D against A fails the paired test.
- **Indeterminate:** anything else, that is d_H = 0 and the paired test passed, but d_D between 429 and 1,428 rows.

**Why these numbers** (calibrated on claude-127's host checks, before this run):
- **d_H = 0.** H reads A's own queries through the same head, so any difference is the head's.
- **3%.** It is three times the integer path's measured residual against its f64 reference on the Stage 4b snapshots: 4 to 5 of 512 tokens at 6 layers, about 1%.
- **10%.** On claude-127's tiny-gain control, its norm-bias defect changed 91 of 512 tokens (18%). That defect still left 509 of 512 queries within 2 LSB, so query drift alone would not have caught it. The drift is therefore a reading here, not the rule.

| Outcome | It means | It decides |
|---|---|---|
| **Faithful on all six** | Each artifact computes its D11 forward: its head is identical, and its tokens differ from D's on at most three times the integer path's measured residual. The three FAILs' top-1 gaps then lie between the plain-float forward and the D11 forward itself: the gate and transition snaps, the Q12 query and the table rounding. | The exporter is cleared as the cause. The three negatives stand as mechanism results of their configurations. The item parks behind the owner's reading of §3.7 (#2028), and any decisive run targets the float-to-D11 forward gap in training. |
| **A departure on any model** | That artifact computes something measurably different from its D11 forward. | An exporter or evaluator defect, so an instrument repair item is opened. The readings say whether it sits in the head (d_H) or the trunk (D against H, and the drift). After the repair, the affected artifacts are re-exported and re-scored under the unchanged bar. No negative stands on an affected artifact until then. |
| **Indeterminate on any model, with no departure** | A's tokens differ from D's more than the measured residual, but not as a different forward does. | An instrument investigation of that model's trunk, led by the drift readings. It is never a mechanism result, and the item does not park until it is settled. |
| **An instrument check fails** | The model, its inputs, or the stored run is not the one claimed. | Not counted. That check is repaired first. |

**Readings, with no bar:**
- per model, D against A, H against A, and D against H, each with b, c, net and the number of rows whose predictions differ;
- the Q12 drift between D's and A's queries: the maximum, and the rows exact, within 1 code and within 2 codes, over all rows and over the rows where D and A differ;
- E against D, which is what the transition snap and the table rounding cost;
- F against D;
- the GPU writer's i32-oracle check on the D panel's first chunk (`d11_q12_oracle_mismatches`).

## 4. Instrument checks (each must be green before the classification counts)

1. **`I-GPU-Tests`.** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on the pod. That includes `gpu_the_head_on_the_d11_queries_is_the_d11_panel` and the earlier D11 GPU tests.
2. **`I-Fetch`.** Every fetched file verifies against its store folder's `SHA256SUMS`, and the DEV data against its pins (section 5).
3. **`I-Save-Load`.** F reproduces the stored `preds-float.json` row for row.
4. **`I-Inputs`.** The stored run's `artifact_sha256` and `sequences_sha256`, and the model's `model.safetensors` sha256 (`--model-sha256`), equal the pins in section 5. The mode checks all three, and the job checks the model files with `sha256sum -c` as well, before any residual run.
5. **`I-Verify-All`.** The stored run's two verify checks are equal at all 82,176 positions.
6. **`I-DEV-Denominator`.** A covers 14,287 rows.
7. **`I-D-Determinism`.** Two D panels are identical.
8. **`I-Head`.** The head on D's own queries writes exactly D's predictions.
9. **`I-D-Stored`, for plain-c126 only.** Stage 4b stored those two models' D11 panels when it trained them (`save/seed-s/preds-d11.json`). The recomputed D must equal them row for row, which checks D across builds.
10. **`I-Finite`.** Every step exits 0, and each step's stdout, stderr and exit code are saved. The log's last line is written before `SHA256SUMS`.

## 5. Base artifacts and data split

**The six models and their artifacts' stored runs,** each pinned by its gap receipt:

| Label | Seed | Model and stored predictions | Stored run | Artifact sha256 | `model.safetensors` sha256 |
|---|---|---|---|---|---|
| plain-c126 | 1 | `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-1/` | `results/claude/d11-fbs-artifact-gap-2026-10-26/gap/run-seed-1.json` | `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305` | `63ae5513ff2da425af564be9ad9b41587a1ba0493f31061c06127a6368aa8e92` |
| plain-c126 | 2 | `…/d11gap/save/seed-2/` | `…/gap/run-seed-2.json` | `d5b435961ecf0a0038a814a107b6b2d17ea36eeef6fd4397d69b208b52dbdcde` | `37aae39685a141fc3dbe2bb0d0be653d002aa739fae9c3b0fcf55f4f06e2e487` |
| d11-c130 | 1 | `results/claude/d11-fbs-d11-train-2026-10-26/d11t/save/seed-1/` | `…/d11t/run-seed-1.json` | `4255d39b3518350c2d6095a458870c3d7456fea28726cc6b7bc7e095befe6eca` | `55f5d6dce5ddb0121ab0a229f4ab62b0947afb61d52a7498879e7201c0843333` |
| d11-c130 | 2 | `…/d11t/save/seed-2/` | `…/d11t/run-seed-2.json` | `b466501e43a3cc6feb15b00ede4ae9cc3ed3c28b7aca7f1795babad49a3df1cd` | `f3c1b2064e885b90eef30b96404c4d17d86179dbaeba3eae37001f0e6316221c` |
| dual-c131 | 1 | `results/claude/d11-fbs-dual-train-2026-10-26/dual/save/seed-1/` | `…/dual/run-seed-1.json` | `0ae7dcd46bbf9c3eb681cd5d93100c6f3684183ee8807d14302770b24f85493a` | `6861db0fb4a8773cb00ac0b50839c6007783b9de1ebf36d948ab38bf99d93a72` |
| dual-c131 | 2 | `…/dual/save/seed-2/` | `…/dual/run-seed-2.json` | `8394a4d3dc911db60f9d04a9b88681f9a3f613ee68a272993effad093897b599` | `a78ce0b942a906e3460de45f1e07db84b677d99beb09613b9373c9401dfac1cf` |

All six runs read the same windows, sha256 `083c1be628f871ff1502e7d05f84cc3f55dda2ac461614a62df9167bcd08893e`.

**DEV data:** `w2-inputs/2026-10-25/driver-data/ft/dev/`:
- `tokens.u16`, sha256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
- `response_mask.u8`, sha256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.

No training data is read.

## 6. Steps and minimum test power

1. `uor-pod up --count 1 --gpu 5090 --ref <pushed head>`.
2. Build `spiralcore-fbs-sheet` (float-train, `--features cuda`) with nvcc, and copy it with its sha256.
3. `I-GPU-Tests`.
4. Fetch the section 5 files, each folder's `SHA256SUMS`, and the DEV data. Check them (`I-Fetch`).
5. For each of the six models, run `spiralcore-fbs-sheet --mode d11-residual --data-root <driver-data> --model-dir <save/seed-s/model> --evaluator <run-seed-s.json> --windows-sha256 <sha> --artifact-sha256 <sha> --model-sha256 <sha> --stored-preds <save/seed-s/preds-float.json> --seed <s> --label <label> --out residual-<label>-seed-<s>.json`. For plain-c126, add `--stored-d11-preds <save/seed-s/preds-d11.json>`.
6. Save the run folder to the store at `results/claude/d11-fbs-residual-2026-10-26/`, with a `SHA256SUMS`.

**Minimum test power:**
- six models, across 3 configurations and 2 seeds;
- 14,287 frozen DEV targets each;
- every A row read from a verified run, so P = 0.

## 7. Cost and stop rule

**Cost:**
- one RTX 5090 on Vast, about $0.56 to $1.11/h, under a 1 h lease;
- about 2 minutes of build and 1 minute of GPU tests;
- a few minutes of fetching (about 700 MB of stored runs);
- six residual runs of a few minutes each (three DEV panels and the head);
- in all, about 30 minutes, under $1.

**Stop rule:**
- Stop and save on a failed `I-GPU-Tests` or `I-Fetch`, a non-finite loss, or any nonzero exit.
- A failed instrument check on one model leaves that model unclassified, and the other models still count.
- Classify every failure as instrument or mechanism before calling it anything.
- Never cancel a healthy run because the lease ran short. Renew it.
- Save everything off the pod before `down`.

## 8. Non-author TEST FITNESS review

**Round 1 (fresh non-author reviewer, 2026-10-26): NOT FIT. Its fixes are all made.**
1. **Blocking: the rule tested "equally good", not "the same forward".** A hit-count paired test misses departures on the rows that both D and A get wrong, about 86% of them. The rule is now row by row: the head identical (d_H = 0), the tokens differing on at most 3% of rows, and the paired test as a further requirement, never alone. An "indeterminate" outcome opens an instrument investigation. The reviewer proposed drift thresholds. The sheet instead calibrates on claude-127's measured residual and defect, because that defect kept 509 of 512 queries within 2 LSB while changing 18% of tokens; the drift stays a reading.
2. **High: a head departure was only a reading,** against the ORDER. It is now a departure by the rule (d_H > 0).
3. **Medium:** the faithful row now says what the rule shows.
4. **Low:** Stage 4b's stored D11 panels are now checked (`I-D-Stored`).
5. **Low:** the model hash is now pinned inside the mode (`--model-sha256`, in `I-Inputs`).

**What it confirmed:**
- D's query path and head are `score_dev_panel`'s.
- A's queries are in the same units.
- E is computed before the tables are rounded and D after.
- The instruments catch a wrong model, run or windows.
- Every pin in section 5 matches its receipt, and every store path exists.
- The cost is inside the caps, and the change obeys CHARTER §3.

**Round 2 (fresh non-author reviewer, 2026-10-26): FIT.**
- All five round-1 findings are resolved.
- `classify_residual` implements the rule at its boundaries: 428 rows is faithful and 429 is indeterminate; 1,428 is indeterminate and 1,429 is a departure.
- The job passes the pins and flags the binary accepts.
- The calibration holds against claude-127. The 3% bound sits above a 95% upper limit on 5 of 512 (about 2.3%).
- **Its low items:**
  - the faithful row now says "three times" the measured residual;
  - the job's head placeholder is set at launch;
  - Stage 4b's stored panel came from `evaluate_dev_panel_d11` (`run_d11_export_gap`, `preds-d11.json`), the function the mode calls.
- **Its medium note, kept for the REPORT.** D is a float32 forward against an artifact with a Q16 residual stream, so a faithful artifact may land in "indeterminate". If one does, its drift and margins are read first. That outcome costs an investigation, never a false result.

TEST FITNESS: FIT
