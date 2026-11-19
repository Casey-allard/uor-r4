# [codex] Value-start ranking and rival veto with the original writer

## What did not work

Every ordinary reply missed at **output token 0** on both artifacts. Original ordered, ranking change alone, veto change alone and both changes together each scored **0/36 exact through EOS**, including **0/24 matched**. Read-disabled and position-only also scored zero. No treatment changed an ordinary selected source or first emitted ID. All 36 supported rows had zero supporting assertions and no value source. Every reply began with ID 1155. All 36 eventually reached EOS on seed one and 35 on seed two, where ordinal 12 hit the output cap.

## What changed

The two value-read switches use the same ordered 8,416-byte bank within each seed. Ranking divides the signed nearest-exemplar margin by the sum of the two distances. Veto relief permits a unique supported positive winner when another candidate falls outside its fitted positive radius. Ties, missing evidence and nonpositive winners still abstain. The role reader, source mask, original token tape, copy cursor, writer and EOS are fixed. The baseline and positional D11 files match the archived originals.

Eight fresh triples require three different correct starts under the same registered structural observations. Two independent text-only readers agreed on all 36 supported questions and six controls. Both artifacts used the same panel. [Registration](SHEET.md) · [Adjudication](panel/reading.json) · [Planted tests](operator/README.md)

## Measurements

Each number applies to each of the two artifacts. The supplied-endpoint columns give the value selector the true requested-entity endpoints and grade the resulting original CUDA writer stream. They measure the downstream component.

| Policy | Ordinary exact /36 | Ordinary matched /24 | Supplied endpoints exact /36 | Supplied endpoints matched /24 |
|---|---:|---:|---:|---:|
| Original ordered | 0 | 0 | 8 | 6 |
| Ranking change | 0 | 0 | 8 | 6 |
| Rival-veto relief | 0 | 0 | 12 | 8 |
| Both changes | 0 | 0 | 12 | 8 |
| Position only | 0 | 0 | 12 | 6 |

With true endpoints, veto relief recovered ordinals **16, 17, 22 and 23** from token-0 error to exact through EOS. No previously exact reply regressed. Ordinals 22 and 23 belonged to the matched set. Every remaining inexact reply still diverged at token 0. On the 24 matched rows, relaxing the veto released 14 abstentions into writes, two correct and twelve from rival positions. The ranking switch changed the numerical margins but left all matched candidate orderings and nominations intact. The combined arm added zero exact replies over veto relief. Its 8/24 equals the registered structural ceiling. For coverage rows 33 and 34, the summary's generic `unseen` means every available candidate was outside its fitted radius. [Ordinary seed one](evidence/ordinary-seed1.json) · [Ordinary seed two](evidence/ordinary-seed2.json) · [Paired conditional grades](evidence/requested-both-seeds.json) · [Radius details](evidence/all-outside.json)

The correct query-name endpoint appeared in the raw candidates on all 36 supported prompts. The ordinary reader selected “the” in the question on 18/36, selected an earlier name in a factual sentence on 10/36 and abstained below its evidence floor on 8/36. Four of the ten earlier mentions named the intended person at the wrong occurrence and six named the other person. On the matched 24 those three categories had 12, 8 and 4 rows. There were no best-score ties on seed one. Both seeds chose the same endpoints. All six absent, conflicting or unspecified controls received an acknowledgment with no factual source in every ordinary arm. [Role trace](evidence/role.json)

On matched row 22, “Oswin took large pink rings. Linnea took old gold coins. Which full object description belongs to the statement about Oswin?”, the ordinary reader picked “the” at [36,37) with score 84. The requested “Oswin” at [41,44) scored 12. Its reply began with ID 1155 in place of 2477, the start of “large.” Supplying [41,44) still left the ordered and re-ranked arms blocked by a rival at distance 80 against radius 72. With veto relief the writer copied from position 8 and produced “large pink rings”, IDs [2477,1481,336,568,1]. On row 0, gold itself lay outside the radius, and the same change released a wrong suffix. [Both prompts and raw outcomes](evidence/examples.json) · [Matched release audit](evidence/veto.json)

## Checks run at the head

The live gates found all 88 frozen FIT sources, all 36 held gold writes and the gold start and two distinct writable rival starts per matched row. Forcing the gold source gave 36/36, including 24/24 matched. Forcing either rival gave 0/24 exact on matched rows although every such write reached EOS. Mutating the key made the forced-gold output score zero against the false key while preserving 36/36 against the independent reference, identical writer output and identical generated IDs.

Wick's waiting dispatcher was suspended before it started a seed. The dedicated Vast RTX 4090 ran the held study. Beforehand, its 84 planted selector decisions matched across all 68 trace words, 11 original writer-chain transitions matched, and all four ordinary-client controls matched their packed artifacts. The host serving library passed 711 tests with one ignored. The independent ordinary-client integration and targeted Rust formatting checks passed. A crate-wide formatting check reported pre-existing differences outside the changed files. [Compute amendment](VAST.md) · [Actual device qualification](operator/vast/qualification.sha256) · [Authorization](operator/vast/authorization.json)

Both cold repeats reproduced their primary ordinary and requested-endpoint summaries and all 427 replay file hashes per seed. The four runs took 7 min 1.83 s. The rental was deleted after 869 s at $0.869/hour, an estimated $0.21. The archive was restored on the workstation and its checksum independently verified on Wick before teardown. [Device replay and timings](evidence/replay.txt) · [Archive and restore record](evidence/storage.json)

## Skills used

The native-space cycle tied the original source, emitted IDs and cursor to the frozen writer. The research-architect review kept the ordinary reply, the role intervention and forced writes independently observable.

## Still open

The next unit should vary the requested person, query phrasing and repeated factual name occurrences independently with the writer fixed. Measure selection of the exact query occurrence before judging value retrieval. Keep genuine negatives and cases where “the” appears near either candidate. Separately, the veto admitted many wrong suffixes, so any value-score change needs to distinguish gold from those rivals on a new holdout. [Prior-panel audit](analysis/previous-panel.md)

## Plain summary

Removing the veto recovered four answers once the correct entity was supplied. In ordinary text, the model never connected the question to the requested entity. Changing how it ranked the value starts therefore could not improve a reply.
