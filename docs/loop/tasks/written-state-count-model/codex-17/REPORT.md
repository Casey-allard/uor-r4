# [codex] Where the learned reader starts copying on Wick

## What did not work

Ordered geometry chose the complete value on **0/24** matched held questions. Order-neutral geometry and position alone each solved **8/24**, the registered ceiling for a selector using only the matched structural information. The two arms shared only two successes. The result closes this configuration on these two lexical families. It leaves open whether a different native representation can distinguish the full value from the connector and its shorter suffixes. [Registered comparison](SHEET.md) · [Held structural ceiling](panel-v5/held-ceiling.json)

## What changed

The study scored the reachable original word starts admitted by the frozen three-slot mask. The held requested-span capture inventoried eight additional start entries per artifact outside that mask. All 36 correct held starts were inside the scored domain. Ordered used the signed D11 orbit distance. Order-neutral sorted roots within each segment and class, preserving segment order and shape. Position alone voted from the relative connector depth and the remaining length. From 88 supervised examples, the Wick CUDA fit produced three equal-size 200-record banks for each frozen artifact. Each bank was embedded in a D11 artifact and loaded by the ordinary Rust text client. The writer, EOS, role selection, training inputs and held prompts were fixed across the three arms. Two independent readers agreed on every supported answer and on the six control meanings before the study. [Panel](panel-v5/held-reading.json) · [FIT](study/FIT-RANK.md) · [Input release and capture receipts](study/LIVE-GO.md)

## Measurements

Each column below uses the same 36 supported prompts on each of two separately fitted artifacts. The matched 24 are a subset of the 36. Recovery and regression compare the same prompt with the frozen ordinary client. Token indices start at zero. Exact requires all authored IDs and the terminal EOS.

| Ordinary text client | Exact through EOS /36 | Exact on matched /24 | First wrong at token 0 /36 | Recovered since frozen | Regressed since frozen | Stopped at EOS /36 |
|---|---:|---:|---:|---:|---:|---:|
| Frozen | 3 | 0 | 33 | — | — | 36 |
| Ordered | 4 | 0 | 32 | 3 | 2 | 35 |
| Order-neutral | 10 | 8 | 26 | 10 | 3 | 35 |
| Position only | 13 | 8 | 23 | 12 | 2 | 36 |

On a recovered row the first wrong token disappeared. On a regressed row it returned at token 0. Every reply still wrong under both selectors diverged at token 0 in both. Ordered and order-neutral each had one supported fallback that reached the output limit. On the six control prompts (two absent, two conflicting, two unspecified), every ordinary arm gave the same acknowledgment, “Got it, I'll remember that.” It supplied no factual value or clarification. We grade these controls separately. [Seed one, every held decision](evidence/seed1.json) · [Seed two](evidence/seed2.json)

**First causal error.** On the matched 24, ordered picked a rival source 18 times and abstained on six because a reachable candidate lay outside the fitted radius. Order-neutral picked a rival ten times and abstained on six. Position picked a rival 16 times. Supplying the requested entity endpoints reproduced the ordinary arms' supported decisions. Forcing the gold value address through the same cursor returned the exact reply on **36/36**. Forcing either other address returned **0/24** exact on the matched triples, and every one of those alternative writes stopped at EOS. Disabling the reader returned **0/36** exact. The first actionable divergence in these triples is selecting or admitting the original value address.

For example, the clause “Alaric will paint glass tiles” requires original address 20 and first ID 2537, the start of “glass.” Frozen chose 19 and wrote “paint glass tiles.” Ordered and position chose 21 and wrote “tiles.” Order-neutral chose 20 and wrote “glass tiles” through EOS. In the matched clause “Alaric will now paint tiles,” position's same choice of 21 became correct. In “Alaric painted fine glass tiles,” ordered and neutral both scored the correct start but abstained because of an out-of-radius rival. These cases distinguish selecting the wrong suffix from suppressing an answer before the writer runs. [Three prompts with emitted IDs](evidence/selected-triple.json) · [Candidate scores and radius checks](evidence/trace-detail.tsv)

## Checks run at the head

We verified all eight raw capture inventories and every referenced cold file. Twelve archive-audit files contained absolute run-directory names. Independent review rehashed all 496 read-map entries and reproduced equality after translating only verified self-root prefixes. The first collector rejection is preserved. Both artifacts exposed all 88 supervised sources and all 36 supported held sources. The eight matched triples each retained three distinct writable outcomes. The live client plus tokenizer plus D11 consumed 91,136,995 bytes for seed one and 89,544,191 for seed two, equal across arms within each seed. Each embedded bank was 8,416 bytes. Changing the grading key made zero of the forced-correct replies match that key while the independent reference still counted 36/36. The generated ID and writer-trace digests stayed fixed. [Original sources and controls](validity/README.md) · [Input release](study/LIVE-GO.md)

Both fresh processes for each artifact reproduced all 386 registered files through the CPU client and CUDA writer. The complete grade summary was byte-identical across all four studies (SHA-256 `be991c93acd50864ede8fe4106ad71382811613de25159f1d561b9047d538dd7`). Seed one's primary and repeat took 141.30 and 145.22 seconds. Seed two took 136.51 and 135.23 seconds. The cost table separates complete process timing and peak RSS from the individual CUDA fit and score operations and the ordinary batch of 42 separately launched clients. These wall times include launch and I/O. [Bank identities and costs](evidence/runtime.tsv) · [Cold equality, wrong-key control and native/client replay](evidence/runtime-receipt.json)

The 1,113,531-byte final evidence archive compared byte-for-byte on Wick and the workstation and passed an independent restore check against 2,507 regular-file hashes. The private store refused the preceding 1,121,693-byte archive at its storage quota. No private-store revision was created. Both verified copies remain available. [Custody, retained artifacts and restore conditions](evidence/storage.json)

The local delivery check found missing machine-readable lab and item fields in the ORDER. We added the two identifiers already present in its heading and issue reference after the measurement. The registered inputs, arms, bars and scientific text are unchanged.

An independent client review ran the pinned synthetic CLI integration test. Planting the correct source wrote IDs `[924,1074,1]` and the rival wrote `[1074,1]`. The legacy library passed 200 tests, the serving-feature library passed 711 with one ignored, and all four legacy CLI tests passed. The WebAssembly serving library compiled. These are host compatibility checks. The held comparisons above use the Wick CUDA study and the ordinary Rust client.

## Skills used

The native-space cycle traced each value choice into the serialized state and exact writer. The research-architect review tested the scorer with independent labels, forced alternatives and a same-byte positional comparator.

## Still open

Use a new held panel for the next test. Separate two failures on TRAIN and development data first. Measure how often the correct address outranks both writable rivals when all three receive scores. Separately count how often a rival outside the fitted radius suppresses an otherwise correct decision. Keep the writer fixed. A change needs to beat the registered structural ceiling, retain appropriate abstention on genuinely unsupported inputs, and survive ordinary text entry and cold replay. This measurement covers the native Rust client on Wick. Browser serving, M1 latency, free prose, reasoning, sustained memory and general next-token prediction each require their own evidence. [Program closure plan](../codex-9/CLOSURE-PLAN.md)

## Plain summary

Starting one word early or late explains many confidently written wrong answers. The writer can return the right text through EOS when given its address. The learned ordered selector still cannot find that address reliably when wording changes.
