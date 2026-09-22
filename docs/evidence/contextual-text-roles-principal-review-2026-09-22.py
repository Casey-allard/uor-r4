#!/usr/bin/env python3
"""Read-only saved-evidence audit; no model inference, fitting, Rust build, or report mutation."""
import collections,copy,hashlib,json,pathlib,re,subprocess
import blake3
P=pathlib.Path
BASE=P('/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20')
REPO=P('/Users/casey.allard/uor-r4/.worktrees/contextual-text-roles'); HEAD='bc9c8171'
sha=lambda b:hashlib.sha256(b).hexdigest(); load=lambda p:json.loads(p.read_text())
out={'schema':'uor-r4.pr1343-independent-evidence-audit/1','scope':__doc__,'submitted_head':HEAD,'roots':{}}
for n in range(1,6):
 root=BASE/f'contextual-text-roles-{n}';r=load(root/'result.json');m=load(root/'manifest.json');xs=[json.loads(x) for x in (root/'rows.jsonl').read_text().splitlines()]
 listed={f['path'] for f in m['files']};actual={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}-{'manifest.json'}
 out['roots'][root.name]={'claimed_at':load(root/'attempt.json')['claimed_at'],'sealed_at':m['sealed_at'],'listed':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'size_errors':[f['path'] for f in m['files'] if (root/f['path']).stat().st_size!=f['bytes']],'blake3_errors':[f['path'] for f in m['files'] if blake3.blake3((root/f['path']).read_bytes()).hexdigest()!=f['blake3']],'rows_sha256':sha((root/'rows.jsonl').read_bytes()),'reported_arms':r['arms'],'rows':len(xs),'running_source':r['running_source'],'source_matches_submitted':{f['path']:sha(subprocess.check_output(['git','show',HEAD+':'+f['path']],cwd=REPO))==f['sha256'] for f in r['running_source']['source_files']},'model_sha256':sha((root/'artifacts/contextual_roles_model.json').read_bytes()),'elapsed_s':r['elapsed_s'],'retained_bytes':sum(p.stat().st_size for p in root.rglob('*') if p.is_file()),'row_initial_correct':sum(x['correct'] for x in xs)}
ROOT=BASE/'contextual-text-roles-5';r=load(ROOT/'result.json');xs=[json.loads(x) for x in (ROOT/'rows.jsonl').read_text().splitlines()];model=load(ROOT/'artifacts/contextual_roles_model.json');EOS=r['task']['eos'];exe=P('/tmp/uor-pr1343-original-competitive-reader')
out['provenance']={'preserved_executable':str(exe),'executable_bytes':exe.stat().st_size,'executable_sha256':sha(exe.read_bytes()),'matches_run5':sha(exe.read_bytes())==r['running_source']['executable_sha256'],'all_run5_source_hashes_match':all(out['roots']['contextual-text-roles-5']['source_matches_submitted'].values()),'qualification':'Source-file hashes match submitted head. git_rev/git_dirty are externally supplied parent labels; roots3,4,5 executable hashes are identical, so root5 alone does not demonstrate new executable byte contents after rebase.'}
# Restricted independent BPE audit of ASCII letters/spaces only; no model implementation.
tokpath=P('/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json');raw=tokpath.read_bytes();tj=json.loads(raw);tj['model']['vocab']={k:v for k,v in tj['model']['vocab'].items() if v<4096};vocab=tj['model']['vocab'];merges=[]
for x in tj['model']['merges']:
 a,b=x.split(' ') if isinstance(x,str) else x
 if a+b in vocab:merges.append(x)
