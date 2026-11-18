# [codex] Connector continuation and EOS after exact copy on Wick

## What did not work, first

Advancing to a later word can discard the first word of a correct value. On #4090’s previously inspected fresh panel, the continuation operator lost 83 correct addresses per seed. Every loss skipped “old”: 42 instances of “old lamp” and 41 of “old ring.” On the 288-prompt historical regression it lost 42 correct starts. A rule restricted to forward moves also cannot recover an address that WordStart already placed too late. Both interventions leave the previously frozen entity choice in place.

## What changed

W reproduces the #4090 WordStart baseline. C retains W unless a longer complete registered connector supports moving forward, or contiguous pieces from W are strictly closer to a TRAIN connector-piece root than to a TRAIN value-onset root and there is a legal landing. The comparison uses the four-root SpiralCore integer orbit distance, bounded by the frozen three-token connector window. E emits EOS ID 1 on the device step where the original copy cursor, after making progress, encounters one of the original source delimiters. CE enables both. At a copy cap, fence, empty copy or abstention, the existing writer controls the outcome. Neither intervention fits new weights. Both seed artifacts use the same measured word/cursor operators. [Protocol](SHEET.md) · [Independent FIT](FIT.md) · [Pre-run release](PRE-RUN.md) · [Native selector](selector/README.md) · [Writer](writer/README.md)

The primary 1,728 prompts and the earlier 1,728-prompt #4090 transfer set were already studied. The separate new transfer set has 1,728 additional prompts with new question wording and two partly or wholly unfamiliar connector families. Its values differ from TRAIN and the two earlier large panels. The sheet described the values as unseen; one of four answer strings, “red” ([677,1]), also appeared in four of the original 16 historical prompts. Every transfer prompt is distinct. In the 432 transfer prompts targeting red, E and CE each give 204 exact replies. In the other 1,296, E gives 674 and CE gives 864; all 190 additional exact replies came from this latter group. Each large panel contains 864 role swaps, with the two connectors independently varied from one to three tokens. Prior (288) and original (16) are historical regressions. Each seed saw exactly the same prompts; report per-seed counts. [Transfer census](TRANSFER-CENSUS.json) · [Grader and case files](rollouts/README.md)

## Measurements

Indices count emitted BPE IDs from zero, and an exact reply must emit the complete requested value immediately followed by EOS, with equal length. “Later” includes removal of the last error. A shared first token can move an error later while the selected value remains wrong.

| Arm | Primary later / tied / earlier vs W | Exact / 1,728 | New transfer later / tied / earlier vs W | Exact / 1,728 | Earlier #4090 transfer later / tied / earlier vs W | Exact / 1,728 |
|---|---:|---:|---:|---:|---:|---:|
| W | 0 / 1,728 / 0 | 0 | 0 / 1,728 / 0 | 0 | 0 / 1,728 / 0 | 0 |
| C | 205 / 1,523 / 0 | 0 | 190 / 1,538 / 0 | 0 | 237 / 1,396 / 95 | 0 |
| E | 423 / 1,305 / 0 | 423 | 878 / 850 / 0 | 878 | 659 / 1,069 / 0 | 659 |
| CE | 628 / 1,100 / 0 | 545 | 1,068 / 660 / 0 | 1,068 | 813 / 820 / 95 | 801 |

On primary, C moved 103 errors from 0→1 and 102 from 0→2. Of the 205 later errors, 122 recovered the requested address and 83 shared a distractor opening. E moved 311 errors from 1→exact and 112 from 2→exact. CE additionally moved 122 from 0→exact and 83 from 0→1. Both-role exact pairs rose from 82/864 under E to 151/864 under CE.

On new transfer, C moved 70 from 0→1 and 120 from 0→2. E resolved 458 at index 1 and 420 at index 2. CE resolved those 878 plus 190 previously wrong at 0; its 660 remaining errors all occur at token 0. They comprise 563 abstentions, 28 distractor addresses and 69 other wrong addresses. Both-role exact pairs rose from 298/864 to 378/864. The signed exact interaction CE−C−E+W is +122 primary, +190 new transfer and +142 earlier transfer; the earlier transfer gained 225 exact replies and lost 83 relative to E. All exacts under E and CE on these panels address the requested value.

Deleting each connector family in turn retained a positive C net movement on primary (+20 to +185) and never a negative on new transfer (0 to +190). The CE−E exact gain ranged from +20 to +102 on primary and from 0 to +190 on new transfer, all counts per reduced panel. All 190 new-transfer gains came from geometric continuation on the short-prefix family; 94 followed “found just” and 96 followed “found and hid”. The matched set contained both direction-of-query cases for every family. The gains establish transfer to these two new extensions; the other connector families supplied no incremental exact gain.

