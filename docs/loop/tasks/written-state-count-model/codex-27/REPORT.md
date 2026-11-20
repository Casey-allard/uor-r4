# [codex] Relation-conditioned assertion selection through the fixed writer

## What did not work

All 32 lexical-transfer prompts failed at assertion selection. Giving the
selector the correct person and the query relation word recovered zero.
Three of the four cue pairs differed in BPE length and never reached the
distance calculation. “Collects” and “gathers” each had two pieces. Their
squared D11 distance was 80 and the registered maximum was 32. The two
lexical families and two queried relations per family define the scope.
The measured result closes this frozen word comparison on them. [Original device evidence and per-cue failure](evidence/transfer-failure.json)

The primary ordered reader answered 0/80 supported prompts through EOS,
even with the correct person and assertion set supplied. In the secondary
recurrence client, 12 supported replies became exact, but no matched
relation-switch pair produced two correct replies. Both readers failed all
six explicit refusal requests. The original default remains in place. [Paired result analysis](evidence/RESULTS.md)

## What changed

Two opt-in integer carriers select original factual heads before the existing
source reader. One requires equality of complete cue token IDs. The other
also admits a bounded nonzero displacement between four-block D11 root words.
Equal best evidence for different cues withholds the read. Repeated
occurrences of the winning cue pass together to the original complete-value
agreement rule. The ordinary entry point consumes the raw user message and
makes its own role and assertion choices. Separate diagnostic interventions
supply a person, a relation word, an assertion set or a value start.
The fixed source-address writer and exact EOS transition served every arm.
No new training state was fitted. [Native operator](operator/README.md) · [Writer and client reproduction](writer/README.md)

## Measurements

The held panel had 92 prompts. Eighty had a unique supported object.
Six unsupported prompts required a literal refusal and six had no specified
response string. Thus 86 had a complete tape to compare. The conflict cases give two distinct objects for a singular request and both source statements can be true. Two blind readers
agreed on all 92, and both flagged the semantic limits of “indexes” versus
“catalogs.” All prompts asked about a person. This panel does not qualify
no-request or multiple-request role handling. [Sealed semantic checks](evidence/SEMANTIC.md)

| Downstream policy and assertion intervention | Correct assertion sets / 92 | Exact supported / 80 | Exact through EOS / 86 | Unsupported factual copies / 12 |
|---|---:|---:|---:|---:|
| Ordered, unfiltered | 4 | 0 | 0 | 0 |
| Ordered, exact cue | 60 | 0 | 0 | 0 |
| Ordered, geometric cue | 60 | 0 | 0 | 0 |
| Ordered, oracle assertions | 92 supplied | 0 | 0 | 0 |
| Recurrence, unfiltered | 4 | 0 | 0 | 4 |
| Recurrence, exact cue | 60 | 12 | 12 | 0 |
| Recurrence, geometric cue | 60 | 12 | 12 | 0 |
| Recurrence, oracle assertions | 92 supplied | 22 | 22 | 0 |

The original asked-person span was correct on all 92. Both filtering arms
selected all 32 literal-overlap targets and all 28 control fact sets. They
selected zero transfer targets. The person-supplied and relation-supplied
lanes reproduced their ordinary choices. Forcing the correct source
gave 80/80 exact replies on each parent. Forcing the opposing value gave
0/80. In recurrence, filtering produced ten incorrect supported source
copies compared with two under unfiltered agreement. All remaining scored
ordinary errors started at output zero. [Ordered rows](evidence/ordered-rows.tsv) · [Recurrence rows](evidence/recurrence-rows.tsv)

On row 2732 the facts say Nina repairs porch swings and paints cedar boxes.
The unfiltered recurrence reader emitted 1155 instead of 4071. When asked
what she repairs, both filtered arms chose fact head [17,19), read source
21, and emitted [4071,1889,1], “porch swings” through EOS. On 2711 the
filtered reader correctly chose the assertion that Owen catalogs copper
lamps but started inside the object. The first wrong token changed from
1155 to 2527 (“lamps”), with 581 expected at offset zero. On 2799 the
requested person had no assertion for the queried relation; filtering
withheld the baseline's unsupported “porch swings.”

