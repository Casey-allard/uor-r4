# Run on one complete held writer grade. All denominators count prompts.
def n(f): map(select(f)) | length;
def count_hits:
  { rows: length,
    person_hit: n(.person_hit),
    head_set_hit: n(.head_set_hit),
    supported: n(.unsupported | not),
    source_hit: n(.source_hit),
    unsupported: n(.unsupported),
    unsupported_no_source: n(.unsupported and .source == null),
    unsupported_false_write: n(.false_write),
    tape_scored: n(.exact != null),
    tape_exact: n(.exact == true),
    supported_tape_exact: n((.unsupported | not) and .exact == true),
    literal_refusal_scored: n(.unsupported and .exact != null),
    literal_refusal_exact: n(.unsupported and .exact == true),
    ended_eos: n(.end_eos),
    first_wrong: [ .[] | select(.first_wrong != null) | {row, first_wrong} ] };
def first_failure:
  if (.lane=="disabled" or .lane=="forced") then null
  elif .person_hit|not then "person"
  elif .head_set_hit|not then "assertion_set"
  elif (.unsupported and .source!=null) or ((.unsupported|not) and (.source_hit|not)) then "source"
  elif .exact==false then "tape"
  else null end;
def first_failures:
  map(first_failure) | group_by(.) |
  map({gate:(.[0] // "none"), rows:length});
def lexical:
  if .stratum != "transfer" then null
  elif .factors[0] == 3 and .factors[1] == 0 then {cue:"colors/paints", tier:"weaker"}
  elif .factors[0] == 3 and .factors[1] == 1 then {cue:"mends/repairs", tier:"stronger"}
  elif .factors[0] == 4 and .factors[1] == 0 then {cue:"gathers/collects", tier:"stronger"}
  elif .factors[0] == 4 and .factors[1] == 1 then {cue:"indexes/catalogs", tier:"weaker"}
  else error("unknown transfer cell") end;
def cue_family:
  if .factors == null then null
  elif .factors[0] == 1 or .factors[0] == 3 then "Nina"
  elif .factors[0] == 2 or .factors[0] == 4 then "Owen"
  else error("unknown held cell") end;
def within_pair:
  if length != 2 or ((map(.factors[1])|sort) != [0,1])
     or .[0].expected_heads == .[1].expected_heads
     or .[0].expected_tape == .[1].expected_tape then
    error("invalid matched requested-relation pair")
  else
    { rows:map(.row),
      order:.[0].factors[2], exchange:.[0].factors[3], instructed:.[0].instructed,
      person_both:all(.person_hit),
      head_flips:(.[0].heads != .[1].heads),
      head_correct_both:all(.head_set_hit),
      source_correct_both:all(.source_hit),
      reply_correct_both:all(.exact == true) }
  end;
def paired:
  group_by([.factors[2],.factors[3],.instructed]) |
  map(sort_by(.factors[1]) | within_pair) as $pairs |
  {pairs:($pairs|length),
   person_both:($pairs|n(.person_both)),
   head_flips:($pairs|n(.head_flips)),
   head_correct_both:($pairs|n(.head_correct_both)),
   source_correct_both:($pairs|n(.source_correct_both)),
   reply_correct_both:($pairs|n(.reply_correct_both)),
   details:$pairs};
def compare($lane; $arm; $base_lane; $prior):
  (map(select(.lane==$base_lane and .arm==$prior)) | INDEX(.row|tostring)) as $base |
  map(select(.lane==$lane and .arm==$arm)) |
  map(. as $current | ($base[.row|tostring] // error("missing contrast match")) |
    {row, stratum, instructed, headers:[.head_set_hit,$current.head_set_hit],
     replies:[.exact,$current.exact], false_writes:[.false_write,$current.false_write]}) as $rows |
  {lane:$lane, candidate:$arm, baseline_lane:$base_lane, baseline:$prior,
   strata:($rows|group_by(.stratum)|map({stratum:.[0].stratum, rows:length,
     head_gains:[.[]|select(.headers==[false,true])|.row],
     head_regressions:[.[]|select(.headers==[true,false])|.row],
     reply_gains:[.[]|select(.replies==[false,true])|.row],
     reply_regressions:[.[]|select(.replies==[true,false])|.row],
     new_unsupported_writes:[.[]|select(.false_writes==[false,true])|.row],
     prevented_unsupported_writes:[.[]|select(.false_writes==[true,false])|.row]}))};
def contrast($lane; $arm; $prior): compare($lane; $arm; $lane; $prior);
def key: [.lane,.arm];
def validate:
  group_by(key) as $g |
  ($g | map(.[0]|key) | map(select(.[0]!="forced"))) as $keys |
  (map(.row)|unique) as $rows |
  (map(select(.lane=="ordinary" and .arm=="unfiltered" and (.unsupported|not))|.row)|sort) as $supported |
  if ($rows|length)!=92 or
     $keys != [["disabled","disabled"],["oracle","assertions"],["ordinary","exact"],["ordinary","geometry"],
       ["ordinary","unfiltered"],["person","exact"],["person","geometry"],["person","unfiltered"],
       ["person_relation","exact"],["person_relation","geometry"]] or
     ($g | any((map(.row)|length)!=(map(.row)|unique|length))) or
     ($g | any(.[0].lane!="forced" and (map(.row)|sort)!=$rows))
    then error("missing or duplicate grade rows or arms")
    elif (map(select(.lane=="forced" and .arm=="value")|.row)|sort)!=$supported or
         (map(select(.lane=="forced" and .arm=="opposing")|.row)|sort)!=$supported or
         any(.[]; .lane=="forced" and ((.arm=="value") != (.exact==true)))
    then error("missing or failed forced-address controls")
    else . end;
validate as $rows |
{ held_rows:($rows|map(.row)|unique|length),
  arms:($rows|group_by(key)|map(
     . as $arm |
     {lane:.[0].lane, arm:.[0].arm, all:count_hits,
      first_failures:first_failures,
      by_stratum:(group_by([.stratum,.instructed]) |
        map({stratum:.[0].stratum,instructed:.[0].instructed} + count_hits)),
      matched_pairs:($arm|map(select(.factors!=null)) |
        group_by([.stratum,cue_family]) |
        map({stratum:.[0].stratum,family:(.[0]|cue_family)} + paired)),
      transfer_by_cue:($arm|map(select(.stratum=="transfer")) |
        group_by(lexical) | map({lexical:(.[0]|lexical)} + count_hits)),
      transfer_by_tier:($arm|map(select(.stratum=="transfer")) |
        group_by(lexical.tier) | map({tier:(.[0]|lexical.tier)} + count_hits))})),
  contrasts: [
    $rows|contrast("ordinary";"exact";"unfiltered"),
    contrast("ordinary";"geometry";"exact"),
    contrast("person";"exact";"unfiltered"),
    contrast("person";"geometry";"exact"),
    contrast("person_relation";"geometry";"exact")
  ],
  intervention_contrasts: [
    $rows|compare("person";"unfiltered";"ordinary";"unfiltered"),
    compare("person";"exact";"ordinary";"exact"),
    compare("person";"geometry";"ordinary";"geometry"),
    compare("person_relation";"exact";"person";"exact"),
    compare("person_relation";"geometry";"person";"geometry")
  ] }
