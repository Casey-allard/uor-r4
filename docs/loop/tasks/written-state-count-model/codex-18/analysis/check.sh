#!/usr/bin/env bash
# Synthetic grading rows only. No model, bank, panel, or client is loaded.
set -eu -o pipefail
here=$(cd "$(dirname "$0")" && pwd)
fixture() {
    jq -n '
    def wrong($i): {exact:false,terminal_eos:true,first_error:{index:$i,expected_id:55,emitted_id:56}};
    def right: {exact:true,terminal_eos:true,first_error:null};
    def observed($arm;$i):
      (if $i==0 and $arm=="ordered" then right
       elif $i==0 then wrong(0)
       elif $i==1 and $arm=="ranked" then right
       elif $i==2 and $arm=="ranked" then wrong(3)
       elif $i==3 and $arm=="ordered" then wrong(2)
       else wrong(0) end);
    {arms:[["ordered","veto","ranked","both","position"][] as $a |
      {arm:("learned-normal-"+$a),positive_rows:[range(36) as $i |
        {id:("x"+($i|tostring)),index:$i,matched_triple:(if $i<24 then ($i/3|floor) else null end),
         gold_source:7,gold_requested:[2,5],requested:[2,5],selected_source:7,earliest_stage:"fixture"}
         + observed($a;$i)]}]}'
}
fixture | jq -ce -f "$here/factorial.jq" | jq -e '
  (.matched.paired[]|select(.before=="ordered" and .after=="ranked")|.movements) as $m |
  $m.recovered==[1] and $m.regressed==[0] and $m.later==[2] and $m.earlier==[3] and
  ($m.same_index|length)==20 and .matched.interaction_exact_count==0' >/dev/null
if fixture | jq '.arms[0].positive_rows[1].id="x0"' | jq -e -f "$here/factorial.jq" >/dev/null 2>&1; then exit 1; fi
if fixture | jq '.arms[0].positive_rows[0].terminal_eos=false' | jq -e -f "$here/factorial.jq" >/dev/null 2>&1; then exit 1; fi
