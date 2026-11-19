def groupname: if .row>=300 then "exposed" elif .row>=209 then "unsupported" elif (.row==203 or .row==204) then "secondary" else "primary" end;
def metrics: map(select(.!=null)) | {rows:length, exact:map(select(.exact_through_eos==true))|length,
  error0:map(select(.first_mismatch.offset==0))|length,
  selected:map(select(.source!=null))|length, false_writes:map(select(.false_copy))|length,
  literal_refusals:map(select(.literal_refusal))|length, eos:map(select(.trace.generated_ids[-1]==1))|length};
[.[].rows[]] as $rows |
{groups:($rows | group_by(groupname) | map({key:(.[0]|groupname),value:
 (. as $g | [ "ref","nl","wl","nc","wc" ] | map(. as $arm | {key:., value:{
  ordinary:([$g[].arms[$arm].ordinary]|metrics),
  supplied_query:([$g[].arms[$arm].supplied_query.writer]|metrics),
  supplied_fact_and_query:([$g[].arms[$arm].supplied_fact_and_query.writer]|metrics)}}) | from_entries)}) | from_entries),
 rows:[$rows[]|. as $r| {row,group:groupname,reading,
  first_ordinary_error:.arms.wc.ordinary.first_mismatch,
  ref_requested_role_words:.arms.ref.decision.role_words[:2],
  reference_supplied_source:.arms.ref.supplied_fact_and_query.writer.source,
  reference_supplied_first_error:.arms.ref.supplied_fact_and_query.writer.first_mismatch,
  wc_supplied_first_error:.arms.wc.supplied_fact_and_query.writer.first_mismatch,
  ordinary_text:.arms.wc.ordinary.text, forced:[.forced_legal_controls[]|{source,exact_through_eos,ids:.trace.generated_ids}]}]}
