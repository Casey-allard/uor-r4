#!/usr/bin/env python3
"""Independent corrected saved-data audit: hashes, typed oracles, exact text/provenance and saved continuations. No model fit/inference/build."""
import collections,copy,hashlib,json,pathlib,subprocess,sys
import blake3
P=pathlib.Path;REPO=P('/Users/casey.allard/uor-r4');BASE=REPO/'.uor-models/realtext-prior-2026-09-20';ROOT=BASE/'structured-argument-binding-principal-1';REVIEW=P('/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4')
load=lambda p:json.loads(p.read_text());sha=lambda b:hashlib.sha256(b).hexdigest();compact=lambda v:json.dumps(v,separators=(',',':'),ensure_ascii=False).encode()
r=load(ROOT/'result.json');wd=load(ROOT/'worlds.json');rows=[json.loads(x)for x in(ROOT/'rows.jsonl').read_text().splitlines()];modelb=(ROOT/'artifacts/contextual_roles_model.json').read_bytes();model=json.loads(modelb);EOS=r['task']['eos'];VOCAB=4096
m=load(ROOT/'manifest.json');listed={f['path']for f in m['files']};actual={str(p.relative_to(ROOT))for p in ROOT.rglob('*')if p.is_file()}-{'manifest.json'}
frozen={'crates/uor-r4-core/src/native_geometric/learner/observed_text_session.rs':'ef1c51b35344dcb98fb37ea9d7608e3efc91d59be993afc58f4aeaefb2e3b24d','crates/uor-r4-core/src/native_geometric/learner/relational_session.rs':'1c89b2b48a4041542b8fadf58aa29a30deedde75d7161258155a7103070112e5','crates/uor-r4-core/src/bin/competitive-reader.rs':'4d866fd6c35329d45d379468150c22ce9cb9c7cabac85c8d6594929a37d1ecad'}
# Default to the published corrected draft head, independent of later worktree switches.
source_commit=sys.argv[1]if len(sys.argv)>1 else '1df229178c6b56ae6dbf95273c9e4eae3282a66e'
source_bytes=lambda path:subprocess.check_output(['git','show',source_commit+':'+path],cwd=REPO)if source_commit else(REVIEW/path).read_bytes()
exe=P(load(ROOT/'attempt.json')['argv'][0]);out={'schema':'uor-r4.pr1344-corrected-independent-audit/1','scope':__doc__,'root':str(ROOT),'seal':{'listed':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'size_errors':[f['path']for f in m['files']if(ROOT/f['path']).stat().st_size!=f['bytes']],'blake3_errors':[f['path']for f in m['files']if blake3.blake3((ROOT/f['path']).read_bytes()).hexdigest()!=f['blake3']]},'source':{'commit':source_commit,'running_source':r['running_source'],'frozen_review_source_sha256':frozen,'actual_source_matches':{f['path']:sha(source_bytes(f['path']))==f['sha256']==frozen[f['path']]for f in r['running_source']['source_files']},'executable_matches':sha(exe.read_bytes())==r['running_source']['executable_sha256'],'model_sha256':sha(modelb),'model_bytes':len(modelb),'model_version':model['version']}}
worlds={w['id']:w for split in ['development','exposed_regression']for w in wd[split]};persons=wd['membership_entities'];vocab=load(REPO/'.uor-models/sources/smollm2-135m-instruct/tokenizer.json')['model']['vocab'];rev={v:k for k,v in vocab.items()if v<VOCAB}
def decode(ts):return ''.join(rev[t]for t in ts if t!=EOS).replace('Ġ',' ')
def aligned(c):return bool(c['byte_lengths'])and len(c['tokens'])==len(c['byte_lengths'])and sum(c['byte_lengths'])==len(c['text'].encode())
def lexical(c,start,length):
 if aligned(c):return [1]+list(c['text'].encode()[sum(c['byte_lengths'][:start]):sum(c['byte_lengths'][:start+length])].strip(b' \t\r\n\v\f'))
 return [0]+list(b''.join(t.to_bytes(4,'little')for t in c['tokens'][start:start+length]))
