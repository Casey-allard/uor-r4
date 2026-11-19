# Slurp results-seedN.json, requested.json, then one packed archive group per
# level/chunk. All archives must pass compare.sh before being opened.
def check($predicate; $message): if $predicate then . else error($message) end;
def one($values; $message):
  ($values | check(length==1; $message))[0];
def seq($end; $len): $end + 1 + $len;
def bit_has($mask; $n):
  (if $n>=1 and $n<=3 then ((($mask / [1,2,4][$n-1]) | floor) % 2)==1
   else false end);
def has_at($mask; $at; $end): bit_has($mask; $at-$end-1);
def chosen_values($value; $continuation):
  {current:$value.current,word_start:$value.word_start,ordered:$value.ordered,
   ordered_and_word_start:$value.both,continuation:$continuation.source};
def gpu_words($b):
  def source: . // 4294967295;
  [$b.fence|source]
  + [($b.value.current|source),($b.value.word_start|source),
     ($b.value.ordered|source),($b.value.both|source),
     $b.value.eligible_mask,$b.value.word_start_mask,$b.value.ordered_mask]
  + [($b.continuation.source|source),$b.continuation.reason,
     $b.continuation.connector_mask,$b.continuation.onset_mask,($b.source|source)];
def alternatives($stage; $values; $gold):
  [$values|to_entries[] | select(.value==$gold) | {stage:$stage,operator:.key}];
def disposition($gold; $word; $first; $eligible; $word_eligible; $found):
  if $found then "explicit_gold_proposal_available"
  elif $eligible|not then "gold_outside_recorded_value_candidates"
  elif $word_eligible|not then "gold_excluded_by_word_start_class"
  elif $gold < $word then "word_start_selected_later_forward_only"
  elif $first == null then "continuation_found_only_connectors"
  elif $first < $gold then "continuation_stopped_at_earlier_nonconnector"
  else "continuation_other" end;