## Traces

With “Leo has kept round coin,” the reference is [1623,2126,1]. W selects position 6 and emits [1095,1623,2126,16,1]; the first wrong token is “ kept” at 0. C selects position 7 and emits [1623,2126,16,1]; the first wrong token is the period at 2. E keeps position 6 and emits [1095,1623,2126,1], still wrong at 0. CE emits [1623,2126,1] exactly.

With “Sam found red. Tom found just black” and a request for Tom, the reference is [1588,1]. W emits [801,1588,16,1], C emits [1588,16,1], E emits [801,1588,1], and CE emits [1588,1]. WordStart begins on “ just” (801). The C witness comes entirely from the frozen geometric distance comparison. On another transfer prompt asking for Sam after “Sam found just red,” CE retains “ just” and starts [801,677,1], still wrong at 0. [Primary cases](rollouts/seed1-primary-cases.tsv) · [New transfer cases](rollouts/seed1-transfer-cases.tsv) · [Earlier transfer cases](rollouts/seed1-fresh-cases.tsv)

The opposite failure occurs in “Jen has old ring. Jim has old lamp. Skip Jim. About Jen, which item did the note list?” The requested value begins at 6, ID 782 (“ old”). E emits [782,2291,1] exactly. C advances to 7 and CE emits [2291,1] (“ ring”, EOS), wrong at token 0. W’s original first error was the period at index 2. On this earlier transfer the 95 earlier errors comprise 83 moves from 2→0 and 12 from 1→0. On the historical 288 prompts C moved 62 errors earlier, none later, and CE fell to 74 exact versus E’s 116. On the original 16, E and CE each produced five exact.

## Checks run at the head

Both seed runs produced the same paired decision counts. The four real arms ran with the grade file withheld for 43,904 trajectories (234,193 CUDA/serving integer transitions); the subsequent graded run reproduced every blinded trajectory’s digest. The seven-arm scored run covered 10,976 prompt/seed rows, 76,832 trajectories and 417,607 device/serving transitions. All 22,560 historical WordStart, Oracle and Disabled trajectories reproduced their #4090 records, including 146,381 writer transitions. The gold-source Oracle+EOS was exact on every measured prompt using the same eight-token copy limit as the other arms. The native-only and original oracle arms produced zero replies exact through EOS. The independent grader rejected a planted period in place of EOS. All ten independently repeated graded files and all ten repeated selector files matched their originals byte for byte. All 192 integer library tests passed, as did the serving-wasm host feature check. See [raw-word finite tests](writer/finite/seed1.json), [input fingerprints](BLIND-INPUTS.json) and [paired counts](rollouts/README.md).

The [sealed Wick bundle](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/b05051bc96a52394f77a4e21818f39482caaa104/results/codex/continuation-eos-20261118/continuation-eos-wick.tar.gz) contains the new tapes, source predictions, bare witnesses, primary and repeat writer outputs, fixtures, build logs and run commands. The downloaded file matched SHA-256 `cac8ff75f3efbe576b6d1f61167ad2ce12dde59b05927b1ed32cf70bd9cd2482`. The unchanged historical inputs are in the [previous Wick bundle](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/58b98f50068eafbfe97bd8f89508757a0d63a49b/results/codex/value-connectors-20261118/value-connectors-wick.tar.gz).

## Skills used

The native-space-cycle skill supplied the shared integer state, committed-token replay and device/serving equality requirements.

## Still open

Continuation and end-of-copy EOS both cleared their preregistered follow-up criteria on primary and new transfer. The stopping intervention fixes a distinct error after a correctly addressed copy in this short-answer format. The geometry of “old” still triggers an unwanted advance, and the training bank supplies uneven evidence across connectors and answer openings. Of the 660 residual transfer errors, 591 have at least one incorrectly located or missing frozen entity boundary; 69 occur with all four boundaries correct. For the latter group the unresolved connector counts are 30 “found just”, 30 “found and hid”, and 9 “held”. Two correctly emitted replies occur in cases whose entity boundaries disagree with gold; their selected value address is nevertheless correct. The split preserves the distinction between locating the entity and locating the value after it. Every measured target requests a bare item plus EOS; deciding when prose should continue after quoting an item remains open.

## Plain summary

On the new 1,728-prompt transfer set, continuation alone recovered 190 requested sources but left the first error at the native token after the answer. EOS alone converted 878 already-correct sources into exact replies. Together the two changes yielded 1,068 exact replies, and every remaining reply diverged on its first token. The serving default and P3 status retain their prior settings.
