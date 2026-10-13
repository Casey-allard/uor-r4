# AGENTS.md — UOR-R4 Geometric Language Model

> **Current operating direction, October 12 — operator phase and training hold.**
> [D25](docs/integration/DECISIONS.md#d25---geometric-operator-phase-and-current-lab-direction)
> records the owner's latest rules: no softmax, softplus, sigmoid or tanh anywhere
> in executed model paths, including objectives, gradients, sampling, teachers,
> comparators and equivalent renamed/table forms; no dense matrix multiplication
> in training or serving; GPU-only model compute, with no host fallback. Offline
> scalar floating-point geometric arithmetic remains permitted on GPU; D11 serving
> remains integer/table based with no floating point or multiplier instruction.
> Kimi is the current project manager; the owner retains strategic authority.
> The [operator plan](docs/labs/operator-phase-2026-10-12/README.md) is the one
> immediate dependency: implement OP2 SpiralCore/FBS projection → OP1 G2 gate → OP3a
> geometric head, then review the complete executed learning/output path before
> any training. G2 is the only OP1 arm; G1 is withdrawn. Geometric feed-forward
> blocks are permitted. OP3b is later research, not a rung0 prerequisite. The
> constraint set is frozen through the first compliant rung0 result; new concerns
> are deferred proposal cards except cost/safety. OP1–OP3 preparation is not
> permission to train a partial trunk.
> **The global training hold and the separate M2 training/rental hold remain.**
> For this Codex projection claim, the owner explicitly authorized development,
> build and finite kernel tests on a CUDA 12.8+ Ubuntu24 pod. This scoped
> permission uses both SpiralCore BFS and Atomic Ternary FBS; it releases no
> training hold or unrelated rental campaign. See the [projection record](docs/labs/spiralcore-fbs-projection-2026-10-12/README.md).
> The owner also authorizes the private HF store for bootstrap fixes, verified
> binaries/toolchains, training models and other needed pod setup material.
> Preserve source identities/checksums and verify round-trips; storage permission
> releases no training hold. See [compute storage](docs/labs/compute.md).
> A FIT review does not lift an owner hold. Local source/build/document work is
> allowed; CPU model/reference execution and new prohibited comparators are not.
> Preserve historical source, measurements, checkpoints and other labs' work.
> Older permissions and dated next actions below cannot reactivate them.
>
> **Owner clarification in this Codex chat:** the candidate mechanisms are E8,
> SpiralCore, icosian operators and FSB routing, with flock/murmuration for
> attention. Both the language-stack and compiler/exact-store/emitter paths
> must learn directly in, carry tokens through and natively consume one geometric
> vector representation through the same writer. Ternary/modulo 256 are possible
> arithmetic substrates, not permission for hidden normalized-exponential or
> dense operations. Preserve exact token/occurrence/version identity separately
> from geometric addresses. The existing 8D geometric block does not freeze the
> model's total width; no 384D MiniLM dependency or new width bar is adopted.

> **Owner direction, October 12: no softmax anywhere ever no matter what.**
> [D23](docs/integration/DECISIONS.md#d23---no-softmax-anywhere-in-the-active-project)
> prohibits softmax in training, serving, sampling, objectives, auxiliary losses,
> teachers and new comparator runs. This includes log-softmax, implicit softmax
> inside cross-entropy, and mathematically equivalent table/renamed implementations.
> There is no training-only, bootstrap or runtime-only exception. Trace the entire
> executed learning/output path before claiming compliance. Preserve historical
> source and measurements as inactive evidence, not permission to execute them.
> Codex's #2246 sampled producer is blocked under this rule. This policy is not a
> claim that existing source is compliant or a comprehensive runtime interlock.
> Existing owner training/pod holds remain; no replacement law is approved here.
>
> **Owner rule, October 12 (same session, stated again) - no CPU model compute,
> and no GPU on work that cannot change a decision ([D24](docs/integration/DECISIONS.md#d24---no-cpu-model-compute-and-no-gpu-on-non-decisive-work)).**
> Model compute never runs on the CPU: training, evaluation and every part of the
> forward that scoring, normalization or the value mix touches are GPU work, and a
> host-side fallback is a defect to fix with a kernel - not a cost to report and
> not mechanism evidence. A configuration these rules already exclude, or a
> comparison whose baseline they forbid, does not get a run. Every prohibited
> softmax site is replaced by a designed geometric mechanism (exact `Z[phi]`/
> zeta-phase interference, typed H4/icosian pairing, Lorentz or address geometry,
> or exact hard selection by address) with a prospective TEST FITNESS review. The
> interlock is `scripts/check_no_softmax.py`; dated `docs/` records keep their text.

> **Owner direction, October 9 — main consolidation and workspace cleanup:**
> `origin/main` is the shared source of truth for every participant. Finish and
> verify the protected merge of each deliverable before starting its successor,
> then clear its redundant local workspace and delivery branches. Branches are
> temporary transport for protected delivery, not alternate research authorities.
> A separate branch or isolated worktree is not mandatory for development; use
> one only where concurrent ownership or protected delivery requires it. Preserve
> unique source, accepted and negative evidence, and other labs’ active work
> before cleanup. Reconcile older unmerged work into main with explicit retained,
> superseded, unfinished or validated status; do not promote it by merging it.
> Work retained rather than activated lands as indexed source patches or a
> restorable source bundle on main. There are no standing branches: `codex/lab-state`
> was retired on 9 October and archived in `docs/history/lab-state-20261009/`.
> **Later October 9 clarification for Codex:** use an owned isolated worktree,
> not Casey's `~/uor-r4` checkout; leave that checkout on clean main while
> preserving owner material. Keep at most one live branch per active PR. Auto-delete is off: after every protected merge, verify
> fresh `origin/main`, then manually remove the GitHub branch, local branch
> and owned worktree. New negative, superseded or paused source that is not
> activated must still land as a patch and INDEX row in
> `docs/history/branch-archive/`. Previously preserved bundles remain evidence.

> **Owner direction, October 9 — delivery cadence:** every PR that completes work
> also carries its canonical record (a dated `docs/labs/<topic>-<date>/README.md` plus a
> newest-first entry in `docs/integration/current-state.md`) and updates every document the
> change makes stale. STATUS/ROADMAP and the #2028 milestone table change when state changes.
> The README is updated only when a capability, headline result, milestone, command or
> mechanism actually changed, in its existing section format, never as a log. Then merge,
> verify, delete the branch and worktree and run `scripts/storage/uor-hygiene --apply`.
> Full checklist: [CONTRIBUTING.md § Delivery cadence](CONTRIBUTING.md#delivery-cadence-every-completed-unit-owner-9-october-2026).

> **Current owner direction, October 1 — D19:** grounded conversation and durable
> memory lead the active canonical plan. Codex rejoins Claude and OpenCode–DeepSeek
> for the owner-authorized continuation; other historical lab assignments are
> not reactivated. The owner's newer exact-head review rule (self-review allowed),
> actual scoped compile/tests and protected merges supersede conflicting older
> council/coordinator delivery prerequisites below. The delivery-evidence check
> is advisory; use the documented manual fallback when deployment is unverified.
> Scientific integrity, D11/D5, unique-material preservation, live job ownership
> and resource admission remain. See [D19](docs/integration/DECISIONS.md#d19--grounded-conversation-and-durable-memory-first).

**Owner clarification, September 19:** D0-b, D1 and D2 in [DECISIONS.md](docs/integration/DECISIONS.md) are owner-approved. D0-b supersedes the old blanket prohibition on additive mathematical linear maps. That September permission is historical: D25 now prohibits dense matrix multiplication in training and serving. The [takeover review](docs/integration/takeover-review-2026-09-19.md) and latest current-state entry reconcile the three model paths and the next bounded task; dated experiment instructions do not override them.

The owner-directed mode is `native_geometric_ai`. Build a learned local language model using the project's prime/zeta/R4 geometry, exact addressed memory and shared typed operators. The objective is useful conversation/memory and coding/reasoning, ultimately frontier capability on consumer M1-class laptops with lower energy and wasted compute. The model remains pre-alpha.

**September 24 learning correction:** [D8](docs/integration/DECISIONS.md#d8--correct-the-training-method-and-reference-ladder) makes the canonical plan's reference/joint-learning/discretization ladder active. A1–A4 are retained scaffolds and negative language evidence; do not resume local selector tuning as the default next step. Use the offline Rust autodiff training tool, verify the actual language-to-context gradient and hard/relaxed bridge, preserve the #1017 language reference and fixed evaluator identities, and complete the declared rung before changing mechanism. A library, integrity check or small fixture is not a language result. Report actual training/evaluation/accelerator work separately from orchestration while retaining the cumulative ledger. The owner retains strategic authority; no transformer serving or new Python model dependency is adopted.

**Owner charter, September 29 — durable autonomous labs (D14).** Any number of labs may join, leave and return. GitHub is authoritative; no provider or session is a permanent director. Claude, Codex, Anti-Gravity and OpenCode/DeepSeek are currently available. Work ownership is a renewable claim, not a fixed monopoly. Follow the [shared protocol](docs/labs/protocol.md), [continuation plan](docs/labs/plan-2026-09-29.md) and [operations/recovery guide](docs/labs/operations.md). A three-seat council with at least two non-author seats and two recorded votes may prospectively correct working policies, interfaces and research direction. The mission, evidence integrity, preservation of unique material and paid spending remain owner boundaries. D11/D5 remain the final serving target. Older fixed lab counts, permanent leadership and provider-specific review mandates are historical. See [D14](docs/integration/DECISIONS.md#d14--durable-autonomous-labs-and-correctable-governance).

## Authority and recovery

Read [README](README.md) → [STATUS](STATUS.md) → [lab entry](docs/labs/README.md) and [current plan](docs/integration/project-track.md) → live tracker [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028) and its milestone issues → relevant [current state](docs/integration/current-state.md) and [decisions](docs/integration/DECISIONS.md) → task source, artifacts and nearest relevant history via the [project map](docs/PROJECT_MAP.md). The [canonical plan](docs/integration/project-track.md) preserves capability responsibilities and the earlier ladders. Current owner instructions and shared policy override dated scheduling. Do not repeat a broad historical audit for each task.

Register and claim work before mutation; heartbeat every five minutes, use twenty-minute leases and checkpoint recoverable source/jobs/artifacts at least every thirty minutes and before quota/compaction/disconnect. Expiry means suspect: verify worker liveness before takeover. Use the admitted runner only after its deployment is verified; adapter/manual/UNVERIFIED status must be explicit. Check live recovery issue #1520 before affected SSD work. Resource admission limits jobs, not the number of labs. Publish completed work and the next dependency on GitHub continuously, using protected PRs and exact-head independent review. README remains a curated research overview, not the running journal.

Refresh `origin/main`, the relevant live issues/PRs, artifact identities and cumulative resource/storage receipts. Choose a workspace that respects current file ownership; an isolated worktree is optional. Preserve the owner's original checkout and all unique research/artifacts. Reuse the [source audit](docs/integration/architecture-2026-09/README.md) and inspect the particular mechanism source; do not repeat the broad audit for routine development. Coordinate independent subtasks with explicit file ownership when useful.

## Native geometric AI agent policy

<!-- agent-execution-policy:start -->
The repository-wide mode is `native_geometric_ai`. Its stable machine contract
is [agent-execution-policy.json](docs/integration/agent-execution-policy.json),
with the readable [execution policy](docs/integration/agent-execution-policy.md).
This section overrides historical process and scheduling rules below. Technical
invariants still apply to the runtime where they are declared.

- Use Rust for data preparation, training, artifact construction and inference.
  Rust is the default for all new code including tooling (D27); the uor-foundation
  sibling crates (uor-addr, UOR-Framework, uor-matmul's lawful
  table/tropical/codec parts, prism, LexLean/PrismPM patterns) are the shared
  ecosystem floor - reuse before rewriting. Python only where Rust genuinely cannot
  do the job.
  Training may use scalar floating-point geometric arithmetic on GPU, but no dense matrix multiplication or prohibited squashing. The training hold remains. Final inference
  executes learned geometric operators through bounded routing, state updates
  and integer/table lookup. A dense transformer hidden behind lookup is not the
  target. Preserve old Python/dense references as evidence; do not add a Python
  model implementation or product dependency.
- Prime addresses and ordered n-lets, fixed zeta-zero phases, R4/S3/H4 state and
  transport, exact `Z[phi]`, chirality/polarity, typed paired-H4/icosian geometry
  and UOR identity are primary mechanisms. Name their implemented roles and
  missing pieces. Architectural priority does not imply measured predictive
  advantage; a failed experiment does not demote the whole architecture.
- Develop both conversation/memory and coding/reasoning toward alpha using the
  same native model path. Keep external research optional and inspect the
  actual source when adopting a mechanism.
- Continue within the owner's authorized objective. A request for the whole
  plan permits its necessary successive tasks; do not stop after one historical
  issue by default. Coordinate independent subtasks, preserve user material and deliver through
  protected pull requests; a separate worktree is optional.
- Configure context/training/evaluation windows and wall-time, RAM, new-storage,
  thread and checkpoint limits for the available machine. Charge cumulative
  work across preparation, training, evaluation, retries and resumes. Diagnose,
  correct and retry within the remaining budget when it can advance the result.
  There is no global 15-minute cutoff or one-retry quota. **Never cancel a
  healthy run mid-way because a self-set budget ran out** (a `max_seconds`,
  step or wall-time estimate, a lease's `--hours`, a planned-token count). If
  the run is still progressing, extend the budget, renew the lease and let it
  finish; record the extension in one line on the owning issue (standing owner
  authorization, owner direction 6 October 2026). The only hard stops are the
  owner caps (≤ 4 pods unless the owner approves more, ≤ $8/h, no new spending
  class), the disk floor and a failed or diverged run.
- Compile and exercise the changed Rust path. Use focused tests for meaningful
  causal, arithmetic, serialization and interface risks and relevant broader
  checks when needed. Do not require a blanket full suite or proof dossier for
  every edit. A queue compatibility acknowledgement is not a test result.
- Keep open development evaluation available during learning and final held-out
  evaluation separate after design selection. Preserve prior results at their
  exact artifact/data/operator/control/budget/decision scope. Distinguish proof,
  measured behavior and hypothesis; `UNAVAILABLE` is not model-quality evidence.

Changes to these stable goals require owner direction and protected delivery.
A task template, stale skill or agent judgment cannot silently change them.
<!-- agent-execution-policy:end -->

**Mechanism briefs (owner request, October 10):**
- Work from [docs/mechanisms/](docs/mechanisms/README.md), which has one page per novel mechanism: the idea, what was tried, and what a fair test needs.
- Every mechanism pre-registration gets a `TEST FITNESS: FIT` review on its milestone before compute.

## Progress control — mandatory before more model compute

**Owner direction, October 10 — a negative closes a configuration, never a mechanism ([D22](docs/integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened)).**
- Stopping a novel mechanism needs an owner decision.
- A tooling or numerical failure is a blocker to fix, not a negative.
- A run that meets its own mechanism target without a significant headline loss is KEEP and iterate, and it resets the count.
- Bars are never narrower than the panel's noise. Decisive runs use ≥ 2 seeds or ≥ 200 fresh rows, and train the mechanism in rather than bolting it on.
- Every development KEEP also reports fresh transfer.
- Five closures are reopened (D22 §5).

**Owner direction, October 9 — three negatives force a pivot ([D21](docs/integration/DECISIONS.md#d21--three-negatives-on-one-line-force-a-pivot-deepseek-trains-the-pointer-fix-codex-stops-the-constraint-line)).**
After three consecutive merged PRs on one line that do not move the owning milestone's
headline acceptance number, the next action on that line is a pivot card on the milestone
issue: stop and archive, or one decisive model-changing run scored on the acceptance panel
with a pre-registered bar. Diagnosis, readings, plans and attribution PRs count toward the
three and never reset it; only a change in the headline number does.
Every lab session follows the [session-goal contract](docs/labs/session-goal.md): the
owner pastes its prompt at session start, and each PR declares its line, count and headline.

Apply the [progress-control rules](docs/integration/agent-execution-policy.md#progress-control--owner-correction-september-25)
and the active contract in [current state](docs/history/current-state-2026-09-25-to-2026-10-02.md#active-execution-contract--durable-labs-september-29).
Keep one short work card in the existing issue: integrated deliverable, observed
blocker, causal change, distinct outcome decisions, fixed conditions, necessary
checks and complete cost. Reject a repeat without new causal evidence and a
decision it can change. Do not turn test failures into a new programme; repair
only failures that invalidate the current deliverable or an applicable contract.
Preserve failed results and accepted models, end validation once its declared
risks are resolved, and advance an independent implementation when a research
branch has no justified next experiment. [D9](docs/integration/DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract)
withdraws the recent64 training follow-up. Training length, evaluation length,
memory access and vector width must be reported separately. All agents, including
RDC/DeepSeek/OpenCode sessions, inherit these rules through this file.

## Research harness — start here, do not reconstruct the rules

**Three labs share this repository, plus this one** — Kimi (project manager), Claude, and Codex
(whose model is Sol). The harness is the shared plane: it records who holds which branch, what each
lab is doing, and what actually ran — across labs, in one place.

**Identify yourself first.** One line, once per session:

```sh
export UOR_LAB=<kimi|claude|codex|deepseek>
```

Without it your receipts are still recorded but attributed to **nobody**, and no other lab can see
who did the work. The harness warns loudly rather than letting that pass silently — an unattributed
record is worse than an absent one, because absence is visible.

**Joining for the first time:** [docs/harness/BOOTSTRAP.md](docs/harness/BOOTSTRAP.md) is a
ten-minute, no-compute check that proves the tool runs on your runtime and that your claims and
presence are visible to the other labs.

Then, across labs rather than within one:

- `python3 scripts/harness/cli.py multilab presence` — what each lab is doing now. Live tier first
  (`~/.local/share/uor-r4/harness/`, real-time, uncommitted), committed record as the fallback; the
  board always says which answered. A lab older than 30 minutes reads **STALE**, and stale is not
  "idle" — it means that lab is working without checking in.
- `python3 scripts/harness/cli.py multilab claims` — who holds which branch, visible to every lab.
  Claim yours with `team claim --branch <name> --lab <lab>` before you start.
- `python3 scripts/harness/cli.py heartbeat -m <milestone>` — the required check-in, which also
  publishes your presence. There is no separate step to forget.

The plan, the milestone headline, the skills, the expert team, the acceptance panels, the
pre-registrations and the lessons from past mistakes are queryable rather than prose to be
re-derived each session. Read [docs/harness/README.md](docs/harness/README.md) once, then use it:

- `python3 scripts/harness/cli.py status` — what is true now: headline, lines, receipts, delivery.
- `python3 scripts/harness/cli.py brief -m m1` — the context for the task, with every source named.
- `python3 scripts/harness/cli.py prep new -m m1` — the pre-registration sheet; get it to
  `TEST FITNESS: FIT` on the milestone before any compute (D22).
- `python3 scripts/harness/cli.py check` — every machine-checkable rule, with the fix for each.
- `python3 scripts/harness/cli.py skill use <id>` / `team dispatch <role>` — one receipt line in
  `docs/harness/ledger/<lab>-<month>.jsonl` (sharded per lab, so concurrent PRs never conflict),
  so the owner and the other labs can see what was actually loaded and deployed.
- `python3 scripts/harness/cli.py mistakes find -f <files>` — what went wrong here before.

The harness is a view over the authorities named in this file, never a replacement for them: the
milestone issues, `DECISIONS.md`, `docs/mechanisms/` and the dated `docs/labs/` records stay
authoritative. It runs no model compute and cannot move a headline number; per D24 no GPU work
may run on it.

## Architecture and claim boundaries

Use the name **UOR-R4 Geometric Language Model**. Technically it is an experimental autoregressive geometric state model with exact addressed memory and learned typed operators. Native attention/context access does not make it a transformer. Historical dense R4/Spin references remain transformers. Do not rename package/CLI/schema identifiers merely to change presentation.

Offline Rust training may use scalar floating-point geometric arithmetic and gradients on GPU, with no dense matrix multiplication or prohibited squashing (D25); training remains held. D25 supersedes the earlier D0-b dense-map stepping stone: native geometric/addressed operators are required; serving remains integer/table/additive with no floating point, transcendental arithmetic or multiplier instruction. Owner decision [D11](docs/integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded) (2026-09-27) supersedes D10: every lab serves under one native contract, ROADMAP §0 R1–R5. Served kernels have no floating point and no multiplier instruction (products of runtime values use tables or exact geometric structure). There is no transformer backbone. Dense per-token weight-map execution is historical evidence, not an active stepping stone under D25. Geometric routing and shared typed operators remain the preferred architecture. These additive maps are still mathematical linear maps; report their dense parameter access and cost accurately. Converted open-weight transformers (D10's SmolLM2) remain inactive historical evidence; no new prohibited comparator or teacher execution. `uor-r4-lut`/`lut-chat` are frozen non-mission comparators, and D10-era results keep their scope. Hidden teacher/provider responses are not adopted. Geometric address/page selection is allowed; the design currently uses shared typed operators. Expert gates are only a conditional future option requiring a demonstrated capability need and full laptop-cost evidence; do not silently adopt MoE or sparse learned gating.

Bounded inference and contextual/copy attention exist. General prose, general reasoning, frontier capability and complete-path energy savings are not established. A successful authored arithmetic or copying panel does not qualify them. Keep candidate admission separate from ranking and harmonic influence. Preserve exact occurrence/version/word identity separately from span boundaries. A prime/hash identity is not a semantic distance. More scalar features cannot recover distinctions erased by representation.

## Execution-lane invariants


- **Native training:** Rust may use scalar floating-point geometric arithmetic and
  gradients on GPU, with no dense matrix multiplication or prohibited squashing,
  to learn geometric read/write/selection and nonlinear operators. Training is held.
  Bind the source/configuration, tokenizer, data and learned parameters to each
  artifact. An offline teacher can be a declared comparator or training source;
  it cannot author serving responses.
- **Native inference:** build toward learned geometric transitions and bounded
  routing/lookup over the primary prime/zeta/R4 representation. Name any
  unfinished operation and its measured cost. Runtime source-model/provider
  access and dense transformer attention/MLP disguised as a lookup are excluded
  from the target. A prototype is not promoted merely because it uses Rust.
- **Typed geometry:** keep R4/S3 compute, Hopf observation, retained fiber and
  torsion, and paired-H4/icosian representation distinct. Preserve exact
  `Z[phi]`, chirality/cosine polarity and artifact-bound zeta identities. Do not
  invent a semantic metric from hash bits or silently drop orientation.
- **Frozen TLA/R4G1 runtime:** its normative kernel remains XOR/AND/OR/shift/
  rotate/popcount/integer add-subtract/compare/table reads, with no multiply,
  divide, float or steady-state allocation. New native training does not weaken
  this existing scoped contract.
- **Artifact determinism:** identical pinned compiler inputs must retain their
  declared artifact determinism. New learned artifacts bind the actual data,
  training seed/configuration, parameters, geometry and format version; do not
  claim bitwise cross-backend training reproducibility without measuring it.
- **Errors:** return `Result` with focused enums at library boundaries; no
  `unwrap`/`expect`/panic on recoverable paths. Preserve `forbid(unsafe_code)`
  in portable runtime and format crates.
- **Claim language:** [formal_vocabulary.md](docs/formal_vocabulary.md) remains
  normative for proof and capability claims. Labels distinguish definitions,
  assumptions, objectives, guarantees and empirical criteria. No blanket proof
  campaign is required to implement or train the next native mechanism.



## Code and documentation ownership

- `crates/uor-r4-core/src/native_geometric/` is the current model implementation; its callers expose native CLI/API/session behavior.
- `crates/uor-r4-router/` includes reusable routing plus historical/exploratory paths; it is not interchangeable with the retained native model.
- Graph format/compiler/runtime/certifier, model-source and proof crates retain their scoped contracts. The `no_std` R4G1 kernel is a separate frozen contract, not blanket language evidence.
- `docs/integration/project-track.md` owns roadmap responsibilities; `current-state.md` owns changing artifact/results/next action. Update claims where asserted, without creating duplicate stage mirrors.
- Per-experiment records and imports are preserved. A dated “next” inside them does not override the live plan. See the [complete README disposition](docs/integration/readme-inventory-2026-09.json).

## Specialist routing and research cadence

**Owner direction, September 26 (historical since September 28; the Codex lab rejoined on 29 September, see the charter above):** the fourth Codex lab operated alongside
Google, OpenCode/DeepSeek/Kimi and Claude, using shared GitHub issues and isolated
worktrees. Its allowance is exhausted, its T4 study transferred to Lab 1, and the three-lab charter above
assigns it no future work. The paragraph and `.codex-lab/` are kept as its record. Read [.codex-lab/README.md](.codex-lab/README.md) for its expert bench,
complete-context task packet, recursive evidence review and cross-lab ownership
protocol. Its whole-project authorization covered successive necessary research
and implementation; old one-milestone stop instructions do not limit that goal.
The canonical plan owns the adaptive attention/inference/prose/chat/reasoning
roadmap. Preserve each lab's active files/jobs, source-bound evidence and all
resource/delivery contracts. A new protocol helper or synthetic benchmark is
not a useful learned conversation result.

**Owner direction, September 25:** use DeepSeek as the default specialist model,
including architecture review, where the configured route is available. Kimi/
Moonshot is reserved for a new explicit owner request. Independent review is an
evidence role, not a requirement to use a more expensive provider. Reuse current
source audits, valid binaries and completed relevant reviews. Choose substantive
implementation and decisive measurements that can change the programme's next
action; add checks only for a concrete unresolved risk. Report complete elapsed
preparation, review and delivery cost alongside model and build time. The model's
acceptance criteria and protected delivery requirements remain in force.

## Resources, verification and delivery

### GPU pods (every session, owner rules of 5 October 2026; Vast.ai default from 13 October, [D26](docs/integration/DECISIONS.md#d26---vastai-is-the-default-provider-for-new-pods))

- Touch GPU pods only through `scripts/pod/uor-pod`. Pods are rented on Vast.ai; Runpod is an emergency backup only: `up --wait` switches to it after 30 min with no Vast offer, when Runpod has stock (logged); choosing it by hand needs `--owner-approved` (owner, 13 October 2026). Never use the CPU where a GPU (CUDA) path can be written, and prefer Rust over Python. Then run `uor-pod status` before any GPU work and read its "Free GPUs you can lease now" line.
- Identity is lab + session: pass `--lab L --session S` (or export `UOR_POD_SESSION`). Lease the GPUs you use (`uor-pod lease POD --lab L --session S --gpus 0,1 --purpose … --hours H`), set `--hours` to the whole job plus margin and renew (`uor-pod renew`) if it runs longer — an expiring lease is never a reason to stop a live job; release when done; leases expire automatically.
- Another session's GPU is never yours, even when idle; never touch another session's lease, files or `KEEP_ALIVE`. Only an expired lease may be taken over, and `lease` checks there is no live job.
- No free GPU and within the caps (≤ 4 running pods, ≤ $8/h, all labs together): `uor-pod up --lab L --session S --purpose … --hours H` (2 × RTX 5090 on the cheapest verified Vast.ai datacenter host; caps count both providers). Never fall back to the laptop CPU for GPU work; caps reached and nothing free: wait for an expiry or ask the owner. **Non-model CPU work runs on the owner's laptop:** source editing, compilation, static/document checks, Rust data preparation and grading stored outputs. Training, evaluation, inference and model/parity references are GPU-only under D24/D25; `train-native-prose` is not a CPU exception. Existing training/rental holds override standing pod admission.

- Without `--gpu`, `up` works down the ladder 5090 → 4090 → PRO 6000 (2 GPUs, then 1). A Vast pod (`vast-<contract>`) has no shared volume: it gets its data from the private HF store `caseyallard/uor-r4-store` (the owner's `hf auth login` token is copied to it) and `down` archives its results to the laptop ([compute.md § Vast.ai](docs/labs/compute.md#vastai-default-provider-13-october-2026)). On Runpod, `up` places pods in the volume datacenters first and then in any allowlisted region; a pod outside EUR-NO-1 is non-canonical ([compute.md § Any region](docs/labs/compute.md#any-region)).
- One job per GPU with `uor-pod run … --gpu K -- CMD`; GPU evaluation uses `device=cuda`.
- Durable output is saved off the pod before it goes: `down` archives a Vast pod's `/workspace/uor-r4/{jobs,pods,runs,evals}` to the laptop, or with `--to-store` to the HF store; `/root/uor-store put DIR results/<lab>/<topic-date>/` saves anything from a pod. Do not keep stopped pods as storage. Release, then `uor-pod down POD` when no other session holds it.
- Local disk is small; `origin/main` (source) and iCloud (results) are the durable stores, so the laptop holds only live work (owner, 9 October 2026). Move results to iCloud as soon as they are used (`~/.local/share/uor-r4/bin/cloud-store put <lab> <dir>`, MD5 round-trip and index; `cloud-store fetch` restores), never keep a second local copy, and put any local-only data under a gitignored path (`/local/`, `/.worktrees/`). After every merge, and whenever free space drops below 30 GB, run `scripts/storage/uor-hygiene --apply`: it removes clean worktrees already in `origin/main` with their branches, idle Rust build folders and local folders already in iCloud (to the Trash), and runs `git gc --auto`; it keeps anything dirty, written within 24 hours, busy or not yet in iCloud. Labs keep no long-lived branches or worktrees. Never put project data, builds or downloads in `/tmp` or `/private/tmp`; use gitignored paths inside your worktree (`/local/`, `target/`), so `uor-hygiene` and worktree removal reclaim them. Keep 30–70 GB free.
- Never read or print API keys, `~/.runpod/config.toml` or `~/.config/vastai/vast_api_key`. Details: [docs/labs/compute.md](docs/labs/compute.md).

Project complete preparation/build/fit/controls/evaluation/retries/checkpoint work before execution: context/data windows, wall time, CPU/threads, peak RAM, new/temporary/retained storage and stop margin. Charge the shared cumulative ledger; an issue or session does not reset it. Training duration is secondary to inference usefulness and efficiency. Self-set wall-time and step budgets are estimates, not stop rules: extend them (and renew leases) rather than cancel a healthy run, and record the extension. Owner caps and machine ceilings still apply; do not incur a new class of external cost. Reuse valid binaries/checkpoints and preserve negative candidates. External GPU compute is authorized only through `uor-pod` (Vast.ai or Runpod, D26) within the caps above (owner, 5 October 2026); no other external or paid compute.

Compile and exercise a changed Rust path with focused checks for real arithmetic, causality, serialization, interfaces and allocation risks. Typical commands use rustup-managed `~/.cargo/bin/cargo`: `cargo fmt --check`, a touched-package offline check and named focused tests. Run actual generated behavior for a model change. Run `python3 scripts/check_claim_wording.py` when editing capability claims. No blanket full suite, proof campaign, ledger/replay framework or corpus run is required for every edit.

Main's ruleset requires five historical status names as compatibility acknowledgements; they execute no formatting, Clippy or tests. Since 3 October 2026 (owner-approved) one CI job, `required-transport`, publishes all five, so a PR or merge-queue entry waits for one shared GitHub runner slot instead of five. Removing the names from ruleset 19597522 altogether is the owner's decision and needs an organization admin. Checks executed at the exact PR head, locally or on the approved GPU pod, carry validation and are posted on the PR before it is queued. Manual native/release QA is available when the changed boundary warrants it. Do not confuse absent fixtures, unrun tests or queue acknowledgements with PASS.

Stage named paths. Push a short-lived branch and deliver through a protected PR, then delete the branch and worktree once it merges (October 9 rule above); never direct-push main, bypass protection, fabricate checks or use admin merge. **Owner clarification, October 7: branches and PRs are temporary delivery steps. Completed, validated work must be merged into `main`; creating a branch, pushing commits, opening a PR or entering the merge queue is not completion. Continue through the protected merge and verify the actual merge commit on fresh `origin/main` plus the delivered source/tree before reporting delivery. Do not leave completed work on an unmerged branch or start its dependent successor while that delivery remains unfinished. If a concrete blocker prevents merge, report the exact blocker and keep the work explicitly pending.** A partial result says `References #N`, not `Closes #N`. Inspect actual merge and source/tree equality when reporting delivery. Update the owning issue and current state with outcome, retained artifact, limitations and next action. Close issues only when their complete acceptance is met; assignment denotes actively working, not future ownership.

## Historical contracts and pitfalls

The [complete previous operating manual](https://github.com/UOR-Foundation/uor-r4/blob/bc03f2d7ffde99608da370808eca542360e54508/AGENTS.md) preserves exact historical experiment limits, frozen release/κ commands and negative findings. Its old stage locks, fixed timers and process sequences are historical, not current authority. Read a scoped historical contract before changing its runtime; do not activate its expensive dormant checks by default.

Preserve `E8 = H4 x H4` as project shorthand for the concrete golden/Galois-coupled icosian construction `H4 ⊕ phi H4`, with fixed basis/glue/maps and inverse witness. The companion is not independent learned state. Hopf S3→S2 observation loses fiber information unless retained explicitly; do not assume a reversible universal S3→S2→S1 pipeline. Fixed finite zeta phases do not require solving classical RH. Geometric structural priority and measured semantic advantage are different claims.

Chain traversal: a rejected or unfollowed link is not proof that a record is absent; prove eviction separately (an overwritten ring slot) before offering an abstention, and cover the cross-product of interacting supported behaviors (for example same-value reassertions with initial/previous/current intents) before qualifying a change. Compare candidates row by row against each prior artifact, not only against the frozen parent or an aggregate score. Freeze independent acceptance criteria before a fresh draw. Create every report directory exclusively (`uor_r4_core::report_output::claim`) immediately after argument validation and before loading any model; seal completed attempts and verify the complete file set (`report_output::verify`); write derived inputs into their own claimed attempt, never beneath a sealed root; never reuse a report root for a retry. Author accepted answers from typed intent once (`uor_r4_core::answer_oracle`) and judge membership only. Run the cheap actual-artifact multi-turn check as soon as a candidate passes construction, before the preservation, comparison and final-fresh campaign.

Missing `/tmp` teacher artifacts make historical parity unavailable, even if a conditional test exits zero. Debug build timing is not optimized serving performance. Shared build caches can retain paths from old worktrees; diagnose the affected crate instead of deleting all caches. Never remove unique research, ignored model parents or intentional dirty checkout material during routine cleanup. Read [the takeover record](docs/integration/handoff-2026-09-07.md) for the latest preserved local paths and budget snapshot.


**Standing owner authorization (2026-09-06):** necessary local model/time/storage allowance extensions are already authorized. Record the complete projection, reason, increment and updated cumulative limit before using each extension; retain cumulative charges and the 128 MiB storage stop margin. Do not ask the owner to approve the same class of necessary increase again. This authorizes no destructive deletion and does not require spending unused allowance. Paid GPU compute is governed by the later, more specific owner rules of 5 October 2026 ("GPU pods" above): `uor-pod up` within the caps is authorized without asking; any other paid or external compute still needs the owner.
