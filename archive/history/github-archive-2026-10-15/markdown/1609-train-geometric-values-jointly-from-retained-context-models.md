# #1609 — Train geometric values jointly from retained context models

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T08:35:02Z
- **Updated:** 2026-10-09T13:17:32Z
- **Closed:** 2026-10-02T08:38:39Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1609
- **Merged:** 2026-10-02T08:38:39Z
- **Merge commit:** 66d09f2ff981390e3b5f1ba0afe79b52e790a439
- **Base / Head:** main / UOR-Foundation:codex/geometric-value-joint-fit-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The geometric value producer had an executed learning bridge but no executable continuation from the accepted context models. This adds a Rust joint-fit driver that preserves those weights, learns only context transition families and the new value head, and makes answer loss primary with separately normalized offline packet supervision.

The driver records the untrained baseline, actual training draws/teacher labels, answer versus auxiliary gradients and weight checkpoints. It saves and independently reloads source/native artifacts, then compares every original/stress row with donor/K1/K2, floating learned, compiled and context-reset results. Exact donor imitation is diagnostic rather than an acceptance gate. The roadmap and current-state point to this causal fit, keeping Claude's session lane separate.

Validation: release target-layout/zero/absence/normalization fixture passes; both accepted-parent construction checks complete with zero optimizer updates and nonzero ordinary answer gradients to transition/value families. Independent reload executes and measured state/action/address/packet/Q16/answer differences are zero on eight original and eight stress rows per seed. Release build, fmt and claim-wording check pass. The evidence receipt binds actual sources, executable and reports. No fit, full-serving, language or energy result is claimed.

References #1512, #973.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5948328013 — 2026-10-02T08:37:45Z — Casey-allard

# Final independent review — joint geometric value driver

**APPROVED for protected delivery at exact commit `41db5aebaacfdadb9a671b4dd5ae6d688385e73d`.** The same frozen executable is ready for the separately declared joint fit. No further source or construction check is requested. This approval does not declare the fit successful or already run.

Reviewer: principal_mathematician, independent of the changes reviewed. The reviewer inspected source, committed documentation, retained records, execution logs, and hashes; no Cargo, inference, training, implementation edit, or git mutation was performed by this reviewer.

The exact commit was independently resolved and the worktree was clean. Its six-file delta from library base `2b598f23f4d286b38cccc79e22c6a1afcf207bd4` contains the two example files, three roadmap/current-state documents, and the evidence JSON. No library implementation changed. The earlier source-only approval in `source-independent-review.md` remains applicable, including both resolved review defects and all twelve independently verified historical K1/K2 report/row pins.

The committed driver SHA256 is `6487df6ffb6208c3ed2a65a99d8369d2e7f6312702ad96a3570e18ed5ba57809`; helper SHA256 is `f8e3088878f8d82550e5a00491d77a0f070a67ca129d03fda2c912b2f90d1682`. Both match the previously reviewed source and the final evidence JSON. The retained release executable independently hashes to `2423c58c4b03cc0d246f1a561de3b3fce430e63516cde4bead530565b447011c`, matching both execution receipts and both report input records. Embedded source-commit metadata is explicitly `UNAVAILABLE`; the receipt instead records the exact source hashes and unchanged library base. The documentation does not present an unavailable embedded identity as verified metadata.

Both actual-parent checks exited zero. Each used check mode, completed exactly one backward batch, and performed zero optimizer updates. Seed1 covered 296 actual positions, with answer denominator 8 and root/category denominators 4736 each; seed2 covered 288 positions, denominator 8 and 4608 respectively. These match sixteen primitive atoms per actual position, and all recorded ordinary-answer gradient norms for the three context transition families and ten value factors are finite and positive. These observations establish joint answer-credit availability on these actual inputs. They do not isolate the value path from address credit or guarantee later learning; the separately retained isolated integration fixture carries that narrower attribution.

Initial and final-check context parameter files independently retain each accepted parent's exact bytes: seed1 `0a2f5f1903c4e4101ea2f2b9fef5062d73917e99ac6b9bda8bafceb7fcf7a3a8`, seed2 `fb8133d136af654488bff8f6c7d1e5be7444a9394e44a6da09d9bed5e8896f8a`. Initial and final-check value parameter bytes also match within each seed. This corroborates zero updates; a directory named `trained` in check mode is not evidence of fitting. Source reload and native compile/reload were actually reached by the successful checks.

Each seed completed eight original and eight stress rows. The raw row files independently contain eight rows per panel, zero floating/native answer-prediction changes, zero primitive-packet differences, and zero raw-Q16 coordinate differences. Reported actual-position latent state, transition-action, and emitted-address disagreement counts are also zero. These are finite construction observations, not global compilation equivalence. Floating/native logits are not bit-identical: the maximum reported differences are 0.00018310546875 for seed1 and 0.00011110305786132812 for seed2. The final receipt includes those differences, so its zero-count statement does not conceal them.

Report hashes independently match the committed receipt:

- Seed1 report: `027b14e5399a3e7d3143d1282e32e1a4059a78f8a1cac7ca8a6ebc11a566f856`; execution receipt: `bdd5a51eafa7c9d3af12065073116d8d3f8c9d2826f938a1494594a378f4dd74`.
- Seed2 report: `2572cb6a7298e44a7bf350fa4b271aceb263d08f523b3b7a04321b884e6f9e61`; execution receipt: `b871d7575023461dc938d5199a5a538754e88535feaae3b961caa22f044f9874`.

All four check-log hashes also match the receipt. The release log records one passing target-layout/mask/normalization fixture after a 3m34s build, and the normal executable build completes in 3.87s. Claim-wording output records a pass; formatting output is empty as recorded by the successful root check. These are root-executed checks inspected by the reviewer, not additional reviewer executions.

The committed roadmap, current-state entry, and evidence consistently classify this as an implemented and exercised construction driver with `fit_status=NOT_RUN`. They preserve the primary-answer objective, separately normalized auxiliary packet losses, continuation of both retained parents, frozen observation/scorer/event/span/base components, explicit partial-budget outcomes, and row-level comparison obligations. Exact donor reconstruction is not a gate. Reset affects context while retaining span memory. No new fitted quality, natural-language capability, geometric superiority, final-linked opcode result, energy result, or complete integer-serving claim is made.

No unresolved material blocker remains for protected delivery or execution of the declared fit with the same frozen binary. Fit outcomes must be reviewed from their own actual artifacts and must not inherit a positive quality verdict from these zero-update checks.



---
_Comment/review ids in this file: 5948328013_

