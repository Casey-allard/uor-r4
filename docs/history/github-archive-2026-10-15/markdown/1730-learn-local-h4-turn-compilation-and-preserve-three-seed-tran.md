# #1730 — Learn local H4 turn compilation and preserve three-seed transfer boundaries

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T07:58:11Z
- **Updated:** 2026-10-09T13:17:26Z
- **Closed:** 2026-10-05T08:05:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1730
- **Merged:** 2026-10-05T08:05:29Z
- **Merge commit:** af1fc0a620a97fdbe711e550d1acdc50085f0439
- **Base / Head:** main / UOR-Foundation:codex/geometric-local-compiler-20261005
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Original endpoint-only turn compilation fits training but fails arbitrary-value transfer, blocking the geometric compiler/store/emitter path. Add masked local current/predecessor/next-word H4 observations and learned Q4 additive heads, with directed-relative and ordered-product modes, real seeded initialization, legacy artifact compatibility and exact original byte spans.

Six source-bound fits (three actual head seeds per arm on one frozen encoder) still select initialization by native development CE and produce no integrated store/native completion gain. Retained trained checkpoints improve separate wording/value diagnostics, without joint transfer or a transport winner. Preserve all negatives and update current state/roadmap toward independent phrase×value training; no default adoption or shared-session change.

Validation: executed source abb20ef7: three integer, seven training, two driver tests, release build, touched-file formatting and claim-wording gate pass. CPU-only approved Linux pod; six fits complete exit0. Independent saved artifact audit passes1,421,046 checks (all30 shadow/Q4 payloads, genuine paired seeds, selection/artifact identity and complete matched inputs). This is not native/encoder/optimizer replication. Full150-file return byte-set/SHA verified and retained with receipts/reviews in iCloud. Final current-main compatibility check is being posted before queue admission; compiler source remains identical to the executed source. Relative/product group-operation counts differ; old Endpoint has fewer parameters. No general chat, energy or family verdict.

References #820, #1552. No issue is closed.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5990520372 — 2026-10-05T08:04:36Z — Casey-allard

Final PR head `90c2ade639d49321ffa1365a56f053d10c56c663` validation complete before protected queue admission:

- Approved CPU-only Linux pod, exact committed source inventory/complete SHA-set checked. `cargo check --offline --release -p uor-r4-integer -p uor-r4-training --example geometric-native-compiler` exit0; `cargo test --offline --release -p uor-r4-integer --lib geometric_turn_compiler::tests -- --nocapture` exit0,3passed. Final-head wrapper18.555s. Prior current-main compatibility wrapper57.029s; total build/check worker time936.316s within declared1200s. Startup readiness/log-collision attempts performed no Cargo/model work, retained separately as infrastructure defects.
- Compiler's three Rust files are byte-identical to executed `abb20ef7`: training7/driver2/integer3 tests and release build pass there; all six model fits terminal exit0. No extra fit or criterion change after results. Final touched-file rustfmt, diffcheck and claim-wording check pass.
- Independent final-head source/docs review PASS. Independent saved parameter/outcome review passes1,421,046 checks; all30 native Q4 payloads reconstructed from F32 shadows, three genuine seed streams, paired initial heads, selected artifact equality and earliest development minimum. Does not rerun encoder/BPE/native/optimizer/BLAKE3 or independently recompute CE from raw scores.
- All six select initialization; retained trained checkpoint diagnostics are not adoption. Exact source/byte spans and actual predicted-store results remain in the evidence note. General chat, architecture/energy advantage and full serving remain unqualified.
- Complete artifacts already roundtrip retained in iCloud; full150-file return equality/SHA checked. Pod8.0GiB; no Codex job/GPU running. Final review/compatibility receipts retained separately. Ruleset acknowledgements execute no scientific tests; not counted above.

References #820/#1552. Exact-head self-review allowed; no admin bypass requested.



---
_Comment/review ids in this file: 5990520372_

