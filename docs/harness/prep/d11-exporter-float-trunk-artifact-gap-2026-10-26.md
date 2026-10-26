# Prep sheet: the exported artifact's own export gap (d11-exporter-float-trunk, claude-129), 2026-10-26

- Item: [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). ORDER: [claude-129](../../loop/tasks/d11-exporter-float-trunk/claude-129/ORDER.md).
- This amends Stage 4b ([its sheet](d11-exporter-float-trunk-stage4b-2026-10-26.md), FIT in round 3). The bar is unchanged. D, the exported computation, is now the artifact itself, evaluated on the GPU by claude-128's qualified evaluator, in place of the D11 float emulation.
- Run: one job on one leased RTX 5090:
  - `spiralcore-fbs-sheet --mode d11-artifact-windows`;
  - `d11-fbs-eval run … --verify` on each seed's artifact;
  - `spiralcore-fbs-sheet --mode d11-artifact-gap` for each seed.

## 1. Question

**Does the exported artifact of the trained trunk keep its DEV score, inside CHARTER §3.7's two-seed noise band?** Stage 4b asked this but could not count an answer, because its emulation of the artifact was not the artifact. Here the artifact's own tokens and Q12 queries are scored.

## 2. Arms

- **F, the float model:** each seed's saved `Fbs_1Atom_Active` model, from claude-126, scored on the 14,287-row `ft/dev` panel in plain float (`evaluate_dev_panel`, `panel_loss`).
- **A, the exported artifact:** each seed's `artifact.uorlut`, run by `d11-fbs-eval` over every DEV window, one fresh session per window. Its prediction at a scored row is the evaluator's token at that offset, and its loss is `MarginHinge(rivals = 8)` on its own Q12 queries through the D11 head on the exported tables. This is the head `panel_loss_d11` uses, fed the artifact's queries in place of the emulation's.

## 3. Bar and decision table

**The bar is Stage 4b's, unchanged.**
- **b** counts the `ft/dev` rows that F hits top-1 and A misses; **c** counts the reverse. Both are over the 14,287 eligible targets.
- **F_I** = max(24, ⌈2√(b + c)⌉).
- **Δ_noise,L** = 0.045679637781173676, claude-126's two-seed DEV loss band (`d11-gap.json`, `noise_band_dev_loss`).

| Gate | PASS when | PASS unlocks | FAIL means |
|---|---|---|---|
| `G-D11-ExportGap` | On **both** seeds, \|b − c\| ≤ F_I **and** \|L_A − L_F\| ≤ Δ_noise,L, with every instrument check of section 4 green | This trunk's D11 export is inside the band. With the audits (claude-127, claude-128) and claude-127's laptop tok/s, the item's done-when is met, and #2820 closes. | Direct post-training rounding into the artifact is disqualified for this trained configuration (CHARTER §3.7). The next configuration trains in the D11 mode before export. A negative closes this configuration, never the mechanism. |

**The expected outcome is FAIL on the top-1 half.**
- Stage 4b's D11 emulation, which agrees with the artifact on all but about 1% of tokens, already gave net gaps of 77 and 353 rows, against F_I of 60 and 69.
- A FAIL is still decisive. It is the first gap read in the artifact's own computation, with no emulation between.

**Readings, with no bar:**
- both seeds' top-1 hits for F and A;
- the net gaps;
- the loss gaps;
- the evaluator's GPU wall time over all 428 windows.

## 4. Instrument checks (each must be green before the gap counts)

1. **`I-Verify-All`.** `d11-fbs-eval run --verify` gives G = H = R at every position of all 428 windows of 192 (82,176 positions) on each seed: the GPU evaluator, its host twin and the Rust integer session.
   - So the artifact's tokens and queries are the shipped runtime's own, at every scored row. P = 0 by construction, with no projection.
   - The evaluator's PTX and cubin sha256s are recorded and expected to equal claude-128's qualified ones (`013d19fd…`, `73eccb70…`; `fbs_core.h` and `evaluator.cu` are unchanged). A difference is recorded, and `I-Verify-All` still decides.
2. **`I-Save-Load`.** Each seed's saved model reproduces claude-126's stored float panel:
   - its predictions equal `preds-float.json` row for row;
   - its plain loss is within 1e-6 of the receipt's `loss_float`.
3. **`I-Inputs`.**
   - The windows file the evaluator reads is the one the sheet wrote: `sequences_sha256` equals the windows file's sha256.
   - The artifact the evaluator ran is this seed's own: the run's `artifact_sha256` equals section 5's pin for seed s, passed as `--artifact-sha256` and checked in `run_d11_artifact_gap`.
   - Every store input matches section 5's sha256s at fetch.
   - The receipt records the sha256s of the model's `model.safetensors` and `config.json`.
4. **`I-DEV-Denominator`.** 14,287 scored rows over 428 windows. The windows mode refuses anything else, and so does the scorer.
5. **`I-Finite`.**
   - Every step exits 0, and each step's stderr and exit code are saved.
   - The artifact loss is finite.
   - The log's last line is written before the hashes.

