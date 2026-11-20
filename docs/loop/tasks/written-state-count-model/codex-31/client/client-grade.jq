# Slurp the independently generated CUDA step JSONL on stdin. The wrapper
# supplies one complete ordinary-client record per (prompt, read mode).
def check($ok; $msg): if $ok then . else error($msg) end;
def whole($min;$max): type=="number" and .==floor and .>=$min and .<=$max;
def sha: type=="string" and test("^[0-9a-f]{64}$");
def valid_step:
  (.token|whole(1;4095)) and (.bucket|whole(0;527)) and (.top_ties|whole(1;4095))
  and (.mass_q48|whole(1;281474976710656)) and (.context_tokens|whole(1;192))
  and (.mass_sha256|sha) and (.prefix_sha256|sha)
  and has("pick_words") and has("pick_successor")
  and (if .pick_words==null then .pick_successor==null else
    (.pick_words|type=="array" and length==3) and (.pick_words[0]|whole(1;190))
    and (.pick_words[1]|whole(0;128)) and (.pick_words[2]|whole(1;190))
    and (.pick_successor|whole(1;4095)) end);
def wrong($a;$b):
  ([range(0; ([($a|length), ($b|length)]|max)) | select($a[.]!=$b[.])] | first);
. as $gpu |
check(($cpu|length)==16 and ($keys[0]|length)==8 and ($blind[0].rows|length)==8;"incomplete ordinary panel") |
check(($receipt|length)==1;"missing singular CUDA receipt") |
$receipt[0] as $bound |
check($bound.schema=="uor-r4.d31.cuda-client-oracle/1"
      and $bound.trunk_sha256==$trunk_sha and $bound.panel_sha256==$blind[0].panel_sha256
      and $bound.prefix_packet_sha256=="a4fb823cdd5031cc7ffdb1c402f7d7df2c149002b1335a102a8a012371f081dc"
      and $bound.candidates_per_step==4095 and $bound.steps==($gpu|length);
      "CUDA receipt identity, candidate width or row count differs") |
($bound|{count_sha256,weights_sha256,fit_manifest_sha256}) as $components |
check([$components[] | type=="string" and test("^[0-9a-f]{64}$")]|all;"invalid CUDA component identity") |
[$keys[0][] as $key | [true,false][] as $read |
  ($blind[0].rows|map(select(.id==$key.id))|check(length==1;"missing blind id")|first.prefix) as $prefix |
  ($cpu|map(select(.id==$key.id and .result.read_enabled==$read))|check(length==1;"missing ordinary arm")|first.result) as $r |
  ($gpu|map(select(.id==$key.id and .read_enabled==$read))|sort_by(.step)) as $steps |
  $r | check(.prompt_ids==$prefix and .artifact_sha256==$trunk_sha and .p1_sha256==$pkg_sha;"client identity or tokenizer mismatch") |
  check(.p1_components==$components;"CUDA and ordinary client bound different fitted components") |
  check((.generated_ids|length)>0 and (.generated_ids|length)<=12 and (.generated_ids==[.p1_steps[].token]);"client token/trace disagreement") |
  check((.generated_ids|length)==($steps|length);"CUDA and client trajectory lengths differ") |
  .generated_ids as $ids |
  check(($ids[-1]==1 or ($ids|length)==12) and ($ids[:-1]|index(1)==null);"invalid stop") |
  [range(0; ($ids|length)) as $i |
    (.p1_steps[$i] + {id:$key.id,read_enabled:$read,step:$i,
      stop:(if $ids[$i]==1 then "eos" elif $i==11 then "max_tokens" else null end)}) as $expect |
    check(($expect|valid_step); "missing or invalid decision words at id \($key.id) step \($i)") |
    check($expect==$steps[$i]; "device mismatch at id \($key.id) mode \($read) step \($i): client \($expect) device \($steps[$i])") |
    $steps[$i]] as $checked |
  {id:$key.id,read_enabled:$read,prompt:$key.text,text:.text,expected:$key.gold,generated:$ids,
   steps:($checked|length),eos:($ids[-1]==1),first_wrong:(if $key.gold==null then null else wrong($ids;$key.gold) end),
   exact:(if $key.gold==null then null else $ids==$key.gold end)}] as $rows |
check(($rows|map(.steps)|add)==($gpu|length);"unexpected CUDA rows") |
{schema:"uor-r4.d31.client-grade/1",components:$components,rows:$rows,
 summary:[$rows|group_by(.read_enabled)[]|{read_enabled:.[0].read_enabled,scored:(map(select(.expected!=null))|length),
 exact_eos:(map(select(.exact==true))|length),steps:(map(.steps)|add),ended_eos:(map(select(.eos))|length)}]}
