<!-- Plan of record, approved by the owner 2026-09-29 ~20:05 UTC. Lab 1 (Claude) owns it; the ROADMAP and D13 will absorb it. -->
# Plan of record (2026-09-29): geometry that replaces the transformer's runtime cost, starting with a model that can chat

## Context

It is 29 September, 20:05 UTC.

**The owner asked for:**
1. merge the 7 open PRs after fixing them;
2. honest answers: do we have geometric attention? can the model chat?;
3. an investigative and adversarial council that finds a credible path;
4. rewrite the direction (ROADMAP, README, issues, docs);
5. new autonomous goals for each lab (Gemini/Anti-Gravity, DeepSeek, Kimi);
6. stop the endless build and test loops;
7. clean up local space on the internal drive and the SSD.

**Standing constraints:**
- Rust by default.
- Sonnet for routine agent work, Opus for frontier research.
- Astra/Codex as external auditor.

**The research goal, in the owner's words:** "Use geometry itself to replace transformers that are horribly expensive in runtime, so a frontier-level model can be stored on and executed from a laptop with frontier-model equivalence."
- Context must be malleable.
- Selection rules may come from nature, for example flocks.
- Spherical harmonics is "the big Lie group win".
- **Parity rule:** keep geometric mechanisms that are close to their matmul equivalents.

## Honest answers (evidence: full-history ledger, 15 read mechanisms, 9 chat artifacts)

### Chat: not achieved

- **Best result: R1.** It is S4 arm A + M-world v1, sealed at 13:16.

  | Category | Development phrasings | Trained phrasings |
  |---|---:|---:|
  | Responsive | 0.52 | 0.93 |
  | Instruction | 0.27 | 0.65 |
  | Relation | 0.01 | 0.59 |

  - The 38-request panel scores about 7/38, with 0/10 on memory.
  - The panel NLL guard fails (2.548 → 2.635).
- **The model learned the form of a reply, not retrieval.** "My name is Alex… What is my name?" gets "You're Zoe".

### Geometric attention: no demonstrated advantage

