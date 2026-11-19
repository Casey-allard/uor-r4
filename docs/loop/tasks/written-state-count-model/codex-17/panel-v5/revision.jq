def rewrite:
  .text |= (gsub(" has now painted ";" will now paint ")
            | gsub(" has painted ";" will paint ")
            | gsub(" had once added ";" will later add ")
            | gsub(" had added ";" will add "));
$old[0] as $before | $new[0] as $after |
if ($before|map(rewrite))!=$after then error("text revision changed unexpected words") else
{changed:[range($before|length)|select($before[.]!=$after[.])],
 unchanged:[range($before|length)|select($before[.]==$after[.])],
 reader_disagreement:[
   range($reader_a[0]|length) as $i |
   $reader_a[0][$i] as $a | $reader_b[0][$i] as $b |
   select([$a.person,$a.answer,$a.status]!=[$b.person,$b.answer,$b.status]) |
   {index:$i,prior_text:$before[$i].text,read_a:$a,read_b:$b}
 ],
 fit_texts_unchanged:true,
 model_outcomes_read:false}
end