tj['model']['merges']=merges;tj['added_tokens']=[x for x in tj['added_tokens'] if x['id']<4096]
derived=json.dumps(tj,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode();ranks={tuple(x.split(' ') if isinstance(x,str) else x):i for i,x in enumerate(merges)};reverse={v:k for k,v in vocab.items()}
def encode(s):
 assert re.fullmatch(r' ?[A-Za-z]+(?: [A-Za-z]+)*',s),s
 result=[]
 for word in re.findall(r' ?[A-Za-z]+',s):
  pieces=list(word.replace(' ','Ġ'))
  while len(pieces)>1:
   opts=[(ranks[(pieces[i],pieces[i+1])],i) for i in range(len(pieces)-1) if (pieces[i],pieces[i+1]) in ranks]
   if not opts:break
   _,i=min(opts);pieces[i:i+2]=[pieces[i]+pieces[i+1]]
  result.extend(vocab[x] for x in pieces)
 return result
def decode(ts):return ''.join(reverse[t] for t in ts if t in reverse).replace('Ġ',' ')
v=r['task']['vocabulary'];names=v['persons'];offices=v['offices'];projects=v['projects'];cues=[v[x] for x in ['office_assert','office_redirect','project_assert','project_redirect']]
nameenc={s:{'plain':encode(s),'spaced':encode(' '+s)} for s in names+offices+projects};cueenc={s:encode(' '+s) for s in cues}
out['tokenizer']={'source_sha256':sha(raw),'derived_sha256':sha(derived),'derived_matches_receipt':sha(derived)==r['inputs']['tokenizer_derived'],'names':nameenc,'cues':cueenc,'saved_decodings_match':all(decode(x['emitted'])==x['emitted_text'] for x in xs),'membership_registry_mismatched_names':[s for s in names if nameenc[s]['plain']!=nameenc[s]['spaced']],'office_lower_upper':{'lower':encode(' office'),'upper':encode(' Office'),'same':encode(' office')==encode(' Office')},'project_lower_upper':{'lower':encode(' project'),'upper':encode(' Project'),'same':encode(' project')==encode(' Project')}}
MASK=(1<<64)-1
def world(wid,ver,ochains,pchains,key,seed):
 st=seed|1;clauses=[];labels=[];texts=[]
 def rand():
  nonlocal st
  st^=(st<<13)&MASK;st^=st>>7;st^=(st<<17)&MASK;return st
 for g,chains in [('Office',ochains),('Project',pchains)]:
  for chain in chains:
   targets=names if key and g=='Office' else offices if g=='Office' else projects;target=targets[rand()%len(targets)]
   for hop,p in enumerate(chain):
    last=hop+1==len(chain);role=(0 if g=='Office' else 2)+(not last);obj=target if last else names[chain[hop+1]];text=' '+names[p]+' '+cues[role]+' '+obj;sub=encode(' '+names[p]);mk=encode(' '+cues[role]);ob=encode(' '+obj);tokens=encode(text);assert tokens==sub+mk+ob
    seg=len(clauses);clauses.append({'seg':seg,'tokens':tokens});labels.append({'seg':seg,'subject':[0,len(sub)],'marker':[len(sub),len(mk)],'object':[len(sub)+len(mk),len(ob)],'role':int(role),'goal':g,'action':'Emit' if last else 'Continue'});texts.append(text)
 return {'id':wid,'version':ver,'clauses':clauses,'labels':labels,'texts':texts}
dev=[world(0,10,[[0,1],[2,3]],[[0],[1,2],[3]],False,0xC0F70001),world(1,11,[[0],[1,2],[3]],[[0,1],[2,3]],False,0xC0F70001+0x9E37),world(2,12,[[0,1],[2,3]],[[0,2],[1,3]],False,0xC0F70001+2*0x9E37)]
final=[world(100,300,[[0,1,2],[3]],[[0],[1,2,3]],False,0xC0F70101),world(101,301,[[0],[1,2,3]],[[0,1,2],[3]],False,0xC0F70101+0x9E37),world(102,302,[[0,1,2],[3]],[[0],[1,2,3]],True,0xC0F70101+2*0x9E37)]
def expected(w,p,g):
 query=encode(' '+names[p]);visited=[];roles=[];spans=[]
 for depth in range(1,7):
  found=None
  for c,l in zip(w['clauses'],w['labels']):
   ss,sl=l['subject']
   if l['goal']==g and c['tokens'][ss:ss+sl]==query:found=c,l;break
  if found is None:return {'answer':[],'terminal':'Unresolved','reads':len(visited),'segments':visited,'roles':roles,'spans':spans}
  c,l=found;os,ol=l['object'];obj=c['tokens'][os:os+ol]
  if c['seg'] in visited:return {'answer':[],'terminal':'Exhausted','reads':len(visited),'segments':visited,'roles':roles,'spans':spans}
  visited.append(c['seg']);roles.append(l['role']);spans.append([c['seg'],os,ol])
  if l['action']=='Emit':return {'answer':obj,'terminal':'Stop','reads':depth,'segments':visited,'roles':roles,'spans':spans}
  query=obj
 return {'answer':[],'terminal':'Exhausted','reads':6,'segments':visited,'roles':roles,'spans':spans}
checks=[]
for x in xs:
 ex=expected(final[x['world']-100],x['person'],x['goal']);want=ex['answer']+[EOS];checks.append({'answer':x['emitted']==want,'expected_text':x['expected_text']==decode(ex['answer']),'depth':x['reads']==x['expected_depth']==ex['reads'],'selected':x['selected']==ex['spans'],'correct_flag':x['correct']==(x['emitted']==want and x['terminal']=='Some(Stop)'),'eos':x['emitted'].count(EOS)==1 and x['emitted'][-1]==EOS})
out['fixture']={'reconstructed_development_text_matches':[[s.lstrip(' ') for s in w['texts']] for w in dev]==r['task']['dev_worlds'],'development':dev,'final':final}
out['primary_recount']={'rows':len(xs),'checks_true_counts':{k:sum(x[k] for x in checks) for k in checks[0]},'read_depth_counts':dict(collections.Counter(x['reads'] for x in xs)),'decoded_output_counts':dict(collections.Counter(x['emitted_text'] for x in xs)),'genuine_multiword_answers':sum(len(x['emitted_text'].split())>1 for x in xs),'answer_token_lengths':dict(collections.Counter(len(x['emitted'])-1 for x in xs)),'source_span_complete':sum(x['selected'] for x in checks),'all_arm_raw_rows_retained':False,'snapshot_bytes_retained':0,'events_retained':0}
# Novelty/composition counts derive from text and authored chains, not model predictions.
dstatements={s for w in dev for s in w['texts']};fstatements=[s for w in final for s in w['texts']];queries=[' '+n+' '+cues[g] for n in names for g in [0,2]]
out['novelty']={'development_unique_statements':len(dstatements),'final_unique_statements':len(set(fstatements)),'final_statement_occurrences_seen_in_development':sum(s in dstatements for s in fstatements),'final_unique_statements_seen_in_development':len(set(fstatements)&dstatements),'all_8_request_wordings_seen_in_fit_three_times':True,'development_read_counts':dict(collections.Counter(expected(w,p,g)['reads'] for w in dev for p in range(4) for g in ['Office','Project'])),'final_read_counts':dict(collections.Counter(expected(w,p,g)['reads'] for w in final for p in range(4) for g in ['Office','Project'])),'final_question_depth_vectors':[{'person':p,'goal':g,'development':[expected(w,p,g)['reads'] for w in dev],'final':[expected(w,p,g)['reads'] for w in final]} for p in range(4) for g in ['Office','Project']],'qualification':'Three-read composition absent from fit; same authored final family evaluated five times with source/model/input repairs. Roots2-5 saved rows byte-identical, so final means exposed regression, not new confirmation.'}
# Inventory actual learned weights; no scorer execution. Reconstruct uninformed tie-rule analytically.
kind_names={1:'first',2:'last',3:'left',4:'right',5:'left2',6:'right2',7:'length',8:'initial',9:'final'};weightrows=[]
for k,ws in model['feature_weights']:
 kind=k>>40;val=k&((1<<40)-1);weightrows.append({'kind':kind_names[kind],'value':val,'token_text':decode([val]) if kind<7 else None,'weights':ws})
fitlabels=[]
for w in dev:
 fitlabels.extend(w['labels'])
 for p in range(4):
  for g in [0,2]:fitlabels.append({'marker':[len(encode(' '+names[p])),len(encode(' '+cues[g]))],'role':g,'object':None,'action':'Emit'})
trueinitial=sum(l['marker']==[1,1] and l['role']==0 for l in fitlabels)
endpoint_to_role={};endpointcollisions=[]
for role,cue in enumerate(cues):
 ts=cueenc[cue];ep=(ts[0],ts[-1],len(ts));endpoint_to_role[ep]=role
allclauses=[c for w in dev+final for c in w['clauses']]+[{'tokens':encode(q)} for q in queries]
for ci,c in enumerate(allclauses):
 ts=c['tokens'];matches=[]
 for s in range(1,len(ts)):
  for le in range(1,min(4,len(ts)-s)+1):
   ep=ts[s],ts[s+le-1],le
   if ep in endpoint_to_role:matches.append((s,le,endpoint_to_role[ep]))
 if len(matches)!=1:endpointcollisions.append({'clause_index':ci,'matches':matches})
out['learning']={'reported':r['learning'],'true_uninformed_marker_role_accuracy':trueinitial,'fit_units':len(fitlabels),'uninformed_tie_rule':'all zero scores => first candidate start1,length1,role0','initial_receipt_bug':'role_initial_correct is overwritten by statement action initial_correct before constructing ObFitReport. Report14 is action baseline, not marker-role baseline.','statement_actions':dict(collections.Counter(l['action'] for l in fitlabels if l['object'] is not None)),'unique_feature_keys':len(weightrows),'nonzero_feature_keys':sum(any(x['weights']) for x in weightrows),'nonzero_scalar_coefficients':sum(v!=0 for x in weightrows for v in x['weights']),'zero_feature_keys':[x for x in weightrows if not any(x['weights'])],'feature_kind_counts':dict(collections.Counter(x['kind'] for x in weightrows)),'rows':weightrows,'ordered_local_context_necessity_established':False,'endpoint_signatures':{str(k):v for k,v in endpoint_to_role.items()},'endpoint_signature_ambiguous_clause_count':len(endpointcollisions),'endpoint_signature_ambiguities':endpointcollisions,'endpoint_scope':'Checking only first/last/length uniquely identifies the four annotated cue spans on every one of the48 statements plus8 questions. This is a static fixture collision audit, not a fitted baseline evaluation. Interior cue permutation preserving endpoints remains invisible to the entire extractor.'}
out['lexical_overlap']={'cue_word_strings_inside_name_words_case_sensitive':sorted(set(' '.join(cues).split())&set(' '.join(names+offices+projects).split())),'cue_token_ids_inside_name_token_ids':sorted(set(t for ts in cueenc.values() for t in ts)&set(t for d in nameenc.values() for t in d['spaced'])),'cedar_person_prefix_matches_cedar_annex':nameenc['Cedar']['spaced']==nameenc['Cedar Annex']['spaced'][:len(nameenc['Cedar']['spaced'])],'qualification':'Office and Project in proper names are capitalized; lowercase complete cue words do not recur literally in names. Shared subword IDs and Cedar prefix overlap are present but do not force cue-context disambiguation.'}
# Exact authored intervention reconstruction from original runner.
w=final[0];edited=copy.deepcopy(w);l=edited['labels'][0];os,ol=l['object'];replacement=encode(' Office Park');edited['clauses'][0]['tokens'][os:]=replacement;l['object']=[os,len(replacement)];removed=copy.deepcopy(w);removeidx=next(i for i,l in enumerate(w['labels']) if l['goal']=='Project' and l['action']=='Emit');del removed['clauses'][removeidx];del removed['labels'][removeidx]
before=expected(w,0,'Office');after=expected(edited,0,'Office');bpr=expected(w,0,'Project');apr=expected(removed,0,'Project')
out['interventions']={'source_edit':{'before':before,'after':after,'changed_clause_before':w['texts'][0],'changed_clause_after':decode(edited['clauses'][0]['tokens']),'expected_terminal_is_unresolved':after['terminal']=='Unresolved','saved_after':r['interventions']['one_position_source_edit']['after'],'saved_selected_spans_match_declared_extent':r['interventions']['one_position_source_edit']['after']['selected']==after['spans'],'qualification':'Edited redirect points to an office phrase with no subject record. Expected abstention is consistent, but saved object span4+7 differs from gold6+5, so the actual parse is wrong and no successful answer-change control is demonstrated.'},'removed_fact':{'segment':removeidx,'text':w['texts'][removeidx],'base_path':bpr,'removed_path':apr,'is_later_fact':len(bpr['segments'])>1},'goal_change':{'expected_office':before['answer']+[EOS],'expected_project':bpr['answer']+[EOS],'saved_project_correct':r['interventions']['request_goal_change_fixed_evidence']['project']==bpr['answer']+[EOS]},'resume':'Single inline snapshot; no snapshot bytes, resumed events or frames retained. Saved matching output is insufficient to audit phase completeness or causal continuation.','order':'Last two BPE IDs of Office question cue are swapped. One malformed observed question rejection is not a matched fitted orderless comparator or positive reordered-language example.'}
out['exposure']={'attempt_count':5,'original_primary_final_correct':[out['roots'][f'contextual-text-roles-{i}']['row_initial_correct'] for i in range(1,6)],'roots2_5_rows_identical':len({out['roots'][f'contextual-text-roles-{i}']['rows_sha256'] for i in range(2,6)})==1,'model_hashes':[out['roots'][f'contextual-text-roles-{i}']['model_sha256'] for i in range(1,6)],'fresh_final_confirmation':False}
out['control_scope']={'membership':'Registry encodes bare names but serving clauses use leading space. All four registry values differ from serving keys. Aggregate13/24 is confounded and all control rows discarded.','maximum_two_read':'18/24 agrees with six depth3 rows being beyond cap, but all individual control events/rows discarded. This is truncation control, not independently trained fixed-depth policy.','reads_disabled':'0/24 is aggregate-only.','geometry':'No H4/geometric competing arm here; finding concerns observation parser, not geometric advantage.','fitserve':'Shared candidate_features serves sparse integer role scorer. Runtime gets loaded model bytes and observed clauses/question, no explicit gold labels at serve. Gold span/role/action/goal supervision and authored clause grammar remain strong offline scaffold.'}
out['resources']={'saved_elapsed_s':sum(x['elapsed_s'] for x in out['roots'].values()),'retained_report_bytes':sum(x['retained_bytes'] for x in out['roots'].values()),'original_model_bytes':(ROOT/'artifacts/contextual_roles_model.json').stat().st_size,'model_runs_by_auditor':0,'builds_by_auditor':0,'energy':'UNAVAILABLE','cleanup':'Root investigator auditing separately.'}
out['script_sha256']=sha(P(__file__).read_bytes());dest=P('/tmp/uor-pr1343-evidence-audit.json');dest.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:out[k] for k in ['provenance','tokenizer','primary_recount','novelty','lexical_overlap','interventions','exposure','resources']},indent=2));print('LEARNING',json.dumps({k:v for k,v in out['learning'].items() if k!='rows'},indent=2))
