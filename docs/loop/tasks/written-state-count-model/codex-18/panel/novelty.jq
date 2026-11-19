# Inputs are the registered historical JSON strings, held raw text and grades,
# and both frozen TRAIN grade sets. No model output is consumed.
def occurrences($haystack; $needle):
  [$haystack[] | select(test("(^|[^A-Za-z])" + $needle + "($|[^A-Za-z])"))] | length;
def distinct_field($rows; $key):
  [$rows[].facts[] | .[$key]] | unique;
def overlap($train; $held; $key):
  distinct_field($train; $key) as $train_values |
  distinct_field($held; $key) as $held_values |
  {train_count: ($train_values | length), held_count: ($held_values | length),
   overlap: [$held_values[] as $value | select($train_values | index($value)) | $value]};

$history[0] as $hist |
$grades[0] as $held |
($base[0] + $supplement[0]) as $train |
{history_unique_strings: ($hist | length),
 exact_prompt_hits: ([$raw[0][] | .text as $text | select($hist | index($text))] | length),
 person_mentions: ([$held[] | (.requested_name // empty), .facts[].name] | unique |
   map({surface: ., hits: occurrences($hist; .)})),
 connectors: (distinct_field($held; "verb_phrase") |
   map({surface: ., hits: occurrences($hist; .)})),
 full_literal_values: (distinct_field($held; "value") |
   map({surface: ., hits: occurrences($hist; .)})),
 frozen_train_field_overlap: {
   factual_names: overlap($train; $held; "name"),
   connectors: overlap($train; $held; "verb_phrase"),
   literal_values: overlap($train; $held; "value")
 }}
