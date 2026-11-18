# [codex] Entity span endpoints and one- to three-token connectors

## What did not work, first

At the two- and three-token lengths the new selector missed the registered 75% extraction bar, even when the frozen query and fact anchors identified the right person. Both word spans were then exact in 627/627 cases, yet correct value starts were 232/236, 49/208 and 36/183 at lengths one, two and three. Of the 201 prompts whose first error moved later, 67 still addressed the wrong source. The original nine correctly matched names with a two-piece “ chose” connector still started the copy one token early. No arm generated an exact value immediately followed by EOS on any of the three panels. These outcomes leave P3 and discretionary copy/start/stop control open.

## What changed

Keep the #4087 geometric role choice, D11 state, greedy native generator and #4086 exact writer. The new selector carries seven u32 fields: original query/fact anchors, inclusive query and fact starts and ends, and selected source. It derives word boundaries from three token classes in the bound release tokenizer (ASCII alphabetic start, continuation, other). Corresponding whole occurrences must have identical token IDs at the same anchor offset. The endpoint arm abstains on an incompatible span. A new connector bank stores length, ordered packed E8 roots and a first-value root learned on TRAIN. Compare the candidate lengths using rational integer scores; on exact ties prefer the longer length. This is an explicit word-boundary prior plus trained connector and value-opening evidence. Evaluation used a prefix-only handoff after fitting, then replayed each seed and arm on its own emitted IDs.

Two separate 1,728-row splits, two tokenizers, 864 role pairs and 576 matched triplets per split. Removing the requested connector left the rest of each triplet identical in token IDs. The distractor connector was varied independently. Held-out has 864 shared connector surfaces and 864 split-specific surfaces. Surface novelty is crossed with length and is tied to value assignment in this design. The two seeds score the same texts. [Panel and factor definitions](PANEL.md) · [Pre-registration](SHEET.md) · [FIT](FIT.md) · [Run authorization](PRE-RUN.md)

## Measurements

Each of the two seeds gave the following counts on 1,728 new held-out prompts. Correct start means the requested written occurrence. First error counts use the integer reference (one- or two-ID value followed by EOS), starting at index zero.

| Selection | Correct start | Both roles / 864 | All three connector lengths / 576 triplets | First wrong at 0 / 1 / 2 |
|---|---:|---:|---:|---:|
| Current #4087 content selector | 229 | 23 | 0 | 1451 / 184 / 93 |
| Entity endpoints only | 241 | 23 | 0 | 1391 / 238 / 99 |
| Variable connector only | 347 | 62 | 43 | 1314 / 308 / 106 |
| Entity endpoints + variable connector | 359 | 62 | 43 | 1254 / 362 / 112 |
| Oracle value start | 1728 | 864 | 576 | 0 / 864 / 864 |
| Copy disabled | 0 enabled | 0 | 0 | 1728 / 0 / 0 |

The combined arm passed both primary directional tests against the current selector. Its first error moved later on **201** prompts (182 from 0→1, 19 from 0→2), tied on **1,523**, and moved earlier on **4** (1→0). It gained 134 previously missed sources and lost four hits. On 115 wrong sources it still produced the correct first ID. Of its 359 correct value addresses, 42 came from choices that did not also identify both full requested name spans. The connector-only arm reached the same aggregate both-role and all-three counts as the combined arm.

| Requested connector | Current → combined correct starts / 576 | Later / tied / earlier error | Correct combined source given corresponding anchors |
|---|---:|---:|---:|
| 1 ID | 224 → 232 | 35 / 537 / 4 | 232/236 |
| 2 IDs | 5 → 54 | 72 / 504 / 0 | 49/208 |
| 3 IDs | 0 → 73 | 94 / 482 / 0 | 36/183 |

The scoring error has a specific direction. Of the two-token correctly anchored cases, 159 stopped one token early. Of the three-token group, 48 stopped two early and 99 stopped one early. On the held-out split-specific “chose” (two IDs) and “also chose” (three IDs) connectors all 105 and 99 correctly anchored cases selected the final piece of “chose” as the first value ID. Shared surfaces also produced early stops: 49/103 correct for “has kept”, 36/84 for “has just kept”. All four one-token regressions with correct anchors instead overshot by two. These are designed surface-level results.

Examples on seed 1 (same selection and first-error outcomes on seed 2):

