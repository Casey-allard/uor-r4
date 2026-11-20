# [codex] Full TRAIN count retention and integer readback

## What did not work, first

The first full-dose Wick release failed its memory admission. It required 25,079,996,594 free device bytes after context initialization and the idle finite replay measured 24,838,406,144. The revised producer accounts for concurrent buffer lifetimes and makes two CUDA allocation failures reportable. Its 23,333,931,156-byte requirement includes 512 MiB reserve. A new finite run had to reproduce all the original bytes before the full run was admitted. These figures are memory bounds. The full run's measured device use appears below. [Initial hold](FIT-FULL.md) · [Revised FIT](FIT-FULL-LIVENESS.md)

The fourteen C128 transport probes all changed their complete next-token mass vectors when high-order groups were dropped. None changed its top token. They establish that the ordinary reader sees the changed distribution. They cannot show that general held prediction improves. No fitting or sealed DEV evaluation contributed to the choice of the retention threshold. [Paired token outcomes](evidence/paired.json)

A review found that the first generic retention verifier printed a native pass on a header-only input with zero probes. The real candidate requires positive probes and an independently sealed full source. The repeat verifier now rejects the 836-byte empty fixture. On the C128 and C1 positive fixtures the old and new reports were byte-identical, with 18 concrete native comparisons each. The repeat recipe also withholds the final crosscheck filename if the comparison is false. The originally released full run keeps its exact executable and recipe in custody. [Planted failure and repair](verify/evidence/hardening.log) · [Release identities](verify/README.md)

The full fixture fits the on-disk budget but its ordinary Linux client **exceeded four decimal GB of peak process memory**. From process start through one generated token on Wick, the untraced text path reached 4,702,887,936 bytes and took 10.15 seconds, including file open and validation. Direct IDs reached 4,592,500 KiB. On the same IDs the 86,536,132-byte trunk alone reached 790,360,064 bytes. The untraced C1 text path peaked at 1,033,576,448 bytes. Roughly 704 MB above physical deployment is common to the C1 and full text runs. These measurements identify a load/serve memory gap without distinguishing startup allocations from steady state. The M1 warm/resident gate remains open. [Wick measurements](evidence/client-process-memory.json) · [Full direct IDs](evidence/full-ids-client-untraced.time) · [Trunk direct IDs](evidence/trunk-client-untraced.time)

With **zero generated tokens**, the full client still reached 4,702,629,888 bytes. Across trunk, C1 and full files the excess over the raw D11 file was between 703.1 and 703.9 MB. The common loader validates twelve sparse tables containing 7,164,583 triples in 85,974,996 serialized bytes. It constructs nested integer maps with 4,297,201 distinct outer keys across those tables. This identifies the trunk loader as the first place to profile live allocations. The contribution of each allocation and the post-load working set still need measurement. [Zero-token controls and section census](evidence/client-load-only.json)

## What changed

The GPU count producer writes the full canonical table by context-key ranges. It seals the artifact after all raw and adjusted count orders agree with the independent census. A streaming Rust pruner keeps complete contexts, computes the cost of each tied support group, selects the smallest feasible support threshold, preserves every original follower word in selected groups and writes new discount spectra. A separately implemented reader checks exact retention by traversing source and candidate. The compiled CUDA reader queries the same compact bytes on device. [Unit contract](SHEET.md) · [Exhaustive verifier](verify/README.md) · [CUDA reader](device/README.md)

## Measurements

