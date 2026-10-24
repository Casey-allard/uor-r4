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

[orig]: ../../archive/integration/DECISIONS-2026-10-19.md