.[0] as $result | .[1] as $requested | .[2:] as $groups
| check($result.schema=="uor-r4.fresh-validity-results/1" and ($groups|length)>=1; "bad source frame")
| [ $result.rows[]
    | select(.semantics=="supported" and .arms.requested.role_exact and (.arms.requested.original_exact|not))
    | . as $row
    | $row.expected_address.start as $gold
    | $row.expected_address.end as $gold_fence
    | $row.expected_role as $q
    | $row.prefix_sha256 as $hash
    | one([$requested.rows[]|select(.index==$row.index)];"missing requested selection") as $request
    | one([
        $groups[] as $group | ($group.endpoint.rows|to_entries[])
        | select(.value.prefix_sha256==$hash and [.value.query_start,.value.query_end]==$q)
        | {group:$group,ordinal:.key}
      ]; "endpoint key ambiguous or missing") as $join
    | $join.group as $g | $join.ordinal as $i
    | $g.sites.sites[$i] as $sites | $g.nomination.rows[$i] as $nom
    | $g.selection.rows[$i] as $selection | $g.chosen.rows[$i] as $actual
    | check($request.prefix_sha256==$hash and $request.paired.role_words[0:2]==$q
       and $request.paired.supporting_facts==1 and $request.paired.complete_values==1
       and $nom.prefix_sha256==$hash and [$nom.nominee,$nom.query_end]==$q
       and $nom.count==1 and ($sites|length)==1
       and $selection.prefix_sha256==$hash and $selection.count==1 and $selection.complete==1
       and ($selection.disagree|not) and $actual.prefix_sha256==$hash
       and $selection.source==$request.paired.choice.source
       and $actual.source==$selection.source and $actual.source==$row.arms.requested.source
       and $actual.generated==$row.arms.requested.generated_ids
       and $row.arms.requested.terminal_eos and $actual.steps[-1].emitted==1
       and $actual.steps[-1].release==1
       and ($row.prefix_original_ids[$gold:$gold_fence]+[1])==$row.expected_ids;
       "requested, ordered group, actual writer or original source changed")
    | [$g.ranks[]
      | check(.chain.bare_sha256==$g.endpoint.bare_sha256
          and .cont.bare_sha256==$g.endpoint.bare_sha256
          and .chain.artifact_sha256==$g.endpoint.artifact_sha256
          and .cont.artifact_sha256==$g.endpoint.artifact_sha256; "rank group identity changed")
      | {rank,unfenced:.chain.rows[$i],fenced_gpu:.cont.rows[$i],writer:.writer.rows[$i],
         gpu_parity_words:.cont.cuda_parity_words}
      | check(.unfenced.prefix_sha256==$hash and .fenced_gpu.prefix_sha256==$hash
          and .writer.prefix_sha256==$hash
          and .fenced_gpu.words==gpu_words(.fenced_gpu.bounded)
          and .gpu_parity_words>0; "rank row mappings or GPU words differ")
      | . + {retained: (.rank<($sites|length))}
      ] as $ranks
    | one([$ranks[]|select(.retained)];"expected one retained assertion") as $r
    | $r.unfenced as $pre | $r.fenced_gpu as $gpu | $gpu.bounded as $bounded
    | $pre.frozen.fact_end as $fact_end
    | ($gold-$fact_end-1) as $gold_len
    | $bounded.value.word_start as $ws
    | $bounded.continuation as $co
    | check(all($ranks[]; if .retained then true else .fenced_gpu.frozen.source==null and
          .fenced_gpu.bounded.source==null and .fenced_gpu.bounded.value.eligible_mask==0 end)
       and $sites[0]==$pre.frozen.fact
       and [$pre.frozen.fact_start,$pre.frozen.fact_end+1]==$row.expected_address.factual_name_span
       and [$pre.frozen.query_start,$pre.frozen.query_end+1]==$q
       and $pre.frozen==$gpu.frozen and $gpu.bounded==$request.assertions[0].assertion
       and $pre.frozen.source==$pre.value.current
       and $gpu.source==$bounded.source and $bounded.source==$selection.source
       and $bounded.fence==$gold_fence and $actual.steps[-1].before[2]==$gold_fence
       and $r.writer.source==$selection.source and $r.writer.generated==$actual.generated;
       "factual fence, alternate rank or writer parity broken")
    | chosen_values($pre.value;$pre.continuation) as $pre_values
    | chosen_values($bounded.value;$co) as $post_values
    | (alternatives("unfenced";$pre_values;$gold)
        + alternatives("fenced_value_and_continuation";$post_values;$gold)
        + alternatives("completed_source";{bounded:$bounded.source,consensus:$selection.source};$gold))
      as $explicit
    | bit_has($pre.value.eligible_mask;$gold_len) as $pre_eligible
    | bit_has($bounded.value.eligible_mask;$gold_len) as $post_eligible
    | bit_has($bounded.value.word_start_mask;$gold_len) as $post_word
    | ([$fact_end+5,$bounded.fence]|min) as $stop
    | [
        range($ws;$stop) as $i
        | select(has_at($co.connector_mask;$i;$fact_end)|not)
        | {address:$i, is_word_start:has_at($bounded.value.word_start_mask;$i;$fact_end),
           onset:has_at($co.onset_mask;$i;$fact_end)}
      ][0] as $first
    | {index:$row.index,id:$row.id,seed:$result.seed,group:$g.group,endpoint_slot:$i,
        prefix_sha256:$hash,query_span:$q,retained_fact_span:$row.expected_address.factual_name_span,
        ordered_rank_count:($ranks|length),actual_assertion_occurrences:$sites,
        alternate_ranks:[$ranks[]|select(.retained|not)|{rank,source:.fenced_gpu.bounded.source,
                                eligible:.fenced_gpu.bounded.value.eligible_mask}],
        gold:{source:$gold,fence:$gold_fence,connector_length:$gold_len,
              expected_ids:$row.expected_ids,expected_text:$row.expected_text,
              eligible_unfenced:$pre_eligible,
              word_start_unfenced:bit_has($pre.value.word_start_mask;$gold_len),
              eligible_fenced:$post_eligible,word_start_fenced:$post_word,
              ordered_exact_fenced:bit_has($bounded.value.ordered_mask;$gold_len),
              geometric_connector:bit_has($co.connector_mask;$gold_len),
              geometric_onset:bit_has($co.onset_mask;$gold_len),
              inside_writer_cap:($gold_fence-$gold>=1 and $gold_fence-$gold<=7)},
        unfenced:{frozen_span_source:$pre.frozen.source,proposals:$pre_values,eligible_mask:$pre.value.eligible_mask,
                  word_start_mask:$pre.value.word_start_mask,ordered_mask:$pre.value.ordered_mask},
        fenced:{proposals:$post_values,eligible_mask:$bounded.value.eligible_mask,
                word_start_mask:$bounded.value.word_start_mask,ordered_mask:$bounded.value.ordered_mask,
                cap_accepted:$bounded.source,continuation_reason:$co.reason,
                connector_mask:$co.connector_mask,onset_mask:$co.onset_mask,
                gpu_words:$gpu.words,gpu_parity_words_for_rank:$r.gpu_parity_words,
                first_forward_nonconnector:$first},
        offsets:[
           range(1;4) as $n | seq($fact_end;$n) as $at
           | {connector_length:$n,address:$at,original_id:$row.prefix_original_ids[$at],
              is_gold:($at==$gold),eligible_fenced:bit_has($bounded.value.eligible_mask;$n),
              word_start_fenced:bit_has($bounded.value.word_start_mask;$n),
              exact_ordered_connector:bit_has($bounded.value.ordered_mask;$n),
              geometric_connector:bit_has($co.connector_mask;$n),
              geometric_onset:bit_has($co.onset_mask;$n)}
          ],
        explicit_gold_proposals:$explicit,
        first_observed_gold_eligibility:(if $pre_eligible then "unfenced_value_eligible_mask"
                    elif $post_eligible then "fenced_value_eligible_mask" else null end),
        missing_explicit_winner_from:(if ($explicit|length)==0 then "initial_unfenced_value_ranking" else null end),
        continuation_disposition:disposition($gold;$ws;$first.address;$post_eligible;$post_word;($explicit|length)>0),
        emitted:{source:$actual.source,ids:$actual.generated,decoded:$row.arms.requested.raw_text,
                 release:$actual.steps[-1].release,first_wrong:$row.arms.requested.first_wrong}}
  ] | check(length==11; "expected exactly the eleven correct-span residuals")