def clausebytes(cs):return compact([{'seg':c['seg'],'tokens':c['tokens'],'text':c['text'],'byte_lengths':c['byte_lengths']}for c in cs])
def control(arm):return {'membership_continuation':{'Membership':{'entities':persons}},'maximum_two_read':{'ReadCap':{'max_reads':2}},'reads_disabled':'ReadsDisabled'}.get(arm,'Normal')
def expected(w,person,goal,arm='contextual_primary'):
 cs={c['seg']:c for c in w['clauses']};visited=[];q=persons[person]
 def result(ans,term):return {'answer':ans,'reads':len(visited),'selected_segments':[v[0]for v in visited],'terminal':term}
 if arm=='reads_disabled':return result([],'Unresolved')
 for _ in range(6):
  if arm=='maximum_two_read'and len(visited)>=2:return result([],'Exhausted')
  matches=[]
  for l in w['labels_for_evaluation_only']:
   c=cs[l['seg']];s,n=l['subject']
   if l['goal']==goal and c['tokens'][s:s+n]==q:matches.append((c,l))
  if not matches:return result([],'Unresolved')
  assert len(matches)==1
  c,l=matches[0];s,n=l['object'];span=[c['seg'],s,n]
  if span in visited:return result([],'Exhausted')
  visited.append(span);q=c['tokens'][s:s+n]
  emit=q not in persons if arm=='membership_continuation'else l['action']=='Emit'
  if emit:return result(q,'Stop')
 return result([],'Exhausted')
errors=[];snaperrors=[];counts=collections.Counter();phases=collections.Counter();keytypes=collections.Counter();arms=collections.defaultdict(list)
def framecheck(f,w,arm):
 er=[];doc=sha(clausebytes(w['clauses']));b=f['binding']
 if f['version']!=3:er.append('session version')
 wants={'model_sha256':sha(modelb),'tokenizer_sha256':r['inputs']['tokenizer_derived'],'world_id':w['id'],'world_version':w['version'],'doc_sha256':doc,'control_sha256':sha(compact(control(arm))),'max_vocab':VOCAB,'eos':EOS}
 for k,v in wants.items():
  if b[k]!=v:er.append('binding '+k)
 if f['goal']!=f['query']['goal']:er.append('query goal')
 if f['reads']!=len(f['visited'])or len({tuple(v)for v in f['visited']})!=len(f['visited']):er.append('visited history')
 cap=f['captured']
 if cap:
  c=next((c for c in w['clauses']if c['seg']==cap['segment']),None)
  if c is None or c['tokens'][cap['start']:cap['start']+cap['len']]!=f['captured_tokens']:er.append('source payload')
  if cap['doc_sha256']!=doc or cap['world_id']!=w['id']or cap['world_version']!=w['version']:er.append('capture identity')
  if f['captured_span']!=[cap['segment'],cap['start'],cap['len']]or f['visited'][-1]!=f['captured_span']:er.append('capture extent')
  if c and cap['key']!=lexical(c,cap['start'],cap['len']):er.append('capture key')
 elif f['reads']or f['captured_tokens']or f['captured_span']is not None:er.append('missing capture')
 want=f['captured_tokens'][:f['cursor']]+([EOS]if f['terminal']=='Stop'else[])
 if f['emitted']!=want:er.append('emitted prefix/EOS')
 if f['terminal']=='Stop'and(not f['captured_tokens']or f['cursor']!=len(f['captured_tokens'])):er.append('incomplete Stop')
 if f['terminal']in['Unresolved','Exhausted']and f['emitted']:er.append('failed emission')
 return er

def outcomecheck(row,w,arm,tag):
 es=row['events'];emitted=[];selected=[];roles=[];attempts=[];er=[]
 for i,e in enumerate(es):
  b,a,eff=e['before'],e['after'],e['effect'];er.extend([i,'before',x]for x in framecheck(b,w,arm));er.extend([i,'after',x]for x in framecheck(a,w,arm))
  if i and b!=es[i-1]['after']:er.append([i,'continuity'])
  if b['goal']!=a['goal']:er.append([i,'goal mutation'])
  if a['reads']>b['reads']:
   selected.append(a['captured']['segment']);roles.append(eff['selected_role']);c=next(c for c in w['clauses']if c['seg']==a['captured']['segment']);keytypes['lexical_capture'if aligned(c)else'token_fallback_capture']+=1
   if eff['action']!='Read'or a['reads']!=b['reads']+1 or eff['selected_segment']!=a['captured']['segment']:er.append([i,'read effect'])
  elif eff['action']=='Read'and eff['selected_segment']is not None:attempts.append(eff['selected_segment'])
  if eff['emitted']is not None:emitted.append(eff['emitted'])
  if eff['terminal']!=a['terminal']:er.append([i,'terminal effect'])
  if eff['next_action']!=(a['pending']if a['terminal']is None else None):er.append([i,'next action'])
 if emitted!=row['emitted']or len(selected)!=row['reads']or es[-1]['after']!=row['final_frame']:er.append(['aggregate output'])
 if selected!=row['selected_segments']or roles!=row['selected_roles']:er.append(['selected refs'])
 if er:errors.append({'tag':tag,'errors':er})
 seen=[]
 for s in row['snapshots']:
  i=s['checkpoint_index'];seen.append(i);f=json.loads(s['snapshot_utf8']);se=[];phases['terminal:'+f['terminal']if f['terminal']else f['pending']]+=1
  if sha(s['snapshot_utf8'].encode())!=s['snapshot_sha256']:se.append('SHA')
  if f!=(es[i]['before']if i<len(es)else row['final_frame']):se.append('boundary')
  if s['resumed_suffix']!=es[i:]or s['resumed_final_frame']!=row['final_frame']or not s['identical']:se.append('continuation')
  se.extend(framecheck(f,w,arm))
  if se:snaperrors.append({'tag':tag,'index':i,'errors':se})
 if seen!=list(range(len(es)+1))or row['resume_identical']!=len(seen):snaperrors.append({'tag':tag,'errors':['coverage']})
 counts.update({'outcomes':1,'events':len(es),'snapshots':len(seen),'successful_reads':len(selected),'rejected_read_attempts':len(attempts)})


