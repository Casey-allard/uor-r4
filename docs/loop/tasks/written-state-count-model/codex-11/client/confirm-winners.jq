# Rows are (previous ID, position of an observed gold target, device winner, raw Q32).
($tsv|split("\n")|map(select(length>0)|split("\t")|map(tonumber))|
    map({key:(.[0]|tostring),value:{position:.[1],winner:.[2],mass:.[3]}})|from_entries) as $device |
[.[] as $run | range($run.generated_ids|length) as $i |
 (if $i==0 then $run.previous else $run.generated_ids[$i-1] end) as $previous |
 $device[($previous|tostring)] as $obs |
 if $obs==null then {checked:false,previous:$previous}
 elif $obs.winner!=$run.generated_ids[$i] or $obs.mass!=$run.generated_mass_q32[$i]
 then error("client differs from observed GPU winner or mass")
 else {checked:true,previous:$previous} end] |
{steps:length,observed_device_pairs:(map(select(.checked))|length),
 missing_contexts:(map(select(.checked|not)|.previous)|unique)}
