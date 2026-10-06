# DeepSeek lab session record — 2026-10-05/06

Lab: `opencode-deepseek-20260930` · adapter: OpenCode desktop, MANUAL client, **no heartbeat automation**
Base: `origin/main` @ `2b21c9a3` · Owner-directed work on the addressed-attention emission path and the
representation/head question.

Recorded under **D20** (#1747): findings below are **localisers that name a missing component**. No
mechanism is retired by this document, and no near-zero delta is treated as a verdict.

## 1. Merged instruments (measurement-only; production serving unchanged)

| PR | what it added | status |
|---|---|---|
| #1759 | mqar-bench logs held-out `first-content` and its count | merged |
| #1761 | `uor-pod` accepts hand-written lease timestamps | merged |
| #1763 | Phase 2b frozen read A/B gate | merged |
| #1769 | mqar-bench `dump_scores=1` T2 instrumentation | merged |
| #1770 | addressed-attention dispatch harness | merged |
| #1773 | arithmetic reachability probe + arithmetic oracle | merged |
| #1774 | emission-context + arith-fit instruments | merged |
| #1780 | head-vs-representation probe | merged |

All are measurement-only. Every report root is sealed and archived to
`icloud:UOR-R4/results/deepseek/` with MD5 verification.

## 2. What was measured, as localisers

**2a. The engine computes exactly and cannot emit it.**
On a 32-example canonical-decimal arithmetic split, with forced read and forced operator, the operator
produced the exactly correct value **80/80**; the engine emitted that byte **0/80**. The value *is* wired
into the emission input byte-wise (`engine.rs:228-238` → `objects.rs:132-134` → `inputs.rs:120`, bits
792..800). The context is a **1024→9 LUT cascade** (`circuit.rs:5`), and at frozen init it erases the
digit: a sweep over the digit's byte field with all other bits fixed yields **2–4 distinct contexts,
never 256**, and is **constant in 3 of 9** examples.

*Named missing component:* the cascade is not trained to route bits 792..800. **Not** a wiring gap.

**2b. The proposed fix was refuted before it was built.**
A council concluded the wiring forces digit collisions (ceiling 8/10) and recommended structural
emission addressing. A Python-only replay of `Topology::seeded` at `WIRING_SEED=0x973`, verified
against an independent forward-Fisher-Yates rebuild (**1380/1380 gate pins agree**), measured
**10/10 separable digits** — and 10/10 at every seed sampled (1024 census, 40 hand-picked, 400 random).
The minimal separating projection is **2 context bits**. Seed-shopping and structural addressing are
therefore unsupported by this evidence.

*Named missing component:* the failure is **downstream** — the learned 257-way readout and/or the fit —
not in the topology. The decisive unmeasured quantity is the **unforced** emission path.

**2c. The STE hypothesis was falsified, and the method was not an STE.**
An opt-in relaxation (`ste.rs`, 568 lines, purely additive) moved the discrete surface **8.5× more**
than the score-function fit (606 of 5520 LUT signs, 147/345 gates, vs 71), so "gradient cannot reach
the discrete tables" is **false**. It still produced a NULL: distinct fit contexts collapsed **18 → 1**
by update 250, and the relaxed gradient decayed **3.69e-5 → 2.46e-43** with `soft_loss == hard_ce` from
u1000. A subsequent audit found the implementation computes the objective from `soft_forward` and
differentiates it analytically — **there was no hard-forward/soft-backward pair**, so "the STE signal
dies" was vacuous. Default path proven byte-identical; suite 50 → 60 passed with only new tests.

*Named missing component:* the **objective** (a collapse attractor) and a genuine gradient-preserving
straight-through derivative — not reachability of the discrete tables.

**2d. The prediction head is not the binding defect on the current artifact.**
On `chat-100m2-C` (sha256 `3d20344c…`), the served head **is the tied embedding** — dense 4096×1024,
rank 1024/1024. Held-out: a plain linear softmax recovers **98.8%** and a **zero-parameter 1-NN**
recovers **93.3%** of served accuracy. Rank: **1025 nonzero singular values, not 9**; the participation
ratio lands at 9.05 but is a **coincidence of the statistic** — the direct control (truncate to rank 9)
costs **6.10 bits and 52 points**. The recorded baseline reproduced to **3.2e-8**.

*Named missing component:* none identified here. This is a **negative result that closes a work class**
— do not fund geometry/routing/typed operators at the prediction step on this evidence.

**2e. Two premise corrections found by reading source before running.**
The 9-parameter `OUTPUT_MIX` sign walk (`shared_core.rs:47-48`) is **not** the served head — the file
states "No retained-model dispatch uses it"; it is the schema-/1 fallback. A council had cited it as
"the defect is the architecture". Separately, the `P − A = 0.5423`-bit measurement was **not**
reproduced: its artifact is not on local disk.

## 3. Operational incidents (programme-level — #820)

**3a. Coordination silently expired.** `coord status` showed the DeepSeek lab heartbeat at
**~Oct 1** against a live sequence of 896. The lab record says why: adapter *"no heartbeat automation"*,
and `coord.rs:492` expires stale labs. A heartbeat was applied this session (sequence 896 → 897,
commit `a678328f`). **The failure is structural and will recur when a session ends.**

**3b. `coord` STORE is `coord.git`, not the runner directory.** Passing the runner dir yields
`invalid lab-runner input: git config: ` — which reads as a missing-identity error and is not one.
`git config user.name/user.email` were unset; both are now set to the owner's identity.

**3c. `host-policy.json` carried dead hardware.** It declared `/Volumes/UOR-Workspace` (60 GiB reserve)
and `/Volumes/X10 Pro` (120 GiB) — both superseded when the drive failed and the workflow moved to
iCloud. Only `internal` remains; backed up to `host-policy.json.bak-20261006`.

**3d. Admission was held and said so.** `admissions-held.json` recorded
`volume internal free bytes 27382095872 below required 43083890688` — i.e. **25.5 GiB against a 40 GiB
reserve**. The declared reserve is **40.00 GiB** (`host-policy.json`), matching the `warning` line of
`agent-execution-policy.json`'s storage watermarks (internal target 60 / warning 40 / stop 25). The
machine now measures **46.51 GiB**, above the reserve; **the hold file is stale and not re-evaluated.**

**3e. A "lost" artifact was catalogued, not lost.** The `olx-form-2` artifact behind `P − A` is indexed
in iCloud as a 1.12 GB object (md5 `2a065c04…`) under lab `workspace-2026-10-01` — the migration off the
failed drive. A probe followed a symlink to dead hardware and declared a clean blocker.

**3f. iCloud placeholders read as an empty mirror.** `du -sh` reports 4.0K for the iCloud tree because
files are **dataless placeholders** (`blocks=0`, full logical size). Archives are durable; local disk
is not consumed. Do not read this as data loss.

**3g. Local policy drifts from repo policy with nothing checking it.** The local `host-policy.json`
(mtime 2026-09-30) only partially reflects `agent-execution-policy.json` in main. **A CI guard comparing
declared watermarks against the local policy would prevent a class of silent divergence.**

## 4. D20 §2 compliance state (honest)

| # | precondition | state |
|---|---|---|
| 1 | parameter- and compute-matched ordinary control per mechanism | **NOT MET** for the geometric read — `mqar-bench`'s `ArmSpec` has only the Stack variant with the architecture hardcoded, so the read has never been compared to a matched ordinary attention model |
| 2 | ≥3 seeds per result | **NOT MET** for the read; met for the arithmetic/emission measurements where seeds were varied |
| 3 | wiring verified reached (firing counter) | **MET** for addressed_attention — `Work::selected_operations` is the firing counter and returned a number (`0` at baseline, `9` with digit spans) |
| 4 | written root-cause case naming what would change the answer | **PARTLY MET** — see §5 |
| 5 | novel geometric candidates invented and tested | **NOT MET** — no new geometric candidate was invented this session |

**§2 is not satisfied, so §3 suspension does not apply. No mechanism may be retired.**

## 5. What would change the answer (D20 §2 condition 4)

- **Unforced emission path** (`CompiledPolicy` argmax rows, no forcing): unmeasured, needs a build. This
  is the single decisive number for §2b.
- **`SampledPolicy::set_target` does not force the symbol** (`policy.rs:235`; the target only feeds CE,
  the draw is `categorical(logits, legal, key)` at `:307`). Any forced-emission harness needs a
  harness-local `Policy` wrapper.
- **Read vs parameter-matched ordinary control** (§2 condition 1) and **≥3 seeds** (§2 condition 2).
- **A contrastive/ranking objective reaching the query/key encoders** — a read-only score function that
  is monotone in distance cannot be reordered by training the read's own parameters; every increasing
  reparameterisation preserves the argmax.

## 6. Next dependencies

1. Run the unforced emission measurement (needs a build).
2. Run the read against a parameter- and compute-matched ordinary control at ≥3 seeds.
3. Automate the lab heartbeat, or make expiry visible — otherwise coordination lapses again.
4. Add the local-vs-repo policy watermark guard.

---

# Addendum — 2026-10-06, later session (rounds 8–12)

The **Next dependencies** above are now discharged or superseded. This addendum records the outcome
so the earlier sections are not read as current.

## 7. The coordination deadlock was real, and it is fixed

**Symptom.** Every `Claim` from every lab was refused:
`invalid lab-runner input: shared policy changed or comparison incomplete; hold admissions and perform
reviewed policy migration`.

**Cause — a closed dependency cycle.**
1. `Action::Claim` calls `require_current_policy` (`coord.rs:1398-1401`), which validates
   `compare/<store pin>...main` via `validate_policy_comparison` (`:226-250`). The store was pinned at
   `ba266ad4` (#1529), **215 commits behind main** — `files=300` *and* a modified `AGENTS.md`, either of
   which trips the guard.
2. The remedy, `Action::AdoptPolicy`, refuses while any attempt is non-finalized (`:425`).
3. Exactly one was: **`opencode-b0-flock-g1a2-20260930-g2`** — reserved against `/Volumes/UOR-Workspace`,
   **the volume destroyed with the drive**, cancelled in the queue and never started.
4. Finalizing it required `owned()` (`:344-362`) — a live, unexpired, same-session claim — and claiming
   required the policy guard to pass.

**Fixing it.** `Action::Abandon` was added (PR **#1791**, `coord.rs` +548/−51): it finalizes a `reserved`
attempt whose exit evidence **positively proves `never_started`**, requires **the reserving session**, and
deliberately does **not** call `owned()` — that omission is the fix. `FinishAttempt` is unchanged for live
work, proven by a regression test. Applied as commit `7bc2af58`. Preconditions then read: **0 live claims,
0 non-finalized attempts.**

The owner then authorised the pointer repair (`e6a40b71`), moving the store to **`e445ee946573`** — PR
#1758's squash-merge commit, verified to pass the guard (`ahead`, 49 files, no guarded files). **No policy
content changed.** Note `86bec270` (D20 itself) **fails** the guard, because `AGENTS.md` changed after it:
the target is the newest guarded-**file** commit, not the newest decision.

Getting a claim through took four more rejections, each a real requirement: the `work_card` must be
literally `sha256:<64-hex>`; **ownership paths must have no trailing slash**; and the scope must not
overlap a live claim — Codex held `.../examples/geometric-generate-update.rs`, so claiming the whole
`examples` **directory** collided. Narrowed to `mqar-bench.rs`; claim granted (seq 907).

## 8. Three more runner/host defects found and fixed

- **A storage WARNING was acting as the hard admission gate** (#1787). The repo declares
  `internal: target 60 | warning 40 | stop 25 GiB`; the local `host-policy.json` had set
  `reserve_bytes = 40` — the *warning* — as the hard gate. It blocked the very cleanup needed to get back
  above the goal. Reserve corrected to the stop level (25 GiB); the checker now **warns** at 40 and
  **gates** at 25. This is the general rule: a goal to maintain must not block.
- **`admissions-held.json` had no removal path** (8+ writes, zero removals; contrast
  `admission-blocked.json`, cleared at `daemon.rs:1028`). A transient condition therefore latched the
  daemon off permanently — live at 39.89 GiB against 40.12 GiB. Fixed by PR **#1793**: holds are classified
  `Terminal | Recheckable` **in the data**, all 13 `host::hold` sites explicitly classified (one flagged
  AMBIGUOUS rather than guessed), and a recheckable hold re-runs the *same* predicate and clears itself.
  **That change alone would not have released the live record** — it predates the `kind` field and so reads
  back Terminal by design. The manual clear was necessary.
- **An orphan audit** (#1790) now reports DONE evidence that `coord` never learned about. It found three
  of thirteen unreconciled, **two absent from `coord` entirely** and **one belonging to another lab** —
  the same failure mode without the consequence, which is why it had gone unnoticed.

## 9. Measurement results

- **The matched ordinary control now exists.** `ArmSpec` had a single `Stack` variant with
  `arch: StackArch::Geometric` hardcoded, so **the geometric read had never been compared to a
  parameter-matched ordinary attention model** (D20 §2 condition 1). That is now implemented and built
  (`arm=transformer`, `mlp=matched`, `read_firing`, `context_reachability`), preserved on
  `deepseek/d20-control` after its authoring agent stopped without reporting.
  Matching is **a number, not an assertion**: reference `rrarra` = **1,370,008**; C6 (layers=6, mlp=395) =
  1,370,496 (**+0.036%**) but carries **+19.9%/+36.0% per-token MACs**; C2 (layers=2, mlp=1527) =
  1,369,984 (**−0.0018%**) with the access term exactly matching. **Neither is a clean control** and each
  carries a stated residual.
- **The norm-entanglement escape closes** (PR #1794). On **identical frozen q/k**, cosine **0.0143** vs dot
  **0.0137** = **1.04×** against a 5× bar. Verdict **PARTIAL**, not "not-recoverable" — the pre-registered
  clauses disagreed and both readings were reported rather than the flattering one.
- **Stage-0's frozen-swap was valid only at layer 0.** Swapping `config.read` re-runs the forward, so q/k
  at layers ≥1 are not the vectors A1 scored (offline-vs-library max abs diff 1.85e-07 at layer 0 but
  **0.197–0.919 at layers 1–5**). **This qualifies how Stage-0's A2b result may be cited.**
- **The headline reframing.** A1's 0.0595 is dominated by the **learned per-distance age bias** — a
  positional prior. With age zeroed, **every** score family collapses to a flat 1.2–1.7% across
  d16..d400; with age restored, every family jumps **7.5–13.5× at d16 only**, and **at long range A1 is
  worse than dot or cosine**. The surviving signal was a **recency effect**, not content identity.
- **Named missing component: content identity in the read's query/key projections.** The candidate set
  **contains** the answer (exact-identity sieve = **1.0000** on the same rows/positions/candidates) and no
  score function over these vectors finds it. The read has a usable **positional** selector and **no
  measurable content selector at any distance**.

## 10. Process findings worth carrying

- **An agent's report about shared state is a hypothesis, not a fact.** Four claims were checked and found
  wrong: `dump_scores=1` "never implemented" (it is, at `mqar_bench_step2/fact.rs:836`, producing 56 MB);
  an artifact "physically absent" (catalogued in iCloud); an admission hold "ACTIVE again" (it was an
  archived file, and the gate was 25 GiB not 40); and a runtime file reported as an empty iCloud mirror
  (dataless placeholders report `du` 4.0K with `blocks=0`).
- **Subagents are not addressable after dispatch** — `send_message` fails for them. Two agents duplicated
  the same 12-run matrix because one concluded a slow agent had died. Prefer `spawn_teammate` when a reply
  path is needed.
- **Verify the tree, not the PR badge.** #1770 merged while two of its three commits were still arriving;
  only `git cat-file -e origin/main:<path>` caught it.
- **Gate a destructive step on its own check.** A stale-hold clear was attempted once with the precondition
  printing `not cleared` and the file moved anyway (restored immediately). The second attempt ran the check
  as a separate script and moved the file only inside the passing branch.
- **The runner test suite has pre-existing flakes**: 3 of 6 unmodified-baseline runs failed identically in
  `unknown_monitor_preserves_paths_child_and_charge_until_stopped_reconciliation` and
  `cancel_finalizes_a_running_job`.
- **Hold the training schedule fixed when comparing two mechanisms.** Six interpretive claims made in this
  session moved the same direction once the schedule was controlled — every one of them had attributed to a
  mechanism something the annealing schedule was doing:

  | claim | as filed | schedule-controlled |
  |---|---|---|
  | attribution of the effect to the geometric read | "definitive" | retracted (n=1) |
  | the working rate band | "narrow window" | an octave (2e-4 → 3e-4) |
  | lr=1e-4 behaviour | "below threshold at any schedule" | constant 1e-4 solves at 0.9605 |
  | the geometric read's distinctive property | "reliability" (var ratio ~2.4e5) | mostly the anneal |
  | the recurrence layers | "carry nothing" | retracted |
  | the hybrid's bimodality | a property of attention reads | mostly the anneal (var 0.2530 → 2.51e-3) |

  The final schedule-controlled head-to-head — same `pattern=rrarra`, same lr=3e-4, same `min_lr=1.0`
  constant schedule, three sealed seeds each — puts the geometric read at **0.9862** against the
  attention-read hybrid's **0.9149**: a **+0.071** capability edge and **22.6×** greater stability, both
  roughly an order of magnitude smaller than the annealed comparison implied. **Six for six in one
  direction is a bias, not six coincidences.** The remedy is procedural rather than attitudinal: vary only
  the mechanism, and fix the schedule — a rule that would have caught all six at the point of the claim
  instead of five rounds later.
- **An instrument's absence from `main` does not mean the experiment cannot run.** The D20 §2
  parameter-matched control was not on `main` (`mlp=matched` and `parameter_match` returned zero hits), and a
  conclusion was drawn from that — "the 2e-4 control test is still unrun" — when a prebuilt binary at
  `~/.cache/uor-r4-d20control/release/examples/mqar-bench` had been able to run it the whole time. Two
  rounds were lost to reasoning about runnability instead of checking it. The same failure produced a
  symbol-level "audit" that reported a present instrument (`read_firing`; on `main` it is
  `best_head_mean_weight_on_key`/`_on_value`) as missing, and an ancestry check that labelled all 23
  `deepseek/*` branches unmerged because squash-merging rewrites the commit so a merged tip is never an
  ancestor. **They fail in both directions.** Check the actual path; it costs one command.
- **A cross-rate comparison is a silent error with unpredictable size.** The D20 §2 verdict paired the
  geometric arm at lr=3e-4 with the control at lr=1e-3; running the control at 3e-4 moved the margin from
  +0.7458 to +0.7455 — a 0.04% error. The same mistake on the lr=1e-4 claim moved a gap from −0.078 to
  −0.056 — **40%**. Nothing distinguished them in advance. Measure the comparator at the rate in question.

# Addendum 2 — 2026-10-06, D20 §2 measurement campaign (later session)

Everything below is sealed evidence on #820. 90 sealed roots under `~/uor-r4-local/mqar-bench/`.
Same panel throughout: `pattern=rrarra` (reference **1,370,008** parameters), `context=512`, `batch=8`,
1800 steps, 460,800 supervised queries, all `stopped_early_at_max_seconds FALSE`.

## 11. The D20 §2 verdict, on same-rate and replicated evidence

| lr | geometric | C6 control (mlp=matched) | margin |
|---|---|---|---|
| **3e-4** | 0.9980 / 1.0000 / 0.9995 → **0.9992** | 0.2637 / 0.2456 / 0.2520 → **0.2537** | **+0.7455** |
| **2e-4** | 0.9985 / 0.9746 / 0.9712 → **0.9814** | 0.2534 / 0.2378 / 0.2451 → **0.2454** | **+0.7360** |

**The verdict was originally stated across rates** (geometric@3e-4 vs control@1e-3). Running the control
at the verdict's own rate moves the margin by **0.0003**. D20 §2 condition 1 is satisfied with a
same-rate comparator, and the result replicates at a second rate.

## 12. The rate governs the geometric read and barely touches attention

| lr | geometric | hybrid (ordinary attention reads) | C6 control |
|---|---|---|---|
| 1e-4 | 0.1753 | 0.2285 | 0.2310 |
| 2e-4 | **0.9814** | — | 0.2454 |
| 3e-4 | **0.9992** | 0.5714 *(annealed)* | 0.2537 |
| 1e-3 | 0.6689 *(var 0.3283)* | 0.0091 | 0.2534 |
| 1e-4 CONSTANT | **0.9605** | — | — |
| 3e-4 CONSTANT | **0.9862** | **0.9149** | — |

**Both edges have different causes.** The floor is a **schedule** effect: at 1e-4 the annealed arm never
acquires (0.1753, probes flat at 1.2–1.4× uniform), while the **constant** 1e-4 arm solves at **0.9605** —
`min_lr=0.1` decays the rate to 1e-5 across the acquisition threshold mid-run. The ceiling is a **rate**
effect: at 1e-3 the read acquires to 3.8× uniform by step 100 and decays to 1.3× by 200, **and constancy
does not save it**.

**The control is near-flat** — 0.2310 → 0.2454 → 0.2537 → 0.2534 across two orders of magnitude, a 0.023
spread — against the geometric arm's 0.175 → 0.981 → 0.999 → lottery. **The capability crossover sits
inside the same factor-of-two window as the schedule floor.**

## 13. The geometric read's edge, schedule-controlled

The only comparison in which the schedule is not a free variable — same `pattern`, same lr=3e-4, same
`min_lr=1.0` constant schedule, three sealed seeds each:

| arm @ 3e-4 constant | mean | variance |
|---|---|---|
| **geometric** (fused read) | **0.9862** | 1.11e-4 |
| **hybrid** (attention reads) | **0.9149** | 2.51e-3 |

**+0.0713 capability, 22.6× stability.** Compare the annealed framing this replaced: a capability
comparison implicitly reading 0.9992 against 0.5714, and a variance ratio of ~240,000×. **Both effects are
real; both are about an order of magnitude smaller once the schedule is fixed.**

## 14. `arm=hybrid` — the read-attribution control (PR #1798, in `main`)

`StackArch::Hybrid`: `pattern`'s `r` layers run the quaternion recurrence unchanged, every `a` layer runs
ordinary causal attention. One enum variant, five match arms, three parser lines — at exactly the three
sites `cargo check` reports as non-exhaustive. Additive; nothing retired.

`probe_steps>none` is rejected for the hybrid with `read binding needs a declared geometric read layer and
head`, which is **correct** — the binding requires a geometric read and a hybrid has none by construction.

## 15. Reproduction notes

- **The parameter-matched control is now on `main`** (PR **#1806**). Carried from `5a37b981` by
  `cherry-pick -x` with authorship preserved; all three conflicts resolved as **union**, plus two
  `StackArch::Hybrid` label arms because the branch predates #1798. Verified after relocation: **0.2529**
  sealed against the original's 0.2529 (3-seed mean 0.2534) — so D20 §2 condition 1 is runnable from trunk.
  **Three cherry-pick attempts were aborted before this one**, on a diagnosis that was wrong twice: the
  hunk-2 conflict is *not* semantic. `main` defines `stage0_tally_dump`/`stage0_sieve`; the branch defines
  `read_firing`/`context_reachability` — different functions added at the same line. One `grep -nE '^fn '`
  on both files would have shown it. That is the third time this session a symbol mismatch was read as an
  architectural question.
- **The firing counter IS on `main`** — as `best_head_mean_weight_on_key` / `_on_value` with
  `uniform_weight_reference`, not as `read_firing`.
