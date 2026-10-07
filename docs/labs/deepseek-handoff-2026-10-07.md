# DeepSeek handoff — 2026-10-07

**Read this top to bottom before touching anything. It is written for a session with
zero prior context.** Everything here is either a fact verified in this session or a
rule learned by violating it. Where I am unsure, it says so.

Owner instruction this session: *"make the correct recommendations and not lose sight
of the goal."* The goal is a **useful** geometric language model — conversation/memory
and coding/reasoning, ultimately frontier capability on consumer M1-class laptops.
**Everything else is instrumental.**

---

## 0. The one-paragraph state of the project

The model is an autoregressive geometric state model with exact addressed memory and
learned typed operators. It trains: the **29M rung reaches validation NLL 1.2857** on
434M tokens. It **does not yet produce correct output at scale**: all **512
construction replies fail complete/entry correctness despite falling loss**. The
mechanism itself is sound — the response-entry fit's own test suite passes **12/12**,
including assertions that entry correctness is 1 and a full free-running generation
matches byte-for-byte. **The open problem is supervision, not capability.**

---

## 1. Bootstrap — the first 20 minutes

```sh
# 1. Work in your OWN worktree. NEVER the owner's checkout.
#    /Users/casey.allard/uor-r4 is the owner's and is read-only to you.
git worktree add -b deepseek/<task> ~/uor-r4-worktrees/<task> origin/main

# 2. Read, in order:
#    AGENTS.md  (especially the "GPU pods (every session, owner rules of
#                5 October 2026)" section — it authorises pods within the caps)
#    docs/labs/README.md -> docs/labs/compute.md -> docs/labs/protocol.md
#    docs/integration/current-state.md   (the live state; newest entry first)
#    docs/labs/deepseek-session-2026-10-06.md  (this lab's record; §10 is method)
#    the live issue: #820

# 3. Check what is already running before you plan anything:
scripts/pod/uor-pod status          # pods, leases, free GPUs, caps
gh pr list --repo UOR-Foundation/uor-r4 --state open

# 4. Heartbeat: ~/.local/share/uor-r4/bin/lab-heartbeat, LaunchAgent installed
#    (org.uor.deepseek-heartbeat, StartInterval 600).
```

**Delivery:** worktree → branch → **protected PR** → merge queue. **Never direct-push
`main`, never bypass protection, never admin-merge.** Self-review is allowed.
Issue a partial result as `References #N`, not `Closes #N`.

---

## 2. THE IMMEDIATE TASK — make prediction produce correct output

### 2.1 What "prediction" means here

The **Generate / response-entry** path: given a prompt, decide to enter a response and
emit the correct bytes. Two numbers in the fit report are the whole scoreboard:

- **`final_entry_correct`** — predicted first token matches **and**
  `response_entry_decision()` returns `Enter`, re-derived **after export**
- **`final_exact_responses`** — the full free-running generation equals the response

**Current: 0 of 512.** That is the blocker. Nothing else matters until it moves.

### 2.2 The diagnosis — established this session, with evidence

Two facts, both verified:

**(a) The mechanism works.** `cargo test -p uor-r4-core --lib response_entry` →
**12 passed, 0 failed**. The passing tests include
`native_response_entry_fitting_bootstraps_actual_selected_entry_and_freezes_numeric_baseline`,
which asserts:

```rust
assert_eq!(report.entry_fit_correct, 1);
assert_eq!(report.final_entry_correct, 1);      // entry correctness
assert_eq!(report.final_exact_responses, 1);    // complete free-running output
```

This test trains entry, **exports**, re-derives the entry decision post-export, and
generates the full response byte-for-byte. **There is no export bug, no admission bug,
and no broken geometry.** When the entry objective is the whole objective, it is 100%
correct.

**(b) The entry objective is drowned at scale.** From the exposure audit at
`/workspace/uor-r4/codex/categorical-joint-fit/read-only-audits-20261007T1339/categorical-joint-fit-exposure-audit-summary.md`:

```
per arm supervised positions:  13328 = entry 1024 + laterCopy 8256 + laterGenerate 4048
loss weight:                   1/(nonempty phases × episode-phase count × B)
                               entry position weight 1/24
"Entry loss can credit all four active groups via fullbank Context/read/potential/
 Generate graph in every update; later positions also change these shared groups"
```

**The scored thing — the entry decision — is 1,024 of 13,328 positions (7.7%),
weighted 1/24, sharing all four parameter groups with the 12,304 later positions that
dominate the gradient.** The loss falls on the 92% nobody scores.

**Falling loss with zero correctness is never "the model cannot predict."** It means
the optimised quantity and the measured quantity are not the same thing. This exact
failure shape appeared four other times this session.

### 2.3 The experiment to run — ~20 minutes, not 12 hours

**Step 1 — isolate.** From the frozen parent, train **only** the entry positions with
later-position credit frozen (or zeroed), and read `final_entry_correct`.

- **Rises above zero** → the joint objective was drowning it. Go to step 2. **Prediction
  is solved at the entry step**, and the fix is a loss decomposition.
- **Stays zero** → the scale-up breaks something the fixture does not exercise. Then
  bisect: 1 example → 8 → 64 → 512, and find the first size that fails.

**Step 2 — rebalance.** Either raise the entry weight off 1/24 toward parity, or
decompose the loss so entry and later positions do not compete for the same four
parameter groups in every update.

**Step 3 — re-run the 512 panel** and read `final_entry_correct` again.

**Do not start a 12-hour run to answer this.** The question is answerable in minutes,
and the infrastructure's observed failure interval is about two hours (see §4).

### 2.4 Where the code is

- `crates/uor-r4-core/src/native_geometric/response_entry_training.rs` — the fit;
  `final_entry_correct` is computed around the loop that rechecks decisions "using the
  exported model", with the comment that *"shared global postings can change
  admission"*
- `crates/uor-r4-core/src/native_geometric/response_entry_training_tests.rs` — the
  12 passing tests; the single-example fixture is the model for a scaled isolation run
- `response_entry_runtime.rs`, `response_entry_types.rs`, `response_entry_snapshot.rs`
- The categorical joint fit is **not on `main`** — it is Codex's branch work, and its
  audits land on the volume under `/workspace/uor-r4/codex/categorical-joint-fit/`

**Coordinate before fitting.** All three labs were given the same instruction to pursue
prediction, so duplication is possible. Check `uor-pod status` leases and the volume
before launching.

---

## 3. What was completed this session (do not redo)

**Merged and verified on `main`** — verify with `git show origin/main:<path> | grep`,
never the PR badge:

| PR | what |
|---|---|
| **#1798** | `StackArch::Hybrid` — the read-attribution control |
| **#1803** | method findings → session doc §10 |
| **#1804** | the D20 §2 campaign → session doc §11–15 |
| **#1805** | the verdict → `docs/integration/current-state.md` |
| **#1806** | the parameter-matched control **landed on `main`** (was branch-only) |
| **#1807** | corrected claims that #1806 had just falsified |
| **#1827** | **Flash is the default CUDA read**; `UOR_R4_CUDA_READ=fused` opts out |
| **#1828** | pod tooling fixes (§4) |
| **#1830** | `docs/compute/bf16-phase2b-report.md` |
| **#1832** | **chat-v1's nine parquet filenames** recorded in the ladder runbook |

**The D20 §2 verdict** (three sealed seeds per cell, ~91 sealed roots under
`~/uor-r4-local/mqar-bench/`):

- At **lr=3e-4**: geometric **0.9992** vs parameter-matched control **0.2537** →
  **+0.7455** on same-rate evidence, **replicated at 2e-4 (+0.7360)**.
- The **control is near-flat** (0.2310 → 0.2454 → 0.2537 → 0.2534 across two orders of
  magnitude); the **geometric arm is not** (0.175 → 0.981 → 0.999 → lottery).
- **Floor is a schedule effect**: constant lr=1e-4 solves at 0.9605 where annealed 1e-4
  plateaus at 0.1753, because `min_lr=0.1` decays the rate below the acquisition
  threshold mid-run.
- **Ceiling is a rate effect**: lr=1e-3 acquires to 3.8× uniform by step 100, decays to
  1.3× by 200, **and constancy does not save it**.
