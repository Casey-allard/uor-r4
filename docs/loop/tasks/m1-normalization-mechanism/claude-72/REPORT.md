# REPORT: m1-normalization-mechanism, claude-72 (score the non-inferiority term)

Lab: claude. Item: [m1-normalization-mechanism](../../../../roadmap/items/m1-normalization-mechanism.md) (#2842). The item status is unchanged. Nothing is released and no compute is cleared.

## What did not work, first

1. **The sheet still does not quote the margin.** Casey's record names its authors (antigravity, deepseek) as the ones to quote `b − c > −F_I` in §3. I gave the sheet's NOT_FIT review (claude-57), so I left its text alone. Until they quote it, the code cites the owner record directly.
2. **This scores DEV only.** The fresh-draw replication term (`b − c ≥ ⌈2√(b+c)⌉`, no floor, Casey's ruling 2 on #2028 6379430093) needs a fresh-V2 scoring path, which does not exist yet. That is a separate unit.
3. **The term is scored, but nothing has been run.** No DEV run of the normalization arms has taken place (training is HELD), so there is no non-inferiority result, only the means to compute one.
4. **`cargo fmt -p uor-r4-discrete` also rewrote `tests/flock_coupling_controls.rs`,** which is unformatted on main. It is outside scope, so I restored it. A side finding for whoever owns that file.

## Cancel probe

At `dc5082f03`, `normalization_dev_paired` printed `"non_inferiority":"UNSCORED: no margin registered (claude-57 NOT_FIT prerequisite 3)"`, and no margin was computed. The task stands. Class: instrument.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-discrete/src/attention_preflight.rs` | **`DevPairedCount.non_inferior`:** `b − c > −F_I`, computed in `dev_paired_close` beside `beats` for both `all` and `P_succ`. |
| | **Report:** the `non_inferiority` field states the rule and its source (owner, 2026-10-24T03:41:41Z, #2028 6379430093), and the schema moves to `uor-r4.normalization-dev-paired/2`, so a saved `/1` report, which says `UNSCORED`, stays unambiguous. Nothing parses `/1`; only the producer and claude-67's REPORT mention it. |
| | **Test:** a new test, `dev_paired_non_inferiority_margin_is_strict`. |
| `crates/uor-r4-discrete/src/driver_source_pin.rs` | Re-pinned from `63b276f7…` to `826c7854f0d9a88e8a1203791aef7e177b3e011ab8f80f2dcbc6dba6ca9f2e49`, because `attention_preflight.rs` is one of the 19 driver sources. Recomputed independently with `cat` over the 19 files in `driver_source_sha256` order, piped to `shasum`. |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- **The margin, pinned on both sides:**

  | b + c | F_I | b − c | non-inferior |
  |---|---|---|---|
  | 24 | 24 (the floor) | −24 | false |
  | 23 | 24 (the floor) | −23 | true |
  | 200 | 29 | −30 | false |
  | 200 | 29 | −28 | true |
  | 196 | 28 (exact root, the margin itself) | −28 | false |

  A contrast that beats is non-inferior, and so is one that ties.
- **Negative gates.** Each was planted after the commit, the `dev_paired` tests were run, and the file was restored from the commit.
  - With the rule's line removed: 2 failed (the struct literal and the new test).
  - With the rule made non-strict (`>=`): 1 failed (the new test).
- **Suite.** `cargo test -q --locked -p uor-r4-discrete` passes on every target. The lib gives 242 passed, 0 failed, 1 ignored (240 before, plus 1 new test and 1 edited test). The bins and the integration tests all pass.
- **CUDA check.** The stub-nvcc `cargo check --locked --offline -p uor-r4-discrete --features cuda --all-targets` exits 0, so the `cuda`-gated report code compiles.

## Checks run at the head

`cargo test -p uor-r4-discrete` (every target), the stub-nvcc `cuda` check, `loop report --check`, `loop pr-check --local`, `git diff --check`.

## Skills used

- **`solution-integrity`:** the margin's source is cited, and both of its sides, including the exact-root edge, fail when the rule is planted wrong.
- **`scope-guard`:** the out-of-scope `fmt` change was restored, and the sheet was left to its authors.

## Still open

- The sheet authors quote the margin and the fresh-draw term in §3.
- A fresh-V2 scoring path for the replication term.
- The structural-norm and both-seeds parts of the bar, and every release, are unchanged and still pending.

## Plain summary

The test that compares two normalization settings could not yet say whether the new one is "no worse" than the plain one, because no margin had been chosen. Casey chose it this morning. The report now applies that margin, and tests check both sides of it exactly.
