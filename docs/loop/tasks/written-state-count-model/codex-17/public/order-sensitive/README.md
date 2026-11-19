# Finite ordered versus order-neutral production scorer

Both 8,416-byte banks contain the same four analytically planted exemplars, labels, shape and class words, radii, and fixed 256-record allocation. Neutral applies the declared within-segment, within-class root sort to those same exemplars. The fixed packet supplies no labels or emitted IDs. Its ordinal is zero, its assertion index is zero, and its candidates are original addresses 8 and 9.

| Tape edit | Ordered | Neutral |
|---|---|---|
| None | Selected 8. Candidate margins 4/2 and 0/4. | Selected 9. Candidate margins 4/2 and 32/4. |
| Swap the two bridge START roots | Selected 9. Margins 4/2 and 64/4. | Selected 9. Margins 4/2 and 32/4. |
| Swap a bridge START root with the word CONT root | Status 4, no address. Margins 0/2 and −48/4. | Status 4, no address. Margins 0/2 and −32/4. |

The class tape, source offsets, candidate masks, and bank records remain fixed across the tape controls. The neutral later candidate's entire trace is identical under the within-class swap. Swapping across classes changes its positive distance from zero to 64, and neither arm selects a source.

A second matched pair uses actual codebook roots from the existing synthetic Mira role/writer prompt, with the same analytic construction and exact original carrier and tokenizer. Its D11s have equal lengths of 86,598,634 bytes. Four invocations of the pinned Wick CPU `uor-chat` used ordinary protocol-2 `--say`; both cold pairs matched byte-for-byte. Prompt IDs and paired role trace words match between arms. Ordered selected address 8 and emitted `[924,1074,1]` (blue green). Neutral selected address 9 and emitted `[1074,1]` (green). Both stopped on EOS. The retained manifest digest for the Wick run is `6bfdb5549213c9b35f7a6ead9aacbb4d70a2b549f18bdb03dbb29c47cff387e4`.

The fixture proves an order-sensitive distinction under the actual scorer and writer, with equivalent input and bank capacity. Its records are analytic. No fitting, GPU execution, held distance, or conclusion about held behavior follows from it.

## Reproduce

The original synthetic carrier and sidecar are identified by `host-inputs.sha256`; the carrier is the archived seed-one finite `favor_correct.d11`. Set SYNTHETIC_D11, SPAN_STATE, and ORIGINAL_RECEIPT to the three corresponding verified inputs listed in `host-inputs.sha256`. The shared product source revision is recorded in `SOURCE-REVISION`.

```sh
out=$(mktemp -d)
CARGO_TARGET_DIR="$out/target" cargo run --offline --locked --release -- \
  "$SYNTHETIC_D11" "$SPAN_STATE" "$ORIGINAL_RECEIPT" "$out" > "$out/host-traces.json"
cmp host-traces.json "$out/host-traces.json"
```

The live writer arguments are `--discrete ARTIFACT --tokenizer TOKENIZER --protocol 2 --temperature 0 --max-tokens 16 --read-mode enabled --say PROMPT`, where the prompt is “Mira kept blue green. Nora kept red. Please give what Mira collected before closing, rather than Nora beside the entrance.” The cold raw outputs, exact locked tool and tokenizer hashes, and both D11 hashes are in the isolated Wick receipt named in `wick-location.txt`.
