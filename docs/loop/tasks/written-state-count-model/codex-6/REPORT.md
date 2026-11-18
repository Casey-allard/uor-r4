# [codex] Value-word boundaries and full ordered connector matching on Wick

## What did not work, first

Strict ordered matching lost 63 correct sources on the primary panel and broke even on fresh transfer. Combining it with the word-start check selected exactly the same addresses, including abstentions, on all 7,520 prompt/seed rows. On correctly joined names, both checks accepted “only” in every one of 152 fresh “found only” cases and “and” in every one of 96 “found and kept” cases. The learned connector “found” is a complete registered sequence, and the next token begins a word. The true connector continues. WordStart alone passed the directional and role-pair criteria on both panels and seeds, but its conditional extraction accuracy missed the registered 75% bar on longer connectors. All arms missed an exact reply ending immediately in EOS. No P3 or conversational milestone change.

## What changed

The four experimental arms share one frozen #4089 query anchor, fact anchor, and both ends of each matched entity. Current uses the prior span selector. WordStart keeps its geometric score and tie rules and excludes candidate value starts whose BPE class is outside the frozen alphabetic START class. Ordered takes the longest exactly registered raw token-ID sequence immediately following the fact and preceding the proposed value. Both applies START before taking the longest exact match. Each treatment abstains if it has no eligible address. The prior writer then uses native generation on the same prefix. The six ordered sequences were collected on CUDA from the identical #4089 TRAIN set, two at each length, occupying 96 bytes of integer payload. No fresh example participated in registration or fitting. The native schema records four source decisions and three eligibility masks per prompt. Oracle and Disabled are additional controls. [Protocol](SHEET.md) · [Input checks](PANEL.md) · [FIT](FIT.md) · [Release](PRE-RUN.md)

## Measurements

Both released seeds gave these counts separately. Current is the #4089 Span arm. A role pair succeeds only if both requested occurrences were selected on the two questions with identical facts. Each panel has 1,728 cases, 864 role pairs and 576 triplets. The first wrong index counts emitted BPE IDs from zero against the value plus EOS. “Tied” compares the index and can include a different wrong ID at that index.

| Panel and arm | Correct value addresses | Both roles | First error later / tied / earlier than Current |
|---|---:|---:|---:|
| Primary, Current | 359 | 62 | 0 / 1728 / 0 |
| Primary, WordStart | 423 | 82 | 64 / 1664 / 0 |
| Primary, Ordered | 296 | 105 | 185 / 1310 / 233 |
| Primary, Both | 296 | 105 | 185 / 1310 / 233 |
| Fresh, Current | 483 | 124 | 0 / 1728 / 0 |
| Fresh, WordStart | 659 | 199 | 183 / 1545 / 0 |
| Fresh, Ordered | 483 | 163 | 96 / 1548 / 84 |
| Fresh, Both | 483 | 163 | 96 / 1548 / 84 |

WordStart repaired 64 requested values on primary; all moved 0→1. On fresh it gained 176 requested value addresses and lost zero. Of 183 later errors, 117 moved 0→1 and 66 moved 0→2. Seven of the later errors came from a distractor with a shared opening. Ordered on primary moved 83 errors 0→1 (distractor), 102 errors 0→2 (requested), 173 errors 1→0 and 60 errors 2→0. It gained 102 correct addresses and lost 165. On fresh it moved 12 errors 0→1 (distractor), 84 errors 0→2 (requested), and 84 errors 1→0. Every earlier error under Ordered resulted from abstaining and returning to native generation. The primary/fresh abstention counts per seed were 461/452 for Current and WordStart, and 1,181/900 for Ordered and Both. Requiring a registered connector raised both-role pair counts even where it reduced individual hits. All-three-length triplets: Current 43 primary / 57 fresh, WordStart 43 / 100, Ordered and Both 84 / 99.

**WordStart by length.** On primary the one- and three-token strata retained every first-error index. All 64 later errors occurred at connector length two. Conditional on the original two anchors being in the correct names at corresponding offsets, WordStart reached 232/236, 109/208 and 36/183 correct addresses at lengths 1/2/3. On fresh it reached 316/415, 144/379 and 199/337. For cases with a correctly recovered entity and a fully registered gold connector, ordered matching reached 296/296 on primary and 483/483 on fresh. Under those same correct-entity conditions fresh Ordered had 0/248 with only a registered proper prefix and 0/400 with no registered prefix. The strict rule improves its measured domain and sacrifices coverage. [Primary transition matrices](rollouts/seed1-primary-summary.json) · [Fresh transition matrices](rollouts/seed1-fresh-summary.json)

