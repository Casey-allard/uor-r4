# Read the sealed per-seed D17-format grade summary. Emit paired ordinal-level results.
def require($condition; $message): if $condition then . else error($message) end;
def arm_name: sub("^learned-normal-"; "");
def slim: {id,index,matched_triple,gold_source,gold_requested,selected_source,
           requested,terminal_eos,exact,first_error,earliest_stage};
def check_row:
    require((.exact|type)=="boolean" and (.terminal_eos|type)=="boolean"
        and (.index|type)=="number"; "invalid observed row")
    | require(if .exact then .first_error==null and .terminal_eos
              else (.first_error.index|type)=="number" and .first_error.index>=0 end;
              "exact / first mismatch / EOS disagree");
def movement($a;$b):
    if $a.exact and $b.exact then "both_exact"
    elif $a.exact then "regressed"
    elif $b.exact then "recovered"
    elif $a.first_error.index < $b.first_error.index then "later"
    elif $a.first_error.index > $b.first_error.index then "earlier"
    else "same_index" end;
def shifts($rows;$a;$b):
    [$rows[] | {ordinal:.index,
      movement:movement(.outputs[$a];.outputs[$b])}] |
    group_by(.movement) | map({key:.[0].movement,value:map(.ordinal)}) | from_entries;
def pair($r;$a;$b):
  {before:$a,after:$b,movements:shifts($r;$a;$b),
   changed_source:[$r[]|select(.outputs[$a].selected_source != .outputs[$b].selected_source)|.index],
   changed_first_emitted_id_at_same_error:[$r[] | .outputs[$a].first_error as $left |
     .outputs[$b].first_error as $right |
     select($left!=null and $right!=null and $left.index==$right.index
       and $left.emitted_id!=$right.emitted_id)|.index]};
def exact($r;$a): [$r[] | select(.outputs[$a].exact)] | length;
def scope($r):
   {rows:($r|length),
    arms:(["ordered","ranked","veto","both","position"]|map(. as $a |
       {key:$a,value:{exact:exact($r;$a),
         eos:([$r[]|select(.outputs[$a].terminal_eos)]|length),
         errors:([$r[]|.outputs[$a]|select(.exact|not)|.first_error.index]|group_by(.)|
                map({index:.[0],count:length})),
         stages:([$r[]|.outputs[$a].earliest_stage]|group_by(.)|map({stage:.[0],count:length}))}})|from_entries),
    paired:([["ordered","ranked"],["ordered","veto"],["ordered","both"],["veto","both"],
             ["ranked","both"],["ranked","veto"],["ordered","position"],["position","both"]]
            |map(pair($r;.[0];.[1]))),
    only_combination_recovers:[$r[]|select(.outputs.both.exact and
      ([.outputs.ordered.exact,.outputs.ranked.exact,.outputs.veto.exact]|any|not))|.index],
    interaction_exact_count:(exact($r;"both")-exact($r;"veto")-exact($r;"ranked")+exact($r;"ordered"))};
["both","ordered","position","ranked","veto"] as $names |
[.arms[]| .arm as $name | ($name|arm_name) as $a | select($names|index($a)) |
  {key:$a,value:(.positive_rows | map(slim|check_row) | sort_by(.index))}] as $list |
require(($list|map(.key)|sort)==$names; "need exactly one observed normal arm of each name") |
($list|from_entries) as $arms |
require(all($arms[]; length==36 and (map(.index)|unique|length)==36 and (map(.id)|unique|length)==36);
        "supported panel cardinality or unique identity") |
[$arms.ordered|to_entries[]|.key as $i|.value as $ref |
 {index:$ref.index,id:$ref.id,matched_triple:$ref.matched_triple,gold_source:$ref.gold_source,
  gold_requested:$ref.gold_requested, outputs:($names|map(. as $a | $arms[$a][$i] |
    require([.id,.index,.matched_triple,.gold_source,.gold_requested] ==
            [$ref.id,$ref.index,$ref.matched_triple,$ref.gold_source,$ref.gold_requested]; "arms not paired") |
    {key:$a,value:del(.id,.index,.matched_triple,.gold_source,.gold_requested)})|from_entries)}] as $r |
($r|map(select(.matched_triple!=null))) as $m |
require(($m|group_by(.matched_triple)|map(length))==[3,3,3,3,3,3,3,3];"eight matched triples required") |
{schema:"uor-r4.fixed-writer-factorial/1", all_supported:scope($r),matched:scope($m),rows:$r}
