# Outside-Contributor Journey Audit — UOR-R4

**Verdict: the project is welcoming in intent and hostile in structure.** The documentation is unusually honest and thorough for a pre-alpha research repo, but a newcomer following CONTRIBUTING.md literally hits a wall within the first ten lines, and the one real outside contributor of the last month experienced a 5.5-hour, five-point rejection followed by a maintainer rewrite of their branch.

**Professionalism grade: C+** — B+/A− for the quality of the underlying documentation, D for the GitHub surface a visitor actually lands on, and C− for the newcomer on-ramp. Nothing here needs research work; it is roughly two days of writing and repo settings.

---

## 1. CONTRIBUTING.md — accurate but misdirecting

**What is correct post-#2643 and post-D37** (verified, not assumed):

| Claim | Location | Verified against |
|---|---|---|
| Rust 1.97.1 pinned, rustup-selected | `CONTRIBUTING.md:22` | `rust-toolchain.toml` → `channel = "1.97.1"` |
| `cargo check/test --locked -p uor-r4-discrete --lib` | `CONTRIBUTING.md:24-25` | workspace member `crates/uor-r4-discrete`; crate README repeats the same two commands verbatim |
| `cargo run -q --locked -p xtask -- figures` | `CONTRIBUTING.md:26` | `xtask/src/main.rs:52` → `"figures" => figures::run(...)` |
| `cargo run -q --locked -p uor-r4-claim-wording -- .` | `CONTRIBUTING.md:48` | crate exists, is a workspace member |
| D37 summary (offline float, matmul/softmax banned, integer serving) | `CONTRIBUTING.md:3` | matches D37 and the README header |
| "`required-transport` executes no tests" | `CONTRIBUTING.md:49` | `.github/workflows/ci.yml:118-124`, job name is literally *"required acknowledgements (5 ruleset names, no tests)"* |
| All `scripts/` and `docs/` paths | throughout | all 18 checked paths exist |

**Findings, worst first.**