- **Schedule-controlled edge**: geometric 0.9862 vs attention-read hybrid 0.9149 —
  **+0.0713 capability, 22.6× stability**.

**bf16 Phase 2b**: flash read **ADOPTED** — improves NLL on both seeds (−0.00262,
−0.00439 against a spread of 0.00017437), **+8.0% end to end at 29M**, but **neutral at
96M (−0.2%)**. That shape-dependence is a real limitation, recorded in the report.

---

## 4. Infrastructure — and every way it will bite you

**Only `scripts/pod/uor-pod` touches pods.** Caps: **≤4 running pods, ≤$8/h, all labs
together.** Paid GPU within the caps **is authorised** (AGENTS.md, owner rules
5 October 2026). Lease ≤30 min renewal; identity is `--lab L --session S`.

### The failures that cost the most time

| symptom | cause | fix |
|---|---|---|
| pod RUNNING, no SSH, `runtimeStatus: initializing` / `awaiting_container` | host rented, container never started. **Never recovers.** | delete and redraw. 4 draws stalled in a row this session. `uor-pod up` now detects at 180 s (`UOR_POD_STALL_SECONDS`) instead of 900 s |
| `CUDA_ERROR_NO_DEVICE` mid-run | host lost the GPUs | resume from checkpoint on a fresh pod |
| `CUDA_ERROR_OUT_OF_MEMORY` at 96M, batch 32, context 384 | **OOMs at f32 AND at bf16.** 32 GB cards | unproven at that shape — **validate at 200 steps before committing** |
| build fails: `package with the missing feature` | `--features cuda` needs `-p uor-r4-training` | always pass `-p` |
| build fails: `failed to download ahash ... --offline was specified` | `parquet-input` needs an uncached crate | **do not use `--offline` for the parquet build** |
| `tar: Cannot change ownership to uid 501` | tars carry Mac uid; root cannot chown on the network volume | **`tar --no-same-owner`** |
| lease expires mid-run | reaper takes the pod | renew every ≤25 min; the reaper waits 20 min idle |
| a long job dies silently | **no monitoring** | see below — this cost 4 hours |

### Rules that are not optional

1. **Durable output goes on `/workspace` (network volume), never `/root`.** `/root` is
   container disk and dies with the pod.
2. **Stage corpora to a durable directory, once.** `resume=OLD_ROOT/checkpoint`
   **verifies identical input sizes and SHA-256**, so rebuilding corpora between pods
   breaks the resume.
3. **Never reuse a report root.** A resume continues in a **new** root.
4. **Never cancel a healthy run because a budget you set yourself ran out.** Extend it,
   renew the lease, note it in one line. Hard stops are only: owner caps, the disk
   floor, and a run that has failed or diverged.
5. **Monitor long jobs from the laptop**, polling the log and process state every
   ~5 minutes. Launch the monitor *before* the job, not after it dies.
6. **A run counts only at the exact final step** (`steps=N`). Assert it — a run that
   dies at 110k of 122k otherwise looks finished.
7. **Check `uor-pod status` and the volume before duplicating another lab's work.**

### Corpora — where they are and the trap

On the canonical volume, `/workspace/uor-r4/data`, as tars:

- `step5-inputs.tar` (1.47 GB) → `corpora/{ts-train,ts-valid,td-train}`, `chat-v0-p2`
- `tokenizer.json.tar` → `tokenizer.json`
- **`chat-v1-p2-train` is NOT shipped anywhere.** It must be built with
  `prepare-chat-parquet` from **9 HuggingFace parquet exports** — the filenames are now
  recorded in `docs/compute/ladder-runbook.md` §2.3 (PR #1832), recovered this session
  from the HF datasets-server index. Building it reproduces the documented
  **589,939 rows / 1,388,814,239 tokens** exactly.

Fixed commands (these are the ones that work):

```sh
cargo build --release -p uor-r4-training --features cuda --example geometric-stack
cargo build --release -p uor-r4-training --features cuda,parquet-input --bin prepare-chat-parquet
tar -xf <tar> --no-same-owner -C <dir>
```

---

## 5. Method rules — learned by violating them

**These are the most valuable thing in this document.** Every one cost real time.

1. **Hold the training schedule fixed when comparing two mechanisms.** **Six**
   interpretive claims this session moved the same direction once the schedule was
   controlled. The remedy is procedural, not attitudinal: vary only the mechanism.
2. **An instrument's absence from `main` does not mean the experiment cannot run.** A
   prebuilt binary ran the control the whole time it was declared unrunnable. Check the
   actual path; it costs one command.
3. **A symbol search is a lead generator, not an audit.** It fails **both** ways: a
   squash-merged branch's tip is never an ancestor of `main` (23 branches falsely
   "unmerged"), and a renamed symbol reads as absent (`read_firing` is on `main` as
   `best_head_mean_weight_on_key`).
