I have everything I need. Writing the audit report.

# Front-Door Audit — UOR-R4, 18 October 2026

## Verdict

**Overall: B− for the README as a record, C for the README as a front door.** The 18 October rewrite is real and current — D37 is correctly reflected, the milestone table is honest, every relative link in the eight front-door docs resolves, and the hard negatives ("no model holds a useful conversation", "0 of 8") survive editing pressure. That is better than most research repos.

The failure is **ordering and audience**, not truth. The front door is written in the order the project lived its work — pod IDs, batch counts, waiver comment IDs — rather than the order a researcher arrives: what is this, does anything work, what would I do here. The GitHub surface outside the files (description, topics, social preview, Pages) is effectively unset, so the first thing an outsider sees is a bare repo card.

**Top-line finding:** the README's first 32 lines contain roughly twenty undefined terms (`rows_v1`, Dose2, Q1, P_succ, K16-A/R, D-Packet, Q4) and a pod shutdown timestamp — 298 words of operator log before the reader learns what "geometry" means here. The project's own rule (`docs/bookkeeping.md:33-35`, "No journal") is violated on the front page.

---

## 1. README.md — section by section

| Section | Lines | Grade | Finding |
|---|---|---|---|
| Title + question + badges | 1-11 | A | Clear, memorable question. Badges accurate; HF links return 200. |
| Status paragraph | 13-32 | **D** | Operator log in the hero slot. Typos, undefined jargon, pod ID and UTC timestamps. |
| The question | 41-55 | A− | Best writing in the file. "Geometry means concrete, exact structures" earns the abstraction. |
| Architecture | 57-88 | B+ | Accurate to `docs/architecture.md`; figure is good (see §4) but has no carry/state element despite the prose. |
| Where we are | 90-109 | B− | Honest table; contradicts ROADMAP on M2/M3; M7 is stale against today's #2654. |
| Current path | 111-134 | B | Good scoped table; dense but defensible. |
| Historical operator phase | 136-160 | B | Correctly collapsed and labelled `dormant`; badge semantics in the figure still say "BUILT & TESTED". |
| Rules | 162-184 | B | Correct D11/D22-D34/D37. **D35 and D36 are missing**, and the table jumps D34→D37 with no explanation. |
| Mechanisms under test | 186-211 | B− | Status vocabulary is excellent; 16 rows of 60-word cells is a wall. |
| Results so far | 213-259 | A− | Exemplary scoping for a public page. |
| Bookkeeping | 261-274 | B | `D0–D34` is stale (D37 exists). |
| How the project is run | 276-290 | A− | Clear, short, links to real procedure. |
| Quick start | 292-310 | C+ | Builds and tests only; no way to see the model do anything, and no statement that none exists. |
| Models and data | 312-344 | A− | Honest; "historical" labelling is correct. |
| Repository map | 346-373 | C | Stats block is wrong (below). |
| Acknowledgements | 375-390 | A | Proper credit, including today's WASM contributor. |

### Specific defects with evidence

**1.1 Typos / broken spacing (mechanical, fix first)**
- `README.md:28` — "seeds1/2"
- `README.md:30` — "cells with100 samples"
- `README.md:32` — "waiver6251027125" (link target is fine; link text is malformed)
- `README.md:304` — "Rust1.97.1"
- `STATUS.md:10` "exits143", `:22` "selectedbccc", `:23` "failedE0063", `:84-86` "with30/30", "takes37.84", "of17.8/19.2hours", `:35` "waiver6251027125"
- `ROADMAP.md:8` "and9-control", `:11` "at23d5", `:15` "under6246440910", `:75` "registered4,096"
- `docs/PROJECT_MAP.md:261` — heading "September23" (missing space)

These read as a systematic formatting bug (number/word concatenation), not individual slips. Worth one sweep commit.

