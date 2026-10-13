# The multi-lab harness: presence, committed claims, and a reconciliation gate

**Question.** The harness was built for one lab. Four labs use this repository (Claude, Codex,
Kimi, DeepSeek), with Kimi as project manager and Claude/Sol carrying research and synthesis.
What actually breaks when four separate vendor CLIs share one coordination layer, and what can a
harness enforce without inventing a runtime that does not exist?

**Line:** no model line — tooling. · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).
No model compute was run; the harness is tooling and cannot move a headline number.

## What reconnaissance established

Two read-only subagents were run before any code was written. Their findings shaped the design and
are worth recording, including the one that removed an assumption:

**"The mesh" does not exist as a coordination concept in this repository.** A tracked search for
`mesh` returns 100 hits across 16 files, and every one is geometric: nearest-neighbour meshes in
the SpiralCore schema, a Delaunay-triangulator disclaimer, `THREE.Mesh` in research simulations.
Two further hits are unrelated *service*-mesh entries in a vendored source catalog. There is no
coordination mesh, no Tailscale or WireGuard mesh, and — importantly — the retired
`lab-runner coord` mechanism is explicitly not to be recreated
([protocol.md](../../labs/protocol.md): *"do not recreate it or run `lab-runner coord`"*).

So "connect to the mesh" resolves to: **GitHub is the bus, `main` is the truth, and the harness
adds what GitHub alone does not give** — a committed record of who holds what and what each lab is
doing between merges. Anything daemon-shaped was ruled out on policy grounds, not just effort.

**The portability constraint is interpreter discovery, not Python version.** On this machine
`python3` is **3.9.6 in a login shell** and **3.15.0 in a non-login agent shell**, because
`/etc/zprofile` → `path_helper` puts `/usr/bin` first while an agent's PATH puts
`/opt/homebrew/bin` first. Every interpreter present (3.9.6 through 3.15.0, including four
uv-managed ones) runs the tool successfully. The declared floor is 3.9 — set by
`Path.unlink(missing_ok=True)` at 3.8, rounded up because 3.8 is end-of-life — so nothing needed
raising and no feature had to be dropped.

## Three failures at four labs, and the fixes

**1. Claims were invisible.** They lived in a gitignored `.harness/claims.json`, one per worktree,
so lab B could not see that lab A held a branch. That is right for one checkout and useless for
four: a claim nobody can read cannot prevent the collision it exists to prevent. Claims now live in
the committed [`docs/harness/claims.json`](../../harness/claims.json), a claim untouched for half a
day stops blocking, and only its holder can release it.

**2. Nothing said what a lab was doing.** Receipts only exist *after* a PR merges, so "what is lab X
doing right now" had no answer anywhere. `heartbeat` now publishes
[`docs/harness/presence/<lab>.json`](../../harness/presence/README.md) as well as the receipt —
deliberately in that command, because the session contract already requires a 90-minute card, so a
lab that follows the contract becomes visible for free and one that never checks in is *visibly
absent* rather than silently unknown. Stale after 30 minutes; an unparseable timestamp reads STALE,
never live.

**3. One ledger, every lab appending.** `ledger/<month>.jsonl` conflicted on every concurrent PR.
The author resolved that union by hand twice in a single session, which is the clearest possible
evidence it does not scale. Receipts are now sharded per lab per month.
`multilab migrate-ledger` moved the existing file across without losing a line — an unreadable line
is quarantined in its own shard, never dropped.

## Enforcement, not a dashboard

A dashboard nobody opens enforces nothing, so the reconciliation went where the action already is.
`.github/workflows/harness.yml` now compares the milestone registry against the **live** issues
using the workflow token and fails the PR when they disagree:

```
$ python3 scripts/harness/cli.py check --only status.registry_vs_github
harness check: 6 rules, all pass
```

It reads GitHub and writes nothing. Today it passes, so the gate is real rather than aspirational.
This is what makes `main` converge regardless of which lab opened the pull request.

## A real performance defect, found by measuring

`mistakes.stale_check` walked the whole tree with `rglob` and then ran three `fnmatch` calls per
(path, pattern) pair for every lesson. With `research/` at 11,171 files that was **46.5 seconds on
Python 3.9** — and it dominated the entire `check`, which is the command the contract asks labs to
run. It now uses `git ls-files`, the index that already exists, and resolves literal paths before
attempting any glob:

```
mistakes.stale_check on Python 3.9:   46.5s -> 0.43s
full suite on Python 3.9:             205s  -> 20s
```

## Evidence

```
python3 -m unittest discover -s tests/harness -t .            -> 262 tests, OK
/usr/bin/python3 -m unittest discover -s tests/harness -t .   -> 262 tests, OK   (Python 3.9.6)
python3 scripts/harness/cli.py check                          -> 43 rules, 0 errors
bootstrap from a foreign cwd, --repo before the subcommand    -> works
cli.py multilab presence                                      -> prints per-lab state, stale last
```

16 new tests cover the properties that matter at four labs: presence is committed rather than
hidden in state; a stale presence never reads live; an unparseable timestamp is stale; another lab
is refused a branch it does not hold; a stale claim does not block forever; only the holder can
release; and migration preserves every receipt including an unreadable one.

## Limitations, stated rather than hidden

- **This is not real-time and cannot be.** There is no socket, no daemon, no push channel. Threads,
  sessions and context are private per vendor by design. A lab that never runs the harness is simply
  absent — and absence is not evidence of idleness.
- **Claims report; they do not lock.** A filesystem cannot stop another vendor's CLI from checking
  out a branch. What enforces the one-checkout policy is `status.branch_claimed` on every `check`,
  the pre-commit hook (opt-in), and the policy itself.
- **Presence is only as good as the check-in.** A lab that works past 30 minutes without a
  heartbeat shows STALE, which is the correct signal but not a live one.
- 11 pre-existing receipts are attributed to nobody (`unknown-<month>.jsonl`). They are preserved,
  not deleted; nothing can recover which lab wrote them.
- Two orphan branches remain: `bpb-split` holds 375 lines not in `main` and belongs to another lab,
  and `deepseek/invariant-router` is checked out in an active worktree.
- The recon subagents reported the checkout as dirty and HEAD-asymmetric while this work was in
  progress. That was this session's own uncommitted change, not corruption; the tree is clean at the
  merge commit.

**Next:** the labs adopt `export UOR_LAB=<lab>` at session start so their receipts and presence are
attributed; `docs/labs/session-goal.md` should name `multilab presence` alongside `team claim`; and
a non-Claude lab needs a scheduled entry point before presence can be continuous rather than
per-session.

## Correction: another lab HAS used the harness

An earlier revision of this record, and two reports to the owner, said no other lab had ever run
this. **That was wrong.** Tracing the committed receipts shows otherwise:

| receipt | branch claimed | outcome |
|---|---|---|
| `2026-10-12T23:15:06` `claim` lab=kimi | `kimi/rung0-prep-2026-10-13` | PR **#2272** merged |
| `2026-10-12T23:44:17` `claim` lab=kimi | `kimi/d27-rust-first-2026-10-13` | PR **#2278** merged |
| `2026-10-12T22:10:42` `skill` x2 lab=kimi | — | the no-matmul interlock record (PR #2265) |
| `2026-10-12T22:44:16` `skill` x2 lab=kimi | — | the D25 hygiene docs sweep (PR #2268) |

The Kimi lab claimed two branches through `team claim` before pushing them, and both PRs merged.
Those branch names are Kimi's own work; they are not strings this author produced. So the claim
path, the sharded ledger and the heartbeat path have all been exercised by a lab that did not
write them.

**What that does and does not establish.** It establishes that a foreign lab can run the tool, claim
a branch, and have the record land in the sharded ledger — which is the adoption question this
record was written to answer. It does **not** establish that presence was seen by a peer while Kimi
was live, because the presence files for that period were the fabricated ones removed in #2275, and
the live tier holds nothing now.

**A real gap this exposed.** Claims are written to the committed store, but the claim receipts above
exist while `docs/harness/claims.json` is empty. So a lab can claim a branch through
`team claim --lab`, have the receipt recorded, and still leave no claim in the shared store — which
means the next lab cannot see that the branch was ever held. That defeats the purpose of moving
claims into git, and it is not fixed by this record.
