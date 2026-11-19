# Usage: jq -e --slurpfile a reader-A --slurpfile b reader-B -f adjudicate.jq grades
if length!=42 or ($a[0]|length)!=length or ($b[0]|length)!=length then error("unaligned readers") else
[ to_entries[] | .key as $i | .value as $grade |
  (if $grade.stratum=="unmentioned" then "absent" else $grade.stratum end) as $status |
  {index:$i, id:$grade.id, authored:[$grade.requested_name,$status,$grade.answer_text],
   readers:[$a[0][$i],$b[0][$i]]} |
  if any(.readers[]; .index!=$i or (.why|type)!="string" or .why=="") then error("missing reading")
  else . + {agreement:all(.readers[]; [.person,.status,.answer] == [$grade.requested_name,$status,$grade.answer_text])} end
] | {rows:length, full_agreements:([.[]|select(.agreement)]|length), disagreements:[.[]|select(.agreement|not)]}
end
