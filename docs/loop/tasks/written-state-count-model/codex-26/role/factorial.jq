def stratum:
  if .row >= 401 then
    if .row >= 413 then "held_unsupported"
    elif (.row==401 or .row==402 or .row==409 or .row==410) then "secondary"
    else "primary" end
  elif (.expected_reply=="I don't know.") then "exposed_unsupported"
  else "exposed_directional" end;
def route($source; $fallback):
  . as $case | .exact as $e |
  (if $fallback then $e.last_supported_clause else null end) as $intervention |
  ($intervention.decision // $e.ordinary) as $decision |
  (if $intervention!=null then $intervention.requested else $e.requested end) as $role |
  (if $intervention!=null then $case.last_supported_reply else $case.ordinary_reply end) as $reply |
  if $reply==null then error("missing writer tape") else
   {row:$case.row,group:($case|stratum),source_policy:$source,role_policy:(if $fallback then "last_supported" else "ordinary" end),
    role:$role,gold_role:$e.gold_query,source:$decision.choice.source,gold_source:$e.gold_source,
    supporting_facts:$decision.supporting_facts,complete_values:$decision.complete_values,values_disagree:$decision.values_disagree,
    exact:$reply.exact_tape,eos:$reply.ends_eos,first_error:$reply.first_mismatch,text:$reply.text}
  end;
[.[] | (if .recurrence_carrier_sha256 then "recurrence" else "legacy" end) as $source |
 if .sealed_client_compared!=true then error("ordinary client unchecked") else .rows[] end |
 route($source;false),route($source;true)] as $rows |
if ($rows|length)!=120 or ($rows|unique_by([.row,.source_policy,.role_policy])|length)!=120 then error("missing duplicate or conflicting factorial rows") else
{summary:($rows|group_by([.group,.source_policy,.role_policy])|map({group:.[0].group,source:.[0].source_policy,role:.[0].role_policy,n:length,
 selected_roles:map(select(.role!=null))|length,correct_annotated_roles:map(select(.gold_role!=null and .role==.gold_role))|length,copies:map(select(.source!=null))|length,
 exact:map(select(.exact))|length,eos:map(select(.eos))|length,first_at_zero:map(select(.first_error.offset==0))|length,
 false_writes:map(select(.gold_source==null and .source!=null))|length})),rows:$rows}
end