The downstream witness shows why better assertion routing alone cannot
repair the primary arm. Given the oracle heads, the continuation step
already pointed at the correct original value on every relevant assertion
for 56 of 80 supported prompts. The following ordered bank rejected them.
On 2703, the continuation selected position 22 for “copper lamps.” The
frozen bank had positive distance 120 within radius 124, but no negative
exemplar for this candidate's exact shape and class signature. The final
reader withheld it and the writer emitted fallback token 1155 instead of
581. The 92 oracle assertion visits include missing class support,
inaccessible starts and vetoes from rival candidates. These counts use an
assertion denominator and include duplicate facts. [Original onset fields](evidence/value-onsets.json) · [Independent frozen-bank decoding](evidence/onset-localization.md)

A separately registered, query-blind static pack retained 1,973,111 original
count records across 1,509,774 complete route keys from the already fitted
1%-TRAIN table. The actual active D11 occupies **125,998,700 bytes**, eight
under the prototype cap including its trunk. Two complete builds matched.
The ordinary client read a real prefix at order two after pruning versus
order seven before pruning. Device and independent oracle agreed on every
route, ranking and all 45,056 Q32 masses per pass over one real and ten planted
prefixes. This is one working capped count reader. The equal-byte P2
prediction comparison remains open. [Packing, readout and coverage](count/README.md) · [Device differential](count/GPU.md)

## Checks run at the head

All 15 planted selector cases passed on the leased RTX 4090. Both fresh
device processes matched all 276 projected held rows for the two parent
models, including all evidence words and membership decisions. The GPU scored
host-discovered query and cue spans. The normal client separately checked
original role, source, complete emitted IDs, cursor steps and stopping
condition. All 552 raw client transcripts from the primary run matched their cold-process counterparts byte for byte across six configurations. [Cold client checks](evidence/cold-replay.txt)

The full serving-feature library suite passed 724 tests with one
ignored. All twelve full harness gates passed. The rented GPU was released
and deleted after verified archive transfer. Its ledger estimates $0.19.
[Non-author FIT](FIT.md) · [Sealed GO](evidence/GO.json) · [Compute ledger](evidence/compute.json) · [Storage](evidence/STORAGE.md)

## Skills used

The native-space cycle bound original token addresses, typed root words,
device selection, the saved carrier and exact ordinary-client replay.

## Still open

The value bank needs real positive and negative support for the candidate
shapes already reaching it, and the source proposer needs legal whole-word
starts beyond its three token offsets. Test those separately on fresh
people, verbs, particles and objects, including genuine early objects and
contradictory assertions. Keep the current veto as a control. Measure each
stage against the original addresses and both directions of each prompt
pair. A forced correct start remains the writer canary.

For relation transfer, test a variable-length whole-word or contextual
representation fitted on disjoint TRAIN contexts against literal identity
and shuffled cue labels. Preserve absent and conflict controls and separate
matched verbatim, inflected, paraphrase and multiword constructions. The
radius cannot repair comparisons that are discarded before distance is
computed. The D27 exact-cue fallback and its secondary recurrence gains
do not establish semantic relation retrieval.

The next prediction unit should package the reserved-TRAIN mixture into an active D11 and run its real mass read through the ordinary client and the CUDA scorer. The original 1% stream has 5,196 records and 20,475,957 tokens. The existing public evidence qualifies the earlier retained-count replay on that slice, while the full fitted mixture still needs its own readback and P1(b) result. Use DEV-tune for deployment verification, then freeze the carrier before the sealed panel. [Existing order-ten replay](../codex-10/REPORT.md) · [Reserved-TRAIN output fit](../claude-439/REPORT.md)

For P2, choose the token-context order on DEV-tune with
adequate order range, and measure persistent geometry, reseeded same-transport
and SplitMix at equal charged bytes and training dose. Reserve sealed DEV-test
for the registered best-control comparison. The capped pilot is lossful and
uses 1% TRAIN. M2 useful replies, ordinary next-token quality and production
latency still require their own gates. The inspected LexLean formal source
and WASM stack have related geometry but no verified correspondence for this
end-to-end D27 mechanism. [Authoritative phase criteria](../../../../research/reviews/pi-review-2026-11-16/DECISION.md) · [LexLean source inspection](evidence/lexlean.md)

## Plain summary

The model can now switch between two assertions about the same person when
the question repeats the relation word. The fixed writer can copy all 80
supported objects from a correct address. The main failures on this panel
are recognizing a differently worded relation and admitting the beginning
of the right value. Only the recurrence-backed opt-in client turned the
literal routing gain into complete replies.
