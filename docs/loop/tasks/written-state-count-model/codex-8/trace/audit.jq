# Grade sealed selector decisions after blind prediction. Rank the original and
# filtered tuple populations independently from their device-computed site scores.
def require($x;$why): if $x then . else error($why) end;
def inspan($i;$l;$r): $i!=null and $l!=null and $i>=$l and $i<=$r;
def bounds($ids;$cl;$i):
  if $cl[$ids[$i]]==0 then [$i,$i]
  else [($i|until(.==0 or $cl[$ids[.]]!=2 or $cl[$ids[.-1]]==0;.-1)),
        ($i|until(.+1>=($ids|length) or $cl[$ids[.+1]]!=2;.+1))] end;
def kind($i;$p):
  if inspan($i;$p.query_start;$p.query_end) then "asked_query"
  elif inspan($i;$p.fact_start;$p.fact_end) then "asked_fact"
  elif inspan($i;$p.query_other_start;$p.query_other_end) then "excluded_query"
  elif inspan($i;$p.other_fact_start;$p.other_fact_end) then "other_fact"
  elif $i==null then "absent" else "outside_entities" end;
def first_loss($a;$g):
  $g.token_positions as $p | $a.frozen as $s |
  if $s.query==null then "no_anchor_pair"
  elif (inspan($s.query;$p.query_start;$p.query_end) and inspan($s.fact;$p.fact_start;$p.fact_end)
    and $s.query-$p.query_start==$s.fact-$p.fact_start)|not then "joint_anchor_ranking"
  elif [$s.query_start,$s.query_end,$s.fact_start,$s.fact_end]!=[$p.query_start,$p.query_end,$p.fact_start,$p.fact_end]
    then "span_expansion"
  elif $s.source==null then "span_source"
  elif $a.word_start==null then "word_start"
  elif $a.continuation.source!=$g.asked_start then "value_address" else "requested_source" end;
def same_word($p;$fl;$fr;$ql;$qr):
  $fr-$fl==$qr-$ql and all(range(0;$fr-$fl+1);$p[$fl+.] == $p[$ql+.]);
def terminal($a;$p;$bounds;$lens):
  $a.frozen as $s | $s.query as $q | $s.fact as $f |
  if $q==null then "no_anchor_pair"
  else $bounds[$q] as [$ql,$qr] | $bounds[$f] as [$fl,$fr] |
    if $fr>=$ql then "span_overlap" elif ($f-$fl)!=($q-$ql) then "anchor_offset"
    elif (same_word($p;$fl;$fr;$ql;$qr)|not) then "unequal_whole_words"
    elif all($lens[]; $fr+1+. >= $ql) then "no_eligible_length"
    elif $a.word_start==null then "no_word_start"
    else "source_present" end end;
def tuples($p;$sites;$b;$g):
  [range(0;($p|length)) as $q | range(0;($q-1)) as $f |
    select($p[$q]==$p[$f]) |
    $b[$q] as [$ql,$qr] | $b[$f] as [$fl,$fr] |
    {key:[($sites[$q][0]+$sites[$f][1]),$q,$f],
     exact_word:($fr<$ql and $q-$ql==$f-$fl and same_word($p;$fl;$fr;$ql;$qr)),
     asked:(inspan($q;$g.query_start;$g.query_end) and inspan($f;$g.fact_start;$g.fact_end)
            and $q-$g.query_start==$f-$g.fact_start)}] | sort_by(.key);
def site($key;$s;$b;$p):
  if $key==null then null else $key as [$c,$q,$f] |
    {cost:$c,q_cost:$s[$q][0],f_cost:$s[$f][1],q:$q,f:$f,anchor_id:$p[$q],
     q_word:($b[$q] as [$l,$r]|$p[$l:($r+1)]),f_word:($b[$f] as [$l,$r]|$p[$l:($r+1)])} end;
def summarise($r;$g;$bank):
  $g.prefix as $p | $r[0].site_scores as $s | $g.token_positions as $pos |
  [range(0;($p|length))|bounds($p;$bank.classes;.)] as $b |
  tuples($p;$s;$b;$pos) as $all | ($all|map(select(.exact_word))) as $legal |
  ($all|map(select(.asked))|.[0].key) as $asked | $all[0].key as $base | $legal[0].key as $repair |
  . | require(($s|length)==($p|length) and $r[1].site_scores==$s and
      $r[0].prefix_sha256==$r[1].prefix_sha256 and
      ($base[1:]//[])==(if $r[0].frozen.query==null then [] else [$r[0].frozen.query,$r[0].frozen.fact] end) and
      ($repair[1:]//[])==(if $r[1].frozen.query==null then [] else [$r[1].frozen.query,$r[1].frozen.fact] end);
      "device candidates disagree with tuple order: "+$g.id) |
  ([ $r[] | (.frozen.query as $q | .frozen.fact as $f |
    require(($q==null and $f==null) or ([$b[$q][], $b[$f][]]==[.frozen.query_start,.frozen.query_end,.frozen.fact_start,.frozen.fact_end]);"bounds: "+$g.id)) |
    terminal(.;$p;$b;($bank.relations|map(.[0])|unique)) as $gate |
    require(($gate=="source_present") == (.continuation.source!=null);"gate: "+$g.id) |
    {first_loss:first_loss(.;$g),gate:$gate,q_occurrence:kind(.frozen.query;$pos),f_occurrence:kind(.frozen.fact;$pos),
     span_source:.frozen.source,word_start,source:.continuation.source,hit:(.continuation.source==$g.asked_start)}
  ]) as [$old,$new] |
  {id:$g.id,prefix_hash:$r[0].prefix_sha256,pair:$g.pair,asked_role:$g.role,query_position:$g.factors.requested_position,
   exclusion:$g.factors.exclusion,lexical_set:$g.factors.lexical_set,connector:$g.factors.asked_connector_surface,
   source:$g.asked_start,asked_query:[$pos.query_start,$pos.query_end],asked_fact:[$pos.fact_start,$pos.fact_end],
   old:$old,new:$new,baseline_pick:site($base;$s;$b;$p),repair_pick:site($repair;$s;$b;$p),
   asked_pick:site($asked;$s;$b;$p),asked_rank:(if $asked==null then null else ($all|map(.key)|index([$asked]) +1) end),
   asked_compatible_rank:(if any($legal[];.asked) then ($legal|map(.key)|index([$asked])+1) else null end),
   asked_survived:any($legal[];.asked),candidate_count:($all|length),compatible_count:($legal|length),
   anchor_reason:(if $base==null or $asked==null then "missing" elif $base==$asked then "requested"
    elif $base[0]==$asked[0] then "cost_tie" else "strictly_lower_cost" end)};
. as $base | $repair[0] as $fix | $bank[0].bank as $b | $grade[0] as $g |
require((.rows|length)==($g|length) and ($fix.rows|length)==($g|length) and (.artifact_sha256==$fix.artifact_sha256);"population/model") |
[range(0;($g|length)) as $i | summarise([$base.rows[$i],$fix.rows[$i]];$g[$i];$b)]
