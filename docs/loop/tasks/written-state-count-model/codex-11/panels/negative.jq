# Historical arm/handoff plus one deliberately corrupt join key.
.[0] as $role | .[1] |
if .rows[0].prefix_sha256!=$role.rows[0].prefix_sha256 then error("invalid control") end |
.rows[0].prefix_sha256=("0"*64)
