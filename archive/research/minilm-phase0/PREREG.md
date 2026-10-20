# Phase 0 pre-registration: MiniLM-teacher clause retrieval vs a lexical baseline

References #820. Written and committed before any arm was scored. The code at
this commit (`crates/uor-r4-training/src/clause_probe.rs`,
`crates/uor-r4-training/src/bin/clause-index-probe.rs`) is the frozen
implementation of every rule below; the results are added in a later commit
without changing these rules.

## Question

Our 96M geometric chat model's memory failures are mostly selection errors: it
copies the other entity's value ("The turtle is Shelby and the goldfish is Flash
… which name did we give the turtle?" → Flash). The proposed mechanism is a
D11-servable semantic clause index: split the conversation log into clauses,
embed each clause with static per-token vectors distilled offline from MiniLM,
quantize those vectors into E8 lattice blocks, score question vs clause with
integer/table inner products and restrict the pointer to the best clause.
Phase 0 asks only whether such an index picks the right clause. It is not a
reply-quality, chat or held-out result.

MiniLM (`sentence-transformers/all-MiniLM-L6-v2`, Apache-2.0) is an offline
teacher and comparator only. It is never served and authors no reply.

## What was done before this commit (no arm scored)

- Downloaded (owner-approved) `model.safetensors` (sha256 `53aa5117…d9db`),
  `tokenizer.json` (`be50c362…2037`), `config.json`, `modules.json`.
