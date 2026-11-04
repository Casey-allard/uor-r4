# ORDER: one-model-end-to-end, claude-276 (P2b's run: the one model trained at the 2,048 window)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `e1ad88afc`. Draft PR #3657.

## Goal

Run [P2b's sheet](../../../../harness/prep/one-model-end-to-end-long-context-2026-11-04.md) as pre-registered (FIT, round 1).
- Train the one model at the 2,048-token window on the long TRAIN (arm (L)), beside arm (S) at 192 for equal positions.
- Seeds 1 and 2 run on one RTX 5090 pod each.
- Score recall across turns on p-long with the window open and shut, memory on and out (L-010), and time both windows.
- Grade R1 to R5 on the laptop and apply the sheet's Bar.

- **Plan step:** Casey's 4 November plan, P2b, the run unit (bound: 1 unit).
- **What it changes in the model:** this is the first model on this line trained on conversations longer than 258 tokens. Its window sees a note up to about 2,000 tokens before the note is asked for. On a positive or partial result, its checkpoints are the candidates P4 (#3471) takes as its base.
- **The pin** is `e1ad88afc5a7f43f5e7714d9ab70b8937aa6bdde`, claude-275's merge commit on `main`, as the sheet's Base artifact names it.

## Cancel probe (run first)

1. Is the pin on `origin/main`, and does the sheet there match the FIT'd text?
2. Are both input folders in the store, each with a `SHA256SUMS`?
3. Do the caps hold two more pods (at most 4 pods and $8/h across all labs)?
4. Does any open order hold these files, or does another lab run this question?
5. Does the job file follow the sheet's Steps, arms, flags and checks, with no reading left to choose after outputs exist?

**Result, at `e1ad88afc`: not cancelled.**
1. Yes. `origin/main` is `e1ad88afc5a7f43f5e7714d9ab70b8937aa6bdde`. Since the FIT at `3d466b3a9`, the sheet has changed only in:
   - the FIT line;
   - the four minors that review named (R4's source, (S)'s positions, I-6, the under-192 row);
   - `860e303a2`'s wording, which names the census 6 as a departure from the stop rule.

   No bar, arm or reading moved.
2. Yes. `uor-store ls` lists `inputs/p-long/` (6 files with `SHA256SUMS`) and `inputs/train-long/` (7 files, `train/` included, with `SHA256SUMS`). claude-275 read both back, 11 of 11 OK.
3. Yes. `uor-pod status`: 0 pods running, $0/h of $8/h, and 14 verified 5090 offers from $0.563/h.
4. No. The only open PR on this item is this unit's draft, #3657.
5. The job file and the grade are written before any output exists, and the pre-run review (done-when 1) checks the job against the sheet line by line:
   - **The job:** `local/claude/p2/job-p2b.sh`, SHA-256 `da1938f7e685a2c88a8ace48f6237115be2f0e2ef85ec296405c5e5089b8997e` at this ORDER. It changes only if the pre-run review finds a fault, and the REPORT records the launch SHA-256.
   - **The grade:** `local/claude/p2/grade-p2b.sh`. It was `cfac639d4e783dc98786642b9f9f9d75f7af4a424c8b34947656b267b7c5a4a3` at this ORDER's first commit, and is `e40789617a59da6f3706c1d380f86813dda1c622d35c276c44aaeb7520fe76f4` after the pre-run review's fixes (below), fixed from here on. It was tested on a synthetic seed folder built from p-long's `queries.json`:
     - its b and c match an independent count (334 and 26 overall, 283 and 24 far);
     - two passing seeds read Positive;
     - identical windows read Negative;
     - disagreeing seeds read Inconclusive;
     - a pass at the wrong window reads Unread.

## Pre-run review (done-when 1)

A fresh non-author reviewer, given the sheet at the pin, this ORDER, the job, the grade and the code at the pin, said **GO WITH FIXES**. It checked the chain, every flag against the CLI parser, every path, the seven p-long and long-TRAIN SHA-256s, the grade's field names against the structs, silent-failure modes, and cost (about 5.7 h a pod at worst, under MAXSEC 25,200; $3–6 a pod; peak GPU memory about 20 GiB of 32).
- **M1 (major), fixed:** (S)'s `config.json` leaves out `context` at 192 (`skip_serializing_if = "is_sheet_context"`), so R4's training clock multiplied by null and the grade stopped. Now `($c.context // 192)`; `SHEET_CONTEXT` is 192 at the pin. Gate: with a real-shaped (S) config the fixed grade reads 22,756 positions a second, and the grade before the fix exits 5 on `number (349520) and null (null) cannot be multiplied`.
- **m1, fixed:** one seed reads `Inconclusive (one seed)`, the stop rule's word.
- **m2, checked:** a seed folder missing one file stops the grade with exit 3, before and after the change; the explicit `out=$(seed …) || exit 3` is kept.
- **m3:** each reading prints its n. On the fixture it is 603 overall and 494 far, and the REPORT compares the run's to these.
- **m4, checked:** the five driver-data SHA-256s in the job equal `hamilton_sheet.rs`'s `PINNED_*` at the pin, and p-v's two equal its sheet (`one-model-end-to-end-keyed-opening-2026-10-30.md:69`).
- **m5, kept:** I-0 runs the sheet's literal command (a debug build), adding `--no-fail-fast -- --test-threads=1`, which runs the same tests.
- **m6, noted:** (S)'s final save at step 21,845 is off its eval grid. That touches only reported readings.
- **m7, checked:** claude-274's job fetched single files the same way and exited 0. The instrument's `Cargo.lock` resolves `--locked --offline --features panel-writer`.

The job is unchanged at `da1938f7e685a2c88a8ace48f6237115be2f0e2ef85ec296405c5e5089b8997e`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-276/**`
- `docs/harness/prep/one-model-end-to-end-long-context-2026-11-04.md` (only the PRE-RUN line, the pin and the outcome; no bar moves)
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`

No code. A repair that needs code is a new unit at a new pin.

## Done when

1. **The pre-run review (D67)** is posted on #3407 before any pod.
   - **The reviewer:** fresh and non-author, given only the sheet, the job file, the grade and this ORDER.
   - **What it covers:** the whole-project chain; that every tool and input exists; what each outcome unlocks; the FIT; and the lease inside the caps.
   - **Before launch:** what it finds is fixed.
2. **Both seeds ran the job,** or each failure is classed instrument or mechanism, with its exit code and logs. A watcher is armed on every pod in the same turn as its launch.
3. **Every output is sealed** with `SHA256SUMS` under `results/claude/one-model-end-to-end-long-context-2026-11-04/sN/`, and read back before its pod goes down.
4. **The laptop grade** runs `grade-p2b.sh` at the SHA-256 above. It checks I-4 to I-6; computes R1 (overall and the far bin), R2, R2b, R3, R4 (tok/s first) and R5; and applies the Bar per seed and across both.
5. **The REPORT** leads with tok/s, puts negatives first, and gives one notebook row.

## Stop and ask if

Stop and decide; never stop and wait.
- **An instrument check fails** (I-0 to I-6): the readings stay unread. Class the fault, and repair it at a new pin in the next unit.
- **(L) projects past 3 hours** (I-2): the job cuts both arms to equal positions, as pre-registered, and records it.
- **A pod is lost mid-run:** relaunch that seed on a new pod into a new folder (L-002), inside the caps.
