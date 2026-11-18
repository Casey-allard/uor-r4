# [codex] Fitted span and connector state

## Operator

The two #4087 trained role state digests and the two D11 artifact digests are pinned in the executable. Bare prefix IDs and the associated blind baseline prediction are joined by SHA-256. The serving implementation recomputes the original anchor pair and source. The SM89 span kernel receives that pair and the original token IDs. It emits three seven-word records in the order anchor query/fact, inclusive query start/end, inclusive fact start/end, and selected value position. Missing positions use the maximum u32. Final reports use JSON null for missing positions. Every device word matched the independently evaluated Rust serving result.

The 4,096-entry word class table is derived from the released ByteLevel BPE tokenizer vocabulary. An ASCII alphabetic piece beginning with the encoded space starts a word; an unprefixed ASCII alphabetic piece continues it. Other pieces, including punctuation, mixed alphanumerics and non-ASCII, are singleton anchors. Both full words must match on raw IDs, lengths and anchor alignment, and the fact must precede the question without overlap. Entity expansion retains the frozen role choice; a partial match between different words releases to ordinary generation.

Each GPU prototype contains five u32 words: connector length, up to three ordered packed E8 roots (unused slots carry the sentinel), and the first value root. A single training pass retains unique prototypes in occurrence order. For a candidate length n (1–3) place the proposed source at fact_end+n+1, before the query's first token. Compare its n connector roots with the prototype in order and the proposed value root with the prototype's first-value root. The exact rational score is four times the mean connector distance plus that value distance. All distances sum coordinate-wise L1 across the four E8 roots. Integer cross-products compare lengths; exact ties choose the longer connector. There is no explicit word-start test at the proposed value.

The connector-only arm uses the old single anchor as both endpoints. The endpoint-only arm uses the expanded spans and the original one-token bridge scores. Each release produced 18 relation prototypes, six per length, and the class table (232 singleton, 2,819 word-start and 1,045 continuation IDs). The banks are identical across releases. The state has 4,186 u32 payload words (16,744 bytes uncompressed); each artifact/tokenizer-bound state file is 9,118 bytes. The TRAIN-only fitting interval was 12 seconds for both models together. Primary prediction took 36 seconds across 4,064 blind prompts with all three arms and native reference comparisons, including file I/O and model loading.

## Sources before closed-loop decoding

Both seeds gave identical source counts. On 1,728 new prompts, 229 frozen selections and 359 full selections reached the requested address. Full repaired 134 and lost four. Connector-only succeeded on 347; endpoint-only on 241. Correct source alone can hide a role error: 42 of the 359 full successes had a different query or fact endpoint from the requested pair. There were 461 full abstentions.

Given both correct full word spans on 627 new prompts, 317 reached the requested source. The following counts are conditional on correct spans, separated by connector vocabulary and actual token count.

| Vocabulary | Actual length | Selected length 1 | Selected length 2 | Selected length 3 |
|---|---:|---:|---:|---:|
| Shared | 1 | 109 | 0 | 0 |
| Shared | 2 | 54 | 49 | 0 |
| Shared | 3 | 48 | 0 | 36 |
| Held-out vocabulary | 1 | 123 | 0 | 4 |
| Held-out vocabulary | 2 | 105 | 0 | 0 |
| Held-out vocabulary | 3 | 0 | 99 | 0 |

The held-out two-token “chose” and three-token “also chose” connectors both terminate in the same BPE piece. Every correct role binding on these two conditions placed the source on that terminal piece, one position early. The shared connectors all start with the same token. The shorter candidate can match that prefix exactly and is graded on the intervening token's distance to a learned first-value root. A word-start eligibility control would address the within-word error. The shared-prefix error can survive that intervention because the next connector word also begins at a word start.

On the earlier 288-prompt panel, passing the full entity endpoints with the *frozen* bridge repaired all 73 cases previously joined on the leading name piece, giving 147 correct sources. The full variable-bridge arm repaired 42 of the 73, giving 116 correct sources. On the original 16 prompts, the full arm still missed all requested values despite nine correct complete entity joins.

## Reproduction

Use the standalone span-selector binary on Wick with the CUDA feature span-selector and the registered AOT compute capability. See [finite provenance](../finite-span/identity.txt) and [state and prediction manifests](outputs.sha256). The manifests retain the original Wick-relative paths. Run from the Wick experiment directory; replace the state output, prediction output and input paths with their per-seed paths. All outputs are created new.

```sh
span-selector fixture
span-selector fit MODEL TOKENIZER TRAIN STATE_OUT
span-selector predict MODEL STATE BASELINE_STATE BARE_PREFIXES BASELINE_PICKS PREDICTIONS_OUT
```