- Teacher parity check (a forward check, not an arm): the Rust BERT forward
  reproduces the published sentence-transformers quickstart cosine matrix for
  ("The weather is lovely today.", "It's so sunny outside!", "He drove to the
  stadium.") = [[1, 0.6660, 0.1046], [0.6660, 1, 0.1411], [0.1046, 0.1411, 1]]
  with max |deviation| 4.5e-5 (tolerance 1e-3), and the tokenizer gives
  `[101, 2023, 2003, 2019, 2742, 6251, 102]` for "This is an example sentence".
- Unit tests of segmentation, value matching, the lexical scorer, the E8
  round trip, the integer ranking and the Hopf map pass.
- Generated the generator-dev split (below) and counted question categories.

## Datasets

1. **generator-dev** (decision set). `dialogue-recall-corpus generate
   generator=v2 seed=1 dialogues=1 dev_seed=1000003 dev_dialogues=300
   source_commit=12f5caa5`, tokenizer
   `bundle-learned-1/tokenizer.json` (sha256 `d36d3e87…0f89`; it only affects
   the token stores, not `dev/dialogues.jsonl`). `panels=` is a directory of
   symlinks to every `data/panels` file except `conversational-v4*`, so this
   probe never reads the held-out panel; the draw is therefore not guaranteed
   byte-identical to a default-panels run. `dev/dialogues.jsonl` sha256
   `f501281c618ea86f21c6af7d993dca4d345b09bd21cec86e6ca028cb0318c5c1`.
   Scored items: every question with a non-null `expect` (376 of 433; the 57
   abstention questions have no expected value and are excluded). History =
   all messages (user and assistant) before the question's user turn
   (`messages[..turn-1]`); question = `messages[turn-1]`. Expected = `expect`;
   forbidden = `forbid` (`forbid_keys` are not used).
2. **v3** (descriptive only, 30 rows). `data/panels/conversational-v3.json`
   rows with category `multi_turn_memory`; history = all user turns but the
   last; question = the last turn. Expected = the `terms` column and forbidden
   = the `forbid` column of `conversational-v3-checks.tsv` (`|` separates
   spellings; `-` is empty; `keys` is not used).

`data/panels/conversational-v4*` is never read or evaluated (the binary also
refuses any path naming it).

## Clause segmentation (frozen)

Per message, in log order:
1. Sentence split after a run of `.`, `!`, `?` (plus closing quotes/brackets)
   followed by whitespace or end of text, and at every `;`.
2. Each sentence is split at every case-insensitive occurrence of `" and "`,
   `" but "`, `" / "` (with the spaces as written; a sentence-initial "And"/"But"
   is not a split).
3. Pieces are trimmed; pieces without an alphanumeric character are dropped.

Recency = later position in the log.

## Value matching and metric (frozen)

`words(text)`: lowercase runs of letters/digits/apostrophes, outer apostrophes
removed, trailing possessive `'s` stripped. A spelling is present in a clause
when every one of its non-stopword words (all of its words if it has none) is
a word of the clause. Stopwords: the fixed `STOPWORDS` list in
`clause_probe.rs` (NLTK English plus contraction pieces; question words are
stopwords).

- **Top-1 clause hit**: the top-ranked history clause contains an expected
  spelling and no forbidden spelling.
- **Same-clause (segmentation limit)**: some history clause contains both an
  expected and a forbidden spelling. Reported per dataset.
- **Reachable**: some history clause contains an expected spelling and no
  forbidden spelling (the ceiling for any arm under this segmentation).
  Reported per dataset.

All arms break ties (including all-equal scores) toward the most recent
clause.

## Arms (frozen)

- **(L) lexical**: number of distinct non-stopword `words` shared by question
  and clause.
- **(M) MiniLM contextual (upper bound)**: full sentence-transformers pipeline
  on the raw question and raw clause text (`[CLS] … [SEP]`, truncation 256,
  mean pooling, L2 normalization), cosine.
- **Static table**: each vocabulary id `t` alone as `[CLS] t [SEP]`, last
  hidden states mean-pooled over the three positions (Model2Vec-style without
  PCA or Zipf weighting), then **centered** by subtracting the vocabulary mean
  vector. Token set of a text: WordPiece ids of its non-stopword,
  non-punctuation BERT basic tokens (fallbacks: all non-punctuation tokens, then
  all tokens). Text vector = sum of the token rows.
- **(S) static float**: cosine of the summed centered rows. (Descriptive
  `S_raw`: the same with the uncentered table.)
- **(E) static E8**: each centered 384-d row is cut into 48 blocks of 8; block
  `y = (2 / rms) · v_block`, where `rms` is the root-mean-square entry of the
  whole centered table, is replaced by its nearest E8 point
  (`b3_e8_codecs::e8_nearest`, E8 = D8 ∪ (D8 + ½)) and stored as doubled
  integer coordinates (`i8`). Text code = integer sum of token codes. Clauses
  are ranked by `dot(Q, C) / |C|` (the cosine up to the constant question norm),
  compared exactly in integers (`sign(dot)·dot²·|C'|²` cross-multiplied in
  `i128`). By bilinearity `dot(ΣQ, ΣC)` equals the sum over token pairs and
  blocks of E8-point inner products, which a finite pair table over the
  realized block codes can supply without multiplying runtime values; the
  probe computes the identical integer directly. The length normalization is
  still a runtime product in this probe (an open serving item, reported as
  such). Reported: scale, global and mean per-token relative quantization
  error, per-token cosine, distinct block codes and pair-table size.
- **(H) quaternionic Hopf** (owner addition, registered here before any
  measurement): on the summed centered static vector, each 8-block `x_b` is
  read as a quaternion pair `(q1, q2)` in the project's icosian Z-basis order
  (`h4.1, h4.i, h4.j, h4.k, phi_h4.1, phi_h4.i, phi_h4.j, phi_h4.k`,
  `canonical_lexical_ingestion::fixed_icosian_profile`, construction
  "icosian golden-coupled H4 ⊕ phiH4"): q1 = the H4 quaternion, q2 = the φH4
  companion. Only this pairing/order is taken from the icosian convention; the
  coordinates are real block coordinates, not icosian Z[φ] coefficients. The
  block norm `n_b` is kept as a separate scalar; the unit block (a point of
  S7) is mapped by the quaternionic Hopf map S7 → S4,
  `B = (2·q1·q̄2, |q1|² − |q2|²)` ∈ R⁵. Score = `Σ_b nQ_b · nC_b · ⟨BQ_b, BC_b⟩
  / sqrt(Σ_b nC_b²)` (fiber discarded). Float arithmetic in this probe.
- **(H_F) Hopf + retained fiber** (descriptive): fiber `f = q1/|q1|` (the S3
  coordinate for the section whose first component is real non-negative, so
  `(q1, q2) = s(B)·f`; `f = 1` if `q1 = 0`); per-block term
  `½(⟨BQ, BC⟩ + ⟨fQ, fC⟩)` in place of `⟨BQ, BC⟩`.
- **(H_E8root) quantized Hopf** (descriptive): each unit block is first
  snapped to the nearest of the 240 E8 root directions (Euclidean E8 roots in
  the same 8 coordinates, normalized to S7) and then mapped as in H.

## Decision (frozen)

Candidates: **E** and **H**, each judged on generator-dev top-1 clause hit `c`
against the lexical rate `L`:

- **BUILD** if `c ≥ 90%` and (`c ≥ L + 5 points` or `L < 85%`);
- **DROP** if `c < 75%` or `c ≤ L`;
- otherwise **REPORT-ONLY**.

Overall: BUILD the clause index if any candidate is BUILD (naming it); DROP if
both candidates are DROP; otherwise REPORT-ONLY. v3 (30 rows) is reported
separately and descriptively; it does not enter the decision. M, S, S_raw,
H_F and H_E8root are descriptive.

## Budget

Laptop CPU only (no GPU, no external compute). New laptop data < 1 GB
(model files 91 MB, generator output < 1 MB, report root < 5 MB, builds in
`~/.cache/uor-claude-target`). The static table (47 MB) is computed in memory
and identified by SHA-256; it is not retained. After the run the model
directory is stored with `cloud-store put claude` as
`minilm-l6-v2-weights` and the local copy is removed.
