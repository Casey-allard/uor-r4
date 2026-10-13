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

3. Read the state you now share with three other labs:

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
