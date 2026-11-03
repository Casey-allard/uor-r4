# ORDER: m4-export-uorlut01-cli, deepseek-4 (copy-code key guard)

Lab: deepseek. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md) (#2859). Base: `b6ebf7002`.

## Goal

Make `fbs-toks --copy-codes` fail closed on a `UORLUT01` artifact whose Hamming copy heads use a key other than the one-token key. Its per-token control and trace reconstruction are defined for `copy_key = 1`; after Claude's #3579 lands, applying that code to key 2 would report misleading consistency counts.

The guard reads only the container header, so it protects copy-key-2 artifacts even before this binary knows the newer snapshot record.

## Cancel probe

1. **Already guarded?** At `b6ebf7002`, `copy_codes` parses the artifact directly and never inspects `hamming.copy_key`. The old parser refuses the unknown field, but after #3579 it would accept key 2 and run the wrong one-token comparison. Not cancelled.
2. **A full pair-aware check available?** No. The per-token control arrays contain halves, while key 2 needs ordered adjacent-token pairs and pair-keyed inconsistency accounting. This unit deliberately refuses rather than reports a false pass/fail.
3. **GPU required?** No. This is a finite header/refusal control.

## Files in scope

- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `docs/loop/tasks/m4-export-uorlut01-cli/deepseek-4/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/m4-export-uorlut01-cli.md`
- `docs/harness/claims.json`

## Done when

1. A missing `hamming.copy_key` is read as the one-token default 1.
2. Any explicit non-one key is refused before artifact parsing or trace comparison, with a message that names the key and says `--copy-codes` is one-token only.
3. Malformed magic, header length, Hamming record and key type are refused.
4. The existing `fbs-toks` tests and the full integer library pass.
5. The REPORT classifies the change as an instrument guard, not a model result.

## Stop if

- Claude changes `copy_code_check` to natively support copy-key 2 before this merges. Merge forward and replace the guard with the working pair-aware path instead of layering a refusal over it.
- The guard needs to parse payload sections. Header-only is sufficient.
