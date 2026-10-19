# Dispatch 6257564516: BVG-K C2 qualification PASS; injective attention pre-flight fails B3_init, so the run is not dispatched — 2026-10-19

- **Lab:** Claude, the single operator named by Kimi's PM dispatch (#2029 `6257564516`).
- **Pod:** Vast `vast-56648533`, 2 × RTX 5090, at main `5c7276cfd8eac2fd8c820ec75395971162d6fe06`. Bootstrap parity PASS. About 13 min, roughly $0.40. Outputs are archived to the HF store under `vast-archive/vast-56648533/` (45 MB, read-back verified), and the pod is deleted.
- **Build and identity:** one release build shared by both jobs. Driver hash `f4509049add30b7b756cf5a9ddf97d5633ad4aa790028d8b1b210986945d09e9`, equal to the static driver in Codex's executor packet. The D29 checker passes.
- **Posted:** #2029 `6257768576`.

## Job 2 (GPU1): BVG-K C2 device qualifier `bvgk-qualify` — PASS

Sheet: `m1-bvgk-stagec2-block-qualification-2026-10-18.md` (FIT). It runs on synthetic fixture devices only and commits only through `apply_block`, as the owner approved (fixture-only scope, apply-free redesign).

| Control | Status |
|---|---|
| V0 source identity | PASS |
| RC re-carry parity | PASS |
| UT / JR / DC / V3 block sweep (decision coverage met) | PASS |
| RF refusals | PASS |
| C1R C1 parity rerun | PASS |

- **Receipt:** `evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json`, SHA-256 `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`.
- **Updates and identity:** 3,419 updates, all on fixtures. `Device::apply` was never called. The rows image source, PTX and cubin equal the C1-qualified hashes.
- **Throughput (diagnostic, K16 Route shape, 64 rows).**
  - K = 1 takes a median 129.7 ms per call.
  - K = 16 takes **259 ms with the joint verify included**: 16.2 ms per coordinate, **about 8×**.
  - C1's scoring-only curve was 16×. When |S_t| ≥ 2, the scratch clone, apply and re-score roughly double a call.
  - This is input to the Stage T cost, not a gate.

## Job 1 (GPU0): injective-codebook attention pre-flight (zero updates)

**Inputs:**
- Codex's authenticated `bound-inputs-v2`, byte-identical to the runbook evidence.
- The rebuilt table file `83c1380a…`, equal to the pin.
- Data pins verified (TRAIN `3dee858f…`, DEV `0919e90f…` / `a838bb1a…`, tokenizer `d36d3e87…`).

**Identities:**
- `attention-build-identity` returns config `0c3404d4…` for relation and `368f4285…` for frozen-zero. It does not emit the scope.
- Checkpoint 0 comes from `init-corpus` (`new_model(0)`): 0 accepted updates, with both arms on model `134fd663…`.
- The DEV evaluation (`eval-corpus`) reports 0 updates and `evaluation_mutates_model = false`.

| Gate (sheet §3) | Result |
|---|---|
| G-Init: checkpoint-0 table equals the pin word for word | **PASS** |
| G-Init negative (a): edited table with stale hashes refused on the host | **PASS** (rc 2) |
| G-Occ: `occupied_cells > 16`, within-cell tie mass 0, no split alias group | **PASS** (256 cells; tie mass 0) |
| G-Cap raw | 14,287 (4,096 distinct vectors; all 674 DEV targets are representatives) |
| **B3_init(P_succ) ≥ 500** | **FAIL: 127 / 3,847** (full panel 427 / 14,287; complement 300 / 10,440) |
| **Net G-Cap > max(500, top1_control(cp0) + 24)** | **FAIL: 127** (control checkpoint-0 full-panel top-1 = 38) |

Per the sheet, a B3_init below 500 means **the run is not dispatched**, and a successor decision is needed.

Not run, because they are moot once B3 fails:
- G-Init negative (b), the GPU status-6 check;
- the inherited parent battery (Gate22/Gate0 sensitivity, Relation C2 ACTIVE/INERT, INIT LOCK).

They remain required for any successor.

### Reading

The injective table does what it was built to do:
- **no aliasing:** a ceiling of 14,287 and zero tie mass;
- **all cells used:** 256 / 256.

It spreads tokens evenly across all 256 cells. With K = 8 admitted cells per row, a target's cell is admitted in about 8/256 ≈ 3.1% of rows, and the measured full-panel rate is 3.0%.

At the seeded start, aliasing was the binding limit. Here the limit is **admission breadth relative to occupancy**. The sheet anticipated this: "with 256 occupied cells, admission at K = 8 may be low from the start."

This is an instrument-reachability result for this configuration (`INSTRUMENT_UNREACHABLE`, sheet S:59). It is not a mechanism negative (D22) and not a training result: nothing trained.

### Successor options (owner and PM decision; none taken)

- **(a) Raise K with occupancy.** P_succ admission ≥ 500 needs about 13%, which is roughly K ≥ 34 at 256 cells. The attention validator pins K = 8, so this needs a new sheet.
- **(b) Keep the injective table but assign cells by address structure**, so related tokens share or neighbour cells. This is the K16 sheet's Lever 1, the address metric.
- **(c) A partially injective table with fewer occupied cells**, trading alias ceiling for admission.

## Evidence

`evidence/` holds the C2 receipt, the D29 outputs, the build identity, both arms' identity, checkpoint-0, evaluation-aggregate and B3 summaries (per-row rows omitted, with the full report SHAs recorded), the census, the negative-control output, the job scripts and the job logs. `SHA256SUMS` lists every file except itself.