| Wick measurement | Observed outcome |
|---|---|
| Original full C | 2,075,608,357 tokens in 515,520 records. The 148,096,362,864-byte artifact has SHA-256 `1afd1c27bac4e963f4885ed275fa45c3e0a4014d3a6242f5deba82f04110ce26`. All eleven orders ran 127 nonempty ranges. |
| Full build | 11 min 39.62 s wall, 8,008,240 KiB peak process RSS, 16,492 MiB maximum whole-device use sampled every two seconds. It admitted 36,066,138,698 available host bytes against a 36,033,545,584-byte floor including reserve. |
| Full retention writer | Threshold 318 produced 3,912,334,481 bytes under the 3,912,936,331-byte compact cap. The mandatory order 0–3 floor was 3,058,504,908 bytes. Wall time was 10 min 21.64 s and peak process RSS was 20,896 KiB. Candidate SHA-256 is `064b691a54895771df1a957b2f97c91163990b6c6b6e09504c43285a97c49ef8`. |
| Independent full audit | Threshold 317 would add 8,049 groups and cost 3,914,255,830 bytes. All retained keys, follower words and eleven section summaries matched. All 11 retained probes and seven omitted probes passed native replay. The verifier took 16 min 53.18 s with 42,609,068 KiB peak RSS while mapping source and candidate. That is verifier memory on Wick. |
| Full high-order coverage | Orders 4–10 retained 2,250,282 of 5,349,741,581 distinct context groups (0.0421%), 4,426,535,571 of 14,503,998,019 raw support (30.52%), and 631,479,999 of 8,092,953,413 adjusted support (7.80%). One target can contribute at several orders, so these totals do not measure the percentage of held targets served. |
| C128 native controls | The all-retained result reproduced all 16,459,406 bytes. Thresholds two and three occupied 4,517,125 and 3,054,007 bytes. The exact floor was 2,074,826 bytes. |
| C1 pruned at 256 MiB | Threshold five occupied 242,774,057 bytes. Threshold four would require 276,123,394. Orders 4–10 retained 2,515,645 of 81,336,788 groups and 50,848,529 of 145,015,409 raw support. |
| C128 direct CUDA | Eleven all-retained, eighteen pruned and eleven floor probes, 440 order records per pass. Two fresh device processes per artifact matched every raw and adjusted count, both totals and Q48 word. |
| C1 direct CUDA | Both fresh processes matched all 198 order records on 11 retained and seven dropped contexts. |
| Full direct CUDA | Both fresh RTX 4090 processes agreed on all 198 order records, including seven dropped-context backoffs. The device output SHA-256 was `8ad2410b264bf512d9b18f343d3ac3f0c51f4ddb10cea05caee06a9dd4b5afde`. End-to-end device process times were 5.60 and 5.51 seconds, including upload of the 3,912,334,481-byte artifact. [Hardware receipt](device/evidence/full-c/primary.json) |
| Count features across batches | On 26 original targets, widths 1, 3, 10 and 26 produced the same 3,752-byte feature tape as the native oracle, twice each. Each process uploaded the 4,517,125-byte compact exactly once. The 598 checked fields include every raw target count, raw total and final Q48 mass. |
| Normal client | C128 original and pruned cost 103,320,836 and 91,378,554 deployed bytes, respectively. The C1 candidate cost 329,635,488 with tokenizer. On each, two fresh ordinary processes for every one of 14 original prefixes matched the complete score digest, winner and tie count under the unit KN weights. |
| Full normal client | The D11 is 3,998,930,520 bytes. With its 265,393-byte tokenizer the deployed fixture costs 3,999,195,913 bytes. All 14 full-dose original prefixes passed the complete score-vector, winner and tie checks twice through fresh ordinary client processes. The fit component here still contains planted unit KN weights. |

[Full receipt](evidence/full-spool.json) · [Full process timing](evidence/full-spool.time) · [Sampled device use](evidence/full-spool-gpu.csv) · [Full pruner receipt](evidence/full-prune.json) · [C1 retention audit](verify/evidence/prod-c1-256m.json) · [C1 client bytes](evidence/client-c1-physical.json) · [Batched device readback and timings](batch/RESULT.md)

The full release used the earlier verifier build. We checked its sealed three-file manifest, the literal true result, matching source and candidate hashes, threshold minimality and all 18 concrete probes before device admission. [Full native audit](evidence/full-verify.json) · [Timing](evidence/full-verify.time) · [Run manifest](evidence/full-retention.sha256)

The ordinary full client used its embedded state without a count sidecar. [Measured component sizes](evidence/client-full-dose-physical.json) · [Full-prefix score checks](evidence/client-full-dose-probes.json) · [Fresh process transcript hashes](evidence/client-full-dose-results.sha256)

**Where the first wrong token falls on actual text.** In the fixed unit-KN readout, swapping Mira's copper and Leon's silver while asking about Mira changed the correct first token from 581 to 3585. Both the 1% and full-dose clients emitted 799 (“ We”) in both directions. Both answers are already wrong at generated index zero. The distant pair has the same last ten framed IDs, so no pure order-ten distribution can separate those two assignments. The near-control prompts changed token 265 to 666 seven positions from the end. In both artifacts the deepest matched order was six and the within-pair distributions again coincided. Going from 1% to full dose changed the numeric score vectors across doses and left the winning ID and effective context depths unchanged on these four prompts. These are single-step transport fixtures. [Frozen inputs](client/HORIZON.md) · [Original token encodings](evidence/horizon-token-ids.tsv) · [C1 outcomes](evidence/client-c1-text.json) · [Full-dose outcomes](evidence/client-full-text.json)

## Checks run at the head

The finite controls include a structurally valid but wrongly omitted group, a changed and rehashed follower word, an incorrect TRAIN digest, a cap one byte below the mandatory floor, a no-clobber retry and a tied support group with spare bytes. Each produced its specified refusal. The finite CUDA witness separately rejected a stale input digest before packet creation and a changed raw count at the first probe. [Native controls](verify/README.md) · [CUDA controls](device/README.md)

All 27 existing n-gram counter host regression tests passed in the Wick review checkout. [Test output](evidence/count-host-tests.txt)

## Skills used

The native-space cycle supplied the shared integer representation, original-token replay and artifact-identity requirements. The native lookup is checked independently of the GPU producer and of the training-size estimates.

## Still open

P1(b) needs the actual R-only fit and paired held score after the occupied compact bytes and native/device reads are frozen. P2 needs four active equal-byte arms. The role track still needs requested, excluded and unspecified user occurrences classified before factual support. The two tracks meet at the ordinary client's grounded and non-copy continuations. The [implementation and acceptance sequence](NEXT.md) identifies the source interfaces, R and sealed reference custody, controls and criteria.

## Plain summary

Wick produced a physical full-dose count component that passes exhaustive retention checks, GPU readback and ordinary-client replay. The near-four-GB file still has a startup memory problem. On the two distant fact swaps the diagnostic count-only client already fails at its first generated token. The next predictive result requires fitting the shared mixture on reserved TRAIN and scoring the frozen candidate on held data.
