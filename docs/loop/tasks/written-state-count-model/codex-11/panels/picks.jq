. as $r | (
  if $arm=="role" then "role" elif $arm=="none" then "no-read"
  elif $arm=="right" or $arm=="wrong" then "forced-read" else "action" end
) as $frozen_arm |
if .schema!="uor-r4.entity-pair-chain/1" or .arm!="role" then error("role handoff corrupted") end |
if ($p|length)==1 and ($p[0].artifact_sha256!=.artifact_sha256 or $p[0].bare_sha256!=.bare_sha256
    or ($p[0].rows|length)!=(.rows|length)) then error("unbound arm") end |
if ($grade|length)==1 and ($grade[0]|length)!=(.rows|length) then error("unbound upper/lower control") end |
.arm=$frozen_arm | .rows=[range(.rows|length) as $i|.rows[$i] as $row|
 (if $arm=="role" then $row.continuation.source elif $arm=="none" then null
  elif $arm=="right" then $grade[0][$i].asked_start
  elif $arm=="wrong" then $grade[0][$i].other_start
  else $p[0].rows[$i] |
      if (.prefix_sha256!=$row.prefix_sha256 or (has("source")|not) or (.source!=null and (.source|type)!="number"))
      then error("mismatched decision or missing address") end |.source end) as $s |
 $row|{prefix_sha256,continuation:(.continuation|.source=$s)}]
