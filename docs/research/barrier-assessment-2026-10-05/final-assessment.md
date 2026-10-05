# UOR-R4: final assessment for the next quarter

**Scope.** Repo at `71b54adf`, the merge of #1705. I checked the head and the latest #820 comments read-only. The newest comment is from 2026-10-05T02:04Z, and the F2 pod A/B (#820, 00:36Z) had not reported by then. Labels: **P** is proof or exact by construction, **M** is measured at the stated scope, **H** is hypothesis. Every one of the five proposals was refuted as a whole. This plan uses only the parts the reviewers marked as worth keeping.

---

## 1. Executive summary

**What it is.** UOR-R4 is a Rust geometric state model: a quaternion-transport recurrence (`r`), an L2 read (`a`) and a pointer head. The ladder ran 8M → 29M → 96M. Chat is grounded by an exact external store and an authored log sieve. Serving goes through a D11 integer engine with no float and no multiplier (#1691).

**Where it is.**
- Base NLL scales: 1.59 → 1.17 (M).
- The open panel is flat at 43–46/232 from 29M to 96M (M).
- With the sieve off, the network alone answers 3/109 MQAR (M).

**Why it is stuck.**
- Three months went into changing how scores are computed while the key content stayed the same.
- Instrument defects were repeatedly read as mechanism failures.
- Bench geometry never matched deployment: gap 1, single-token keys, and the `add_prefix_space:false` copy boundary.
- The panel itself was never decomposed. Nobody has measured which panel failures recall could fix at all.

**What #1701/#1704 actually established.**
- **P:** `<g·x, x> = Re(g)|x|²`. A pure-imaginary predecessor code is therefore self-incoherent: a query tuned to the predecessor fires on the successor, never on the key itself.
- **M, bench only:** with this code, recall becomes robust (1.000 at every distance; 1024/1024 unseen pairings). The production conv route already supplies the same signal on some seeds (`rrarra` seed 2 = 0.9995; `rararr` = 1.000 with no shift).
- So "canonical token lineage" is a cheap, exact, D11-friendly **Carry/robustness operator**. That it is the missing piece is **H**, and it is not yet tested against a learned shift or conv.

**What will unstick it.** Mostly measurement, plus three engineering pieces:
1. Training-free audits: a panel failure taxonomy, a tokenizer/format audit, a teacher-forced rehearsal probe, and a D11 profile.
2. Read the pending F2 pod A/B, plus one dose-only control arm.
3. A bench that matches deployment, with learned-shift controls.

Then three pieces of engineering:
- port F2 to D11, if it earns its place;
- fix the copy-token boundary;
- move whichever lever the taxonomy names, which is probably the corpus and knowledge rather than attention.

---

## 2. The integrated solution

The plan has one served stack and one decision about attention: the **stack L2 read with predecessor lineage on its keys**. The Codex supplied-record bank (#1705, 0–1/32 fresh) comes off the critical path and closes process gap P1.

### A. Carry: lineage keys

**Default to adopt if Step 1 passes:**
- **What it is.** F2, `k'_s = k_s + L_j k_{s−1}`, from `geometric_stack.rs:8913-8944`.
- **Its status.** Accept it as a robustness operator. Do not claim it as semantic geometry.
- **What has to be measured before calling j "canonical".** j must beat three alternatives: an identity shift (which also fires on the key itself), a random fixed SO(4) per lane, and a learned W_prev or learned width-4 depthwise causal conv on k. These are the H3 (arXiv:2212.14052) and Canon (arXiv:2512.17351) forms.
- **If those controls tie,** keep j only because it is the cheapest exact D11 form: a signed permutation plus an add. Retract the "canonical" claim.

**Multi-lag extension, opened only if Step 2 shows a real lag gap:**
- **P:** iterating j is wrong, because j² = −1 makes lag 2 anti-coherent.
- If lags 2–3 are needed, use orthogonal lanes or disjoint lane blocks with `{1, i, j, k}` codes, or a dyadic 2T `ω = (1+i+j+k)/2` code. The reviewers showed this gives the same exact lags as the icosian code at lower serving cost.
- Z[φ]/order-10 icosian codes come back only if a window longer than 6 is shown to be needed.
- In every case the same controls apply: identity, random SO(4), learned conv, and disjoint-lane concatenation (arm h).

### B. Serving form under D11

Use shift-on-read from the cost-serving review of ILC:
- Cache only raw `k_s`.
- Score with `|q − k'_s|² = |q|² + |k_s|² + |k_{s−1}|² + 2X_s − 2⟨q, k_s⟩ − 2⟨j⁻¹q, k_{s−1}⟩`, where `X_s = ⟨k_s, j k_{s−1}⟩` is computed once at write.
- This leaves the key cache unchanged and keeps rollback exact.
- It removes the refusals at `stack_export.rs:346, 991` and the refusal test at `:1726`, in about 200–250 lines, as already designed in #1704 §3.
- Do not build KV codecs or LSH. #1691 profiled weight maps at 84.7% of time and the read at about 10% at context 384, and the served engine refuses positions at or beyond its trained context (`session.rs:1433`).
- So D5 sparsity effort belongs in **weight-row routing**, not KV admission.

### C. Copy

The `m`(79) vs `Ġm`(283) boundary (catalogue 8.3) caps copy credit at 0/33. It also makes a key rehearsed at the start of a reply a *different token tuple* from the asserted key. That breaks both the pointer and any exact or soft n-let rehearsal match.
- Fix this before any chat-level lineage verdict.
- Two options: protocol 2 (#1591), or a fixed copy-identity table that maps each leading-space/plain token pair to one id. A table read is D11-legal.

### D. Exact-learned join, later

Replace the authored sieve's **answer path** with an exact **admission** source scored by a learned ranker:
- an interned word-id index with eviction counters, which makes absence provable;
- tabulation hashing (arXiv:1011.5200), which uses XOR and table reads only.

This is ADR-0003 stage 4 done with integer ids, not primes. **P:** square-free products carry no metric. The sieve stays the fallback until the learned ranker matches it.

### E. Geometry where it has earned a role

Three measured places:
- transport in `r`: +0.023 nats over U(1), 2 seeds (#1639);
- exact signed-permutation lineage: F2;
- B1 2I finite-group lanes: 17/18 runs tracked A5 at length 4,096. It was closed inside seed spread, not refuted.

Prime, zeta, Hopf and E8 remain identity labels or matched-control arms only.

---

## 3. Experiment plan

Each step lists its decision rules and kill criterion, frozen before running.

### Step 0. This week, laptop, training-free, about one day total

Every report root is claimed exclusively and sealed. Open fixtures only.

| Sub-step | What | Decision rule |
|---|---|---|
| **0a** | Taxonomy of the 186 failing open-panel rows at 96M-C: knowledge-absent / incoherent / instruction / recall-anaphora-consistency. | If recall-type rows are under 15%, no attention mechanism can move the panel more than a few rows. The panel lever is then corpus/knowledge (Step 5), and retrieval work is gated only on sieve-off MQAR. |
| **0b** | Tokenizer/format audit of the 109 D19 rows with the #1017 tokenizer. Compare the token tuples of the key in the assertion, the query and the reply start, at m = 1, 2, 3. Also count the share of "It's/That's {v}" replies (`milestone_world_v2.rs:1022`). | If more than 10% of tuples mismatch, Step 4 (copy fix) runs before any chat lineage A/B is read as evidence. |
| **0c** | Teacher-forced probe on the 29M lr5e-4 and 96M checkpoints, sieve off. (i) Free-run, recording the reply-form split. (ii) Force the gold "{k} is", BPE-exact, and score the first value piece. (iii) Same with the copula swapped. | If (ii) is high, the network can already do n-let match: the bottleneck is rehearsal or dose, and lineage work is not on the critical path. If (ii) is low only on multi-piece/copula rows, Step 2 is justified. |
| **0d** | Re-run the #1691 sampling profile at 96M D11, at 1 and 4 threads, with the context filled to 384. | If the read share is under 30%, all KV-sparsity/D5-admission work is closed and D5 effort goes to weight-row routing. |

**Kill:** none. These are instruments. Each result is posted on #820 the same day.

### Step 1. Pod, already in flight

- **Arms:** read the F2 A/B (base + Arm C, `key_shift` off vs `add`, sieve-off D19). Add two arms at raised MQAR dose (about 12%): **off@high** and **F2@high**, 2 seeds, preferably at 29M.
- **Budget:** about 4 short fine-tunes. Project the cost against the cumulative ledger. The owner also needs to record the GPU authorization in DECISIONS (synthesis contradiction #6).
- **Decision:**
  - If off@high comes within 5/109 of F2@high, Carry is not binding: ship F2 only as robustness and move effort to Steps 4–5.
  - If F2@high exceeds off@high by at least 10/109 on both seeds (McNemar p < 0.05), with dev NLL inside the seed spread, promote F2 and start Step 3.
- **Kill:** if both arms stay at or below 15/109, lineage is not the lever on D19. Go to 0b/0c and Step 4.
- **Pre-registered:** a null result counts as evidence. There is no "lag mismatch excuses the null" clause.

### Step 2. Laptop, about 1 day of code and about 4–6 h wall

- **Bench:** extend `examples/mqar-bench.rs` with `layout=fact`:
  - 1–3-piece keys;
  - copula gap g ∈ {0, 1, 2, 3}, with gap tokens drawn from a vocabulary that overlaps keys and values;
  - REHEARSE (`prefix=self`) and BARE ("It's") answer forms;
  - case and prefix-space variants;
  - an R-nlet/rehearse reference rule;
  - a `save_model` option, so bench roots keep weights.
- **Patterns:** `rararr` and `rrarra`, the production-like patterns. `aaaaaa` is diagnostic only.
- **Arms:**

  | Arm | Seeds |
  |---|---|
  | none | 3 |
  | F2 | 3 |
  | learned width-4 depthwise causal conv on k | 3 |
  | identity shift | 2 |
  | random SO(4) per lane | 2 |
  | learned W_prev | 2 |

  About 26 runs × about 15 min.
- **Decision:**
  - If none ≥ 0.95 on the g ≥ 1, multi-piece cells across seeds, there is no lag problem in the production patterns. Stop lineage research.
  - If F2 ≥ 0.95, F2 is sufficient.
  - If the learned conv is within 0.03 of F2 on every cell, the geometric claim for j is retracted. F2 is kept as the cheapest exact form.
  - If REHEARSE passes and BARE fails for all arms, the remaining gap is reply-form/data: train "{k} is {v}" targets. It is not a mechanism gap.
  - Arms must beat the rehearse+R-nlet rule to count as a capability.
- **Conditional 2b:** only if the conv patterns fail g ≥ 1 cells while some lineage arm passes. Run lags 0–3 in disjoint lanes vs Q8 superposition vs learned conv, 3 seeds, with the same kill rules.

### Step 3. Gated on Step 1 promotion

- **Work:** D11 F2 shift-on-read port, a bit-exact float-vs-integer test, GPTQ export, `save_state`/`restore_state` parity.
- **Accept:** integer recall within 0.02 of float on the bench and on D19; read top-1 agreement ≥ 0.98; ids/s change ≤ 3% vs 32.5 (#1691).
- **Kill:** fidelity misses accept, and two remedies have failed: straight-through training of the export snap, and fixed gains in {0, 1}.

### Step 4. Copy boundary, gated on 0b

- **Work:** protocol 2 or a copy-identity table.
- **Accept:** the copy panel moves above 0/33, to at least 15/33, with sieve-off D19 not worse.
- **Kill:** no copy gain after the table fix means the pointer itself is the problem. Then revisit the Lorentz-scored pointer, which was never run (D18 §10).

### Step 5. Knowledge/corpus lever, gated on 0a

- **Arms:** if knowledge-absent is the largest category, run 29M, 2 seeds, matched tokens: current mix vs a mix with an admissible open-domain knowledge corpus.
- **Accept:** open panel +10 rows paired (p < 0.05) with instruction following not worse.
- **Kill:** no panel gain means the plateau is coherence/capacity. Then the next lever is model capacity per J/token, not data.

### Step 6. Exact-learned join, after Steps 1 and 4

- **Work:** the interned-word admission index with a learned ranker, replacing the sieve answer path.
- **Accept:** grounded session within 10 of 1008 with the sieve answer path removed.
- **Kill:** if the 0b-style false-negative rate exceeds the sieve's 4/109 misses, keep the sieve.

---

## 4. What to stop, and which retired mechanisms to revisit

### Stop

- **Score-geometry swaps** (Lorentz, H4 potentials, Hopf sectors, 2I read codes) until key content is decided. Lesson 3: the content stayed the same while the scoring changed.
- **ILC-style work:** icosian/Z[φ] KV codecs, LSH, multi-probe and KV sparse admission. It is not justified at context 384 (#1691; `session.rs:1433`).
- **Codex supplied-record cue-carrier fits on the critical path** (#1705, 0–1/32 fresh).
- **Ladder spend above 29M** before 0a.
- **Single-seed verdicts.** The #1698 → #1704 reversal is the example.
- **Benches whose layout makes the pass a tautology:** oracle segmentation, fixed 3-token copulas, unique disjoint keys.
- **Pre-registered clauses that excuse a null.**
- **"j-powers" multi-lag**, since j² = −1 (P).
- **Gating retrieval mechanisms on the open panel.** The panel remains the *programme* milestone.

### Revisit, narrowly

| Mechanism | Narrow reason |
|---|---|
| B1 2I finite-group lanes | Closed on +0.018 NLL inside a 0.063 seed spread. Re-test with 3 seeds. It is the only measured 2I capability. |
| Lorentz-scored pointer | Never run. Revisit only after the copy boundary is fixed. |
| ADR-0003 stage 4 | As integer-interned admission (Step 6), not prime arithmetic. |
| A1/D18 outcome D | Reinterpret only, with no rerun. Its signature matches missing Carry plus BPE keys plus the copy boundary. |
| D2 margin gate (0.30) | Unsatisfiable as defined. Redefine it before any D2 rerun. |
| KVAR Q8 relative energy | No longer degenerate once keys carry lineage. Low priority, after Step 2. |

The reviewers' work confirms that the Spin/HELM-D frame, Hamiltonian flow, O(1)-state exact recall and primes as a metric all stay retired (P).

---

## 5. Roadmap update and process changes

### Milestones

| Milestone | Target | Acceptance |
|---|---|---|
| **M0** | Week 1 | Step 0 results on #820. DECISIONS entries for: (a) which attention serves chat (stack L2 + lineage); (b) GPU authorization; (c) whether "no RoPE" covers fixed signed-permutation lineage operators. |
| **M1** | Weeks 2–3 | Sieve-off D19 MQAR ≥ 30/109 at 29M, network alone, on both seeds. Dev NLL within seed spread. Attribution split across dose, F2, rehearsal and copy. |
| **M2** | Weeks 3–5 | F2 served under D11 meeting the Step 3 acceptance figures. A J/token baseline measured on the M1 (R5). |
| **M3** | Weeks 4–8 | Copy ≥ 15/33. Grounded session ≥ 998 with the sieve answer path replaced by learned ranking over exact admission. |
| **M4** | End of quarter | Open panel ≥ 60/232 at 2 seeds, paired p < 0.05, through the lever 0a names. No energy or "frontier" claims. |

### Process changes

1. Each work card carries a frozen decision rule that names the next action for every outcome. A run with no decision it can change is not launched (D9).
2. A bench–deployment parity checklist is required for every bench claim: gap, key pieces, prefix space, reply form, context ≥ 317.
3. Seed minimums: at least 3 seeds for bench verdicts and at least 2 for chat verdicts.
4. Every new mechanism includes an identity control and a learned-shift or learned-map control.
5. Instrument check first: when a result is null, check the measurement before the mechanism.
6. Findings propagate in the same PR. Each result updates the retired-mechanisms catalogue row and #820, which prevents a fourth rediscovery of predecessor identity (ADR-0003 → #1580 → #1701).
7. Bench roots keep weights.

---

## 6. Honest uncertainty

- **A dose-only fix.** If Step 1 shows off@high ≈ F2@high, lineage was never the chat barrier. The three months were then mostly data and instrument problems. This is plausible, given 0.8% MQAR dose and `rararr` = 1.000 without the shift.
- **The plateau may not move with any recall fix.** It may be capacity or knowledge (G5, H). Nothing here proves the corpus lever works at 29–96M on TinyStories-class models.
- **Canonical j may turn out to be decorative.** Learned W_q/W_k can absorb any fixed orthogonal frame. The cross terms are not zero for distinct learned projections (P, from the reviewers).
- **Step 0 instruments can be wrong.** The panel is judged by qwen2.5:7b, and inherited panels had contradictory duplicate rows (E4). A wrong taxonomy would misroute Steps 4–5.
- **D11 quantization fidelity is unmeasured** for selection-sharp reads, even for F2. Step 3 may fail on fidelity.
- **Coding/reasoning has no milestone this quarter beyond retrieval.** Composition and multi-hop indirection (the "It's {v}" BARE form) may require state or mechanisms none of these proposals supply.
- **Citations.** arXiv:2609.15545 and 2609.16183, and the MQAR-dose figure, are carried from the synthesis and #820. I did not re-check them independently. Line numbers are as of `71b54adf`.

**Key files:**
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/crates/uor-r4-training/src/geometric_stack.rs`, at `:8913-8944`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/crates/uor-r4-training/examples/mqar-bench.rs`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/crates/uor-r4-training/src/milestone_world_v2.rs`, at `:53-55, :1022, :2205-2244`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/crates/uor-r4-training/src/stack_export.rs`, at `:346, :991, :1726`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/crates/uor-r4-integer/src/stack/session.rs`, at `:1433`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/crates/uor-r4-integer/src/stack/kernels.rs`, at `:841`
- `/Users/casey.allard/uor-r4/.worktrees/claude-d11-chat/docs/integration/DECISIONS.md`