4. **A cross-rate comparison is a silent error of unpredictable size.** The verdict
   paired geometric@3e-4 with control@1e-3 → 0.04% error. The same mistake on the 1e-4
   claim → **40%**. Nothing distinguished them in advance.
5. **Never quote an unsealed root.** A run is evidence only when sealed.
6. **Never print probe output through a width-limited formatter** — `cut -c1-170` hid
   the field that reversed a conclusion.
7. **Read the execution path; do not grep-and-infer.** Three consecutive wrong line
   pointers came from inferring that a symbol's location was the dispatch.
8. **Let the compiler enumerate match sites.** Adding an enum variant is the one case
   where the compiler is the authoritative list: `cargo check` named exactly three.
9. **Read the section that governs, not the sentence you remember.** I told the owner
   paid compute was unauthorised by reading an older standing note and missing the
   later, more specific GPU-pods section. **Read the whole section.**
10. **A close tie is not a failure.** A rule that rejects *improvement* is a defective
    rule — Gate 1 did exactly that (`|d_s| ≤ spread` is symmetric) and was replaced by
    Gate 2's one-sided test rather than left standing.
11. **Prefer `edit`/`write` and `jq` over Python.** Rust for computation; no Python as
    glue. (No Python was committed this session — verified across all PRs.)
12. **Own your errors in the record.** Retract explicitly and mark retracted claims
    uncitable. This session retracted: "the attribution is definitive" (n=1), "the
    recurrence layers carry nothing," "a narrow rate window," "1e-4 is below threshold
    at any schedule," and "d400 collapses" — all six in the same direction.

---

## 6. Open threads and who owns them

- **Prediction / `final_entry_correct`** — the priority. Codex holds the categorical
  joint fit; the diagnosis above is filed on #820.
- **96M rung** — unclaimed. Needs the OOM understood at that shape (validate at 200
  steps), and **the ladder's original 96M was f32 while any new run would be bf16**, so
  the NLL is not directly comparable to the recorded step-5,000 figure of 1.694.
- **Token budget** — the strongest untested capability lever. The 29M rung used
  **434M tokens = 15 tokens/parameter**; modern practice is 20–100+. Loss was still
  falling at every checkpoint, and lr 0.0005 beat 0.001 by 0.013 NLL. **More tokens on
  a shape that already trains in 53 minutes is cheap and measurable today.**
- **`nsys` before/after profile** — UNAVAILABLE: Nsight Systems is not on the pod image
  (`/opt/nvidia` has `nsight-compute` only; no `nsight-systems` apt package). The
  *before* half is on trunk (Phase 2a, #1757); the *after* half needs a different image.
- **Pod queue** — was 5 pods against a cap of 4 earlier; currently small. Always check.

---

## 7. If you do only one thing

**Run the entry-isolation experiment in §2.3.** It is ~20 minutes, it always returns a
result, and it converts "loss falls but nothing is correct" into either a solved
prediction path or a precise bisect target.

**Then, if that lands, spend the next hour on the token budget (§6)** — it is the
cheapest available capability gain and it needs no new mechanism.

**Do not** start a long run to answer a question a short run can answer. That is the
loop this project has been in: experiments longer than the infrastructure's
mean-time-to-failure, with failures discovered late. Shorter loops, validated configs,
and failures that shout.
