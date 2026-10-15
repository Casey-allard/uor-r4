# A claim dies with its branch: `clean-merged --apply` releases the stale-claim class

Date: 2026-10-15. Lab: deepseek. Line: no model line - tooling. Task source: Kimi PM follow-up (2 of 2),
[#2028 6195505734](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6195505734). No model
compute, no GPU, no pod, no headline change.

## What was wrong

**A claim outlives the branch it names.** A lab claims a branch, opens the PR, the PR merges, the branch is
deleted - and the entry stays in `docs/harness/claims.json` until its owner notices. Measured while this was
written: `deepseek/harness-write-path` merged as `b72f904c4` and its branch was deleted, while the claim stayed
in main's store and woke `multilab.claim_not_merged` on every `check`.

The cost is not the warning. It is the next lab that wants that branch name: `team claim` refuses with
`already held by lab ...` for work that is already in main, and the collision has to be untangled by hand - a
conflict in the FILE rather than in the work. The sweep-record agent hit the same class on this queue, and its
own `team claim` released the stale entry.

## What changed

| file | change |
|---|---|
| `crates/uor-r4-harness/src/team.rs` | `stale_claims` (read-only preview) and `release_merged_claims` (the write): releases every claim whose branch is in the caller's PROVEN-MERGED list, in the caller's own committed store, and republishes the live mirror so the release cannot be resurrected by a stale mirror |
| `crates/uor-r4-harness/src/main.rs` | `git clean-merged` reports the stale claims in its read-only run, and `--apply` releases them alongside the branch deletion it already performs. A refusal (R10 in the shared checkout, or an unwritable store) is reported, not fatal: the branch deletion it accompanies stands |
| `AGENTS.md`, `docs/harness/README.md`, `docs/harness/pm-rules.md` | the post-merge sequence and the claims section state the behaviour, so a lab reading the rule knows the claim goes with the branch |

**Why this place.** `clean-merged` already establishes mergedness by **content** - the merged PR's merge-commit
tree against the branch tree - because a squash merge leaves no ancestry link and `merge-base --is-ancestor`
would call merged work unmerged. That is exactly the proof a release needs, so the post-merge step every lab
already runs is where the claim goes too. Nothing is guessed: a branch that is not in the proven list keeps its
claim, and an `Unknown` state keeps it as well - "cannot prove it" is not "dead".

## Verification

- `cargo test -p uor-r4-harness --offline`: **534 unit tests + 1 new integration test** (`tests/merged_claim_release.rs`)
  pass; `fmt --check` and `clippy --all-targets` clean.
- **Each half is pinned by a test that fails when it is removed:**

  | fix removed | test that fails |
  |---|---|
  | the release call in `clean-merged --apply` | `a_merged_branchs_claim_is_released_and_an_open_branchs_is_kept` (CLI) |
  | `save_claims` inside the release | `a_claim_on_a_merged_branch_is_released_and_the_others_are_kept` (library) |
  | the live-mirror republish | the same library test, which seeds the mirror first - without the republish the mirror resurrects the released claim on the next read |

- The integration fixture is the real shape: main pushed to a bare origin, one branch **squash-merged** into it,
  one branch left open, and two claims in the store. The report names one stale claim and writes nothing;
  `--apply` deletes the merged branch, releases its claim from the checkout **and** the live mirror, and leaves
  the open branch's claim alone.

## Review response (Kimi PM, exact head `a98b2aa0d`)

**R1 — the release deleted other labs' LIVE-only claims.** `publish_claims_live` was handed the caller's
committed store, and the live mirror IS the only home of a peer's uncommitted claim (the tier's whole
purpose: visible before its PR lands). Measured by the reviewer: `peer/live-only` vanished from the mirror
and from `multilab claims`. **Fixed:** the mirror is now republished from the **merged view minus the
released branches** — foreign live entries survive, and the released branches are removed explicitly
because `origin/main` still names them until this PR lands. Pinned by seeding the mirror with a
`peer/live-only` claim and asserting it survives the release *and* is not imported into the committed store.

**R2 — the motivating case was not covered, and the docs implied it was.** `branch_states` enumerates
`refs/heads`, so a claim whose branch is already deleted (the case this record opens with) is invisible to
the release. **Fixed by making the residue visible instead of guessing at it:** `clean-merged` now REPORTS
`claims whose branch is gone` — branches absent both locally and on `origin`, so an open PR is never named —
each with the exact `uor-harness team release --branch <branch> --lab <lab>` line that clears it. Nothing is
released on this signal: **"gone" is not "merged"**, and after a squash merge there is neither a tree nor an
ancestry link to prove it with. The README, `pm-rules` and `AGENTS.md` now state the mechanism's real reach,
and `remote_branches` returns `None` (report nothing) rather than an empty list when the remote cannot be
read, so an offline checkout cannot turn every peer claim into a residue. Pinned in the integration test:
`ghost/gone` is reported with its fix line and is still in the store after `--apply`.

## What this does not do

- It does not release a claim at `team claim` time. A lab that wants a name held by a *merged* branch still meets
  the refusal until the post-merge step runs - deliberately: releasing another lab's claim inside a claim would
  hide the collision rather than clear it once, in the step whose job is exactly that.
- It does not touch a claim whose branch state is `Unknown` or `Unmerged`. An open PR keeps its claim.
- It does not release a claim whose branch is gone: it reports it (R2 above). The residue needs a human
  because the proof does not exist, not because the command is shy.
- It does not fix the same read-view-as-write-source pattern in `claim_branch`/`release_branch`
  (`team.rs` publishes the committed store there too). The reviewer recorded it as a follow-up; the fix is
  the same one-liner and it is worth its own PR rather than riding this one.
- It does not change the rule that reports the class (`multilab.claim_not_merged`); the rule stays the detector,
  and this makes the repair mechanical.

## Cost

Laptop CPU only: source edits, one build, three test runs and three guard-removal probes. No pod, no GPU, no new
dependency, no format change - the claims store keeps its shape.