failures=[];clauseerrors=[]
for w in worlds.values():
 for c in w['clauses']:
  if decode(c['tokens'])!=c['text']:clauseerrors.append([w['id'],c['seg'],'token/text mismatch'])
  if [len(decode([t]).encode())for t in c['tokens']]!=c['byte_lengths']:clauseerrors.append([w['id'],c['seg'],'decoded byte lengths'])
for row in rows:
 w=worlds[row['world']];arm=row['arm'];tag=[row['split'],arm,row['world'],row['person'],row['goal']];ex=expected(w,row['person'],row['goal']);cx=expected(w,row['person'],row['goal'],arm);ok=row['emitted']==ex['answer']+[EOS]and row['terminal']=='Stop';want=cx['answer']+([EOS]if cx['terminal']=='Stop'else[])
 arms[row['split']+'/'+arm].append({'correct':ok,'depth':row['reads']==ex['reads'],'reported_flag':ok==row['correct'],'oracle':row['independent_expected']==ex and row['expected_answer']==ex['answer']and row['expected_depth']==ex['reads'],'decoded':row['emitted_text_without_eos']==decode(row['emitted']),'control_semantics':row['emitted']==want and row['terminal']==cx['terminal']and row['reads']==cx['reads']and row['selected_segments']==cx['selected_segments']})
 outcomecheck(row,w,arm,tag)
 if arm=='contextual_primary'and not ok:failures.append(tag)
 # Every request is the known named subject followed by the observed cue, with a lexical key.
 if row['events'][0]['before']['query']['key']!=[1]+list(decode(persons[row['person']]).strip().encode()):errors.append({'tag':tag,'errors':['initial request lexical key']})
out['arms']={k:{'rows':len(xs),'complete':sum(x['correct']for x in xs),'depth_correct':sum(x['depth']for x in xs),'flag_oracle_decode_control_match':all(x['reported_flag']and x['oracle']and x['decoded']and x['control_semantics']for x in xs)}for k,xs in arms.items()};out['primary_failures']=failures;out['rows_counts']=dict(counts);out['row_capture_key_modes']=dict(keytypes)
out['alignment']={'statement_clauses':48,'aligned_statements':sum(aligned(c)for w in worlds.values()for c in w['clauses']),'fit_clauses':len(wd['fit_clauses']),'aligned_fit_clauses':sum(aligned(c)for c in wd['fit_clauses']),'aligned_row_questions':sum(aligned(row['question'])for row in rows),'token_text_length_errors':clauseerrors,'cross_BPE_boundary_join_panel':'NOT_RUN: this exposed fixture retains initial spaces; focused unit coverage is separate','identity_tag_observation':'All successful captures in this panel use tag1 lexical bytes. Tag0 fallback disjointness is a source/unit-test property, not evidence of mixed-regime joins.'}
out['learning']={'reported':r['learning'],'span_feature_keys':len(model['segment_weights']),'nonzero_span_feature_keys':sum(any(v)for _,v in model['segment_weights']),'cue_feature_keys':len(model['feature_weights']),'scope':'Reported training extent0→48 and cue-role48/48 are distinct quantities; all downstream contextual outputs are independently checked, no decoder execution by auditor.'}
ints=r['interventions'];details={};oracle_errors=[];base=ints['one_source_object_span_edit'];w0=worlds[100]
checks=[('base',base['before'],base['before_clauses'],base['before_expected'],'Office'),('source_edit',base['after'],base['after_clauses'],base['after_expected'],'Office'),('cycle',ints['cycle']['outcome'],ints['cycle']['clauses'],ints['cycle']['expected'],'Office'),('required_terminal_fact_removed',ints['required_terminal_fact_removed']['outcome'],ints['required_terminal_fact_removed']['clauses'],ints['required_terminal_fact_removed']['expected'],'Office'),('request_goal_change',ints['request_goal_change']['outcome'],w0['clauses'],ints['request_goal_change']['expected'],'Project')]
for name,row,cs,saved_expected,goal in checks:
 labels=copy.deepcopy(w0['labels_for_evaluation_only']);byseg={c['seg']:c for c in cs};labels=[l for l in labels if l['seg']in byseg]
 if name in ['source_edit','cycle']:
  l=next(l for l in labels if l['seg']==0);start=l['object'][0];l['object']=[start,len(byseg[0]['tokens'])-start]
 w={'id':100,'version':300,'clauses':cs,'labels_for_evaluation_only':labels};ex=expected(w,0,goal);want=ex['answer']+([EOS]if ex['terminal']=='Stop'else[])
 coherent=all(aligned(c)and decode(c['tokens'])==c['text']and c['byte_lengths']==[len(decode([t]).encode())for t in c['tokens']]for c in cs)
 outcomecheck(row,w,'contextual_primary',name)
 match=ex==saved_expected and row['emitted']==want and row['reads']==ex['reads']and row['terminal']==ex['terminal']and row['selected_segments']==ex['selected_segments']
 if not match or not coherent:oracle_errors.append(name)
 details[name]={'expected':ex,'saved_expected_matches':ex==saved_expected,'actual':{k:row[k]for k in ['emitted','reads','selected_segments','terminal']},'decoded_answer_without_eos':decode(row['emitted']),'coherent_observed_clauses':coherent,'output_path_depth_matches':match}