**C-1 — The only "what to work on" pointer aims at blocked work.** `CONTRIBUTING.md:29-32` names M6 (#2034), M5 (#2033) and M7 (#2035) as "Good first areas". All three carry `status:not-started`, and each declares a hard dependency on the model that does not exist:

- #2033: *"depends on #2032 M4"*
- #2034: *"depends on the shared native model from M1–M4"*
- #2035: *"Expose the accepted `D11` integer model artifact (#2032 M4)…"*

This is not a theoretical flaw. The maintainer review on PR #2654 blocked the contributor partly on *"M7 (#2035) is gated on M4 serving under D11/D32"* (comment `IC_kwDOS1sZc8AAAABdHGTqQ`, finding 5). CONTRIBUTING.md sent them there.

**C-2 — Step 2 of the workflow is impossible for anyone outside the org.** `CONTRIBUTING.md:41`: *"add the label `lab:claude`, `lab:codex` or `lab:deepseek` … and set `status:in-progress`."* `Cyborg-Hawk-AI` is `author_association: CONTRIBUTOR` on a fork with `maintainer_can_modify: false`. They have no triage rights. There is no documented "human/outsider" path for the claim step — the parenthetical *"(humans: a comment suffices)"* is easy to miss and still says nothing about who applies the label.

**C-3 — The harness section is unbuildable for a newcomer.** `CONTRIBUTING.md:102-129` presents `uor-harness status|brief|prep|check|lint` as the entry point. `uor-harness` is a locally compiled Rust binary — on this machine `~/.cargo/bin/uor-harness`, built per `AGENTS.md` via `CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness`. CONTRIBUTING never says it must be built. A newcomer gets `command not found` at the first command in the section. Compounding this, `CONTRIBUTING.md:119` switches mid-section to `cli.py skill use` / `cli.py team dispatch` while lines 109-113 use `uor-harness` — two entry points, no explanation, left over from the Rust port (#2337). `docs/harness/README.md:107` itself still shows bare `uor-harness status` under a heading that promises *"no install"*.

**C-4 — Lab-only infrastructure presented as universal.** Lines `45`, `61`, `137-138`, `154`, `157-162` require `uor-pod` (cloud credentials + owner provisioning), `cloud-store` (the owner's iCloud store, defined only in `AGENTS.md:438` as `~/.local/share/uor-r4/bin/cloud-store`), and `--lab <lab>` where the legal values are `kimi|claude|codex|deepseek` (`docs/harness/README.md:103`). There is no `human` or `external` value and no statement that these are unavailable to outsiders.

**C-5 — The hard rules are not stated.** CONTRIBUTING references only D37, D11, D5, D9, D14, D24/D25. The three decisions that actually blocked PR #2654 — **D23** (no softmax anywhere, including objectives, sampling and teachers), **D25** (no dense matmul in training or serving), **D29** (no banned mechanism in *any compiled or loaded module*, presence counts) — appear nowhere. The single relevant sentence is a blockquote fragment at line 3 ("retaining matmul/GEMM and softmax/squashing bans"). To learn them a contributor must open `docs/integration/DECISIONS.md` — 1,777 lines, 38 decisions, D0 through D37, with no index of which ones bind a contribution.

**C-6 — No acquisition path for data or artifacts.** The contributor's own issue #2655 states: *"The parity run needs the S4-arm artifact and the chat-v0 held-out set, which I do not have."* CONTRIBUTING explains how to *store* results (`cloud-store put`) but never how to *obtain* inputs. README lists Hugging Face repos; CONTRIBUTING never connects them.

**C-7 — The `workflow` label does not exist.** `CONTRIBUTING.md:189`: *"Open an issue labelled `workflow`."* All 30 repo labels enumerated; no `workflow`. Following this instruction is impossible.

**C-8 — Step 1 sends humans to an AI-lab document.** `CONTRIBUTING.md:20` → `docs/labs/README.md`, titled *"Joining the UOR-R4 research labs"*, which is a client-onboarding doc for Codex/Claude/Anti-Gravity/OpenCode with a "Read these in order" list beginning with the 460-line `AGENTS.md` and client prompt packets. A human contributor is the wrong audience for every line of it.

**C-9 — Build/test section omits the CUDA reality.** `CONTRIBUTING.md:22-27` gives three commands that all run on the default feature set, which `crates/uor-r4-discrete/README.md` candidly says *"has no CUDA model execution."* A newcomer can pass every documented command and still never execute the model. That is fine as a boundary, but CONTRIBUTING doesn't say it — the crate README does.

---

## 2. Repo surface — bare

| Surface | State | Evidence |
|---|---|---|
| Description | **empty string** | `gh repo view --json description` → `""`; `community/profile` → `"description": null` |
| Topics | **none** | `repositoryTopics: null` |
| Homepage | set, resolves 200 | `https://uor-foundation.github.io/uor-r4/` — but the page titles itself *"UOR-R4 Sovereign AI Developer Studio"*, not the model |
| Default branch | `main` | correct |
| License | MIT, detected | `LICENSE`, community profile `license.key = mit` |
| CITATION.cff | present and well-formed | root, CFF 1.2.0, keywords, authors |
| Issue templates | 2 forms on disk | `.github/ISSUE_TEMPLATE/{implementation,research-experiment}.yml` — but `gh repo view --json issueTemplates` returns `[]`, and `community/profile` reports `"issue_template": null`. No `config.yml`. |
| PR template | present, 1 file | `.github/pull_request_template.md`, heavyweight |
| CODE_OF_CONDUCT | inherited only | `https://github.com/UOR-Foundation/.github/blob/main/CODE_OF_CONDUCT.md` |
| SECURITY | inherited only | org `SECURITY.md` (1,668 B); repo has none |
| Discussions | **disabled** | `hasDiscussionsEnabled: false` |
| Wiki | enabled, empty | `hasWikiEnabled: true` |
| CODEOWNERS | absent | no review routing |
| Stars / forks / watchers | 15 / 5 / **0** | |
| Open issues | 10, all internal | #2028–#2037 — roadmap, milestones, compute board |

**Labels.** 30 labels, well-structured for internal use (`roadmap:*`, `status:*`, `lab:*`, `milestone`, `compute`) and useless for outsiders. The two GitHub defaults aimed at newcomers are **used on zero issues, ever**: `gh issue list --label "good first issue" --state all` → `[]`; same for `help wanted`. There is no crate-scope label, no `question` usage, no triage label, and no `workflow` label despite CONTRIBUTING pointing at it.

**Net:** a researcher arriving from a paper, a post, or a search finds an unnamed repository with no topics, no tagline, no discussions, and an issue list of ten internal planning tickets. The README badge block is genuinely good — license, Rust 1.97.1, pre-alpha status, `0 of 8 milestones complete`, last commit, HF link — and `CITATION.cff` exists. That is the entire professional surface.

---

## 3. Navigation — good docs, one contradiction, one dead end

| Doc | Size | Assessment |
|---|---|---|
`docs/architecture.md` | 53 lines | **Excellent.** "Follow one token" table maps every arrow to a source file with line-level honesty about what each does and does not do. Best newcomer document in the repo. |
`crates/uor-r4-discrete/README.md` | ~50 lines | **Excellent.** Repeats CONTRIBUTING's two commands verbatim, states the CUDA/default-feature boundary, names D37/D11 honestly, links current evidence. |
`docs/README.md` | 543 lines | Good "Find your way" table (lines 8-19), then ~480 lines of historical research map. A newcomer reads the table and stops; fine, but the page is 90% sediment. |
`docs/PROJECT_MAP.md` | 263 lines | Opens well with *"Current compiled navigation — 18 October 2026"* and a 4-row table, then immediately reverts to dated narrative from September 25 onward. |
`crates/uor-r4-core/README.md` | — | **Direct contradiction.** Line 3: *"Cargo now selects `src/native_support.rs`… Historical core model/learner/session sources below are retained unchanged but **excluded from this library build**."* Line 6, under "## Historical implementation and evidence": *"**Active native model:** `native_geometric` implements the Rust training/artifact/session path used by `r4 geometric`."* Calling a source "Active" one line after declaring it excluded will mislead every newcomer who reads it. |

**The traced journey, exactly as CONTRIBUTING prescribes:**

1. `CONTRIBUTING.md:20` → read README, STATUS.md, `docs/labs/README.md`, PROJECT_MAP → **hits an AI-client onboarding doc**. (Dead end for humans.)
2. `CONTRIBUTING.md:22-27` → build commands → **all succeed**, and `uor-r4-discrete` is genuinely buildable and testable without CUDA. (This part works.)
3. `CONTRIBUTING.md:29-32` → pick a "good first area" → **all three blocked on M1–M4**.
4. `CONTRIBUTING.md:33` → "Ask on the milestone issue" → milestone issues are `status:in-progress` and owned by `lab:deepseek` / `lab:codex` / `lab:claude`. **No open door.**
5. `CONTRIBUTING.md:102-129` → `uor-harness status` → **command not found**.

Steps 1, 3, 4 and 5 are dead ends. Only step 2 (compile and test a crate) actually works, and it is the one step that requires no permission.

---

## 4. The outside-collaborator trail — what actually happened

**Timeline, 2026-10-18 (all UTC):**

| Time | Event |
|---|---|
| 09:58:29 | PR #2654 opened by `Cyborg-Hawk-AI` (`CONTRIBUTOR`, fork, `maintainer_can_modify: false`). **1,998 additions, 36 deletions, 29 files, no labels, no draft state.** |
| 09:58:37–43 | Issues #2655, #2656, #2657 opened — within 15 seconds, **no labels, no templates used** (titles lack the `[impl]`/`[research]` prefixes the forms require). |
| 10:06 | First review, `CHANGES_REQUESTED`, **8 minutes after open** — a single monolithic comment with five blocking findings. |
| 15:18:59 | `APPROVE` by `Casey-allard` at head `c921e0b56`, after the maintainer rewrote the branch (`667db6749..c921e0b56`). |
| 15:20:59 | Merged by `Casey-allard` via merge queue. |
| 15:26 | All three issues closed with near-identical boilerplate thank-yous redirecting to #2032/#2033. |

**What the reviewer blocked on** (`IC_kwDOS1sZc8AAAABdHGTqQ`), all of which is documented *somewhere*, none of which is in CONTRIBUTING:

1. **D29 facade bypass** — a renamed `serving-wasm` feature re-exposed the retired historical CPU runtime. *Documented in `AGENTS.md`, D29, and the crate's own `#[cfg] compile_error!`.*
2. **D11/R2 multiplier violations** — 2 `i64.mul` in the emitted wasm, plus `i32x4_dot_i16x8` in `stack/simd.rs`.
3. **Fatal `simd128` arithmetic bugs** — `swizzle_dot_arm_t` indexed bytes not elements; `dot_dot_arm_m` computed a dot product instead of a table lookup; both passed only because `target_feature = "simd128"` is false on x86_64, so tests silently ran the scalar fallback.
4. **Broken PRNG normalization** in `xtask/src/qubo_codec.rs` — `>>33` divided by `u64::MAX`, freezing every weight at `-0.2`.
5. **Repository policy, security and delivery-cadence violations** — `ops/measure_final_rnd.py` calling `sudo -S perf` with a **hardcoded password (`input='test\n'`) and a hardcoded `/home/yaz/loop.sh`**; **846,868 bytes of committed binaries** (`pkg-serving/synthetic-stack.lut`, `uor_r4_bg.wasm`); no claim in `docs/harness/claims.json` (confirmed: zero matches for `cyborg|bai/|wasm-serving`); no `docs/labs/<topic>-<date>/README.md`; no `current-state.md` entry.

**Friction a better CONTRIBUTING or templates would have prevented:**

- **The gating error was CONTRIBUTING's fault, not the contributor's.** They picked M7 because the guide named it.
- **No "never commit" list.** Binaries, Python scripts, and scratch files (`handoff/work-cards.md`) all landed. `.gitignore` covers some patterns but the contributor had no stated rule.
- **The D27 "Rust default for tooling" rule is invisible.** `CONTRIBUTING.md:183` says *"use Rust for all model code (no Python model implementation or dependency)"* — a measurement script is not "model code", so the contributor reasonably read it as allowed. D27 covers *all* new tooling.
- **No secrets guidance anywhere in CONTRIBUTING**, and no repo-level `SECURITY.md`. The hardcoded sudo password is exactly what a "never commit credentials; report privately here" line prevents.
- **The delivery cadence is documented twice** (`CONTRIBUTING.md:64-100` and the PR template checklist) **and enforced zero times.** The contributor left every box unticked and nothing failed; the PR merged anyway after a maintainer remediation pass.
- **Zero inline review comments.** `gh api repos/.../pulls/2654/comments` → empty. Five blocking findings on a 1,998-line diff were delivered as one wall of text with no file anchors, so the contributor could not work through them incrementally.
- **Responsiveness was good but the ending was not the contributor's.** Eight minutes to first review is excellent. But the branch was ultimately hardened and merged by the maintainer, and the contributor's three follow-up issues were closed 8 minutes after merge with identical boilerplate. They are credited in `README.md:387` ("Yazan Barakat ([@Cyborg-Hawk-AI])… PR #2654") — that part is done right.

---

## 5. Static build check — documented paths

**Verified present and correct:** `rust-toolchain.toml` (1.97.1), `crates/uor-r4-discrete` as workspace member, `xtask` with a real `figures` dispatch (`xtask/src/main.rs:52`), `crates/uor-r4-claim-wording`, `scripts/storage/uor-hygiene`, `scripts/pod/uor-pod`, `scripts/storage/research-data`, `scripts/storage/worktree-audit`, `docs/labs/working-in-one-checkout.md`, `docs/labs/compute.md`, `docs/harness/README.md`, `docs/harness/CONTRACT.md`, `docs/labs/README.md`, `docs/PROJECT_MAP.md`, `docs/architecture.md`, `docs/formal_vocabulary.md`, `docs/geometry.md`, `STATUS.md`, `ROADMAP.md`, `docs/labs/protocol.md`, `docs/labs/session-goal.md`, `LICENSE`, `CITATION.cff`. **No documented in-repo file is missing.**

**Documented but not obtainable/executable:**

| Item | Line | Problem |
|---|---|---|
| `uor-harness` | 109-113, 137 | No in-repo binary, no build step. Only exists at `~/.cargo/bin`. |
| `cli.py skill use` | 119 | Mixed entry point; `scripts/harness/cli.py:593` does implement `skill use`, but the binary is the documented front door. |
| `cloud-store` | 61, 154-155 | Owner machine only; defined solely in `AGENTS.md:438`. |
| `uor-pod` | 45, 157-162 | Script exists; requires Vast/Runpod credentials and owner provisioning. |
| `workflow` label | 189 | **Does not exist.** |
| `docs/harness/claims.json` claim | not mentioned | File exists; CONTRIBUTING never references it. Reviewer cited its absence. |

**Minor:** CONTRIBUTING:112 uses `--body-file` while the PR template uses `--body`; both are accepted (`crates/uor-r4-harness/src/main.rs:128-129`), so this is style drift, not a defect.

---

## 6. Against professional open-research norms — ranked by trust impact

| # | Gap | Impact on an outside researcher |
|---|---|---|
1 | **Empty repo description, zero topics** | Highest. Ten seconds on the page and there is no statement of what this is. Search cannot find it; GitHub search by topic never surfaces it. |
2 | **No newcomer-addressable work** | The three recommended milestones are blocked; `good first issue` and `help wanted` are unused; all 10 open issues are internal planning tickets. The contributor's only route was to invent a 2,000-line PR. |
3 | **No Discussions tab** | No low-stakes place to ask "is this wanted?" before writing code. The only channel is a milestone issue owned by a lab. |
4 | **Hard rules are not readable in one place** | D23/D25/D29 blocked a real PR and are invisible from CONTRIBUTING. A 38-decision, 1,777-line log is not an onboarding document. |
5 | **No repo-level SECURITY.md and no secrets rule** | A contributor committed a hardcoded sudo password. Org-level inheritance exists but is not signposted. |
6 | **No CoC in-repo** | Inherited and functional; invisible unless you know to look at the org. |
7 | **PR template unenforced** | A 12-item checklist with no required check is decoration. It did not survive first contact with an outsider. |
8 | **Issue templates not surfacing** | API reports `issueTemplates: []` and `issue_template: null`. Both forms are heavy research instruments with no lightweight "question"/"bug" path, and no `config.yml` to route lost first-timers. |
9 | **`workflow` label missing** | CONTRIBUTING's own escalation path is a 404. |
10 | **Empty wiki, no CODEOWNERS** | Minor surface noise; no review routing for external PRs. |
11 | **CI gives an outsider nothing** | A normal PR runs only `required-transport` — five green checkmarks that execute no compile. The contributor got no automated signal that 2,000 lines did not build. |

**Credit where due:** the README badge block, `CITATION.cff`, `docs/architecture.md`, `crates/uor-r4-discrete/README.md`, the honesty of the status claims, the 8-minute first response, and the README acknowledgement are all above the norm for a pre-alpha research repository.

---

## 7. Ranked fix list — hand to an implementer as-is

### P0 — do first, all cheap

**FIX-1. Repo settings (no file changes, ~5 minutes).**
- Set description: `"A geometric language model: pre-alpha research toward transformerless local AI with exact addressed memory. 0 of 8 milestones complete."`
- Add topics: `rust`, `language-model`, `geometric-deep-learning`, `research`, `pre-alpha`, `wasm`, `multiplier-free`, `local-ai`, `quaternion`, `cuda`.
- Confirm homepage `https://uor-foundation.github.io/uor-r4/`.
- Enable Discussions with categories `Announcements`, `Q&A`, `Ideas`, `Show and tell`.
- Disable the empty wiki.

**FIX-2. Fix the misdirection in `CONTRIBUTING.md:29-32`.** Replace the "Good first areas" list with a truthful, self-updating block:

```markdown
## What is actually open

**Model work is gated.** M1–M4 (#2029–#2032) own the model; M5–M8 each declare a
dependency on the M4 served model and are `not-started`. Do not start model work
without a maintainer saying a slot is open.

**Genuinely open now:**
- Documentation and claim-wording fixes (open an issue first).
- Reproducing a published negative result and reporting a discrepancy.
- Tooling and CI improvements that do not touch the model path.
- Reading `docs/integration/current-state.md` and identifying a cause.
- Issues labelled `good first issue` — ask and one will be seeded for you.

Ask on #2028 before starting. A maintainer will tell you within a day whether the
work is wanted; if it is not, that is a normal answer, not a rejection.
```

**FIX-3. Add `docs/contributing/HARD-RULES.md`** — one page, twelve rows, the only document an outsider must read before writing code. Columns: *Rule | Decision | What it means in your diff | How to check*. Must include at minimum:

| Rule | Ref |
|---|---|
No softmax, softplus, sigmoid or tanh anywhere — including losses, softmax-in-cross-entropy, sampling, teachers, and renamed/table forms | D23 |
No dense matrix multiplication in training or serving | D25 |
No banned mechanism in any *compiled or loaded module* — presence counts, not just execution | D29 |
Serving is integer/table only: no float, no multiplier instruction in the declared kernel | D11 |
Offline training may use float, but must report its D11 export gap | D37 |
New code and tooling default to Rust; Python only where Rust genuinely cannot | D27 |
Never commit model binaries, corpora, wasm bundles or generated artifacts | `.gitignore` |
Never commit credentials, passwords or absolute local paths | — |
Model compute is GPU-only; no CPU fallback | D24 |
Every claim names artifact, data split and comparison | `docs/formal_vocabulary.md` |

Link it as the first line after the intro in CONTRIBUTING.md and from `.github/pull_request_template.md`.

**FIX-4. Add `docs/contributing/OUTSIDE.md` — "Contributing without lab access."** Two-column table: *You have* / *You do not have*. State plainly: you do not need `uor-harness`, `uor-pod`, `cloud-store`, or a `lab:*` label; do not run them; a maintainer will run the gates for you. Give the exact minimum viable loop:

```sh
git clone https://github.com/UOR-Foundation/uor-r4 && cd uor-r4
cargo check --locked -p uor-r4-discrete --lib
cargo test  --locked -p uor-r4-discrete --lib
cargo run   -q --locked -p xtask -- figures
cargo run   -q --locked -p uor-r4-claim-wording -- .
```

**FIX-5. Add `SECURITY.md`** at repo root (even if it mostly delegates to the org policy): supported versions (pre-alpha, none), private reporting via GitHub Security Advisories, explicit "never commit credentials, tokens, passwords or absolute home paths", and a 72-hour acknowledgement commitment.

### P1 — before the next outside PR

**FIX-6. Create the missing `workflow` label** (colour `#1D76DB`, description "Change to CONTRIBUTING.md, AGENTS.md or the contribution workflow"). `CONTRIBUTING.md:189` currently points at a label that does not exist.

**FIX-7. Add `.github/ISSUE_TEMPLATE/config.yml`:**
```yaml
blank_issues_enabled: true
contact_links:
  - name: Ask a question
    url: https://github.com/UOR-Foundation/uor-r4/discussions
    about: Not sure the work is wanted? Ask here before writing code.
  - name: Roadmap and current state
    url: https://github.com/UOR-Foundation/uor-r4/issues/2028
    about: What is active, gated, and blocked.
```
Plus a lightweight `question.yml` and `docs.yml` form. The two existing forms are research instruments; a newcomer has no small door.

**FIX-8. Add a `docs/contributing/`-style outside path to the PR template.** Keep the research checklist for lab PRs, but put a three-line fork at the top:

```markdown
> **Outside contributor?** Use this shorter checklist and ignore the lab items below:
> - [ ] I ran `cargo check`, `cargo test` and `uor-r4-claim-wording` locally and pasted the output
> - [ ] No binaries, corpora, generated artifacts or credentials in the diff
> - [ ] `git grep -n -i '<what changed>' -- '*.md'` and stale docs updated
> - [ ] I understand model work may be gated; a maintainer will confirm or redirect
```

**FIX-9. Seed `good first issue`.** There is no venv of open work. Concretely available today from this audit: fix `crates/uor-r4-core/README.md`'s "Active native model" contradiction; add the `workflow` label usage docs; write the `uor-harness` build line into CONTRIBUTING; correct `CONTRIBUTING.md:119` from `cli.py` to `uor-harness`; reconcile `--body` vs `--body-file`. Five real, self-contained issues.

### P2 — structural

**FIX-10. Fix `crates/uor-r4-core/README.md`.** Line 6's "**Active native model:** `native_geometric` implements…" directly contradicts line 3's "excluded from this library build". Rename the heading to "Historical implementation and evidence" scope and change the first sentence to *"**Historical native model:** `native_geometric` implemented…; Cargo does not select it."*

**FIX-11. Make CONTRIBUTING's harness section honest.** Precede it with:

> The harness is a locally compiled helper (`cargo build -p uor-r4-harness`). If you do not have it, skip this section — a maintainer will run these gates for you. Nothing here blocks a documentation or tooling contribution.

Then standardise every invocation to `uor-harness` and delete the `cli.py` references at lines 119-125.

**FIX-12. Redirect `CONTRIBUTING.md:20`.** Point humans at `docs/architecture.md` and `docs/PROJECT_MAP.md`'s "Current compiled navigation" table — not `docs/labs/README.md`, which is an AI-client onboarding document. Add one line: *"`docs/labs/` is for the AI research labs; you do not need it."*

**FIX-13. Give CI a real signal on PRs.** Add a `cargo check --locked -p uor-r4-discrete --lib` job for PRs touching `crates/**` or `Cargo.*`. Today a normal PR gets five green acknowledgement checks that compile nothing — the single most misleading signal an outside contributor can receive.

**FIX-14. Add `.github/CODEOWNERS`** routing `docs/`, `crates/`, `.github/` to the maintainer, so external PRs auto-request review rather than sitting unattributed.

**FIX-15. Add a "Getting data" paragraph to CONTRIBUTING §Data.** State what is public (the HF model and dataset cards named at `README.md:316-338`), what is not (the sealed S4-arm artifact, `chat-v0` held-out panels), and what to do instead — ask on #2028 and a maintainer will scope a reproduction that uses public inputs.

**FIX-16. Add the acquisition/enforcement gap to the PR checklist.** Either make D22/D23/D25 ratchets required checks (`uor-harness ratchet d23`, `d25`), or state explicitly in CONTRIBUTING that they are advisory and a maintainer will run them. The current template implies enforcement that does not exist.

---

### One-line summary for the PM

The single highest-leverage change is **FIX-2 plus FIX-3**: stop telling outsiders to work on M5/M6/M7, and give them a one-page statement of the hard rules. Those two edits would have changed the entire shape of PR #2654 — the contributor would have picked a tooling task instead of a 2,000-line WASM runtime, and would have known not to commit binaries, a Python script, or a hardcoded password.