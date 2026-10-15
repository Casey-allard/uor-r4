# #1386 — Adopt integrated attention programme and deliver the first matched Rust artifacts

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T17:06:59Z
- **Updated:** 2026-10-09T13:17:34Z
- **Closed:** 2026-09-24T18:15:16Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1386
- **Merged:** 2026-09-24T18:15:16Z
- **Merge commit:** b020f34a09816ba235984cedde1c50f4c8e9cae2
- **Base / Head:** main / UOR-Foundation:codex/integrated-attention-a1-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Owner-adopted programme and first integrated artifact

References #973 and #820. The owner directed cleanup, adoption of the principal attention programme and implementation. D7 records that direction; the programme is also the active header of both issues. D0-b and D4–D6 remain unchanged.

### Changes

- Recover 4.84 decimal GB of physical disk by deleting four checked regenerable Cargo incremental caches. Preserve source, models, research, histories, binaries and warm dependency libraries.
- Implement one Rust integrated model artifact/session: 2I or C120 finite geometry, contextual keys/queries, exact bounded raw history and owned token records, unary/pair relative energy, selected state actions, read/write gates and a normalized sparse Generate/Copy/Stop output tree.
- Add source/data/tokenizer-bound Rust fitting, export, reload, generated-output panels, snapshot replay and a read-only source diagnostic.
- Publish the current result, complete generated text and provenance, the precise failure diagnosis, resource receipts and the next same-model learning decision.

### Executed result

The first pair learned zero writes because weak missing future-use annotations were treated as negative writes. Preserve those artifacts. One prospectively recorded positive-unlabeled all-write warmup corrects the label error without changing data/seed/epochs/rates.

The corrected pair indexes all 6,160 development tokens. C120 admits 271/1,421 annotated sources; 2I admits 251/1,421; neither selects an annotated source correctly. Read-enabled development loss is 9.383427 bits/token for C120 and 9.533292 for 2I (NoRead 9.380686 / 9.554751). Generated answers and prose/code continuations remain degenerate; 0/4 read-enabled correction variants are answered correctly per arm.

The loaded diagnosis resolves the mechanism: at all 28 inspected answer positions per arm, all 64 candidate records have one identical four-lane code and tied energy. Correct-source rank before gating is 0/28. An offline forced-correct-source head probe also worsens total answer NLL. This supports a coupled discriminative-address/source-to-language training change; additional energy weights cannot distinguish identical codes. It is not a geometry-family rejection.

### Validation actually run

- Offline library and driver checks.
- 25 focused integrated-module tests at initial implementation source de3c2acd; 1,129 unrelated tests filtered out.
- Release builds from committed source for initial fits, the write-objective correction and read-only inspection.
- Four complete fits, four separate-process replays and two read-only inspections. All report roots claimed, sealed and verified.
- All 48 generation rows reproduce after snapshot restoration and byte-for-byte in a separate process; each complete evaluation JSON also reproduces.
- Named Rust formatting checks, whitespace check and capability wording check.

Actual generated-step logical parameter reads are about 7.3–7.8k/token, including code formation, against 40,038,150 stored parameter bytes. Maximum observed model-run RSS is 393,330,688 bytes. These are not physical DRAM/energy measurements or whole-kernel D0-b qualification. Offline diagnostics use floating point; served model decisions use selected integer tables/rows.

### Scope and delivery

Milestone A remains OPEN. This is a retained development candidate, not the default application model, useful language qualification, learned selective writing, nontrivial frame transport or geometric superiority. The next integrated change trains distinct causally available source codes and candidate-conditioned language utility together, with contrastive alternatives and diverse grounded prose/Rust plus natural next-token learning.

Owner-authorized main delivery: this PR is retargeted to `main` and includes the complete reviewed stack #1380, #1381, #1382, #1383, #1384 and #1385. It will land as one consolidated change through the required protected merge queue; the component PRs retain their history and evidence. Source head: `d1f5524c1d2b302cf2c07fea6b47309460b85f66`. Earlier experimental results remain negative or narrowly scoped as recorded; inclusion does not promote their claims. Historical CI/merge queue names are compatibility acknowledgements, not test results. The shared ledger and storage closeout are committed in `docs/evidence/integrated-attention-a1-closeout-2026-09-24.json`.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

