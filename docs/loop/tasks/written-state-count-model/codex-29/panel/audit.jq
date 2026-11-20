# Run with --slurpfile gold P.gold.jsonl --slurpfile blind P.blind.jsonl
# --slurpfile person P.person.jsonl --arg panel held|role
def by_row: map({key:(.row|tostring),value:.})|from_entries;
def slice($ids;$range): ($ids[$range[0]:$range[1]]|tojson);
($blind|by_row) as $ids
| ($person|by_row) as $roles
| if ($gold|length)!=($ids|length) or ($gold|length)!=($roles|length)
    or ($gold|group_by(.row)|any(length!=1)) then error("duplicate or missing row") else . end
| [$gold[] | . as $g | $ids[$g.row|tostring].ids as $t | $roles[$g.row|tostring] as $r
  | if $g.role_targets.requested!=$r.requested or $g.role_targets.excluded!=$r.excluded
      then error("role endpoint readback differs at \($g.row)") else . end
  | (slice($t;$r.requested)) as $name
  | if any($g.selected[] as $i|$g.assertions[$i].person; slice($t;.)!=$name)
      then error("requested role selected a different person") else . end
  | if $g.role_targets.excluded!=null and
       ([$g.assertions[].person|select(slice($t;.)==slice($t;$g.role_targets.excluded))]|length)!=1
      then error("excluded query does not bind a unique original factual name") else . end
  | if $g.role_targets.unrequested!=null and
       ([$g.assertions[].person|select(slice($t;.)==slice($t;$g.role_targets.unrequested))]|length)!=1
      then error("unrequested query does not bind a unique original factual name") else . end
  | if $g.matched_pair!=null and
       ($t[$g.local_pair_tape.start:]!=$g.local_pair_tape.ids or
        $g.local_pair_tape.gold_offset!=$g.gold_source-$g.local_pair_tape.start)
      then error("local token suffix or source offset differs") else . end
  | if $g.gold_source!=null and $g.expected_tape!=($t[$g.gold_source:$g.fence]+[1])
      then error("gold tape is not verbatim through EOS") else . end
  | .] as $rows
| if $panel=="held" then
    if ($rows|length)!=31 then error("wrong source roster") else . end
    | {rows:31,primary:($rows|map(select(.stratum=="matched"))|group_by(.family)|map(
        if ((map([.connector_arm,.object_arm])|sort)!=[[0,0],[0,1],[1,0],[1,1]]) then error("incomplete core crossing")
        elif any(.[];.connector_arm==1 and (.wrong_source>=.gold_source)) then error("long row lacks earlier rival")
        else {family:.[0].family,connector_pieces:map(.gold_source-.assertion_head[1])} end)),
       contexts:($rows|map(select(.matched_pair!=null))|group_by(.matched_pair)|map(
        if length!=2 or .[0].local_pair_tape.ids!=.[1].local_pair_tape.ids or .[0].expected_tape==.[1].expected_tape
        then error("invalid context-only pair")
        else {family:.[0].matched_pair,rows:map(.row),local_gold:map(.local_pair_tape.gold_offset)} end))}
  elif $panel=="role" then
    if ($rows|length)!=18 then error("wrong role roster") else . end
    | {rows:18,primary:($rows|map(select(.stratum=="role"))|group_by(.family)|map(
        if ((map(.role_factors)|sort)!=[[0,0,0],[0,0,1],[0,1,0],[0,1,1],[1,0,0],[1,0,1],[1,1,0],[1,1,1]])
        then error("incomplete role crossing")
        else {family:.[0].family,unrequested:([.[]|select(.role_targets.unrequested!=null)]|length)} end)),
       direct:($rows|map(select(.stratum=="role_direct")|.row))}
  else error("invalid panel") end