**1.2 The status paragraph is a lab log** — `README.md:13-32`
Pod `56510253` down at 07:53:30 UTC, `P_succ` 2,034→225/205, "Q4 timing waiver…comment id" — none of it helps an outsider decide whether to care. It also duplicates `STATUS.md` verbatim in substance. Suggested replacement (~110 words, keeps every claim):

> **Pre-alpha, 18 October 2026.** We are building a local language model whose state, memory and next-token choice are geometric instead of a transformer's dense attention. **0 of 8 milestones are complete, and no model here holds a useful conversation yet.** What exists is a working integer substrate — a shared token-vector table, ordered context, Route/Flock/Relation reads and a shared writer in [`crates/uor-r4-discrete`](crates/uor-r4-discrete/README.md) — plus a scoped diagnostic pilot that learned 236 vector updates. The latest paired attention attempt stopped at its pre-registered Q1 admission gate in both arms: a configured admission collapse, not an accuracy result or a mechanism verdict ([record](docs/labs/attention-rows-q1-diagnosis-2026-10-18/README.md)). The immediate priority is qualifying the FIT'd K16-A/R redirect before any further training.

Move pod IDs, waiver IDs and timestamps to `STATUS.md`, where they already live.

**1.3 Undefined jargon used as if universal**
`R14` (`README.md:97`) is defined only in `docs/harness/pm-rules.md:319`; `rows_v1`, `Q1`, `D-Packet`, `K16-A/R` are never defined anywhere a newcomer would look. First use should expand: "the registered Q1 admission gate (the point where the proposal path must admit candidates)".

**1.4 Stale facts**
- `README.md:266` — "Owner decisions D0–D34" → D0–D37 (D35, D36, D37 all exist; `docs/integration/DECISIONS.md:1672,1712,1740`).
- `README.md:371` — claims "8,131 `#[test]` functions". Actual count across `crates/` + `xtask/`: **6,452**. Claims "14 open issues"; live count is **10**. Claims "D0 to D27". (Commit count 1,990 is unverifiable here — this clone is shallow.)
- `README.md:103` — M7 "not started … local API only". **#2654 merged today** (commit `966c28519`) delivers a D11 integer WASM serving runtime with Node round-trip verification and `docs/wasm-serving/`. The milestone acceptance is still unmet, but "not started" is now false. Suggested cell: *"component delivered | API and Studio remain local-only; a D11 WASM runtime for the historical integer stack merged in #2654, but the Pages deploy stays paused (D36) until it can serve the compliant model."*

**1.5 D37 compliance — correct.** `README.md:36-39`, `:118-119` and `:180` all state offline-float-legal / matmul and softmax banned / serving separately qualified. This is the current rule. No page I checked still asserts a total float ban **except `ROADMAP.md`** (see §2).

**1.6 Claim language — compliant.** No prohibited phrases ("machine-verified", "exact equivalence", "provably equivalent") in README, STATUS, CONTRIBUTING, ROADMAP, docs/README, docs/architecture. "0 of 8", "no model holds a useful conversation", "architectural priority is not a measured advantage" all match `docs/formal_vocabulary.md` §1-2.

---

## 2. Support docs — navigation coherence

| Page | Grade | Finding |
|---|---|---|
| `docs/architecture.md` | **A** | The model page: 53 lines, one token followed end-to-end, every arrow traced to a source file, cost honesty at `:30-34`. Other pages should copy this shape. |
| `CONTRIBUTING.md` | B+ | Workflow is clear and actionable. Two real problems (§2.2). |
| `STATUS.md` | B | Fit for purpose as a status record; not newcomer-facing, and doesn't claim to be. |
| `ROADMAP.md` | **D** | Stale D32 float ban; 97-line log before the milestone table; milestone statuses contradict README. |
| `docs/README.md` | **D** | 463 of 543 lines are a pre-D23 historical programme. |
| `docs/PROJECT_MAP.md` | **D** | 240 of 263 lines are September archaeology, including a "Current navigation, September 25 UTC" heading. |

