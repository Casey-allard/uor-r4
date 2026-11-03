# ORDER: cpu-training-pilot, deepseek-91

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `d1d465eaeea0063cf83c3bd3260659a6ebf3f976`.

## Goal

Move the CPU full-path evidence beyond a trained writer beside synthetic
records. Build one finite artifact whose Hamming margins, opening and reply
memory tables, and point-key rows are derived from training examples and
selected through the exact CPU/serving contract, then export it and rerun the
complete served-forward audit.

This is component evidence on a synthetic initial trunk. It does not claim a
public trained model, language quality or GPU qualification.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-91/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. The finite control starts from a parsed FBS artifact with active Hamming
   reads and no turn memory or point key.
2. It derives an opening entry and later reply entries from supplied
   prompt/reply examples. Duplicate keys use the commonest target with the
   smaller token id as the tie break, and every key is sorted.
3. It derives point-key rows from opening queries whose exact memory key is
   absent, including the exact query, target and Q12 point conversion.
4. It calibrates every Hamming read/head margin by exact served target
   rank/margin over a finite candidate set. At least one margin changes from
   its initial value.
5. It trains the writer table through the existing exact rank-margin trainer,
   synchronizes that change back to the source snapshot, and exports the full
   artifact with all four mechanism classes active.
6. Two independent constructions are byte-identical. The final artifact
   passes the complete CPU/serving served-forward audit with zero query,
   token, kept-cell and admitted-cell mismatches.
7. Controls show that the trained Hamming, memory and point records each
   change the complete decision path; the opening and reply memory hits and
   the point fallback are separately observed.
8. Focused and full CPU crate tests, strict Clippy, formatting, report,
   roadmap and local PR checks pass.

## Stop if

- Any derived record can be exported without being present in the parsed
  artifact or changes a non-target record.
- Hamming calibration cannot find an exact score change, or the point fallback
  cannot be reached after the memory miss.
- Export parsing or CPU/serving parity fails.
