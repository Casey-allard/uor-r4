# #2264 — Use exact SpiralCore axis transport and record shared native vectors (References #2030)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T02:04:59Z
- **Updated:** 2026-10-13T02:12:10Z
- **Closed:** 2026-10-13T02:09:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2264
- **Merged:** 2026-10-13T02:09:04Z
- **Merge commit:** 862f6363d38456df3f6f4dbcfeaf400fdbfe1312
- **Base / Head:** main / UOR-Foundation:codex/native-vector-transport-20261012
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

SpiralCore finite actions previously composed with a generic 8×8 numeric contraction, and E8 root-table construction duplicated that loop. They now compose and apply the existing geometric action by signed axis selection, preserving the canonical representation and fixed operator identity.

Line: unified geometric route/action/learning integration · count 7/3 · headline 437/512 → 437/512

References #2030, #2029 and #2032. This owner-directed source removal follows the existing pivot recorded by the [owner](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6111898445) and [current source claim](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6136085443). The inherited source count advances 6/3 → 7/3; nothing resets it. This is a static operator dependency, not another preparation PR naming a held run. No experiment, model training, model evaluation or reference forward executes here.

The shared-writer direction is recorded consistently: both model paths must directly learn/use the same native geometric vectors and writer, with E8/SpiralCore/icosian/FSB and flock/murmuration. 8D is the existing structural block; total width has no adopted numeric bar or MiniLM dependency. The writer and direct vector learning are not implemented by this PR. All training and M2 rental holds remain. The concurrently merged Hamilton-gate PR #2260 and Hamilton-projection PR #2262 with both complete records are preserved.

Decision: KEEP source identity repair; NOT YET PROMOTED as a model capability. M1 memory 20/40, reply 37/232 and M2 saved 437/512 plus opened 0/128 unchanged. Fresh qualification NOT_RUN. No full-project prohibition-removal, GPU parity, performance or energy claim. The no-softmax ratchet is a partial instrument and retains 1649 textual sites.

Removed generic source is retained in docs/history/branch-archive/native-vector-transport-2026-10-12/generic-contraction-removal.patch with SHA256 manifest and reverse-apply check. The patch's blank-line context markers are deliberately preserved verbatim with a file-specific whitespace attribute.

Validation at the exact PR head: scoped offline core/training library compile; 3 selected static algebra tests covering the canonical 64-action composition/digest, all 240 E8 roots and inverse transport, malformed actions and byte-negation overflow; touched-source rustfmt; no-softmax ratchet; claim wording; JSON/new-link/patch restore checks; harness rule check and PR lint. Commands and actual outcomes will be posted before queueing. Static geometry construction is non-model work; device/model checks remain NOT_RUN. Builds use an owned target, 2 jobs, incremental/debug info off. Sparse-checkout setup failures and final passes are retained in the closeout receipt archive.

- [x] Research record: docs/labs/native-vector-transport-2026-10-12/README.md
- [x] New top entry in docs/integration/current-state.md with Next; peer entry preserved
- [x] AGENTS, machine policy, current plan, README, STATUS, ROADMAP and harness count updated where changed
- [x] Claim-wording check passes
- [x] No model compute; no GPU/pod/rental; no FIT or hold release
- [x] Checks run at this exact head (commands and outcomes below; self-review posted before queue)
- [ ] Tracker #2028 source/count update after verified merge; existing historical results preserved
- [ ] Fresh-main source/tree verification, receipts iCloud round-trip, owned-only transport cleanup and preservation-scoped hygiene

Exact-head receipt: `cc74da4c74017b1f7f587b581f39f4eb777bb5cd`. All commands return 0.

```text
/cargo check --offline -p uor-r4-core --lib -> PASS (0.523 s)
/cargo check --offline -p uor-r4-training --lib -> PASS (17.373 s)
/cargo test --offline -p uor-r4-core --lib spiralcore_operator::finite_composition_table_tests -- --test-threads=1 -> PASS (0.548 s)
/rustfmt --edition 2021 --check crates/uor-r4-core/src/spiralcore_operator.rs crates/uor-r4-training/src/e8_memory.rs -> PASS (0.072 s)
python3 scripts/check_claim_wording.py -> PASS (0.817 s)
python3 scripts/check_no_softmax.py -> PASS (0.848 s)
git diff origin/main --check -> PASS (0.062 s)
git apply --reverse --check docs/history/branch-archive/native-vector-transport-2026-10-12/generic-contraction-removal.patch -> PASS (0.017 s)
python3 scripts/harness/cli.py check -> PASS (35.447 s)
```

Cargo uses the owned target, 2 jobs, incremental/debug info off and TMPDIR inside the owned local directory. Selected static tests: 3 passed / 0 failed; 18 new relative-link targets, policy JSON, retained patch SHA256/reverse apply, count7/3 and independent M2 holds pass. Harness: zero errors, warnings only for historical missing receipts, preserved peer branches and standing caps; PR lint zero errors and warnings.

Separate whole-path observation, not a clearance: #2262 eliminates the expanded real GEMM call but retains the dense block sum `y_j = sum_k W_jk Hamilton x_k`. The current no-hidden-contraction requirement still needs that structure and its full parameter access explicitly reviewed; preserving its source here does not qualify it or release training. No peer source was removed.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6136622197 — 2026-10-13T02:08:12Z — Casey-allard

Exact-head self-review (per current owner permission): cc74da4c74017b1f7f587b581f39f4eb777bb5cd.

Reviewed all 16 scoped paths against fresh main19500bf22, including merge of peer #2260/#2262. SignedMatrix8 constructors produce signed permutations; commutator/anticommutator outputs use a different wide type. Decoder validation gives exactly one +/-1 entry in each row and distinct columns. Composition selects right[left-axis] with sign equality; application selects the coordinate and checked negation. Canonical array/wire unchanged. Both E8 builders share this exact application, preserving catalogue/orbit ordering. Invalid deserialized axes and i8MIN negation return Result errors. No generic numeric contraction remains in these bodies.

Actual commands/outcomes at this exact head are in the PR body: offline core/training compile PASS, touched-source rustfmt PASS, 3 static algebra tests PASS (canonical group composition/digest, 240 roots/all64 actions/inverses, malformed/overflow), whitespace/claim wording/ratchet/patch reverse application PASS; harness0errors and PR lint0errors/0warnings. These selected fixtures construct static geometry only; no model/CPU reference forward, GPU, fit or generated result. New relative links/JSON/patch SHA256 and unchanged independent holds verified. CI harness succeeds; five compatibility statuses are transport acknowledgements, not Rust tests.

Removed source is indexed/restorable. Both complete peer source/record changes preserved. Shared native writer/direct learning/total width and full prohibition removal remain unqualified; no headline/reset/hold release. Source count6/3->7/3 is carried honestly under the owner pivot and explicit removal instruction. Additional source observation for PM: Hamilton-projection #2262 removes expanded GEMM but still iterates a dense sum over all learned input/output quaternion blocks. That must not be described as whole-path absence of hidden contractions. Its source is retained without clearance; exact signed-axis transport in this PR is a different finite-action operation.

Verdict: KEEP this bounded static operator/source dependency; no model capability promotion. Proceed through protected merge only.



---
_Comment/review ids in this file: 6136622197_

