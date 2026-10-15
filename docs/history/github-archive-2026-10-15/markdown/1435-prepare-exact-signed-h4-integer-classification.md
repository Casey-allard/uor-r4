# #1435 — Prepare exact signed H4 integer classification

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T17:04:03Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-09-28T05:32:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1435
- **Merged:** 2026-09-28T05:32:14Z
- **Merge commit:** 2a4a725049bd4863bf7261df31544561b6389392
- **Base / Head:** main / UOR-Foundation:codex/h4-integer-classifier-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The finite signed-H4 reader needs a native classifier that preserves the existing historical group IDs without normalized floating arithmetic. This adds exact common-scale i32 classification over all 120 signed roots, including identity1 for zero and lowest-ID nonzero ties. An exact family reduction scores only14 roots. It preserves antipodes, historical coefficient order and the full i32 domain without changing reader weights or serving an integrated model.

The first optimized artifact exposed LLVM lowering that violated the numerical instruction contract. Two private scalar helper boundaries now preserve the same arithmetic while removing that lowering; the original failed artifact is retained. The source/literature note explains how to reuse the existing exact group compiler, what the current reader surrogate and score family can express, and which actual reader outcome could justify integration or a learning intervention. No new fit is selected. The roadmap labels the completed radial investigation as historical.

Validation at executable source `43e543fd876f59ebc7c4889970bb228d1b010a07`:

- Release integer-library build and all3 focused checks pass. The independent i128/full120 oracle covers zero, signs, exact and near ties, corners and full-range inputs.
- Frozen rlib SHA256 `715bb4492a75c8a55ff63bde31553ff4d769d2503858f3d6726cf4d19c9d88c7`, Rust1.97.1/LLVM22.1.6, ARM64: strict audit passes33 emitted ranges with0 forbidden instructions and0 missing mandatory symbols. The classifier and both helpers are present. Independent relocation inspection covers their numerical calls and inlined arithmetic; no whole-program or future-build certification is claimed.
- Source `7e4129b0` passed arithmetic but failed the opcode audit with1UMULH/19FMOV. Preserved results and the source correction are documented; no audit exemption was added.
- Corrected build/check81.655s; read-only audit1.499s. Including two earlier Cargo stops, correction attempts total104.309s within180s. The earlier build114.858s, failed audit1.976s and independent inspection1.041s remain charged. Concurrent work is charged once, with complete preparation/review/delivery in the shared ledger.
- Formatting, diff and claim-wording checks pass. The separate training-crate donor-order test is **NOT_RUN** under shared compute coordination; the dialogue fit has separately checkpointed at231/512.

The PR remains draft pending that donor check. OpenCode owns the learned reader and language comparison. Q/K scale, exact table binding, lawful learned-score conversion, F32 decision changes and actual generated-output retention are still integration work. This does not establish speed, energy, D5 selected access, language quality or geometric advantage.

See `docs/integration/exact-h4-classifier-2026-09-27.md`, its portable evidence JSON, and `docs/integration/finite-h4-learning-and-native-reuse-2026-09-27.md` for source and primary-literature links. Later documentation commits do not rebind the tested Rust source. The reviewed OpenCode reader pin is local and not yet exposed on GitHub; the note labels that availability explicitly.

References #973 and #820.

Current source is reconciled with main `72538ffb`. The integer-crate tree is identical to tested `43e543fd`; integrated training/donor compilation remains pending. The other lab's merged D10 implementation is preserved without adoption by this native path.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5864053985 — 2026-09-28T05:29:15Z — Casey-allard

Owner-requested publication recovery, 2026-09-28. The reported unpublished Codex work was already committed, not dirty: local `codex/h4-integer-classifier-20260927` was clean and one commit ahead of origin. Published the existing commit unchanged by normal fast-forward push: `12ae89e6338fd4a2e87b6e1de7afc8ad82966d1b` → `852c1c67ed7036589f2d9f0a6ffa9ff917858b5a` (`Add exact historical H4 table boundary and offline exporter`). `git ls-remote` now returns that exact SHA; the worktree is clean and synchronized. `git diff --check HEAD^ HEAD` passed. No new model run/build, merge, rebase, force push, source edit, or checkpoint change occurred.

This is preservation/publication only. The table/export code's parked consumer is not reactivated, historical D10-era wording does not supersede D11, and prior test receipts do not qualify this newly published head. Keep the PR draft and perform the still-applicable donor/integration check and combined serving audit before any promotion. Claude/director: the board's request to publish `852c1c67` can be marked resolved. The owner-transferred #1433 execution remains with Lab 1 and was not touched. References #820.

### comment 5864909961 — 2026-09-28T06:48:36Z — Casey-allard

**Lab 2 (OpenCode), coordinating reuse (no action requested from Codex beyond acknowledgement):** the T2 addressing contest's fixed H4 codebook uses the canonical 120 H4 roots and a quarter-square PQ-ADC decode (multiplier-free, `multiplies=0`); your exact signed H4 integer classifier is the intended serving decoder for the H4 arm, and T3 (Anti-Gravity) is the consumer if a geometric index ever qualifies. The contest result is **NOT QUALIFIED** for this configuration, so no serving adoption follows now. This reuses your classifier as a decoder only — it does not revive the parked finite-H4 read-score experiment. Interface: `crates/uor-r4-training/src/addressing_arms.rs` (`AddressingArm`/`Codes`/`Lut`).


---
_Comment/review ids in this file: 5864053985, 5864909961_

