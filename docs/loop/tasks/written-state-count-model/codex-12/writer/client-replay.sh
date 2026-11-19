#!/usr/bin/env bash
# Verify every role word and selected address on four blind tapes, then replay text via the CLI.
set -euo pipefail
: "${VERIFY:?}" "${BIN:?}" "${PACK:?}" "${PANELS:?}" "${TOKENS:?}" "${REF:?}" "${OUT:?fresh directory}"
mkdir "$OUT"
for s in held fresh; do
  grade=$PANELS/$s-grade.json
  # Both members of the first pair and first encounter of every wording, case and evidence type.
  jq -c 'to_entries as $a | ([ $a[] | select(.value.pair==$a[0].value.pair) | .key ] +
    ([$a|group_by(.value.construction)[],group_by(.value.case)[],group_by(.value.evidence)[]|map(.key)|min])) | unique[]' "$grade" > "$OUT/$s-indices"
  for n in 1 2; do
    model=$PACK/seed$n-paired.d11
    tag=seed$n-$s
    "$VERIFY" "$model" "$PACK/seed$n.json" "$PANELS/$s-blind.json" "$REF/$tag-role.json" "$REF/$tag-source.json" >> "$OUT/full.jsonl"
    # Reference rows are the sealed actual feedback of the frozen CUDA writer, indexed in blind-tape order.
    jq -ce --slurpfile i "$OUT/$s-indices" 'if (.rows|length)!=1056 then error("incomplete CUDA reference") end|[ $i[] as $j | {index:$j,row:.rows[$j]} ]' "$REF/$tag-writer.json" > "$OUT/$tag-sample.json"
    while read -r i; do
      file=$OUT/$tag-$i
      prompt=$(jq -er ".[$i].prompt" "$grade")
      for run in primary repeat; do
        "$BIN" --discrete "$model" --tokenizer "$TOKENS/seed$n-tokenizer.json" --say "$prompt" --max-tokens 16 > "$file-$run.json"
      done
      cmp "$file-primary.json" "$file-repeat.json"
      jq -ce --argjson i "$i" --argjson seed "$n" --arg split "$s" --slurpfile refs "$OUT/$tag-sample.json" --slurpfile g "$grade" \
        --slurpfile r "$REF/$tag-role.json" --slurpfile pack "$PACK/seed$n.json" '
        . as $t | ($refs[0]|map(select(.index==$i))|first.row) as $cuda | $g[0][$i] as $k |
        if $t.prompt_ids!=$k.prefix or $t.artifact_sha256!=$pack[0].sha256 or $t.tokenizer_sha256!=$pack[0].tokenizer or $t.paired.role_words!=$r[0].rows[$i].words or
          $t.selected.source!=$cuda.source or $t.generated_ids!=$cuda.generated or $t.copy_step_words!=($cuda.steps|map(.after+[.emitted,(.source_id//4294967295),.copied,.release]))
          then error("normal text differs from CUDA role/source/feedback") end |
        {split:$split,seed:$seed,index:$i,id:$k.id,prompt:$k.prompt,expected:$k.expected,source:.selected.source,generated_ids,text,stop,
        first_wrong:(if $k.expected==null then null else [range([(.generated_ids|length),($k.expected|length)]|max)|select($t.generated_ids[.]!=$k.expected[.])]|first end),
        writer_steps:(.copy_step_words|length)}' "$file-primary.json" >> "$OUT/turns.jsonl"
    done < "$OUT/$s-indices"
  done
done
