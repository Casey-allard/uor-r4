# The frozen writer may generate unsupported literal text after declining a copy.
# Harm is any exact read when there is no defined target, independent of that text.
def first($pred; $want):
  if $want == null or $pred == $want then null
  else [range([($pred|length), ($want|length)]|max) |
        select($pred[.] != $want[.])][0] end;
def wrong($rows; $key):
  $rows | map(.[$key] | if . == null then "complete_or_undefined" else tostring end) |
  group_by(.) | map({key:.[0],value:length}) | from_entries;
def summary:
  {rows:length,
   answerable:map(select(.expected != null))|length,
   old_exact:map(select(.old_exact))|length, new_exact:map(select(.new_exact))|length,
   recovered:map(select(.new_exact and (.old_exact|not)))|length,
   lost:map(select(.old_exact and (.new_exact|not)))|length,
   old_unsupported_copy:map(select(.expected==null and .old_copied))|length,
   new_unsupported_copy:map(select(.expected==null and .new_copied))|length,
   new_harm:map(select(.expected==null and .new_copied and (.old_copied|not)))|length,
   source_changed:map(select(.old_source != .new_source))|length,
   old_first:wrong(.;"old_first"),new_first:wrong(.;"new_first")};
. as $b | $old[0] as $a | $grade[0] as $g |
if ($a.bare!=$b.bare or $a.model!=$b.model or ($g|length)!=($a.rows|length) or
    ($g|length)!=($b.rows|length))
then error("unbound grade") else
 [range($g|length) as $i |
   $a.rows[$i] as $left | $b.rows[$i] as $right | $g[$i] as $gold |
   if $left.prefix_sha256 != $right.prefix_sha256 then error("unbound writer row") else
   {id:$gold.id, pair:$gold.pair, kind:$gold.kind, evidence:$gold.evidence, case:$gold.case,
    construction:$gold.construction, expected:$gold.expected, old_source:$left.source,
    new_source:$right.source, old_ids:$left.generated, new_ids:$right.generated,
    old_exact:($gold.expected!=null and $left.generated==$gold.expected and $left.generated[-1]==1),
    new_exact:($gold.expected!=null and $right.generated==$gold.expected and $right.generated[-1]==1),
    old_first:first($left.generated;$gold.expected), new_first:first($right.generated;$gold.expected),
    old_copied:($left.steps|any(.copied==1)), new_copied:($right.steps|any(.copied==1))}
   end] as $rows |
 if $mode == "rows" then $rows else
   {total:($rows|summary),
    kinds:($rows|group_by(.kind)|map({key:(.[0].kind // "positive"),value:summary})|from_entries),
    evidence:($rows|group_by(.evidence)|map({key:(.[0].evidence // "historical"),value:summary})|from_entries)}
 end
end
