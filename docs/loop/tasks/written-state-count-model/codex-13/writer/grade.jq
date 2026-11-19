def arms: ["original", "region", "adverse", "role_oracle"];
def first($got; $want):
  if $want == null or $got == $want then null
  else [range([($got|length), ($want|length)]|max) | select($got[.] != $want[.])][0] end;
def flags($w; $want):
  first($w.generated;$want) as $first |
  {source:$w.source, ids:$w.generated, exact:($want != null and $first == null and $w.generated[-1] == 1),
   first_wrong:$first, expected_id:(if $first == null then null else $want[$first] end),
   wrong_id:(if $first == null then null else $w.generated[$first] end),
   copied:($w.steps | any(.copied == 1)), stop:(if $w.generated[-1] == 1 then "eos" else "cap" end),
   delimiter_eos:($w.steps[-1].release == 1 and $w.generated[-1] == 1 and ($w.steps | any(.copied == 1)))};
def count(f): map(select(f)) | length;
def summary($arm):
  {rows:length, answerable:count(.expected!=null), unsupported:count(.expected==null),
   exact:count(.[$arm].exact), unsupported_copy:count(.expected==null and .[$arm].copied),
   nonempty_delimiter_eos:count(.[$arm].delimiter_eos), eos:count(.[$arm].stop=="eos"),
   recovered:count(.[$arm].exact and (.original.exact|not)), lost:count(.original.exact and (.[$arm].exact|not)),
   source_changed:count(.original.source != .[$arm].source), ends_at_cap:count(.[$arm].stop=="cap"),
   first_wrong_changed:count(.expected!=null and .original.first_wrong != .[$arm].first_wrong),
   first_wrong_transitions:(map(select(.expected!=null) | [.original.first_wrong,.[$arm].first_wrong] |
       map(if . == null then "exact" else tostring end) | join("→")) | group_by(.) | map({key:.[0],value:length}) | from_entries)};
def all_arms: . as $d | [arms[] as $arm | {key:$arm,value:($d|summary($arm))}] | from_entries;
def groups($field): group_by(.[$field]) | map({key:(.[0][$field] // "unknown"),value:all_arms}) | from_entries;
$a[0] as $orig | $grade[0] as $g |
{original:$orig,region:$region[0],adverse:$adverse[0],role_oracle:$oracle[0]} as $writer |
if any($writer[]; .bare != $orig.bare or .model != $orig.model or (.rows|length) != ($g|length))
then error("models, tapes, or writer lengths differ") else
[range($g|length) as $i | $g[$i] as $row |
  if any($writer[]; .rows[$i].prefix_sha256 != $orig.rows[$i].prefix_sha256)
  then error("writer row prefix mismatch") else
    {id:$row.id,pair:$row.pair,evidence:$row.evidence,kind:$row.kind,case:$row.case,construction:$row.construction,
     prefix_sha256:$orig.rows[$i].prefix_sha256,expected:$row.expected} +
    ([arms[] as $arm|{key:$arm,value:flags($writer[$arm].rows[$i];$row.expected)}] | from_entries)
  end] as $r |
{overall:($r | all_arms), evidence:($r | groups("evidence")), kind:($r | groups("kind")),
 case:($r | groups("case")), construction:($r | groups("construction")), rows:$r}
end
