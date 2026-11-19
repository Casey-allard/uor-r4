def summary:
  group_by(.stratum) | map({key:.[0].stratum,value:{
    rows:length,endpoint_exact:map(select(.endpoint_exact))|length,requested:map(select(.requested!=null))|length,
    source:map(select(.source!=null))|length,through_eos:map(select(.exact==true))|length,
    terminal_eos:map(select(.terminal_eos))|length,
    first_wrong:(map(select(.expected!=null))|group_by(.first_wrong)|map({key:(.[0].first_wrong|if .==null then "exact" else tostring end),value:length})|from_entries)
  }})|from_entries;
def pairwins: map(select(.stratum=="supported"))|group_by(.pair)|map(select(length==2 and all(.[];.exact)))|length;
def term: if .exact then "exact" else (.first_wrong|tostring) end;
def transitions($a;$b): ([$a[]|{key:.id,value:.}]|from_entries) as $index |
    [$b[]|select(.stratum=="supported")|. as $r| {from:($index[$r.id]|term),to:($r|term)}]
    | group_by([.from,.to])|map(.[0]+{n:length});
($enabled[0].rows) as $p | ($disabled[0].rows) as $n |
($oracle[0].rows) as $o | ($wrong[0].rows) as $w |
($literal[0].rows) as $l | ($current[0].rows) as $c | ($word[0].rows) as $s |
([$l[]|{key:.id,value:.}]|from_entries) as $lh |
([$enabled[0].rows[]|{key:.id,value:.}]|from_entries) as $public |
(if ($o|length)!=256 or ($l|length)!=24 or ($p|length)!=416 then error("wrong row counts") else . end) |
{seed:$seed,public:($p|summary),no_read:($n|summary),correct_request:($o|summary),wrong_request:($w|summary),
  public_pair_wins:($p|pairwins), request_oracle_pair_wins:($o|pairwins),wrong_oracle_pair_wins:($w|pairwins),
  public_literal_source_correct:([$p[]|select(.stratum=="supported" and .source_matches_requested_value)]|length),
  public_to_correct_request:transitions($p;$o), public_to_wrong_request:transitions($p;$w),
  disabled_to_public:transitions($n;$p), oracle_misses_to_literal:transitions($o;$l),
  oracle_misses_to_current:transitions($o;$c),oracle_misses_to_word_start:transitions($o;$s),
  per_family:([$o[]|. as $r|{family,correct_request:.exact,public:$public[$r.id].exact,after_literal:(if .exact then true else $lh[$r.id].exact end)}]
       |group_by(.family)|map({family:.[0].family,n:length,public:map(select(.public))|length,correct_request:map(select(.correct_request))|length,
        with_miss_only_onset:map(select(.after_literal))|length})),
  examples:([$l[0],$o[]|select(.exact==false)][0:1]|map(. as $r|{id:.id,text:$blind[0][.index].text,requested:$labels[0][.index].requested_name,expected:$labels[0][.index].answer_text,
     client:($public[$r.id]|{generated,source,text,first_wrong}),correct_endpoint:([$o[]|select(.id==$r.id)|{generated,source,first_wrong}][0]),forced_literal:($lh[$r.id]|{generated,source,first_wrong})})),
  unsupported_copy_ids:[$p[]|select(.expected==null and .source!=null)|.id],
  control_texts:([$p[]|select(.expected==null)|{stratum,text,acceptable_prose,terminal_eos}]
    |group_by([.stratum,.text,.acceptable_prose,.terminal_eos])|map(.[0]+{n:length}))
}
