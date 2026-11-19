def cell:
  tostring | gsub("\\|"; "\\|") | gsub("\n"; " ") | gsub("\r"; " ");
def row($fields): "| " + ($fields|map(cell)|join(" | ")) + " |";
def boolean: if . then "yes" else "no" end;
def token:
  if . == null then "∅"
  else "\(.id) \(.piece|@json)" end;
def mismatch:
  if .mismatch == null then
    if .exact then "exact" else "n/a" end
  else .mismatch | "\(.index) \(.expected|token) → \(.actual|token)" end;
def arm:
  "q=\(.query // "∅") \(.endpoint) \([.query_tokens[]?|token]|join(" ")); src=\(.source // "∅") \(.source_kind.classification) \(.source_token|token); \(mismatch); EOS \(.terminal_eos|boolean)";
def negative_arm:
  "q=\(.query // "∅") \(.endpoint); src=\(.source // "∅"); \(.negative.classification); \(.emitted|length) IDs, EOS \(.terminal_eos|boolean); \(.text)";
def counts($a;$b):
  .arms[$a] as $old | .arms[$b] as $new |
  ["\($old.span) → \($new.span)",
   "\(.span_changes.recovered // 0) / \(.span_changes.regressed // 0)",
   "\($old.sources.correct_value // 0) → \($new.sources.correct_value // 0)",
   "\($old.exact) → \($new.exact)",
   "\(.exact_changes.recovered // 0) / \(.exact_changes.regressed // 0)",
   .wrong_to_different_wrong,.new_wrong_sources,.early_unseen_gold_wrong_releases];
def cells($a;$b):
  row(["Family","Exclusion first","Name B first","Rows","Correct spans","Span gains / losses",
       "Correct sources","Exact replies","Exact gains / losses","Wrong changed to different wrong",
       "New wrong sources","Wrong releases with earlier unseen gold"]),
  row([range(12)|"---"]),
  (.[] | row([.family,.exclusion_first,.name_b_first,.rows] + counts($a;$b)));

"# D21 paired results, seed \(.seed)",
"",
"Each position is zero-based. Query coordinates and source offsets are positions on the original prefix. Token pieces are shown with their vocabulary IDs. A mismatch gives the output position followed by the expected and emitted token. A missing token is ∅.",
"",
"## Ordinary role comparison",
"",
([.supported.role + {family:"Total",exclusion_first:"—",name_b_first:"—"}] + .roles_by_cell
 | cells("reference";"clause")),
"",
"## Supplied query and connector veto",
"",
([.supported.value + {family:"Total",exclusion_first:"—",name_b_first:"—"}] + .values_by_cell
 | cells("supplied";"veto")),
"",
"## Completion and control scope",
"",
row(["Arm","Supported rows","Supported terminal EOS","Supported 16 IDs without EOS",
     "Unsupported rows","Unsupported terminal EOS","Unsupported 16 IDs without EOS"]),
row([range(7)|"---"]),
(. as $r | ($r.supported.role.arms + $r.supported.value.arms + $r.supported.other_controls)
 | to_entries[] | .key as $arm | $r.unsupported.arms[$arm] as $negative |
 row([$arm,96,.value.eos,.value.cap_without_eos,
      (if $negative==null then 0 else 6 end),($negative.eos // "—"),($negative.cap_without_eos // "—")])),
"",
"## Supported prompt evidence",
"",
row(["Ordinal","Prompt ID","Gold query","Gold source","Reference","Clause","Supplied ordered","Earlier connector veto"]),
row([range(8)|"---"]),
(.rows[] | select(.stratum=="supported") |
 row([.ordinal,.id,"\(.gold_query) \([.gold_query_tokens[]|token]|join(" "))",.gold_start,
      (.arms.reference|arm),(.arms.clause|arm),(.arms.supplied|arm),(.arms.veto|arm)])),
"",
"## Unsupported controls",
"",
row(["Ordinal","Prompt ID","Reference","Clause","No read"]),
row([range(5)|"---"]),
(.rows[] | select(.stratum!="supported") |
 row([.ordinal,.id, (.arms.reference|negative_arm),
    (.arms.clause|negative_arm),(.arms.no_read|negative_arm)]))
