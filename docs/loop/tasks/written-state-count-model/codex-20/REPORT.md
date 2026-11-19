# [codex] Exclusion-last requested roles and crossed object sources

## What did not work

The user-text-only ranker recovered zero requested occurrences on the 32 fresh supported questions. Both ordinary arms selected the excluded person's query occurrence on every question and produced zero exact replies. Every reply first differed from the expected answer at output token 0. Under this wording, removing assistant framing reduced the rival's lead and left its winner intact.

## What changed

The new, opt-in integer role policy replaces each out-of-user-text lookup in the requested contrast with the existing missing-context root. It preserves the original full prompt, candidate endpoints, assertion-head requirement and separate excluded decision. The ordinary baseline is a byte-identical rebuild of D19 assertion agreement. Both arms embed the same value bank, span and paired-parent bytes. The independent supplied arm replaces only the requested query endpoints. The downstream match, value decision, cursor, token IDs and EOS still come from the frozen machinery. [Implementation](operator/README.md) · [Registered comparison](SHEET.md)

## Measurements

Each frozen seed artifact saw the same 32 supported prompts and six ambiguity controls. Two wording blocks independently crossed two people, two fact orders, two object assignments and two connector assignments. Both names are eligible in every supported row. In the “setting aside the entry for” block the requested and rival contrasts were 36 and 78 at baseline, then 36 and 72 with user-only scoring. In the “leaving out the entry for” block the contrasts were −16 and 58, then −16 and 52. Both forms kept the requested candidate's context. Replacing the final rival's off-text context raised the excluded-role component of its requested contrast from −22 to −16 and lowered the rival contrast by six. The independent excluded decision retained its original words.

| Arm | Correct query occurrence | Original gold value start | Exact through EOS | First wrong at token 0 |
|---|---:|---:|---:|---:|
| D19 assertion agreement | 0/32 | 0/32 | 0/32 | 32/32 |
| User-text-only | 0/32 | 0/32 | 0/32 | 32/32 |
| Correct query span supplied | 32/32 supplied | 4/32 | 4/32 | 28/32 |
| Opposing query span supplied | 0/32 | 0/32 | 0/32 | 32/32 |
| Read disabled | 0/32 | 0/32 | 0/32 | 32/32 |
| Gold value start forced | 32/32 supplied | 32/32 forced | 32/32 | 0/32 |

Supplying the correct span removed the first mismatch entirely on rows 21, 22, 31 and 32. Both the Caspian and Winifred queries used span [45,49), with the original source at position 9 or 20 depending on fact order. On those rows the ordinary trial began with ID 1155 (“ G”). Supplying the query produced [563,2537,1731,85,1], “small glass jars” followed by EOS. On rows 19, 20, 33 and 34 the correct answer started with 564 (“ br”). Supplying the query changed the first wrong ID from 563 (“ small”, the rival's object) to 1155 (the fallback). The other 24 supported tapes stayed wrong and unchanged. The independent excluded span overlapped the supplied requested span on 16 rows; the oracle intervenes on just one role.

| Object | Connector | Exact replies with query supplied | Source decision |
|---|---|---:|---|
| red brass keys | found | 0/4 | Gold in radius, margin −8/2 |
| red brass keys | might take | 0/4 | Gold onset distance 76 exceeds radius 72 |
| silk scarves | found | 0/4 | Gold distance 92 exceeds radius 56 |
| silk scarves | might take | 0/4 | Gold distance 120 exceeds radius 92 |
| brushes | could still find | 0/4 | Gold margin −4/6, another candidate unseen |
| brushes | took | 0/4 | No negative-class support for the gold shape |
| small glass jars | could still find | 0/4 | Gold margin +16/4, earlier rival distance 52 exceeds radius 48 |
| small glass jars | took | 4/4 | Original start selected, margin +16/2 |

Each line crosses two identities and both fact orders. The four unseen “took brushes” rows correspond to the single missing class shape in the pre-run census. [All candidate margins and radii](evidence/value-cells.md)

Switching the connector flipped exactness in four of 16 matched pairs while keeping the expected answer tape fixed. Changing the person while preserving object, connector and target fact slot changed zero of 16 emitted tapes. Reversing the facts also changed zero of 16 emitted tapes. [Complete row table](evidence/span-table.md)

The 32 supplied successes in the query column measure the intervention. Its four independently selected original values are the downstream result. [Per-prompt identities, spans, source positions and first wrong IDs](evidence/rows-seed1.jsonl) · [Second artifact](evidence/rows-seed2.jsonl)

## Checks run at the head

The two independent readers agreed on all 38 prompts before scoring. Both tokenizer tapes and the correct and opposing endpoint preparations passed verification. The packed native payloads and rebuilt baseline matched across the two policies. The live planted perturbation changed the baseline winner from [14,16) to [11,13) and left the treatment at [11,13) on both prompts. All eight role words matched the native ordinary implementation. The writer reproduced the archived control through EOS, and the prior client rejected the new schema. Forced correct source wrote 32/32 exact on held prompts, forced opposing source wrote 0/32, and the independent wrong answer key rejected the unchanged correct tapes on all 32 rows. No factual source was selected on the six ambiguous or unsupported questions in either ordinary arm. The two artifacts differed on the generic acknowledgment for conflict row 17 beginning at offset 7. Within each artifact both ordinary arms emitted identical tapes on all 38 prompts. A generic fallback ended in EOS and was not graded as an answer on those six. [Independent FIT](FIT.md) · [Pre-run review](PRE-RUN.md)

The four-run dispatch took 6 minutes 30.61 seconds. For each seed its primary and fresh-process replay matched on 215 deterministic file hashes and the entire summary. The archive restored 2,451 files and two symbolic links on the workstation. Wick verified the second archive and all 14 model hashes. The private Hugging Face store rejected the combined build-stage and results package at its quota. A compact upload of the held and finite outputs succeeded. Its revision-pinned download reproduced all four raw and cold manifests. The failed SSH allocation and the successful rental were both terminated, with an estimated combined charge of $0.47 at the quoted rates. [Cold replay](evidence/cold-replay.txt) · [Archive readback](evidence/storage.json) · [Compute ledger](evidence/compute.json)

## Skills used

The native-space cycle supplied the identity, artifact, operator and replay checks for the exact integer source and token path.

## Still open

Keep D19 as the ordinary reference. For requested roles, vary which clause is affirmative independently of name order and carry that clause's signed role evidence to its matched entity. Train or select the rule on separate wordings, then cross positive-first, exclusion-first and an explicit positional control on fresh names. For values, the four positive-margin long-connector jar failures isolate a second hypothesis. Compare a geometrically justified connector-end eligibility rule against the frozen all-candidate veto, with the requested span supplied and unchanged value counts, on fresh long-connector pairs. Preserve true unseen-gold and wrong-source controls before changing eligibility. The other six failing combinations require separate coverage or margin evidence. The panel does not measure general chat, training speed or open-ended generation.

## Plain summary

The writer can copy all 32 gold values through EOS when it receives their source positions. Giving it only the correct person was enough on four prompts, all with “took small glass jars.” Both the requested role and the downstream source policy still limit ordinary replies.
