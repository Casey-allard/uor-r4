# Four frozen seed-1 writer cases

The sheet preselected the first eligible unique supported request from each held wording family. This finite replay uses those four prompt IDs. The writer is the frozen integer paired-copy model. The role experiment contributes only the original spans of occurrences classified as requested. The binder takes a span when there is exactly one and abstains when there are zero or multiple. Each arm uses the same native self-fed decode and exact-copy stop.

## Reproduction

Build `clause-role-writer` from this directory and `bind-clause-writer-cases` from `adapter/` with `cargo build --release --offline --locked` and a lab-owned target directory. Set `T` to the pinned seed-1 tokenizer, `W` to the writer binary and `A` to the adapter binary. Set `I` to the preserved `writer-input` directory and `OUT` to a new run directory. The input directory contains the four original ordinary traces, the frozen `selected.json`, the original model `seed1.d11`, and `seed1-grade.json` from the two GPU role readouts.

The ordinary traces come from protocol-2 `uor-chat --discrete "$I/seed1.d11" --tokenizer "$T" --say "$TEXT" --max-tokens 16` with the selected text and memory reads enabled. Run the adapter, writer and independent summary in that order.

```sh
mkdir "$OUT"
"$A" prepare "$T" "$I" > "$OUT/cases.json"
"$W" "$I/seed1.d11" "$OUT/cases.json" > "$OUT/trace.json"
"$A" summarize "$T" "$I" "$OUT/trace.json" > "$OUT/summary.json"
```

The adapter validates the frozen selection and tokenizer hashes, re-encodes each complete protocol-2 prefix and compares it with the corresponding ordinary trace. It checks the one gold span against the selected person and re-encodes the full period-delimited assertion using the same tokenizer. The original tape must contain the correct name, linking words, exact object ID sequence and following stop at the independently verified offsets. The selected people all have assertion span `[9,10)`, object start 12 and gold IDs `[975,560,2527,85,1]`. The final ID is EOS. No reply text comes from the learned role features.

The writer compares the native reference with the ordinary CLI on the exact input prefix, artifact SHA, complete paired witness, emitted IDs and every cursor-step word. A fresh second process reproduced the case input, complete trace and summary byte for byte. It rejects a forced-onset arm that fails to emit the complete gold sequence. The adapter summarizes emitted IDs, decoded text, EOS and first-error offset from the retained raw traces. The [checksum manifest](../evidence/writer.sha256) binds the retained input and output bytes. The original files remain in the D24 local run root under `writer-input/` and `writer-run-01/`.

## Observations

All four ordinary CLI traces matched the native reference on every checked field. All six arms reached EOS on all four prompts. Exact reply through EOS was 4/4 with the forced gold onset and 0/4 in each of the ordinary, learned Identity, learned Positional, forced gold-person and disabled-source arms. Every failing arm diverged on its first emitted token.

| Held family | Ordinary reference | Learned Identity requested spans | Learned Positional requested spans |
|---|---|---|---|
| Far | “ kites” from source 23 | Empty | Empty |
| Near | “ kites” from source 23 | Empty | Empty |
| Right | “ Got it, you telling me.” with no source | Empty | Emily `[44,45)`, no source |
| Root order | “ kites” from source 23 | Emily `[49,50)` and William `[57,60)` | Empty |

Forced gold-person replay supplied Emily's exact query span in each case and returned no bound onset each time. Every learned Identity and Positional response also had no source and matched its own disabled-source response. The fixed cursor copied “ brass lamps” and stopped at EOS as soon as its correct onset was supplied. The four probes use the same two people and the same fact prefix, so these counts describe only these path interventions.

The original tapes expose a finite connector-boundary contrast. In the assertion for Emily, the exact two-token sequence for “ owns” (`[1263,85]`) ends immediately before onset 12. William's separate assertion has the same connector ending immediately before onset 22, the start of “ paper kites”. In three ordinary replies the binder instead picked 23, the first token of “ kites”, whose immediate predecessor is ID 1323, the token for “ paper”. The splice can be distinguished with this observed connector on these tapes. General connector recognition and learned use of that feature were outside the finite replay.

## Where the supplied-person path stopped

The production `paired_copy_witness` found exactly one Emily assertion on each tape, at `[9,10)`, with exclusive period fence 16. The legacy eligibility mask was `0b111` for positions 11, 12 and 13, respectively +2, +3 and +4 from the fact's last token. The word-start mask was `0b010`. Positions 11 and 13 were continuation pieces. **The correct onset 12 was already representable.** The unconstrained legacy selector picked 13. Its WordStart restriction picked 12 and the continuation stage retained it. The exact connector bank recorded `ordered_mask=0` for this assertion.

The attached ordered onset bank replaced that provisional choice. Its sole admitted candidate, onset 12, had connector depth 2, first-word width 2 and class signature 153. The nearest positive's integer distance was 124, equal to the bank radius, but the shape and class signature had no negative distance. The production scorer therefore emitted no margin and returned status 2, unseen. The binder had zero complete values and returned no source. The forced-onset arm separately showed that the unchanged cursor could write all four object tokens and reach EOS from 12.

The [source witness](source-witness.json) retains per-case counts and the common candidate scores. The `clause-source-witness` binary in `adapter/` calls `paired_copy_witness` on the original cases, reconstructs the legacy candidate words and calls the same typed `Bank::score_legacy_row` used by the production onset decision. It verifies status and source against the production witness. Run `clause-source-witness "$I/seed1.d11" "$OUT/cases.json"` to reproduce the full receipt. Two fresh executions emitted identical bytes.