## 5. Base artifact and data split

**Base artifacts and models.** These are claude-126's, from `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/`:

| File | Seed 1 sha256 | Seed 2 sha256 |
|---|---|---|
| `save/seed-<s>/artifact.uorlut` | `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305` | `d5b435961ecf0a0038a814a107b6b2d17ea36eeef6fd4397d69b208b52dbdcde` |
| `save/seed-<s>/model/config.json` | `f1674fba2481a6520adddcea85e92e34fa2e059761e4bd307af5124e1acbc6c4` | `a9ba99b3239df7108158521a78958cf90288134f36b6c4858f186e387e4abdd2` |
| `save/seed-<s>/model/model.safetensors` | `63ae5513ff2da425af564be9ad9b41587a1ba0493f31061c06127a6368aa8e92` | `37aae39685a141fc3dbe2bb0d0be653d002aa739fae9c3b0fcf55f4f06e2e487` |
| `save/seed-<s>/preds-float.json` | `1ae56205dd45a91690c88985dca7e92bd062d9dc58a2b24fd519ea1138008649` | `89342eccc79d030f4d1530ede018e4fee6f45e1e577a3e9fffab7cd1e2d35be0` |

The receipt `d11-gap.json` has sha256 `88b6a5f988fe144671365a258a100e5cf3935f98e917040b38f90df2637a1e8a`.

**Data split.** The `ft/dev` panel, from `w2-inputs/2026-10-25/driver-data/ft/dev/`:
- `tokens.u16`, sha256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`;
- `response_mask.u8`, sha256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.

The loader checks both pins itself. The panel is 400 documents, 428 windows of 192 and 14,287 eligible targets. The training split and `fresh_v2` are not read.

## 6. Steps and minimum test power

1. `uor-pod up --count 1 --gpu 5090 --ref <pushed head>`.
2. Build `spiralcore-fbs-sheet` (float-train, `--features cuda`) and `d11-fbs-eval` (the instrument, `--features cuda`) with nvcc. Copy both binaries to `/workspace/gap-bin/` with their sha256s.
3. Fetch the section 5 files and check them against the pins (`I-Inputs`).
4. `spiralcore-fbs-sheet --mode d11-artifact-windows --data-root <driver-data> --out windows.json`. Record its sha256.
5. For each seed:
   - `d11-fbs-eval run <artifact> windows.json run-seed-<s>.json --verify` (`I-Verify-All`);
   - then `spiralcore-fbs-sheet --mode d11-artifact-gap --data-root <driver-data> --model-dir <save/seed-<s>/model> --evaluator run-seed-<s>.json --windows-sha256 <sha> --stage4b d11-gap.json --stored-preds <preds-float.json> --seed <s> --out gap-seed-<s>.json`.
6. Save the run folder to the store at `results/claude/d11-fbs-artifact-gap-2026-10-26/`, with a `SHA256SUMS`.

*Note after the run:* step 5's `d11-artifact-gap` also takes `--artifact-sha256 <seed's pin>`, added in FIT round 1 and passed by the job for each seed.

**Minimum test power:**
- 2 seeds;
- 14,287 frozen DEV targets;
- every scored row read from the artifact's verified run: P = 0, against Stage 4b's projected P ≥ 110.

## 7. Cost and stop rule

**Cost:**
- one RTX 5090 on Vast, about $0.56/h, under a 1.5 h lease;
- about 10 minutes of bootstrap and builds;
- about 2 minutes of GPU per seed for the evaluator;
- the verify on the pod's CPU, about 15 to 20 minutes per seed, since the reference session steps 82,176 positions;
- a minute or two of GPU for each seed's float panel;
- in all, under $1.5.

**Stop rule:**
- Stop and save on any nonzero exit or failed instrument check. Localize it; never re-run blindly.
- Never cancel a healthy run because the lease ran short. Renew it.
- Save everything off the pod before `down`.

## 8. Non-author TEST FITNESS review

TEST FITNESS: PENDING

**Round 1 (fresh non-author reviewer, 2026-10-26): FIT, with two required changes before the run.** Both are made in `I-Inputs` above:
1. The scorer checks the evaluator's `artifact_sha256` against the seed's own pin.
2. The receipt records the model files' sha256s.

**What the reviewer confirmed:**
- **The row mapping.** The evaluator's position `w · 192 + offset` is the hidden state the float panel scores.
- **The loss.** It is `panel_loss_d11`'s counterpart: the same D11 head on the exported tables, fed the artifact's own `q / 4096`, with a row-weighted mean that does not depend on chunking.
- **No emulation.** The A side runs no trunk forward.
- **The bar** is Stage 4b's.
- **`I-Verify-All`** makes the scored outputs the shipped runtime's own. That is why a PTX hash differing from claude-128's may be recorded rather than gating.
- **§3.5 holds.** The CPU runs only integer parity references, and neither feeds the gap.

TEST FITNESS: FIT
