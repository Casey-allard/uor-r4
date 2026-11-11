def family($report; $name):
  $report[0].by_archetype[$name].asked_value_present;

$model_fresh[0] as $fresh |
(
  (family($model_fresh; "memorize") +
   family($model_fresh; "paraphrase") +
   family($model_fresh; "selection") +
   family($model_fresh; "update"))
) as $grounded |
family($model_fresh; "memorize") as $memorize |
family($model_fresh; "paraphrase") as $paraphrase |
family($model_fresh; "selection") as $selection |
family($model_fresh; "update") as $update |
(
  $fresh.by_archetype["wrong-memory-control"].abstained +
  $fresh.by_archetype["no-memory-control"].abstained
) as $abstained |
(
  [$fresh.rows[]
   | select(.scored_question and .archetype != "copy" and .distractor_present)]
  | length
) as $distractors |
(
  [$fresh.rows[]
   | select(.scored_question and .archetype != "copy")]
  | length
) as $distractor_denominator |
family($model_fresh; "copy") as $copy |
(
  [$fresh.table_baseline.by_archetype.memorize,
   $fresh.table_baseline.by_archetype.paraphrase,
   $fresh.table_baseline.by_archetype.selection,
   $fresh.table_baseline.by_archetype.update]
  | map(.asked_value_present) | add
) as $table_grounded |
(
  [family($base_fresh; "memorize"),
   family($base_fresh; "paraphrase"),
   family($base_fresh; "selection"),
   family($base_fresh; "update")]
  | add
) as $base_grounded |
(
  $grounded >= 20 and
  $memorize >= 5 and
  $paraphrase >= 5 and
  $selection >= 5 and
  $update >= 5
) as $p1 |
($abstained >= 10 and $distractors <= 10) as $p2 |
($copy >= 11) as $p3 |
(
  if ($p3 | not) then 0
  elif $p1 and $p2 then 1
  elif $p1 then 2
  elif $p2 then 3
  else 4
  end
) as $row |
{
  schema: "uor-r4.aligned-corpus-smoke-decision/3",
  run_id: $run_id,
  source_head: $source_head,
  code_pin: $code_pin,
  implementation_source: $implementation_source,
  job_sha256: $job_sha256,
  launcher_sha256: $launcher_sha256,
  watcher_sha256: $watcher_sha256,
  scorer_sha256: $scorer_sha256,
  review_sha256: $review_sha256,
  input_manifest_sha256: $input_manifest_sha256,
  bundle_sha256: $bundle_sha256,
  base_artifact_sha256: $base_artifact_sha256,
  scenario_receipt_sha256: $scenario_receipt_sha256,
  preflight_development_sha256: $preflight_development_sha256,
  preflight_fresh_sha256: $preflight_fresh_sha256,
  contract_semantic_blake3: $contract_semantic_blake3,
  artifact_sha256: $artifact_sha256,
  p1_grounded: $p1,
  p2_controls: $p2,
  p3_copy_positive: $p3,
  row: $row,
  grounded_asked_value: $grounded,
  by_family: {
    memorize: $memorize,
    paraphrase: $paraphrase,
    selection: $selection,
    update: $update
  },
  control_abstentions: $abstained,
  distractor_denominator: $distractor_denominator,
  non_copy_distractors: $distractors,
  copy_asked_value: $copy,
  previous_token_grounded: $table_grounded,
  base_artifact_grounded: $base_grounded
}
