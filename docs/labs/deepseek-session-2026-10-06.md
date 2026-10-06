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