changed=[c['seg']for c,d in zip(base['before_clauses'],base['after_clauses'])if c!=d]
bc,ac=base['before_clauses'][0],base['after_clauses'][0];start=w0['labels_for_evaluation_only'][0]['object'][0]
valid_edit=changed==[0]and bc['tokens'][:start]==ac['tokens'][:start]and base['replacement']=='Oren'and decode(ac['tokens'][start:]).strip()=='Oren'
out['interventions']={'reported_checks':r['independent_checks'],'all_expected':r['interventions_all_expected'],'details':details,'oracle_errors':oracle_errors,'exact_single_object_edit':valid_edit,'different_successful_answer':details['base']['actual']['terminal']=='Stop'and details['source_edit']['actual']['terminal']=='Stop'and details['base']['actual']['emitted']!=details['source_edit']['actual']['emitted'],'subword_perturbation':ints['subword_order_perturbation'],'scope':'Word-piece permutation is a rejected observed request, not semantic generalization evidence.'}
out['all_saved_counts_including_interventions']=dict(counts);out['phase_counts']=dict(phases);out['frame_errors']=errors;out['snapshot_errors']=snaperrors
origrows=[json.loads(x)for x in(BASE/'structured-argument-binding-5/rows.jsonl').read_text().splitlines()];original={(x['split'],x['arm'],x['world'],x['person'],x['goal']):x for x in origrows};diff=[]
for row in rows:
 k=(row['split'],row['arm'],row['world'],row['person'],row['goal']);o=original[k]
 if any(row[x]!=o[x]for x in['emitted','reads','terminal','selected_segments','selected_roles']):diff.append({'key':k,'before':{x:o[x]for x in['emitted','reads','terminal','selected_segments','selected_roles','correct']},'after':{x:row[x]for x in['emitted','reads','terminal','selected_segments','selected_roles','correct']}})
out['original_comparison']={'row_count':len(rows),'behavior_changes':diff,'original_root_preserved':str(BASE/'structured-argument-binding-5'),'corrected_model_differs':modelb!=(BASE/'structured-argument-binding-5/artifacts/contextual_roles_model.json').read_bytes()}
out['reported_counts_match']={a['split']+'/'+a['arm']: all(out['arms'][a['split']+'/'+a['arm']][k]==a[v]for k,v in [('rows','total'),('complete','complete'),('depth_correct','depth_correct')])for a in r['arms']}
out['all_material_checks_pass']=all(out['reported_counts_match'].values()) and not errors and not snaperrors and not clauseerrors and not failures and not oracle_errors and valid_edit and all(out['source']['actual_source_matches'].values())and out['source']['executable_matches']and model['version']==5 and all(not out['seal'][k]for k in['missing','unlisted','size_errors','blake3_errors'])and all(v['flag_oracle_decode_control_match']for v in out['arms'].values())and all(r['independent_checks'].values())
out['qualification']='Corrected exposed regression and causal instrumentation pass. Richer ordinary-language fixture, matched H4 comparison and genuinely new final qualification remain NOT_RUN.'
P('/tmp/uor-pr1344-corrected-audit.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'pass':out['all_material_checks_pass'],'arms':out['arms'],'counts':out['all_saved_counts_including_interventions'],'frame_errors':len(errors),'snapshot_errors':len(snaperrors),'oracle_errors':oracle_errors,'behavior_changes':len(diff),'output':'/tmp/uor-pr1344-corrected-audit.json'},indent=2))
