# #1127 — Add resumable Rust memory fitting and explicit response supervision

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-05T03:42:43Z
- **Updated:** 2026-10-09T13:17:54Z
- **Closed:** 2026-09-05T03:59:58Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1127
- **Merged:** 2026-09-05T03:59:58Z
- **Merge commit:** 3abf9d7e85f70416c95161863b4413cc42a6912c
- **Base / Head:** main / UOR-Foundation:codex/resumable-native-memory
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The existing memory fitter supervised one bounded vector repeatedly: on the expanded corpus, only 4,096 of 147,652 positions received loss. This adds `geometric fit-memory-stream`, separating distinct target exposure from the live buffer and restoring actual weights, feature registry, stage/cursors, calibration sums and source/configuration identities. Calibration and epoch selection replay the same full population. Optional source-bound token spans focus loss on authored responses while every token still builds causal geometric memory; existing fitting and inference behavior remain available unchanged.

A native Rust joint probe prepares balanced whole-task prose/Rust sources, validates splits and token boundaries, saves actual outputs, compiles unchanged generated code and uses real compiler feedback for bounded repair attempts. Expanded supervision raised the same eight-epoch broad comparison from 32.8879% to 35.3499%, still below memory-disabled at 36.2485%. Whole-population and response-only fits reach 6/32 and 7/32 exact prose completions, respectively, versus 7/32 memory-disabled; both remain 0/32 exact Rust and 0/8 sampled compilations. Response masking does not establish broader quality. The preserved finite reader still answers 96/96 in each family. Records retain all positive, partial and negative outputs, registry saturation and the remaining learned read/operator/write limitation.

Validation: 25 native core tests, 13 native CLI/service tests and 4 joint-probe tests pass, including exact interrupted/resumed artifacts, batch-size invariance, full-population calibration, supervision identity and source protection. Release CLI/probes compile and run; focused Clippy, formatting, architecture policy and claim wording pass. Independent implementation/result review found no blocking issue. The protected workflow's native test job will be checked separately; its other four required status names are compatibility acknowledgements, not audit/fuzz/WASM/Gate-C evidence.

Model work remains within the inherited allocation: 716.243/1,800 seconds cumulative, including preparation, failed preparation, fitting, resumes and evaluation; no paid compute. Both alpha capability groups and learned geometric writes/composition remain open. References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

