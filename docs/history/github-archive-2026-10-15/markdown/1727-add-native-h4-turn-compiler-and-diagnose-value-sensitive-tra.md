# #1727 — Add native H4 turn compiler and diagnose value-sensitive transfer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T06:58:34Z
- **Updated:** 2026-10-09T13:17:26Z
- **Closed:** 2026-10-05T07:00:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1727
- **Merged:** 2026-10-05T07:00:47Z
- **Merge commit:** 6699796b1a1e2e53c7cacdc9a65ec5c0d47b0b76
- **Base / Head:** main / UOR-Foundation:codex/geometric-native-compiler-20261005
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Raw-language native turn compilation was missing from the geometric store/reader path. This adds a saved signed-H4/Q4 compiler using existing TurnCompiler, Example and exact byte-span contracts, plus an actual predicted-write/read/reload/native-consumer research driver. The retained fit learns all152 training actions but fails value transfer: familiar wording/new values gets1/64 exact writes. It is not adopted as a serving default.

The zero-update audit separates that compiler failure from the consumer: an authored exact-template control passes the same compiler instrument and actual store4/4, while native selected-record answers are2/4 and all-bank0/4 with natural statements/questions. Reports bind checkpoint step/hash and actual input identity, distinguish executed calls from NOT_RUN, seal failures and preserve every checkpoint. Current state and the plan now prioritize stable local word/predecessor/frame observations and independent reader input parity. References #820, #1552, #973.

Validation:
- Linux CPU-only: two integer and three training compiler tests PASS at aeeea0c0; exact-template control test and release example build PASS at abe7a05a. Fit64updates complete; diagnostic audit0updates, no reselection/fresh evaluation.
- Final documentation head contains identical Rust/source implementation to abe7a05a. Exact-head touched-file rustfmt, claim-wording and diff checks PASS. Whole-workspace formatting finds pre-existing unowned relation_compiler.rs formatting; not edited. Metal unavailable, not counted.
- Independent source review PASS at abe7a05a and saved-outcome review144 checks/0errors; these are separate from execution. Final documentation review posted below.
- All build attempts1216.791s through audit; fit worker2.227s, audit2.220s; original120min resource card unchanged, CPU48/RAM8GiB/noGPU. All checkpoints/shadows/sealed results round-trip MD5 verified in cloud-store, geometric-native-compiler-retained-20261005, outerMD5 f99cbcda0f9c155249ca667e31c4e53b. Pod owned7.9GiB, no live Codex job at audit completion.

Scope: authored two-relation component and plumbing diagnostic on one frozen carrier, not chat, general reasoning, energy or a replicated architectural verdict. Three actual learning seeds and two independent chat lineages are required before those corresponding verdicts; panel draw seeds and repeated deterministic fits do not count. Shared stack_grounded_session.rs/relation_compiler.rs unchanged.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5989673951 — 2026-10-05T06:59:55Z — Casey-allard

Exact-head delivery review PASS at4fff85c7a53fee382918262a581ecf0d9c1c26b3, tree97fbf95586c39f94341126cf678cf1633973e59b; no blockers. Independent reviewer confirms onlyfour documentation/evidence paths differ from tested audit sourceabe7a05a: Rust implementation byte-identical, report/review identities match, no reader/seed/chat/default claims inflated. No fabricated final-doc-head model rerun.

Actual validation remains source-bound: integer2 + training3 tests at original fit sourceaeeea0c0; reference exact-template unit test/release driver/zero-update audit atabe7a05a; all PASS. Actual Linux CPU run receipts, commands, exits, hashes and limitations are in docs/evidence/geometric-native-compiler-2026-10-05.json. Final-head touched-file rustfmt, claim-wording and whitespace/source-delta checks PASS. Whole-workspace formatting has pre-existing unowned relation_compiler.rs formatting, reported explicitly; no Metal test ran. Five GitHub historical status names are compatibility acknowledgements, not executed tests.

144 independent saved-outcome arithmetic checks/0errors, with model/BPE/gradient reexecution excluded. Exact-template compiler/store success remains authored plumbing; native all-bank0/4 is retained. All checkpoints/shadows/sealed reports in cloud-store geometric-native-compiler-retained-20261005; outerMD5 f99cbcda0f9c155249ca667e31c4e53b round-trip verified and specific index read verified. Local staging duplicate removed; unique local receipts and pod original artifacts preserved. No live Codex pod job, /root/codex7.9GiB.



---
_Comment/review ids in this file: 5989673951_

