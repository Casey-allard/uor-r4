# Principal attention investigation — shared review contract

Status: investigation complete, 2026-09-24. This is a review/planning milestone, not a model qualification or an adopted policy change.

## Owner request and common question

Investigate the whole UOR-R4 Geometric Language Model, including historical and non-serving mechanisms, and synthesize the strongest coherent route to useful transformerless geometric attention, exact input-history memory, and multiplier-free serving. Develop first-principles mechanisms and use internal knowledge and primary external literature. Do not replace the investigation with isolated test repairs. Distinguish an attractive hypothesis from a demonstrated advantage and a complete integrated model from separately measured components.

## Live boundary

- Source reviewed: `3101c06061726d9438a22649a25b98558525e8d0`, isolated full worktree `/Users/casey.allard/uor-r4-worktrees/kvar-20260924`, review branch `codex/principal-attention-plan-20260924`.
- Refreshed protected `origin/main`: `552d847d49fb263966165004b835f2f53cccaae1`. The recent #1380–#1384 work is an unmerged stack; do not conflate it with protected delivery. Live GitHub remains authority.
- Read README → project-track → current-state → model-direction → PROJECT_MAP; then DECISIONS D0-b and D4–D6, source audit `docs/integration/architecture-2026-09/`, takeover and research-leader handoffs, and source implementing each claim.
- The original `/Users/casey.allard/uor-r4` contains unique intentional dirty research. Read relevant material there; never edit or clean it.
- Rust offline training may use floats/matrix multiplication. Served numerical kernels target integer/add/subtract/shift/table work, low-bit learned weights, no Transformer or hidden provider. D5 requires sparse per-token parameter access in the terminal design; a dense ternary scan is not that end state. Geometry advantage is an open hypothesis.

## Current result to explain, not merely extend

The best current natural-text path is a small ordinary integer RNN, not a geometry-driven predictor. It scans 611,814 packed parameter slots per ordinary step (153,052 nonzero terms); compiled sparse-in-weight storage reduces this but is not sparse relative to model scale. Frozen-state readout adds only 0.022 bits beyond the order-2 count prior. A separately trained addressed KVAR store greatly exceeds chance; the matched Q8 residual does not establish a gain. The explicit-address loaded language bridge copies a changed source but leaves uncopied wording unchanged. The latest learned 72-cell sparse-read gate almost reproduces an ordinary recent cache, loses slightly to it on prose/Rust, and produces 0/8 exact eight-token continuations. Candidate addressing and integrated content-conditioned language remain unresolved. Do not propose tuning the exposed gate again.

## Review roles and independent deliverables

1. Mathematics: representation, Hamilton product versus Hamiltonian, exact-memory capacity, reversible/discrete dynamics, noncommutative transport, invariance and metric checks; attempt at least three materially different routes and identify decisive obstructions and a preferred construction.
2. Computer science/frontier research: attention semantics, information flow, credit assignment, candidate admission, sparsity and output distribution; compare current primary research and actual implementations with the proposed design.
3. Engineering/development architect: whole-repository mechanism census and historical integration, serving and training call paths, artifact contracts, hardware costs, coherent model spine, alternatives and migration.
4. Principal investigator: independently recover history/source/evidence, reconcile all three reviews, challenge assumptions, and publish one executable programme.

Every reviewer must consider the whole goal and latest result, inspect live source, consult relevant internal knowledge and primary external sources, mark coverage and uncertainty, and supply concrete source paths/lines and URLs. Mechanism claims need actual source; bibliographic novelty must not be asserted from a search alone. A later cross-review will reconcile disagreement before final synthesis.

Owned output files are `principal-attention-mathematics-2026-09-24.md`, `principal-attention-computer-science-2026-09-24.md`, and `principal-attention-engineering-2026-09-24.md` under this directory. Reviewers may write only their own report; no model/source changes, model execution, cargo, installs, indexing, external compute or automatic GitHub messages. Root owns the packet, synthesis, canonical pointers and delivery.

## Prospective resources

This review reuses the full clean worktree and source/artifacts; no training/build/corpus run or new index. Local source/document/symbolic work: at most 50 minutes charged inclusive of preparation, parallel review and validation; four agents maximum, one small local process at a time per reviewer, expected incremental RAM below 2 GiB, new retained/temporary storage below 25 MiB. Start receipt: cumulative 462,839,320 ms, limit 466,200,000 ms, remaining 3,360,680 ms; projection 3,000,000 ms with 360,680 ms remaining. Start physical free: 26,463,140 KiB; preserve the 25,904,021,504-byte reserve plus 128 MiB stop margin. No paid external compute, destructive cleanup or model promotion. Charge elapsed review once at completion; record any necessary extension prospectively under the standing authorization.

## Required final synthesis

Separate known results, source facts, derived necessities, and proposed mechanisms. Explain exact storage versus semantic compression; how query/key/value and writes are learned; what Hamiltonian means operationally; how geometric transport changes selection; full token-to-output computation and parameter access; the compatible ordinary comparator; one integrated language/memory/code milestone; later scaling/energy milestones; and explicit stop/redirect rules that prevent endless component-test loops. Record decision status, remaining owner decisions if any, and protected versus unmerged evidence.

## Investigation closeout

The [principal synthesis](principal-attention-plan-2026-09-24.md) and three independent reports complete the review. The final cross-review corrected raw-event versus semantic-version ownership, the conditions for relative-frame invariance, exact replay traces and migration, Q8 versus full 2I actions, hidden floating table initialization, bounded posting/prefix enumeration, Copy/Generate likelihood ambiguity, causal encoder inputs, and full optimizer/checkpoint sizing. The recommended next milestone is one jointly trained geometric/ordinary attention-language artifact. It is a proposal under existing owner decisions, with no implementation or capability promotion in this change.

Executed document checks: claim-wording gate; Git whitespace/diff check; existence of all 76 local links in the five new Markdown reports; generated Mermaid static lint on canonical JSON and the embedded Markdown diagram. The authored JSON was checked against the relevant schema fields and by the skill's structured validator; a full third-party JSON Schema validator was unavailable in both installed Python runtimes. Mermaid checks are static, not renderer acceptance. No Cargo build, training, model replay or performance measurement was run because this change only adds research/planning documents and navigation pointers. Current and dormant implementation facts were established by source inspection and retained receipts; historical findings were not relabeled as new experiments.

The original dirty checkout, all unique artifacts, historical negatives and worktrees remain preserved. Delivery uses a separate protected pull request on the existing unmerged stack; merging remains an owner action. The cumulative accounting closeout is recorded with the final delivery receipt.

### Final delivery and cumulative accounting

Protected PR [#1385](https://github.com/UOR-Foundation/uor-r4/pull/1385) is open on #1384; no merge was performed. The [closeout receipt](../evidence/principal-attention-review-2026-09-24.json) records a conservative inclusive **40-minute** charge within the 50-minute projection. The recovery-to-handoff accounting window is approximately 16:15–16:55 UTC, including the initial unclocked recovery allowance and final delivery; it is not a measured model-runtime claim. Cumulative charge advances from **462,839,320 to 465,239,320 ms** under the unchanged **466,200,000 ms** limit, leaving **960,680 ms**. Parallel reviewer wall time is charged once. Physical free space at closeout is **26,321,453,056 bytes**, leaving **283,213,824 bytes** above the protected reserve and 128 MiB margin. New review-document content is **158,156 bytes** before this small receipt/closeout addition. No allowance increase, cleanup or training was needed.
