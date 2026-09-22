#!/usr/bin/env python3
"""Independent saved-data audit; no model fitting/inference/build. Manifest and trace arithmetic only."""
import collections, datetime, hashlib, json, pathlib, subprocess, sys
try:
 import blake3
except ImportError:
 blake3=None
P=pathlib.Path; REPO=P('/Users/casey.allard/uor-r4'); BASE=REPO/'.uor-models/realtext-prior-2026-09-20'; HEAD='9cb02f6e77eeae6d026e830f7128625c135f7e26'
load=lambda p:json.loads(p.read_text()); sha=lambda b:hashlib.sha256(b).hexdigest(); compact=lambda v:json.dumps(v,separators=(',',':'),ensure_ascii=False).encode()
out={'schema':'uor-r4.pr1344-original-independent-audit/1','scope':__doc__,'submitted':HEAD,'roots':{}}
for i in range(1,6):
 root=BASE/f'structured-argument-binding-{i}'; fs=sorted(p for p in root.rglob('*')if p.is_file()); rec={'root':str(root),'retained_bytes':sum(p.stat().st_size for p in fs),'files':[str(p.relative_to(root))for p in fs],'claimed_at':load(root/'attempt.json')['claimed_at']}
 if(root/'manifest.json').exists():
  m=load(root/'manifest.json');listed={f['path']for f in m['files']};actual={str(p.relative_to(root))for p in fs}-{'manifest.json'};r=load(root/'result.json')
  rec.update({'sealed':True,'sealed_at':m['sealed_at'],'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'size_errors':[f['path']for f in m['files']if(root/f['path']).stat().st_size!=f['bytes']],'blake3_errors':[f['path']for f in m['files']if blake3.blake3((root/f['path']).read_bytes()).hexdigest()!=f['blake3']]if blake3 else 'UNAVAILABLE: blake3 module','running_source':r['running_source'],'source_matches_submitted':{f['path']:sha(subprocess.check_output(['git','show',HEAD+':'+f['path']],cwd=REPO))==f['sha256']for f in r['running_source']['source_files']},'model_sha256':sha((root/'artifacts/contextual_roles_model.json').read_bytes()),'rows_sha256':sha((root/'rows.jsonl').read_bytes()),'elapsed_s':r['elapsed_s'],'learning':r['learning']})
 else:rec['sealed']=False
 out['roots'][str(i)]=rec
root=BASE/'structured-argument-binding-5';r=load(root/'result.json');wd=load(root/'worlds.json');rows=[json.loads(x)for x in(root/'rows.jsonl').read_text().splitlines()];modelb=(root/'artifacts/contextual_roles_model.json').read_bytes();model=json.loads(modelb);EOS=r['task']['eos'];VOCAB=4096
exe=P('/tmp/uor-pr1344-original-competitive-reader');out['executable_matches_preserved']=exe.exists() and sha(exe.read_bytes())==r['running_source']['executable_sha256'];out['running_revision_scope']='git_rev/git_dirty describe source-root git command: declared clean base conflicts with uncommitted source hashes; use recorded file and executable bytes.'
worlds={w['id']:w for split in ['development','exposed_regression']for w in wd[split]};persons=wd['membership_entities'];vocab=load(REPO/'.uor-models/sources/smollm2-135m-instruct/tokenizer.json')['model']['vocab'];rev={v:k for k,v in vocab.items()if v<VOCAB}
# Authored ASCII fixture only. GPT2 byte alphabet space is U+0120; all other used bytes are literal ASCII.
def decode(ts):return ''.join(rev[t]for t in ts if t!=EOS).replace('Ġ',' ')
def aligned(c):return bool(c['byte_lengths'])and len(c['tokens'])==len(c['byte_lengths'])and sum(c['byte_lengths'])==len(c['text'].encode())
def lexical(c,start,length):
 if aligned(c):return list(c['text'].encode()[sum(c['byte_lengths'][:start]):sum(c['byte_lengths'][:start+length])].strip(b' \t\r\n\v\f'))
 return list(b''.join(t.to_bytes(4,'little')for t in c['tokens'][start:start+length]))
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
 w=worlds[row['world']];arm=row['arm'];tag=[row['split'],arm,row['world'],row['person'],row['goal']];ex=expected(w,row['person'],row['goal']);ok=row['emitted']==ex['answer']+[EOS]and row['terminal']=='Stop'
 arms[row['split']+'/'+arm].append({'correct':ok,'depth':row['reads']==ex['reads'],'reported_flag':ok==row['correct'],'oracle':row['independent_expected']==ex and row['expected_answer']==ex['answer']and row['expected_depth']==ex['reads'],'decoded':row['emitted_text_without_eos']==decode(row['emitted'])})
 outcomecheck(row,w,arm,tag)
 if arm=='contextual_primary'and not ok:failures.append({k:row[k]for k in ['split','world','person','goal','question_text','expected_text','expected_depth','emitted_text_without_eos','reads','selected_segments','selected_roles','terminal']})
out['arms']={k:{'rows':len(xs),'complete':sum(x['correct']for x in xs),'depth_correct':sum(x['depth']for x in xs),'flag_and_oracle_and_decode_match':all(x['reported_flag']and x['oracle']and x['decoded']for x in xs)}for k,xs in arms.items()};out['primary_failures']=failures;out['rows_counts']=dict(counts);out['row_capture_key_modes']=dict(keytypes)
out['alignment']={'statement_clauses':48,'aligned_statements':sum(aligned(c)for w in worlds.values()for c in w['clauses']),'fit_clauses':len(wd['fit_clauses']),'aligned_fit_clauses':sum(aligned(c)for c in wd['fit_clauses']),'aligned_row_questions':sum(aligned(row['question'])for row in rows),'token_text_length_errors':clauseerrors,'cross_BPE_boundary_join_test':'NOT_RUN: every authored sentence/request retains initial space','summary_correction':'Aligned-byte keys are exercised. Claim of all keys falling back is false.'}
out['learning']={'reported':r['learning'],'span_feature_keys':len(model['segment_weights']),'nonzero_span_feature_keys':sum(any(v)for _,v in model['segment_weights']),'cue_feature_keys':len(model['feature_weights']),'span_assignment_accuracy_scope':'0 to 48/48 reported and distinct initial field source verified, no independent decoder execution','cue_role_accuracy':r['learning']['cue_role_correct'],'regression_reason':'Correct spans with wrong cue role on Mara project follows Ivo: learned role2 Emit instead of role3 Continue; separate cue classifier trained one pass.'}
ints=r['interventions'];details={}
for name in ['one_source_object_span_edit','cycle']:
 v=ints[name];cs=v['after_clauses']if name.startswith('one_')else v['clauses'];row=v['after']if name.startswith('one_')else v['outcome'];c=cs[0];ex=v['after_expected']if name.startswith('one_')else v['expected'];details[name]={'clause':c,'tokens_decode':decode(c['tokens']),'token_count':len(c['tokens']),'length_count':len(c['byte_lengths']),'aligned':aligned(c),'text_matches_tokens':decode(c['tokens'])==c['text'],'expected':ex,'actual':{k:row[k]for k in ['emitted','reads','selected_segments','terminal']},'initial_query_key':row['events'][0]['before']['query']['key'],'edited_subject_fallback_key':lexical(c,0,2)}
 outw={'id':100,'version':300,'clauses':cs};outcomecheck(row,outw,'contextual_primary',name)
base=ints['one_source_object_span_edit'];outcomecheck(base['before'],{'id':100,'version':300,'clauses':base['before_clauses']},'contextual_primary','intervention-base')
for name in ['required_terminal_fact_removed','request_goal_change']:
 v=ints[name];cs=v.get('clauses',base['before_clauses']);outcomecheck(v['outcome'],{'id':100,'version':300,'clauses':cs},'contextual_primary',name);details[name]={'expected':v['expected'],'actual':{k:v['outcome'][k]for k in ['emitted','reads','selected_segments','terminal']}}
out['interventions']={'reported_checks':r['independent_checks'],'all_expected':r['interventions_all_expected'],'details':details,'subword_order_perturbation':ints['subword_order_perturbation']['outcome'],'diagnosis':'Source-edit and cycle mutate tokens only, retaining stale text and byte lengths. Clause falls back to token-byte key while request stays lexical ASCII key; first subject fails to match. This is an instrument/input construction error, not evidence against geometric attention.'}
out['all_saved_counts_including_interventions']=dict(counts);out['phase_counts']=dict(phases);out['frame_errors']=errors;out['snapshot_errors']=snaperrors
out['resources']={'live_json_observed':{'cumulative_ms':311552510,'limit_ms':333700000},'preceding_review_cumulative_ms':311552510,'preceding_limit_ms':316700000,'allowance_increment_ms':17000000,'prior_run_charge_recorded_ms':0,'completed_instrument_elapsed_s':sum(out['roots'][str(i)]['elapsed_s']for i in[4,5]),'worktree_git_birth_local':'2026-09-21T22:35:01-0400','live_ledger_birth_mtime_local':'2026-09-21T22:35:54-0400','submitted_commit_local':'2026-09-21T23:01:18-0400','worktree_birth_to_commit_ms':1577000,'scope':'Missing complete prior debit. These filesystem/git wall timestamps exclude earlier recovery and later delivery; they are a partial wall-time reconstruction, not measured CPU/model time. No matching DeepSeek Codex thread found in narrow state5 lookup. No prior projection/debit file among report roots or matching /tmp names.'}
out['conclusion']={'saved_trace_internal_checks_pass':not errors and not snaperrors and not clauseerrors,'all_result_counts_match':all(v['flag_and_oracle_and_decode_match']for v in out['arms'].values()),'instrument_contract_pass':False,'new_argument_binding_milestone_pass':False,'geometry_advantage':'NOT_RUN','new_final':'NOT_RUN','recommended_repair_order':['repair coherent surface/token construction and identity representation','fit/measure cue roles separately from span extents','retain/refit semantic regression tests','then execute readable argument-placement and query fixture with equal-support H4 comparator']}
P('/tmp/uor-pr1344-original-audit.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'roots':{k:{a:v.get(a)for a in ['sealed','source_matches_submitted','blake3_errors']}for k,v in out['roots'].items()},'arms':out['arms'],'counts':out['all_saved_counts_including_interventions'],'frame_errors':len(errors),'snapshot_errors':len(snaperrors),'alignment':out['alignment'],'output':'/tmp/uor-pr1344-original-audit.json'},indent=2))
