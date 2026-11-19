# [codex] Requested-entity occurrence ranking with the ordered value policy

## What did not work

Putting the exclusion after the target in the “about” construction still selected the named distractor on all 16 supported prompts in that cell. Four requests for Maren (37, 41, 53 and 57) changed from fallback to a wrong factual answer, “small glass jars,” copied from Ivor's fact. Their first error stayed at output token zero. Supplying the correct query endpoints raised the fixed downstream path to 16/64, all for the same object literal.

## What changed

Both people appear in each supported query, and the facts and query name orders are counterbalanced. The panel crosses requested person, factual order, “for/about”, presence of “the” and placement of the exclusion for two fresh fact pairs. Two independent readers agreed on the 64 supported and six unsupported controls. Across the 64 supported rows, 48 request a value absent from frozen value TRAIN and 16 request the previously seen “brushes”. Both seed artifacts replay the same texts.

The treatment admits whole words in the final query clause whose original token IDs match an earlier assertion head. It ranks these occurrences using the frozen sparse requested-role score minus excluded-role score. A unique signed maximum can win even when negative. The existing excluded role, ordered value policy, source mask, bank, original token tape, writer and EOS are fixed. The new D11 schema preserves the original contained bytes. This comparison has no training or optimizer step.

## Measurements

Both seed artifacts produced the same per-prompt span and first-error records. Offsets below are zero-based. The complete original traces also retain differences in the continuation after an error. [All 64 prompts and spans](evidence/seed1-span-table.md) · [Expected and emitted first-mismatch IDs by row](evidence/seed1-spans.jsonl) · [Seed one verification](evidence/seed1-verification.json) · [Seed two verification](evidence/seed2-verification.json)

| Query wording | Supported per seed | Correct query occurrence, baseline → treatment | Exact through EOS, baseline → treatment |
|---|---:|---:|---:|
| “for” with the exclusion last | 16 | 16 → 16 | 4 → 4 |
| “for” with the exclusion first | 16 | 16 → 16 | 4 → 4 |
| “about” with the exclusion last | 16 | 0 → 0 | 0 → 0 |
| “about” with the exclusion first | 16 | 0 → 16 | 0 → 4 |
| Total | 64 | 32 → 48 | 8 → 12 |

Each of the four people gained four correct query occurrences. The 16 recoveries are rows 3, 7, 11, 15, 19, 23, 27, 31, 38, 42, 46, 50, 54, 58, 62 and 66. Neither adding “the” nor reversing the two factual sentences changed the recovery pattern. Before the change 56 replies diverged at zero; afterwards 52 did. Four token-zero errors disappeared entirely on ordinals 46, 50, 62 and 66. There were no exact-reply regressions and no shift to a later nonterminal token. Of the eight ordinary output tapes that changed, four were newly exact and four still had the wrong opening token. The other twelve recovered query spans produced the same inexact output as before.

On row 46, the ordinary reader moved from “took” at [17,18) to query “Ivor” at [42,45). It selected original position 18 and emitted [563,2537,1731,85,1], “small glass jars” and EOS. The prior first token was 1155. On row 37, with “about Maren, ignoring Ivor”, the new reader chose the rival at [41,44) over the target at [35,38). It copied the same Ivor object when the correct first token was 564 for “brushes.”

With both names admitted in a matched quartet, changing the position of the exclusion changed the frozen geometric evidence. In the “about” wording followed by the exclusion, target contrast was −16 and rival contrast −12, so the rival won. With the exclusion first, target contrast was +6 and rival contrast −54. The right-side features for the target changed from comma and the pieces of “ignoring” to period, newline and assistant framing. The left-side target roots were identical. The same trace repeated after swapping the requested person. [Exact roots and scores](evidence/role-attribution-seed1.json)

The supplied-endpoint arm explains the low end-to-end ceiling under the frozen value policy. Every supported query found one matching factual assertion. The scorer admitted Ivor's original value start on all 16 prompts. Elspeth's scored starts all had nonpositive margins. Leander's gold lay outside its radius. On Maren's cases another candidate fell outside its radius while gold's own margin was nonpositive. Forced original sources and opposing-source controls separately exercise the writer. [Supplied-source traces](evidence/supplied-value-seed1.json)

## Checks run at the head

Against the 32 paired reversals of the requested person, the baseline named the correct occurrence in both directions on 16 pairs and the treatment on 24. The other eight treatment pairs chose the distractor in both directions. The treatment chose the last query name on 48/64 prompts and the first on 16/64. Either fixed positional shortcut scores 32/64 on this panel. No pair produced two exact replies in either arm. Among the six unsupported controls each ordinary arm produced zero factual writes. The new reader abstained on the two unspecified and two absent people; both conflict queries retained a requested span. The ordinary acknowledgment is not a graded answer to an unsupported request. The forced-gold writer produced 64/64 exact on each seed, the opposing forced source 0/64, and the wrong-key scorer rejected the unchanged tape on all 64. The read-disabled and deliberately opposing-role arms also scored 0/64. [Unsupported cases](evidence/seed1-negative.jsonl)


Both frozen baselines rebuilt byte-for-byte. The six planted cases exercised both winners, a tie, a sole negative-contrast winner, an incomplete match and missing support. Both legacy and intervention matched the eight raw CUDA role words in all six cases. The treatment normal client reproduced [924,1074,1] from an archived non-held fixture. The legacy reader rejected the new schema. A preliminary SASS scan caught a reciprocal instruction emitted for signed division. The final CUDA scorer uses four shift/compare steps and passed the integer-only instruction scan. [Pre-run review](PRE-RUN.md) · [Independent FIT](FIT.md) · [Device record](evidence/finite.txt)

Each cold repeat matched all 310 replay file hashes and the complete summary for its primary seed. The independent reader compared the ordinary-client tapes directly with the sealed labels and original prompt IDs. A second review replayed 2,670 writer steps from 280 ordinary baseline and treatment records. The complete four-run dispatch took 52 min 24.25 s. A missing GNU time binary initially stopped the launcher before any held output; the retry used the Wick tool and the same sealed stage. [Replay manifests and timing](evidence/cold-replay.txt) · [Archive verification](evidence/storage.json)

The rental ran for 3,627 s at $0.631/h. The initial allocation selected an incompatible 5090 and was closed after 65 s without running study inputs. Estimated total compute at the quoted rates is $0.65. Both instances were deleted. The verified archive restored 2,318 files and two symbolic links locally, and Wick verified an independent copy and all 14 retained model pins. [Rental records](evidence/compute.json)

## Skills used

The native-space cycle governed the artifact schema, integer score, source position, cursor, token identity and replay from ordinary text.

## Still open

Both strict per-seed advancement criteria passed, with zero new factual writes on unsupported prompts. The registered next step is fresh replication with independent person/value assignments. The four newly correct outputs are four formulations of a single literal. Freeze the writer for that replication. The paired traces provide a specific role test: vary post-target exclusion wording, train/dev lexical roots and terminal framing independently, and measure whether the rival's score overtakes the target. The supplied-span evidence isolates the separate failure to recover the other three literal values.

## Plain summary

The requested role changed in the intended direction on 16 fresh questions. Four of those changes survived the frozen value reader and produced a complete exact reply. The same intervention made four existing mistakes more specific by copying the other person's object.
