# External audit packet: Codex geometric-attention review, 2026-09-28

**What this is.** An external research audit written by a Codex session and handed to the owner, who delivered it to the director at about 05:50 UTC on 2026-09-28 with the request to "synthesize it into our current project and correct anything you believe to be correct".
- It is advisory. It is not a lab result, a trained model or a promotion.
- Its source pin is `15d2ce3f` (PR #1449), with the #820 board read at its 04:55 UTC revision.
- It ran on a hosted Linux container, plus read-only Desktop Commander inspection of this Mac.

**Layout:**
- `packet/`: the 18 delivered files, byte-for-byte. SHA-256 values are at the end.
- `director-reproduction/`: the director's reruns and re-derivations.
- The project changes this audit produced are in [whole-project synthesis §7](../../integration/whole-project-synthesis-2026-09-28.md#7-external-audit-integration-codex-2026-09-28) and ROADMAP ruling 11.

**What the director did.**
- Read every file in full. The two HTML maps are static; one has an inline tab and slider script with relative links only.
- Checked each source claim against the named commit.
- Reran both supplied probes locally.
- Re-derived the mathematics and recomputed one construction independently on actual 2I roots.

## Claim ledger

**Status meanings:**
- **Verified in source:** the director read the cited lines.
- **Reproduced:** the director reran the supplied script, with identical checks and script SHA-256.
- **Re-derived:** the director's own derivation or independent computation.
- **Reported:** no script supplied, not reproduced.
- **Superseded:** later project state.
- **Adjusted:** the director changed the recommendation; the reason is given.

| # | Claim | Packet source | Status | How |
|---|---|---|---|---|
| 1 | T2's harness scores **every** previous event, but its reported decode cost charges only the s retained events. It is therefore a compressed exhaustive scan, not a demonstrated sparse index. | `local_t2_observation.json`, `RESEARCH_DECISION.md` §4, `LAB_RESEARCH_BRIEF.md` | **Verified in source** | See note 1. |
| 2 | In T2, the H4 and E8 arms decode to **unit** roots while k-means decodes to **raw** centroids, and no arm has a per-key gain. The geometric-versus-learned contrast is therefore confounded by magnitude. | `local-source-observation.md` | **Verified in source** | See note 2. |
| 3 | #1438's finite read has four parts: per-lane L2 normalization of 16 four-coordinate lanes, a snap to the 120 roots of 2I, the exact relative element, and a learned 4→8→1 tanh MLP per lane, summed over lanes. Magnitude never reaches the scorer. It trains with a straight-through surrogate, and its smooth branch differentiates the live key history. | NOTE §1, NOTE2 §2 | **Verified in source** | See note 3. |
| 4 | #1438's measured outcome and scope. | NOTE2 §2, RESEARCH_DECISION §2 | **Verified in record** | See note 4. |
| 5 | Unit normalization can reverse a dot ranking, even with exact 2I directions and no angular error. | NOTE §2, NOTE2 §3, SYNTHESIS2 §2 | **Re-derived** | See note 5. |
| 6 | `Re(conj(q)·k) = q·k`, and `q·k = ‖q‖‖k‖·Re(u⁻¹v)`. | NOTE §2, NOTE2 §3 | **Re-derived** | Real part of the Hamilton product; `u⁻¹ = conj(u)` for units. |
| 7 | **Finite-sum read reduction.** Records sharing a key cell and coefficient aggregate exactly: `N(q)=Σ_{d∈S} κ(d)ρ(d)M(qd)` and `Z(q)=Σ_{d∈S} κ(d)C(qd)`, with NoRead when Z=0. | `MATHEMATICAL_NOTE.md`, NOTE §4 | **Reproduced**, and **re-derived** | See note 6. |
| 8 | **Binding obstruction.** Bucket sums and counts cannot tell {Alice→blue, Bob→green} from {Alice→green, Bob→blue}. | NOTE §5, SYNTHESIS2 | **Reproduced** in both probes, and **re-derived** | — |
| 9 | Group closure gives no extra resolution (`GG=G`). A group action cannot overwrite, because an overwrite is not injective. | NOTE §2, SYNTHESIS2 §3–4 | **Re-derived** | — |
| 10 | Endpoint transport factors into a change of frame: `Σ aᵢρ(q⁻¹kᵢ)vᵢ = ρ(q)⁻¹ Σ aᵢρ(kᵢ)vᵢ`. | NOTE §4 | **Re-derived** | ρ is a homomorphism. |
| 11 | Error bounds. | NOTE §7, NOTE2 §4.5 and §5, `MATHEMATICAL_NOTE.md` | **Re-derived** | See note 7. |
| 12 | **Orbit codebooks.** Four fixed representatives give 480 distinct codes. A 1,920-entry table `A[i,j,r]`, with `r=g_Q⁻¹g_K`, reproduces all 230,400 pair scores. | SYNTHESIS2 | **Reproduced independently** | See note 8. |
| 13 | **Occupancy.** With M=120 cells, s=8 probes and N=256 keys, about 17.1 candidates are expected. With M=120², it is 0.142, and most queries are empty. | `MATHEMATICAL_NOTE.md` | **Re-derived** | N·s/M and (1−s/M)^N. |
| 14 | Synthetic quantization tables and coarse certificates. | `geometric_attention_probe.json`, SYNTHESIS2, `LAB_RESEARCH_BRIEF.md` | **Reported** | See note 9. |
| 15 | Roadmap scope corrections: (a) the geometry verdict names parameterisations, not families; (b) capacity and exposure are a hypothesis; (c) code-loss parity selects a base without qualifying dialogue, and code BPE and #1017 are different token identities. | RESEARCH_DECISION §3 and §5, NOTE2 §1 | **Accepted** | Applied in synthesis §7 and in ROADMAP §1, §2 and ruling 11. |
| 16 | "The full-exposure base result is not present." "#1437 remains draft; cycle-5 arms unrun." | NOTE §1, SYNTHESIS2 | **Superseded in part** | See note 10. |
| 17 | Lab allocation: "Claude owns the causal attribution study." | RESEARCH_DECISION, `LAB_RESEARCH_BRIEF.md`, SYNTHESIS2 | **Adjusted** | See note 11. |
| 18 | Margin certificates as a pruning mechanism. | NOTE §7, SYNTHESIS2 | **Not adopted** | See note 12. |
| 19 | T2's local report: recall@16 on all positions is h4 0.968, h4-rht 0.970 and kmeans120 0.981. | `local_t2_observation.json` | **Reported** | See note 13. |

**Notes:**
1. At `b0f70c63` in Lab 2's unpublished worktree, `examples/joint-addressing-contest.rs`:
   - `arm_ranking` scores all codes (lines 338–345);
   - its caller passes `code_cache[index][..previous]` (line 832);
   - `decode_spec` charges `per_event.table_reads * s + lut.table_reads` (lines 366–392).
   - The packet also cites `924ffb2d`, with the same pattern.
2. `addressing_arms.rs` at `b0f70c63`: lines 713–720 for H4 and E8, line 725 for k-means.
3. `crates/uor-r4-training/src/geometric_read.rs` on main: lines 1–19, 365–373 and 460–490.
4. `geometric-read-result-2026-09-27.md`, lines 32–35, 43, 50 and 61–66:
   - read NLL 1.984752805 → 2.004471395;
   - complete answers 27 → 17 of 32;
   - cost 1.84× against a threshold of 1.5×;
   - an initial disturbance of about +0.4 nats;
   - arm C (norm-controlled) deferred, so the attribution is unresolved.
5. Two worked cases:
   - With q=(1,0,0,0), A=(4,3,0,0) and B=(6,8,0,0), the dot scores are (4, 6) and the unit scores (0.8, 0.6).
   - With q=(1,0,0,0), kA=(¼,0,0,0) and kB=(1,1,1,1), the dot scores are (0.25, 1) and the unit scores (1, 0.5). `kB/2` is a 2I element; this was recomputed on the 120 roots in `director-reproduction/orbit_check_results.json`.
6. Reproduced with `algebra_probe.py` (`e1cf6961…`) and `group_attention_probe.py` (`22c895f8…`):
   - 1,728,000 associativity, representation and left-frame cases;
   - 1,200 direct-versus-addressed reads;
   - 485 NoRead queries;
   - 256 insert/evict steps.
   The local outputs are `*.local.json`; their timestamps and runtimes differ from the packet's, and every check agrees. Re-derivation: each record lies in exactly one relative class `d=q⁻¹k`.
7. The bounds:
   - `|q·k−q̂·k̂| ≤ ‖q−q̂‖‖k‖+‖q̂‖‖k−k̂‖`;
   - a margin Δ>2ε preserves the argmax;
   - omitted mass `η ≤ m·e^U/(Z_S+m·e^U)`;
   - the one-step read error `‖o−ô‖ ≤ 2Rη + R·min{2, e^{2ε}−1} + ε_v`.
8. `director-reproduction/orbit_check.py` builds the 120 roots with φ, checks closure, and compares all pairs. The maximum discrepancy is 1.2×10⁻¹³ in float64.
9. The reported numbers:
   - unit 2I: 25/192;
   - exact-gain oracle: 121/192;
   - 4-bit dyadic gain: 112/192;
   - coarse certificates: 0/192;
   - SYNTHESIS2's orbit table: 39–95/128.

   The generator scripts (`verify_attention_math.py`, `finite_group_checks.py`, `orbit_codebook_checks.py` and the two probe generators) are not in the packet. The data is random synthetic data, some arms use unequal bits, and none of it is language evidence.
10. D0 was decided at 05:50 UTC: the stack scored 1.998113 against the control's 2.011149 (#1437, receipt comment 5864376036). The cycle-5 memory arms remain unrun.
11. D6, the evaluation-only information audit, goes to **Lab 2**, which owns both #1438's reader and evaluator and T2's query/key accessor. Lab 1 runs #1433's endpoints and D2. This follows the packet's own OpenCode allocation of "reusable traces … admission-versus-ranking accounting".
12. The packet's own result is 0/192 certified at the coarse precision. It is a later option, not a prerequisite.
13. Read from a local, unpublished report (`full-2/addressing-contest.json`, `e277fb7d…`). It has not been re-verified. Lab 2 publishes it with the cost relabel (ruling 11).

NOTE is `GEOMETRIC_ATTENTION_RESEARCH_NOTE.md`, NOTE2 is `GEOMETRIC_ATTENTION_RESEARCH_NOTE2.md` and SYNTHESIS2 is `RESEARCH_SYNTHESIS2.md`.

## What the packet gets right, in one paragraph

The failed geometric read did not test "geometry"; it tested one parameterisation that changed three things at once and never ran its attribution control. The project's own records already said this, but the roadmap's summary did not. The T2 contest, as coded, measures compression fidelity under an exhaustive scan, and its geometric arms are handicapped by discarding magnitude. So neither result can retire geometric reads or crown ordinary ones. The packet's constructive idea is that a group-relative score can **compile to addresses**: visit cells `g_q·d` for a small support S, and keep exact postings, because sums lose bindings. That gives geometry a computational role, reducing what is read, rather than a cosmetic one. It needs a gain channel, an equally informed ordinary control and a whole-path cost account. The director adopts the information audit first (D6), and the geometric address channel only conditionally (D7).

## Packet file identities (SHA-256)

| File | SHA-256 |
|---|---|
| `GEOMETRIC_ATTENTION_RESEARCH_NOTE.md` | `77f9c77d8ffb0d6a1b759949af45a1418328eba09aaba4c5a569fb9a9facb20e` |
| `GEOMETRIC_ATTENTION_RESEARCH_NOTE2.md` | `82791129cf8ab08e00f3b8ebdf2006393527051004060fb996fd67a504c3cfdb` |
| `GEOMETRIC_ATTENTION_SYNTHESIS.md` | `378e124f553a2d8ba4d7477db6b71ec14ddc9b946c187fc6fa133e0c63dc9402` |
| `LAB_RESEARCH_BRIEF.md` | `1303c57b173312e33547d50d245dad437fd3e0b567076847887420631d10fc0a` |
| `MATHEMATICAL_NOTE.md` | `08be6e4c8e5385a37c48157fa02207be30e5130cc02b5a246ac6b2f2f9222786` |
| `RESEARCH_DECISION.md` | `a5af567c0765687d971d44119990975bacadc683f5a7ebb8a37abed09197f3aa` |
| `RESEARCH_SYNTHESIS.md` | `105ef34aad967063bd1989f269c108e077d89e81361ebbd78759405e070f7ae5` |
| `RESEARCH_SYNTHESIS2.md` | `da591b52c997a5ac668b5ff8893cd7abdfd8a7dafd2eafcb0842cdaf94e50017` |
| `algebra_probe.py` | `e1cf69612e0859f8f6f60064bba957266283b90f8d132edc232b197c3e3f770b` |
| `algebra_probe_results.json` | `045ab38d2df1b8cbbded3c9df05e17ab14c73151f8b4f77419ba98615534d0a6` |
| `decision_map.html` | `eda2d1ec69b0b7185f7b26aaed96c31db25a5d77dc620254c1645cd4cc0b0834` |
| `geometric_attention_probe.json` | `ced7ebbaf5818cca16c7467a0764b2c79345e59896fa78db3a945d6ae589c3b5` |
| `group_address_probe.json` | `0123a833a4e5ad5ea8c1a343f63fdefeea6d2d02d8b805d47f078be55464c313` |
| `group_attention_probe.py` | `22c895f899a4c927f5dbe4cf38f882ed2b833902efe9277b9fa34dfc87dff57e` |
| `group_attention_probe_results.json` | `2515c2300d60c8477e293a4cce729d79425981585cbba87ee751ae098bf36576` |
| `local-source-observation.md` | `7a721e6c9238ee16b2047f935841d5ac7ac4846e2de78ac6dffb8e9d89f1e40c` |
| `local_t2_observation.json` | `c3b440f3ca79de2b76ebafababff6cc8743fc99e05736e5357a8f8c0f4db6b61` |
| `mechanism-map.html` | `504b87beffdd88b736bea460928c5a0ac531aeb23eb1d46ac5a6e8687e4deeef` |

Reproduce the probes with the system Python that already has NumPy; nothing was installed. Each script writes its results next to itself, so run a copy, keeping `packet/` byte-for-byte:

```bash
cp docs/evidence/external-codex-audit-2026-09-28/packet/algebra_probe.py "$TMPDIR/" && OPENBLAS_NUM_THREADS=1 /usr/bin/python3 "$TMPDIR/algebra_probe.py"
```
