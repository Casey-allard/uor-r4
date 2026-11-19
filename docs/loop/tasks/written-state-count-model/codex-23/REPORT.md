# [codex] Ordinary client generation from the retained S2 count table

## What did not work

The retained 1%-TRAIN count table returned the same eight tokens for “What is a map?”, “Where is the nearest library?” and “Why does ice float?”. Each reply read “ We need answer. Need par” and reached the eight-token limit. All three began with the same order-seven key over the suffix beginning at the question mark and ending with the Assistant role marker. The subject of each question occurred before that suffix. The first lookup tried longer keys; the deepest retained hit discarded the distinguishing content. Subsequent selected keys, counts, rankings, masses and tokens were identical across the three trajectories. The primary question was fixed before inference and the two additional questions before their respective runs. These open questions have no single gold first token. [Ordinary client outputs](evidence/real-client.json) · [Framing control](framing-control.md)

The independent source-boundary readers found multiple plausible literal starts in the proposed disinfect/repaint pair. We rejected that pair for single-answer evaluation. No source-boundary improvement follows from it. [Blind premise check](premise.md)

## What changed

The opt-in D11 S2 schema now selects the retained successor table in the ordinary greedy client. The passive carrier continues to use its original predictor. Activation preserves every payload byte and changes the schema digit in the header. The expert uses native original-token suffix keys through order ten, up to four retained successors per key, a fixed 8/256 uniform floor and the remaining 248/256 distributed by the retained counts. Its integer mass sums to 2^32 over legal output IDs. This unit used frozen state and introduced no fit or writer change. The comparison measures activation at equal artifact length. An equal-byte comparison with a trained geometric reader is still open.

## Measurements

| Ordinary text input | Passive result | Active result | First changed token and stop |
|---|---|---|---|
| Planted red / blue, same last nine input IDs | Both emitted [1155, 302] | Red [677, 1], blue [924, 1] | Both first errors at 0 disappeared; exact through EOS rose from 0/2 to 2/2. |
| What is a map? | “ Got it, your steerk” | “ We need answer. Need par” | 1155 to 799 at offset 0. Both reached the eight-token cap. |
| Library / ice questions | Passive arm outside the framing control | Same reply and all eight decisions as the active map run | Both reached the cap. |

The planted files were each 86,536,760 bytes. Matched active route orders were ten for the first token and two for EOS. The original proposed prompt pair shared all ten trailing token IDs. Removing “fox” before observing any generated output put the contrast in the tenth slot. On the actual count table, both carriers were 379,223,660 bytes and differed at byte 71 (one-based). The packed section held 14,634,359 records, totaling 292,687,180 record bytes. The first selected counts on all three questions were 16, 64, 512 and 4 for IDs 201, 287, 799 and 3184. The active map trajectory used orders 7, 8, 9, 10, 10, 10, 10 and 10. [Planted outputs and full control check](evidence/finite.json)

## Checks run at the head

The ordinary client and finite fixture exercised both text and raw-ID entry points, passive compatibility with original predictions, stale input and tokenizer refusal, invalid serialized ranks, ties, backoff, document boundaries and target support. Fresh map processes repeated each schema output byte for byte. The integer library suite passed 720 tests with one ignored. The activator regression rejects a correctly pinned passive model whose vocabulary the active schema cannot load. The native-space contract check passed.

An SM89 RTX 4090 independently read the planted rows and the full retained count rows. On ten planted prefixes and eight self-fed map prefixes the device reproduced the selected route, counts, all 4096 masses and top-eight ranking on both primary and cold runs. That is 73,728 individual masses per pass. The finite oracle separately reconstructed the expected values. The initial compiler emitted a floating-point reciprocal for integer division. Exact bounded integer division replaced it and the qualifying SASS passed inspection. An initial rental launch failed before CUDA execution; the absolute-path retry completed. Wick's 4090 was occupied. The rental closed after 117 seconds at $0.869/hour (about $0.028). [Device and artifact custody](evidence/storage.json)

## Skills used

The native-space cycle supplied the integer representation, original-address and exact-replay criteria for the finite check.

## Still open

P1 still needs its registered trained mixture and scored panels. P2 still needs an equal-byte persistent geometric reader and the prescribed comparators. The measured failure gives that reader a concrete job: carry request content through the user-to-assistant boundary and change the selected evidence when the question changes. Compare different questions in the same frame and the same question across frames. Source-role and connector-boundary work still needs independently unambiguous held language. No timing or useful-chat bar was measured here.

## Plain summary

The ordinary client can now use order-ten counts to distinguish two texts and end the reply correctly. On the retained research artifact, three distinct questions take the same suffix route before token zero and generate the same incomplete reply.
