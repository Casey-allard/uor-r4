# Harness bootstrap — paste this into one lab session

**Purpose:** prove the multi-lab harness works in a session that did not write it. One lab, one
session. Nothing here runs model compute, and nothing here is a deliverable — it is a smoke test of
the coordination loop with a real foreign client.

**Why test it with a foreign lab at all:** every test so far is the harness exercising itself. On
12 October two smoke-test records written with `UOR_LAB=kimi` and `UOR_LAB=claude` reached the
committed record and were indistinguishable from two labs genuinely checking in. The failure mode is
silent, so the only real test is a lab that has never touched this code.

---

## The prompt

Copy everything below into the target lab's session.

```text
You are running a 10-minute bootstrap check of the research harness. This runs no model compute
and changes no research state. Report exactly what you observe, including anything that fails or
reads as confusing - a confusing result is the finding.

1. Identify yourself for the session (this is the one required step):

     export UOR_LAB=<kimi|claude|codex|deepseek>

   Without it, your receipts are recorded but attributed to nobody, and the harness will print a
   warning saying so. Tell me if you see that warning.

2. Confirm the tool runs on your interpreter. Report the version and the exit code:

     python3 -V
     python3 scripts/harness/cli.py --repo "$(git rev-parse --show-toplevel)" check --only arena

3. Read the state you now share with the other labs:

     python3 scripts/harness/cli.py --repo "$(git rev-parse --show-toplevel)" multilab presence
     python3 scripts/harness/cli.py --repo "$(git rev-parse --show-toplevel)" multilab claims

   Report: which labs appear, which tier answered ("via"), and whether the board is empty. An empty
   board is the EXPECTED result - no lab has checked in yet. Say so rather than reporting a fault.

4. Claim a scratch branch and check it is visible to peers (this is the point of the exercise):

     git checkout -b bootstrap/<your-lab>
     python3 scripts/harness/cli.py team claim --branch bootstrap/<your-lab> --lab "$UOR_LAB"
     python3 scripts/harness/cli.py multilab claims

   Report the claim line exactly as printed. Then release it and delete the branch:

     python3 scripts/harness/cli.py team release --branch bootstrap/<your-lab> --lab "$UOR_LAB"
     git checkout main && git branch -D bootstrap/<your-lab>

5. Check in once, so the other labs can see you:

     python3 scripts/harness/cli.py heartbeat -m <your milestone> --next "<your next step>"

   Then run `multilab presence` again and confirm YOUR lab appears, with `via live`.

6. Do NOT commit anything for this test. Do not open a PR. Leave the tree as you found it apart
   from the receipts, which are expected.

REPORT BACK, in this order:
  a. your interpreter version and whether every command above exited 0
  b. the exact output of `multilab presence` at step 5
  c. anything that was confusing, mislabelled, or looked wrong
  d. whether you would want this tool in your normal session, and what would have to change first
```

---

## What to look for in the report

| observation | what it means |
|---|---|
| a command exits non-zero | the bootstrap is not portable to that runtime; the exact error identifies the fix |
| `multilab presence` shows no lab after step 5 | the two-tier write is not reaching the shared directory; check `$UOR_HARNESS_LIVE` |
| `via live` is absent, `via durable` appears | the live tier is unwritable on that runtime, which is survivable but worth knowing |
| the identity warning appeared | `UOR_LAB` was not exported in the shell the command ran in — a real usability finding |
| any record appears for a lab that did not run this | **stop and tell me**; that is the misattribution failure from 12 October recurring |
| the lab finds step 3's empty board confusing | the board should say "no lab has published presence yet" — if it does not, that is a wording defect |

## Success

One foreign lab, one session: claims visible to peers, presence published and visible, the tool
running on that lab's own interpreter, and no record anywhere attributed to a lab that did not run
it. That is the whole test. Anything else is a finding.

---

# Installing the harness on your lab

This is the per-lab install. **One command for the required part; two optional commands for the
conveniences.** Nothing here needs admin, a package manager, or a network fetch beyond the repo you
already have.

## Every lab: identity (required)

```sh
export UOR_LAB=<kimi|claude|codex|deepseek>
```

Put it where your runtime sets its environment so it is present in every session — a shell rc, the
runtime's own config, or your lab's startup file. **Without it your receipts are recorded but
attributed to nobody**, and no other lab can see who did the work. The harness prints a loud warning
rather than letting that pass silently, because an unattributed record is worse than an absent one.

