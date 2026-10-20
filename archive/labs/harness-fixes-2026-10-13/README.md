# Harness fixes routed from the Claude lab — including the one blocking rung 0

**Date:** 2026-10-13 · **Lab:** DeepSeek · **Line:** no model line — tooling · **Count:** 0/3
**Owning work:** owner routing of Claude's list (#2028 comment 6149175296); D24, D27

## Item 1 — the rung 0 blocker: a CPU-reference parity test gated every pod

`scripts/pod/uor-pod-bootstrap.sh` ran `--test cuda_stack_ops_parity` on **every `uor-pod up`** and
refused the pod unless it passed. **That test checks CUDA kernels against CPU references, which D24
forbids** — model compute never runs on the CPU, and a host-side fallback is a defect to fix with a
kernel, not a cost to report. Gating the pod on it made every `up` depend on a prohibited execution.

**Fixed:** the gate is now **the build**. `BUILD.json` records `"parity": "NOT_RUN (D24)"` with the
reason in `parity_summary`, and `cache_ok`/`store_build_ok` accept it — **`PASS` stays accepted** so
caches published before this change remain valid rather than being silently rebuilt.

**The test is kept as source evidence.** Nothing was deleted:
`crates/uor-r4-training/tests/cuda_stack_ops_parity.rs` is untouched. Device-only controls remain
available with `up --device-controls <filter>` (Claude's optional suggestion; not implemented here).

## Item 3 — `multilab touch` delegated as a bare `touch`

The parser overwrote the command on every non-flag word, so a subcommand group lost its head:
`uor-harness multilab touch` set the command to `multilab` and then to `touch`, and delegated `touch`,
which Python rejected as `invalid choice: 'touch'`. **The first word is the command now; later ones are
its arguments.** Verified: `multilab touch` reaches Python with the group intact.

## Item 4 — `notes` read only the first 100 comments

On #2029 (231 comments) it reported an **old** note as "latest" and missed every recent one — the exact
failure the command exists to prevent, in the command itself. **`--paginate` fixed it**, measured:

```
#2029  single page -> 100 notes        #2029  --paginate -> 231 notes
```

The `at least` label is now tied to whether the read was actually complete: a paginated fetch states
the count as fact, while a `--cached` read or a failed fetch says `at least`, because it does not know
what it missed. **Reporting a floor as a fact is worse than reporting it as a floor.**

## Item 5 — stale advice in `onboard`

Its "Next:" lines told new labs to run `cli.py …` — the **old entry point**, after the port replaced it —
and said **"check in every 90 minutes"** when R3 and the live tier settled on **five**. Both fixed, and
the reminder of the wrong cadence is worth noting: `onboard` is the one command every new lab runs
first, so a stale line there propagates to every lab that joins.

`harness_answers` was also shelling out to **the Python harness** to ask whether the harness works, so a
broken Rust binary still reported ok. It calls the format checker in-process now.

**New check: `uor-harness` must be on `PATH`.** It is the interface, so a lab without it has no harness
— and every documented command would fail with `command not found` rather than a diagnosis.

## Item 6 — a stale binary cost a lab real time

Claude's machine had an old `uor-harness` and it failed with **`unknown option --line`** — an error
describing the SYMPTOM (a flag newer source accepts) and pointing nowhere near the CAUSE (the binary
predates the flag). Nothing knew which commit it had been built from, so nothing could say "you are
behind".

`build.rs` now bakes in the commit, the build time and whether the tree was dirty; `version` prints
them, and **`onboard` and `heartbeat` warn when the binary is provably behind `origin/main`** — naming
the rebuild command and the symptom.

**Only a proven-behind build warns.** No network, a shallow clone, or a branch ahead of `origin/main`
are all normal, and warning on them would train labs to ignore the warning that matters.

**The honest limitation, and it is the point of the item:** a binary too old to contain this check
cannot warn about itself. It only protects labs that rebuild once. That is why it is wired into
`heartbeat` — the one command every lab runs every five minutes.

## Verification

| check | result |
|---|---|
| `bash -n scripts/pod/uor-pod-bootstrap.sh` | syntax OK |
| `cargo fmt --check` | clean |
| `cargo clippy --all-targets -- -D warnings` | 0 findings |
| `cargo test -p uor-r4-harness` | **132 passed** (6 new) |
| `python3 -m unittest discover -s tests/harness -t .` | 319 passed |
| `uor-harness check` | **0 errors** |
| `notes` on #2029 | **231** (was "at least 100") |

## Limitations, stated

- **Items 2 and 6-of-Claude's-list are not done here.** The NVRTC enforcement work (`check_no_matmul` in
  prlint/CI, the semantic checker over CUDA raw-string sources, and a check that no `nvrtc` call
  remains after the module migration) stays with me and is **not started**; Claude builds the module
  split itself. Nothing here claims any of it.
- **`--device-controls` is not implemented.** Claude described it as optional; `up` has no such flag
  today, so a lab wanting device-only controls has no supported way to ask for them.
- **The parity test still exists and still cannot be run under D24.** Keeping it as evidence is right,
  but a lab that runs it anyway is executing the prohibited configuration — nothing prevents that.
- **`version` compares against the local `origin/main` ref**, which is only as fresh as the last
  `git fetch`. A stale ref makes a stale binary look current; a lab that has not fetched in a day gets
  a check that is a day out of date.
- **I removed the `parity` failure exit.** The bootstrap now exits non-zero only when no build was
  published, so a parity regression that once stopped a pod would no longer stop one — which is
  intended, but it does mean the bootstrap is a weaker signal than before.
