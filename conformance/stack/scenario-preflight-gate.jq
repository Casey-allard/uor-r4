.schema == "uor-r4.cpu-scenario-grade/2" and
.split == $split and
.artifact_sha256 == $base_artifact_sha256 and
.selected_rows == 1024 and
.prompt_ineligible_rows == 0 and
.corpus.max_tokens == $corpus_context and
(.rows | length) == 1024 and
(.selected_ranks | length) == 1024 and
([.rows[].scenario_id] | unique | length) == 1024 and
(.table_baseline.rows | length) == 1024 and
.by_archetype.copy.asked_value_present == $copy_hits and
(
  [.by_archetype.memorize,
   .by_archetype.paraphrase,
   .by_archetype.selection,
   .by_archetype.update]
  | map(.asked_value_present) | add
) == 0 and
(
  [.by_archetype["wrong-memory-control"].abstained,
   .by_archetype["no-memory-control"].abstained]
  | add
) == 0 and
(
  [.rows[]
   | select(.scored_question and .archetype != "copy" and .distractor_present)]
  | length
) == 0 and
(
  [.rows[]
   | select(.scored_question and .archetype != "copy")]
  | length
) == 882 and
(
  .table_baseline.by_archetype |
  [to_entries[].value |
   .asked_value_present, .abstained, .distractor_present]
  | add
) == 0
