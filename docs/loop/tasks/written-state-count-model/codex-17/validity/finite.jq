# Entirely planted 94+42 mathematical fixture. No project artifact or prompt.
def mock($i;$hold):
  (if $hold then $i<36 else ($i<36 or $i>=42) end) as $pos |
  ($i%3) as $k | ([0,10,11,12,13,14,15,(16+$i)]) as $t |
  ("case-\($i)") as $id |
  {g:{id:$id,prefix_sha256:$hashes[0][$i],family:"dummy",requested:[6,7],stratum:(if $pos then "supported" else (["conflict","unmentioned","unspecified"][$i%3]) end),
      matched_triple:(if $hold and $i<24 then ($i/3|floor|tostring) else null end),
      facts:[{name_span:[0,1],delimiter:[5,16]}],gold_assertion_index:(if $pos then 0 else null end),
      gold_start:(if $pos then 2+$k else null end),expected:(if $pos then ($t[2+$k:5]+[1]) else null end)},
   b:{prefix:$t},
   r:{ordinal:$i,prefix_sha256:$hashes[0][$i],requested_span:[6,7],original_tape_len:8,
      assertions:(if $pos then [{fact_span:[0,1],fence:5,legacy_candidates:[range(3) as $j |
        {original_start:(2+$j),first_word_end:(3+$j),connector_slots:(1+$j),word_slots:1,
         slot_classes:([range(2+$j)|1]),writer_reachable:true}]}] else [] end)}};
def cap($rows):{schema:"uor-r4.legacy-assertion-candidates/1",arm:"requested-diagnostic",rows:($rows|map(.r))};
[range(94)|mock(.;false)] as $f | [range(42)|mock(.;true)] as $h |
{fc:cap($f),hc:cap($h),fg:($f[:42]|map(.g)),extra:($f[42:]|map(.g)),
 hg:($h|map(.g)),fb:($f|map(.b)),hb:($h|map(.b))}
