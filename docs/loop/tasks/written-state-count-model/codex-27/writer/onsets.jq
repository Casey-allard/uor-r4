def supported: map(select(.expected_value!=null));
def agrees($rows): all($rows[]; .witness.assertion.continuation.source==.expected_value_span[0]);
def counts: group_by(.) | map({value:.[0],count:length});
def oraclehit: . as $r | any(.assertions[]; .expected_value_span[0]==$r.decision.choice.source);
.[0] as $ordered | .[1] as $rec |
if (($ordered|length)!=92 or ($rec|length)!=92 or ($ordered|map(.row))!=($rec|map(.row))) then error("crossed downstream witness panels") else
{
  supported: ($ordered | supported | length),
  ordered_oracle_exact_addresses: ($ordered|supported|map(select(oraclehit))|length),
  recurrence_oracle_exact_addresses: ($rec|supported|map(select(oraclehit))|length),
  ordered_continuation_correct_on_all_selected_assertions: ($ordered | supported | map(agrees(.assertions)) | counts),
  ordered_assertion_states: ([$ordered|supported|.[]|.assertions[]|{status:.witness.onset.status,continuation_already_gold:(.witness.assertion.continuation.source==.expected_value_span[0])}]|counts),
  recurrence_assertion_states: ([$rec|supported|.[]|.assertions[]|{outcome:.witness.contextual_source.outcome,source_gold:(.witness.contextual_source.source==.expected_value_span[0])}]|counts),
  example_2703: {ordered:($ordered[]|select(.row==2703)),recurrence:($rec[]|select(.row==2703))}
} end
