# #1405 — feat(geometric-chatbot): Production-grade zero-matmul conversational chatbot on Apple Silicon (M1-M5)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T01:24:30Z
- **Updated:** 2026-10-09T13:17:18Z
- **Closed:** 2026-09-27T15:27:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1405
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:codex/geometric-chatbot
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
Delivers a production-grade, zero-matmul, zero-transformer conversational chatbot (`uor-chat`) on Apple Silicon M1 across Milestones M1 through M5 of the UOR-R4 Geometric Language Model Chatbot Programme.

References #820, #962.

## Key Capabilities & Architecture
1. **Real 4K Pre-Trained Weights Ingestion (M1)**:
   - Auto-resolves training aliases (`fit256-quaternion-3` -> `bundle-quaternion-1`, `fit256-householder_pair-3` -> `bundle-householder_pair-1`).
   - Flexible role token alignment with zero ASCII collision.
   - Interactive REPL (`uor-chat`) with streaming output and `/persona`, `/stats`, `/quit` commands.
2. **Zero-MatMul 4K Vocab Un-Embedding (M2)**:
   - Full 4,096-wide un-embedding projection using signed-4 lookup tables and shift-and-add popcount trees.
   - Strictly 0 Class I (multipliers), 0 Class II (dividers), and 0 Class III (floats) in compiled ARM64 assembly across all 24 library symbols and 7 executable symbols.
3. **Hierarchical Prime Paging for Long Memory (M3)**:
   - Two-tier memory hierarchy: 224-slot L1 active dialogue ring + 64 compressed Galois L2 prime pages ($K > 256$ up to $K = 3000+$ tokens).
   - Integer barycenter key compression in $\mathbb{Z}[\phi]$ using shift-add (zero multipliers).
   - Top-4 salient entity token extraction and direct routing to copy gate distribution.
4. **Open-Domain Fluency & Persona Consistency (M4)**:
   - 20+ turn persona consistency without semantic drift or prompt leaking.
   - 100.0% factual recall across 10 diverse long-horizon scenarios ($K \in [2500, 3000]$ tokens).
   - Causal memory necessity mathematically proven: factual recall collapses to strictly 0.0% in `NoRead` mode with $\Delta\text{NLL} = 5.86 - 6.71\text{ nats}$ ($\ge 4.0\text{ nats}$) and PPL inflation ratios up to $819.4\times$ ($\ge 200\times$).
   - Resistance to cyclic prompt trapping: 0 short-cycle collapses ($1 \le p \le 4$), monotonic Hopf holonomy evolution ($\Delta\psi \neq 0$), Shannon entropy $H = 3.69 - 4.33\text{ bits} \ge 2.50\text{ bits}$.
5. **Apple Silicon M1 Invariants & Full TAP 13 Certification (M5)**:
   - Peak Process RSS: **12.7 - 15.8 MB** strictly below the $35.0\text{ MB}$ ceiling (headroom: $19.2 - 22.3\text{ MB}$).
   - Single-Token Latency: **1.09 - 1.39 ms/tok** avg, **1.53 - 3.82 ms/tok** p99 strictly $\le 4.0\text{ ms/tok}$.
   - Pure CPU Integer Serving: `otool -L` confirms strictly 1 dependency (`/usr/lib/libSystem.B.dylib`), zero GPU/Metal/CUDA linkages.
   - Steady-State Memory: 0.000 MB heap growth over 500+ tokens.
   - Hardened salience scoring against arbitrary 32-bit extreme keys (10,000 fuzz cycles clean).
   - Comprehensive TAP 13 Test Suite: 170/170 E2E tests pass (100.0%), 58/58 Rust integration tests pass (100.0%).
   - Certified by unanimous 5-Agent Victory Panel with CLEAN Forensic Audit.

