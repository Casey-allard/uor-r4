# Input-only overlap audit against codex-12/13 paired-role and region receipts.
def overlap($a; $b): ($a|unique) - (($a|unique) - ($b|unique));
def low: ascii_downcase | gsub("\\s+";" ");
def normalized_wrapper: low | gsub("\\{[rbcx]\\}";"{name}");
def historical_question($names):
  low | gsub("\\b(" + ($names|join("|")) + ")\\b";"{name}");
def sections($g):
  {names: ([$g[] | (.facts[].name,.question_mentions[][0]) | low] | unique),
   literals: ([$g[] | .facts[].value | low] | unique),
   wrappers: ([$g[].delexicalized | normalized_wrapper] | unique),
   questions: ([$g[].question | low] | unique)};
def normalized_text($blind): [ $blind[].text | low ] | unique;
def prefixes($blind): [ $blind[].prefix ];
def audit($new_grade; $new_blind; $new_bounds; $historical):
  sections($new_grade) as $new |
  ([range($new_blind|length) as $i |
    $new_blind[$i].prefix[$new_bounds[$i].start:$new_bounds[$i].end][]] | unique) as $ids |
  {full_names_and_case_aliases: overlap($new.names;$historical.names),
   literals: overlap($new.literals;$historical.literals),
   wrappers: overlap($new.wrappers;$historical.wrappers),
   actual_questions: overlap($new.questions;$historical.questions),
   exact_prompt_texts: overlap(normalized_text($new_blind);$historical.texts),
   exact_prompt_ids: overlap(prefixes($new_blind);$historical.tapes),
   shared_question_token_ids: overlap($ids;$historical.question_token_ids)};
($old_train[0] + $old_held[0] + $old_fresh[0]) as $historical_grade |
($old_held_regions[0] + $old_fresh_regions[0] + $old_nuisance_regions[0]) as $regions |
# Source: span-connector-panel/paired_role.rs, all three roster lists and literal adjunct names.
(["kim","bob","ruma","rumi","nala","navi","moe","sam","may","joe","zura","zuri","lava","lavi","jim","roy",
  "ben","eve","vela","veli","sani","sana","umber","onyx","cedar","oak","harbor","summit"]) as $names |
(["For {name} rather than {name}, give the item.","Excluding {name}, what did {name} keep?",
  "Ignoring {name}, which item was kept by {name}?","Regarding {name}, what did the line name?",
  "Which item belongs to {name}?","Find the item for {name}. Ignore {name}, please.",
  "For {name}, beside the mention of {name}, name the item.",
  "Between {name} and {name}, what item did the unnamed person keep?",
  "For either {name} or {name}, tell me their item.","Explain why {name} and {name} are different trees."]
  | map(low)) as $train_wrappers |
{names:$names,
 literals:([$historical_grade[].assertions[].value|low]|unique),
 wrappers:($train_wrappers + [$regions[].question|historical_question($names)]|unique),
 questions:([$regions[].question|low]|unique),
 texts:([$historical_grade[].prompt|low]|unique),
 tapes:[$historical_grade[].prefix],
 question_token_ids:([$regions[]|.prefix[.start:.end][]]|unique)
} as $historical |
{
  scope:"Codex-12 paired role TRAIN/held/fresh grades and Codex-13 held/fresh/nuisance regions. TRAIN question-piece overlap is captured by the current TRAIN-to-held comparison, not historical region intervals.",
  train: audit($train_grade[0];$train_blind[0];$train_bounds[0];$historical),
  held: audit($held_grade[0];$held_blind[0];$held_bounds[0];$historical)
}