**2.1 `ROADMAP.md:34-35` contradicts D37 today.** It states: *"Current learning permission — D32: discrete/exact GPU model computation and instruments only; no floating-point forward, gradient, optimizer or matrix product."* That is the superseded rule. D37 (`DECISIONS.md:1740`) explicitly revises D32 items 1 and 5. Any outsider who reads the roadmap instead of the README gets the wrong constraint set. Replace with: *"Current learning permission — D37: offline training may use float forward pass, gradients and optimizer in a separately scanned geometric module; matmul/GEMM and softmax/squashing remain banned everywhere; the active discrete path stays integer and serving is a separately qualified D11 export."*

**2.2 Milestone statuses disagree across pages.** README (`:98-99`) and `milestones.svg` say M2 **held**, M3 **paused**. `ROADMAP.md:144-145` says M2 **"in progress; training/rental held"**, M3 **"in progress"**. Pick one vocabulary and use it in all three places; the README/figure version is the honest one.

**2.3 `CONTRIBUTING.md:118-122` names a retired tool.** It instructs `cli.py skill use <id>` and `cli.py team dispatch <role>`, then says `uor-harness` writes the line. `AGENTS.md` states the Rust harness is the only implementation and the Python scripts are dated evidence. Change to `uor-harness skill use <id>` / `uor-harness team dispatch <role>` / `uor-harness mistakes capture`.

**2.4 `CONTRIBUTING.md:29-32` suggests unrealistic first tasks.** "Good first areas: M6 (exact arithmetic), M5 (energy plan), M7 (API/WASM)" — M6 is the project's hardest open research problem (README `:102` calls exact arithmetic "the visible gap"). For an outside researcher, a realistic first contribution is a scoped negative reproduction, a claim-wording fix, or a figure. Also add the honest line: *"There is no runnable chat demo yet; the quick start builds and tests the substrate. The closest thing to observable behaviour today is reproducing a recorded count on the pinned DEV set."*

**2.5 `docs/README.md`** — the top 37 lines are good and current (D37 at `:23-27`). Then `:38-501` is a pre-D23 programme narrative that reads as the research map, including `:69-71` "Ordinary softmax/retained attention remain established" and `:462` a `[PASS]` for `R4SoftmaxReferenceGeneratorV1`. It is honestly banner-labelled "Historical", but it is 85% of the page a newcomer is told is the documentation hub. **Move `:38-501` to `docs/history/research-map-2026-10.md`** and leave one line + link. Keep `:502-543` (how to read claims, maintenance) where it is.

**2.6 `docs/PROJECT_MAP.md`** — keep `:5-22` (current compiled navigation) and `:80-94` ("Read the authorities in this order", which is genuinely useful). Move `:24-79` and `:96-263` to `docs/history/source-navigation-2026-09.md`. The page currently opens with the right answer and then buries it under three weeks of superseded navigation.

**2.7 `docs/geometry.md:10-21`** repeats the README status paragraph at the top of the page the README recommends **first** (`README.md:34`). If a reader follows the one clear instruction on the front page, they hit the same wall. Lead with the quaternion/600-cell material; keep status to one sentence linking STATUS.

**2.8 `docs/bookkeeping.md:3-4`** is still marked "**Proposed** … for the PM (Kimi) to adopt", yet `README.md:274` cites it as how the README is kept current. Either adopt it or stop citing it as authority. Its `:21` "README current path table → the operator plan" is itself stale (that plan is the dormant float phase).

---

## 3. Figures — file by file

All SVGs carry `role="img"` with `<title>`/`<desc>` — good accessibility hygiene. I rasterised and viewed each at native size.

