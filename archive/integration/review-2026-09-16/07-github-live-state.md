> **UOR-R4 Project Knowledge Base — source report D (GitHub live state; API was proxy-blocked, see caveat).** Produced 2026-09-16 by a Claude research session at Casey's request, from the GitHub clone (main @ a5655e93, 2026-09-14) and live sources. Read-only audit; nothing in the repo was modified. Treat "[Inference]"/"[ASSESS]" as reviewer judgement and everything else as quoted/derived from the cited files. Index: `00-project-brain-index.md` (this folder) / `claude/00-project-brain-index.md` (Claude Project).

# D — GitHub live state of UOR-Foundation/uor-r4 (collected 2026-09-16)

## 0. Access caveat (read first)

**The GitHub REST API and github.com HTML were not reachable from this session.** Every `curl` to
`https://api.github.com/...` and `https://github.com/...` returned HTTP 403 from the session's agent
proxy with the body: *"GitHub access to this repository is not enabled for this session. Use add_repo to
request access..."* (raw responses saved in `/home/claude/work/reports/github-raw/*.json`, 378 bytes each;
`/users/...` and `/orgs/...` paths additionally returned *"This GitHub API path is not available: sessions are
bound to their configured repositories"*). The `X-RateLimit-Limit: 15000` header on `/rate_limit` shows the
proxy is authenticating, but no repository was attached to this session, so the API budget was unusable.
The `add_repo` tool is not available in this session's toolset (the MCP server that would provide it failed to connect).

What **was** reachable and was used instead:

| Source | Status | Used for |
|---|---|---|
| `git ls-remote https://github.com/UOR-Foundation/uor-r4.git` (git smart-HTTP, live 2026-09-16) | OK — 1,595 refs | branches, tags, `refs/pull/N/head` (PR numbers), `refs/pull/N/merge` (open PRs), gh-pages |
| Local partial clone `/home/claude/work/uor-r4` (main @ a5655e93, identical to live `refs/heads/main`) | OK | commits, merged PRs, authors, tags, workflows, gh-pages history |
| `https://raw.githubusercontent.com/...` | OK | READMEs of sibling repos |
| `https://uor-foundation.github.io/uor-r4/` and `https://casey-allard.github.io/...` | OK | Pages sites, worker JS, WASM HEAD |
| In-repo issue snapshot `docs/integration/issue-reconciliation-2026-09.json` (schema `uor-r4.issue-reconciliation/1`, date **2026-09-07**, base commit bc03f2d7) | OK | full census of 422 issues (411 closed / 11 open) with titles, labels, assignees, updatedAt/closedAt and the 11 open bodies **as of 2026-09-07 (pre-refresh)** |

