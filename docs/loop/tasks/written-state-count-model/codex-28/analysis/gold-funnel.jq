# jq --slurpfile gold PANEL/held.gold.jsonl --arg arm ARM -f gold-funnel.jq STAGE/visits.json
def missing: 4294967295;
def reason:
  if .[4] == 1 then "copy_cap"
  elif .[4] == 2 then
    if .[5] == missing and .[6] == missing then "missing_both"
    elif .[5] == missing then "missing_positive"
    elif .[6] == missing then "missing_negative"
    elif .[7] == 65535 then "unfitted_radius"
    else "positive_outside_radius"
    end
  elif .[8] == 0 or .[8] >= 2147483648 then "nonpositive_margin"
  else "positive_margin"
  end;
def count_by(f): group_by(f) | map({key:(.[0]|f),value:length}) | from_entries;
def signed: if . >= 2147483648 then . - 4294967296 else . end;
map(select(.lane=="assertions")) as $visits
| [$gold[] | select(.stratum=="matched") | . as $key
  | [$visits[] | select(.row == $key.row and .stage.head == $key.assertion_head)] as $matches
  | if $matches|length!=1 then error("wrong oracle visit count at ($key.row)") else $matches[0].stage end
  | . as $stage
  | [.candidate_words[] | select(.[0] == $key.gold_source)] as $gold_words
  | if $gold_words|length>1 then error("duplicate gold candidate at ($key.row)") else . end
  | ($gold_words[0] // null) as $candidate
  | [.candidate_words[] | select(.[0] != $key.gold_source and .[4] == 2) | {source:.[0], reason:reason, words:.}] as $rivals
  | ($candidate != null and $candidate[4] >= 2 and $candidate[5] != missing and $candidate[6] != missing and $candidate[7] != 65535) as $supported
  | ($candidate != null and ($candidate|reason) == "positive_margin") as $positive
  | ($candidate != null and $candidate[4]==3) as $within_radius
  | (if $positive then
      any($stage.candidate_words[];
        .[0] != $key.gold_source and .[4] == 3
        and (.[8]|signed) * $candidate[9] >= ($candidate[8]|signed) * .[9])
    else false end) as $seen_rival_matches_or_exceeds
  | (if $candidate == null then "ineligible"
     elif $positive then
       if $stage.onset.status==2 then
         if $rivals|length==0 then error("unseen veto without rival")
         elif $seen_rival_matches_or_exceeds then "positive_veto_and_seen_rival_matches_or_exceeds"
         else "positive_vetoed_only_by_unseen_rival"
         end
       elif $stage.onset.status==3 then "positive_tied"
       elif $stage.onset.source == $key.gold_source then "positive_gold_selected"
       elif $stage.onset.status==0 then "positive_outscored"
       else error("unexpected status for positive candidate")
       end
     else ($candidate|reason)
     end) as $category
  | {row:$key.row,family:$key.family,connector_arm:$key.connector_arm,
      object_arm:$key.object_arm,head:$key.assertion_head,gold_source:$key.gold_source,
      candidate:$candidate,eligible:($candidate!=null),supported:$supported,within_radius:$within_radius,
      positive:$positive,category:$category,rivals:$rivals,onset:$stage.onset,completion:$stage.completion}
] as $rows
| {arm:$arm,matched:($rows|length),groups:($rows|group_by(.connector_arm)|map({
    connector_arm:.[0].connector_arm,n:length,
    eligible:(map(select(.eligible))|length),
    supported:(map(select(.supported))|length),
    within_radius:(map(select(.within_radius))|length),
    positive:(map(select(.positive))|length),
    categories:(count_by(.category)),
    sole_veto_rival_reasons:([.[]|select(.category=="positive_vetoed_only_by_unseen_rival")|.rivals[]]|count_by(.reason))
  })),rows:$rows}