**Token traces.** In “Joe took pink. Leo chose yellow. About Leo, which item was listed?”, expected [1344,1]. Current copied “ose” at position 10 and generated [1935,1344,16,1]. WordStart chose position 11, yielding [1344,16,1], so first wrong moved from “ose” at 0 to period where EOS was expected at 1. Of the 105 correctly aligned two-token “chose” primary cases, WordStart recovered 60 and overshot by one token to the second word of the value in 45. With “also chose”, all 99 formerly on “ose” moved to “ ch” (641), a legal word start inside the connector. The wrong token changed and the first-error index stayed zero. Ordered and Both abstained on all 204.

With “Leo has kept round coin”, the old source began “ kept” (1095). Ordered and Both started on “ round” and emitted [1623,2126,16,1]. The first wrong moved 0→2. By contrast, in “Jim found white. Jen found only gold. About Jen, which item did the note list?”, all four selectors emitted [1370,2551,16,1] at selected position 10, one early. The target was [2551,1]. Only the one-token connector “found” was registered there. “only” (1370) is a legitimate word start and an incorrect start of the answer. [Case traces, primary](rollouts/seed1-primary-cases.tsv) · [Case traces, fresh](rollouts/seed1-fresh-cases.tsv)

**Historical checks and release.** On the earlier 288 one-connector prompts WordStart retained 116 correct addresses, while Ordered and Both abstained on all 288. On the original 16 prompts WordStart recovered 5 of the nine correctly joined “chose” misses and moved first wrong 0→1 on five. Current, Ordered and Both reached zero correct addresses there. The exact writer copied every correctly addressed fresh value, and the oracle copied all values on all panels. After all 423 primary WordStart successes, free generation emitted period 16. After the 659 fresh WordStart successes, it emitted period on 632, exclamation ID 3 on 26, and ID 436 on one, at both seeds. Every case required EOS 1 at this step. [Historical controls](HISTORICAL-CONTROLS.json) · [Source-stage report](value-selector/README.md)

## Checks run at the head

The two selector fits took 12 seconds total and all 7,520 independent CUDA and Rust prediction replays matched on baseline and intervention sources and masks. Endpoints were copied from the old span decisions; every historical and fresh decision was independently checked against the frozen implementation. The writer compiled to the identical AOT cubin b41d72ce289ff5daf29055687e2586929321fed6ddb800d10cddfc650e2eeb28. Exact writer transitions were checked on Wick against integer serving on the identical self-fed prefix. The 45,120 trajectories contained 309,325 transitions. All eight writer files and all eight selector prediction files matched separate-process repeats byte for byte. Current, Oracle and Disabled reproduced 12,192 historical trajectories and all 79,760 historical control transitions exactly. The runner refused a mismatched prediction file; the independent summarizer rejected a planted word-start mask error. [Sealed writer inputs](BLIND-INPUTS.json) · [Primary and repeat hashes](writer/SHA256SUMS)

The private evidence bundle contains input tapes, previous span states/predictions, the new states and predictions, both new writer runs, executable and CUDA identities, transfer censuses and run logs. Its downloaded copy matched SHA-256 311e21dfca6977284a1d3811a10c9c4d1baf2599d7b8c2568fd257d5558f74b1. [Wick bundle](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/58b98f50068eafbfe97bd8f89508757a0d63a49b/results/codex/value-connectors-20261118/value-connectors-wick.tar.gz)

Reproduction from the bundled data uses the frozen prediction command described in the source-stage report and the following CUDA scorer command with a new output file. The compiled manifest binds the model, grade, both states and both blind selection files.

```sh
d11-scorer value-rollout MODEL GRADE SPAN_STATE SPAN_PICKS EXACT_STATE VALUE_PICKS NEW_REPORT
```

## Skills used

The native-space cycle supplied the integer-state representation, bit-identical route checks, fixed-state serialization and serving differential contract.

## Still open

WordStart is the only arm to clear the preregistered default-candidate criterion. This is evidence on alphabetic BPE values and these connector families. A value token can begin a word inside either the connector or a multiword answer. Exact matching is informative when the complete connector is in the bank, but the model needs evidence for distinguishing that event from an unregistered extension of a short known connector. The tests did not qualify a role-selection, release or general chat change.

## Plain summary

Rejecting interior word pieces delayed first errors on both panels without any earlier first errors. Exact connector matches corrected familiar longer phrases and also suppressed previously correct addresses on unfamiliar phrases. Adding the word filter to that strict rule changed zero final addresses in these runs.
