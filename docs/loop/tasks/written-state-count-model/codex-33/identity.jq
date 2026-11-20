# Slurp the independent oracle JSONL as input. Supply pinned GPU and loader JSON objects.
def check($ok; $reason): if $ok then . else error($reason) end;
def sha: type=="string" and test("^[0-9a-f]{64}$");
[.[]|select(has("raw") and has("adjusted"))] as $oracle |
$gpu[0] as $device | $loader[0] as $reader |
check(($oracle|length)==11 and ($device.counts.raw|length)==11 and ($device.counts.adjusted|length)==11
      and ($reader.counts.raw|length)==11 and ($reader.counts.adjusted|length)==11;"incomplete order set") |
check($reader.artifact_sha256==$outer and ($outer|sha) and $reader.model_schema=="uor-r4.lut-discrete-count/1"
      and $reader.components.count_sha256==$device.counts.artifact_sha256
      and $reader.p1_trunk_sha256==$device.base_trunk_sha256;"component identity mismatch") |
[range(0;11) as $order | ["raw","adjusted"][] as $domain |
   $device.counts[$domain][$order] as $gpu_row | $oracle[$order][$domain] as $cpu_row |
   $reader.counts[$domain][$order] as $d11_row |
   check($gpu_row.order==$order and ($gpu_row.triple_sha256|sha) and ($gpu_row.context_sha256|sha)
         and ($gpu_row.count_of_counts|length)==4;"invalid device order") |
   ["order","targets","grams","contexts","count_of_counts","triple_sha256","context_sha256"] as $fields |
   [$fields[] as $field | select($gpu_row[$field]==null or $gpu_row[$field]!=$cpu_row[$field]
          or $gpu_row[$field]!=$d11_row[$field]) | {field:$field,gpu:$gpu_row[$field],
          oracle:$cpu_row[$field],loader:$d11_row[$field]}] as $bad |
   check(($bad|length)==0;"order \($order) \($domain): \($bad|tojson)") |
   {order:$order,domain:$domain,targets:$gpu_row.targets,grams:$gpu_row.grams,contexts:$gpu_row.contexts,
    count_of_counts:$gpu_row.count_of_counts,triple_sha256:$gpu_row.triple_sha256,context_sha256:$gpu_row.context_sha256}]
as $checked |
{schema:"uor-r4.d33-count-identity/1",source_sha256:$device.train_sha256,
 records:$device.train_records,tokens:$device.train_tokens,outer_sha256:$outer,
 components:$reader.components,bytes:$reader.bytes,matching_domains:($checked|length),
 first_divergence:null,rows:$checked}
