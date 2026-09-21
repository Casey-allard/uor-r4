#!/usr/bin/env python3
"""Independent saved-event audit of corrected PR1339 exposed replay; no model execution."""
import collections, hashlib, json, pathlib, struct
import blake3
P=pathlib.Path; BASE=P('/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20'); ROOT=BASE/'shared-transition-principal-1'; REPO=P('/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4')
sha=lambda b:hashlib.sha256(b).hexdigest(); load=lambda p:json.loads(p.read_text())
if not (ROOT/'manifest.json').exists():raise SystemExit('Replay is not sealed; no audit result created.')
r=load(ROOT/'result.json');m=load(ROOT/'manifest.json');a=load(ROOT/'attempt.json');rows=[json.loads(l) for l in (ROOT/'rows.jsonl').read_text().splitlines()];listed={x['path'] for x in m['files']};actual={str(p.relative_to(ROOT)) for p in ROOT.rglob('*') if p.is_file()}-{'manifest.json'};exe=P(a['argv'][0])
out={'schema':'uor-r4.pr1339-corrected-exposed-replay-independent-audit/1','scope':'Read-only artifact/hash/source verification and saved actual-prediction recount. No new model execution, fit or build. Corrected exposed regression, not fresh final qualification.','root':str(ROOT),'integrity':{'listed_files':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'seal_errors':[x['path'] for x in m['files'] if (len((ROOT/x['path']).read_bytes())!=x['bytes'] or blake3.blake3((ROOT/x['path']).read_bytes()).hexdigest()!=x['blake3'])],'source_hash_matches':{x['path']:sha((REPO/x['path']).read_bytes())==x['sha256'] for x in r['running_source']['source_files']},'executable_path':str(exe),'executable_sha256':sha(exe.read_bytes()) if exe.exists() else None,'executable_matches':sha(exe.read_bytes())==r['running_source']['executable_sha256'] if exe.exists() else False,'result_sha256':sha((ROOT/'result.json').read_bytes()),'rows_sha256':sha((ROOT/'rows.jsonl').read_bytes())},'counts':{},'artifacts':{},'report_count_checks':{}}

def rlst(p):
 b=p.read_bytes();c=0
 def take(n):
  nonlocal c
  z=b[c:c+n];assert len(z)==n;c+=n;return z
 def u32():return struct.unpack('<I',take(4))[0]
 assert take(4)==b'RLST';version=u32();cyclic=bool(take(1)[0]);legacy=True if version==1 else bool(take(1)[0]);dom=[];codes=[]
 for _ in range(2):n=u32();dom.append([u32() for _ in range(n)])
 for _ in range(2):n=u32();codes.append(list(take(n)))
 n=u32();stop=[list(take(2)) for _ in range(n)];dl=u32();end=c+dl;assert take(4)==b'RLRD';dv=u32();ns=u32();st=[]
 for _ in range(ns):
  s,valid=take(2);nk=struct.unpack('<H',take(2))[0];ts=[u32() for _ in range(nk)];sc=[struct.unpack('<i',take(4))[0] for _ in range(nk)];i=min(range(nk),key=lambda j:(-sc[j],ts[j])) if nk else None
  st.append({'state':s,'top_token':ts[i] if i is not None and valid else None})
 assert c==end==len(b)
 return {'bytes':len(b),'sha256':sha(b),'version':version,'cyclic':cyclic,'legacy_exhaustion_stop':legacy,'value_domain':dom[0],'action_domain':dom[1],'value_codes':codes[0],'action_codes':codes[1],'stop_table':stop,'decoder_state_count':ns,'states':st}
for p in (ROOT/'artifacts').glob('*.rlst'):out['artifacts'][p.stem]=rlst(p)
finite=load(ROOT/'artifacts/finite_shared_transition.json');out['artifacts']['finite_shared_transition']={'bytes':(ROOT/'artifacts/finite_shared_transition.json').stat().st_size,'sha256':sha((ROOT/'artifacts/finite_shared_transition.json').read_bytes()),'initial_entries':len(finite['initial']),'recurrent_entries':len(finite['recurrent']),'values':finite['values'],'actions':finite['actions'],'stop_table':finite['stop_table']}
vo=ROOT/'artifacts/value_only_lexical.rlds';out['artifacts']['value_only_lexical']={'bytes':vo.stat().st_size,'sha256':sha(vo.read_bytes())}
for x in r['artifacts']:out['artifacts'][x['arm']]['hash_matches']=out['artifacts'][x['arm']]['sha256']==x['artifact_sha256']
old=collections.defaultdict(list)
for line in (BASE/'shared-transition-6/rows.jsonl').read_text().splitlines():
 x=json.loads(line);old[x['item_id']].append(x)
