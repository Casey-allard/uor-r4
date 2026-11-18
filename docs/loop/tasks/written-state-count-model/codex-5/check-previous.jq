# Compare the three arms with matching policy and state between #4087 and this experiment.
def controls:
  [.rows[]|select(.arm=="Content" or .arm=="Oracle" or .arm=="Disabled")
   |{case,pair,role,arm,expected,choice,asked_start,other_start,selected,right_source,whole_value,first_free,first_wrong,
     wrong_expected,wrong_emitted,generated,steps,stop}]|sort_by(.case,.arm);
($previous|first) as $previous_run |
if .schema!="uor-r4.span-writer/1" or $previous_run.schema!="uor-r4.content-writer/1" or
   .inputs.artifact!=$previous_run.artifact_sha256 or .writer_cubin_sha256!=$previous_run.writer_cubin_sha256
then error("different artifact, writer or schema") else . end |
(controls) as $current | ($previous_run|controls) as $prior |
if $current!=$prior or ($current|length)!=3*$n
then error("previous control differs or missing rows") else
  {cases:$n,artifact_sha256:.inputs.artifact,prior_grade_sha256:$previous_run.grade_sha256,new_grade_sha256:.inputs.grade,
   cases_times_arms:($current|length),identical_steps:($current|map(.steps|length)|add),
   arms:($current|group_by(.arm)|map({arm:.[0].arm,identical_trajectories:length,identical_steps:(map(.steps|length)|add)}))}
end
