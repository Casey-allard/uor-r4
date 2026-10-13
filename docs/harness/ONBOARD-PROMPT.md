# Onboarding prompt — paste this into each lab

**Who this is for:** Claude, Codex/Sol, and Kimi. One prompt each; the only line that changes is the
identity.

**What it does:** verifies that your session can use the harness, then has you do one small piece of
real work with it. Ten minutes. **No model compute, no GPU, no training.** Nothing here is a research
result.

**One thing does write:** `prep new` creates a sheet under `docs/harness/prep/`. That is the point of
step 4 — it is the only way to find out whether the D22 gate works for a lab that did not write it —
and step 8 says to remove it afterwards if you are not going to use it.

**Why it matters:** the harness is how four labs share one repository without colliding — it records
who holds which branch, what each lab is doing, and what actually ran. Kimi and Codex have already
used it for merged work. What has **not** been exercised by a foreign lab is `prep new` (the D22
pre-training gate) and `team dispatch`. Those are the gaps this closes.

---

## The prompt

Replace `<LAB>` with `claude`, `codex`, or `kimi` — once, at the top. Everything else is literal.

```text
You are onboarding onto the uor-r4 research harness. This is ~10 minutes, runs no model
compute, and changes no research state. Report exactly what you observe, including anything
that fails or reads as confusing — a confusing result is the finding.

1. Identify yourself. This is the one required step; without it your work is recorded but
   attributed to nobody, and the harness will say so.

     export UOR_LAB=<LAB>

2. Verify your machine, in Rust. The build is seconds — the crate depends only on serde.

     cd <repo>
     CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness
     UOR_LAB=<LAB> ./local/harness-target/debug/uor-harness onboard --repo "$PWD"

   Report the full output. It exits 0 when ready and 3 when something FAILs.
   If you have no Rust toolchain, say so and use the Python fallback instead — the harness
   itself is Python, so you are not blocked:

     export UOR_LAB=<LAB>
     uor-harness check --only arena
     uor-harness multilab presence
     uor-harness multilab claims

3. Look at the state you now share with the other labs:

     uor-harness multilab claims
     uor-harness multilab presence

   Report which branches are claimed, by whom, and whether any lab reads STALE. Stale is not
   idle — it means that lab is working without checking in.

4. EXERCISE `prep new` — this is the part nobody has run yet, and it matters most. It is the
   D22 gate: the pre-registration sheet that must reach `TEST FITNESS: FIT` on a milestone
   before any compute. Pick a milestone you are actually working toward and generate the
   sheet, but DO NOT fill it in and DO NOT run compute:

     uor-harness prep new -m <milestone>

   It writes `docs/harness/prep/<milestone>-<mechanism>-<date>.md`. Report: did it produce a
   sheet, is the output clear, and is it obvious what you would have to fill in before the
   milestone could be considered FIT? A lab that has never used it should be able to tell.
   If you are not going to keep working the sheet, delete it before you finish - step 8.

5. EXERCISE `team dispatch` — the other path with no foreign receipts:

     uor-harness team roster
     uor-harness team dispatch <a role you would actually use>

   Report whether the role you wanted existed and whether dispatching it recorded what you
   expected.

6. Claim a scratch branch and confirm peers can see it, then release it:

     git checkout -b onboard/<LAB>
     uor-harness team claim --branch onboard/<LAB> --lab "$UOR_LAB"
     uor-harness multilab claims
     uor-harness team release --branch onboard/<LAB> --lab "$UOR_LAB"
     git checkout main && git branch -D onboard/<LAB>

7. Check in once, so the others can see you are alive:

     uor-harness heartbeat -m <your milestone>

   Then run `multilab presence` again and confirm your lab appears.

8. Do NOT commit anything for this exercise and do not open a PR. The receipts from steps 4-7
   and the prep sheet from step 4 are the only things that should change. Delete the sheet if
   you are not going to use it, and leave the tree otherwise as you found it.

REPORT BACK, in this order:
  a. your interpreter version, and whether every command exited 0
  b. the exact `onboard` output (or the Python fallback output)
  c. step 4: what `prep new` produced and whether it was clear what to fill in
  d. step 5: whether `team dispatch` did what you expected
  e. the `multilab presence` output after step 7
  f. anything confusing, mislabelled, or wrong
  g. whether you want this in your normal session, and what would have to change first
```

---

## What to look for in the reports

| observation | what it means |
|---|---|
| `onboard` exits 3 | read the FAIL line; `identity` means the export did not reach that shell |
| `prep new` produces nothing or is unclear | a real defect in the D22 gate — the highest-value finding available here |
| `team dispatch` has no role you would use | the expert team registry is missing something; report which |
| a command exits non-zero | the exact error identifies the fix; report it verbatim |
| a lab appears in `presence` that did not check in | **stop and report** — that is the misattribution failure that produced the fabricated records removed in #2275 |
| `cargo` is missing | not a blocker; use the Python fallback and say so |

## Why step 4 is the important one

Every other path has been exercised by a lab that did not write it: claims and releases by Kimi and
Codex, heartbeats by both, committed presence by both, `skill use` by Kimi. `prep new` has **zero
foreign receipts**, and it is the gate that stands between a lab and prohibited compute under D22. If
it misbehaves for a lab that did not write it, the failure mode is a lab doing compute it should not
— so a report that it is clear and correct is worth more than the rest of this prompt combined.