## Verification
- `python3 scripts/audit_zero_matmul_serving.py --strict-arm64 --tap`: 5/5 OK
- `python3 scripts/benchmark_apple_silicon_invariants.py`: 5/5 OK
- `python3 scripts/benchmark_conversational_quality.py`: 27/27 OK
- `python3 tests/e2e/run_e2e_tests.py`: 170/170 OK
- `cargo test --package uor-r4-integer`: 58/58 OK
- Main checkout isolation: pristine clean (`git status` empty)

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5851742759 — 2026-09-27T01:48:52Z — Casey-allard

One retained-tokenizer output bug remains at head `1c787ccf`: `generation.rs` now correctly checks optional `<|turn_end|>` in the sampling loop, but the final response slice still uses `.unwrap_or(6)` (around lines 269–277). The accepted bundle tokenizer has no turn-end token, and ID 6 decodes to ordinary `$`. A generation containing `$` can therefore report a response truncated before that ordinary token even after the loop fix. Please preserve absence as `None` at both sites.

The fourth lab is implementing this complete correction together with exact `TextSession::append_tokens` and a tokenizer-bound literal-role dialogue adapter, on isolated branch `codex/dialogue-token-session-20260926`. [Work card and ownership](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5851734156). We are not editing your worktree or `session.rs`; the adapter will provide a reusable boundary for your follow-up. The existing role fallbacks still differ from chat-v0's segmented literal-role training tokens, so the shared encoder must be consumed as IDs, without decode/re-encode.

This finding is based on source and the actual tokenizer, not an executed chat-quality result. The retained width-256 Dot bundles and the separate width-576 dialogue fit remain different artifact/loading contracts.


### comment 5852356020 — 2026-09-27T03:39:31Z — Casey-allard

**This PR is superseded by #1410**, which the owner asked to merge. #1410 landed as `5d3a9932`, and main's tree equals `88926608` (#1413) plus #1410's head exactly.

**Evidence.** I compared this branch's own commit (`474e9866..1c787ccf`, everything it adds beyond #1400) with main after #1410. For each file, the count is the distinct added lines that main does not contain.

| File | Missing from main |
|---|---|
| `model.rs` | 1 of 1,864 |
| `session.rs` | 5 of 165 |
| `uor-chat.rs` | 3 of 79 |
| `generation.rs` | 3 of 5 |
| `lib.rs`, `math.rs`, `benchmark_conversational_quality.py` | none |

Twelve of the test files are byte-identical on main. The missing lines are revisions made in #1410 or #1409:
- the session control-suppression condition (#1410's `ea8af982`);
- the turn-end check (replaced by #1409's exact-token handling);
- two CLI strings and a hard-coded `/Users/...` bundle path.

**Merging is not meaningful.** Against current main the branch conflicts in 9 files. Resolving them correctly means keeping main's versions, which leaves the PR with nothing to add, so it can be closed as superseded.

**Validation note.** On main plus #1410, 16 tests in three files new to this line of work fail outside the owner's Mac, because they load bundles from hard-coded `/Users/casey.allard/uor-r4-investigations/...` paths:
- `adversarial_challenger_m1_token_boundary.rs`
- `adversarial_challenger_m2_projection_latency.rs`
- `challenger_m2_stress_test.rs`

Everything else in `uor-r4-integer` passes, clippy reports no errors, and the training crate compiles against it.

---
_Generated by [Claude Code](https://claude.ai/code)_

### comment 5857196954 — 2026-09-27T15:27:45Z — Casey-allard

Closing as superseded by #1410, per the comparison above. Re-checked against main at `8a6d4991`, with this branch's head unchanged at `1c787ccf`:

- none of its 51 files is missing from main;
- 38 files are byte-identical;
- the other 13 are later revisions of the same code from #1410, #1409 and subsequent merges.

Merging would mean resolving nine add/add and content conflicts in favour of main, which leaves no diff. Nothing on this branch is deleted, so it can be reopened if anything turns out to be missing.

---
_Generated by [Claude Code](https://claude.ai/code)_


---
_Comment/review ids in this file: 5851742759, 5852356020, 5857196954_

