. as $frame | $frame.select as $select | $frame.ranks as $ranks | $frame.fallback |
# Reuse actual completed writer trajectories for the identical selected source.
. as $fallback | $select[0] as $s |
if $fallback.bare != $s.bare_sha256 or $fallback.model != $s.artifact_sha256 or
   ($fallback.rows|length) != ($s.rows|length) or
   any($ranks[]; .model != $fallback.model or .bare != $fallback.bare
       or (.rows|length) != ($fallback.rows|length))
then error("unbound selection or actual writer ranks")
else
  [range($s.rows|length) as $i | $s.rows[$i] as $chosen |
      (if $chosen.source == null then $fallback.rows[$i]
       else [$ranks[].rows[$i] | select(.source == $chosen.source)][0]
       end) as $row |
      if $row == null or $row.prefix_sha256 != $chosen.prefix_sha256 or $row.source != $chosen.source
      then error("selected row has no matching actual writer trajectory") else $row end] as $rows |
  {schema:"uor-r4.chosen-cuda-trajectories/1",bare:$fallback.bare,model:$fallback.model,
   transitions:([$rows[].steps|length]|add),rows:$rows}
end
