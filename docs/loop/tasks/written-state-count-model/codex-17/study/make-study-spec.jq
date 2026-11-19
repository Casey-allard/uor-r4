# jq -ne --argjson seed N --arg root ABS --rawfile files reviewed-files.tsv --slurpfile captures CAPTURE_ROOTS -f this-file
# Every nonempty line of files is name<TAB>absolute path<TAB>externally pinned sha256.
[ $files | split("\n")[] | select(length>0) | split("\t")
 | if length==3 and (.[1]|startswith("/")) and (.[2]|test("^[0-9a-f]{64}$"))
   then {key:.[0],value:{path:.[1],sha256:.[2]}} else error("malformed pin") end
] as $pins
| if ($pins|map(.key)|unique|length) != ($pins|length) or ($captures|length)!=1
  then error("duplicate pin or capture root set") else
  {schema:"uor-r4.value-rank-study-wick/1",seed:$seed,wick_root:$root,files:($pins|from_entries),captures:$captures[0]}
  end