Verify:

```sh
python3 scripts/harness/cli.py --repo <repo> heartbeat -m m1 --print
```

`--print` shows the card without recording a receipt. If you see the `NO LAB IDENTITY` warning, the
export did not reach that shell.

## Optional: the liveness hook (Kimi only, so far)

Kimi is the only runtime on this machine with a per-prompt hook, so it is the only lab that can
appear live without anyone typing anything. If your runtime has an equivalent, this is the shape:

```
UserPromptSubmit  ->  python3 <repo>/scripts/harness/cli.py multilab touch --lab <your-lab>
```

`touch` refreshes your timestamp, **preserves** the intent your last `heartbeat` published, and
writes only the uncommitted live tier — so a prompt never dirties the repository. Measured at 0.061s
against a 5s hook timeout.

Kimi's is installed at `~/.kimi-code/bin/uor-context-hook`, registered in `~/.kimi-code/config.toml`,
backed up at `~/.kimi-code/bin/uor-context-hook.bak-20261012`. If your lab has no hook, your presence
is per-session and that is expected — check in with `heartbeat` and you are visible.

## Optional: the pre-commit branch check

Refuses a commit on a branch another lab has claimed:

```sh
python3 scripts/harness/cli.py --repo <repo> git install-hooks
```

## Verify the install

```sh
python3 scripts/harness/cli.py --repo <repo> check --only arena     # the tool runs here
python3 scripts/harness/cli.py --repo <repo> multilab presence      # you can see the other labs
python3 scripts/harness/cli.py --repo <repo> heartbeat -m <milestone>
python3 scripts/harness/cli.py --repo <repo> multilab presence      # you now appear "via live"
```

Success is: the tool runs on **your** interpreter, you can see the other labs, and after your
heartbeat you appear on the board yourself. Then read
[BOOTSTRAP.md](BOOTSTRAP.md) above and run it once.

## Python version

Declared floor **3.9**. Verified on 3.9.6 and 3.15.0. Note `python3` differs by shell on this
machine — `/etc/zprofile` reorders PATH, so a login shell gets Homebrew's and an agent shell may get
`/usr/bin`. Both work; if you hit a version error, the tool names the requirement rather than
failing obscurely.

## Where the shared state lives

| tier | path | what it is |
|---|---|---|
| **live** | `~/.local/share/uor-r4/harness/` (`$UOR_HARNESS_LIVE`) | what peers see now. Uncommitted, cleared routinely — treat as advisory, never as history |
| **durable** | `docs/harness/` in the repo | the record. Survives restarts and reaches `git` |

Readers prefer live and fall back to durable, and always say which answered (`via live` /
`via durable`).

---

# For the project manager: where decisions are kept

Kimi is the project manager, so its decisions have to be **durable and findable by the other labs**,
not held in a session. Four places, in order of authority:

1. **`docs/integration/DECISIONS.md`** — the canonical decision register (`D1`…`D27`). A decision
   that binds the project belongs here, by its own number.
2. **The milestone issue on GitHub** — the working authority the labs actually read each session.
   `#2028` is the tracker; `#2029`–`#2036` are the milestones.
3. **`docs/integration/current-state.md`** — what changed, newest first, with a **Next:** line.
4. **`docs/labs/<topic>-<date>/README.md`** — the dated record for a unit of work.

A decision filed *only* in a chat session is not findable by the other labs, and it will be
re-litigated. If it binds anyone else, it goes in one of the four above **and** is cited from the PR
that acts on it.

The reverse direction — reaching a lab directly — is file-based, because there is no socket or
daemon and the retired `lab-runner coord` must not be recreated:

- **Kimi reads** `~/.kimi-code/UOR-R4-HANDOFF.md` at session start (a stale navigation aid; the repo
  is authoritative) and `~/.kimi-code/UOR-R4-START.md` when the owner runs the `/goal` command.
- **This lab publishes** those files, so a request can be queued for Kimi's next session.
- Backups are kept beside each (`*.bak-*`) and one `cp` reverts them.

**Test-result convention for the PM.** A lab's claims about its own results are not evidence. The
receipts in `docs/harness/ledger/<lab>-<month>.jsonl` and the merged PR are the record, and
`multilab presence` says who is live. When a milestone turns on a number, read the number in the
artifact rather than the summary in the message.
