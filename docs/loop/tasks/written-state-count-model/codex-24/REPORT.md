# [codex] First held clause-local prototype fit

## What did not work

The position-sensitive clause reader identified the exact requested set on 192 of 1,024 unambiguous prompts. The order-neutral reader got 320. A constant empty answer got 384. On the 640 prompts requesting at least one person, each fitted reader got 96 exactly right. Position increased false requests on empty cases from 160 to 288. Both failed the registered advance decision. Keep the current served role policy. [Decision criteria](SHEET.md) · [Independent raw regrade](analysis/raw-audit/README.md)

Even the supplied correct person failed to yield an exact reply on the four fixed writer cases. All four share the two facts about Emily’s brass lamps and William’s paper kites. The ordinary client selected the second person’s interior source 23 on three cases and emitted 402 instead of the required 975 at output 0. It emitted 1155 at output 0 on the other. The positional fit supplied the correct Emily span on one case. The downstream reader still withheld the source, and the first mismatch remained at 0. With gold onset 12 the original cursor emitted [975, 560, 2527, 85, 1] and stopped at EOS on all four. [Fixed-writer cases and provenance](writer/README.md)

The gold-person trace localizes the Emily failure. Her assertion was found, the original word-start mask admitted only the correct onset at 12, and continuation retained it. The frozen ordered bank had a positive exemplar exactly on its radius (124). It had no negative exemplar for the candidate’s connector depth, word width and class signature, so it returned unseen. That abstention replaced a reachable source with generic generation. [Native onset and bank words](writer/source-witness.json)

## What changed

The CUDA fitter accumulated three integer class sums and counts from 1,024 TRAIN prompts. Each equal-size 800-byte bank represents the three exact class means. Prediction reloads those bytes, compares exact rational L1 distances and abstains on a tied minimum. Both actions use the same radius-12 E8 projector, original token IDs, candidates, clause fences and sampled language. The held split changes names, objects and full wording. Half the unambiguous prompts retain keep/drop cue words. Half change the cue. A punctuation-bounded query and a title-case whole-word filter provide the candidate mentions. These are diagnostic inputs to the role comparison. [Geometry and state](device/README.md) · [Sealed panel and eligibility](panel/README.md)

## Measurements

| Held result per artifact | Identity | Positional |
|---|---:|---:|
| Exact requested set, 384 no-request prompts | 224 | 96 |
| Exact requested set, 512 single requests | 64 | 64 |
| Exact requested set, 128 dual requests | 32 | 32 |
| All mention roles correct, 1,024 prompts | 256 | 96 |
| Correctly recognized unsupported requested mentions, 192 | 48 | 48 |
| Correctly recognized supported requested mentions, 576 | 144 | 144 |
| Both polarities correct on a requested-set swap, 128 prompt pairs | 0 | 32 |

Scoring Positional in place of Identity gained 160 exact sets and lost 288. On a clause-order-only flip each reader preserved its chosen set on all 512 pairs. Fact-order-only flips gave the same invariance. For the 32 separately unresolved choices, Identity emitted zero / one / two people on 0 / 20 / 12 prompts and Positional on 4 / 28 / 0. These prompts have no unique requested-set gold. [Complete paired counts and confusion matrices](evidence/per-artifact.json)

| Wording form, 256 prompts each | Identity exact | Positional exact |
|---|---:|---:|
| Cue before person, new wording and directive | 96 | 0 |
| More distant cue, keep/drop retained | 96 | 0 |
| Opposite cues present, their order gives polarity | 128 | 96 |
| Cue after person, new wording and directive | 0 | 96 |

On the same-root-order challenge, all 64 matched person-level polarity pairs had equal Identity vectors and different Positional vectors. Neither three-centroid reader got both sides correct on any of those 64 pairs. On the unambiguous panel the 2,048 mention rows collapse to 11 distinct Identity or 12 Positional vectors. Exhaustively checking consistency of gold request sets at those addresses yields ceilings of 896 and 1,024 exact prompts respectively. This bound uses held labels and supplies no deployable decision rule. The two original seed artifacts emitted byte-identical projection tables and features, so both runs exercise the same geometry and language. [Exhaustive ceiling and raw-word verifier](analysis/raw-audit/README.md)

### Two contextual boundary witnesses

Two independent readings of the full noun phrases agreed on pairs where preceding context changes the answer to an identical sentence. In “Morgan can help train drivers,” the railway payroll context calls for “train drivers,” and the driving-instructor context calls for “drivers.” Both starts reached the frozen reader at offsets +3 and +4 from the final person token. The relative candidate descriptors were identical, and it chose +3 both times. The payroll context also introduced another Morgan-headed assertion; the final binder had one complete value across two assertions and withheld its source.

In “Jo can help guide dogs,” the assistance-animal context calls for “guide dogs,” and the evacuation-drill context calls for “dogs.” The tokenizer split “guide” into two pieces. The late gold onset moved to +5, outside the reader’s three possible starts at +2, +3 and +4. Both readings selected +3. The cursor could copy all four intended answers through EOS when given their original starts. These are finite witnesses with a supplied query occurrence. The relative scoring descriptors matched within each pair; the original absolute offsets and full prompt tapes differed. A request for the complete phrase matters because a head noun alone can identify a recipient in ordinary conversation. [Original tapes, candidates and cursor replay](boundary/witness.json)

## Checks run at the head

The Wick build produced ahead-of-time CUDA 12.8.93 SM89 executables. On the leased RTX 4090, a finite 3-class fixture matched 99 state words and 24 prediction words, changed winners after a training-label permutation, preserved ties and refused eight malformed inputs. For each seed and arm every raw projected word agreed with the Rust projector on TRAIN and held. Two new-process fits and predictions agreed on bank bytes, coordinates and decisions. The independent verifier reconstructed all 33,792 held class/distance words from the actual banks. The stage excluded held grades, metadata and source answers. A swapped bank failed the bound digest check. Each of four ordinary protocol-2 CLI calls reproduced the native baseline path through every writer step. The GPU lease terminated after 156 seconds at $0.869/h, approximately $0.038. [FIT](FIT.md) · [Pre-run review](PRE-RUN.md) · [Original evidence custody](evidence/storage.json)

## Skills used

The native-space cycle supplied the integer state, original address, actor identity and raw-word replay checks.

## Still open

D15’s joint role-and-reply prerequisite remains 248/256 and the last registered supplied-person result remains 232/256. D24 is a separate role fit. The immediate role question is whether realistic TRAIN coverage and a representation that retains multiple examples per role transfer better at equal model bytes than a single class mean. Use genuinely new wordings, keep incidental, absent and multiple positives, and require an advantage over the empty and positional controls. For the source reader, test address coverage and context separately. On fresh examples enumerate complete word starts up to the assertion fence, applying the existing copy-length limit, so tokenizer segmentation cannot hide a late candidate. With both starts present, supply or ablate a geometric state from the preceding sentence while holding the local target unchanged. Test whether the choice follows the contextual switch. For the independent unseen-bank failure, fit both genuine positive and negative examples of the observed candidate shape on separate TRAIN material, then test held onsets and wrong releases at equal bytes. Avoid globally lifting the veto. The Morgan witness also requires the binder to distinguish the relevant assertion from an incidental fact about the same person. P1(a,b), the four equal-deployment-byte P2 arms, and useful ordinary dialogue retain their original gates.

## Plain summary

The positional representation preserved role distinctions that this fitted read failed to transfer to new wording. On the finite writer cases the gold person was insufficient, and the gold value start restored every reply through EOS. Fitting more instances of the same four prompt patterns would increase the row count while reusing the same small set of geometric observations.