Everything marked **"not fetched"** below could not be obtained live. Issue data is therefore **9 days stale
(2026-09-07)** and issue bodies are the pre-reconciliation versions; the 09-07 refreshed bodies are not in the repo at all
(the only body receipt, `docs/integration/native-update-receipt.json`, dates from the earlier 2026-09-03 update, PR #1092).

---

## 1. Repository metadata

| Field | Value | Source |
|---|---|---|
| full_name | UOR-Foundation/uor-r4 | ls-remote |
| description | **not fetched** (API). README H1: "UOR-R4 Geometric Language Model — An open-source research project to replace transformer-based large language models with a learned geometric language model running on local, commercially available hardware." | README.md @ main |
| homepage | **not fetched**. Pages site live at https://uor-foundation.github.io/uor-r4/ | curl |
| stargazers / forks / subscribers | **not fetched** | — |
| license | MIT (LICENSE: "Copyright (c) 2026 The Universal Object Reference Foundation (UOR)"; Cargo `license = "MIT"`) | clone |
| default branch | `main` (`HEAD -> refs/heads/main` in ls-remote) | ls-remote |
| created_at | **not fetched**. First commit on main: 2026-06-09T14:18:18-04:00 "Initial commit" by Casey-allard | git log |
| pushed_at | **not fetched**. Latest commit on main: 2026-09-14T20:07:27Z (#1280); latest gh-pages commit 2026-09-14T20:12:21Z. Two `git ls-remote` runs today (about 15 minutes apart) returned identical ref sets. | git |
| size | **not fetched** | — |
| topics | **not fetched** | — |
| open_issues_count | **not fetched** live. Snapshot 2026-09-07: 11 open + 2 created (#1172, #1173) = **13 open per ROADMAP.md**; 0 open PRs inferred (see §4) | snapshot / ls-remote |
| has_pages | effectively **true** (gh-pages branch exists; site serves 200, `last-modified: Mon, 14 Sep 2026 20:13:03 GMT`) | curl |
| has_discussions / has_wiki | **not fetched** | — |
| Branches | **734** `refs/heads/*` (live). Prefixes: 512 other, 136 `codex/`, 28 `feat/`, 23 `copilot/`, 16 `fix/`, 7 `docs/`, 6 `feature/`, 5 `agent/`, 1 `chore/` | ls-remote |
| Commits on main | **1,074** | git rev-list |
| PR refs | **856** `refs/pull/N/head`; highest PR number **1280** | ls-remote |
| Workspace crates | `uor-r4-wasm-router` (lib, v0.1.0), bin `r4`, `bdd`; toolchain pinned 1.97.1 in deploy.yml | Cargo.toml |

---

## 2. Open issues

### 2.1 List (snapshot 2026-09-07; live list **not fetched**)

Census: 422 issues total, 411 closed, 11 open; reconciliation then "refreshed all 11 open issues and added
#1172/#1173. No issue was closed as complete" (ROADMAP.md). So the expected live open count is **13** unless
something changed after 09-07 (ROADMAP.md and project-track.md at main@09-14 still list the same 13).

| # | Title | Labels | Assignees | updatedAt (snapshot) | Comments |
|---|---|---|---|---|---|
| 1173 | Run the native geometric model in GitHub Pages AI Studio | not fetched | not fetched | created 2026-09-07 | not fetched |
| 1172 | Complete the native capability API and WASM model runtime | not fetched | not fetched | created 2026-09-07 | not fetched |
| 1140 | Step 4/4: Learn typed operator composition for conversation and Rust reasoning | enhancement | Casey-allard | 2026-09-07T01:50:03Z | not fetched |
| 1139 | Step 3/4: Learn geo-transformer routing and selected computation | enhancement | Casey-allard | 2026-09-07T01:50:01Z | not fetched |
| 1088 | Develop and verify executable coding and controlled workspace capability | — | — | 2026-09-05T21:39:47Z | not fetched |
| 973 | Build and qualify the native Rust prime/zeta/R4 geometric model | — | Casey-allard | 2026-09-06T02:26:45Z | not fetched |
| 965 | Qualify and release a useful native geometric AI for conversation and coding | enhancement | — | 2026-09-05T21:39:47Z | not fetched |
| 964 | Specify and verify the implemented native geometric serving guarantees | enhancement | — | 2026-09-05T21:39:46Z | not fetched |
| 963 | Improve measured native geometric runtime cost and resumable resource use | enhancement | — | 2026-09-05T21:39:45Z | not fetched |
| 962 | Develop native conversation and identity-scoped persistent geometric memory | enhancement | — | 2026-09-05T21:39:45Z | not fetched |
| 955 | Develop and qualify multi-step geometric reasoning | enhancement | — | 2026-09-05T21:39:44Z | not fetched |
| 954 | Qualify grounded correctness, contradiction handling and abstention | enhancement | — | 2026-09-05T21:39:43Z | not fetched |
| 820 | TRACKER: native Rust prime/zeta/R4 geometric intelligence | — | — | 2026-09-06T00:49:39Z | not fetched |

Author of every issue: **not fetched** (the snapshot carries no author field; all assignees present are `Casey-allard`, GitHub user id 112731500).
Ordered roadmap (snapshot `roadmap[]`, mirrored in ROADMAP.md): 1139 → 1140 → 973 → 962 → 954 → 955 → 1088 → 963 → 964 → 1172 → 1173 → 965, all under tracker 820.

Body sizes after the earlier 2026-09-03 update (native-update-receipt.json, chars): #973 65,316 · #820 57,818 · #954 17,233 · #964 6,205 · #963 5,605 · #955 5,580 · #962 5,359 · #965 5,212.

### 2.2 Body summaries (pre-refresh bodies from the 2026-09-07 snapshot)

Common boilerplate present in #954/#955/#962/#963/#964/#965/#1088: *"The owner-adopted immediate build sequence is **#1137 -> #1138 -> #1139 -> #1140**, under #973. ... Rust learning may use matrix multiplication; serving remains bounded learned prime/zeta/R4 state/routing and integer/table operators."* Each keeps its historical spec inside a `<details>` block with "Outcome / … / Definition of done".

**#820 TRACKER: native Rust prime/zeta/R4 geometric intelligence** (8,816 chars)
- What: programme tracker. "Latest execution checkpoint": #1138's bounded exact-relation handoff passes "84/84 OPEN and 28/28 reserved answers AND write sequences"; PR #1144 delivers; storage PR #1143 merged at 50caf04d. Immediate plan: #1137 (done, PR #1142 @ 223aed71) → #1138 (PR #1144 pending) → #1139 → #1140. "#973 remains the native model parent. Broader #954 correctness -> #955 reasoning -> #962 conversation/integration -> #963 optimization -> #964 guarantees -> #965 release follow." #1088 "owns broader executed coding/workspace capability and is also required for release."
- Why: "The owner has restored one destination: a Rust geometric AI using primes, fixed zeta-zero phases and R4/S3/H4 mechanisms throughout preparation, training, artifact construction and inference." "a dense transformer hidden behind lookup does not meet the goal."
- Acceptance: none explicit at tracker level; superseded trackers #1083/#1084/#1087/#1089/#1090/#1091/#940 "consolidated in the responsibility map. Their closure means scope moved, not capability, QA or administrator action completed."
- Resources: "Cumulative model work is 1,267.771/1,800 seconds, leaving 532.229; unchanged storage cap 6,459,228,160 bytes and 128 MiB margin. This cycle used 38.821s model/412.722s engineering, with no deletion or paid compute."

**#973 Build and qualify the native Rust prime/zeta/R4 geometric model** (50,184 chars; 16 dated sections)
- What: parent issue for the native model. "Immediate direction": *"The target is a geo-transformer LLM that replaces dense transformer execution with learned geometric placement, routing and selected operators, aiming at frontier capability on normal laptops. This remains an objective, not current demonstrated capability."* Immediate #1139 work: "one jointly trained native block connecting contextual semantic codes, bounded angular/prime admission, selected exact payloads, learned integer/table transformation and token prediction... The new block is NOT_IMPLEMENTED / NOT_RUN."
- Current checkpoint (09-05): sparse admission artifact `067adbf0` — "64 exactly guarded NoWrite signatures"; "Final warm medians on the longer repeated-filler workload are 23.301 ms parent, 1.151 ms geometric and 1.110 ms sparse... this is not a general 21-fold speedup or a geometric superiority result."
- Why: "Preparation, training, artifact construction and inference run in Rust. Training may use matrix multiplication. Serving must execute learned geometric operators and bounded route/state/table operations, without a Python model dependency, external response provider or dense transformer concealed behind lookup."
- Acceptance: no single checklist; the body repeatedly states "Do not infer completion from this issue header", "Its model family is an experimental finite-state statistical language model."
- Resources: "Cumulative model work is unchanged at 1,363.884/1,800 seconds (436.116 remaining)"; "Peak sampled direct model RSS is 777,273,344 bytes. Storage cap remains 6,459,228,160 bytes with 128 MiB margin; final sampled known storage is 5,702,713,344 bytes."

**#1139 Step 3/4: Learn geo-transformer routing and selected computation** (5,189 chars)
- What: "Current result: learned supported-source versus NoRead selection. Retain d59070c2 ... in protected PR #1160 ... PR #1159 is merged at be32b410." Measured: "Fresh complete answers improve 14/20 to 20/20 versus 10/20 matched exact-code continuation... Complete combined construction improves 545/603 to 551/603 with no lost correct answer."
- Immediate next (numbered 1–4): fix "the observed supported-location prompt about cyra that emits 13 instead of Paris through inherited numeric admission"; "Learn numeric-versus-word choice jointly... do not introduce a query-template parser, universal Unknown rule or another suffix head"; keep lineage explicit; "Resume the broader #1139 routed learned-block and #1140 useful conversation/Rust composition requirements after this observed admission repair."
- Acceptance: none as checkboxes; "This increment does not complete the broader issue acceptance." Goal statement: "a learned geometric language model using bounded prime/zeta/R4 routing, state and integer/table operators on ordinary laptops... General syntax, prose and reasoning are unqualified."
- Resources: "This cycle charged 155.350s model and 524.036s engineering versus 420/900s point projections... Cumulative model use 3605.626/4170s leaves 564.374s... Peak sampled known storage 6,837,837,824 bytes stayed below the effective 7,546,662,912-byte ceiling; peak sampled model-child RSS 1,141,374,976 bytes stayed below 4GiB."

**#1140 Step 4/4: Learn typed operator composition for conversation and Rust reasoning** (7,973 chars)
- What/Build: same "Current result"/"Immediate next" text as #1139, then "Build": "Learn which operator and operands a query requires. Select/admit the needed computation before executing it... Include a bounded sequence of at least two supported operations with an intermediate value/state that influences the next decision."
- Acceptance (verbatim checklist, all unchecked):
  - [ ] "Both grounded conversational composition and generated Rust handle changed operands/identifiers/dependencies on predeclared first-use tasks."
  - [ ] "Real execution of generated Rust matches the requested behavior; compilation alone is insufficient."
  - [ ] "Operator/operand intervention demonstrates causal use of the intermediate state, with no supplied final answer, provider or post-hoc explanatory trace standing in for computation."
  - [ ] "Prior binding, memory and routing behavior remains within the declared preservation criteria; count all admission, operator and output work."
  - [ ] "Record the accepted assembled artifact and remaining limitations as the handoff to broader #954 correctness, #955 reasoning, #962 conversation and #1088 coding qualification."
- Architecture: "Use the same native Rust model for conversation/memory and coding/reasoning... without a hidden dense transformer, Python model dependency or response provider."
- Resources: identical to #1139.

**#954 Qualify grounded correctness, contradiction handling and abstention** (20,882 chars)
- What: "receives the accepted assembled artifact from #1140". Historical Outcome: "provider-free geometric generation can produce answers that are correct with respect to a declared source of truth, use the full bounded global context when required, surface contradictions, and abstain when support is insufficient. Fluency, teacher agreement, exact recall, and nonzero geometric activity are not correctness evidence."
- Acceptance (Definition of done, verbatim): "Grounding, contradiction, clarification, and abstention are typed serving outcomes. / Independent oracles and fixed denominators make the result auditable. / Global-context dependence is causal, not inferred from a trace. / Exact recall and unseen-history geometric selection are reported separately. / Every answer or abstention is reproducible from the pinned artifact and route state. / #955 receives a correctness-qualified #969/#953/#973 inference path."
- Resources: none stated; body carries dated blocker checkpoints 2026-08-28 → 2026-09-02 (#1075/#1077/#1079 completions).

**#955 Develop and qualify multi-step geometric reasoning** (9,147 chars)
- What: "broadens and qualifies reasoning only after that [#1140] handoff and #954's grounded correctness... #1088 consumes this capability." Outcome: "multi-step reasoning as explicit, auditable composition of route transitions... Fluent prose, teacher agreement, exact lookup, or a long internal trace is not reasoning evidence."
- Acceptance (DoD): "Intermediate route states and branch decisions are canonical, causal, and replayable. / Novel composition is separated from exact or near-exact recall. / Matched ablations demonstrate that multi-step qualified state, not final-answer lookup, is load-bearing. / Correctness and abstention use the frozen #954 contract. / Cost scales with visited route nodes and reused ancestors, not the full corpus. / #962 receives a reasoning-qualified ... engine."
- Resources: none stated.

**#962 Develop native conversation and identity-scoped persistent geometric memory** (9,110 chars)
- What: "expands the accepted model after #954/#955 into meaningful multi-turn conversation, identity-scoped durable memory, restart, reset, export/forget and real CLI/service/workbench behavior. It also absorbs #1084's remaining interface integration." (Snapshot: #962 interface scope transfers to #1172.)
- Acceptance (DoD): "The normal CLI/HTTP chat path exercises the #969-qualified local path plus #953-qualified decoded grammar/sentence loop and #973-qualified higher-scope attention end to end. / Startup, update, restart, export, and forget semantics are explicit and bounded. / Memory influence is causally visible against a disabled-memory control. / Product status reports separately: text coherence, route coverage, correctness, abstention, isolation, latency, and storage. / No fallback can silently convert a typed abstention or decode failure into provider output."
- Resources: none stated.

**#963 Improve measured native geometric runtime cost and resumable resource use** (9,425 chars)
- What: "#1139 owns the first matched selective-routing work reduction. This issue owns broader complete-path optimization, useful CPU/resource scheduling and resumability... absorbs #1087's remaining serving realization and #1083's touched artifact/session lineage."
- Why: "No run may consume hours merely because completion is assumed to be near." "Long-run hard gate": "A run with predicted wall time over eight hours does not launch."
- Acceptance (DoD): "The serving and compile hot paths have bounded cost models. / Node-local invalidation and exact reuse are implemented and traced. / The selected hardware-specific execution plan and deterministic reduction are supported by measured comparison. / Checkpoint/resume preserves canonical output identity. / A runtime watchdog enforces disk and eight-hour stop policies. / No claimed optimization is accepted without a matched baseline on the deployed path."

**#964 Specify and verify the implemented native geometric serving guarantees** (9,952 chars)
- What: "absorbs the remaining #1087 serving-contract qualification, #1083 typed identity/derivation guarantees, and #1089 research/publication responsibility... Publication follows implemented, measured results."
- Acceptance (DoD): "Normative symbols and claim statuses match the shipped code and artifact schema. / The serving source/artifact closure is exact and machine-checkable. / ... / The product can prove providers and source weights are absent at runtime. / Every empirical claim links its population, denominator, matched control, and artifact identities. / L1 receives a frozen contract and an explicit list of still-unproven capabilities."

**#965 Qualify and release a useful native geometric AI for conversation and coding** (9,277 chars)
- What: "absorbs #1090's final capability scorecard and #940's release-governance responsibility. Before release, qualify both conversation/memory and executed coding/reasoning, complete cost/portability/security, reproducible artifact loading, installation and rollback. Require #964 guarantees and #1088 executed coding/workspace acceptance." Note on CI: "The five historical required names currently include compatibility acknowledgements; they do not establish dependency audit, fuzz, WASM or Gate C execution."
- Frozen release identity: "Serving contains no transformer, MoE, dense-matmul, source-attention, Ollama, hosted model, or equivalent fallback."
- Acceptance (DoD): "The readiness audit is positive. / Bounded product QA passes every predeclared floor, or the release is explicitly rejected. / Double builds produce byte-identical artifacts and κ values. / A fresh local install produces real provider-free chat and persists/forgets hive memory as declared. / The release manifest, semantic-qualification status, evidence bundle, known limits, rollback path, and support policy are frozen. / If accepted, #940 receives the exact minimal release-rule design and the administrator explicitly activates it. / The milestone closes only after the release artifact and issue verdicts agree."

**#1088 Develop and verify executable coding and controlled workspace capability** (4,578 chars)
- What: "#1140 owns initial generated-Rust typed operator composition. This issue extends the accepted reasoning and same-model service/session integration into meaningful repository-context edits, multi-file repair and controlled tool iterations. It absorbs #1084's coding-facing interface obligations. Require actual execution feedback and requested semantic correctness, not merely compilation or a patch viewer. #965 depends on this capability."
- Acceptance (Deliverable and DoD): "Progress from repository-context reading and single-file patches to multi-file repair and bounded tool iterations. Use held-out task identities, scoped workspace access, actual diff/save/reopen, executed results and verifiable outcomes. Separate harness/UI functionality from model coding skill; clear attribution for any external reference model. Include failures, context provenance and the resource budget."
- Ownership: "Planned lane, unassigned... Next: define a minimal executable task/patch/result schema."

**#1172 Complete the native capability API and WASM model runtime** — body **not fetched** (created 2026-09-07, after the snapshot). Rationale from roadmap: "One coherent model must expose its actual abilities to applications before Studio integration can be meaningful." Receives #962's interface scope and part of #1083/#1084/#1087.

**#1173 Run the native geometric model in GitHub Pages AI Studio** — body **not fetched**. Rationale: "The final user-facing goal is the already developed Studio running our own local geometric model in the browser." Receives part of #1084's integration/workspace scope.

### 2.3 Comments on #820 and #973

**Not fetched** (API blocked). Indirect evidence only: repo docs link 27 distinct `issues/820|973#issuecomment-…` permalinks (e.g. `973#issuecomment-5520112973` is cited as the owner "scope_amendment_url"), i.e. the owner uses issue comments as decision records. Human vs bot commenter logins: **not fetched**.

---

## 3. Closed issues — 30 most recently closed (snapshot 2026-09-07; live **not fetched**)

Closed per month (census): 2026-07: 128 · 2026-08: 243 · 2026-09 (to 09-07): 40. Labels across all 422: graph-compiler 82, enhancement 57, conformance 29, bug 15, documentation 4, status: on-hold 4, question 1.

| # | closed_at | Title |
|---|---|---|
| 1138 | 2026-09-06T01:11:38Z | Step 2/4: Retain exact relations with learned writes and updates |
| 1137 | 2026-09-05T23:17:16Z | Step 1/4: Learn role-aware source selection and causal response commitment |
| 1091 | 2026-09-05T21:39:53Z | Evaluate concrete NEMESIS and W33 mechanisms for the native geometric model |
| 940 | 2026-09-05T21:39:53Z | DORMANT: review historical ruleset compatibility when admin is available |
| 1090 | 2026-09-05T21:39:52Z | Measure native conversation, memory, reasoning, coding and resource capability |
| 1089 | 2026-09-05T21:39:51Z | Develop the geometric research paper from exact claims and observed evidence |
| 1087 | 2026-09-05T21:39:50Z | Qualify the native geometric integer/table serving contract |
| 1084 | 2026-09-05T21:39:49Z | Expose the same native Rust geometric artifact through CLI and local service |
| 1083 | 2026-09-05T21:39:48Z | Integrate typed UOR identity and exact arithmetic in the native geometric model |
| 1107 | 2026-09-04T04:29:22Z | Freeze the native Four-fact workbench source candidate |
| 1108 | 2026-09-04T03:58:22Z | Enforce deterministic source-only execution for automated agents |
| 1105 | 2026-09-03T17:17:55Z | Freeze the native service API and artifact-ownership contract |
| 1102 | 2026-09-03T16:15:40Z | Implement and qualify the frozen native learned-reference bridge |
| 1086 | 2026-09-03T14:17:44Z | Specify native artifact export, loader and reference behavior for the learned path |
| 1094 | 2026-09-03T13:39:13Z | Implement and compare the fixed text-to-clause adapter against supplied segmentation |
| 1096 | 2026-09-03T12:05:49Z | Repair isolated runtime readiness after the #1094 preflight stop |
| 1085 | 2026-09-03T04:53:04Z | Specify general language and context transfer from the accepted learned interface |
| 1082 | 2026-09-03T04:25:09Z | Diagnose token-frame exposure and pooled-role displacement on the frozen construction views |
| 1081 | 2026-09-03T04:12:29Z | Adopt integrated research workflow, source dossier and coherent current roadmap |
| 1079 | 2026-09-03T02:25:21Z | Qualify learned soft-role and compound binding through unchanged R4 |
| 1077 | 2026-09-03T01:52:17Z | Learn a position-independent language interface for compound binding |
| 1075 | 2026-09-03T01:11:26Z | Preserve compound binding through unchanged R4 inference |
| 1073 | 2026-09-03T00:39:46Z | A1Q-H: learn compound owner-object binding through explicit fact-level Q/K/V |
| 1071 | 2026-09-03T00:03:16Z | A1Q-H: learn four-fact binding across cyclic fact orders at the matched dose |
| 1069 | 2026-09-02T23:28:59Z | A1Q-H: test direct owner-plus-object query encoding at the matched learning dose |
| 1067 | 2026-09-02T22:55:46Z | A1Q-H: test query-object answer readout with the matched English learning dose |
| 1065 | 2026-09-02T21:41:06Z | A1Q-H: diagnose retained English construction binding without training |
| 1063 | 2026-09-02T21:01:51Z | Learn English supplied-context binding with the qualified attention cell |
| 1061 | 2026-09-02T19:48:47Z | A1Q-H15: preserve exact-data associative recall through coherent R4 inference |
| 1059 | 2026-09-02T19:23:38Z | A1Q-H14: preserve qualified associative recall through coherent R4 inference |

Note the 09-05T21:39 block: seven issues closed within 5 seconds — the bulk "superseded/consolidated" closure recorded in #820. Snapshot text: "All 411 remain closed as historical evidence; closure is not interpreted as success."

---

## 4. Pull requests (derived from git; API **not fetched**)

Method: `refs/pull/N/head` from live ls-remote (856 PRs, max #1280); merged PRs identified on `main` by squash subject suffix `(#N)` or `Merge pull request #N from …`; head-branch names recovered from still-existing `refs/heads/*` whose tip SHA equals `refs/pull/N/head`.

- Merged PRs on main: **830** (807 squash, 23 merge commits). PR refs not in main: 27 (#1–5, 9, 10, 51, 53, 54, 61, 81–88, 144–146, 319, 453, 455, 594, 975) — closed-unmerged or open.
- **Open PRs: 0 inferred** — the live ls-remote contains **zero** `refs/pull/N/merge` refs (GitHub keeps one per open PR that can be merged). Cannot be confirmed without API.
- Merged per month: 2026-07: 161 · 2026-08: 459 · 2026-09 (1–14): 210.
- Merge mechanics: committer `GitHub <noreply@github.com>` on 926/1,074 commits (web squash-merge); gh-pages deploy commits are authored by `github-merge-queue[bot]` (357 of 395), which indicates a **merge queue** is in use on main.

**Fractions by head-branch prefix**

| Window | codex/ | copilot/ | agent/ | issue-NNN-* | r5/ | other | unknown (branch deleted) |
|---|---|---|---|---|---|---|---|
| All 830 merged | 139 (16.7%) | 15 (1.8%) | 5 (0.6%) | 290 (34.9%) | 59 (7.1%) | 212 (25.5%) | 110 (13.3%) |
| 200 most recent (#1037–#1280, Sep 1–14) | **136 (68%)** | 0 | 0 | 41 (20.5%) | 0 | 3 | 20 (10%) |
| 100 most recent (#1181–#1280, Sep 8–14) | **82 (82%)** | 0 | 0 | 5 | 3 | — | 10 |

All 23 `copilot/*` PR heads and 5 `agent/*` heads date from July 2026 (copilot-swe-agent[bot] commits: 45 on main, all July). Since Sept 1 the workflow is essentially **Codex-branch → squash-merge by Casey Allard**.

**Fractions by merge-commit author**

| Window | Casey Allard | Ari (= Alex Flom, `me@ari.io`; presumed login `afflom`) | Copilot | maurathat |
|---|---|---|---|---|
| All 830 | 732 (88.2%) | 88 (10.6%) | 9 (1.1%) | 1 |
| Since Aug 1 (669) | 625 (93.4%) | 44 (6.6%) | 0 | 0 |
| Since Sep 1 (210) | **210 (100%)** | 0 | 0 | 0 |

Ari's activity spans 2026-07-18 → 2026-08-10 only (last: #608 "docs: plan open-weight behavioral recompilation"). Casey Allard has 829 authored commits on main under two emails (amnhealthcare.com and a local MacBook address) plus 36 as "Casey-allard".

**Recent merged PRs (all squash, author Casey Allard):** #1280 2026-09-14T20:07 `docs/geometric-language-model-research-map-2026-09-14` "docs: map geometric language research, architecture and evidence" · #1279 18:54 `codex/independent-neighbor-transfer` · #1278 18:07 `codex/unknown-neighbor-learning` · #1277 16:37 `codex/neighbor-role-transfer` · #1276 16:02 `codex/styled-role-learning` · #1275 14:26 `codex/query-participation` · #1274 13:51 `codex/role-transfer-stability` · #1273 13:24 `codex/learned-occurrence-roles` · #1272 12:40 `codex/contextual-role-diagnostic` · #1271 08:14 `codex/phrase-order-identity` · #1270 07:51 `codex/correspondence-runs` · … · #1253 (09-13 19:01) `codex/geometric-attention-research` "Synthesize relational geometric attention and prime spectral research" · #1244–#1252 addressed-attention/Hamming series · #1236–#1243 geometric decoder/angular-tree negatives (full list in `github-raw/merged_prs_from_git.json`).

**Reviews on 3 recent PRs: not fetched** (`/pulls/N/reviews` blocked). Indirect: issue bodies refer to "protected PR", "protected automatic merging enabled and CI pending", and #965 says the ruleset's "five historical required names currently include compatibility acknowledgements" — i.e. merges gated by required checks, reviewer identities unknown.

---

## 5. Releases and tags

- `/releases`: **not fetched**. `release.yml` header states a pushed `v*` tag "builds BOTH frontends... and attaches them to a DRAFT GitHub Release... Nothing is published by this workflow... the maintainer... publishes the draft by hand" (#741 decision, 2026-08-19). The uor-r4-wasm-chat README links a "macOS DMG Release v3.3.7" on *Casey-allard/uor-r4-wasm-chat/releases*, not on uor-r4.
- Tags (live ls-remote, **3**):

| Tag | Type | Target commit | Date | Message / subject |
|---|---|---|---|---|
| `v0.1` | annotated (cf547db8 → d1db980c) | "Release pipeline: tag-triggered dual-frontend builds + verified install-release (#741) (#815)" | tagged 2026-08-18 21:51:58 -0400 | "uor-r4 v0.1: first release. Binds code d1db980c + smollm2-360m-broad bundle (see attached release-bundle.json) + inference contract 1.0.0. Decision record: #741 issuecomment-5336515298." |
| `v0.1.0-alpha` | lightweight (032f255d) | "feat: integrate sovereign AI studio, wire native geometric model, add r4-native-chat CLI, and update alpha release documentation (#1195)" | commit 2026-09-08T06:28:53Z | — |
| `d3-corpus-simple-wiki-20231101` | lightweight (944ac0d3) | "batch-1: process amendment + #255 memory-lift harness (running batch) (#287)" | commit 2026-07-29 | corpus pin |

Note: `v0.1` binds a **smollm2-360m** teacher bundle (the pre-"native recovery" architecture); `v0.1.0-alpha` (09-08) is the first tag after the native-geometric pivot.

---

## 6. GitHub Actions

`/actions/runs` and `/actions/workflows`: **not fetched**. From `.github/workflows/` at main:

| File | name | Triggers |
|---|---|---|
| ci.yml | `ci` | pull_request, workflow_dispatch |
| deploy.yml | `Deploy to GitHub Pages` | push to main, workflow_dispatch — Rust 1.97.1 + wasm-pack 0.13.1, `wasm-pack build --target web`, copies index.html/index.css/geometric_prime_router_webapp.html/coi-serviceworker.js/r4_worker.js/assets/pkg to `dist/` and publishes to gh-pages |
| formal_verification.yml | `Formal Verification (Kani)` | workflow_dispatch only |
| release.yml | `release` | push tags `v*` → draft release with `r4` CLI (linux x86_64 + macOS arm64) + wasm frontend |

Run conclusions/dates: **not fetched**. Proxy for deploy-run history: gh-pages has **395** deploy commits; per day Sept 1–14: 9, 16, 16, 6, 13, 14, 16, 25, 13, 7, 3, 2, 21, 13 (tracks merges exactly). Last deploy 2026-09-14T20:12:21Z from main@a5655e93. Committers on gh-pages: github-merge-queue[bot] 357, Casey-allard 29, "auser" 9.

---

## 7. Contributors and sibling repositories

`/contributors`, `/orgs/UOR-Foundation/repos`, `/users/Casey-allard/repos`, `/users/afflom/repos`: **not fetched**. Git-derived contributors on main (1,074 commits):

| Author | Commits | Notes |
|---|---|---|
| Casey Allard / Casey-allard | 865 (784 + 45 + 36) | owner/maintainer; all Sept merges |
| Ari `<me@ari.io>` | 154 | committer on 16 of Casey's July commits is "Alex Flom <alexander.flom@gmail.com>"; repo docs name `afflom`; active 07-18 → 08-10 |
| copilot-swe-agent[bot] + Copilot | 54 (45 + 9) | July 2026 only |
| maurathat | 1 | |

Repos referenced from uor-r4 docs (link counts): UOR-Foundation/UOR-Framework 252, uor-addr 56, uor-matmul 34, prism 19, atlas-12288 16, kappa-registry 14, F1 2; afflom/lean4-prod 15, WASM-GEMM-GNAF 15, matmul 14, LexLean 14, UOR-Atlas-UTQC 11; Casey-allard/uor-r4-wasm-chat 8, prime-router 1.

README summaries (raw.githubusercontent.com, branch `main`; stars/language/pushed_at **not fetched** for all):

- **Casey-allard/uor-r4-wasm-chat** (10,071 B; `UOR-Foundation/uor-r4-wasm-chat` does not exist — 404): "UOR-R4 Sovereign AI Developer Studio (v3.3.7) — The 100% In-Browser & Native macOS Sovereign AI Studio • 8D Gosset E8 Geometric Cognitive Core • Native Metal & WebGPU Acceleration". Claims "1,500+ tok/s" on Apple Silicon Metal and "12–18+ tok/s" in-browser WebGPU, a "5-Model SOTA Sovereign Neural Catalog" of **Qwen 2.5 (0.5B) variants and "GLM-5.3 Flash (0.5B)"**, Monaco IDE with diff engine, terminal execution, Git/GitHub worktree, Tauri v2 macOS DMG, MIT. Live at https://casey-allard.github.io/uor-r4-wasm-chat/ (HTTP 200, 329,557 B). This is the Studio UI that #1195 ported into uor-r4; its README describes transformer (Qwen) models, unlike the uor-r4 copy which lists only the native geometric model.
- **Casey-allard/prime-router** (6,376 B): "R4 Prime Router — Evolving Hypersphere Brain World Model": Hopf S³ geometry + GCD prime-seeded coordinates for "zero-weight geometric sequence generation", dual engine with a "Transformer Voice (Ollama Gemma `gemma4:e2b`)", UOR attestation endpoints. Precursor to uor-r4's `geometric_prime_router_webapp.html`.
- **UOR-Foundation/uor-addr** (6,421 B): "Typed content-addressing for data passing across system boundaries — deterministic, verifiable `sha256:<64hex>` labels with structural equivalence"; canonicalizes via JCS/canonical S-expressions/XML-C14N/DER then hashes; `no_std`, allocation-free ψ-pipeline. (afflom/uor-addr is the older "uor-addr-1" reference implementation, CC0.)
- **UOR-Foundation/UOR-Framework** (2,250 B): Rust workspace implementing the UOR ontology "for content-addressed, symmetric, multi-metric object spaces with algebraic structure based on Z/(2^n)Z"; ontology v0.5.2 = 34 namespaces · 474 classes · 948 properties · 3,601 individuals; exported as JSON-LD/Turtle/OWL/JSON Schema/SHACL; `uor-foundation` crate on crates.io.
- **UOR-Foundation/prism** (3,038 B): "Prism standard library" façade crate `uor-prism` re-exporting `uor-foundation` plus Layer-3 axes (crypto, numerics, tensor, fhe) per UOR-Framework wiki ADR-031.
- **UOR-Foundation/uor-matmul** (32,075 B): "Decode the code, accumulate exactly, encode once." MatMul on coded operands — weight tier is a codec, exact accumulation, single encode; every entry point (SIMD/scalar/tile/table) must produce identical bytes, asserted by `CD-*` gates. uor-r4 pins it for offline training only (serving forbids it).
- **UOR-Foundation/kappa-registry** (36,198 B): single-binary content-addressed server speaking Docker/OCI, Git, S3, Nix and AT Protocol on one port/one data directory; sixth surface "kappa-distribution" adds typed edges, transactions, delta bundles; capability-edge authorization with delegation chains.
- **UOR-Foundation/atlas-12288** (9,884 B): "UOR Prime Structure FFI" — Lean 4 implementation of the Prime Structure/Φ-Atlas-12288 (48×256 elements, R96 resonance classifier mapping 256 bytes to 96 classes, Φ boundary encoding, "Truth ≙ conservation" budgets) with Go/Rust/Node/C FFI.
- **UOR-Foundation/F1** (24,893 B): "The 𝔽₁ Square with an Intersection Theory — an active research program toward `Spec ℤ ×_{𝔽₁} Spec ℤ`"; explicitly "Status: the Riemann Hypothesis is OPEN... The crux is never asserted as proven"; Lean 4 with sorry-free proof layer.
- **UOR-Foundation/PrismPM** (4,895 B): compiles `.lex.tex` Prism models via LexLean → Lean → lean4-prod into Cargo/Core-Wasm/browser/"Hologram" artifacts; v0.3.0 "not yet released".
- **UOR-Foundation/nest-uor** (5,927 B): Python structural content addressing for NANDA Town's DataFacts layer 12; stdlib-only canonicalizer with conformance vectors.
- **UOR-Foundation/research** (1,268 B): official research directory; one project listed (`atlas-embeddings`: exceptional Lie groups from the Atlas of Resonance Classes); MIT.
- **UOR-Foundation/website** (10,497 B): uor.foundation site; UOR is "an open data standard that gives every piece of digital content a single, permanent identifier based on what it is, not where it is stored."
- **UOR-Foundation/kappa-distribution**: README is 21 bytes (title only).
- **afflom/matmul** (3,091 B): "A repository template with the gate machinery and none of the content" (claim register + ledger + Gherkin suites; `just vv` gate).
- **afflom/WASM-GEMM-GNAF** (5,816 B): Lean 4 mechanization toward "a kernel-checked global-optimality theorem for a committed WebAssembly GEMM binary, under the pinned UOR-GNAF authority"; status verbatim: "Global optimality... has not been established"; terminal answer `WorkloadIncomplete`; Lean 4.30.0 sorry-free partial inventory. (This is the "GNAF/NAF" source uor-r4 cites; no repo named `naf` or `hologram` exists under UOR-Foundation or afflom — raw 404.)
- **afflom/UOR-Atlas-UTQC** (5,380 B): "parametric, BDD-driven, V&V-gated realization of The UOR Atlas UTQC" — a modular tensor category on a content-addressed substrate, "the basis of the entire UOR ecosystem of tooling including Hologram Holospaces"; density refuted on the 2-dim SU(2) block, established on PU(22)/PU(576) with cited lemmas; dedup metrics "are engineering measurements, not a proven quantum advantage."
- **afflom/lean4-prod** (16,366 B): "Generate real, raw, performant Rust from actual Lean 4" by walking Lean's LCNF IR into `no_std` Rust.
- **afflom/LexLean** (18,324 B): "closed-lexicon LaTeX-to-Lean 4 compiler" (`LEXLEAN-SPEC-1`), every capability a claim in `model/ids.toml`.

---

## 8. GitHub Pages

**https://uor-foundation.github.io/uor-r4/** — HTTP 200, 311,532 B, `last-modified: Mon, 14 Sep 2026 20:13:03 GMT` (matches gh-pages deploy 20:12:21Z from main@a5655e93).
- `<title>`: **"UOR-R4 Sovereign AI Developer Studio"** (a second `<title>` "Sovereign AI Interactive Sandbox" appears in an embedded sandbox template); `<h1>` "⚡ Sovereign Quantum Neural Lattice".
- External scripts: marked (jsdelivr), KaTeX 0.16.9 + auto-render, pdf.js 3.11.174 (+ pdf.worker), three.js r128 + OrbitControls, Monaco 0.45.0 loader.
- WASM/workers: `MODEL_CONFIGS` has exactly one entry `uor-r4-api` — `fullName: 'UOR-R4 API (Native Geometric Language Model)'`, `source: 'uor:native-geometric/api/1'`, `localPath: './pkg/uor_r4_wasm_router_bg.wasm'`, `tier: 'Native Geometric Pre-alpha'`, `dtype: 'Z[phi] golden integer ring, zero-matmul'`, `desc: 'Experimental native geometric model. Load an artifact to inspect its identity; general prose and broad reasoning remain unqualified.'`. Model worker `./assets/js/uor_model_worker.js?v=recovery-1` (module worker) imports `../../pkg/uor_r4_wasm_router.js` and calls `native_geometric_init/create_session/ingest/generate_step/finish_generation/export_session/import_session/cancel`. Header comment: "Native model worker. Every emitted byte comes from the explicitly loaded artifact." `requireModel()` throws "Load a retained native model JSON file before generating. **No model is bundled.**" UI text: "No model loaded. Select a retained native model JSON file." `r4_worker.js` (12,942 B) is the older R4G1 router worker (`set_r4g1_production_bundle`, schema-2 envelope, CID verifier).
- `pkg/uor_r4_wasm_router_bg.wasm`: HTTP 200, **3,124,810 B**.
- Mock/reference-backend strings: **none** — zero hits for "mock", "reference-backend", "simulat", "stub", "ollama", "localhost". "Fallback" occurs only for GitHub-token user lookup, Wikipedia search fallback and blob download. The page contains a "LIVE WEB SEARCH & GROUNDING ENGINE" (Wikipedia opensearch/summary fetches) with `isWebSearchActive = false` and the comment "Explicit native model evaluation does not inject external search material", plus GitHub API (`callGithubApi`) and Tauri-native write paths inherited from the Studio.
- gh-pages tree (22 files): `assets/images/*` (9 PNG/JPG studio screenshots incl. hero_banner.jpg, architecture_diagram.jpg), `assets/js/uor_model_worker.js`, `coi-serviceworker.js`, `geometric_prime_router_webapp.html`, `index.css`, `index.html`, `pkg/{LICENSE,README.md,package.json,uor_r4_wasm_router.d.ts,uor_r4_wasm_router.js,uor_r4_wasm_router_bg.wasm,uor_r4_wasm_router_bg.wasm.d.ts}`, `r4_worker.js`.
- `git log -5 origin/gh-pages`: 2026-09-14 20:12:21 / 18:11:37 / 16:09:25 / 14:33:07 / 13:29:32 +0000, all `github-merge-queue[bot]` "Deploying to gh-pages from @ UOR-Foundation/uor-r4@<sha> 🚀" (a5655e93 #1280, cfe89267 #1278, b2c94494 #1276, bee8fa8d, f5c730b7). 395 commits total on gh-pages.

**https://casey-allard.github.io/uor-r4/** — **HTTP 404** ("Site not found · GitHub Pages", 9,115 B). There is no Casey-allard fork Pages site at that path. **https://casey-allard.github.io/uor-r4-wasm-chat/** is live (200, 329,557 B) — the Studio's original home.

---

## 9. Commit cadence (git, `main`)

Commits per ISO week (author date):

| Week | Dates | Commits |
|---|---|---|
| 2026-W29 | Jul 13–19 | 18 |
| 2026-W30 | Jul 20–26 | 296 |
| 2026-W31 | Jul 27–Aug 2 | 90 |
| 2026-W32 | Aug 3–9 | 147 |
| 2026-W33 | Aug 10–16 | 102 |
| 2026-W34 | Aug 17–23 | 116 |
| 2026-W35 | Aug 24–30 | 51 |
| 2026-W36 | Aug 31–Sep 6 | 98 |
| 2026-W37 | Sep 7–13 | 102 |
| 2026-W38 | Sep 14–16 (partial; last commit Sep 14 20:07Z) | 18 |

(W24 Jun 8–14: 36 initial commits; W25–W28: 0.) Total 1,074 commits; 830 of the 1,038 commits since July 1 (80%) are PR merges (807 squash + 23 merge commits), and since Sept 1 every commit on main is a squash-merged PR, so recent commit counts ≈ merged-PR counts.

Merged PRs per day, Sept 1–16 (`(#NNNN)` in subject, author date UTC):

| Date | 09-01 | 09-02 | 09-03 | 09-04 | 09-05 | 09-06 | 09-07 | 09-08 | 09-09 | 09-10 | 09-11 | 09-12 | 09-13 | 09-14 | 09-15 | 09-16 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Merged PRs | 11 | 17 | 16 | 17 | 13 | 16 | 16 | **29** | 13 | 7 | 3 | 3 | **31** | 18 | **0** | **0** |

Total Sept 1–14: 210 merged PRs (15/day average), 100% authored by Casey Allard, 136 from `codex/*` branches. **No commits to main and no ref changes on Sept 15–16** (as of ~19:00Z on 09-16). Peaks: 09-08 (#1195 studio integration + v0.1.0-alpha tag) and 09-13 (geometric-attention/addressed-attention series #1236–#1262).

---

## Raw files

`/home/claude/work/reports/github-raw/`: `ls-remote-live-2026-09-16.txt` (1,595 refs), `gitlog_main.tsv`, `merged_prs_from_git.json` (830 PRs), `analysis.json`, `analyze.py`, `pages_uor-foundation.html`, `pages_casey.html`, `pages_uor_model_worker.js`, `pages_r4_worker.js`, `pages_*_headers.txt`, `readmes/*.md` (21 READMEs), `census-bodies/issue-*.md` (11 snapshot bodies), and the 403 API responses (`repo.json`, `issues_*.json`, `pulls_*.json`, `releases.json`, `tags.json`, `actions_*.json`, `contributors.json`, `org_repos.json`, `casey_repos.json`, `afflom_repos.json`, `c820.json`, `c973.json`).


---
**Addendum (main session, 2026-09-16, via github.com page fetch):** repository page fetch showed 8 stars, 2 forks, but two independent crawler snapshots the same day show 15 stars, 4–5 forks — treat the live count as ~15/4–5; issues tab shows 12–15 open depending on render; #1172 and #1173 are open, labeled `roadmap`, authored by Casey-allard, no comments; #1139 open (labels enhancement, roadmap, roadmap:next; milestone "Native Rust geometric intelligence — conversation, memory, reasoning and coding"); #973 and #820 have no comment threads — decision history lives in their edited bodies; #234 (maurathat) is closed. Ari = GitHub `auser` (NOT Alex Flom/afflom) — correct the report's "Ari (= Alex Flom)" line.
