# Use this program after sealing the independent blind endpoint files.
($tok[0].model.vocab|to_entries|map({key:(.value|tostring),value:.key})|from_entries) as $vocab |
def pieces($row): $row.prefix|map($vocab[tostring]);
def word_end($ps;$i):
  ([range($i+1;$ps|length)|select($ps[.]|test("^[A-Za-z]+$")|not)]|first // ($ps|length));
def lexical($row;$name):
  if $name==null then [null,null] else
    pieces($row) as $ps |
    ([range($ps|length) as $i | select($ps[$i]|test("^Ġ[A-Za-z]+$")) |
      word_end($ps;$i) as $end |
      select(($ps[$i:$end]|join("")|ltrimstr("Ġ"))==$name) | [$i,$end]] | last)
    // error("gold name missing in tokenized prompt: "+$name)
  end;
def requested($g):
  if $g|has("query_start") then [$g.query_start,$g.query_end]
  elif $g.token_positions then [$g.token_positions.query_start,$g.token_positions.query_end+1]
  elif $g.kind=="positive" then lexical($g;$g.role)
  elif $g.kind=="absent" then lexical($g;$g.absent_name)
  else [null,null] end;
def excluded($g):
  if $g|has("excluded_start") then [$g.excluded_start,$g.excluded_end]
  elif $g.token_positions and $g.token_positions.query_other_start!=null then
    [$g.token_positions.query_other_start,$g.token_positions.query_other_end+1]
  elif $g.excluded_role then lexical($g;$g.excluded_role)
  else [null,null] end;
def metrics:
  {rows:length,
   requested:map(select(.q[0]!=null))|length,
   requested_exact:map(select(.q[0]!=null and .q==.pick))|length,
   excluded:map(select(.e[0]!=null))|length,
   excluded_exact:map(select(.e[0]!=null and .e==.exclude))|length,
   whole_role:map(select(.q==.pick and .e==.exclude))|length,
   excluded_as_requested:map(select(.e[0]!=null and .pick==.e))|length,
   requested_as_excluded:map(select(.q[0]!=null and .exclude==.q))|length,
   requested_null_low:map(select(.q[0]!=null and .pick[0]==null and .low_evidence))|length,
   requested_null_tie:map(select(.q[0]!=null and .pick[0]==null and .tied))|length,
   no_request:map(select(.q[0]==null))|length,
   correct_abstain:map(select(.q[0]==null and .pick[0]==null))|length,
   asked_absent:map(select(.kind=="absent"))|length,
   asked_absent_correct:map(select(.kind=="absent" and .q==.pick))|length,
   requested_start_right_end_wrong:map(select(.q[0]!=null and .q[0]==.pick[0] and .q[1]!=.pick[1]))|length,
   wrong_different_word:map(select(.q[0]!=null and .pick[0]!=null and .pick[0]!=.q[0] and .pick[0]!=.e[0]))|length,
   requested_missed:map(select(.q[0]!=null and .pick[0]==null))|length,
   pair_requested:(map(select(.q[0]!=null))|group_by(.pair)|map(select(length==2))|length),
   pair_requested_both_exact:(map(select(.q[0]!=null))|group_by(.pair)|map(select(length==2 and all(.[];.q==.pick)))|length),
   pair_both_roles_exact:(map(select(.q[0]!=null))|group_by(.pair)|map(select(length==2 and all(.[];.q==.pick and .e==.exclude)))|length),
   old_requested_exact:map(select(.q[0]!=null and .old_start==.q[0]))|length,
   prior_requested_exact:map(select(.q[0]!=null and .prior_start==.q[0]))|length,
   repaired_vs_prior:map(select(.q[0]!=null and .prior_start!=null and .prior_start!=.q[0] and .pick==.q))|length,
   broken_vs_prior:map(select(.q[0]!=null and .prior_start==.q[0] and .pick!=.q))|length};
def category($key): group_by(.[$key])|map({key:.[0][$key],value:metrics})|from_entries;
$g[0] as $grades | $a[0].rows as $rows | ($old[0].rows // []) as $older |
if ($grades|length)!=($rows|length) then error("grade row count") else . end |
[range($grades|length) as $i | $grades[$i] as $g | $rows[$i] as $r |
  {id:$g.id,pair:$g.pair,kind:($g.kind // "positive"),case:($g.case // "historical"),
   construction:($g.construction // (if $g.query_form!=null then ["regarding","excluding-after","set-aside"][$g.query_form] else "historical" end)),
   direction:(if $g.asked_role!=null then ($g.asked_role|tostring) elif $g.construction then ($g.id|split("-")|last) else "historical" end),
   prompt:$g.prompt,q:requested($g),e:excluded($g),
   pick:[$r.query_start,$r.query_end],exclude:[$r.excluded_start,$r.excluded_end],
   old_start:$older[$i][0],prior_start:$older[$i][4],words:$r.words,
   low_evidence:($r.requested_score<12),tied:(($r.requested_score==$r.requested_runner_up) and ($r.requested_score>=12)),
   requested_text:(if $r.query_start==null then null else pieces($g)[$r.query_start:$r.query_end]|join("") end)}
] as $all |
{overall:($all|metrics),by_construction:($all|category("construction")),by_case:($all|category("case")),by_kind:($all|category("kind")),
 by_direction:($all|category("direction")),
 by_construction_direction:($all|map(.+{k:(.construction+":"+.direction)})|category("k")),
 by_construction_case:($all|map(.+{k:(.construction+":"+.case)})|category("k")),
 examples:($all|group_by([.construction,.case,.kind,(.q==.pick),(.e==.exclude),.low_evidence,.tied])|map(.[0]))}
