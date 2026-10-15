# #2341 — Claude's six harness items, including the rung 0 blocker (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T15:54:08Z
- **Updated:** 2026-10-13T16:15:10Z
- **Closed:** 2026-10-13T16:14:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2341
- **Merged:** 2026-10-13T16:14:52Z
- **Merge commit:** ec3da4f988542915e8cce48811bfb6d7797fa2c2
- **Base / Head:** main / UOR-Foundation:deepseek/onboard-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Owner routed Claude's six harness items (#2028 comment 6149175296). Tooling only.

## Item 1 — the rung 0 blocker

`scripts/pod/uor-pod-bootstrap.sh` executed `--test cuda_stack_ops_parity` on **every `uor-pod up`** and refused the pod unless it passed. **That test checks CUDA kernels against CPU references, which D24 forbids** — so every `up` depended on a prohibited execution.

**Fixed:** the gate is **the build**. `BUILD.json` records `"parity": "NOT_RUN (D24)"` with the reason in `parity_summary`; `cache_ok`/`store_build_ok` accept it; and **`PASS` stays accepted** so caches published before this change remain valid rather than being silently rebuilt.

**The test is kept as source evidence** — `crates/uor-r4-training/tests/cuda_stack_ops_parity.rs` is untouched.

## Item 3 — `multilab touch` delegated as a bare `touch`

The parser overwrote the command on every non-flag word, so the subcommand group lost its head: `multilab` then `touch`, delegating `touch`, which Python rejected as `invalid choice`. **The first word is the command now; later ones are its arguments.**

## Item 4 — `notes` read only the first 100 comments

On #2029 (231 comments) it named an **old** note "latest" and missed every recent one — **the exact failure the command exists to prevent, in the command itself.** Measured fix:

```
#2029 single page -> 100 notes        #2029 --paginate -> 231 notes
```

The `at least` label is now tied to whether the read was **complete**: a paginated fetch states the count as fact; a `--cached` read or a failed fetch says `at least`. **Reporting a floor as a fact is worse than reporting it as a floor.**

## Item 5 — stale advice in `onboard`

Its "Next:" lines pointed new labs at `cli.py` — the **old entry point**, after the port replaced it — and gave the cadence as ninety minutes when the live tier settled on five. `onboard` is the one command every new lab runs first, so a stale line there propagates to every lab that joins.

`harness_answers` was also shelling out to **the Python harness** to ask whether the harness works, so a broken Rust binary still reported ok. It checks in-process now.

**New check: `uor-harness` must be on `PATH`** — it is the interface, so a lab without it has no harness and every documented command fails with `command not found` rather than a diagnosis.

## Item 6 — a stale binary cost a lab real time

Claude's old `uor-harness` failed with **`unknown option --line`**: the **symptom**, pointing nowhere near the **cause**. `build.rs` now bakes in the commit, build time and dirty state; `version` prints them; and `onboard`/`heartbeat` warn when the binary is **provably behind `origin/main`**, naming the rebuild and the symptom.

**Only a proven-behind build warns.** No network, a shallow clone, and a branch ahead of `origin/main` are all normal, and warning on them would train labs to ignore the warning that matters.

**The honest limit, and it is the point of item 6: a binary too old to contain this check cannot warn about itself.** It protects only labs that rebuild once — which is why it rides `heartbeat`, the command every lab invokes every five minutes.

```
bash -n scripts/pod/uor-pod-bootstrap.sh        -> syntax OK
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 132 passed (6 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
uor-harness check                               -> 0 errors
notes on #2029                                  -> 231 (was "at least 100")
```

Record: `docs/labs/harness-fixes-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **Items 2 of Claude's list is NOT started and not claimed.** The NVRTC enforcement work (`check_no_matmul` in prlint/CI, the semantic checker over CUDA raw-string sources, and a check that no `nvrtc` call remains after the module migration) stays with me. Claude builds the module split itself.
- **`--device-controls` is not implemented.** Claude described it as optional; `up` has no such flag, so a lab wanting device-only controls has no supported way to ask.
- **The parity test still exists and is still prohibited under D24.** Keeping it as evidence is right, but **nothing stops a lab from invoking it anyway** and executing the forbidden configuration.
- **`version` compares against the local `origin/main` ref**, only as fresh as the last `git fetch`. A lab that has not fetched in a day gets a check that is a day out of date.
- **The bootstrap no longer exits non-zero on a parity failure** — only when no build was published. Intended, but it means the bootstrap is a weaker signal than before.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

