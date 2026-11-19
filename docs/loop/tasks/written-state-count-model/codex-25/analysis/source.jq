def require($condition; $message):
  if $condition then . else error($message) end;

def fields: split("\n") | map(select(length > 0) | [scan("[^[:space:]]+")]);
def numeric: map(tonumber);
def group_rows: group_by(.[0]) | INDEX(.[0][0]);
def contexts($raw): $raw | fields | map(numeric) | INDEX(.[0]);
def packet($raw): $raw | fields | .[1:] | map(numeric) | group_rows;
def trace($raw): $raw | fields | group_rows;
def donor($row): if $row >= 201 and $row <= 208 then ($row + ($row % 2 * 2) - 1) else $row end;
def anchor: .fact_and_query[0][1] - 1;
def offset($anchor): if . == null then null else . - $anchor end;

($held_key[0] + $exposed_key[0] | INDEX(.row)) as $key
| (packet($held_rows)) as $rows
| (packet($swapped_rows)) as $swapped
| (trace($held_trace) + trace($exposed_trace)) as $trace
| (trace($swapped_trace)) as $swap_trace
| (trace($held_results) + trace($exposed_results)) as $results
| (trace($swapped_results)) as $swap_results
| [contexts($train_context), contexts($held_context), contexts($exposed_context)] as $contexts
| [201,203,205,207] as $pair_starts
| {
    pre_fact_reads: (
      ["train", "held", "exposed"] as $splits
      | [range(0; 3) as $i
        | $contexts[$i] | [.[] | .[1:]]
        | {split:$splits[$i], rows:length, distinct:(unique|length)}]
    ),
    distinct_pooled_pre_fact_reads: ([$contexts[][] | .[1:]] | unique | length),
    common_read: ($contexts[0]["101"][1:]),
    pairs: [$pair_starts[] as $first
      | ($first+1) as $second
      | {
        rows:[$first,$second],
        same_local_projection: (
          [$first,$second] | map(. as $row | $rows[$row|tostring] | map(.[5:37])) | .[0] == .[1]
        )
      }],
    rows: [$key[] | . as $k | ($k.row|tostring) as $id
      | (if .fact_and_query then anchor else 0 end) as $anchor
      | {
        row, reading,
        gold:(.gold|offset($anchor)),
        early_late:(.onsets|map(offset($anchor))),
        wide:(.candidates|to_entries|map((.key|tonumber)-$anchor)|sort),
        narrow:(.candidates|to_entries|map(select(.value)|(.key|tonumber)-$anchor)|sort),
        legacy: (
          [.legacy.facts[]?
          | select(.fact_span == $k.fact_and_query[0])
          | .assertion as $a
          | {selected:($a.source|offset($anchor)), legal:[
              range(0;3) as $bit | ([1,2,4][$bit]) as $unit
              | select((($a.value.eligible_mask / $unit | floor) % 2) == 1)
              | select((($a.value.word_start_mask / $unit | floor) % 2) == 1)
              | select(($a.fence - ($anchor+$bit+2)) < 8)
              | $bit+2
            ]}]
        ),
        arms: [$results[$id][] | . as $result | (.[1]) as $arm
          | [$trace[$id][] | select(.[2] == $arm and .[3] == "1")
             | {offset:(.[1]|tonumber|offset($anchor)), positive:(.[4]|tonumber),
                negative:(.[5]|tonumber), margin:(.[6]|tonumber)}] as $scored
          | ($scored | map(.margin) | max) as $max
          | {
            arm:($arm|tonumber), reason:$result[2],
            selected:(if $result[3]=="NA" then null else $result[3]|tonumber|offset($anchor) end),
            max_margin:$max,
            max_ties:([$scored[] | select(.margin == $max)]|length),
            nearest_equal:([$scored[] | select(.positive == .negative)]|length),
            distances:$scored
          }]
      }],
    swaps: [$rows | to_entries[] | .key as $id
      | ($id|tonumber|donor(.)|tostring) as $donor
      | .value as $original
      | $rows[$donor] as $donated
      | $swapped[$id] as $changed
      | ($key[$id].fact_and_query[0][1] // 1) as $end
      | ($key[$donor].fact_and_query[0][1] // 1) as $donor_end
      | {
        row:($id|tonumber), donor:($donor|tonumber),
        exact_packet: ([$original | keys[] as $i | .[$i][0:37]+$donated[$i][37:69]] == $changed),
        donor_coordinates_differ: ($original[0][37:69] != $donated[0][37:69]),
        exact_trace: (
          [$trace[$id] | keys[] as $i | .[$i] as $own
          | if ($own[2]|tonumber) < 2 then $own
            else $trace[$donor][$i] as $other
            | [$id, ($other[1]|tonumber|. + $end - $donor_end|tostring)] + $other[2:] end]
          == $swap_trace[$id]),
        exact_decision: (
          [$results[$id] | keys[] as $i | .[$i] as $own
          | if ($own[1]|tonumber) < 2 then $own
            else $results[$donor][$i] as $other
            | [$id] + $other[1:3] + [
                $other[3] | if . == "NA" then . else tonumber|. + $end - $donor_end|tostring end
              ] end]
          == $swap_results[$id])
      }]
  }
| require(all(.rows[]; (.arms | map(.arm)) == [0,1,2,3]);
          "rows lack four ordered result arms")
| require(all(.rows[]; . as $row | all(.arms[];
    ([.distances[].offset] == (if (.arm % 2 == 0) then $row.narrow else $row.wide end))
    and all(.distances[]; .margin == .negative - .positive)
    and (if .reason == "empty" then .max_margin == null and .selected == null
         elif .reason == "nonpositive" then .max_margin <= 0 and .selected == null
         elif .reason == "tie" then .max_margin > 0 and .max_ties > 1 and .selected == null
         elif .reason == "pick" then .max_margin > 0 and .max_ties == 1 and
              (.selected as $pick | .max_margin as $max | any(.distances[]; .offset == $pick and .margin == $max))
         else false end))); "device trace does not support its decision")
| require(all(.swaps[]; .exact_packet and .exact_trace and .exact_decision);
          "context swap disagrees with the local recipient or aligned contextual donor")
| require(all(.rows[] | select(.wide|length>0); .narrow == .legacy[0].legal);
          "narrow and legacy eligibility differ")
| . as $audit
| .groups = (
    {primary:[201,202,205,206,207,208], secondary:[203,204], refusal:[209],
     empty:[210,211], exposed:[301,302,303,304]} | to_entries
    | map(.key as $group | .value as $ids
        | {group:$group, size:($ids|length),
           arms:[range(0;4) as $arm
            | [$audit.rows[] | select(.row as $row | $ids | index($row))
               | {gold, selected:.arms[$arm].selected, reason:.arms[$arm].reason}] as $group_rows
            | {arm:$arm, exact:([$group_rows[] | select(.gold == .selected)]|length),
               reasons:($group_rows | group_by(.reason)|map({reason:.[0].reason,count:length}))}]})
  )