- No geometric read has beaten a matched ordinary control in the main-line model.
- The Lorentz read ties dot (−0.0001).
- 2I and E8 codes used as addresses lose at equal bits (#1438, D6, T2, G v2: 0.29 vs 0.43).

### What works

- **Exact identity** (token or prime keys): KVAR 0.83, D2 1.000, G-binding 0.967, the sieve 273/273.
- **Geometry as structure:** the trained-in 2I transport (+0.011), now served natively by #1506, and the A5 lanes.
- **QAT** on `geometric_s1`: +0.018 nats. Self-reported. The S2 "56/58" figure is only integer-vs-own-served-forward kernel agreement.

### Root causes

1. **No in-context retrieval or copy.**
   - The native learner had pointer-copy and recalled Alex and green; the stack dropped it at D0.
   - M-world v1 drew values from about 10 per relation, so memorizing beat copying.
   - Hybrid literature agrees that attention must be forced to do recall (arXiv 2603.08859, 2510.00258, 2609.04434).
2. **Starvation.**
   - Models saw at most about 3.3 tokens per parameter; compute-optimal is about 20.
   - The 256-token window discards 88% of chat-v0's responses, and repeated fits over the 14,826 that remain raise panel NLL.
3. **Process.**
   - Long serial runs.
   - Instrument churn.
   - The OpenCode harness kills runs longer than about 7 minutes.
   - The ledger was overwritten: labs see 780M ms instead of 820M.
   - Docs sprawl: 1,801 files, a README of dated logs, and issues frozen on the September 7 ladder.

## Owner decisions (recorded; they become D13)

| Topic | Decision |
|---|---|
| §8 instruction panel | New wording of trained instruction types |
| Offline teacher | Local SmolLM2 (135M/360M/1.7B-Instruct on the SSD), used for both data and distillation. It never serves |
| Compute | All local compute, in parallel across CPU and GPU. Outside compute comes later |
| Milestone model | Up to 30M |
| Track B | Starts now, on 135M → 360M → 1.7B |
| Lab control | A GitHub directive-board issue per lab, plus goal files the owner pastes |
| Cleanup | Regenerable caches and clean, merged worktrees may go after a 2-hour notice. Unique data only by the owner's pick |
| External audit | Astra via computer-use in the Codex app, with access requested at execution |
| **Geometric parity rule** | A geometric mechanism stays whenever it is within tolerance of its matmul equivalent (0.02 nats, or 0.03 accuracy). Arms are paired in the same run with 2 or more seeds. One-seed ties decide nothing. The ordinary form replaces it only beyond tolerance, and the geometric form stays in the toolbox |

## Direction: two tracks, four geometric pillars

### Track A — the native geometric chat model (§8 milestone, 1.5–7M, up to 30M once Metal lands)

This track proves the mechanisms end to end under D11.
- **Retrieval first.** Build a retrieval mechanism (pointer or flock) and train it on a curriculum that forces its use.
- **Then knowledge.** Teacher paraphrases, sequence-level and on-policy corrected replies, 20 tokens per parameter.
- **Durable memory** is an exact persisted log of user turns. The AERM store and prime keys become an **index into the log**, not a parallel store.

### Track B — geometric conversion of pretrained transformers (toward frontier equivalence)

- **Method.** Distill SmolLM2 into a geometric runtime:
  - keep the teacher's full-width Q/K/V projections and use layer-wise attention transfer (LoLCATs, Mamba-in-the-Llama, MOHAWK, SUPRA). The #950/#951 precedent failed because it replaced attention with a tiny mixer trained from scratch;
  - keep the teacher's tokenizer, so logit KL is exact.
- **Runtime.** A candle-nn Llama with the `metal` feature serves as both teacher and student. `uor-r4-model-source` is the bit-exact CPU reference.
- **Two levers:**
  - attention replacement, which pays at long context;
  - **E8 lattice weight coding** (QuIP# E8P-style), which pays because weight bytes dominate per-token cost at ≤2k context.
- **Reporting.** Every arm reports the NLL gap to the teacher, bytes and multiply-adds per token at 512, 2k and 8k context, and tokens/s on the M1. Claims stay relative to the teacher.

### The four geometric pillars

| Pillar | Mechanism | Track A | Track B |
|---|---|---|---|
| **Flock attention** (starlings' about 7 nearest neighbors) | Sink + local window + the k nearest keys by Lorentz rank, weighted from a rank table. Minkowski-product ranking needs no arcosh; no exponentials. k=1 is the pointer. Arms: k∈{1,7,16,64}, adaptive k at near-ties (arXiv 2607.07724), rank vs softmax-over-k | The stack's read layers, and the pointer over the log | Replaces teacher attention beyond the window |
| **Spherical-harmonic linear attention** | Gegenbauer / spherical-harmonic features of normalized q and k up to degree L, giving an O(1) recurrent state. Precedent: GM-Net 2605.13262, SLAY. Degree-L harmonics equal polynomial kernels (Based) and cannot make spiky attention (Hedgehog), so they are paired with flock, low-rank plus sparse (Scatterbrain). Example size: d′=16, L=2 gives 152 features per head | Optional | The low-frequency part of each head |
| **Quaternion/2I transport** | The existing S4 transport; later, the recurrent state in 2I irrep blocks (dimensions 1–6, Wigner-D) | The main line | The recurrent carrier for unbounded context |
| **E8/2I lattice codes** | Weight coding at 2–4 bits with table lookups (D11). uor-matmul has an E8 codec; `atlas-embeddings` has exact E8 roots | D11 export | MLP and embedding weights |

**Reuse:**
- The dormant, certified, multiplier-free **top-M route-attention operator** (#604: `uor-r4-graph-runtime/src/route_attention.rs`) and its VP-tree index become flock attention's serving kernel.
- uor-matmul's tropical `gemm_selected` gives exact top-1 selection. Flock needs an added k-selection pass.
- `joint_model`'s copy mixture and its integer copy gate in `uor-r4-integer` port directly to the pointer.

**Context malleability.** The 256 limit is structural and has to be removed at each point:
- a `PrefixPolicy::TruncatedPrefix{keep_last}`, so long responses are kept rather than discarded;
- a log-bucketed age table in the read, replacing the heads×context table;
- a D11 ring buffer plus the exact log, replacing preallocated context×width buffers.

Recurrence carries unbounded state.

## The first 48 hours: parallel tracks (all through the runner)

| ID | Owner | First decisive experiment | Smoke gate | Wall time | Kill criterion |
|---|---|---|---|---|---|
| **A1 Retrieval** | Lab 1 (Sonnet engineer, Opus adversary) | M-world v2 with open value pools: 5k+ names plus random-syllable names per episode, MQAR at distances 16, 64 and 200, and copy tasks. One run on a 1.5M `rrarra` stack with 4 arms: plain; + pointer (flock k=1); + flock k=7; dot-score control on the best arm | 64-example overfit: loss falls ≥50% in 10 minutes | 4 arms × 30 minutes, 2 at a time | No arm reaches held-out recall 0.9: run a transformer control to separate a read defect from a curriculum defect |
| **A2 Context** | Kimi | `TruncatedPrefix`, the log-bucketed age table, the D11 ring buffer | Train at 256, evaluate at 1,024: in-window tokens ≤0.02 nats worse | 6 h engineering, 1 h run | Buckets cost >0.02 at 256: keep exact ages up to 256 and buckets beyond |
| **A3 7M fit** | Lab 1, after the A1 gate | The 7M stack with A1's winner, trained on chat-v0 (truncated prefixes), M-world v2 and teacher paraphrases, 143M tokens. Pre-registered dose-response curve (3 sizes × 2 exposures) | The same binary at 1.5M | ≤20 h, one lane | Development score <0.5 at half the tokens: stop and diagnose. Projection <0.80 at the largest affordable size: rethink the size |
| **B0 Training-free flock** | DeepSeek | Inside SmolLM2-135M, each head keeps sink + window 64 + exact top-k, k∈{1,7,16,64}, rank vs softmax-over-k. 256 held-out windows of 2,048 tokens | The dense path reproduces the reference NLL exactly | ≤2 h | Softmax-over-k at k=64 is >0.10 nats worse: flock goes only through B2 |
| **B1 Geometric index** | DeepSeek | A 2I/E8 cell index over captured keys: recall of the true top-k | Recall@16 ≥0.9 while scanning ≤10% of keys | 3 h | Fails even at 25%: keep an exact scan with (max,+) selection and claim no O(k) cost |
| **B2 Harmonic/hybrid transfer** | Anti-Gravity | First, a candle-nn Llama on Metal matching model-source within 1e-4. Then layer-wise transfer (MSE on attention output; frozen Q/K/V plus LoRA). Arms: harmonic L∈{1,2,3}, d′∈{16,32}, each alone and combined with flock | One layer's operator error falls ≥50% in 500 steps | 8 h engineering, ≤6 h runs | All-layer hybrid at L=2 is >0.15 nats worse: harmonic goes to the toolbox |
| **B3 E8 weights** | Anti-Gravity after B2's engineering (or Kimi) | E8/D4 codec on SmolLM2-360M's MLPs at 2, 3 and 4 bits vs round-to-nearest 4-bit | Exact round trip | ≤3 h | E8 at 3 bits is >0.05 nats worse than RTN 4-bit: record and stop |
| **M Metal port** | Anti-Gravity (after B2) | Metal forward and backward for the 7 CPU-only stack ops; `cpu-accelerate` first | Parity with CPU | ongoing | This is the critical path to 30M (about 14 days on CPU) |

**Track B size ladder:**
- 135M now;
- 360M after the B0 and B2 gates;
- 1.7B or Llama-1B layer-wise only, because of the 16 GB of memory.

**Energy.** J/token needs `powermetrics` (sudo), which is an owner action when a cost report is due.

## Execution steps

### Step 0 — Unblock infrastructure first (Kimi, first ~6 hours; I verify)

**The ledger:**
- `ledger/charges/*.json` and `ledger/extensions/*.json` become the source of truth.
- `model-time.json` becomes a derived view, written only by `lab-runner ledger rebuild` under `std::fs::File::lock` with an atomic rename.
- Reconstruct it now: limit 820M, adding Lab 3's 10:24 charge, R1's charge, and the keys/decode runs.
- Record a **48-hour extension of about 310M ms** under the standing authorization.

**The runner, `tools/lab-runner` (Rust):**
- A LaunchAgent (macOS has no `setsid`), with a queue at `/Volumes/UOR-Workspace/runner/{queue,running,done}`.
- **Job spec:** `{id, lab, cwd, argv, threads, rss_gib, gpu, wall_s, kill_criterion, exclusive}`. Specs without a kill criterion or wall bound are rejected.
- **Admission:** total threads ≤8, total RSS ≤11 GiB including GPU unified memory, one GPU job, `exclusive` for timing runs.
- **Completion:** a job is killed at `wall_s`, then the runner writes `exit.json`, the logs and the peak RSS, and charges the ledger automatically.
- **CLI:** `submit`, `status`, `tail` and `cancel` each return in seconds, so they work inside the 7-minute harness.

### Step 1 — Merge train (in parallel; order #1494 → #1500 → #1503 → #1501 → #1506 → #1490 → #1505)

**Per PR:**
1. Merge `main` in (never rebase shared branches).
2. Run `cargo fmt --check`.
3. Run the touched packages' tests.
4. Run `scripts/check_claim_wording.py`.
5. Get a Sonnet non-author review.
6. Recheck the head, merge through the queue, verify the squash patch, and notify.

**Specific fixes:**
- **#1494:** merge as is.
- **#1500:** relabel `evidence …json:237` ("commit" is actually an executable-hash prefix).
- **#1503:**
  - refresh the body (corpus-3 `0ad2342b…`, 7 tests);
  - note that R1 used v1 with closed value pools;
  - file the 4 oracle regressions for M-world v2.
- **#1501:** apply the wording fixes (`:50` "against the artifact's own f32"; #1500 and #1502 status) and note "superseded by D13".
- **#1506:**
  - accept the 0.21% root mismatch only if every mismatch is a tie at a snap boundary, and list them;
  - accept the shared-format changes;
  - run the quiet-machine cost as a runner job (a follow-up, not a blocker).
- **#1490:**
  - fresh-root re-runs through the runner: `e8-matched-bit-1`, `s2-turn-optimization-2`, QAT B and C;
  - reword "56/58" as kernel agreement and add the float comparison;
  - move results B and C to their own results PR if review prefers.
- **#1505:**
  - `keys-9` (the full pre-registered run, `lexical_below_0_5_at_both_splits = true`) becomes the headline;
  - earlier runs are exploratory;
  - add a result doc and evidence JSON;
  - Lab 1 re-read, then merge as a recorded negative (D12).

### Step 2 — Council (Workflow; 4-hour cap; runs alongside and cannot cancel A1 or B0)

| Stage | Agents | Output |
|---|---|---|
| Evidence | 3 Sonnet internal readers (the ledger, zoology and pointer history, serving and cost) + 3 Opus external researchers | Structured briefs. The external literature covers LoLCATs, Mamba-in-the-Llama/MOHAWK, SUPRA, Based, Hedgehog, Scatterbrain, Memorizing Transformers, top-k/kNN attention, QuIP#, Zoology, GM-Net and harmonic kernels |
| Proposals | 4 Opus architects: Track A retrieval-first; Track B conversion; harmonic-first; knowledge and scale | Each names its first experiment, gate, runtime-cost claim, test cost and kill criterion |
| Red team | Per proposal, 3 Opus skeptics with distinct lenses: "won't generalize or reach teacher equivalence"; "won't serve under D11, or the cost win is illusory in bytes and ops per token"; "endless loop" | Schema `{refuted, reason, killing_experiment}`. A proposal survives if at least 2 of 3 fail to refute it |
| Judge | 1 Opus judge, then 1 completeness critic | D13 and `docs/integration/path-to-chat-2026-09-29.md`. The judge may reorder tracks but cannot cancel running ones |

**Astra audit.** Computer-use in the Codex app: I request access, type the audit packet, and fold the critique back in.

### Step 3 — Direction documents (scripted `git mv`; no deletions)

- **README.md:** ≤120 lines covering:
  - what the project is;
  - the architecture (the four pillars, retrieval, exact memory, D11 serving);
  - current capability in one line;
  - a quick start (build, `uor-chat`);
  - a repository map;
  - links.

  The dated logs move to `docs/history/`.
- **New and replaced files:**
  - **STATUS.md** (new): one page where each lab edits only its row;
  - **ROADMAP.md**: the tracks, gates, kill criteria, owners and the anti-stall rules below, replacing the restart packet;
  - **DECISIONS.md**: gains D13, the owner decisions plus the council verdict.
- **`docs/` layout:** `architecture/`, `results/` (which takes current-state.md, trimmed and corrected), `decisions/`, `labs/` (charter, protocol, runner, machine rules, goal files), `plans/`, and `archive/2026-{08,09}/`.
- **Index:** a generated `docs/INDEX.md` (`cargo xtask docs-index`) and a link check.
- **Issues:**
  - close capability issues [01]–[12] with pointers;
  - open 3 epics (Track A, Track B, Infra) and 4 lab directive-board issues;
  - label #1476.

### Step 4 — Lab goals (goal files for the owner to paste, plus directive issues)

- **Every goal starts with the same "read this first" chain:** README → STATUS → ROADMAP → the lab's directive issue → `docs/labs/`. Each lab spawns its own investigator, adversary and engineer subagents.
- **Lab 1:** the council, D13, the merge-train rulings, A1 → A3, the teacher-data specification, the Track B conversion specification.
- **The DeepSeek lab:** B0 → B1 → the flock arm inside the stack; then close #1505.
- **Anti-Gravity (Gemini):**
  - fix #1490 (split, re-runs, the 56/58 wording);
  - candle Llama on Metal → B2 → B3;
  - the Metal port of the 7 stack ops;
  - the teacher paraphrase generation for Track A.
- **The Kimi lab:** the ledger, the runner, space cleanup, A2, the docs moves, delivery of the merge train, S1.2 (the bundle and reload equality), #1476.

### Step 5 — Space cleanup (Kimi executes; I approve each class)

**Measured about 19:52 UTC:**
- **Internal drive:** 19 GiB free. `~/uor-r4/.worktrees` holds 31 × about 1.2 GB = 35 GB; `.claude/worktrees` 9 GB.
- **SSD:** 35 GiB free. BuildCaches holds 158 GB, about 100 GB of it stale since 09-26/27. `uor-r4-models` 25 GB is unique.

**Rules:**
- **Manifest first:** path, size, class, modification time and `lsof +D`, posted on #820 with a 2-hour notice for lab-owned items.
- **Removal order:**
  1. stale BuildCaches, keeping one active cache per lab;
  2. `target/` directories inside worktrees;
  3. worktrees merged by the merge train, with `git worktree remove` and never `--force`;
  4. `.claude/worktrees`;
  5. the failed `zeros_*.dat` downloads.
- **Never touched:** `uor-r4-models/sources`, `native-joint-learning-*`, sealed roots, anything unpushed.
- **Unique data:** a hash-duplicate report (a small Rust tool) plus zstd archive proposals, as an **owner menu**.
- **Targets:** internal ≥40 GiB, SSD ≥120 GiB, with 30 GiB reserved for Track B traces. Traces are streamed per layer in fp16.

## Anti-stall rules and the lab loop (go into the ROADMAP)

**The lab loop:**
1. Read STATUS and the lab's directive issue.
2. Take the top item.
3. Pre-register it in one issue comment: arms, gate, kill criterion, wall time.
4. Run a smoke of 10 minutes or less.
5. Submit the run to the runner.
6. Engineer the next item while it runs.
7. Seal the result, open a PR, and update the lab's STATUS row.

**Rules:**
1. Hitting the wall-time bound is a result, not a reason to extend.
2. Two consecutive negatives on an item mean escalating to Lab 1 with a one-paragraph diagnosis.
3. A lane idle for more than 30 minutes is a bug.
4. PRs are reviewed within 2 hours and merged within 24, or a blocker is named.
5. Instruments are frozen before the treatment run, and fixes must never draw from the generator.
6. Geometric arms are dropped only under the parity rule.
7. No run longer than 1 hour without a pre-registered decision that it can change.

## Top risks and mitigations

1. **CPU and memory contention** (a teacher and 3 labs on 8 cores and 16 GB). Mitigations: runner admission, the teacher on Metal, and only one long CPU lane (A3).
2. **Track B repeats #951.** Mitigations: full-width projections, a training-free B0 first, fixed kill gaps.
3. **Retrieval passes MQAR but fails English.** Mitigation: A1's gate includes M-world v2 development relations with unseen values and unseen phrasings.
4. **The ledger and records diverge.** Mitigations: a derived ledger and automatic runner charging.
5. **The council and docs rewrite consume the 48 hours.** Mitigations: the 4-hour cap, A1 and B0 starting at hour 0, and a scripted docs move.

## Verification

- **Merge train.**
  - No PRs open.
  - Each squash patch equals its PR diff, noted on the PR.
  - `cargo test -p uor-r4-training --lib` passes on `main` after the last merge.
- **Council.** D13 and the path memo are merged; the red-team tallies and Astra's critique are recorded.
- **Docs.**
  - The README is ≤120 lines with no dated logs.
  - The link check reports 0 broken internal links.
  - `check_claim_wording.py` passes.
- **Issues.** The old issues are closed with pointers, and the 3 epics and 4 lab boards exist.
- **Runner and ledger.**
  - A job longer than 10 minutes submitted from an OpenCode session seals after that session exits.
  - Concurrent charges both land (a unit test).
  - The rebuilt `model-time.json` reads limit 820M plus the extension.
- **Cleanup.** `df -h / /Volumes/UOR-Workspace` before and after meets the targets, with a manifest posted and no unique artifact removed without an owner pick.
- **Research gates** (pre-registered and sealed):
  - A1 smoke;
  - B0 at k=64 within 0.10 nats;
  - B2 single-layer operator error;
  - each track reporting both its quality gap and its bytes and ops per token.

## Update, 20:10 UTC: the Codex lab returns (owner direction)

The owner reports that Codex's allowance has reset. Under owner direction it rejoins as a fifth lab.

**The Codex lab owns Track B's core:**
- the candle-nn Llama on Metal, matching `uor-r4-model-source` within 1e-4;
- **B2**, harmonic and hybrid layer-wise attention transfer on SmolLM2-135M;
- the conversion pipeline that B0 and B3 plug into.

It is also a council member: external research and adversarial audit, alongside Astra.

**Anti-Gravity's list becomes:**
1. fix #1490;
2. **B3**, E8 weight coding;
3. the **Metal port** of the 7 stack ops;
4. teacher paraphrase generation for Track A.

The **ledger** was rebuilt at 20:08 UTC: 782.5M of 1,130M ms. The records are in `charge-2026-09-29-lab1-reconstruct.json` and `extension-2026-09-29-lab1-48h.json`. Until the Kimi lab's locked `ledger` tool lands, no lab writes `model-time.json` without re-reading it immediately before the write, under a lock file.
