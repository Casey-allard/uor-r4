# #1353 — One shared Generate/Copy/Stop path: ordinary continuation and grounded exact copying (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T03:58:59Z
- **Updated:** 2026-10-09T13:17:45Z
- **Closed:** 2026-09-23T04:01:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1353
- **Merged:** 2026-09-23T04:01:51Z
- **Merge commit:** 126a8d9ae3fe422980071830862030dd5d46c312
- **Base / Head:** main / UOR-Foundation:codex/ordinary-lexical-milestone
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Executes the milestone the [V3 principal review](docs/integration/transferable-lexical-principal-review-2026-09-23.md) and [next brief](docs/integration/deepseek-ordinary-lexical-step-2026-09-23.md) set, against protected base `44c2ccd27cea5d0178f19cca2f33332db4a95ce1` (PR #1352 merge).

## What is delivered

One native served model (`learner/transferable_lexical.rs`, runner `bin/ordinary-lexical.rs`) implements the briefed responsibility:

```text
c        = pinned request meaning + selected evidence/result + exact provenance
h_0      = learned_init(c)
a_t      = learned Generate(token) / Copy(owned occurrence) / Stop
x_t      = token actually emitted by a_t
h_(t+1)  = learned_update(h_t, representation(x_t), event, c)
```

* readout rows = the whole pinned 4096-token vocabulary (Generate) + Copy + Stop; Copy is removed from the scored set with no live owned occurrence and cross-entropy is normalised over legal rows only — a declared hard constraint, labelled as such, not a learned preference;
* the owned span, version authority and the copy cursor stay outside the compressed state;
* one signed 4-bit embedding table is shared between recurrent feedback and the evidence fingerprint, which uses a fixed cyclic position rotation so a reordered span changes it with no position-indexed table;
* serving is ternary weights with power-of-two row shifts plus one artifact-bound dyadic score scale: adds, subtracts, shifts, compares, table reads; no multiplier or float in the declared kernel;
* ordinary source-separated prose and an authored grounded world train **one** artifact under a declared three-phase curriculum.

## Executed evidence

* **Preflight A** — temporal meaning authored from typed source state (`committed && prior_differs`): **32/32** identity-balanced regimes exact on the loaded artifact, four typed blocks separated across regimes, retained route-key rule **contradicted on 16** of them.
* **Preflight B** — two copied identities with request, typed facts, copy length and events fixed: **2/2 actual arms exact**, identity-erased arms **alias**, source-disabled **loses the exact copy**.
* **Held-out source/relation compositions: 3/3** exact (requested views absent from training).
* Loaded five-phase mixed session, 32 served decisions; separate process restores all 36 cases (`REPLAY_OK 36 cases`) including served actions, emitted tokens and state digests.
* **Measured negative, reported plainly:** 7.3722 bits/target on 24 held-out documents versus a **tuned interpolated `(prev,cur)` count reference at 5.1217** (paired interval [−2.4454, −2.0734]) and donor E at 6.8701; +1.7105 [+1.4973, +1.9359] over the fit-only unigram. **Greedy generation collapses.** Per-document 5.64–8.71.
* Cost on the named M1: 673.6 µs/token, 466,711 B artifact, 56,457 nonzero weight reads per served step, **93.9 MiB peak RSS** measured externally, 0 quantised embedding collisions, **energy UNAVAILABLE**.

## Integrity notes

The temporal oracle's `!key_changed` conjunct was revised **prospectively** after a held-out composition exposed it as over-specified; the sealed receipt records the revision and the earlier exposure. The post-copy preflight words are declared class labels, not semantic claims. Two independent runs produced a **byte-identical artifact** (`sha256 fe3e8a63…`); exposed roots `ordinary-lexical-1` and `ordinary-lexical-2` are preserved. 11 focused tests, `cargo fmt --check`, an offline release build and a sealed-root verify with 0 unlisted files pass. Energy, sampling, donor adaptation, broader conversation, executed Rust and any geometric advantage remain open. Ledger charged 3,500,000 ms → 377,915,640 / 390,700,000 ms with the storage margin intact.

References #820, #973. Neither is closed: their complete acceptance is not met.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

