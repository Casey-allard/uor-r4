# Lab Note: M3 Memory-Turn Discriminator and Margin Gate (`#2855`)

> **25 Oct 2026 (antigravity-63, #3207): core-cofactor marker gate on the lawful sieve compiler, measured on the GPU (`vast-57652110`, 1x RTX 5090). The registered bar is met (`registered_bar_met = true`).**
> - sheet: [`preregistration-core-cofactor-gate-2026-10-25.md`](preregistration-core-cofactor-gate-2026-10-25.md) (FIT `#2855` comment `6396634979`, D67 pre-run GO `#2855` comment `6396643627`);
> - code: `CoreCofactorTable::derive` in `crates/uor-r4-training/src/sieve_marker_gate.rs`, order-independent exact-explanation dominance (`sieve_marked_reading`) in `src/sieve_frame.cu`, and `src/bin/prose-panel-gate.rs` (built at commit `67852848263f7df71362efaa411b2e7f50ebc6b7`).
>
> **Negatives first.**
> - **`212/220` of `R2` (`Split::Development` phrasings) remain `Unresolved`** (`"no frame survives the markers, sieve and alignment"`), with `0` wrong writes. Their templates either introduce surface words outside the 79-word `Split::Train` prime lexicon (`"you"`, `"can"`, `"correction"`, `"reside"`, `"raised"`, `"earn"`, `"living"`, `"buddy"`, `"goes"`, `"by"`, `"secret"`, `"remind"`, `"tell"`, `"most"`, `"best"`, `"brings"`, `"luck"`, `"these"`, `"days"`, `"did"`, `"say"`, `"give"`, `"was"`, `"again"`, `"occupation"`, `"grow"`, `"up"`, which tokenize to `0u32` and cannot match outside `{v}`) or use syntactic orders not closed by training-only fourth-proportional core/shell completion (`"The name's {v}."`, `"{v} is my lucky number."`).
> - First-launch throughput decreased slightly from `claude-102`'s `740,484` tokens/s (`84` frames, `6` markers) to **`708,843` tokens/s** (`6,038` µs upload-to-download; **`4,677,595` tokens/s** at `915` µs on the warm second in-process launch, and **`671,161` tokens/s** at `6,377` µs / `922` µs second launch on the `cuda,sieve-cuda` build) because the kernel evaluates `1,179` frames (`84` base + `1,095` derived) and `12` markers per turn.
>
> **What held (registered bar met).**
> - **Lead metric (tokens/s):** **`708,843` tokens/s** (`732` turns, `4,280` words, `6,038` µs upload-to-download; second launch `915` µs) on `sieve-cuda`, and **`671,161` tokens/s** (`6,377` µs; second launch `922` µs) on `cuda,sieve-cuda`.
> - **Frozen non-relation prose panel (`P`, 50 turns, `7c29c665…`):** **`false_assert = 0/50`**, **`false_overwrite = 0/50`**, **`clean_unresolved = 50/50`** (`false_correct_action = 0`, `false_query = 0`, `0` surviving frames across all `50` panel turns).
> - **Training phrasings with development values (`R1`):** **`462/462` (`100.0% >= 95%`)** exact act, relation, and value span, recovering all `56` nested-span misses from `claude-102` (`406/462`).
> - **Unseen development phrasings (`R2`):** compiles `R2` **`8` of `220`** (`> 0/220` baseline; all `8` instances of the unseen development template `"My dog's new name is {v}."` compiled to the exact `Correct` action, relation `2` `pet_name`, and exact character span via fourth-proportional completion from `Split::Train`, while all remaining `212/220` abstained cleanly as `Unresolved` with `0` wrong writes).
> - **Instrument & determinism checks:**
>   - Finite CUDA controls: `12/12` (`prose-panel-sieve controls`, `failed = 0`, `4,830` µs, sha256 `4a6ec7acdd6d48525f8546843e7ed2d3f1dec50ba4693e83644ead4f317376c3`) and `16/16` (`prose-panel-gate controls`, `failed = 0`, `5,355` µs, sha256 `ebd630d05f206fdd996c3e8ff71040bafba50d239726f85477a142d10a87beff`).
>   - D29 PTX interlock: `5/5` tests pass on both `sieve-cuda` (`d29-sieve-cuda.txt` sha256 `fe8ad63e…`) and `cuda,sieve-cuda` (`d29-cuda.txt` sha256 `fe1b2863…`), with zero `ex2`, `lg2`, `tanh`, `mma`, or `wmma` instructions in `sieve_frame.ptx` (`72fa1e9dd70ba3ef9e0c617e48e58b0b2b11ae9fba3bf4644606fd57c896b56a`).
>   - Two-launch and two-build determinism: `deterministic_second_launch = true` on both builds, and `decisions.json` (`sieve-cuda`) and `cuda-decisions.json` (`cuda,sieve-cuda`) are byte-identical (`cebeadad6641f4adb52bcab6cef89947964a50747cf418269051f74a84dcf24b`).
> - **Pins & artifacts:** frame table `379c2ecc4f1d04b49168d76e43ac93f3d07730e1016e0a281608740efb46604f`, base marker table `cd79399490c0abcfc14253b5680a282fc840672c5932e925d62a3b95375a125d`, core-cofactor table `b5bd388410a82f14ca6a855f0e420b159f99ad1dd77467e46f33b390ead829d4`, compiler artifact `1c92c7429d784ab681dd23e851aef006ac0cfdeeae454bd20f20fff22d7bccb7`, PTX `72fa1e9dd70ba3ef9e0c617e48e58b0b2b11ae9fba3bf4644606fd57c896b56a`. Executed on one NVIDIA GeForce RTX 5090 (driver `580.95.05`, CUDA `12.8`, `rustc 1.97.1`), pod `vast-57652110`. All outputs (`record.json` sha256 `6b9b7dd4704e5596cb2597aeed16a9b11ed9baa2b76f7b5c39e7f985ddb69c22`, `cuda-record.json` sha256 `8267a12d697bf3dbc91963a1f7e8325cea880588041adf0a0ced1775e92559b4`, `SHA256SUMS`) are archived in `caseyallard/uor-r4-store:results/antigravity/prose-panel-gate-2026-10-25/`.

> **25 Oct 2026 (claude-102, #3201): a lawful marker gate on the sieve compiler, measured on the GPU. The result is a negative for this configuration.** The `relation_compiler.rs` work below gates a `SparseSoftmax` compiler that CHARTER §3.2 bans. It is kept as history. The run's pieces:
> - sheet: [`preregistration-marker-gate-2026-10-25.md`](preregistration-marker-gate-2026-10-25.md) (FIT `6395084106`, pre-run GO `6395098979`);
> - code: `crates/uor-r4-training/src/sieve_marker_gate.rs`, the kernel `sieve_frame_compile_marked` in `src/sieve_frame.cu`, and the binary `src/bin/prose-panel-gate.rs`.
>
> **Negatives first.**
> - **R1 fell to 406/462 (87.9%),** below the 95% floor. All 56 misses are "the surviving readings disagree", on 7 update templates whose frame carries a word the assert frame lacks: "Actually, … {v} now." and "Sorry, my lucky number is really {v}.". The marker reading (`actually`/`sorry` plus the assert frame) takes that word into the value, after it ("X now") or before it ("really X"), against the plain update frame's "X". The two readings name the same expert with different spans, so the gate refuses.
> - **R2 stayed at 0/220,** all `Unresolved`. The unseen phrasings differ from the training frames in their core wording, not in a leading discourse marker, so this marker set reached none of them.
>
> **What held.**
> - The panel stays clean: false_assert 0/50, false_overwrite 0/50, clean 50/50, with 0 surviving frames on any panel turn.
> - 740,484 tokens/s (732 turns, 4,280 words, 5,780 µs).
> - Finite CUDA controls: 12/12 for the sieve (lean build) and 14/14 for the gate (both builds).
> - D29 passed on the lean and `cuda` builds (5 tests each).
> - A second build gave byte-identical decisions (`66b01121…b936`).
> - Pins: frame table `379c2ecc…`, marker table `cd793994…`, PTX `75e837bc6ff7704602912f890d9b99811713598d9b77643253f3d0c12dc06710`.
> - One RTX 5090 (driver 580.178.04), pod `vast-57633156`, run ref `63cd966ad`. Outputs are in the HF store under `results/claude/prose-panel-gate-2026-10-25/` (`record.json` sha256 `771e15aa…930c`).
>
> **Reading (D22: this closes this configuration only).**
> - The span disagreement is specific: a correction frame's own extra word falls inside the marker reading's 1–4 word value, after it ("X now") or before it ("really X"). A next configuration could keep the marker rule but bar base-frame words from the value, or let readings that agree on the expert resolve to the shorter span.
> - R2 says the larger fact for this configuration: this marker set reached none of the unseen core wordings. This gate's real work is generalizing the core of the frame while keeping the panel clean.

- **Roadmap item:** [`m3-memory-turn-discriminator-gate`](../../roadmap/items/m3-memory-turn-discriminator-gate.md) (`#2855`)
- **Milestone:** M3 — Exact Memory & Grounded Sessions (`#2031`)
- **Mechanism:** [`exact-memory`](../../mechanisms/exact-memory.md)
- **Source:** [`crates/uor-r4-training/src/relation_compiler.rs`](../../../crates/uor-r4-training/src/relation_compiler.rs)
- **Draft pre-registration sheet:** [`preregistration-DRAFT-2026-10-23.md`](preregistration-DRAFT-2026-10-23.md) (`TEST FITNESS: PENDING`)

---

## 1. Exact Decision Boundary Where Non-Relation Prose Compiles to `Assert` or `Correct`

Inspection of [`crates/uor-r4-training/src/relation_compiler.rs`](../../../crates/uor-r4-training/src/relation_compiler.rs) and the `#1718` (`world=v2`) session evidence locates two coupled boundaries where non-relation turns become false memory writes:

1. **`OpPolicy::UnlessQuery` in [`resolve_op_policy`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1572-L1604):**
   In the `#1718` configuration (`compiler-save-op-v25-rawtable` + `op-model-v25`, `op_policy = unless_query`), when the table head returns `CompiledAction::Unresolved` on a non-relation turn, `resolve_op_policy` delegates to `op()`. Because `op-model-v25` was trained predominantly on closed-world relation turns (`Op: none` is a small minority of training rows), it frequently generates `Op: assert <closed_relation> <word>` or `Op: update <closed_relation> <word>` using a surface word from the prompt. [`parse_op`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1465-L1516) locates `<word>` at a word boundary in `source` and emits `CompiledAction::Assert` or `CompiledAction::Correct`, which `UnlessQuery` accepts unconditionally.
   - Example (`#1718` turn 1): `"The weather has been unusually warm this week."` $\rightarrow$ false `Assert`.
   - Example (`#1718` turn 2): `"I enjoy walking in the park after dinner."` $\rightarrow$ false `Assert`.
   - Example (`#1718` turn 3): `"Actually, let us set that topic aside for another conversation."` $\rightarrow$ leading `"Actually,"` triggers `update`, producing a false `Correct`.
2. **Table / span classification boundary (`GatedTurnCompiler<C>` and `SavedCompiler`):**
   On the table/span path, classification previously picked the argmax relation and act without checking whether the additive linear score gap $\Delta_{\text{rel}} = s_{r^\star} - \max_{c \ne r^\star} s_c$, the write-act gap $\Delta_{\text{act}} = s_{a^\star} - \max(s_{\text{query}}, s_{\text{none}})$, or the span boundary margin $\Delta_{\text{span}}$ clears a positive threshold, and without checking whether the turn's words outside `span` actually form a speaker-grounded statement for the target relation. Because `SavedCompiler`'s legacy Stage-A `Softmax` and `SparseSoftmax` heads use floating-point exponentials and dense dot products, they remain compiled out behind `#[cfg(any())]` as on `main` (per CHARTER §3.2–§3.4), and `SavedCompiler` passes `None` for the `StatementMargin` until a lawful integer geometric head replaces its compiled-out legacy heads. For compiled, integer-only turn compilers, [`GatedTurnCompiler<C>`](../../../crates/uor-r4-training/src/relation_compiler.rs) wraps any `TurnCompiler` with a configurable `MemoryTurnGate` (`new`, `with_memory_turn_gate`, `set_memory_turn_gate`, `memory_turn_gate`) and an optional integer `StatementMargin` callback (`with_margin_fn`) so the two-arm comparison (Arm A `MemoryTurnGate::disabled()` vs Arm B `MemoryTurnGate::default()`) runs without source edits.

---

## 2. Integer / Additive Gate Design (`StatementMargin` + `MemoryTurnGate` + `GatedTurnCompiler`)

Implemented in [`crates/uor-r4-training/src/relation_compiler.rs`](../../../crates/uor-r4-training/src/relation_compiler.rs) with zero squashing functions, zero dense matrix multiplications, and zero core floating-point operations (D23/D25/D32 ratchets unchanged; all legacy Stage-A `Softmax`/`SparseSoftmax`/`SavedCompiler` items and `milestone_world_v2` / `stack_tracking` Stage-A code remain compiled out behind `#[cfg(any())]` as on `main`):

- **[`StatementMargin`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1640-L1690):**
  Computes integer milli-unit score gaps from linear class scores via `saturating_sub`, `saturating_add`, and `Iterator::max`, wired into [`GatedTurnCompiler::with_margin_fn`](../../../crates/uor-r4-training/src/relation_compiler.rs) (`SavedCompiler`'s margin half remains `None` behind `#[cfg(any())]` until a lawful integer geometric head replaces its compiled-out legacy float heads):
  - `relation_gap_milli`: $\Delta_{\text{rel}} = s_{r^\star} - \max_{c \ne r^\star} s_c$
  - `act_write_gap_milli`: $\Delta_{\text{act}} = s_{a^\star} - \max(s_{\text{query}}, s_{\text{none}})$
  - `span_margin_milli`: $\Delta_{\text{span}} = \text{start\_margin} + \text{end\_margin}$
- **[`MemoryTurnGate`](../../../crates/uor-r4-training/src/relation_compiler.rs#L1715-L2150):**
  1. **Score-gap margin gate (`check_margin`):** Requires $\Delta_{\text{rel}} \ge 250$, $\Delta_{\text{act}} \ge 250$, and $\Delta_{\text{span}} \ge 500$ whenever a `StatementMargin` is provided.
  2. **Structural memory-turn discriminator (`check_turn_structure`):**
     - Passes `QueryCurrent`, `Query`, and `Unresolved` through untouched.
     - Rejects candidate writes on interrogative turns (ending in `'?'` or opening with question/request cues such as `"what"`, `"where"`, `"who"`, `"remind"`, `"tell"`).
     - Rejects negated statements (`"not"`, `"never"`, `"isn't"`, `"don't"`, etc.), conditional/hypothetical turns (`"if"`, `"whether"`, `"unless"`, `"suppose"`, `"were"`), and third-person reported speech (`"said"`, `"told"`, `"claimed"`, etc.).
     - Rejects candidate writes whose frame attributes the relation to a third-person subject via general syntactic rules: any third-person subject/possessive pronoun (`"his"`, `"her"`, `"their"`, `"he"`, `"she"`, `"they"`), any possessive noun `<noun>'s` outside the closed-relation slot possessives (`"name's"`, `"dog's"`, `"cat's"`, `"pet's"`, `"friend's"`), or a kinship/person noun modified by `"my"`/`"our"` (with zero panel-specific noun blocklist).
     - Requires the words outside `span` to match the speaker-grounded lexical frame of the target closed relation (`1..=10`, [`canonical_relation_name`](../../../crates/uor-r4-training/src/relation_compiler.rs); bare `"I'm {v}"` / `"I am {v}"` for `user_name` requires a capitalized proper-name token so lowercase small talk like `"I'm tired."` is rejected, and `code_word` requires present-tense `"code word is"` / `"code word be"` grounding so narrative `"The code word in the film was Rosebud."` is rejected) or a first-person copular open-relation phrase (`relation >= DERIVED_RELATION_ID_BASE`).
- **[`gate_compiled_action`](../../../crates/uor-r4-training/src/relation_compiler.rs) & [`resolve_op_policy_gated`](../../../crates/uor-r4-training/src/relation_compiler.rs):**
  Any candidate `Assert` or `Correct` rejected by `MemoryTurnGate` is converted to `CompiledAction::Unresolved { reason }` and is **never** turned into a read (`QueryCurrent` / `Query`).

---

## 3. Finite Host Verification Summary (Pre-Flight Controls; GPU Measurement Pending)

Verified on host via `cargo test -p uor-r4-training --lib relation_compiler` (finite host pre-flight against synthetic adversarial controls and generator templates; the pre-registered GPU evaluation on `compiler-save-op-v25-rawtable` + `op-model-v25` awaits non-author `TEST FITNESS: FIT` and owner release on `#2855`):

| Test | Scope | Result |
|---|---|---|
| `memory_turn_discriminator_and_margin_gate_controls` | Additive score-gap margin checks ($\Delta_{\text{rel}}, \Delta_{\text{act}}, \Delta_{\text{span}}$); preservation of `QueryCurrent`, `Query`, and `Unresolved`; write-to-read prohibition on questions; rejection of `#1718`, negation, conditional, reported-speech, lowercase `I'm <adj>` small talk, and narrative `code_word` controls under `OpPolicy::UnlessQuery` | **PASS** |
| `memory_turn_gate_preserves_all_world_v2_relation_statement_templates` | Iterates directly over `milestone_world_v2::relations()` across `Split::Train` and `Split::Development` (`79 / 79` generator `assert` + `update` templates across all `10` relations), plus `17` teacher-paraphrase templates and `3` open-relation templates (`>= DERIVED_RELATION_ID_BASE`) | **PASS (`79 / 79` generator + `17 / 17` paraphrase + `3 / 3` open preserved, `0` false rejections)** |
| `gated_turn_compiler_wires_configurable_gate_and_integer_margin` | Verifies that `GatedTurnCompiler<C>` implements `TurnCompiler` with configurable `with_memory_turn_gate` / `set_memory_turn_gate` and optional integer `StatementMargin` callback (`with_margin_fn`), rejecting sub-margin or non-relation writes while preserving `QueryCurrent`, `Query`, and `Unresolved` | **PASS** |
| `memory_turn_gate_achieves_zero_false_writes_on_frozen_nonrelation_prose_panel` | Host synthetic-adversary check via `grade_prose_panel`: all `50` frozen rows across all `6` strata of [`data/panels/nonrelation-prose-v1.txt`](../../../data/panels/nonrelation-prose-v1.txt) swept through `GatedTurnCompiler` across all `10` closed relations and derived open relations (`1,100` adversarial candidate writes; deliberate-break control `MemoryTurnGate::disabled()` yields `50 / 50` false writes, gated yields `0 / 50`) | **PASS (`0 / 50` false writes across all `22` host sweeps)** |
