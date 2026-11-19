# [codex] What a clause-relative role reader can observe

## What did not work

Clipping the six original role offsets at the boundaries of each name’s own clause makes opposite instructions indistinguishable when the polarity cue is ten tokens before the name. On both frozen seeds, changing only which person to keep and which to drop preserved both candidates’ complete six-key inputs and their two signed scores. Refitting those six keys alone cannot resolve these paired prompts. The projection also has a separate fitting gap. Moving the cue directly before the name changed the keys, but the original bank gave both candidates zero evidence for both roles.

The existing order-ten S2 count artifact cannot supply an equal-byte prediction result. Its table survives serialization and supports exact reads, but the ordinary predictor does not use its counts for the next token. The currently active depth-four count mixer and the experimental role artifact run different tasks. The P0 sealed scoring reference also filters retained counts with known evaluation queries and targets, information that a deployed predictor does not receive. [Measured artifact sizes, corpus pins and outstanding controls](equal-bytes.md)

## What changed

The native role library exposes a checked projection for one original whole-word candidate and its half-open clause. It uses the existing six-offset key function, signed count scorer and missing-root convention. The current client’s role and writer policies continue to use their existing entry points. The finite driver cold-loads the paired D11, verifies its original role bytes and tokenizer binding, and exposes the inputs and scores for direct word-level GPU comparison.

## Measurements

Both seeds used the original full token tapes and the same two assertion-supported people. Rosalind and Benedict had separate statement heads. In a matched pair, the name positions and clause boundaries stayed fixed and the target switched. In the order control, both clauses moved while the requested person stayed fixed.

| Cue position relative to name | Six-key observation per seed | Frozen native scores (requested, excluded) |
|---|---|---|
| −10, beyond the three-token left window | Both names retained the same six keys on every polarity and order swap | (44, −8) for both |
| −1, inside the left window | The key at −1 switched between 174217 for “keep” and 163555 for “drop” | (0, 0) for both |

For example, “please keep the exact item wording for Rosalind; please drop the exact item wording for Benedict” and the same sentence with “drop” and “keep” exchanged require opposite answers. Their far-cue keys were [36080, 74004, 139551, 262147, 327684, 393221] for both people. The requested span switched from [41,45) to [57,60). Each frozen bank gave both people contrast 52. With a strict unique-winner rule this is a tie. These are finite input and score observations. No held reply or EOS rate was measured in this unit.

The already-implemented radius-12 native projector can see the missing cue in this construction. On each artifact, both its identity action and its positional action produced a different 32-coordinate vector on all eight matched person-level polarity flips, with L1 distance 32. All eight order-only comparisons per action were bit-identical, with distance zero. The largest absolute coordinate was 6 versus a saturation bound of 48. We then fixed the root multiset and changed only order. “please keep rather than drop Rosalind” and “please drop rather than keep Rosalind” switch the correct target. On both seeds, each of four matched person comparisons gave **L1 distance 0 for Identity and 48 for Positional**. With the requested person fixed, reversing the clauses changed zero coordinates for either action. The cue roots 43143 and 32481 exchanged slots −5 and −1. The short six-key view distinguishes this particular swap at −1, but fails the earlier −10 fixture. Of the three tested feature maps only the existing radius-12 positional action distinguished both constructions. These measurements compare the Rust feature words on hand-built inputs. A learned role decision and a correct served reply still require separate evidence. [Registered order control](finite/order-control.md) · [Paired feature results](evidence/positional-summary.json)

## Checks run at the head

The six-key projection returned the same raw words as the frozen CUDA scoring function on the original SM89 RTX 4090 implementation. Two seeds, eight finite prompts per seed, two candidates per prompt and eight words per candidate gave 256 checked words per pass. A second process for each seed reproduced all output bytes. Candidate regions were checked against the uncut tape and original assertion heads. Each seed’s D11 role section matched the archived role bytes exactly. Both stored tokenizers reproduced the full submitted text. Existing library tests also cover split-name rejections, left and right missing-root fences, a planted local score reversal and the long shared-tail collision. The fixed writer was outside the intervention. The ordinary client/EOS result for the preceding test is in the [D21 report](../codex-21/REPORT.md).

Wick's GPU was occupied. Its nvcc built the SM89 executable. The finite device test used a rented RTX 4090, driver 580.95.05, for an estimated $0.03 including allocation and teardown. A verified copy of the device evidence was retained on Wick and the workstation before the lease ended. The private store subsequently returned the complete finite-evidence archive at its pinned revision, and every archived file passed readback. [Storage and compute receipt](evidence/storage.json) · [Reproduction](finite/README.md) · [Seed 1 scores](evidence/native-seed1.json) · [Seed 2 scores](evidence/native-seed2.json)

## Skills used

The native-space cycle supplied the artifact, integer coordinate and shared-operator replay checks. This work performed no fit or gradient update.

## Still open

D15 already observed positional separation on 84 pairs. Its registered joint test stopped because supplied-person end-to-end replies reached 232/256 per seed against a floor of 248. D22 adds the exact six-key collision and the clause-local, same-root order control. The D15 joint fit remains gated. First test whether a new observable connector-boundary signal distinguishes early genuine values from later values with the same old candidate evidence, and rerun the supplied-person writer prerequisite on fresh labels. Any earlier role-only fit needs a separately registered endpoint and supports only a role claim. [D15 decision and dependencies](../codex-15/DECISION.md)

The next role fit needs a clause-sized field with independently labeled requested, excluded and unspecified occurrences. D12’s 224 absent-source examples all carry a valid request. D15’s 1,152 training rows include valid incidental mentions with no request and requested people with no source. Both archives carry a single requested span and lack the dual-affirmative case. Preserve separate role and source decisions. Include dual positives, absent and ambiguous requests, a supported plus an unsupported positive, co-mentioned names, both clause positions, different fact order, and previously unseen wording. Require role decisions to survive order-only swaps and to invert on polarity swaps. Run the fixed writer on the same held panel before accepting a role gain.

For the equal-byte control, the measured seed-1 depth-four artifact supplies a concrete prototype cap of 125,998,708 bytes. Its common trunk uses 86,536,132. Count the remaining 39,462,576 bytes against every head, index, route table and mandatory slot. First qualify an active order-ten P1 carrier on the ordinary predicted-token and per-token probability path, with TRAIN-only retention, a total-mass floor and state readback. The final P2 comparison also requires the persistent geometric reader and its output expert, a lossless longer-context control, and P1(a,b) on the measured carrier. [Byte-level feasibility](equal-bytes.md)

## Plain summary

On the constructed prompts, the source entities survive and the request cue disappears before the narrow role scorer can compare them. A pre-existing native representation preserves the distinction. The remaining test is whether training on that same representation gives the ordinary client the correct person and reply on fresh language.
