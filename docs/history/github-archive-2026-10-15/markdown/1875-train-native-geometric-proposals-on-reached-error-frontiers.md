# #1875 — Train native geometric proposals on reached error frontiers

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T04:11:13Z
- **Updated:** 2026-10-08T04:58:09Z
- **Closed:** 2026-10-08T04:58:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1875
- **Merged:** 2026-10-08T04:58:08Z
- **Merge commit:** 9af3873e19e6e0270ac1d0625b3bf61fc287de60
- **Base / Head:** main / UOR-Foundation:codex/astra-reference-replay-20261008
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The previous native-code proposal improved loss mostly on teacher prefixes the model never reached. This adds an opt-in Rust preparation and learning path that freezes actual first-error frontiers plus complete accepted trajectories, and uses the same explicitly weighted contexts for gradient ranking and native candidate selection.

The original accepted parent yields 504 failed-entry terms and 84 successful-trajectory terms, with one unit of objective weight per component. The fixed 18-slot Context/Potential/Generate/U bank executes 17 candidates and retains one native no-op; there is no B8 sampling, Adam step, adaptive replacement or repeated round. Full Generate/physical Copy support, independent native exports, frozen prototypes/bridge, and previous modes remain intact.

Actual execution at `c0bb0861cf529be0c95e752aaccab4c82dd09801` selects one U code change. Native CE falls **9.0839949284 → 9.0120230102**, but complete accepted replies remain **8/512 → 8/512**. All eight original replies and all 398 diagnostic positions remain correct; 44 generated sequences change and no additional reply completes. Every one of the 588 scored prefixes is reached by both endpoints. The selected U class improves 126 failed-entry target probabilities without changing any to a correct winner. This is bounded optimization evidence, not useful-chat qualification or model promotion.

Validation: executed-source formatting, 30 focused fitter tests (zero failures or ignored tests), and release CUDA build pass. Initial and final actual graph/native parity each cover 588 positions. Independent saved-evidence readers verify trajectories, all 18 proposal slots, 11,172 objective occurrences, native packing, untouched masters, and final selection. Separate Rust complete-file seal verification passes. Reader-only serialization/schema assumptions were repaired with failed attempts retained; the model was not rerun. These readers are not independent forward or backward replications. Final-head validation and preservation receipts are posted below before queueing.

The current-state entry retains the accepted parent, records the measured negative, and names the next hypothesis: parent-preserving learning of the existing categorical selected-source-to-output action map alongside Context. That mechanism is NOT_RUN and is not claimed to be a proven remedy. No unchanged wider bank or dose sweep follows from this result.

References #820. [Prospective work card](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6051865011); [588-position parity clarification](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-6051973935).


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6052108203 — 2026-10-08T04:17:42Z — Casey-allard

Exact-head implementation/admission checks at `c0bb0861cf529be0c95e752aaccab4c82dd09801`: changed formatting PASS; actual release CUDA fitter30tests PASS/0failed/0ignored (102.13s including compilation); release build PASS18.94s. Mathematical and adversarial exact-head source reviews find no remaining blocker. These are actual scoped checks; CI compatibility statuses are acknowledgements only.

Rust preparation COMPLETED with zero native calls/optimizer steps. Plan SHA256 `de64f5103982908edaa94a5bec04cf4e4ace0ea8e82460ba0b7ef759166589a0`, preparation report `d87b782656d117772c3ec5c6b2ab07b1fb68647152b8cb04b78589eab2c077d8`, manifest `c029ace269d5a417eb012492e3839170e32bb37a9eb362ff5c32eebe6aea927a`. Independent saved-evidence reader PASS matches all512 trajectories,588 explicit terms, actual prefixes/weights, physical Copy IDs, and separate398 old-reference identities/weights. Preparation1.839736s wrapper; reader1.141661s. This has not yet measured a new trained candidate.

Runtime/source/checks/plan/reviews preserved in private HF `caseyallard/uor-r4-store`, revision `fe135c9e885d5a5e9e869d74200e5cc45fc5fd7b`, prefix `codex/astra-native-frontier/runtime-c0bb0861cf-attempt1`, manifest SHA256 `20b1110e3c4918951755f420757a9008d6f42594b3f1ddf69a178f4ed85ab9f5`. One owned4090 admitted through uor-pod within current caps for the fixed one-round pilot; worker is preparing and will verify every downloaded runtime/parent file before execution. Actual pilot remains NOT_RUN at this comment.


### comment 6052604144 — 2026-10-08T04:57:04Z — Casey-allard

Exact-head delivery review and executed checks — `1b71a81b1d6a5c5662ce4712acc137266427ed30`.

Independent adversarial source/document review PASS at this exact head. The three Rust files are byte-identical to reviewed and executed `c0bb0861cf529be0c95e752aaccab4c82dd09801`; the new current-state section matches the independently audited8→8 result and retains the categorical-map successor as an unexecuted hypothesis. Current main `ae4f3b5b155362e0e9b19abfde172a6c381016c4` is included.

Actual final-head checks on an owner-authorized validation pod, GPU masked, two CPU threads:

- Changed-path `rustfmt --edition 2021 --check --config skip_children=true`: PASS,0.073s.
- `cargo test --locked --offline --release -p uor-r4-training --example geometric-frozen-map-fit`: **30 passed,0 failed,0 ignored**,215.930s including cold compilation.
- `cargo build --locked --offline --release -p uor-r4-training --example geometric-frozen-map-fit`: PASS,5.443s.
- Claim-wording and `git diff --check`: PASS locally. Claim wording is advisory about wording, not scientific validation.

These final-head compile/tests use default CPU features. The separately identified executed c0bb source also passed CUDA-feature30tests/release build and actual initial/final588-position graph/native parity. No model experiment was repeated for final delivery. Compatibility status names are queue acknowledgements, not these test results.

Independent saved-evidence endpoint/candidate readers and separate Rust complete-file BLAKE3 verification pass. Candidate reader independently reconstructs all18 proposal slots and11,172 objective occurrences; source/packing, untouched masters, selection and actual full512 own-prefix outcomes match. The retained serialization/schema reader faults were repaired without rerunning the model.

Preservation: private HF `caseyallard/uor-r4-store`, revision `30313ab2feff435af8ec247254414ccdb0cf36d1`, prefix `codex/astra-native-frontier/frontier-0001-c0bb0861cf`;427,844,306-byte archive SHA256 `bf3af0934b993c990712b675f12a3d26f9ec7426e078519197aca0e52650af86`. Fresh archive and manifest downloads match. The model pod is released/deleted. The previous shared compile host had been deleted by its owner; a recorded necessary3600s cold-delivery extension raised the cumulative projection339270→342870s before replacement validation compute. Existing spending caps remain unchanged. The replacement validation host will be deleted after final delivery receipts are preserved.

Manual delivery evidence fallback: deployment of the admitted delivery runner remains UNVERIFIED. The exact-head source review and actual checks above provide the documented manual evidence; no administrative bypass or fabricated check is used. This PR references #820 and does not close the programme acceptance.



---
_Comment/review ids in this file: 6052108203, 6052604144_

