# Fourth-lab integration review

September 26, 2026. References #973 under #820. **Source/evidence review, not a
new model result.** Main was refreshed to
`4b62608702481abd3f18915dc3718e311b86b8aa`. PR #1400 was OPEN at
`474e98665e77c20283d9b63f5ebb5f20ace29b18`. Worktree/process observations below
are a dated snapshot and must be refreshed before acquiring ownership.

The fourth lab should integrate the useful work of the concurrent labs while
independently checking what each result establishes. Shared GitHub issues and
isolated worktrees are the owner-selected coordination method. This review
changed no occupied implementation, ran no build or model, and did not rerun
the advertised test suites.

## Current programme and occupied work

The [completed language continuation](language-continuation-result-2026-09-26.md)
improved natural likelihood but missed its fixed prose criteria: continuous
0/5 in both arms, integer 0/5 quaternion and 1/5 ordinary. Neither candidate is
promoted. The [current work card](../history/current-state-2026-09-25-to-2026-10-02.md#executed-same-checkpoint-emissionselection-diagnostic-september-27)
prioritizes emission/selection localization with existing checkpoints; another
exposure-only fit is not justified by that result. The owner checkout remained
at September 25 commit `413a32fc`, whose next-action text was stale. The
[canonical plan](project-track.md) and refreshed main take precedence over that
checkout and copied historical briefs.

| Observed track | Evidence and ownership boundary |
|---|---|
| Google/Antigravity, `codex/geometric-chatbot` | Worktree `/Users/casey.allard/uor-r4/.worktrees/geometric-chatbot`, open [PR #1400](https://github.com/UOR-Foundation/uor-r4/pull/1400), dirty integer session/model/generation/math and benchmark files. During inspection, cargo PID68801 was a child of Antigravity language-server PID41152; `lsof` bound its cwd to this worktree. This supports active Google ownership at that observation, not perpetual ownership or a completed result. |
| `codex/canonical-address-routing-20260925` | Head `50ac2527`, with dirty `dialogue.rs`, `joint_model.rs`, `joint_parallel.rs`, `lib.rs` and new `joint_scan.rs`. Commit history records R1b/R1c dialogue learning and the R2 scan design. The provider/session owning these edits was not established. |
| Active chat-v0 training | PID22596, `uor-r4-training-metalacc`, had cwd `/Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/r1d` and remained running on refresh. It must be included in machine resource coordination. Its parent was PID1; that does not identify a provider or prove the training result. |
| `codex/emission-localization-20260926` | Existing worktree at refreshed main, clean when inspected. Treat it as reserved pending the owning chat's status; a clean worktree does not establish availability. |
| `codex/geometric-lm-goal` | Staged/untracked tests and research documents across core/integer/evaluation. Preserve it. Provider ownership and completion were not verified. |
| Other labs and owner material | Claude and OpenCode applications were running, but application presence did not map their current tasks to branches. The owner checkout contained intentional dirty configuration/team files and untracked `.agents/teamwork/`, `.omo/`, `.opencode/` and research material. None is disposable or a fourth-lab editing surface. |

Record each lab's issue, worktree/head, owned paths, active artifact/process and
next decision in the shared coordination surface. Do not infer ownership from a
branch prefix, author name, application presence or old handoff. The fourth lab
can work on the shared dialogue protocol without modifying the occupied
session and training modules.

## PR #1400: useful engineering and unsupported capability interpretation

The PR describes an interactive conversational chatbot, 80% five-turn entity
recall, a large NoRead perplexity ratio and a completed independent audit. Its
**committed source** supports a narrower interpretation. These findings are
bound to the reviewed head, not to the subsequent dirty continuation.

| Claimed interpretation | Executed-test construction visible in source |
|---|---|
| Learned conversational recall | The [bundle factory, lines191–194](https://github.com/UOR-Foundation/uor-r4/blob/474e98665e77c20283d9b63f5ebb5f20ace29b18/crates/uor-r4-integer/tests/conversational_benchmarks.rs#L191-L194) constructs `IntegerModel::synthetic_for_test()`. The 80% test uses it at lines422–424. It does not load the retained learned language artifact. |
| Correct requested entity after five turns | [Lines463–493](https://github.com/UOR-Foundation/uor-r4/blob/474e98665e77c20283d9b63f5ebb5f20ace29b18/crates/uor-r4-integer/tests/conversational_benchmarks.rs#L463-L493) choose the entity token with the largest measured memory mass among matching dialogue occurrences. [Lines528–537](https://github.com/UOR-Foundation/uor-r4/blob/474e98665e77c20283d9b63f5ebb5f20ace29b18/crates/uor-r4-integer/tests/conversational_benchmarks.rs#L528-L537) pass on token presence, positive read mass and probability uplift. Neither criterion requires a generated complete answer or identification of the original fact occurrence. |
| Measured zero correct NoRead answers | The [telemetry at lines561–568](https://github.com/UOR-Foundation/uor-r4/blob/474e98665e77c20283d9b63f5ebb5f20ace29b18/crates/uor-r4-integer/tests/conversational_benchmarks.rs#L561-L568) explicitly writes `recall_pass: false` for NoRead. This field is not a measured generated-answer score. |
| Actual generated answers qualify the chatbot | The separate [string oracle at lines633–643](https://github.com/UOR-Foundation/uor-r4/blob/474e98665e77c20283d9b63f5ebb5f20ace29b18/crates/uor-r4-integer/tests/conversational_benchmarks.rs#L633-L643) counts correct answers, but its [final assertion at lines677–681](https://github.com/UOR-Foundation/uor-r4/blob/474e98665e77c20283d9b63f5ebb5f20ace29b18/crates/uor-r4-integer/tests/conversational_benchmarks.rs#L677-L681) checks only absence of forbidden distractors. Zero correct answers can satisfy that assertion. |

These tests can exercise memory, probability and session plumbing. They do not
establish useful learned conversation. Preserve the streaming/session work and
its engineering evidence; correct its claim scope before integration. Repeating
the same tests cannot resolve this mismatch. The relevant next evidence is the
actual selected learned bundle generating complete responses under a compatible
dialogue protocol, with Read and NoRead judged using the same answer criterion.

## Geometry and architectural hypotheses needing sharper boundaries

Untracked teamwork reports in the owner checkout reason from loss of a Hopf
fiber to claimed causes of entity confusion and cyclic text, and from 2,000
distinct phase states to “ergodicity.” Relevant local records are
`.agents/teamwork/teamwork_preview_explorer_m2_2/handoff.md:95–101,115–117` and
`.agents/teamwork/teamwork_preview_challenger_chatbot_m1_it2_2/handoff.md:122–142,156–158`.
Those language-causal conclusions do not follow from the reported checks.

Distinct internal states need not produce distinct outputs; an incrementing
counter can pass a noncollision check without influencing prediction. A finite
noncollision trace is not an ergodicity proof. The committed
[topological-loop test](https://github.com/UOR-Foundation/uor-r4/blob/474e98665e77c20283d9b63f5ebb5f20ace29b18/crates/uor-r4-integer/tests/adversarial_topological_loops.rs#L41-L73)
uses a synthetic model and supplied assistant tokens. Retained fiber and
orientation may be useful representations, but their proposed language role
needs a traced predictive path and a relevant intervention. This observation
does not reject Hopf geometry as a research direction.

The active R2 branch is also a **model change**, not an exact speedup of the old
cell. Its committed
[scan design](https://github.com/UOR-Foundation/uor-r4/blob/50ac2527f95508de114080d36e76826d7781a3b5/docs/integration/chat-r2-chunked-scan-design-2026-09-25.md#L8-L14)
acknowledges that distinction. The inspected uncommitted `joint_scan.rs:9–26`
uses a token-driven recurrence and does not feed the read/output update back
into it; lines56–62 permit a local32/64 window. A scan/sequential parity check
can validate the new cell's implementation, but cannot establish retention of
the old model's full causal reads. The design's
[local-window rationale](https://github.com/UOR-Foundation/uor-r4/blob/50ac2527f95508de114080d36e76826d7781a3b5/docs/integration/chat-r2-chunked-scan-design-2026-09-25.md#L60-L66)
must be reconciled with main's deferred-admission policy before any promotion.
Discovery may proceed on its own declared branch and comparisons; keep the
accepted full256 artifacts intact.

## Integration recommendations

1. **Agree on one dialogue protocol before joining implementations.** Bind the
   protocol version to tokenizer identity, role markers, boundary encoding,
   response-loss mask, session context/eviction semantics and artifact metadata.
   A marker string or numeric role ID in one synthetic vocabulary cannot be
   assumed to mean the same thing in a learned tokenizer. Require an explicit
   compatibility check or migration; preserve historical artifacts unchanged.
2. **Review generated behavior early.** Separate session mechanics, retained
   learned capability and geometry attribution. The first relevant loaded
   transcript should precede expansion of a benchmark suite. Source presence,
   positive read mass and a model being called are not complete answers.
3. **Acquire only unoccupied work.** Use shared issues plus isolated worktrees;
   coordinate active model processes and cargo usage across labs. Read other
   labs' source and receipts without editing their active files. Reconcile
   materially different objective/context/artifact contracts before integration.
4. **Use contextual specialists, not ritual review waves.** Every packet needs
   the programme goal, current retained artifact, relevant negative history,
   exact source/callers, affected neighbors and the decision it can change.
   Reuse the existing [team packet](../../.kimi-code/TEAM.md) and
   [anti-loop protocol](../../.kimi-code/RESEARCH-PROTOCOL.md). Reconcile their
   DeepSeek-default routing with the local OpenCode Kimi mapping before claiming
   a provider policy is consistently installed; do not alter another lab's route
   silently.
5. **Keep discovery adaptable and promotion honest.** A new hypothesis can earn
   an independent prototype or a prospectively revised research question. It
   cannot retroactively turn a failed gate into a pass. For each bug or repeat,
   identify the affected deliverable/invariant and the distinct decisions the
   result can change; park unrelated failures after recording them.
6. **Follow the owner priority through one integrated model.** Geometric
   attention, inference, prose, chat and reasoning are linked responsibilities.
   Use the existing emission-localization and chat-v0 work to choose the next
   learning change; do not start competing exposure or topology campaigns from
   the stale September25 next-action text. Retain ordinary controls and measure
   useful quality with complete laptop cost when claiming efficiency.

The immediate fourth-lab contribution is a shared artifact-compatible dialogue
protocol and an evidence-based integration decision. The open questions are the
current owner/session of the R1d/R2 track, Claude/OpenCode's exact active paths,
and any newer branch-specific owner decisions. None prevents independent
protocol design or the claim-scope correction identified above.
