# #639 — tokenizer: SentencePiece/Unigram adapter as a versioned registry entry (pinned source + license precondition)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-12T11:09:33Z
- **Updated:** 2026-08-15T00:00:56Z
- **Closed:** 2026-08-15T00:00:56Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/639
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Spawned from #601 (Ari-program campaign). #601 records `sentencepiece-unigram` as a recognized-but-rejected family in the versioned tokenizer-adapter registry (`hf_bpe::adapter_constructor` refuses it by name via `SourceIngestKind::UnknownTokenizerAdapter`); this issue implements a real SentencePiece/Unigram adapter as a new `(family, version)` registry entry with its own `TokenizerAdapter` policy derivation and differential source fixtures. Scope mirrors #601: no change to the byte-level BPE family, the legacy llama2.c tokenizer, or any κ-pinned artifact; an unsupported SentencePiece feature set must fail closed by name, never be approximated. PRECONDITION (from #601's own scope): a chosen source snapshot with pinned revision and license record — do not start without one. Filed unassigned per board rules.

## Comments (2)

### comment 5294022775 — 2026-08-14T13:45:48Z — Casey-allard

## Pinned source + license record (precondition satisfied)

Casey chose **T5** as the SentencePiece Unigram source.

- Repository: `google-t5/t5-base`
- Revision: `a9723ea7f1b39c1eae772870f3b547bf6ef7e6c1`
- Tokenizer file: `spiece.model` — 791,656 bytes, `blake3:18af7072534a34d626ca166c92f5e37c7ccfcd6ce6877c3c25d325bc8201090d` (sha256 `d60acb128cf7b7f2536e8f38a5b18a05535c9e14c7a355904270e15b0945ea86`)
- License: **Apache-2.0** (permits pinning/vendoring; license text captured with the descriptor)

## Decomposition (scoped from the code)

The registry's `AdapterConstructor = fn(&[u8]) -> Option<HfBpeTokenizer>` returns the byte-BPE type concretely, and `TokenizerKind` is a fixed `Legacy | HfBpe` enum — so a real Unigram family is a multi-PR build, not a one-liner:

- **B/639-1 — source pin.** `models/t5-base-tokenizer.json` descriptor (repo/revision/κ/bytes/scope/license, mirroring `models/gpt2-124m.json` #669) + presence-gated κ-verification test; `.uor-models/sources/t5-base-tokenizer` gitignored. Small, self-contained.
- **639-2 — registry generalization.** Introduce a tokenizer trait (or widen `AdapterConstructor` / `TokenizerKind`) so a family can encode/decode without being an `HfBpeTokenizer`, with `hf-byte-bpe/1` behavior byte-unchanged (regression gate).
- **639-3 — Unigram adapter.** Parse the `spiece.model` proto (vocab + log-probs) and implement Unigram encode (Viterbi/forward-DP) + decode, its `TokenizerAdapter` policy derivation, and the versioned `(sentencepiece-unigram, 1)` registry entry replacing today's refuse-by-name. Unsupported SP features fail closed by name, never approximated.
- **639-4 — differential fixtures.** Presence-gated parity of the adapter against a reference (e.g. the HF `tokenizers` output) on pinned T5 prompts; determinism double-run.

No change to `hf-byte-bpe`, the legacy llama2.c tokenizer, or any κ-pinned artifact throughout.

### comment 5299333573 — 2026-08-15T00:00:55Z — Casey-allard

## Parent close-out

The pinned-source SentencePiece/Unigram adapter is now complete end to end.

Merged delivery chain:

- #706: pinned T5 tokenizer source and presence-gated identity checks
- #708: tokenizer registry generalized beyond byte BPE
- #710 and #711: Unigram parser/Viterbi core and sanctioned error-surface correction
- #716: nmt_nfkc normalization and initial immutable sentencepiece-unigram/1 registration
- #719 / issue #718: comprehensive consumers, reference-correct current /2, external differential fixture, runtime export, provenance, graph binding, and fail-closed serving

The final merge is a3def9dbb63f3ea54f248a02a61606ee6fd44905. Its binding merge-queue run is https://github.com/UOR-Foundation/uor-r4/actions/runs/31851321265 and full delivery evidence is recorded at https://github.com/UOR-Foundation/uor-r4/issues/718#issuecomment-5299332003.

The pinned-source precondition, versioned registry entry, unsupported-feature refusal, independent reference differentials, deterministic double export, source/runtime identity binding, docs, and every in-scope host consumer are satisfied. Closing the parent as completed.


---
_Comment/review ids in this file: 5294022775, 5299333573_

