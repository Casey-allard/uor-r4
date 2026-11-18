# Verify complete frozen select + writer controls against the corresponding #4089 panel.
def controls($trained):
  [.rows[] | select(.arm==$trained or .arm=="Oracle" or .arm=="Disabled")
   | if .arm==$trained then .arm="Current" else . end
   | {case,pair,role,arm,factors,gold_positions,choice,expected,asked_start,other_start,selected,right_source,
      whole_value,first_free,first_wrong,wrong_expected,wrong_emitted,generated,steps,stop}]
  | sort_by(.case,.arm);
($previous|first) as $p |
if .schema!="uor-r4.value-writer/1" or $p.schema!="uor-r4.span-writer/1" or
   .inputs.artifact!=$p.inputs.artifact or .inputs.grade!=$p.inputs.grade or
   .writer_cubin_sha256!=$p.writer_cubin_sha256
then error("artifact, grade, schema or writer mismatch") else . end |
controls("Current") as $current | ($p|controls("Span")) as $old |
if $current!=$old or ($current|length)!=3*$n or
   ([$current[].case]|unique|length)!=$n or
   ($current|group_by(.case)|all(map(.arm)==["Current","Disabled","Oracle"])|not)
then error("changed or incomplete historical control") else
  {cases:$n,artifact_sha256:.inputs.artifact,grade_sha256:.inputs.grade,writer_cubin_sha256:.writer_cubin_sha256,
   identical_trajectories:($current|length),identical_steps:($current|map(.steps|length)|add),
   by_arm:($current|group_by(.arm)|map({arm:.[0].arm,cases:length,steps:(map(.steps|length)|add)}))}
end
