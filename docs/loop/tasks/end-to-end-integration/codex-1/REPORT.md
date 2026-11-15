# REPORT: end-to-end-integration, codex-1

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic integration-architecture` was refused because the
  item file does not exist on `origin/main` until this unit adds it, and the existing item scopes
  still overlap `Cargo.lock`. This unit adds no dependency and does not touch `Cargo.lock`; it
  continues under the owner's D85 direction.
- **The selection protocol is a rule, not a result.** "Global optimality" needs one frozen corpus
  and one metric vector; neither exists yet, so ADR-0007 fixes the rule and leaves the measurement
  to I1–I6. No mechanism is promoted by this unit.
- **The inventory that motivated the map read the preserved `codex/one-model-copy-priority`
  checkout**, which is behind `main` and lacks D85/D86, `/12`, `/13` and the Q31 lane state. Its
  file:line citations are stale; the map's statuses come from the mechanism board and `main`, and
  the catalogue is used as evidence of shape, not of exact lines.
- **Some mechanisms do not map to a stage without invention.** `Geometric stack` and `Native
  geometric engine` are whole models, not stages; `E8 address memory`, the pair experts and the
  pair rows have no board status yet; the zeta modules are address/phase machinery only. The ADR
  says so rather than forcing them into a column.
- **The `ReadOut` contract may be too small for the relation read**, whose support is a typed,
  multi-sweep set rather than positions with weights. This is recorded as a risk; I1 must either
  fit it or split the contract.
- **CIRISKernel's own claims do not transfer as stated.** It has no rolling counter, no attention
  and no zeta function; its checkpoint is a storage fold that proves resumability and
  authorisation stability, not byte safety, and it explicitly concedes unbounded retention and
  reader-relative forks. The ADR imports only the settlement discipline for `Context`.
- **e-means' scaling laws are not adopted.** They were fitted at `10⁵ ≤ n ≤ 10⁸`, `256 ≤ k ≤ 65536`,
  and the paper says they are not optimal for small or low-dimensional data. Only the recipe is
  attached, at a bit-identical-export gate.
- No code, schema, dependency or GPU run is changed; no claim about model quality or useful
  conversation is made.

## What changed

- `docs/reference/adr/0007-stage-interfaces-for-end-to-end-integration.md`: seven stage interfaces
  (`Tokenizer`, `Embedding`/`Codebook`, `SequenceMixer`, `Position`, `Read`, `Context`,
  `Head`/`Loss`), the fixed `ReadOut` shape, four integration rules, the selection protocol, the
  28-mechanism map with an action each, the four attachments (e-means → `Codebook`, CIRISKernel →
  `Context`, RoPE → `Position`, JEPA → `Loss`, graph-compiler induction → `Codebook`), and the I1–I6
  unit sequence.
- `docs/roadmap/items/end-to-end-integration.md`: the item, its done-when, and its out-of-scope.

Size: about 230 lines of docs.

## Measurements

None: this is an architecture decision. The gates it sets are measurable and named (bit-identical
parity, the e-means one-third-iteration MSE gate, full-history equivalence for `Context`, frozen
panels for the lossy arms).

## Checks run at the head

- `uor-harness loop report --check` — at the head.
- The docs link check (CI `docs archive and links`) — at the head.
- `git diff --check` — at the head.
- The exact-head review — at the head.

## Skills used

- `solution-integrity`: checked that the interfaces and gates answer the actual gap (no shared
  seam, no metric) rather than adding another mechanism.
- `scope-guard`: docs only; no mechanism status, schema or code touched.
- `trace-back`: the inventory's stale checkout was caught and the map re-grounded on the board and
  `main`.

## Still open

- **I1** the `Read` interfaces and parity harness (keystone), including the relation-read contract.
- **I1b** the exact code→positions index for single-code heads (the paused read work).
- **I2** the `Context` ring and settled stub, importing the CIRISKernel discipline and replacing the
  four different overflow behaviors.
- **I3** `Codebook::fit` with an e-means arm at one site; **I4** the `Position` arm; **I5** the
  `Loss` arm; **I6** the composition and cost ledger.
- The frozen corpus and metric vector must be chosen before I1 lands.

## Plain summary

The 28 mechanisms now have one architecture: seven stage seams, one mechanism per stage per
artifact, a parity-first selection rule, and an ordered set of units. e-means belongs to codebook
fitting; CIRISKernel's settlement discipline belongs to context compaction; RoPE is a position arm,
JEPA a loss arm, and the graph compiler's induction a codebook builder. Nothing is promoted; the
next move is the read interface and its parity harness.
