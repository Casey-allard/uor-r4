# #786 — research: dormant-mechanism re-evaluation — why each is dormant, which activation gates are satisfiable now, and whether the roadmap still matches the original spirit

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-18T14:51:43Z
- **Updated:** 2026-08-18T18:17:23Z
- **Closed:** 2026-08-18T18:17:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/786
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Goal (maintainer direction, 2026-08-18)

We built a lot of mechanisms, measured them, and parked them. Go back through the dormant set and answer, per mechanism: **why is it dormant** (measured negative vs. never-measured-on-real vs. awaiting a prerequisite), **is its activation gate satisfiable today**, **could it help the current mechanisms** (R4G1 serving quality, the #655 track), and — across the whole review — **is the roadmap still correct in the original spirit of the project** (a CPU-first, transformerless LLM with the P-4 integer kernel)? Deliverable is a decision record, not silent activations: every ledger activation gate stays in force; anything worth re-measuring gets its own pre-registered contract brought back for approval.

## Inventory (the 18 `open` ledger rows in `model/ledger.toml` / CONFORMANCE.md, grouped by why-dormant)

**(1) Measured NEGATIVE on record** — dormant for a reason; re-open only if the recorded measurement is invalidated (see the corpus-era check below) or the ledger gate is newly satisfiable:
`cd_space` (#400: CD term executed 0/1,998 before removal), tropical route composition (#626: 1H/2H at 14.51 bits / 0.44% vs NULL 8.57 / 6.2%), IPF reconstruction (#457: consistent joint lands at the unigram floor), reconstructability-as-split-criterion (#456: sub-unigram), code-space subdivision (#460), Hopf sector transport (#422/#306), FMM far-field (#290), granularity (#393), E8 group-keying (#395). Special case: **bott_fock** (#424) — the negative is the *as-shipped decay* (+0.16pp, one SE); the measured lossless ceiling is **+1.02pp** and the record itself says retuning `>> 2` → `>> 7` would recover it. Cheapest positive-ceiling candidate in the whole set; the pre-declared bar already exists in the record.

**(2) Built + synthetically verified; real arms UNAVAILABLE at the time — prerequisites have since moved:**
- `r4-route-attention/1` (#604) — packed, P-4-scanned, differentially tested bit-for-bit, witness-replayable, zero-alloc asserted. The closest transformer-shaped, goal-aligned mechanism in the tree, never evaluated on a real teacher.
- `route-fit/1` ladder (#605) — synthetic 5-stage PASS with a valid instrument; real-teacher stage was UNAVAILABLE for "pinned SmolLM2 snapshot absent from the build env" — **the snapshot IS present on the dev host now** (257 MB at `.uor-models/sources/smollm2-135m-instruct`). Real-corpus stage cited "#531 saturation corpus not yet produced" — **#531 closed COMPLETED 2026-08-11**; verify what scale/β outputs it actually pinned and whether they satisfy the prereq (the ledger reason-strings predate the closure).
- target-operator certificate (#606) — composes the above; currently NOT_PASSING by construction until a real ladder exists.
- `msa-structured-selector/1` (#643) — plug-compatible with route-attention by construction; the named next action (shared A/B harness wiring vs route-attention) was never done.

**(3) Placeholders / certification-and-analysis surfaces, never wired by design:** packed ROUT routing programs + typed transitions (#159 ph.2), holographic encoding (#126), predictive sufficiency, shortlist evaluator, anti-degeneracy, fairness/provenance, reference compiler IR, rate-distortion analyzer, monograph validator, behavioral probes, patch overlay (induction/lifecycle; the patch *chain* is live), standalone `build_graph`.

## The audit-informed validity check (do this FIRST)

#755 established that sharded-observation corpora can be story-scrambled on disk (135m-instruct was 99.93% non-contiguous; 360m-broad 0%), and that the pre-#755 compiler reconstructed context from physical adjacency. **For every group-(1) negative: identify which corpus the measurement ran on and whether its record order predates/was affected by the #755 fix.** A negative measured on a scrambled-context corpus is not a valid negative and must be re-flagged (not re-run yet — flagged, with a re-measure contract for approval). The 500k committed fixture is believed contiguous (legacy generation path); verify rather than assume, per record.

## Deliverables

1. `docs/dormant_reassessment_<this issue>.md` — one row per mechanism: why dormant (with the record cited) → activation gate → satisfiable today? → plausible bearing on current serving/quality? → verdict: **keep-dormant / re-measure (attach pre-registered contract) / retire**. Corpus-era validity column included.
2. A ranked re-measure shortlist with contracts, brought back for maintainer approval before any run. Expected top candidates on current evidence: (a) **#605 real-teacher route-fit arm** (prereq newly satisfied; the pre-registered contract already exists in `route_fit_report.rs` — no new methodology needed), (b) **bott_fock `>> 7` retune** (pre-declared bar exists), (c) msa A/B harness wiring (only if (a) is positive, per its own contract).
3. A closing **roadmap paragraph**: confirm or amend ROADMAP.md against the original spirit, in light of the table — this is the maintainer's "is the roadmap still correct" question, answered with the evidence in one place.

## DoD

The table exists and is complete (all 18 rows + the corpus-era column); the shortlist + contracts posted here; the roadmap paragraph written; zero activations performed under this issue.

**Next action:** build the table + corpus-era check (reading work, no runs), starting from CONFORMANCE.md's ledger rows and each cited record.


## Comments (2)

### comment 5331999634 — 2026-08-18T17:52:19Z — Casey-allard

## Reassessment record (2026-08-18)

The full record is PR #794 → `docs/dormant_reassessment_786.md`: per-mechanism why-dormant table for all 22 open ledger rows, the corpus-era validity check, the S1–S3 re-measure shortlist with draft contracts, and the roadmap-spirit conclusion. Zero activations; zero ledger rows changed. Summary + the parts that need maintainer eyes:

### Corpus-era validity (the check this issue front-loads)

Question: were any dormant-mechanism negatives measured on a #755-era ordering-corrupted corpus? **No.** The committed 500k fixture pair is perfectly story-contiguous — measured today on the pinned dev machine:

```python
import struct
recs = open('crates/uor-r4-core/tests/fixtures/c_recs.bin','rb').read()
n = len(recs)//48   # 48-byte v3 records, story u32 at offset 0
stories = [struct.unpack_from('<I', recs, i*48)[0] for i in range(n)]
crossings = sum(1 for i in range(1,n) if stories[i] != stories[i-1])
seen, revisits, prev = set(), 0, None
for s in stories:
    if s != prev:
        if s in seen: revisits += 1
        seen.add(s); prev = s
meta = open('crates/uor-r4-core/tests/fixtures/c_meta.bin','rb').read()
print('records', n, 'stories', len(seen), 'crossings', crossings, 'revisits', revisits, 'done_flag', meta[24])
# records 500000 stories 2507 crossings 2506 revisits 0 done_flag 1
```

2,506 crossings is the contiguous ideal for 2,507 stories; 0 revisits means no story is split. Router-harness corpora (hopf/geometry/lexical) never pass through `load_corpus_bytes`, so the #755 mechanism could not have touched them. **Every fixture-based dormant negative stands** (#626 tropical, #424 bott-fock bound, and the related non-ledger negatives #290/#393/#395/#422/#306/#457/#456/#460).

### Verdicts (short form)

19 of 22 rows: **keep dormant** — measured-negative on a now-verified-valid corpus, structurally dead, or certification scaffolding that needs an adoption decision rather than a measurement. Three advance to a shortlist:

- **S1 — #605 real-teacher route-fit arm** (rows 18/20/21: `r4-route-attention`, `route-fit`, `target-operator-certificate`). Prerequisite A (pinned SmolLM2 snapshot) is now **met** on the dev host. Prerequisite B is **open**: #531 (saturation corpus) closed COMPLETED on 2026-08-11 **without a closure record** — no verdict, no corpus/β CIDs, outputs unlocated. ⚠️ **Maintainer decision required**: (a) produce a saturation corpus per #531's own method (hours, needs a run contract), or (b) amend the #605/#643 pre-registered contracts to name an existing broad corpus (e.g. the #509 `smollm2-360m-broad` corpus, 0% non-contiguous) — an explicit contract amendment, declared before any run. Either way the compute cost is the #603 `full/1` trace-profile observation pass. Draft contract is in the doc; it finalizes on #605's thread after the (a)/(b) call.
- **S2 — #424 bott-fock decay retune** (`>> 2` → `>> 7`): cheap, fixture-scale, pre-declared bar already in the record (+1.02pp measured ceiling vs +0.16pp ≈ 1 SE as shipped). Ready to contract-and-run on approval.
- **S3 — #643 msa vs route-attention A/B**: strictly after S1 positive, per its own pre-registered contract.

### Roadmap-spirit conclusion

The dormant set contains **no bypassed working mechanism**. The #783/#785 arc localized the current quality losses to serving-side aggregation and decoding (corpus ordering — fixed by #755; the CLI serving the emission-less converter graph while the engine misread store-container bytes as scores — fixed by #785 C1b/C1c, PRs #792/#793; attractor basins on real evidence — open as #784), not to the dormant substrate. **The roadmap as reconciled on 2026-08-18 is confirmed in the original spirit**: bundle quality first, with S1/S2 as the sanctioned probes of the largest untested mechanisms, and no dormant activation except through its own ledger gate.

Closing when PR #794 merges; the S1 (a)/(b) decision and S2 approval are the named next actions (S1 pointer going to #605's thread).


### comment 5332264947 — 2026-08-18T18:17:21Z — Casey-allard

Closing with records: PR #794 merged (docs/dormant_reassessment_786.md). Corpus-era check measured (no dormant negative invalidated), 22-row why-dormant table complete, roadmap-spirit conclusion recorded (confirmed in the original spirit — quality losses live in serving-side decode, not the dormant substrate). Zero activations; zero ledger rows changed. Named next actions: the S1 (a)/(b) corpus decision (pointer on #605) and S2 approval — both maintainer calls, both pre-contracted in the doc.


---
_Comment/review ids in this file: 5331999634, 5332264947_