| Figure | Renders | Accurate to current state | Quality / notes |
|---|---|---|---|
| `docs/figures/architecture.svg` (1200×660) | Yes | Mostly | **A−**. Clean, professional, correct counts (8D/16 roots/64 actions) matching `docs/architecture.md:25-28`. Three issues: (a) **no state/carry element**, though README `:16` and `:112` say "input/**carry**/output writer" — the model is described as a state model but drawn as a feed-forward read; (b) `rows_v1` appears as unexplained jargon inside the figure; (c) "equal sum → snap" omits the ternary gates that `docs/architecture.md:18` documents. |
| `docs/figures/architecture-narrow.svg` (390px) | Yes | Yes | **A**. Genuinely better than the desktop version: single column, legible at 390px, same facts. Best-executed figure in the set. |
| `docs/figures/milestones.svg` | Yes | **Partly** | **B+**. Good layout, M1-M8 all visible, badges match README. **M1 reads "paired dose: partial"**, which is stale: the Dose1 was owner-stopped and Dose2 stopped at Q1 admission collapse (`README.md:19-21`). Should read "Dose2 stopped at Q1; admission collapse". |
| `docs/figures/geometry/compliant-path.svg` | Yes | **Label conflict** | **B**. Craft is high and the header correctly says "Historical float operator path (12-13 October)". But every node carries a **`BUILT & TESTED`** badge while the path is `dormant` under D32. Mitigated because it only appears inside a collapsed `<details>` with a `dormant` summary — but the badge vocabulary fights the status vocabulary. |
| `docs/figures/geometry-stack.svg` | Yes | **Label conflict** | **B−**. H1 reads "Geometric primitives and operators: what each does and where it is used" — present tense, current-looking — and **12 rows say `BUILT & TESTED`**, including G2 Hamilton gate, recurrence gates, rank-margin and OP3a, all `dormant`. Only the subtitle ("Historical operator inventory… float source is inactive") saves it. Not linked from README; only from the archived `docs/history/geometry-float-form-2026-10.md`. A search engine or a curious contributor will find it and believe it. |
| `docs/figures/geometry/operators.svg` (1200×600) | Yes | **Title stale** | **A−** visually. Clear "each replaces a banned operation" framing. Title says "the **compliant trunk**" — that trunk is the pre-D32 float path, not the active one. Rename to "the earlier float trunk (inactive)". |
| `docs/figures/geometry/quaternion-rotation.svg` | Yes | Yes | **A**. Excellent math communication: unit-quaternion sphere, component decomposition, and an honest note that additive recurrence leaves S³. |
| `docs/figures/serving-pipeline.svg` | Yes | Yes | **A−**. "Historical serving paths and D11 target" — honest, includes the "Next: export the compliant trunk" gap. **Not embedded anywhere reader-facing.** It is the best available visual for the serving paragraph and should be used there. |
| `docs/figures/training-pipeline.svg` | (metadata only) | Labelled historical | Generated by `xtask`, not embedded in any reader-facing page. Orphaned. |
| `assets/images/*.jpg,png` (9 files, ~9 MB) | Yes | **No — actively misleading** | **F as front-door material.** `hero_banner.jpg` is sci-fi concept art showing "COGNITIVE INTELLIGENCE HUB · LIVE NEURAL MANIFOLD", "SYNAPTIC FLOW: 1.4 TB/s", "OPS/S: 315 PETAFLOP S", "LATENCY: 1.1ms", dated **SEP 14, 2077**. None of it is real. It is correctly captioned as historical in `assets/images/README.md:3-7`, but it is one copy-paste away from becoming the README hero and turning an honest project into a false one. |

**Figure verdict:** craft is consistently high — better than most research repos — and the generated-figure discipline (`xtask figures` fails on a stale SVG) is real. The systemic weakness is **badge vocabulary**: "BUILT & TESTED" is used for dormant float source in two figures, which is exactly the class of error the README's `active`/`dormant`/`archived-negative` vocabulary exists to prevent.

---

## 4. GitHub surface (outside the files)

