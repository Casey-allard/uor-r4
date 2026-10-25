# DECISIONS

Index. On 19 October 2026 the owner adopted [CHARTER.md](../../CHARTER.md), which replaces D0–D41 as the rules (D42). Each entry below keeps its original heading, so every existing `#dNN…` link still lands here, and gives the decision in one sentence and where it now lives: `→ CHARTER.md §x` (kept), `merged into §x`, or `retired (reason)`. The full original text is preserved byte-identical at [archive/integration/DECISIONS-2026-10-19.md][orig]. New decisions are charter changes, recorded in [docs/rules/changes.md](../rules/changes.md); from D43 on, each also gets a row in [the table at the end](#decisions-after-the-charter-from-d43).

---

## D0 — What "no matmul at serving" means

Serving forbids dense matrix multiplication; first statement of the sparsity test, superseded the same day by D0-b.
→ retired (superseded by D0-b; its sparsity test survives as D5, CHARTER.md §3.9). [Full text][orig]

## I1–I5 — Serving efficiency invariants

Serving efficiency invariants: sparse access, bounded candidates, bytes per token, multiplier-free kernels, exact memory.
→ CHARTER.md §3.9. [Full text][orig]

## D0-b — What "no matmul at serving" means, adopted

Additive mathematical linear maps were adopted as a serving stepping stone.
→ retired (superseded by D11 for serving and D25/D30 for dense maps). [Full text][orig]

## D1 — Falsification before extension

Falsify a mechanism before extending it.
→ retired (superseded by D2/D12/D20/D22; a negative closes a configuration, CHARTER.md §4.1). [Full text][orig]

## D2 — How a mechanism's contribution is judged

A mechanism's contribution is judged by measured ablation; enablers are a separate class.
→ merged into §4.1 (enablers survive as PARKED with reopen conditions). [Full text][orig]

## D3 — Takeover clarification and policy synchronization

Takeover clarification synchronizing policy, including an offline matmul permission.
→ retired (superseded by D25/D30; offline matmul permission withdrawn). [Full text][orig]

## D4 — Goal decomposition and the geometry gate

Goal decomposition and the geometry gate: geometric advantage is measured against controls.
→ merged into §4.4 (lawful non-neural controls only, per §3.10). [Full text][orig]

## D5 — Per-token parameter sparsity is the terminal serving invariant

Per-token parameter sparsity is the terminal serving invariant.
→ CHARTER.md §3.9 (D5-b). [Full text][orig]

## D6 — The target objective is a long-range information probe

The target objective is a long-range information probe.
→ merged into §4.4 (long-range probe items plus count-table controls). [Full text][orig]

## D7 — Adopt the integrated attention-language programme

Adopt the integrated attention-language programme.
→ retired (superseded by D8/D19). [Full text][orig]

## D8 — Correct the training method and reference ladder

Correct the training method and the reference/joint-learning/discretization ladder.
→ retired (superseded by D32/D37 for training method and D28 for neural references). [Full text][orig]

## D9 — Prevent experiment loops and preserve the context contract

Prevent experiment loops: a repeat needs new causal evidence; preserve the context contract.
→ merged into §4.4 (pivot note) and §5.3 (intent note replaces the work card). [Full text][orig]

## D10 — A converted open-weight backbone is the interim chat vehicle; serving arithmetic restated

A converted open-weight backbone (SmolLM2) as the interim chat vehicle.
→ retired (superseded by D11). [Full text][orig]

## D11 — Native, multiplier-free, transformerless serving for every lab; D10 superseded

Every lab serves under one native, multiplier-free, transformerless integer contract (ROADMAP §0 R1–R5).
→ CHARTER.md §3.6. [Full text][orig]

## D12 — Gates promote, never kill; reopen geometric candidates; port the native engine's mechanisms; keep a geometric toolbox

Gates promote, never kill; geometric candidates reopen; keep a geometric toolbox.
→ merged into §4.1 (ACTIVE/READY/PARKED, no terminal status; the board keeps PARKED mechanisms documented). [Full text][orig]

## D13 — Record the September 29 two-track owner plan

Record of the 29 September two-track owner plan.
→ retired (superseded by D14/D18/D19). [Full text][orig]

## D14 — Durable autonomous labs and correctable governance

Durable autonomous labs, GitHub as authority, and a three-seat council for working policy.
→ merged into §5.1 (owner boundaries) and §5.2 (GitHub authority); the council is retired by D42 (owner plus one PM seat). [Full text][orig]

## D15 — Converted students may become served candidates after a D11 audit; runtime and energy claims are measured

Converted students may become served candidates after a D11 audit.
→ retired (superseded by D17; excluded by D28/D11). [Full text][orig]

## D16 — Working rules from the 29 September council (council authority under D14)

Working rules from the 29 September council.
→ retired (superseded by D17; council retired by D42). [Full text][orig]

## D17 — Transformer-free conversion, deterministic parity decisions and integration recovery

Transformer-free conversion, deterministic parity decisions with paired seeds and measured tolerance.
→ merged into §4.3. [Full text][orig]

## D18 — One retrieval question for 1–14 October; Track B, cost and memory-port work parked with re-entry conditions

One retrieval question for 1–14 October; other tracks parked with re-entry conditions.
→ retired (window expired; its D5-b re-entry gate travels with D38, §3.9). [Full text][orig]

## D19 — Grounded conversation and durable memory first

Grounded conversation and durable memory lead the plan; exact-head self-review allowed.
→ merged into §1 and §5.5. [Full text][orig]

## D20 — Geometry stays first; the no-loss rule is suspended only after exhaustion, so a measurement can decide

Geometry stays first; the no-loss rule is suspended only after exhaustion.
→ merged into §3.1 and §4.1 (no terminal status, so no suspension procedure is needed). [Full text][orig]

## D21 — Three negatives on one line force a pivot; DeepSeek trains the pointer fix, Codex stops the constraint line

Three non-moving results on one line force a pivot; one-off lab orders of 9 October.
→ merged into §4.4 (pivot note); the lab orders are retired (executed or overtaken). [Full text][orig]

## D22 — A negative closes a configuration, never a mechanism; five closures reopened

A negative closes a configuration, never a mechanism; power and transfer rules; five closures reopened.
→ CHARTER.md §4.1 and §4.3. [Full text][orig]

## D23 - No softmax anywhere in the active project

No softmax anywhere, including objectives, gradients, sampling, teachers and equivalent forms.
→ CHARTER.md §3.2. [Full text][orig]

## D24 - No CPU model compute, and no GPU on non-decisive work

No CPU model compute, and no GPU on work that cannot change a decision.
→ CHARTER.md §3.5, scoped by D42 to research compute; client serving of the D11 artifact is allowed (§3.6). [Full text][orig]

## D25 - Geometric operator phase and current lab direction

Geometric operator phase: squash and matmul bans, GPU-only compute, rule freeze.
→ retired (superseded by D32/D37); its bans live on in §3.2 and §3.3. [Full text][orig]

## D26 - Vast.ai is the default provider for new pods

Vast.ai is the default pod provider; Runpod is the emergency backup; caps of 4 pods and $8/h.
→ CHARTER.md §5.4. [Full text][orig]

## D27 - Rust first, and the uor-foundation ecosystem as the shared floor

Rust first, with the uor-foundation ecosystem as the shared floor.
→ CHARTER.md §3.11. [Full text][orig]

## D28 - No transformer comparisons until the native system works end to end

No transformer or neural comparisons until the native system works end to end.
→ CHARTER.md §3.10. [Full text][orig]

## D29 — No banned mechanism in any compiled or loaded module; vendor-runtime content counts

No banned mechanism may exist in any compiled or loaded module, including vendor runtime content.
→ CHARTER.md §3.4. [Full text][orig]

## D30 — Hard substrate rules: no matmul or softmax anywhere (no wrapper laundering), Rust only, GPU-only heavy compute, pod kernel builds

Hard substrate rules: no matmul or softmax anywhere, Rust only, GPU-only heavy compute, pod kernel builds.
→ merged into §3.2, §3.3, §3.4, §3.5 and §3.11. [Full text][orig]

## D31 — No result is ruled out on an instrument technicality

No result is ruled out on an instrument technicality.
→ CHARTER.md §4.2. [Full text][orig]

## D32 — Discrete geometric substrate: ternary/Hamming/quaternion computation, no floating point in model computation including learning

Discrete ternary/Hamming/quaternion substrate with no float in model computation; zeta-phase exception.
→ merged into §3.7 (discrete path stays active; item 1 float ban revised by D37) §3.8 (zeta exception), §3.12 and §3.1. [Full text][orig]

## D33 — Delegated D4 execution for the rung-0 A2 pilot: Claude runs owner-release after his posted review

Delegated owner-release for the rung-0 A2 pilot.
→ retired (one-off; superseded by D40). [Full text][orig]

## D34 — Revised attention/memory sequence accepted: reranker-first relational read plus a separate D19 object-store crate

Revised attention/memory sequence: reranker-first relational read plus a D19 object-store crate.
→ merged into §5.8 (approved sequence, kept in NOW.md). [Full text][orig]

## D35 — Seed-2 packet re-issue required before any seed-2 GO; registered seed-2 held for the admission iteration

Seed-2 packet re-issue required before any seed-2 GO.
→ retired (superseded by D39). [Full text][orig]

## D36 — GitHub Pages: keep the last good deploy live; deploy job skipped until a D11 served wasm engine lands

GitHub Pages keeps the last good deploy; deploy skipped until a D11 WASM engine lands; legacy crates left broken pending a decision.
→ merged into §2 and §3.6; its "browser serving counts as CPU model compute" clause is replaced, and the legacy crates are repaired (D42). [Full text][orig]

## D37 — Floating point is legal in offline training only; D32's learning-path float ban is revised

Floating point is legal in offline training only, with a reported export gap.
→ CHARTER.md §3.7 (why-bans quote in §1; archived sheets in §4.5). [Full text][orig]

## D38 — D5 split: D5-a residency and D5-b sparsity (owner 2026-10-18)

D5 split into D5-a residency and D5-b per-token sparsity.
→ CHARTER.md §3.9. [Full text][orig]

## D39 — M1 closing bar, K16 reachability gate, post-qualification sequence, and lab self-direction (owner 2026-10-18)

M1 closing bar, K16 reachability gate, post-qualification sequence and lab self-direction.
→ merged into §4.8 (bar (d) replaced by D42), §4.2 (gate), §5.8 (sequence) and §5.3 (self-direction). [Full text][orig]

## D40 — Owner-identity phrases are typed by the owner personally (owner 2026-10-18)

Owner-identity phrases are typed by the owner personally, with `UOR_LAB` unset.
→ CHARTER.md §5.1 (unchanged; the K16 ratification is a retired one-off). [Full text][orig]

## D41 — One-time offline donor corpus exception (owner 2026-10-19)

One-time offline donor corpus exception (`distill-d1`), on its conditions.
→ CHARTER.md §3.10 (sole exception), §5.1 (typed release), sequence in §5.8. [Full text][orig]

## D42 — Charter adopted (19 October 2026)

The owner (Casey) adopted CHARTER.md at checkpoint 1 of the repository renormalization, answering eight questions in chat:

1. Q1 client serving: D24 governs research compute only; serving the D11 integer artifact on client devices (CPU, browser WASM, Apple silicon), including tokens/s and energy measurement, is allowed; D36's browser-as-CPU clause is replaced (§3.5, §3.6).
2. Q2 pre-registration: a one-page sheet with a non-author FIT (a fresh-context subagent counts) only before GPU training or evaluation and before any run a milestone claim rests on; builds, finite kernel tests and host checks need none (§4.5).
3. Q3 releases: one typed owner release per pre-registered sheet, covering its arms, seeds and stop rules; D40 unchanged (§5.1).
4. Q4 records: one notebook page per research line with one row per result; dated evidence folders only for sealed artifacts; NOW.md plus notebooks replace current-state.md; STATUS/ROADMAP change only with state (§5.6).
5. Q5 coordination: owner plus one PM seat (Kimi, with an owner-named cover lab); labs self-direct from NOW.md; D14 council retired; only the owner changes the charter; PM operating notes may not contradict it (§5.2).
6. Q6 M1 bar: (d) replaced by integer top-1 through the D11 export beating the unigram (1,226/14,287) and T2 (6,300/14,287) tables on the same DEV panel, two seeds, tokens/s reported; BPB returns later; (a), (b), (c), (e) unchanged (§4.8).
7. Q7 harness: shared view with automatic presence and no manual heartbeat cadence; merge queue serializes itself; `adopt_policy` hold dropped; claims record the charter version; discretionary harness features frozen until a model beats both count tables; D27 ports continue (§5.3, §5.5).
8. Q8 legacy crates: `uor-r4-graph-compiler`, `uor-r4-graph-certify`, `uor-r4-api`, `uor-r4-proof-model` and root `src/` are repaired on the current core, not archived; r4g1-graph and native-geometric stay PARKED (§2.4).

The same session approved the renormalization decisions: archive byte-identical with nothing deleted; `projects/` for side projects; GitHub markdown plus an mdBook site with the Studio at `/studio`; web client is a local in-browser WASM integer model; desktop client is a Tauri app sharing the web UI; D22 kept with ACTIVE/READY/PARKED and no terminal state.

## Decisions after the charter (from D43)

From D43 on, each decision gets one row here: what was decided, the reason, the alternatives turned down, and what it would cost to undo. This table is append-only. An old row is never edited; a later row supersedes it and says so. The owner's words and the full reasoning are in the dated entry in [docs/rules/changes.md](../rules/changes.md).

| Id | Date | Decision | Reason | Alternatives turned down | Cost to undo |
|---|---|---|---|---|---|
| D43 | 2026-10-20 | Every lab and subagent works in the loop adapted from Ron Brazil's Loop Kit (CHARTER §5.10). Records drifting is lesson L-001. | All four failure modes had happened, and records drifting cost the most. | Keeping the old process with more written rules. | Medium: drop §5.10 and the loop tooling. |
| D44 | 2026-10-20 | The kit's text stays in local skills folders only. The repository holds our own words, crediting the kit. A local check guards this, and no local snapshot is taken. | The kit is not for redistribution and the repository is public. GitHub, the archive and the HF store already cover backup. | Publishing the kit; hashes of its text in CI; the kit's snapshot step. | Low. |
| D45 | 2026-10-20 | Labs run continuously, one unit of work per fresh session, until an owner decision, held work or an empty queue. | Autonomy toward the research goal, with no single session growing until it drifts. | One task, then waiting for a go. | Low: turn the supervisor off. |
| D46 | 2026-10-20 | A code or model PR needs an exact-head review from a different vendor's lab. Doc-only and generated PRs may use a fresh-context self-review (§5.5). | Rules were broken quietly under same-lab review. | Same-lab self-review for everything; PM approval of every merge. | Low. |
| D47 | 2026-10-20 | The kit's note names are used, with STATE and HANDOFF rendered by the harness. `NOW.md` and `STATUS.md` fold in. Each lab keeps its own PICKUP, ORDER and REPORT. | Hand-kept shared status drifted, and five labs cannot safely edit one file. | Hand-written notes; keeping `NOW.md` and `STATUS.md`. | Medium: restore the archived pages. |
| D48 | 2026-10-20 | No line cap on PRs. Size is reported and scope is enforced. | A cap would block honest research PRs; bookkeeping matters more. | The kit's line budget, with oversized changes rejected unread. | Low. |
| D49 | 2026-10-20 | No agent-usage budget. The GPU cap is unchanged. | The owner's choice. | A weekly budget per lab. | Low. |
| D50 | 2026-10-20 | Subagents inherit their parent's order, with the same or a narrower scope, at any depth. They never claim, open or merge a PR, post as the lab or decide. | The same rules must reach every level. | Depth limits; read-only subagents. | Low. |
| D51 | 2026-10-20 | The harness freeze in §5.3 is lifted entirely. | The loop harness is the next priority. | Lifting it for the loop only. | Low: restore the sentence. |
| D52 | 2026-10-20 | All charter edits for the loop are approved: labs can no longer lift the GPU caps, the owner is the last fallback when the PM and its cover lab are both offline, and the owner's kill switch stops every lab. Teams: Claude leads architecture with Kimi reviewing; Codex and DeepSeek build the automation; Antigravity builds the client adapters. Amended 21 Oct by the owner: while Claude runs alone, Claude builds every loop setup PR, and each other lab's client adapter is added when that lab joins. | Cross-vendor work from the first PR. Amendment: only Claude runs at first. | Approving only part of the edits; one pooled team. In the amendment: leaving the setup items to labs that are not running. | Low. |
| D53 | 2026-10-20 | The 2-hour task limit is a soft check-in. Real progress continues, a spiraled or stalled session closes, and the PM prompts any lab that stopped. | The owner's words in the change log. | A hard 2-hour stop. | Low. |
| D54 | 2026-10-20 | The policy gate stays advisory on GitHub and is enforced locally at queue time. | There is no org admin to change the ruleset or add a GitHub App. | A required check pinned to a GitHub App. | Low: require it once an admin can. |
| D55 | 2026-10-20 | Lab identity is shown by tags on claims, PRs, reviews and comments. There are no signing keys. | Each lab is one vendor session on the owner's laptop. | Per-lab keys; a GitHub user or macOS user per lab. | Low. |
| D56 | 2026-10-20 | An owner release is approved by the owner in the asking lab's own chat and recorded verbatim with the time. The typed phrase is retired (§5.1). | The owner's choice. | The typed phrase; a hardware signing key. | Low. |
| D57 | 2026-10-20 | Private client memories keep only pointers to the rules, and Codex memories are off for this repository. | Private copies of rules drifted; one already contradicted D46. | Turning all memories off; leaving them as they were. | Low. |
| D58 | 2026-10-20 | Codex keeps full sandbox access. | The owner's choice. | Workspace-write with network. | Low. |
| D59 | 2026-10-21 | Until the other labs are running the loop, the loop's setup PRs get fresh-context reviews from the authoring lab, never from the other labs. This is an exception to D46 for the setup only. Amended the same day by the owner: whichever lab built a setup PR, its exact-head review comes from Claude's fresh-context reviewers, because Claude leads the architecture (D52). | A lab that does not run the loop yet cannot meaningfully review the setup that installs it. | Asking Kimi or another lab to review the setup. In the amendment: each lab reviewing only its own setup PRs. | Low: it ends when the other labs run the loop. **Ended by the owner on 2026-10-22** ("End D59 now", #2028 comment 6342668364): setup PRs take any other-vendor fresh-context review, like every PR. The switch is `docs/harness/d59.json`. |
| D60 | 2026-10-23 | The D11 finite probe is complete (3/3 at `127632bc3` and independently confirmed), but the premise stays held until the registered two-seed M1 closing bar. | Repeating the finite command adds no information and cannot establish language capability. | `closed` (unsupported); `open` with repeated no-new-info probes. | Low: reopen when the closing-bar sheet and owner release are ready. |
| D61 | 2026-10-24 | The position finite-control premise probe is `closed` (10/10 at `b03d08e85`); the registered paired order test remains pending, with no order-effect or language claim. | The finite probe ran and repeating it adds no information; its exact controls do not answer the registered model experiment. | `open` with repeated probes; `held` despite the finite command being answered; any mechanism-complete claim. | Low: reopen if the result or coverage is shown invalid. |
| D62 | 2026-10-24 | The normalization finite-control premise probe is `closed` (13/13 at `5844d911c`); the registered N_snap versus unscaled/raw trunk test remains pending, with no trunk or language claim. | The finite controls answered the probe and repetition adds no information; they do not answer the registered model experiment. | `open` with repeated probes; `held` despite the finite command being answered; any mechanism-complete claim. | Low: reopen if the result or coverage is shown invalid. |
| D63 | 2026-10-24 | The exact-memory arithmetic premise probe is `closed` (4/4 at `9aaaf13bb`); the address-as-read oracle still needs its held checkpoint, fresh-context FIT, PM dispatch and owner approval, with no durable-conversation claim. | Exact arithmetic is answered and repetition adds no information; it does not measure addressed dialogue recall. | `open` with repeated probes; `held` despite the finite command being answered; any durable-conversation or oracle claim. | Low: reopen if the result or coverage is shown invalid. |
| D64 | 2026-10-24 | The reasoning-ablation instrument premise probe is `closed` (3/3 at `9aaaf13bb`); the frozen-panel factorial-drop versus memorized-template-flatness question remains pending, with no reasoning claim. | The four-arm instrument's finite controls are answered and repetition adds no information; they do not answer the panel's causal question. | `open` with repeated probes; `held` despite the finite command being answered; any reasoning-capability claim. | Low: reopen if the result or coverage is shown invalid. |
| D65 | 2026-10-24 | The single-lab rule: every step is one a single lab can finish alone. Other labs and the owner can speed work up or reverse it, never hold it, and no step posts a question and waits for the answer. The owner: "if there was only one single lab running what would the process be? a single lab can't park and wait for another lab or me to review"; "Posting and waiting for an answer is where we lose." An ask to the PM that goes unanswered becomes the asking lab's own decision review (D68). | From 23 Oct 14:10Z to 24 Oct 13:43Z the labs merged 70 PRs and closed no issue. Then every lab stopped behind posted asks nobody answered: a repair whose author believed it was waiting for review, PM asks, and approvals waiting in one lab's own chat. | A 30-minute review fallback (rejected by the owner); parking held work until a release. | Medium: restore the waits in CHARTER §5.1, §5.2, §5.5 and §5.10. |
| D66 | 2026-10-24 | Review is part of the unit. Before a PR goes ready, its lab runs a fresh-context adversarial review that has seen only the item chain, the ORDER and the diff, never the plan, posts the `uor-review v1` record and queues the PR in the same session. Other labs review after the merge: with nothing of higher unblock value, a lab audits recent merges by other vendors, and a finding becomes a repair item any lab can take. Replaces D46 (§5.5). | The owner: "Reviewers waiting 30 minutes for other labs to see it is the entire problem." A pre-merge review by another vendor turns every PR into a wait on a lab that may not be running. | Cross-vendor review before the queue (D46); the same with a 30-minute timeout. | Low: restore D46 in §5.5 and in the review gate. |
| D67 | 2026-10-24 | A GPU or training run, held work included (Training, M2, Stage B), goes ahead when the lab's own pre-run review passes, inside the GPU caps, the run that unblocks the most items first. The adversarial reviewer must show the whole-project chain the run serves and why it is the right test now; that the mechanism is finished and every tool, input and checkpoint exists in the repository or the HF store; what a pass and a fail each unlock, refusing a run whose result would change nothing; the sheet's FIT; and a lease inside the caps. Replaces the owner release (§5.1, D40's release, D56). | The owner: "i never see these requests for released, so I would say do not park any work at all for any reason" and "We don't want to be doing runs if they are not going to unlock further work, like when we finish a mechanism and are simply waiting on the gpu training or something to happen which unlocks further work, we want to do those as highest priority. It's all about removing blockers at this point." He answered "Accurate (Recommended)" to a read-back whose option read "The current HELD items (training, M2, Stage B, D40) become runnable through the pre-run review, within caps." | Parking held items until a release; keeping the named holds while lifting the rest. | Medium: restore the release in §5.1 and the holds. |
| D68 | 2026-10-24 | Bars, statuses, scope fixes, classifications and stop rules are decided by the lab: a decision card with its recommendation, approved by the lab's top session only after its own fresh-context adversarial decision review passes. The review reads the charter, the milestone chain and earlier decisions before it approves, and refuses a card without that context. Replaces decision cards to the PM or owner (§5.2). | The owner: "make sure before the decisions are made that the entire project is understood and that we understand from a whole project context what this step is for and what it is achieving and that it makes sense to do it - do not approve without larger context". | Cards that wait on the PM or the owner. | Low. |
| D69 | 2026-10-24 | The owner's queue is a log, not a gate. One place lists every run, decision and in-unit review made on the owner's behalf, oldest first, each with its record and open to reversal. The owner sets only the charter, §3 (with its exceptions) and the GPU caps; a lab that needs one of them changed proposes it in the queue and goes on with other work. Until the queue view is built, each record is posted on its item's issue starting `[<lab>] owner log:`, an interim form Claude chose to carry out "One queue". Ends the asks in a lab's own chat (D56). | The owner chose "One queue (Recommended)" and said "there should not be a point where anything is waiting on my decision". On 24 Oct his pending approvals sat unseen in one lab's chat while three labs waited on them. | Asks in each lab's own chat (D56); keeping D56 with a list on the bench. | Low. |
| D70 | 2026-10-24 | The PR whose unit meets its item's "done when" writes `Closes #N` and sets the item to `done` in the same PR, and its review checks the claim (§5.5). | The owner chose "The finishing PR (Recommended)". 70 merges on 23–24 Oct closed no issue, because `loop close` writes only `References #N` and closing took a separate PM record PR. | PM-only record PRs with a time limit; the owner closing every issue. | Low. |
| D71 | 2026-10-24 | A supervisor that survives restarts starts a headless session for any idle lab that owns the next move, on clients that can run headless, and shows the other clients as idle on the bench. A session ends on `loop next`, never on a wait (§5.10). | The owner chose "Yes, where headless (Recommended)". When the owner restarted the laptop at 13:58Z on 24 Oct, every lab stopped and nothing brought them back. | Alerts only; Claude only. | Low: unload the supervisor. |
| D72 | 2026-10-24 | D11 integer serving (UORLUT01) moves from ACTIVE to READY until a lawful trained model is exported through it. Hopf observation splits in two on the board: the kept-fibre map is READY, and sector routing stays PARKED with its reopen condition. | The owner chose "D11 READY, split Hopf (Recommended)" in Claude's chat at 2026-10-24T03:41:41Z (#2874). The engine and its zero-multiply audits are green, but no model built under today's rules has been exported through it. The kept-fibre map is built and unit-tested and its language test never ran; only sector routing failed. | D11 stays ACTIVE as the end-of-pipeline contract; Hopf observation stays one PARKED row. | Low: a board and page edit. |
| D73 | 2026-10-24 | A model trained in float offline (D37) counts toward M1, M2 and the product once it is exported to the D11 integer artifact and scored there, with the export gap reported. A discrete-trained model also counts. Claude's own decision under D68, not the owner's: architect dispositions are advisory, and no status is terminal (CHARTER §4.1, D22), so an architect RETIRE reads as PARKED with its reopen condition. | The owner chose "Both counts once exported and discrete-trained" in Claude's chat at 2026-10-24T03:41:41Z (#2875). The ruling does not say how architect dispositions meet the READY/PARKED statuses, so Claude decided that part under D68 with a fresh-context decision review. | Only a discrete-trained model counts (D39 item 1 as written); a float model counts without export; an architect RETIRE ends a mechanism. | Medium: milestone claims may come to rest on exported float models. |
| D74 | 2026-10-24 | Spin-native weights and ternary/XOR arithmetic move from PARKED to READY. | The owner chose "Keep both READY" in Claude's chat at 2026-10-24T03:41:41Z (#2876). Their caveats stand: spin-native weights reached 1.5148 BPB only through a banned dense Hamilton-block multiply, and a dense ternary sum is still dense (D25). | Keep both PARKED; build a geometric alphabet first. | Low: a board and page edit. |
| D76 | 2026-10-25 | Merge gates keep only what guards the project. `loop pr-check` and `loop queue` print six bookkeeping rules as advisory WARN: scope, commit order, rule sources, rendered-file lock, read path and FILEMAP. Order, banned presence, REPORT, owner paths, the review record and the required checks still fail a PR. Banned presence now also scans added files: before D76 it skipped them, so a new file could bring banned code in unseen. The docs workflow's link, claim-wording and catalog/figure steps become advisory, and archive verify stays hard. The September `relative-query-check` workflow is deleted. The ruleset drops "branch must be up to date" and the queue's minimum wait. | Casey, 24 Oct: "please dont let us get locked down in CI hell, if something doesn't matter to the project it can be removed or skipped". Then, by 2026-10-25T00:03:59Z, he chose "Yes, change them (Recommended)". Since 21 Oct these rules have repeatedly cost a unit an extra PR, and 25 of the last 30 CI failures were documentation or records gates. | Keep all twelve rules hard; drop the five acknowledgements, which would remove the CI policy verdict `loop queue` requires. | Low: records may drift, and the findings still print on every PR. |

[orig]: ../../archive/integration/DECISIONS-2026-10-19.md
