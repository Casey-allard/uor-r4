# [codex] Requested entities, value agreement and the first served count predictor

## What did not work, first

The learned selector still chooses the excluded person on all 96 trials of one held wording. It made 64 unsupported copies there. Supplying only the requested query span eliminates those 64. On seven single-fact prompts the correct query and fact still lead to a value address exactly two tokens beyond the gold start. The unchanged writer copies the token at that address. On older ambiguous requests the repaired gate now copies on 48/48. Keep the decision diagnostic. Ordinary conversation remains open. All three newly packaged count predictors emit sixteen spaces in response to each of the four normal text questions. [Wrong source positions](panels/QUERY-VALUE-FAULTS.json) · [Client transcripts](client/RESULT.md)

## What changed

A typed requested-word selector and a distinct support rule now expose query-span loss separately from fact and value errors. The downstream value selector, continuation, cursor and EOS policy were fixed. An additional comparison keeps the same query and permits the earliest address only when every admissible supporting occurrence yields the same complete copied IDs through EOS. It uses real candidate trajectories and increases work with the number of admissible occurrences. Disagreement or an incomplete copy withholds. The supplement followed the primary result and reused its held tape. [Nomination and training state](NOMINEE-RESULT.md) · [Agreement](AGREEMENT-RESULT.md) · [Writer controls](panels/RESULT.md)

The native D11 loader now serves a full-support, normalized integer mixture of order-zero and order-one counts. The conditional weight is learned on separate TRAIN records. One full file and one pruned file were scored on Wick after reload, then run from the client through actual generated feedback. The full state and top-four state came from identical input records. Two exact-size carriers with identical retained keys separate count rounding from the change in key coverage. [Count results](COUNT-RESULT.md) · [Equal-byte dose](equal-bytes/RESULT.md)

## Measurements

| Same 288 held prompts, each of two seeds | Exact of 96 answerable | False copies of 192 unsupported |
|---|---:|---:|
| Historical ungated reader | 54 | 192 |
| Historical action gate | 25 | 88 |
| New nominee with old permission | 54 | 120 |
| Old nominee with unique support | 11 | 64 |
| New nominee with unique support | 26 | 64 |
| New nominee with complete-value agreement | 48 | 64 |
| Query-occurrence oracle with unique support | 41 | 0 |
| Query-occurrence oracle with complete-value agreement | 77 | 0 |
| Forced correct value / read disabled, on answerable rows | 96 / 0 | — |

The joint strict arm gained 15 exact replies over the historical gate and lost 14. In both directions the first discrepancy moved between output index zero and exact EOS. Complete-value agreement then recovered 22 duplicates from index zero with no new false copy on this tape. The known “name” trap went from 24 wrong nominations to zero. Prior unseen exact-through-EOS rose from 794/1,728 to 1,008, still below the ungated 1,406. Two seeds share these constructed questions. They do not double the number of independent prompts. [Paired transcripts](panels/PAIRED-EXAMPLES.json) · [Actual generated outcomes](panels/RESULT.md)

On 21,928,475 held bytes, the unigram scored 4.301066 BPB, unpruned fitted bigram mixture 2.835339, and rounded top-four refitted mixture 3.584614. Restoring the original counts at exactly 86,670,824 bytes and weight 102/256 scored 3.581048. Changing the conditional weight on the full rows first cost 0.317118 BPB. At that weight, deleting rows cost 0.428591 and rounding survivors cost 0.003566. These three signed differences account for the 0.749275 overall cost along this intervention order. All arms have a positive mass for every eligible original token. The unpruned model first misses a teacher-forced target at tape position 2, predicting 201 after 201 when the corpus gives 55. In normal text the first error comes at reply index zero. The final assistant colon is identical across the prompts, and this model reads one preceding ID. [GPU measurements and denominators](count/result.json) · [Crossed weight](count/crossed/) · [Normal client](client/RESULT.md)

## Checks run at the head

Independent FITs, frozen inputs, byte comparisons of repeated selector and actual-feedback writer outputs, CPU/GPU native count mass and greedy parity, corrupted handoff and artifact controls, exact-size reconstruction checks, reloaded D11 client transcripts, focused Rust tests for default prediction, unchanged CLI modes and restored rows. Source and artifact receipts accompany each instrument.

## Skills used

Native-space-cycle and UOR research architect. Integer device and host operators, sealed state words and real generated trajectories underlie the measurements.

## Still open

1. Train requested and excluded roles on swapped counterfactual pairs at fixed facts. Hold out an entire exclusion construction and include requests with no referent. Grade the query endpoints and ambiguous abstention before touching the writer. The current case feature needs a lower-case and capitalized-distractor control.
2. Freeze an oracle query and real fact endpoint. Instrument candidate value starts around the seven +2 errors and the 12 duplicate pairs whose realized copies disagree. Train or derive a local connector/value boundary with a first-invalid-position trace. Preserve positive single-fact replies, genuinely conflicting values, and unsupported abstention on fresh records.
3. Keep the full-support unigram and exact native scorer. Increase context order and vary the number of stored successor keys under a matched total artifact budget. Compare same-byte count and written geometric predictors on held overall and novel contexts, plus prompt-conditioned rank on the assistant boundary. Retain the unpruned matched baseline. The 1% two-order control leaves the official full-corpus P1(b) and stronger prediction targets open.
4. Rejoin qualified role, value and ordinary generation through one saved native state, then measure both role-swapped factual replies and unseen non-copy prose from the normal client. General language, reasoning, client-scale latency and energy remain unqualified. The existing [closure plan](../codex-9/CLOSURE-PLAN.md) ties later commitments to those outcomes. No milestone changed here.

## Plain summary

Exact emission already works when supplied the right value address. Distinct upstream errors now have identifiable prompt positions and measured oracle headroom. Storing more successor evidence improved prediction on continuous text; the ordinary reply path still needs state that distinguishes questions ending in the same assistant marker.
