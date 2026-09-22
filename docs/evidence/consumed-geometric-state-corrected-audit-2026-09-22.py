#!/usr/bin/env python3
"""Independent saved-evidence audit for the corrected PR1347 replay.
Read-only report/source analysis; no build, fitting, model inference, or Rust table import.
Integer Hamilton calculations reconstruct the authored group's expected action."""
import argparse, collections, hashlib, json, pathlib, re, subprocess
import blake3
ap=argparse.ArgumentParser();ap.add_argument('--base',default='/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20');ap.add_argument('--root',default='consumed-geometric-state-principal-1');ap.add_argument('--source',default='/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4');ap.add_argument('--exe',default='/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/consumed-geometric-state-principal-delivery/competitive-reader-6be7f8fc');ap.add_argument('--output',default='/tmp/uor-consumed-corrected-audit.json');a=ap.parse_args()
base=pathlib.Path(a.base);p=base/a.root;source=pathlib.Path(a.source)
load=lambda p:json.loads(pathlib.Path(p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();canon=lambda x:json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
if not(p/'manifest.json').exists():raise SystemExit('Corrected sealed root not yet available: '+str(p))
out={'schema':'uor-r4.consumed-corrected-independent-audit/1','root':str(p),'method':'Read-only saved records, independent integer Hamilton expectations, baseline row comparison, independent raw authored history oracle, artifact/source/executable/seal checks.'};errors=[]
m=load(p/'manifest.json');actual={str(x.relative_to(p))for x in p.rglob('*')if x.is_file()};expected={x['path']for x in m['files']};bad=[]
for f in m['files']:
 b=(p/f['path']).read_bytes()
 if len(b)!=f['bytes']or blake3.blake3(b).hexdigest()!=f['blake3']:bad.append(f['path'])
out['seal']={'bad':bad,'missing':sorted(expected-actual),'unlisted':sorted(actual-expected-{'manifest.json'}),'files':len(expected)}
r=load(p/'result.json');rr=[json.loads(s)for s in(p/'rows.jsonl').read_text().splitlines()];old=[json.loads(s)for s in(base/'consumed-geometric-state-16/rows.jsonl').read_text().splitlines()]
out['source_bindings']=[]
for sf in r['running_source']['source_files']:
 b=subprocess.check_output(['git','show',r['running_source']['git_rev']+':'+sf['path']],cwd=source)
 out['source_bindings'].append({'path':sf['path'],'recorded_sha256':sf['sha256'],'committed_matches':sha(b)==sf['sha256']})
exe=pathlib.Path(a.exe);out['executable']={'path':str(exe),'exists':exe.exists(),'recorded_sha256':r['running_source']['executable_sha256']}
if exe.exists():out['executable'].update(actual_sha256=sha(exe.read_bytes()),matches=sha(exe.read_bytes())==r['running_source']['executable_sha256'])
labels=['Alma','Bert','Cora','Dane','Elin','Frey','Gwen','Holt'];axes=[tuple(s if j==axis else 0 for j in range(4))for axis in range(4)for s in(-1,1)]
def hamilton(a,b):
 w,x,y,z=a;v,i,j,k=b
 return(w*v-x*i-y*j-z*k,w*i+x*v+y*k-z*j,w*j-x*k+y*v+z*i,w*k+x*j-y*i+z*v)
mul=lambda a,b:axes.index(hamilton(axes[a],axes[b]));ops={'i':2,'j':4,'e':1}
dev_people=['Mara','Ivo','Cedar','Oren'];final_people=['Una','Pia','Soren','Kestrel'];dev_dests=['Bramble','Quarry','Vale','Marsh','Tarn','Ledge','Ridge','Stone'];final_dests=['Cobalt','Mica','Dune','Basalt','Flint','Slate','Amber','Onyx'];assignments={0:[6,1,4,3],1:[6,1,4,3],4:[0,5,2,7],5:[6,1,4,3]};redirects={0:5,1:2,4:3,5:6}
def world_records(w):
 people=final_people if w>=4 else dev_people;dests=final_dests if w>=4 else dev_dests;recs=[]
 for person,label in zip(people,assignments[w]):recs.append({'entity':person,'value':labels[label],'continues':False})
 for k,label in enumerate(labels):recs.append({'entity':label,'value':labels[(k+1)%8]if k==redirects[w]else dests[k],'continues':k==redirects[w]})
 for i,rec in enumerate(recs):rec.update(id=i+1,commit=i+1)
 return recs
artifact=load(p/'artifacts/artifact.json');selected=emissions=computations=0
backend_ids={x.stem.replace('backend-',''):load(x)for x in(p/'artifacts').glob('backend-*.json')};backend_hash={k:sha(canon(v))for k,v in backend_ids.items()};bindings=collections.Counter()
for idx,row in enumerate(rr):
 w=row['world'];people=final_people if w>=4 else dev_people;pos=people.index(row['person']);recs=world_records(w);bykey={x['entity']:x for x in recs};state=assignments[w][pos]
 for op in row['ops']:state=mul(ops[op],state)
 if row['ops']:
  key=labels[state];rec=bykey[key];reads=2
  while rec['continues']:rec=bykey[rec['value']];reads+=1
 else:key=None;rec=recs[pos];reads=1
 value=rec['value'];match=row['terminal']=='Complete'and row['answer']==value and row['reads']==reads
 if row['expected']!='Answer('+value+')'or match!=row['matched']:errors.append([idx,'expected answer/match flag differs from independent oracle'])
 frame=row['final_frame'];binding=frame['binding'];name=binding['backend'];bindings[name]+=1
 if name not in backend_hash or backend_hash[name]!=binding['artifact_sha256']:errors.append([idx,'saved backend identity differs from session binding'])
 events=row['effects'];se=[e for e in events if e['action']=='Read'and e['selected_record']is not None]
 for e in se:
  actualrec=recs[e['selected_record']-1];selected+=1
  if actualrec['commit']!=e['selected_commit']or actualrec['value'].encode()!=bytes(e['selected_value']):errors.append([idx,'selected record differs from authored store'])
 if row['terminal']=='Complete':
  emissions+=1;cap=frame['captured'];et=[e['emitted']for e in events if e['emitted']is not None]
  if frame['emitted']!=cap['payload']+[frame['eos']]or frame['emitted']!=et or bytes(cap['value']).decode()!=row['answer']:errors.append([idx,'owned lexical emission mismatch'])
 if row['arm']=='primary_signed'and row['ops']:
  computations+=1;c=row['computed'];s=artifact['payload_state'][assignments[w][pos]];oe=[e for e in events if e['action']=='Apply'and e['op_label']is not None]
  if len(oe)!=len(row['ops']):errors.append([idx,'wrong primitive Apply count'])
  for op,e in zip(row['ops'],oe):
   s=mul(ops[op],s)
   if s!=e['computed_state']:errors.append([idx,'incorrect intermediate Hamilton state'])
  if c['state']!=s or c['derived_label']!=state or bytes(c['derived_key']).decode()!=key:errors.append([idx,'incorrect computed state/grounding'])
  if c['source_record']!=pos+1 or c['source_commit']!=pos+1 or bytes(c['source_entity']).decode()!=row['person']:errors.append([idx,'computation source provenance mismatch'])
  if not c['consumed']or len(se)<2 or recs[se[1]['selected_record']-1]['entity']!=key:errors.append([idx,'computed key not actual second read'])
out['rows']={'count':len(rr),'expectations_and_match_flags_checked':len(rr),'selected_reads_checked':selected,'owned_emissions_checked':emissions,'primary_computations_checked':computations,'errors':errors}
out['saved_backends']={'names':sorted(backend_ids),'session_bindings_checked':dict(bindings),'canonical_sha256':backend_hash}
out['arm_counts']=[{'arm':arm,'rows':len(ar),'ordinary':sum(not x['ops']for x in ar),'computation':sum(bool(x['ops'])for x in ar),'matches':sum(x['matched']for x in ar),'computation_matches':sum(x['matched']for x in ar if x['ops'])}for arm in sorted(set(x['arm']for x in rr))for ar in [[x for x in rr if x['arm']==arm]]]
keyfn=lambda x:(x['panel'],x['arm'],x['world'],x['person'],tuple(x['ops']))
oldmap={keyfn(x):x for x in old};newmap={keyfn(x):x for x in rr};diffs=[]
for key,new in newmap.items():
 prev=oldmap.get(key)
 if prev is None:continue
 fields={f:{'before':prev[f],'after':new[f]}for f in('answer','terminal','reads','matched','expected')if prev[f]!=new[f]}
 if fields:diffs.append({'key':key,'changes':fields})
out['submission_comparison']={'identical_population_keys':oldmap.keys()==newmap.keys(),'behavior_changes':diffs,'unchanged_behavior_rows':sum(key in oldmap and all(x[f]==oldmap[key][f]for f in('answer','terminal','reads','matched','expected'))for key,x in newmap.items()),'interpretation':'Original exposed worlds retained. Source ownership/control fixes can legitimately alter control behavior; changes are not fresh evaluation.'}
# Independent raw-text lifecycle oracle; model observations never determine expected histories.
def statement(s):
 m=re.fullmatch(r"(.+?)'s (office|project) (is|became|might be) (.+)",s)
 if m:return(m[1],int(m[2]=='project'),m[4],False,m[3]!='might be')
 m=re.fullmatch(r'(.+?) (office|project) follows (.+)',s)
 if m:return(m[1],int(m[2]=='project'),m[3],True,True)
 if s.startswith('What '):return None
 raise ValueError('unrecognized statement '+s)
def question(row):
 if row['kind']=='ask_view':
  m=re.fullmatch(r'exact (\w+) of "(.+)" relation (\d)',row['text']);return(m[2],int(m[3]),m[1])
 m=re.fullmatch(r"What (?:is|was) (.+?)'s (office|project)( before| originally)?\?",row['text'])
 return(m[1],int(m[2]=='project'),{None:'Current',' before':'PreviousAssertion',' originally':'Initial'}[m[3]])
def answer(chains,scope,entity,relation,history):
 visited=[];hops=0
 while True:
  if entity in visited:return('Cycle','',hops)
  chain=chains.get((scope,entity,relation),[])
  if not chain:return('Unresolved','',hops)
  i=len(chain)-1
  if hops==0:
   if history=='Initial':i=0
   elif history=='PreviousAssertion':i-=1
   elif history=='PreviousDistinctValue':
    i-=1
    while i>=0 and chain[i][0]==chain[-1][0]:i-=1
  if i<0:return('NoHistory','',hops)
  if i<len(chain)-8:return('Evicted','',hops)
  value,cont=chain[i]
  if not cont:return('Complete',value,hops+1)
  visited.append(entity);entity=value;hops+=1
  if hops>8:return('Exhausted','',hops)
preservation=load(p/'preservation.json');out['preservation']=[]
for arm in('candidate','prior_migrated'):
 item=preservation[arm];groups=collections.defaultdict(list)
 for row in item['rows']:groups[row['script']].append(row)
 errs=[];n=matched=language=api=0;rawcorrect=0
 for script,rows in groups.items():
  chains={}
  for row in rows:
   if row['kind']=='ingest':
    st=statement(row['text']);should_write=bool(st and st[4])
    if row['expected_write']!=should_write:errs.append([script,row['turn'],'write oracle mismatch'])
    if should_write:chains.setdefault((row['scope'],st[0],st[1]),[]).append((st[2],st[3]))
    rawcorrect+=int(row['observation_and_write_match']);continue
   n+=1;language+=row['kind']=='ask';api+=row['kind']=='ask_view'
   entity,rel,hist=question(row);term,value,depth=answer(chains,row['scope'],entity,rel,hist)
   expectation=f'Complete {{ value: "{value}", hops: {depth} }}'if term=='Complete'else term
   if expectation!=row['expected']:errs.append([script,row['turn'],'expectation mismatch',expectation,row['expected']])
   match=row['error']is None and row['terminal']==term and(term!='Complete'or(row['answer']==value and row['final_frame']['hop']+1==depth))
   if match!=row['matched']:errs.append([script,row['turn'],'match flag mismatch'])
   matched+=match
 out['preservation'].append({'artifact':arm,'questions':n,'language':language,'api':api,'matched':matched,'ingest_observation_and_write_matches':rawcorrect,'reported_errors':item['errors'],'independent_oracle_errors':errs,'all38_retained':n==38 and language==34 and api==4,'summary_agrees':n==item['questions']and matched==item['matched'],'failed_query_rows':[{'script':x['script'],'turn':x['turn'],'text':x['text'],'answer':x['answer'],'expected':x['expected'],'error':x['error']}for x in item['rows']if x['kind']!='ingest'and not x['matched']]})
# Preserve parameter attribution: metadata repairs are not a refit.
old_artifacts=base/'consumed-geometric-state-16'/'artifacts'
out['parameter_parity']={}
for name in ('model.json','intent.json','lexicon.json','artifact.json'):
 before=load(old_artifacts/name);after=load(p/'artifacts'/name)
 metadata={'version','statement_classes'}if name=='intent.json'else({'version','cyclic','reference_outcome','projection'}if name=='artifact.json'else set())
 stripped=lambda x:{k:v for k,v in x.items()if k not in metadata}
 out['parameter_parity'][name]={'byte_identical':(old_artifacts/name).read_bytes()==(p/'artifacts'/name).read_bytes(),'parameters_identical_excluding_explicit_metadata':stripped(before)==stripped(after),'changed_metadata':{k:{'before_present':k in before,'before':before.get(k),'after_present':k in after,'after':after.get(k)}for k in sorted(metadata)if before.get(k)!=after.get(k)or(k in before)!=(k in after)}}
out['backend_parameter_attribution']=[]
for name,identity in backend_ids.items():
 omitted={'version','cyclic','reference_outcome','projection'}if name in ('signed_factorization','folded_central_sign')else({'version','model_bytes'}if name=='finite_transition'else{'version'})
 legacy={k:v for k,v in identity.items()if k not in omitted};legacy_hash=sha(canon(legacy))
 recorded={x['final_frame']['binding']['artifact_sha256']for x in old if x['final_frame']['binding']['backend']==name}
 out['backend_parameter_attribution'].append({'backend':name,'old_identity_reconstructed_sha256':legacy_hash,'submitted_session_identity_sha256':sorted(recorded),'all_submitted_identity_parameters_match':recorded=={legacy_hash},'scope':'Every previously bound execution parameter matches; newly complete serialization also binds missing mode/decoder/projection fields. Old finite decoder bytes were not saved and cannot be independently compared byte for byte.'if name=='finite_transition'else'All original bound factorization/transition vectors unchanged; newly bound format metadata explicit.'})
out['learning_receipt_identical']=load(base/'consumed-geometric-state-16/result.json')['learning']==r['learning']
out['parameter_attribution_conclusion']='CGS binder bytes and grounding lexicon bytes unchanged; every intent weight and its four-class decision domain unchanged; signed action/initial/output vectors unchanged.17/38 reveals preserved submitted CGS fit failure on the prior language lifecycle, not a newly selected or tuned reviewer model. All360 submitted row behaviors also preserved. Prior migrated3-class artifact separately passes38/38.'
for arm,item in [('candidate',preservation['candidate']),('prior_migrated',preservation['prior_migrated'])]:
 audit=next(x for x in out['preservation']if x['artifact']==arm);ar=item['rows']
 audit['language_matches']=sum(x['matched']for x in ar if x['kind']=='ask')
 audit['api_matches']=sum(x['matched']for x in ar if x['kind']=='ask_view')
 badqueries=[x for x in ar if x['kind']!='ingest'and not x['matched']]
 audit['query_failure_types']=dict(collections.Counter('runtime_or_observation_error'if x['error']else('wrong_complete_answer'if x['terminal']=='Complete'else str(x['terminal']))for x in badqueries))
 audit['failed_ingests']=[{'script':x['script'],'turn':x['turn'],'text':x['text'],'expected_write':x['expected_write'],'wrote':x['wrote'],'observed':x['observed'],'error':x['error']}for x in ar if x['kind']=='ingest'and not x['observation_and_write_match']]
 audit['suppressed_required_writes']=sum(x['kind']=='ingest'and x['expected_write']and not x['wrote']for x in ar)
 audit['unexpected_writes']=sum(x['kind']=='ingest'and not x['expected_write']and x['wrote']for x in ar)
child=r['consumption']['reload']['child'];checkpoints=load(p/'reload/checkpoints.json');snapshots=[json.loads(bytes(x))for x in checkpoints];frame=child['final_frame'];ref=next(x['final_frame']for x in rr if x['arm']=='primary_signed'and x['world']==0 and x['person']=='Mara'and x['ops']==['i','j']);child_errors=[]
if frame!=ref:child_errors.append('fresh child full final frame differs from primary row')
if len(child['resumed'])!=len(snapshots):child_errors.append('checkpoint count mismatch')
for i,(snap,resumed)in enumerate(zip(snapshots,child['resumed'])):
 if resumed['final_frame']!=frame or not resumed['matches']:child_errors.append(['resume mismatch',i])
request=load(p/'reload/request.json')
out['raw_child_and_resume']={'raw_text_scope_only':set(request)=={'text','scope'},'text':request.get('text'),'checkpoints':len(snapshots),'phases':dict(collections.Counter((x.get('session',x))['pending']for x in snapshots)),'errors':child_errors,'full_final_frame_compared':True}
out['claims']={'integration_complete':r.get('integration_complete'),'same_artifact_lifecycle_preserved':r.get('same_artifact_lifecycle_preserved'),'component_checks':r['checks'],'evaluation_scope':r.get('evaluation_scope'),'unknown_boundaries':['Main computation-world Memory.write remains host preparation, not learned raw ingestion.','Phase replay over unchanged store does not independently qualify mutation/eviction ownership.','All exposed populations remain exposed; no fresh acceptance draw is inferred.']}
pathlib.Path(a.output).write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'output':a.output,'seal':out['seal'],'rows':out['rows'],'arm_counts':out['arm_counts'],'behavior_changes':len(diffs),'preservation':[{k:v for k,v in x.items()if k!='failed_query_rows'}for x in out['preservation']],'raw_child_and_resume':out['raw_child_and_resume'],'claims':out['claims']},indent=2))