| Surface | State | Verdict |
|---|---|---|
| Repository description | **empty string** (`gh repo view`) | **D.** The About panel is blank. This is the single highest-impact fix in the audit. |
| Topics | `null` | **D.** No discovery path via `language-model`, `rust`, `geometric`, etc. |
| Social preview | default GitHub card (`usesCustomOpenGraphImage: false`) | **C−.** Every shared link shows the generic Octocat card instead of the architecture diagram. |
| Homepage URL | `https://uor-foundation.github.io/uor-r4/` | Points at the Pages Studio, which the README never mentions. |
| Pages | `status: built`, source `gh-pages`, last deployment **13 Oct**, build type legacy | Live but stale. Serves the "UOR-R4 Studio" shell: "No model loaded… Throughput: unavailable until generation". |
| Latest release | `v0.1`, published **19 Aug 2026** | Predates every current mechanism. Not referenced by README. Confusing as the only release. |
| Stars / forks | 15 / 5 | Consistent with "outside collaborators arriving now". |

### D36 — is the condition met by #2654?

**No.** D36 (`DECISIONS.md:1712-1735`) re-enables the deploy "until M4 lands a D11-compliant served wasm engine". `#2654` (merged today, `966c28519`) delivers a real D11 integer WASM runtime — 7/7 serving symbols pass the zero-mul/zero-float call-graph audit, Node round-trip is byte-identical — but it serves the **historical `uor-r4-integer` stack** with a **synthetic `UORLUT01` fixture**, not the compliant discrete model. The deploy workflow still builds the root crate, which D36 §3 records as broken on retired core modules, and `.github/workflows/deploy.yml:21` still has `if: ${{ false }}`. So the decision is correct as written, but **nothing on the front page says any of this**, and a reader who sees `docs/wasm-serving/` plus a merged runtime will assume the site works.

Recommendation: one sentence in README M7 and one in `docs/architecture.md`, plus reopen/re-scope the Pages question now that a runtime exists — its remaining blockers have changed shape from "no engine" to "engine exists, no compliant artifact, build path broken".

---

## 5. Approachability — can an outside ML researcher answer the six questions?

| Question | Answerable from the front door? | Where it breaks |
|---|---|---|
| What is this? | **Yes, late** — `README.md:41-55` is excellent, but it is 40 lines down and preceded by 300 words of run logs. | Hero slot occupied by status log. |
| Why is it interesting? | **Mostly.** The transformer-cost → integer-laptop framing lands. Missing: why geometry specifically could work, in one concrete sentence with a mechanism example. | `README.md:43-46` states the bet; `:48-52` lists the objects but doesn't connect them to a payoff. |
| What works today? | **No.** Only inferable by negating the negative statements. No positive capability list exists anywhere on the page. | No "works / doesn't work" artifact. |
| What doesn't work? | **Yes, emphatically** — arguably over-weighted. | Fine; honesty is the project's strength. |
| How can I contribute? | **Partly.** The workflow is precise, but there is no runnable entry point and the suggested first tasks are the hardest ones. | `CONTRIBUTING.md:18-33`. |
| What are the rules? | **Yes, but incomplete and heavy.** | `README.md:162-184` missing D35/D36; undefined `R14`. |

---

## 6. Top 10 improvements, ranked by impact

**1. Set the repository description and topics.** Five minutes, largest single gain, currently blank.
Suggested description: *"Experimental native Rust geometric language model: exact addressed memory and learned typed operators, targeting integer-only serving on a laptop. Pre-alpha — no model holds a useful conversation yet."*
Topics: `language-model`, `rust`, `research`, `geometric-deep-learning`, `integer-inference`, `cuda`, `wasm`, `quaternion`, `e8-lattice`, `pre-alpha`.

**2. Replace `README.md:13-32` with the ~110-word version in §1.2.** Move pod IDs, waiver comment IDs and UTC timestamps to STATUS. This alone moves the front door from "lab log" to "research project".

