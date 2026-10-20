# Native geometric composition learning — October 3

References #1552 and #1512. [Evidence](../evidence/geometric-composition-learning-2026-10-03.json). This is a declared readout-only learning continuation, not general chat, a capacity proof or a geometry comparison.

## Outcome

The [fixed-weight transfer](geometric-transfer-2026-10-03.md) completes only2/16 development rows. New varied-source supervision through the existing learner improves original-case replies to20/20 at intermediate checkpoints, but does not complete any of the eight new construction responses or improve the sixteen transfer rows beyond2/16.

| New updates | Original complete /20 | Original EOS | Construction complete /8 | Construction EOS | Transfer complete /16 | Transfer EOS |
|---:|---:|---:|---:|---:|---:|---:|
|0|18|20|0|8|2|16|
|16|20|20|0|2|2|10|
|32|20|20|0|8|2|16|
|48|20|20|0|8|2|16|
|64|18|18|0|5|2|11|

Retain checkpoints32 and48 as unadopted component candidates. Checkpoint32 is the earliest with20/20 original replies and restored EOS across all panels. It is a conservative starting candidate for the next learning change, not a composition-qualified model. Final64 has two losses relative to48, repeating Klotdradburg: the correct literal and period now appear, followed by runaway continuation. Its complete flags equal the starting18/20. Every stage is preserved; no product default changes.

There is partial learning. The same construction B8 mean episode CE falls1.756587 at the first pre-update batch to0.972457 before update64. These are matched construction cases evaluated at different pre-update weights, not a full-data canonical CE measurement. Six of eight final construction responses first diverge after the first token: positions2,2,0,0,2,1,6,4. One repeated-word response correctly emits three singer words before failing termination. Complete replies remain the goal; prefixes and CE do not substitute for them.

The next intervention is **existing geometric context adaptation**, on the same frozen data with ordinary answer credit, starting from retained32 or48. The run deliberately discarded nonzero context gradients. Enable those existing parameter/gradient families together with the readouts, preserve the original-case and previous-artifact rows, then examine actual complete own-prefix responses. This changes which existing geometry can learn; it does not require a new cursor, framework or blanket collision audit. That task has since been [executed with exact-family isolation](geometric-context-adaptation-2026-10-03.md); current state owns its next action. This readout-only result neither proves family incapacity nor retires the H4/context design.

## Learning and provenance

Executedsourcee0117909bde8aa7a4f3c13a5973b18fb1f090f3f extends only the standalone example. It reuses the same readout_fit loop, prepared ordinary native-mass answer CE, AdamW, projection, checkpoint/export and generated-output methods. Starting weights are verified coadapt checkpoint0064. Learned context source bits and packed context/exponential tables are fixed; context is excluded from optimizer registration and clipping. Fresh AdamW moments are disclosed. Updates64, B8, lr.003, betas.9/.999, epsilon1e-8, decay0, readout clipping1 are unchanged.

The new training set contains the original20 preservation cases plus eight typed literal sources: `singer dancer singer`, `dancer singer dancer`, `singer Brimfold`, `dancer Louston`, `Brimfold singer`, `Louston dancer`, `singer singer singer`, `dancer dancer dancer`. Source bytes and lexical views are label-free. `FrozenAnswers` Current bare-literal plus period are authored once; the construction panel is exclusively claimed and sealed before fitting. Eight sources are distinct and disjoint from the sixteen existing transfer literals. The cyclic schedule starts with the longer construction B8; the eight new cases receive19 visits each and the twenty original cases18,512 visits total. Per-example answer tokens and EOS share ordinary CE; no teacher/provider generates served responses.

Exports0/16/32/48/64 are independently reloaded and produce original20, construction8 and development16 actual replies. Baseline18/20 exactly replays saved evidence. Compact action traces preserve scores, probabilities, alias masses and prefixes. Every row is compared against start and previous stage; original cases also compare against the original4/20 parent. The inspected transfer panel informed this repair and is development evaluation. This mode explicitly omits another unchanged canonical114 diagnostic; all3,268 actual training positions bind loss to native target mass.

## Verification and resources

Independent source review approves the causal change. Nine focused Rust tests and optimized build pass on CPU-only Runpod Linuxx86_64. Independent arithmetic verifies all3,268 training positions, balanced visits, all220 generated rows; a second raw review checks2,574 generation steps, aliases/ties, first divergences, all regressions and exact initial replay. Source context safetensors, packed context and exponent table each have a single unchanged SHA across five exports. All135 returned files/331,924,095 bytes match remote SHA256 and size inventory. Rust report claim/seal/verify runs on the pod; no independent local BLAKE3 rehash is claimed.

Tests99.049s, build274.936s, fit305.301s; fit maximum child RSS1,620,905,984B. FirstB8 takes5.206s and projects391.076s remaining with evaluation/stop reserve, within1187.572s remaining of the1200s cap. Worker RAM cap8GiB was recorded before use for the longer examples; no cap was silently increased. No GPU, GitHub research runner or laptop model CPU. Metal unavailable onLinux, not counted.

Complete transfer+continuation preparation/build/learning/evaluation/review/return/delivery/cleanup is charged conservatively60min once: cumulative1,064,235,028/1,130,000,000ms, limit unchanged. Pod jobcard is released; ownedarea6,477,288KiB (~6.18GiB), within its10GiB boundary. Unique models, negative rows and executable are retained locally at `~/uor-r4-local/workspace/research/geometric-transfer-20261003/composition-return`. No other lab paths or processes and no session hook were touched.
