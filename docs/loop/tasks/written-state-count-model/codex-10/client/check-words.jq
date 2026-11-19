# Inputs are a normal client JSON record, sealed Wick evidence and the original text input.
. as $turn | $requests[0][$case] as $request |
(if $arm == "enabled" then "role" else "no-read" end) as $condition |
if $turn.model_schema != "uor-r4.lut-discrete-role-copy/1"
   or $turn.artifact_sha256 != $pack[0].artifact
   or $turn.tokenizer_sha256 != $pack[0].tokenizer
   or $turn.read_enabled != ($arm == "enabled")
   or $turn.prompt_ids[0] != 0 or ($turn.copy_step_words | map(.[4])) != $turn.generated_ids
   or (($turn.generated_ids[-1] == 1) != ($turn.stop == "eos"))
then error("wrong carrier, protocol, control or emitted trace") end |
([ $reference[] | select(.seed == $seed and .panel == $request.panel and .index == $request.index and .arm == $condition) ]) as $matches |
if $request.panel=="noncopy" then
  if $matches != [] then error("unexpected device reference") end
else
  if ($matches|length) != 1 then error("missing or duplicate reference") end |
  $matches[0] as $ref |
  if $turn.prompt_ids != $ref.prefix or $request.prompt != $ref.prompt or
     $turn.generated_ids != $ref.writer.generated or $turn.selected.source != $ref.writer.source
     or $turn.copy_step_words != ($ref.writer.steps | map(.after + [.emitted, (.source_id // 4294967295), .copied, .release]))
     or ($condition == "role" and ([$turn.selected.query,$turn.selected.fact] != [$ref.selector.frozen.query,$ref.selector.frozen.fact]))
  then error("client differs from the sealed CUDA prompt, selector or writer") end
end |
{seed:$seed,read_mode:$arm,panel:$request.panel,index:$request.index,prompt:$request.prompt,expected:$request.expected,
 selected:$turn.selected,output:$turn.generated_ids,text:$turn.text,stop:$turn.stop,
 exact_through_eos:(if $request.expected == null then null else $request.expected==$turn.generated_ids end),
 device_steps_checked:(if $request.panel=="noncopy" then 0 else ($turn.copy_step_words|length) end)}
