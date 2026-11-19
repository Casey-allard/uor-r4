# The role-only handoff is the sole address candidate. Grade rows are used only for the forced-source control.
. as $role | if .arm!="role" or .schema!="uor-r4.entity-pair-chain/1" then error("invalid role source") end |
if $arm=="action" and (($pred|length)!=1 or $pred[0].arm!="action" or $pred[0].artifact_sha256!=.artifact_sha256
  or $pred[0].bare_sha256!=.bare_sha256 or ($pred[0].rows|length)!=(.rows|length)) then error("unmatched action seal") end |
if $arm=="forced-read" and ($grade[0]|length)!=(.rows|length) then error("unmatched forced-source key") end |
.arm=$arm |
.rows=[range(.rows|length) as $i | .rows[$i] |
 (if $arm=="action" then $pred[0].rows[$i] as $p |
     if $p.prefix_sha256!=.prefix_sha256 or ($p.allowed|type)!="boolean" or
      $p.source!=(if $p.allowed then .continuation.source else null end) then error("gate changed source or prompt") end |
     $p.source
  elif $arm=="no-read" then null
  elif $arm=="forced-read" then ($grade[0][$i].forced_fact_source//4)
  elif $arm=="role" then .continuation.source else error("unknown arm") end) as $source |
 {prefix_sha256,continuation:(.continuation|.source=$source)}]