for xs in old.values():xs.sort(key=lambda x:x['primitive_index'])
by=collections.defaultdict(list)
for x in rows:by[x['arm'],x['split']].append(x)
complete=lambda x:x['stopped'] and x['tokens']==x['expected']
correct=lambda x:x['read'] and x['payload_seq_abs'] is not None and x['payload_seq_abs'][1]==x['intended_payload_abs'] and x['selected_payload']==x['intended_payload']
for (arm,split),xs in by.items():
 seqs=collections.defaultdict(list)
 for x in xs:seqs[tuple(x['observed_primitives'])].append(x)
 md=out['artifacts'].get({'h4_shared_transition':'shared_transition_h4','c120_shared_transition':'shared_transition_c120'}.get(arm,arm));ds={x['state']:x['top_token'] for x in md['states']} if md and 'states'in md else None
 d={'items':len(xs),'complete':sum(map(complete,xs)),'token_hits':sum(sum(i<len(x['tokens']) and x['tokens'][i]==t for i,t in enumerate(x['expected'])) for x in xs),'token_total':sum(len(x['expected']) for x in xs),'stopped_correctly':sum(x['stopped'] and len(x['tokens'])==len(x['expected']) for x in xs),'reads':sum(x['read'] for x in xs),'correct_selected_sources':sum(map(correct,xs)),'correct_source_complete':sum(correct(x) and complete(x) for x in xs),'wrong_source_complete':sum(not correct(x) and complete(x) for x in xs),'terminal_counts':dict(collections.Counter(x['steps'][-1]['kind'] if x['steps'] else 'EMPTY' for x in xs)),'unique_sequences':len(seqs),'sequences_all_values_complete':sum(all(map(complete,ss)) for ss in seqs.values()),'complete_flags_match':all(x['complete']==complete(x) for x in xs),'step_emissions_match_tokens':all([s['emitted'] for s in x['steps'] if s['kind']=='Emit']==x['tokens'] for x in xs),'instruction_span_matches':all(x['observed_primitives']==x['prefix'][x['instruction_start']:-2] for x in xs),'expected_matches_original':all(x['expected']==[z['expected'] for z in old[x['item_id']]] for x in xs),'read_occurrence_matches_prefix':all(x['payload_seq_abs'] is not None and x['source_seq_abs'] is not None and x['source_seq_abs'][0]==x['payload_seq_abs'][0] and x['source_seq_abs'][1]+1==x['payload_seq_abs'][1] and x['prefix'][x['payload_seq_abs'][1]]==x['selected_payload'] for x in xs if x['read']),'decoder_tokens_match_artifact':all(s['emitted']==ds.get(s['post_state']) for x in xs for s in x['steps'] if s['kind']=='Emit') if ds else None}
 if arm=='h4_shared_transition':d['tokens_match_original_every_item']=all(x['tokens']==[z['emitted'] for z in old[x['item_id']] if z['emitted'] is not None] for x in xs)
 out['counts'][arm+'/'+split]=d
for a in r['arms']:
 k=a['arm']+'/'+a['split'];d=out['counts'][k];out['report_count_checks'][k]={f:d[f]==a[f] for f in ['items','complete','token_hits','token_total','stopped_correctly']}
out['all_report_counts_match']=all(all(z.values()) for z in out['report_count_checks'].values())
# Independently verify serialized finite transitions have exactly the development supervision labels.
init=collections.defaultdict(set);recur=collections.defaultdict(set)
for x in by['h4_shared_transition','dev']:
 for j,(p,y) in enumerate(zip(x['observed_primitives'],x['expected'])):
  if j==0:init[x['intended_payload'],p].add(y)
  else:recur[x['expected'][j-1],p].add(y)
