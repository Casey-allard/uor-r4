# Marker gate on the lawful sieve compiler (`m3-memory-turn-discriminator-gate`, #2855, claude-102)

- **Status:** pre-registration, written before any panel or R2 output of this compiler exists. A fresh FIT and a pre-run review follow on #2855. Lab: claude.
- **Supersedes** [`preregistration-DRAFT-2026-10-23.md`](preregistration-DRAFT-2026-10-23.md), which gates `relation_compiler::SavedCompiler`. That compiler's `SparseSoftmax` heads are banned by CHARTER §3.2, and its module is test-only. The draft is kept as history.

## 1. Question

Can the lawful sieve compiler compile wordings it was not built from, by composing training-derived discourse markers with exact training frames, while ordinary prose still never writes to the store? claude-94's baseline was R2 0/220 with panel 0/0/50 (#2856).

## 2. The mechanism (`sieve_marker_gate.rs`; kernel `sieve_frame_compile_marked` in `sieve_frame.cu`)

- **Markers** are cofactors. A word sequence `m` is a marker when some training slot frame `f` is `m` followed by another training slot frame `g` of the same relation, with the same suffix, so `sig(f) = sig(m) · sig(g)`.
  - It is an update marker when it turns an assert frame into an update frame, and neutral when it never changes the act.
  - A sequence that would erase a correction, or that plays both roles, is dropped.
  - Found from the training phrasings only: `actually`, `sorry`, `sorry i meant` (update); `remember`, `please note that`, `people` (neutral). Marker table sha256 `cd79399490c0abcfc14253b5680a282fc840672c5932e925d62a3b95375a125d`, pinned by a test.
- **Markers are relation-independent once derived.** A marker found on one relation's frames may precede any relation's slot frame.
- **Compile.** Each reading strips at most 2 markers from the front, then runs claude-94's sieve and exact alignment on the rest.
  - Markers never precede a query frame.
  - An update marker reads the frame as a correction: its expert becomes `p_relation · q_update`.
- **Margin gate.** Every word outside the value is a marker word or a frame word. Every surviving reading, over every strip, must name one expert and one span; otherwise the turn is `Unresolved`.
- **Unchanged from #2856:** the frame table (sha256 `379c2ecc…604f`), the word rule, the value cap of 4 words, and integer AND and compare only, with no float, score, softmax or matrix product.
- **Host steps:** tokenizing and building the tables are data preparation, and grading reads the GPU's stored decisions.

## 3. Arms, in one launch

| Arm | Turns | Scored as |
|---|---|---|
| **P, the panel** | the 50 frozen turns (`7c29c665…`) | `grade_prose_panel`, a fresh seeded store per turn, writes on |
| **R1** | training phrasings with 8 development values (seed 2856), and each query phrasing once | exact act, relation and span |
| **R2** | development phrasings, the same | exact act, relation and span |

## 4. Bar and outcomes

- **Registered bar.** All four must hold:
  - the owner's panel bar, false_overwrite 0/50, false_assert 0/50, clean 50/50 (#2028 `6356034555`);
  - R1 at least 95% exact;
  - R2 above claude-94's 0/220, so a clean panel by refusal is not a pass;
  - the level of R2 is reported, with no further bar.

| Outcome | Meaning | Unlocks |
|---|---|---|
| An instrument check fails | an instrument fault | repair, then rerun |
| Panel false count above 0 | the marker rule lets prose write | negative for this configuration (D22); the next configuration narrows the markers |
| Panel clean, R1 ≥ 95%, R2 above 0 | bar met for this configuration | `Closes #2855` |
| Panel clean, R2 still 0 | no generalization from markers | negative for this configuration; the gate stays open |
| R1 below 95%, whatever the panel and R2 show | the marker readings break training frames, for example through overlapping markers such as `sorry` and `sorry i meant` disagreeing | negative for this configuration (D22); the next configuration narrows the marker set or the agreement rule |

A pass is claimed at the reported R2 level only: "compiles R2 x of 220", never "generalizes". R2 is deterministic from the text, but it was not computed on the host before this sheet. Its only computation is the registered GPU run.

## 5. Instrument checks

1. **Finite CUDA controls** (`prose-panel-gate controls`): 14 hand-built turns over 6 frames and 2 markers, each with a literal expected decision. They cover plain, update and neutral markers, two markers, the 2-marker cap, an unexplained lead word, a marker before a query, an update before a suffix frame, a marker reading that disagrees with the plain reading, and the empty turn. All 14 must match.
2. **claude-94's 12 sieve controls** (`prose-panel-sieve controls`) still pass, with the same PTX module.
3. **Determinism:** two in-process launches give identical decisions, and a second build (`cuda` against `sieve-cuda`) gives a byte-identical `decisions.json`.
4. **D29 interlock** on both features: the PTX must carry no `ex2`, `lg2`, `tanh`, `mma` or `wmma`.
5. **Host:** `cargo test -p uor-r4-training --lib` passes 74, including the marker-table pin. `ratchet d23` and `d25` are at baseline.

## 6. Pins

| Object | Digest |
|---|---|
| Panel | `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7` (checked by `load_frozen`) |
| Frame table | `379c2ecc4f1d04b49168d76e43ac93f3d07730e1016e0a281608740efb46604f` (`--expect-table`) |
| Marker table | `cd79399490c0abcfc14253b5680a282fc840672c5932e925d62a3b95375a125d` (`--expect-markers`) |
| `sieve_frame.ptx` | recorded by the controls step, then passed as the required `--expect-ptx` |
| Binaries and job script | recorded beside the outputs |

## 7. Cost and stop rule

- One RTX 5090 on Vast, at most 1 hour of lease, about $1 (claude-94's run took about 6 minutes).
- The probe ladder: (a) both control sets and D29, (b) the run's launch time as the throughput check, (c) the run.
- **Stops:** a pin mismatch stops before any launch; a failed control stops the run, and the fault is repaired; outputs go to the HF store (`results/claude/prose-panel-gate-2026-10-25/`, with a SHA256SUMS) before the pod comes down.
- **Lead metric:** tokens/s, upload to download.