**3. Add a "What works today / what does not" pair immediately after the status paragraph.** Concretely:
- *Works:* integer 8D carrier with 16 roots and 64 signed actions; Route/Flock/Relation reads; shared input/output writer; D19 exact byte store; 25/25 CUDA construction controls; 1,085 finite parity cases; a D11 WASM runtime for the historical stack (#2654).
- *Does not:* useful conversation, general prose, reasoning or coding; any unified served model; any measured laptop energy figure; running the current model in a browser.
This answers the one question the README currently cannot.

**4. Fix the milestone table across three artifacts.** Make README `:98-99`, `ROADMAP.md:144-145` and `milestones.svg` agree on M2 **held** / M3 **paused**; rewrite the M7 cell for #2654 as in §1.4; change the figure's M1 line from "paired dose: partial" to "Dose2 stopped at Q1: admission collapse".

**5. Correct `ROADMAP.md:34-35` to D37.** It is the only page still teaching the superseded total float ban, and it is the page an outsider reads for the rules. Use the replacement sentence in §2.1.

**6. Add D35 and D36 to the README rules table and fix "D0–D34".** D35 (seed-2 held) and D36 (Pages paused) both explain visible facts a newcomer will otherwise find inexplicable — an absent release and a site that loads no model.

**7. Split `docs/README.md` and `docs/PROJECT_MAP.md`.** Move the historical bulk to `docs/history/` with dated banners; leave the current 37 lines and the "read the authorities" table. This converts the two worst navigation pages into useful ones without deleting a word of evidence. Keep one line at the bottom of each: *"Earlier programme narrative (pre-D23, retained as history): [link]."*

**8. Fix the two mislabelled figures.** In `xtask/src/figures.rs`, change `geometry-stack.svg` H1 to "…(historical float form)" and its 12 badges from `BUILT & TESTED` to `HISTORICAL · INACTIVE`; do the same for `compliant-path.svg`. Rename `operators.svg`'s title from "the compliant trunk" to "the earlier float trunk (inactive)". Badge vocabulary must match the `active`/`dormant`/`archived-negative` vocabulary the README already teaches.

**9. Create a real hero/social image and quarantine the fake one.** Generate a 1280×640 card from `xtask` (title, the one-line question, the narrow architecture flow, "pre-alpha · 0/8 milestones") and set it as the GitHub social preview. Rename `assets/images/hero_banner.jpg` to something that cannot be mistaken for a banner (e.g. `historical-concept-art-2077.jpg`) and keep the existing caption; never embed it.

**10. Make the README's own defects impossible to re-introduce.** Fix the spacing typos in §1.1 and add the `crates/`+`xtask/` `#[test]` count (6,452 today), live issue count (10) and current decision range (D0–D37) to whatever regenerates that block — or delete the commit/test statistics entirely and link GitHub insights. Also add a short table of contents after the badges; 395 lines and 5,045 words with no map is a lot to ask.

**Two smaller items worth batching into the above:** embed `serving-pipeline.svg` in the serving paragraph (it is the best explanation of D11 in the repo and currently unused), and fix `CONTRIBUTING.md:118-122` to name `uor-harness` rather than the retired `cli.py`.

---

### What I verified vs. did not

- **Verified:** README/STATUS/ROADMAP/CONTRIBUTING/docs pages read in full or in relevant part; all relative links and image paths in the eight front-door docs resolve (script check, 0 broken); HF model, HF dataset, Pages and release URLs return 200; Pages status/deployments via `gh api`; repo description/topics/social preview via `gh repo view`; milestone issue titles/states via `gh`; test-function count by direct enumeration; figures rasterised with headless Chrome at native viewBox and inspected visually.
- **Not verified:** commit count (this clone is shallow — `git rev-parse --is-shallow-repository` = true), so I do not claim the 1,990 figure is wrong, only unverifiable here; `training-pipeline.svg` visual quality (orphaned, inspected via title/desc only); live GitHub issue comments linked from the README (not fetched).