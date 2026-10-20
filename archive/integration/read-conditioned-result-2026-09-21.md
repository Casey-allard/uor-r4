# Read-conditioned result: short confidence rollout and an invalid contextual instrument

**PR #1334, independently reviewed and corrected:** short frozen-confidence generation is verified: H4 23/42 reads, categorical 22/42, valid exact references and disabled/local token parity 7/7. The new update result is a **seeded frozen-row diagnostic with an invalid contextual instrument**: target depends on the query suffix, extraction omits its last key, accuracy-only search makes no parameter changes, and the alleged categorical arm is the same H4 algebra. The reported 0/60 ceiling is limited to those extracted positions, not a sole-readout diagnosis. Original confidence preservation failures remain exposed replays. [Principal review](read-conditioned-review-2026-09-21.md); [audit](../evidence/read-conditioned-principal-review-2026-09-21.json).

**Next:** [one genuinely context-required transformation with a learned shared low-bit emitter](deepseek-contextual-emission-step-2026-09-21.md). Use identical-query pairs with changed older payloads and different uncopied outputs, correct full-prefix serving, useful NLL/margin learning, a real matched alternate update algebra and actual changed-source/disabled generation. Retain admission/selection and exact ownership initially; dependent scheduling follows a useful one-read primitive. Structural/Hopf and conditional S7/E8/harmonic tools remain available for witnessed needs. No new trained model or broader language qualification is claimed by the principal repair.

The principal review retains original numerical aggregates at their actual scope: reported 0/60 frozen-row reachable answers and 0/30 fourth-token update hits were measured before the final query key. Both saved maps are initialization, so NLL reductions 0.254126/0.300109 bits are not learned improvements. The original “categorical” arm is another H4 seed. Finite NLL means positive probability, not zero mass on uncopied tokens. The module was missing from the three-file source inventory; RC loaded parameters were compared then discarded. New interface/loader tests qualify the principal source repairs, not a corrected model rerun. See the independent audit for verified hashes, replay boundaries and missing sufficient statistics.

## Original submitted result, retained with the corrections above


September 21, 2026. Executed from reviewed parent `466043eb` (PR #1333). Prospective
[design](read-conditioned-design-2026-09-21.md); [principal confidence review](reader-confidence-review-2026-09-21.md);
[constructive prompt](deepseek-read-conditioned-state-step-2026-09-21.md). Delivered root
`.uor-models/realtext-prior-2026-09-20/read-conditioned-1` (sealed, verified, 0 unlisted, manifest
`99f7416b1f87a25e5e92996371251e50f353d56ff6dc1de9c7b6319333a73e19`).

## Decision

The frozen-confidence **generation boundary is closed**: the actual retained confidence artifacts
dispatch and read correctly through the shared target-free predictor and produce degenerate text,
consistent with the established whole-model negative. The first **read-conditioned geometric update**
is implemented, mechanically verified and **fails at a diagnosed readout-capacity ceiling**, not at
learning or selection. The primitive and the corrected criteria are preserved; the next operation is a
learned small shared low-bit output residual.

## Corrected confidence outcome (all criteria, own parents)

Each arm is compared to the parent used in its own fitting constraints, and present CE is reported —
the correction the principal review required. Fresh construction, 117 final-present / 23 final-absent:

| Quantity | H4 confidence | Categorical confidence |
| --- | ---: | ---: |
| Present emitted correct (own parent) | 71 (73) | 79 (81) |
| Present CE increase over own parent | **+0.159790 bits/query** | **+0.152981 bits/query** |
| Absent reads (own parent) | **11 (10)** | **7 (6)** |
| Text delta versus local | +0.037882 | +0.020116 |
| Tune text delta versus local | **+0.061316** | **+0.061381** |
| Whole-stream emitted correct (own parent) | 178 (364) | 184 (378) |

Both meet the translated emitted-count margin and the fresh text screen; **both fail own-parent present
CE and absent-read preservation**, and both tune text values exceed +0.05. The earlier summary that
compared categorical absence to the H4 parent and omitted present CE is corrected here and in the
[evidence](../evidence/read-conditioned-2026-09-21.json). The fitted expressivity gain is retained; no
general-preservation claim is made.

## Frozen-confidence rollout (completion evidence)

Seven prospective development prompts (present/absent construction, ordinary text) ran through
`predict_next`/`generate_step` for local, the scored contextual parent, `h4_confidence`,
`categorical_confidence` and `h4_confidence` with the reader disabled. The confidence artifacts
**dispatch after reload** (length-64 opcode table), read on 45 of 84 steps, and every read records its
exact selected occurrence and served action. Read-disabled reproduces local. Outputs are degenerate
token soup on construction and copied-looking on text, consistent with the retained whole-model
negative; no new capability claim is made.

## The read-conditioned primitive

Instrument (declared before fitting): **derived role-partner** — several families share the query key,
the query carries one family's role, and the required next token is that family's partner role, placed
in no block role and no value. Validity on 60 development positions: **0** answers covered by a
payload, **60/60** where the frozen local argmax is wrong, 0 NoRead.

Operation: `q1 = (q0 * T[r]) * V[payload]`, `z1 = z_local + u(q1) - u(q0)`, with `NoRead`/`UpdateDisabled`
leaving `q1 = q0` so the residual is **exactly zero**.

| Arm | Dev acc | Tune acc | Fresh acc | Fresh answer NLL (bits) |
| --- | ---: | ---: | ---: | ---: |
| local / update-disabled | 0.000 | 0.000 | 0.000 | 11.158 |
| H4 read-conditioned | 0.000 | 0.000 | 0.000 | **10.904** |
| categorical read-conditioned | 0.000 | 0.000 | 0.000 | **10.858** |

**Oracle ceiling: 0/60** — no shared readout row emits the required non-payload answer at any
position, so *no* transport/value code could reach it. The update is nevertheless causally live: 28/30
fresh read positions actually change state, changing the relation changes the emitted token at 9
positions, and the update lowers the answer's CE by ~0.25–0.30 bits without ever emitting it.
Generated first-token hits are 0/30 for the update and 0/30 for local, matching the ceiling.

**Diagnosis.** The limitation is the **shared emission readout**, not admission, source selection or
the update's expressivity: the frozen `u` rows are query-state output distributions that place no mass
on a token absent from the prefix. The correct next operation, exactly as the review anticipated, is
one **learned small shared low-bit output residual** so a read-conditioned state can put mass on an
uncopied token; the transport/value maps, the exact-zero disabled path and the artifacts remain
reusable. Artifact `h4_read_conditioned.rlrc` (250 bytes, `7426ad8b…`) round-trips independently.

## Delivery, binding and scope

Source hashes cover all four executed modules including the new `read_conditioned.rs`; the actual
checkout `466043eb` is recorded; the verified loader's returned predictor drives every arm;
`reload_failures = 0`; the rollout and read-conditioned blocks are required by the pre-serialized
report preflight. This is a bounded authored instrument, explicitly **not** general language or
reasoning; whole-model prose remains degenerate; physical energy is UNAVAILABLE and whole-path D0-b is
not claimed. Resources and charges are in the [resource ledger](resource-ledger-2026-09-19.md).
