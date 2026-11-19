# D21 paired findings

The ordinary clause selector chose the excluded person's query span on all
96 supported prompts on both seeds. The reference chose the last name on
every prompt and selected the requested span on 48. The clause arm recovered
zero spans, lost 48, and fell from eight exact replies to zero. It changed
24 incorrect token tapes to different incorrect tapes and produced 20 new
wrong-source writes. These counts and affected prompt ordinals matched
across seeds. In each family and name order, the clause arm lost all eight
reference-correct spans when the exclusion came first.

In both polarity orders, clause contrasts favored the excluded name by
9 points for inventory (41 requested, 50 excluded) and by 56 points for
collection and supplies transfer (28 requested, 84 excluded). The reference
favored the last name by 6 points in every family. These comparisons measure
the frozen scores on these prompt forms. Individual cue-token contributions
were outside the design.

With the requested span supplied, the earlier-connector veto lifted exact
replies from 16 to 24 per seed. The same eight prompts gained the exact
original source and retained EOS. All eight combined “could still find”
with “small glass jars.” The veto preserved 24 existing wrong-source writes
(16 connector starts and eight suffix starts). It introduced zero new
wrong sources and retained 48 unseen outcomes. The ordered arm recorded
zero ties. The veto left all 32 supplies transfer outcomes unchanged.
The 16 earliest-gold transfer rows all had a scored gold candidate.
Eight of those still had an overall unseen outcome because a different
candidate was unseen. The held panel exposed zero earlier-unseen gold
candidates. The finite adverse witness supplies evidence for that risk.

## Three prompt examples

All examples matched across seeds. Prompt ordinals, original prefix
offsets and output positions count from zero.

| Ordinal | Paired result | Original source offset | First output mismatch |
| --- | --- | --- | --- |
| 2 | Both ordinary replies were wrong. The clause arm changed a fallback to the excluded person's “small glass jars.” | None → 9 | Output 0 expected token 564 (“ br”). Reference emitted 1155 (“ G”). Clause emitted 563 (“ small”). |
| 41 | The reference emitted “green glass beads.” The clause arm copied the other person's connector and value, “take linen napkins.” | 8 → 19 | Output 0 expected 1074 (“ green”). Clause emitted 754 (“ take”). |
| 8 | The supplied ordered arm abstained from source copy. The veto copied “small glass jars” exactly as IDs [563, 2537, 1731, 85, 1]. | None → 11 | Ordered output 0 expected 563 (“ small”), emitted 1155 (“ G”). Veto matched through EOS. |

On ordinal 8 the first candidate at source 9 was outside its positive
radius (positive distance 52, negative distance 44, radius 48).
The next two admitted candidates had margins −4/3 and +16/4. The
first candidate's status changed from unseen to outside radius.

The six unsupported controls covered two each of conflict, missing person
and omitted identity. The reference, clause and no-read arms produced
fallbacks without a factual source on all six controls per seed. Each
ended with EOS. Supplied, veto, wrong-role and forced-source comparisons
used the 96 supported prompts. Both seeds had one cap-limited no-read
reply at ordinal 40. It reached 16 IDs without terminal EOS. Every
supported reply in the other arms ended with EOS. Forced gold matched all
96 original replies through EOS. Forced wrong matched zero original
replies and copied the designated rival. Regrading the forced-gold tape
with the mismatched key gave zero exact while its original-key score
remained 96.

The seed tables cover the complete paired prompts. The JSON summary
reports matching prompt outcomes across the two seeds and binds its counts
to the locally inspected artifact manifests.
