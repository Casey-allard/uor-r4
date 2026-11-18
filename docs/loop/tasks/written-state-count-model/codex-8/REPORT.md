# [codex] Earliest entity loss in the 563 source abstentions on Wick

## What did not work, first

The legacy content selector ranked pairs that shared a token before asking whether those tokens belonged to the same complete word. On every one of the 563 #4093 focal abstentions it chose the opening token of “Tava” in one mention and “Tavi” in the other, or vice versa. Matching offsets and disjoint positions passed. The last word piece differed, so the next stage rejected the pair and returned a null source. WordStart and continuation never had an address on these cases.

## What changed

For every prompt, B uses the #4093 chain. R moves the existing disjoint/equal-offset/full-token-sequence check ahead of the minimum over entity pairs. B and R enumerate the same initial pairs and use the same trained integer query-role and fact-context energies and earliest (query, fact) tie order. If B's winner passes its span check, that winner remains the minimum in R. O supplies only the annotated last positions of the requested query word and its matching fact word. All three use the same untouched downstream span operator, WordStart, geometric continuation, exact copy and EOS release. Only O has access to labels; it still has to choose the value following the entity. [Sheet](SHEET.md) · [Independent FIT](FIT.md) · [Staged release](PRE-RUN.md)

The focal cohort is the exact #4093 transfer tape, two released seeds, 1,728 prompts per seed, with its 563 null rows pinned by prefix digest and case ID before new scoring. For the new name challenge each prompt, answer, connector assignment, word position and role have a matched focal prompt. Jim/Jen replace Sam/Tom; Zava/Zavi replace Tava/Tavi. The new pair’s common first token is 1856 instead of 273. Both three-piece pairs share their second piece, 1410, and differ on the last, 67 versus 75. The test checks transfer of the repair to this name substitution within the same question formats. Two earlier large panels supply historical controls. [Challenge census](CHALLENGE-CENSUS.json) · [Independent rank trace](trace/README.md)

## Measurements

The requested full-word pair was available and eligible in 563/563 cases. The first loss was the joint query/fact pair ranking. The eventual abstention occurred one operator later, at the span equality check. The incorrect pair had a strictly lower energy than the best requested pair in 501 cases. In the other 62, the energies tied and the legacy index order chose the wrong pair. The best corresponding requested pair placed second on 340 prompts, third on 137, fourth on 68, fifth on 5 and sixth on 13. Evaluating only the top two old pairs would miss 223 requested pairs. The two occurrences could be confused before the selector found the question: 216 winners paired the requested query with the other fact, 185 paired the requested written fact with the other written fact, 135 paired the other written fact with the requested written fact, and 27 paired the excluded query with the requested fact. These counts describe a joint selection, not two sequential learned decisions.

| Focal group, per seed | Prompts | #4093 exact | Repaired exact | Anchor-only oracle exact | First wrong later / tied / earlier, R vs B |
|---|---:|---:|---:|---:|---:|
| Originally abstained | 563 | 0 | 441 | 527 | 495 / 68 / 0 |
| Previously selected a source | 1,165 | 1,068 | 1,068 | 1,087 | 0 / 1,165 / 0 |
| Full focal panel | 1,728 | 1,068 | 1,509 | 1,614 | 495 / 1,233 / 0 |

Errors count emitted BPE tokens from zero; exact requires matching every requested value ID and immediately emitting EOS 1. All 563 originally failed at token 0. Of these, 441 moved from 0 to exact through EOS, 54 moved from 0 to 1 on a shared distractor opening, and 68 stayed at 0. Every one of the 441 exacts came from the requested value address. The 1,165 baseline non-null cases retained identical sources and complete writer trajectories. Both questions of a matched pair were now exact in 654/864 pairs, up from 378/864; the anchor-only oracle reached 750/864.

With whole-word identity enforced, 473/563 selected the requested query and fact at corresponding offsets. Among them, 439 selected the correct value and 34 still missed downstream. Another 90 selected an incorrect entity correspondence: 72 matched the excluded query to the distractor fact, and 18 matched other words. Two of those 90 nevertheless landed on the requested value address and produced an exact reply. Thus the remaining 122 reply failures comprise 88 wrong-entity outcomes and 34 wrong-value outcomes after a correct entity match. The oracle also misses 36 of the 563. An oracle that supplies the value address would conceal that boundary error. [Source census](entity/source-census.json) · [Writer and grading](writer/README.md)

### Three paired traces