- Leo, “has just kept”, expected pink [1481,1]. The frozen selector handed over position 10 and wrote [801,1095,1481,16,1]. The combined selector supplied position 12 and wrote [1481,16,1]. First wrong moved 0→1, from “ just” before the answer to period instead of EOS.
- Leo, “also chose”, expected yellow [1344,1]. Both selectors found the same fact at position 8. The old output began [641,1935]; the new source was 11 and output began [1935,1344]. The requested value starts at 12. Moving the address one token closer left first wrong at index 0, now “ose”.
- Rana, “has”, expected “ round stone” [1623,1881,1]. The frozen output began with the final name piece [67,1139,1623,1881,…]; the combined arm wrote [1623,1881,16,1]. First wrong moved 0→2.
- On all 359 correctly addressed combined-arm values, the exact writer completed the value and native generation released to period ID 16 instead of EOS ID 1. The oracle copied 1,728/1,728 values; 1,727 then received a period. In one case native generation instead continued from ID 86 (“t”) to the decode cap.

[Seed 1 paired token traces](rollouts/seed1-new-cases.tsv) · [Seed 2 paired token traces](rollouts/seed2-new-cases.tsv) · [Per-length and surface counts](rollouts/seed1-new-summary.json)

## Historical cohorts

On #4087’s 288 held-out prompts the old selector addressed 74 values. Endpoint-only reached 147, connector-only 74, and combined 116. Specifically, endpoint-only repaired all **73/73** known leading-name-piece cases; the new connector score preserved **42/73** repairs and discarded 31. On the 16 original prompts all four selector arms still had zero correct source addresses. The combined arm had 15 ties and one earlier first error, losing an accidental shared opening. All nine previously correct name joins recovered both word spans but still selected “ose”. [Historical cohort summaries](rollouts/README.md) · [Immutable control comparisons](HISTORICAL-CONTROLS.json)

## Checks run at the head

The model and tokenizer identities, CUDA prototype fit, word classes, scores, ties and 21-word selection parity are in [the selector receipts](span-selector/README.md). Both seed fits yielded 18 distinct connector/value-root prototypes, six at each length. TRAIN fitting took 12 seconds total, and primary blind prediction for both seeds and all three panels took 36 seconds, excluding the separately executed baseline run. Every prediction file matched independent-process replay. Both old-state files, span-state files, blind outputs, artifacts and grade arrays were bound before writer execution. [Blind input digests](BLIND-INPUTS.json)

The unmodified writer cubin was b41d72ce289ff5daf29055687e2586929321fed6ddb800d10cddfc650e2eeb28. Per seed the six-arm new-panel run took 23.47/23.17 seconds on Wick including native driver and Rust differential checks. All six primary writer files matched fresh-process repeats. Verified 146,416 writer transitions on 24,384 trajectories, with source IDs, emitted IDs, native readout, cursor and release compared at each step. The 1,824 historical controls matched the earlier outputs on all 12,097 transitions. The final bounds-hardened selector rejected a malformed maximum address and reproduced all six blind prediction files; final writer verification and mismatched-input refusal are in [the verification receipts](verification/verified.sha256).

The pinned private store bundle includes tokenized inputs, baselines, both fits, all blind predictions, primary and repeat raw steps, binary identities and run logs. Retrieved copy matched SHA-256 c601f8ff97db3e88bad72e6f39c5d6c4b9ba72b008a3cdea2219765e0f20d8a0. [Wick evidence bundle](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/a46df86d18c1182375000d0f9ca11fd0edc797d7/results/codex/span-connectors-20261118/span-connectors-wick.tar.gz)

## Skills used

The native-space cycle supplied the representation, serialization and raw-bit replay contract. The exact writer was verified once more against the historical controls.

## Still open

Keep matched name endpoints as a separately measurable component. The next discriminating experiment is to test whether requiring a valid value-word start eliminates copying “ose”, and whether ordered evidence for the complete connector distinguishes a longer connector from its valid shorter prefix. Test those two changes separately on fresh word splits and new connector families. A value-word-start check alone would still admit the word “kept” inside “has just kept”. Role selection and when native generation should terminate remain separate unresolved stages.

## Plain summary

Carrying the end of a name repaired the specific name-piece error. The new connector scorer improved the overall paired first-token result but repeatedly returned an address inside the connector. Correct addresses still produced complete copied values, followed by a native continuation that missed the expected EOS.
