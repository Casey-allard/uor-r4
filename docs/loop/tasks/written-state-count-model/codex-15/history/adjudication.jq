# The independently recorded forced-source adjudication of the 24 exposed losses.
def index: INDEX(.rows[];.id);
def merged:
  if length!=3 then error("expected the three pre-recorded source overrides") else map(index) end;
$exposed[0] as $misses | merged as $arms |
if ($misses|length)!=24 or ($arms|any(.[];length!=24))
then error("wrong exposed adjudication census") else
[ $misses[] as $m |
  [ "Current", "LiteralStart", "WordStart" ] as $names |
  [$arms|to_entries[] | .value[$m.id] as $r |
    if $r.source!=$m.gold or $r.generated!=$m.expected or $r.terminal_eos!=true or $r.exact!=true
    then error("source intervention did not match archived grade exactly for "+$m.id)
    else {type:$names[.key],source:$r.source,generated:$r.generated} end ] as $replays |
  {id:$m.id,w:$m.w,c:$m.c,gold:$m.gold,expected:$m.expected,archived_replays:$replays}]
end
