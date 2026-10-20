# Derived-state decoding: retained lexical evidence and corrected composition interpretation

**Principal correction:** the [review](derived-state-decoder-review-2026-09-21.md) and [independent audit](../evidence/derived-state-decoder-principal-review-2026-09-21.json) supersede the original diagnosis. The 104/120 winner is a reported decoder-only count on new seeds, without a saved winning artifact; 33/120 is a different population. Composition probe operations were missing from the trained domain. Interventions, identity and local parity were overstated; four completed roots verify, while root 3 is partial. Original numerical outcomes remain below; use the separate corrected replay for repaired-source behavior.


**Corrected exposed replay:** the exported, independently loaded value-only winner retains 161/180 development, 108/120 tune and 104/120 final emitted answers. Composition remains negative: H4 92/192 development and 3/64 held-out positions; C120 106/192 and 4/64. H4 lacks a grounded decoder outcome at 60/64 held-out positions despite 64 actual reads. Three association rollouts begin correctly but lose meaningful continuation. The fixed-selected-operand operation pair works; the end-to-end operation change loses its read. This directs the successor toward learned transition/outcome consistency, independent source ownership and persistent response phase. These are exposed regression results, not new final qualification.

September 21, 2026. Executes the [derived-state decoder brief](deepseek-derived-state-decoder-step-2026-09-21.md) and
the [principal review](geometric-computation-review-2026-09-21.md) on merged `fdc1607f` (PR #1337,
verified equal to the merged head). Isolation worktree `.worktrees/derived-state-decoder`; owner
checkout untouched. New mode `--mode=derived-state-decoder`. [Evidence](../evidence/derived-state-decoder-2026-09-21.json).

## Mechanism

The residual readout scores `F(q1) - F(q0)`. For any finite group and any `F`, right multiplication
permutes the group, so `sum_g (F(g*a) - F(g)) = 0`: that difference cannot supply a frame-independent
boost across a whole action orbit. This run decodes the **relative result** instead:

```text
q1 = (q0 * T_bind[r]) * U_op[observed query role] * V[value]     exact finite products
s  = inverse(q0) * q1 = T_bind[r] * U_op[query] * V[value]       frame cancels
emit = decoder[s]                                                learned token shortlist
```

`U_op` is keyed by the **observed query role token** of the actual causal prefix, never by the
fixture's hidden operation index. The decoder is a bounded learned per-state token shortlist
(<= 16 grounded states, k = 4). **A computed identity is an ordinary emittable state and is distinct
from `NoRead`**, which is the absence of a grounded read or of a learned state and preserves the
frozen local prior. This is a new, separately versioned artifact (`RLDS`/`RLRD`); the old
zero-residual interface and its artifacts are untouched.

## The decoder is useful on the association panel

New populations (seeds `0xC0F20011/12/21`); the preserved PR #1337 regression is unchanged and is
not re-fitted.

| Arm | dev /180 | tune /120 | final /120 |
| --- | ---: | ---: | ---: |
| frozen local prior | 0 | 0 | 0 |
| H4 residual readout (retained regression) | 61 | 41 | 33 |
| derived-state decoder, all factors | **166** | 80 | 91 |
| derived-state decoder, selected-value factor only | 161 | 108 | **104** |
| development-fitted selected-value dictionary | 161 | 108 | 104 |

The reported value-only decoder counts match its same-run selected-value dictionary with eight grounded states. This is useful lexical association evidence. The prior residual population differs, so 33→104 is not a paired improvement. The value-only winner was not exported or independently served; only the full-factor artifact was retained. Full-factor development 166/final 91 suggests overfitting or unnecessary distinctions at this scope, not a universal finding about binding geometry.

## Composition against familiar primitives

Fixture v2 (modulus 10 with a witnessed order-10 element, 8 observed-query operations, 192 development
and 64 held-out positions (48/16 unique cells)). A **development-internal probe** (operations 6-7, disjoint from the fit
operations, with every probe output class covered by the fit operations) supplies the generalization
pressure that a raw fit-objective lacks. Selection uses development only.

| Arm | dev /192 | held-out /64 |
| --- | ---: | ---: |
| local / NoRead | 0 | 0 |
| H4 derived state (primary, probe-first) | 103 | 8 |
| matched additive C120 | 132 | 20 |
| payload-only table | 38 | 1 |
| operation x payload table (unseen-cell fallback) | 172 | 8 |
| development constant | 24 | 8 |

The reported probe score rises from 0/48 to 27/48, but probe labels directly guide every coordinate choice. These are supervised development counts. The model operation vocabulary contains six fit operations only; both probe operations fall back to identity. Primary H4 totals 103/192 and 8/64 independently recount; C120 totals remain reported because its rows are absent. The 192/64 positions are 48/16 distinct cells with four contexts each. The declared negative remains.

The shared factor family constrains missing products, and the observation graph is connected. Unknown/noninjective decoder labels may leave ambiguity, but independent per-cell freedom is not established. The selected-value and two-input controls also fit on all development positions, while the geometric decoder calibrates on a source-correct inner subset; they do not have identical fitting scopes.

## Original interventions and generated output (loading claim corrected)

| Original control | Correct scope |
| --- | --- |
| Relevant payload changed | Token 4088→4087 changes, but expected 4095→4094: sensitivity, not correct causal computation |
| Operation changed | Source-role and query-role positions both change; fixed-evidence interpretation withdrawn |
| Distractor changed | One wrong output remains unchanged; invariance only |
| Required source removed | Read absence observed; the original equals_local boolean does not independently compare local output |
| Read/update disabled | Single-prefix structural booleans, not complete local-logit parity |
| Claimed composed identity | Target class 0 selected, but learned state 25 differs from actual H4 identity 1 |

Three three-token completions repeat the same held-out cell (op 0,value 1), emit [4087,29,86], and miss first answer 4094. Loaded artifact equality was checked, but the loaded objects were discarded and the original in-memory models served evaluation/interventions. There is no learned response-phase/EOS result.

## Corrected next direction

Retain the finite lexical decoder and separate binding, operation and output roles. Repair supervised domain initialization and actual winner export/load/use; distinguish absent reads, unknown operands and missing decoder states from a valid identity. Then learn shared primitive transitions and owned result/phase continuation through complete emitted answers or dependent reads. Do not inject a hidden numeric operation index through a one-parameter power rule. The [next research brief](deepseek-shared-transition-continuation-step-2026-09-21.md) provides the coherent milestone and contributor autonomy.

## Limitations

Exposed association regression seeds; a constructed, algebra-realisable composition rule; 3-token
generation only; energy `UNAVAILABLE`; whole-path D0-b not claimed. Completed roots `derived-state-decoder-{1,2,4,5}` have six valid listed files each; `-3` is partial/unsealed and retains a 527-byte association artifact. Primary `-5` and diagnostic `-4` remain unchanged. The two new source hashes recorded by `-5` do not match submitted 16b7148e; formatting drift is a possibility, not verified provenance.