fi={(x,y):z for x,y,z in finite['initial']};fr={(x,y):z for x,y,z in finite['recurrent']}
out['finite_training_audit']={'all_initial_dev_labels_unique':all(len(s)==1 for s in init.values()),'all_recurrent_dev_labels_unique':all(len(s)==1 for s in recur.values()),'initial_artifact_exactly_dev':len(fi)==len(init) and all(fi.get(k) in s for k,s in init.items()),'recurrent_artifact_exactly_dev':len(fr)==len(recur) and all(fr.get(k) in s for k,s in recur.items()),'no_heldout_labels_needed':True,'footprint_note':'JSON table uses more bytes than RLST; outcome is a competent shared-state baseline, not an equal-byte efficiency comparison.'}
out['paired_complete']={}
for split in ['dev','held_out_length4','held_out_reversal']:
 h={x['item_id']:x for x in by['h4_shared_transition',split]}
 for arm in ['c120_shared_transition','finite_shared_transition']:
  xs=by[arm,split];out['paired_complete'][split+'/'+arm]={'both_correct':sum(complete(x) and complete(h[x['item_id']]) for x in xs),'h4_only':sum(not complete(x) and complete(h[x['item_id']]) for x in xs),'control_only':sum(complete(x) and not complete(h[x['item_id']]) for x in xs),'neither':sum(not complete(x) and not complete(h[x['item_id']]) for x in xs),'same_actual_selected_source':all(x['source_seq_abs']==h[x['item_id']]['source_seq_abs'] and x['selected_payload']==h[x['item_id']]['selected_payload'] for x in xs)}
out['interventions']=r['interventions'];iv=out['interventions'];pay=iv['payload_changed_same_query_and_primitives'];order=iv['order_changed_fixed_evidence_end_to_end'];em=iv['retained_state_under_decoder_label_permutation']
out['control_recounts']={'payload_changed_indices':[i for i,(a,b) in enumerate(zip(pay['a']['prefix'],pay['b']['prefix'])) if a!=b],'payload_pair_complete':complete(pay['a']) and complete(pay['b']),'payload_pair_correct_sources':correct(pay['a']) and correct(pay['b']),'order_changed_indices':[i for i,(a,b) in enumerate(zip(order['forward']['prefix'],order['reversed']['prefix'])) if a!=b],'order_evidence_unchanged':order['forward']['prefix'][:3]==order['reversed']['prefix'][:3],'order_pair_complete':complete(order['forward']) and complete(order['reversed']),'source_removal_actual_prefix':iv['source_removed']['prefix'],'source_removal_read':iv['source_removed']['read'],'source_removal_selected_payload':iv['source_removed']['selected_payload'],'source_removal_tokens':iv['source_removed']['tokens'],'source_removal_terminal':iv['source_removed']['steps'][-1]['kind'],'read_disabled_read':iv['read_disabled']['read'],'read_disabled_tokens':iv['read_disabled']['tokens'],'emission_permutation_changed':em['original_tokens']!=em['modified_tokens'],'emission_permutation_states_identical':em['original_states']==em['modified_states']}
w=iv['noncommuting_final_state_witness'];out['witness_recount']={'primitives':w.get('primitives'),'h4_final_state_changes':w.get('h4_final_state_forward')!=w.get('h4_final_state_reversed'),'c120_final_state_same':w.get('c120_final_state_forward')==w.get('c120_final_state_reversed'),'h4_forward_complete':w.get('h4_tokens_forward')==w.get('expected_forward'),'h4_reversed_complete':w.get('h4_tokens_reversed')==w.get('expected_reversed'),'tokens_forward':w.get('h4_tokens_forward'),'tokens_reversed':w.get('h4_tokens_reversed'),'expected_forward':w.get('expected_forward'),'expected_reversed':w.get('expected_reversed')}
out['generation']=[{'value':z['value'],'primitives':z['primitives'],'emitted':z['emitted'],'expected':z['expected'],'complete':z['stopped'] and z['emitted']==z['expected'],'flag_matches':z['complete_correct']==(z['stopped'] and z['emitted']==z['expected'])} for z in r['generated']]
out['source_receipt']=r['running_source'];out['screen']=r['screen'];out['resources']={'elapsed_s':r['elapsed_s'],'audit_model_execution':False,'new_build':False,'energy':'UNAVAILABLE'};out['limits']=['Exposed replay, no fresh final evaluation.','Fixture supplies instruction-span boundaries and expected intermediate labels for fitting.','Stop remains supervised input exhaustion; RLSTv2 now distinguishes absent-policy exhaustion. No learned general scheduler.','One local computed-state lifetime inside serve; persistent exact occurrence leases/session state and dependent read NOT_RUN.','Four generated examples are four values on one primitive sequence.','Independent recount verifies retained events/parameter tables, not a second predictor execution.','No whole-path latency/energy, natural prose, coding or general reasoning qualification.'];out['script_sha256']=sha(P(__file__).read_bytes());dest=P('/tmp/uor-pr1339-corrected-replay-audit.json');dest.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'output':str(dest),'integrity':out['integrity'],'all_report_counts_match':out['all_report_counts_match'],'counts':out['counts'],'control_recounts':out['control_recounts'],'witness_recount':out['witness_recount'],'finite_training_audit':out['finite_training_audit'],'resources':out['resources']},indent=2))
