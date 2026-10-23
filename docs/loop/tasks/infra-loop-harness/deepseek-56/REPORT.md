# REPORT: infra-loop-harness, deepseek-56 (banned-presence pr-check rule)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904. Base: `3df3798e4`. `References #2904`.

## What did not work, first

- **The first design mapped items and was broken by review twice.** Version 1 followed each base `#[cfg(any())]` item to its head counterpart; an independent subagent review found const/macro items missed, rename/re-header bypass, duplicate-name false positives, parent-gate false failures, D32 literal/scope errors, rope/gemm substring false positives, char-literal brace corruption, and a block comment between gate and item skipped. Version 2 counted compiled sites per file but was still comment-blind (`#[cfg(any())] // note`), did not follow out-of-line `mod x;`, missed `pub(crate)`/`use` items, corrupted spans on raw strings, missed `__half` and float-literal forms, over-counted D25 helper names, and scanned quadratically.
- **Claude took two of those fixes on the branch** (gate match on the comment-stripped line; following gated out-of-line modules) and added three tests. A second subagent implementation then widened the rule to revision-level gated-file sets and the structure view; the two implementations were merged by adopting the structure-view implementation and keeping Claude's fixes and tests.
- **Two of Claude's tests had a fixture that is not a ratchet site:** `D25_FN` was `pub fn dense_matmul() {}`, a bare identifier the D25 ratchet does not count, so the expected 0 -> 1 rise could only pass under the old over-counting scanner. The fixture is now a real `.matmul(` call; the tests then fail/pass for the right reason.

## What changed

| File | Change |
|---|---|---|
| `crates/uor-r4-harness/src/loopprcheck.rs` | `banned-presence` rule: `strip_structure` (comments, strings, char literals, raw strings blanked) for gate/item structure; `gate_kind` outer/inner gates; `item_prefix` parser covering visibility groups (`pub(crate)`/`pub(in ...)`), modifiers, `use`, `extern crate`, `extern "C"`, macros; `item_start_indices` + binary search; `inner_gate_end`; `gated_files`/`gated_by_ancestor`/`mark_gated_tree`/`direct_mods` following gated out-of-line modules transitively; ratchet-faithful D23/D25/D32 site counters (D32 types including `__half`/`__nv_bfloat16`, decimal/exponent/suffix/hex floats); per-file base-vs-head rise comparison with deduplicated details |
| `crates/uor-r4-harness/tests/loop_prcheck.rs` | Claude's three tests (comment gate, gated out-of-line module to `cfg(test)`, module removal) plus regressions: un-gated banned item, `cfg(test)` move, rename, benign/`properties`/char-literal, deletion, parent gate, D32 float/`__half`, `pub(crate)`, inner gate, raw-string brace, D25 helper names; 45 tests |
| `docs/harness/README.md` | the pr-check rules list documents the compiled-site delta, gate forms, module following, `cfg(test)` not exempt, and comments/strings counted out |
| this folder | ORDER and REPORT |

## Measurements

- **Real heads:** #3079 `1ddfa1cb`/`54de128d` -> `FAIL banned-presence 1 changed file(s)`, `relation_compiler.rs: D23 sites in compiled code rise from 0 to 19`; re-gated `8f36239a` -> PASS.
- **Tests:** `cargo test -p uor-r4-harness --test loop_prcheck` -> 45 passed. fmt clean; clippy `-D warnings` clean; ratchets d23/d25/d32 at baseline (4451/93/31); full harness suite green (CI).
- **Full harness suite:** every group green except one pre-existing timing-flaky loop-drive test (`a_productive_then_silent_client_is_closed_at_the_second_window`, unrelated to this diff); CI's harness job is the authority.
- **Reviewer cases now pinned:** comment gate, out-of-line module moves both directions, `pub(crate)`/use items, inner gates, raw-string braces, `__half`, D25 helper names.

## Checks run at the head

`cargo test -p uor-r4-harness --test loop_prcheck`, full harness suite, fmt, clippy `-D warnings`, `uor-harness ratchet d23/d25/d32`, real-head pr-check runs, `loop report --check`, `loop pr-check --local`; recorded review evidence includes two independent subagent reviews (the owner directed using subagents instead of waiting on other labs) and Claude's assist commit `930e74ce7`.

## Skills used

- `solution-integrity`: two designs were disproved by independent review and replaced; each reviewer case became a regression test; the real PR heads are the planted fault.
- `scope-guard`: shared registration files only where the rule and docs must live; the test file inside item scope; no rule change, no owner path.

## Still open

- The rule reads source; macro-expanded code and gates created by macros stay outside it. D32 counts the literal forms the ratchet sees, not every possible float spelling. Other labs' counted review remains the normal path; this PR carries independent subagent reviews under the owner's 2026-10-23 direction.

## Plain summary

- A diff-level pr-check rule now fails any PR that compiles more D23/D25/D32-banned code than its base: it understands outer/inner/comment/inline/module gates, follows gated out-of-line modules, ignores comments/strings/raw-string structure, and matches the ratchets' site grammar; proven against the real #3079 violating and repaired heads with 45 passing tests.