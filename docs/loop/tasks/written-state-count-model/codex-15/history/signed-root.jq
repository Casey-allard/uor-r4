def blocks($word): [1,16,256,4096] | map($orbit[0][(($word / .)|floor)%16]);
def abs: if . < 0 then -. else . end;
def l1($b): [range(length) as $i | (.[ $i ] - $b[$i]) | abs] | add;
def pair($a;$b):
 {ids:[$a,$b],roots:[$roots[0][$a],$roots[0][$b]],
  positions_equal:(blocks($roots[0][$a])==blocks($roots[0][$b])),
  l1:( [blocks($roots[0][$a])[] []] | l1([blocks($roots[0][$b])[] []]))};
{orbit_count:($orbit[0]|length),unique_signed_roots:($orbit[0]|unique|length),
 matches:{
  chose_scarlet:[pair(641;2346),pair(1935;2211)],
  chose_silver:[pair(641;3585),pair(1935;322)],
  old_only:[pair(782;1370)],
  old_just:[pair(782;801)]
 }}