- In “Tava has green box. Tavi has green bag. For Tavi, what item did the record mention?”, B chose query position 20 in Tavi and fact position 4 in Tava. Their energy was 100 (44 query + 56 fact). The correct (20,11) scored 136 (44 + 92), second overall. B rejected its winner and native decoding opened with ID 1155, wrong at 0. With full-word eligibility, (20,11) ranked first; R copied source 15 and emitted [1074,1216,1], “green bag” and EOS.
- With the same two facts and “Leave out Tava. What item did the record mention for Tavi?”, R selected a legitimate occurrence of **the excluded name**, (21,4) with cost 148. The requested pair (36,13) scored 172 and ranked second among eligible pairs. R emitted [1074,616,1] (“green box”), so the first error moved from 0 to 1. The labeled anchor pair produced the requested [1074,1216,1].
- In “Tavi found black. Tava found just red. For Tava, what item did the record mention?”, both R and O reached the requested entity. The frozen value selector began one token too soon and the same writer emitted [801,677,1] (“just red”) against [677,1] (“red”). The first error stayed at 0.

### Controls and transfer

| Panel, 1,728 prompts per seed | B exact | R exact | O exact | R vs B later / tied / earlier |
|---|---:|---:|---:|---:|
| Focal #4093 transfer | 1,068 | 1,509 | 1,614 | 495 / 1,233 / 0 |
| Matched name challenge | 1,070 | 1,503 | 1,602 | 487 / 1,241 / 0 |
| Earlier primary | 545 | 811 | 1,344 | 308 / 1,420 / 0 |
| Earlier #4090 transfer | 801 | 979 | 1,104 | 194 / 1,534 / 0 |

On the 555 original challenge abstentions, 433 moved from token 0 to exact, 54 from 0 to 1 and 68 stayed at 0. Across all challenge prompts, both-role exact pairs rose from 380 to 650 of 864. The registered direction and exact-through-EOS tests passed on both decision panels at both seeds. Both seeds see the same language examples. [Focal paired counts](writer/rollouts/seed1-transfer-summary.json) · [Challenge paired counts](writer/rollouts/seed1-challenge-summary.json)

## Checks run at the head

Wick’s RTX 4090 executed the integer ranker and all downstream operators. At each site the independent serving implementation checked integer scores, winning addresses, complete span, WordStart and continuation; the historical decoder and completion cubins matched #4093. The original three panels have case-level historical writer controls. For the new name panel an independent run of the four previously released selector executables checked the baseline. Every change in final output must trace to the source selection; identical sources were required to produce the same steps. Across four panels at two seeds, 41,472 complete trajectories were scored. The independent Rust and CUDA paths agreed on all 141,850 actual-feedback transitions; 10,368 historical CE trajectories and 50,544 historical transitions reproduced exactly. All 24 selector files matched separate-process repeats byte for byte. Both finite device controls and hostile-input checks ran before the bound writer. Fresh integer library tests (192) and host serving-wasm feature build passed. [Selector receipts](entity/README.md) · [Writer checks](writer/README.md)

All 24 complete writer outputs matched separate-process repeats byte for byte. The raw tapes, primary and repeat picks and writer traces, limited oracle inputs, commands, and diagnostics are in the [sealed Wick evidence](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/a7aa91284de556f70c3f2472e68bedeeab9cb1e5/results/codex/entity-abstention-20261118/entity-abstention-wick.tar.gz). A pinned download reproduced SHA-256 `2ad769cbd249705dcc84a178a016b53c74f82795078549f0c2232c0d9bd55654`. A final rebuild spelled out the tuple destructuring in the legacy adapter after the source scanner mistook its field accessor for a floating literal. The four cubins were byte-identical and all 24 predictions from the final-source binary reproduced the measured files. [Rebuild receipts](entity/final-source) record both binary identities. The unchanged historical inputs are described in [the preceding run](../codex-7/REPORT.md).

## Skills used

The native-space-cycle skill supplied the requirement that the selected routes, addresses, copy cursor and emitted IDs must replay bit-for-bit on GPU and serving implementations.

## Still open

A word may appear in the request, in an exclusion and among the facts. Requiring the complete word removes the partial-name collisions measured here; it does not assign semantic roles to repeated complete words. Among the 563, the largest remaining incorrect entity group contains 72 selections of the excluded name. The 34 failures after a correct entity match instead implicate the unchanged value boundary. The oracle and R overlap on 439 exact replies, O alone is exact on 88 and R alone on two. Keep those distinct when estimating the value of another entity improvement. This is a local experiment on templated short answers. Serving defaults and the broader milestone retain their prior settings.

## Plain summary

Moving the existing whole-name check ahead of ranking recovered an exact answer through EOS for 441 of 563 previously unanswered source selections. The full focal panel gained 441 exact replies without losing any. The next observable role error is selecting the explicitly excluded name even when the word identity is correct.
