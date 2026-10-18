# Coverage census instrument (D39 item 2) — 2026-10-18

- **Lab:** Claude (self-directed under D39 item 7; intent posted at #2029 6253130000).
- **Line:** M1 (#2029). Instrument only; no model execution, no compute, $0.
- **Source:**
  - `crates/uor-r4-discrete/src/coverage_census.rs`;
  - binary `coverage-census` (`src/bin/coverage-census.rs`).

## Why

D39 item 2 makes two things mandatory:
- a **G-Cap** reachability gate on every admission sheet: the DEV-row alias ceiling must exceed Bar (i), net of admission;
- **coverage metrics** in every qualification packet: distinct vectors, an alias-group histogram, and within-cell tie mass.

The W0 census that motivated D39 was a local Python script (Kimi PM, `census.py` / `dev_ceiling.py`). It also re-derived each token's cell on the host from codebook roots. D24 forbids that host recomputation of the model's addressing, and D27 makes Rust the default for tooling. This instrument makes the census durable and D24-clean.

## What it computes

Input:
- a driver config;
- a **native** `DriverCheckpoint` (not an authentication wrapper);
- optionally, the GPU token-cell export receipt (`uor-r4.attention-token-cells/1`, from `discrete-stack attention-export-token-cells`).

Provenance goes through the existing `PreparedDriver::load` and `validate_checkpoint`. The receipt is bound to the exact checkpoint by `driver_checkpoint_sha256`, and the cells file by its own SHA-256.

| Metric | Definition |
|---|---|
| `distinct_vectors` | number of distinct full vectors (all blocks) in the codebook |
| `alias_group_size_histogram` | group size → number of groups, where a group is tokens with an identical vector |
| `dev_alias_ceiling_rows` | DEV rows whose target is the lowest-ID member of its alias group (the only member that can win the tie-break) |
| `dev_targets_alias_representative` | distinct DEV target tokens that are representatives |
| `g_cap.verdict` (with `--bar-rows`) | `ALIAS_CEILING_ABOVE_BAR` if ceiling > bar, else `INSTRUMENT_UNREACHABLE`. Admission netting comes from the admission receipt, not from this census |
| `within_cell_tie_mass` | tokens in alias groups of size > 1, and DEV rows whose target is a non-representative alias member. These are exact evaluation ties (see below) |
| `cells.*` (receipt only) | occupied cells, occupancy histogram, collision pairs, and **crowding**: cells holding more than one distinct vector, with the tokens and DEV rows in them. Crowded vectors compete on token distance; they are not ties |

Without a receipt, `cells.status` is `UNAVAILABLE_NO_GPU_TOKEN_CELL_RECEIPT`. Cell metrics are never estimated on the host.

**Why the alias ceiling is exact.**
- At evaluation, `table_block` (`discrete.cu`) returns the token's own row unchanged.
- So byte-identical vectors always get the same `token_cell` and the same `token_distance`.
- The writer and evaluator order candidates by (token distance, token ID), so only the lowest-ID alias can ever win.
- The census enforces this at runtime: a receipt that places one alias group in two cells is refused as an instrument fault (D31).

## Validation

- **Unit tests:** 6/6 pass. They cover:
  - lowest-ID representatives;
  - ceiling counting;
  - histogram;
  - the G-Cap verdict;
  - tie mass;
  - crowding counting distinct vectors rather than tokens;
  - refusal of a split alias group;
  - refusal of malformed specs, tables, targets and cell maps.
- **Negative gate:** with the split-group check set aside, its test FAILs; restored, all 6 pass.
- **Spec binding:** the checkpoint's model spec must equal the config's spec.
- **Cross-check against the independent W0 census**, on the same stored checkpoints and the same 14,287 DEV rows, loaded through the real `PreparedCorpus` provenance:

  | Checkpoint | This instrument | W0 |
  |---|---|---|
  | rung-0 Q4 (`checkpoint-16404`) | 264 distinct vectors, ceiling 8,948 | 264 / 8,948 |
  | initial-model snapshot | 16 distinct vectors, ceiling 1,986 | 16 / 1,986 |

  Both match exactly.
- **End-to-end refusal:** a checkpoint that is not the native form is refused (exit 1), with a sealed `FAILED` report in the reserved output file.
- **Not yet exercised end to end:** a full positive run on a matching native config/checkpoint pair. None is on the host yet; the K16 qualification checkpoints are still in custody.
- `driver_source_sha256` is unchanged, because no file in its include list (`training.rs:19–33`) is touched.

## Limits

- The GPU token-cell exporter currently admits only the attention-learning configs it validates (`attention_preflight.rs`). Route configs, such as the K16 redirect, therefore have no cell receipt yet, and their cell metrics stay `UNAVAILABLE`. Widening the exporter is a separate change. It touches `attention_preflight.rs`, which is in the driver hash list.
- G-Cap nets out admission only through the separate admission receipt.

## Usage

```
coverage-census CONFIG CHECKPOINT OUTPUT [--token-cells RECEIPT] [--bar-rows ROWS]
```

OUTPUT is created exclusively.
