# Pre-Registration Sheet (DRAFT): M3 Memory-Turn Discriminator and Margin Gate (`#2855`)

**Status:** `DRAFT` (`TEST FITNESS: PENDING`; bar `PROPOSED, NOT REGISTERED`)
**Item:** [`m3-memory-turn-discriminator-gate`](../../roadmap/items/m3-memory-turn-discriminator-gate.md) (`#2855`, Milestone M3 [`#2031`](https://github.com/UOR-Foundation/uor-r4/issues/2031))
**Mechanism:** [`exact-memory`](../../mechanisms/exact-memory.md) — turn-compiler decision boundary (`StatementMargin` + `MemoryTurnGate` in [`crates/uor-r4-training/src/relation_compiler.rs`](../../../crates/uor-r4-training/src/relation_compiler.rs))
**Parent evidence:** [`#1718`](https://github.com/UOR-Foundation/uor-r4/issues/1718) (`compiler-save-op-v25-rawtable` + `op-model-v25`, `op_policy = unless_query`) and [`m3-nonrelation-prose-panel`](../m3-nonrelation-prose-panel/README.md) (`#2854`)

---

## 1. Question and Mechanism Under Test

Can an integer / additive memory-turn discriminator and score-gap margin gate (`StatementMargin` + `MemoryTurnGate` in [`crates/uor-r4-training/src/relation_compiler.rs`](../../../crates/uor-r4-training/src/relation_compiler.rs), with zero squashing functions and zero dense matrix multiplications under CHARTER §3.1–§3.3) eliminate false `CompiledAction::Assert` and `CompiledAction::Correct` writes on the frozen 50-row non-relation prose panel ([`data/panels/nonrelation-prose-v1.txt`](../../../data/panels/nonrelation-prose-v1.txt)) while preserving `world=v2` session accuracy and leaving `QueryCurrent`, `Query`, and `Unresolved` handling intact?

### Exact Decision Boundary Identified in `relation_compiler.rs`

1. **Policy precedence boundary ([`resolve_op_policy`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1529-L1606)):** Under `OpPolicy::UnlessQuery` (the active policy in `#1718`), whenever `table` returns `CompiledAction::Unresolved`, `resolve_op_policy` unconditionally accepts any `CompiledAction::Assert` or `CompiledAction::Correct` emitted by `op`, provided the op's generated token matches a word start in `source` ([`parse_op`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1465-L1516)). On non-relation turns (such as `#1718`'s `"The weather has been unusually warm this week."`, `"I enjoy walking in the park after dinner."`, and `"Actually, let us set that topic aside for another conversation."`), `op-model-v25` emits a closed relation and a word from the turn, and `UnlessQuery` commits a false memory write.
2. **Table / span boundary (`GatedTurnCompiler<C>` and `SavedCompiler`):** On the table/span path, argmax classification and span decoding do not require a minimum additive score gap $\Delta_{\text{rel}} = s_{r^\star} - \max_{c \ne r^\star} s_c$, a minimum write-act gap $\Delta_{\text{act}} = s_{a^\star} - \max(s_{\text{query}}, s_{\text{none}})$, or a speaker-grounded relation frame outside the value span. Because `SavedCompiler`'s legacy Stage-A `Softmax` and `SparseSoftmax` heads use floating-point exponentials and dense dot products, they remain compiled out behind `#[cfg(any())]` as on `main` (CHARTER §3.2–§3.4), and `SavedCompiler` passes `None` for `StatementMargin` until a lawful integer geometric head replaces its compiled-out legacy heads. [`GatedTurnCompiler<C>`](../../../crates/uor-r4-training/src/relation_compiler.rs) wraps any compiled `TurnCompiler` with a configurable `MemoryTurnGate` and optional integer `StatementMargin` callback (`with_margin_fn`).

### Gate Law (`StatementMargin` + `MemoryTurnGate`)

For any candidate write (`CompiledAction::Assert { relation, span }` or `CompiledAction::Correct { relation, span }`):
- **Non-writes untouched:** `CompiledAction::QueryCurrent`, `CompiledAction::Query`, and `CompiledAction::Unresolved` bypass the gate unchanged.
- **Never write-to-read:** Any candidate write rejected by the gate returns `CompiledAction::Unresolved { reason }` and is never converted into `QueryCurrent` or `Query`.
- **Additive score-gap check (`MemoryTurnGate::check_margin`):** When linear integer score gaps are supplied (`StatementMargin`), require:
  $$\Delta_{\text{rel}} \ge \tau_{\text{rel}} = 250,\quad \Delta_{\text{act}} \ge \tau_{\text{act}} = 250,\quad \Delta_{\text{span}} \ge \tau_{\text{span}} = 500$$
  in integer milli-units (`saturating_sub` / `saturating_add` only).
- **Structural memory-turn discriminator (`MemoryTurnGate::check_turn_structure`):** Require that the turn is non-interrogative (does not end with `'?'` or open with an interrogative/request token), carries a valid non-empty word span, is not negated (`"not"`, `"never"`, `"isn't"`, `"don't"`, etc.), is not conditional/hypothetical (`"if"`, `"whether"`, `"unless"`, `"suppose"`, `"were"`), does not report third-person speech (`"said"`, `"told"`, `"claimed"`, etc.), has no third-person subject/possessive pronoun (`"his"`, `"her"`, `"their"`, `"he"`, `"she"`, `"they"`), has no `<noun>'s` possessor outside the closed-relation slot possessives or kinship/person noun modified by `"my"`/`"our"`, and contains a speaker-grounded lexical frame outside `span` matching the target closed relation (`1..=10`; bare `"I'm {v}"` / `"I am {v}"` for `user_name` requires a capitalized proper-name token and `code_word` requires present-tense `"code word is"` / `"code word be"` grounding) or a first-person copular open-relation phrase (`relation >= DERIVED_RELATION_ID_BASE`).

---

## Bar

**Status: PROPOSED, NOT REGISTERED** (requires owner/PM registration on `#2855` before any GPU run).

1. **Primary non-relation prose bar (`B_prose`):** `0 / 50` false memory writes (`false_assert_count == 0`, `false_overwrite_count == 0`, `false_correct_action_count == 0`, `is_all_clean() == true`) on the frozen 50-row non-relation prose panel [`data/panels/nonrelation-prose-v1.txt`](../../../data/panels/nonrelation-prose-v1.txt) (`5,111 B`, `sha256: 7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`) under `grade_prose_panel`.
2. **Primary `world=v2` regression bar (`B_v2`):** On the 500-dialogue `world=v2` session benchmark (2,535 turns, seeds `21, 22, 23`), gated compiler (`GatedTurnCompiler` / `with_memory_turn_gate(MemoryTurnGate::default())`) achieves `0` regression on complete-dialogue reply accuracy relative to the unguarded `#1718` baseline (`with_memory_turn_gate(MemoryTurnGate::disabled())`, `99.60%`, `498 / 500` dialogues; `2,533 / 2,535` turns) and `0 / 79` generator template rejections across all 10 `world=v1` and `world=v2` relations across `Split::Train` and `Split::Development`.
3. **Write-to-read invariant (`B_invariant`):** `0` instances where a rejected `Assert` or `Correct` is converted into `QueryCurrent` or `Query`.

---

## Base artifact

- **Source commit:** `antigravity/m3-memory-turn-discriminator-gate` (PR `#3079`, [`crates/uor-r4-training/src/relation_compiler.rs`](../../../crates/uor-r4-training/src/relation_compiler.rs)).
- **Compiler & op-model recipe (`#1718`):** `compiler-save-op-v25-rawtable` + `op-model-v25` (`op_policy = unless_query`), bound to the 6-layer `w=512, h=8, m=1536, ctx=256` trunk and the 4,096-token BPE tokenizer (`sha256: 8fd8bc5b7ef075c36ef1cc095655b4e437ca2e9a45f466eb62f4b869304fa2e2`).
- **Frozen panel:** [`data/panels/nonrelation-prose-v1.txt`](../../../data/panels/nonrelation-prose-v1.txt) (`5,111 B`, `sha256: 7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`), verified by [`crates/uor-r4-training/src/nonrelation_prose_panel.rs`](../../../crates/uor-r4-training/src/nonrelation_prose_panel.rs).

---

## Data split

- **Training / fitting split:** Only `world=v2` synthetic relation dialogues (`milestone_world_v2`, seeds `1..=20`) are used to fit the compiler and op-model. Zero rows, n-grams, or templates from [`data/panels/nonrelation-prose-v1.txt`](../../../data/panels/nonrelation-prose-v1.txt) ever enter training or threshold tuning (CHARTER §4.3).
- **Held-out evaluation splits:**
  1. Frozen 50-row non-relation prose panel [`data/panels/nonrelation-prose-v1.txt`](../../../data/panels/nonrelation-prose-v1.txt) (`50` turns across 6 strata: `small_talk`, `opinion_preference`, `factual_world_statement`, `nonrelation_question`, `anaphoric_discourse`, `statement_near_miss`).
  2. Held-out `world=v2` session benchmark (`500` dialogues / `2,535` turns per seed across seeds `21, 22, 23`).

---

## Steps

1. **Finite host pre-flight (`0` GPU hours):**
   ```sh
   cargo test -p uor-r4-training --lib relation_compiler
   cargo test -p uor-r4-training --lib nonrelation_prose_panel
   ```
   Confirm `relation_compiler` unit tests (including `memory_turn_discriminator_and_margin_gate_controls`, `memory_turn_gate_preserves_all_world_v2_relation_statement_templates`, `gated_turn_compiler_wires_configurable_gate_and_integer_margin`, and `memory_turn_gate_achieves_zero_false_writes_on_frozen_nonrelation_prose_panel`) and `5 / 5` `nonrelation_prose_panel` tests pass on host.
2. **GPU panel + `world=v2` session evaluation (after non-author `FIT` and owner release):**
   - Run the two-arm comparison (Arm A: `compiler.with_memory_turn_gate(MemoryTurnGate::disabled())`; Arm B: `compiler.with_memory_turn_gate(MemoryTurnGate::default())`) on [`data/panels/nonrelation-prose-v1.txt`](../../../data/panels/nonrelation-prose-v1.txt) and on `world=v2` held-out dialogues (seeds `21, 22, 23`, 500 dialogues/seed).
   - Emit sealed JSON summary with per-stratum false-write counts, unresolved counts, and `world=v2` dialogue/turn accuracy.

---

## Stop

- **Immediate stop on pre-flight failure:** Any failure of the finite host tests in Step 1 halts execution before pod allocation.
- **Stop after two-arm evaluation:** Single forward evaluation pass across the 50-row frozen prose panel and 3 seeds of 500 `world=v2` dialogues (`7,655` total turns per arm); no iterative re-running or post-hoc threshold tuning on the held-out panel.

---

## Cost

- **Compute target:** 1x RTX 4090 pod via `scripts/pod/uor-pod`.
- **Wall-clock duration:** `< 10` minutes (`~0.15` GPU-hours).
- **Estimated spend:** `<= $0.15` at `~$0.60–$0.89/h` (well within the `$8/h` lab-wide cap).

---

## Minimum test power

- **Non-relation prose panel:** `50` frozen turns across 6 strata (`10` `small_talk`, `8` `opinion_preference`, `8` `factual_world_statement`, `8` `nonrelation_question`, `8` `anaphoric_discourse`, `8` `statement_near_miss`), plus `11 x 2 x 50 = 1,100` adversarial write-injection probes in the deterministic host sweep. Because the target bar on non-relation prose is an exact zero-false-write invariant (`0 / 50`), a single false write falsifies `B_prose`.
- **`world=v2` session regression check:** `3` independent held-out seeds (`21, 22, 23`) $\times$ `500` dialogues/seed = `1,500` dialogues (`7,605` turns, exceeding the CHARTER §4.3 `>= 200` row and `>= 3` seed floor), plus deterministic verification across all `79` `world=v1` and `world=v2` statement/update templates across `Split::Train` and `Split::Development`.

---

## Verdict

TEST FITNESS: PENDING - M3 #2031 m3-memory-turn-discriminator-gate
