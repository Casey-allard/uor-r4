# Injective codebook initialization for new lineages (D39 item 3) — 2026-10-18

- **Lab:** Claude, self-directed (D39 item 7). Intent: #2029 6253613139.
- **Line:** M1 attention. Source and host checks only; no model execution, no compute, $0.

## Problem

The default seeded device initializer (`discrete_init`) gives **16 distinct token vectors** at checkpoint 0, according to the W0 census reproduced by `coverage-census` (#2678). The Relation read ranks context nodes by relation score over those payloads, so at initialization it can separate only 16 classes of key. Attention runs so far had to diversify the codebook first, through slow single-coordinate moves that also eroded admission. No run so far has isolated whether the read can learn.

The qualified PrimeZeta constructor (#2433) is injective, but no training lineage could use it.

## Change

1. **`DriverConfig.codebook_initialization`** (`training.rs`):
   - `seeded_device_v1` is the default. It is serde-skipped, so existing config bytes and hashes are unchanged (pinned by a test).
   - `pinned_table_v1 { table: FilePin }` names an exact integer codebook (`uor-r4.codebook-table/1`). The FilePin sits inside the config, so the table is bound into the lineage.
2. **`CodebookTable::load`** reads and checks only host metadata: the file's SHA-256 against its pin, the schema, vocabulary/blocks against the spec, the length, and the word digest. It refuses on any mismatch (pinned by a test).
3. **`PreparedDriver::new_model`** builds the device initializer's model, then replaces only the codebook through `Device::from_snapshot`. That path's CUDA `discrete_validate` admits every word as a root (D24). It then re-reads the table and requires an exact match. Routes, gates, seed and epoch stay the initializer's.
4. **`prime-zeta-codebook`** (a new binary) exports the qualified constructor's table, or a matched tangent/normal null.
   - It cross-checks the constructor's own digest against the pinned-table digest and writes its output exclusively.
   - f64 stays inside the separate `prime_zeta` module up to its integer boundary, as in `prime-zeta-qualify` (D37 and the October 14 zeta exception). The lib never references it.

`driver_source_sha256` changes, because `training.rs` is in its include list. The K16 packet stays bound to its own built binary and its recorded pins.

## Measured: G-Cap evidence (host census, no model execution)

| | Seeded checkpoint 0 | PrimeZeta V4096 B4 |
|---|---|---|
| Distinct vectors | 16 | **4,096** |
| Occupied writer cells | ≤ 16 | **256 / 256** |
| DEV alias ceiling (14,287 rows) | 1,986 | **14,287** |
| DEV targets that can win a tie (of 674) | 4 | **674** |
| Exact tie mass (DEV rows) | 12,301 | **0** |

The table digest is `e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804`. It is bit-exact for seeds 1 and 2, as qualified.

## Checks

- **Host tests:** `uor-r4-discrete` lib 98/98 (1 ignored), including the 2 new tests.
- **Negative gate:** with the default not skipped, the byte-identity test FAILs.
- **CUDA type check:** the stub-nvcc `--features cuda --all-targets` check passes (type check only).
- `fmt` is clean. The export binary refuses an existing output file.
- **Not run:** `new_model` with a pinned table on a real GPU. That is the pre-registration's G-Init gate (§3) and runs in its pre-flight battery after FIT.

## Limits (from the non-author adversarial review, APPROVE-WITH-NITS; all findings applied or documented)

- **Pin path is host-specific.** The FilePin's absolute path is part of the config bytes, as it is for corpus pins. A run's config and lineage must use the path on the host where it runs, and the table file must exist there on resume.
- **Injectivity is declared and recounted, not assumed.** The table file carries `distinct_vectors`, which the loader recounts. The exporter also cross-checks it against the constructor's own count. "Pinned" does not by itself mean injective; the count says whether a table is.
- **No GPU test yet of the `new_model` pinned path.** The pre-registration's G-Init gate covers it in the pre-flight battery: the table equals the pin word for word, routes, gates and epoch equal the unpinned initializer's, and a non-root word must be refused with status 6.

## Next

The pre-registration draft `docs/harness/prep/m1-attention-injective-init-DRAFT-2026-10-18.md` changes exactly one variable from the FIT'd parent attention sheet: the initial codebook. Registration needs TEST FITNESS from non-author seats; running it needs PM dispatch and owner release.

## Follow-up: attention-runner interlocks (AGY FIT review 6253870519)

- **Distinct scope.** A pinned-codebook config produces `m1-attention-learning-injective-{arm}-k8-seed{N}-batches0..16404`, so owner releases and run scopes can never collide with the seeded Dose 1 scopes. The seeded strings are unchanged, and a test pins both.
- **G-Init enforced in code.** `PreparedDriver::validate_checkpoint` refuses a pinned lineage's checkpoint, before any accepted update, if its `table` differs from (or lacks) the pinned table. A test covers this, with a negative gate.
- **Pin format.** `DriverConfig::validate` checks the pin's SHA-256 format.
- **Deferred to the registration PR,** since both depend on the final sheet text:
  - dispatch of `ATTENTION_SHEET_SHA256` by `codebook_initialization`;
  - the sheet's H1 title, to satisfy the existing prefix check.

## Subagent FIT review and fixes (owner direction, 18 October)

Two fresh-context reviewer subagents reviewed the sheet. The first covered the sheet and its source, and the second the fix diff. Both returned FIT-WITH-FIXES. Everything they found is applied, and the sheet is **TEST FITNESS: FIT**.

- **F1.** `validate_plan_header` now picks the registered sheet by `codebook_initialization`. A seeded init binds the parent sheet and is unchanged. A pinned init binds this sheet. Each is refused with the other's SHA. A unit test pins both SHAs and both title prefixes to the committed sheets. A negative gate confirmed it: with the fix reverted, the pinned-plan test fails.
- **F3.** The `injective-` scope requires `distinct_vectors == V` and carries a 12-hex table-SHA prefix. The seeded scope string is byte-identical.
- **Bar (i).** It is a ranking bar: `top1_control(P_succ, cp0) + 24`. The gate is net G-Cap > max(500, Bar (i)). It is necessary, not sufficient.
- **F2.** A Sequencing section: register now, and dispatch only after D39 W1 concludes, with the configuration re-checked first.
- **F5.** The G-Init negative control is split into a host refusal and GPU status 6.
- **F6.** G-Init is a checkpoint-0 property.
- **Re-pin rule.** The sheet does not pin the driver hash inside itself, because that would be circular. The release packet binds it through `execution.driver_source_sha256`.

Training stays HELD. Registration releases nothing. Dispatch also needs the owner's release, which Casey types (D40).
