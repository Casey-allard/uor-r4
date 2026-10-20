#!/usr/bin/env python3
"""Read-only independent evidence audit; only --output is written.
Uses raw authored strings to reconstruct expected semantics, stored bytes for
selected-record/emission ownership, BLAKE3 for complete seals, and git byte hashes.
This is evidence analysis, not a model implementation or additional evaluation draw.
"""
import argparse, pathlib, json, hashlib, collections, subprocess, re
import blake3
ap=argparse.ArgumentParser()
ap.add_argument('--base',default='/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20')
ap.add_argument('--source',default='/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4')
ap.add_argument('--root',default='scoped-memory-principal-4')
ap.add_argument('--output',default='/tmp/uor-consumed-scoped-audit.json')
a=ap.parse_args();base=pathlib.Path(a.base);root=base/a.root;wt=pathlib.Path(a.source);old=base/'scoped-correction-memory-14'
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(p.read_text())
rows=lambda p:[json.loads(l) for l in (p/'rows.jsonl').read_text().splitlines()]
r=load(root/'result.json');rr=rows(root);original=rows(old)
out={'schema':'uor-r4.consumed-scoped-regression-audit/1','root':str(root),'method':'Read-only audit of report, raw authored assertions/questions, learned artifacts and source/executable bytes; no model execution','seals':[]}
for name in ['scoped-memory-principal-2',a.root]:
 p=base/name;m=load(p/'manifest.json');actual={str(x.relative_to(p)) for x in p.rglob('*') if x.is_file()};listed={x['path'] for x in m['files']}
 out['seals'].append({'root':name,'listed_files':len(listed),'bytes':sum(x['bytes'] for x in m['files']),'errors':[x['path'] for x in m['files'] if (len((p/x['path']).read_bytes())!=x['bytes'] or blake3.blake3((p/x['path']).read_bytes()).hexdigest()!=x['blake3'])],'missing':sorted(listed-actual),'unlisted':sorted(actual-listed-{'manifest.json'})})
out['provenance']={'git_rev':r['running_source']['git_rev'],'git_dirty':r['running_source']['git_dirty'],'source':[]}
for f in r['running_source']['source_files']:
 b=subprocess.check_output(['git','show',r['running_source']['git_rev']+':'+f['path']],cwd=wt)
 out['provenance']['source'].append({'path':f['path'],'recorded_sha256':f['sha256'],'commit_matches':sha(b)==f['sha256'],'working_file_matches':sha((wt/f['path']).read_bytes())==f['sha256']})
exe=base/'consumed-geometric-state-principal-delivery/competitive-reader-6be7f8fc'
out['provenance']['executable']={'path':str(exe),'actual_sha256':sha(exe.read_bytes()),'matches':sha(exe.read_bytes())==r['running_source']['executable_sha256']}
out['provenance']['root2_rows_identical']=(root/'rows.jsonl').read_bytes()==(base/'scoped-memory-principal-2/rows.jsonl').read_bytes()
out['provenance']['root2_caveat']='Caller git_dirty=false despite pending documentation edits; model source hashes match committed source. Root3 corrects caller flag to true, same binary/rows/artifacts, not another fresh evaluation.'
out['artifacts']={n:{'sha256':sha((root/f'artifacts/{n}.json').read_bytes()),'unchanged_from_submission':(root/f'artifacts/{n}.json').read_bytes()==(old/f'artifacts/{n}.json').read_bytes()} for n in ['binder','intent']}
groups=collections.defaultdict(list)
for x in rr:groups[x['control'],x['script']].append(x)
primary=[x for x in rr if x['control']=='normal' and x['script'] in ['development','exposed_regression','final'] and x['kind']!='ingest']
out['counts']={'rows':len(rr),'kinds':dict(collections.Counter(x['kind'] for x in rr)),'events':sum(len(x.get('effects',[])) for x in rr),'reads':sum(e['action']=='Read' for x in rr for e in x.get('effects',[])),'selected_reads':sum(e.get('selected_record') is not None for x in rr for e in x.get('effects',[])),'primary':{'questions':len(primary),'kinds':dict(collections.Counter(x['kind'] for x in primary)),'matches':sum(x['matched'] for x in primary),'terminal_counts':dict(collections.Counter(x['terminal'] for x in primary))}}
key=lambda x:(x['control'],x['script'],x['turn'],x['kind']);new={key(x):x for x in rr};diff=[]
fields=['text','scope','expected','matched','answer','terminal','hop','wrote','learned_intent','learned_is_question','expected_write']
for x in original:
 y=new[key(x)];delta={f:[x.get(f),y.get(f)] for f in fields if x.get(f)!=y.get(f)}
 if delta:diff.append({'key':key(x),'fields':delta})
out['original_preservation']={'compared_rows':len(original),'fields':fields,'differences':diff}
errors=[];selected=0;completed=0
for group,gg in groups.items():
 records={}
 for x in gg:
  if x['kind']=='ingest':
   if x['wrote']:records[x['outcome']['Wrote']['id']]={**x,**x['outcome']['Wrote']}
   continue
  for e in x['effects']:
   if e.get('selected_record') is None:continue
   selected+=1;rec=records.get(e['selected_record'])
   if rec is None or rec['commit']!=e['selected_commit'] or rec['value_key'].encode()!=bytes(e['selected_value']):errors.append([group,x['turn'],'selected record/value/commit'])
   if rec and rec['commit']>x['pin_view'] and group[0]!='unpinned':errors.append([group,x['turn'],'future selection'])
  cap=x['final_frame']['captured']
  if cap and x['terminal']=='Complete':
   completed+=1
   if x['final_frame']['emitted']!=cap['payload']+[x['final_frame']['eos']]:errors.append([group,x['turn'],'captured emission'])
out['ownership']={'checked_selected_reads':selected,'checked_complete_captures':completed,'errors':errors}
# Independent syntax interpretation; never use logged expected/learned intent to construct it.
def assertion(text):
 m=re.fullmatch(r"(.+?)'s (office|project) (is|became|might be) (.+)",text)
 if m:return(m[1],int(m[2]=='project'),m[4],False,m[3]!='might be')
 m=re.fullmatch(r'(.+?) (office|project) follows (.+)',text)
 if m:return(m[1],int(m[2]=='project'),m[3],True,True)
 if text.startswith('What '):return None
 raise ValueError(text)
def question(text):
 m=re.fullmatch(r"What (?:is|was) (.+?)'s (office|project)( before| originally)?\?",text)
 if m:return(m[1],int(m[2]=='project'),{None:'Current',' before':'PreviousAssertion',' originally':'Initial'}[m[3]])
 m=re.fullmatch(r'exact (\w+) of "(.+)" relation (\d)',text)
 if m:return(m[2],int(m[3]),m[1])
 raise ValueError(text)
def evaluate(chains,scope,entity,relation,history,capacity):
 visited=[];hops=0
 while True:
  if hops>=6:return('Exhausted','',hops,None)
  if entity in visited:return('Cycle','',hops,None)
  chain=chains.get((scope,entity,relation),[])
  if not chain:return('Unresolved','',hops,None)
  index=len(chain)-1
  if not hops:
   if history=='Initial':index=0
   elif history=='PreviousAssertion':index-=1
   elif history=='PreviousDistinctValue':
    index-=1
    while index>=0:
     if index<len(chain)-capacity:return('Evicted','',hops,None)
     if chain[index]['value']!=chain[-1]['value']:break
     index-=1
  if index<0:return('NoHistory','',hops,None)
  if index<len(chain)-capacity:return('Evicted','',hops,None)
  selected=chain[index]
  if not selected['continues']:return('Complete',selected['value'],hops+1,selected)
  visited.append(entity);entity=selected['value'];hops+=1
errors=[];checked=0
for group,gg in groups.items():
 chains={};capacity=2 if group[0]=='capacity_2' else 8
 for x in gg:
  if x['kind']=='ingest':
   c=assertion(x['text'])
   if c and c[4]:chains.setdefault((x['scope'],c[0],c[1]),[]).append({'value':c[2],'continues':c[3]})
   continue
  entity,relation,history=question(x['text']);term,value,depth,_=evaluate(chains,x['scope'],entity,relation,history,capacity)
  expected=f'Complete {{ value: "{value}", hops: {depth} }}' if term=='Complete' else term
  if expected!=x['expected']:errors.append([group,x['turn'],'expected',expected,x['expected']])
  matched=x['terminal']==term and (term!='Complete' or (x['answer']==value and x['hop']+1==depth))
  if matched!=x['matched']:errors.append([group,x['turn'],'match flag'])
  checked+=1
out['independent_semantics']={'query_expectations_checked':checked,'match_flags_checked':checked,'errors':errors}
normal=[x for x in groups['normal','development'] if x['kind']!='ingest'];small=[x for x in groups['capacity_2','development'] if x['kind']!='ingest']
out['capacity']={'paired_questions':len(normal),'identical_inputs':all((x['text'],x['scope'],x['turn'])==(y['text'],y['scope'],y['turn']) for x,y in zip(normal,small)),'differences':[{'turn':x['turn'],'text':x['text'],'normal':[x['terminal'],x['answer']],'capacity2':[y['terminal'],y['answer']]} for x,y in zip(normal,small) if (x['terminal'],x['answer'])!=(y['terminal'],y['answer'])],'qualification':'These two existing questions distinguish retention-limited semantics from full history; capacity+2 eviction correctness is additionally covered by executed module/runner tests, not a new language panel.'}
out['historical_dependent']=[{'text':x['text'],'answer':x['answer'],'expected':x['expected'],'matched':x['matched'],'pin_view':x['pin_view'],'read_commits':[e['selected_commit'] for e in x['effects'] if e.get('selected_record') is not None]} for x in groups['normal','historical_dependent_review'] if x['kind']=='ask']
# Fresh child receives raw language asks plus explicitly labelled API asks. Independently evaluate
# every child answer against the serialized final store (not the original chronological questions).
queries=load(root/'reload/queries.json');store=load(root/'reload/store.json');child=r['restart_child'];chains={}
assert len(queries)==len(child['answers']), 'child answer count mismatch'
assert len(normal)==len(small), 'capacity paired population mismatch'
for rec in sorted(store['records'],key=lambda x:x['commit']):
 key=(bytes(rec['scope']).decode(),bytes(rec['entity']).decode(),rec['relation']);chains.setdefault(key,[]).append({**rec,'value':bytes(rec['value']).decode()})
child_errors=[]
for i,(q,observed) in enumerate(zip(queries,child['answers'])):
 ent,rel,hist=question(q['text']) if q['kind']=='ask' else (q['entity'],q['relation'],q['history'])
 term,value,depth,selected=evaluate(chains,q['scope'],ent,rel,hist,store['capacity'])
 expected={'terminal':term,'hop':max(0,depth-1),'selected':selected['id'] if selected else None,'emitted':selected['payload']+[r['inputs']['eos']] if selected else []}
 if observed!=expected:child_errors.append({'index':i,'expected':expected,'observed':observed})
cp=load(root/'reload/checkpoints.json')
out['restart']={'child_status':r['interventions']['save_reload']['child_status'],'queries':len(queries),'query_kinds':dict(collections.Counter(q['kind'] for q in queries)),'raw_query_semantic_fields':[i for i,q in enumerate(queries) if q['kind']=='ask' and set(q)-{'kind','scope','text'}],'independent_child_answer_errors':child_errors,'checkpoint_count':len(cp),'checkpoint_phases':dict(collections.Counter(c['session']['pending'] for c in cp)),'checkpoint_views':sorted(set(c['session']['view'] for c in cp)),'resumed_matches':sum(c['matches'] for c in child['resumed']),'resumed_exact_answer_matches':len(cp)==len(child['resumed']) and all(x['answer']==y['expected'] for x,y in zip(child['resumed'],cp)),'post_restart_write':child['post_restart']['wrote'],'post_restart_answer':child['post_restart']['answer'],'post_restart_view':child['post_restart']['frame']['view'],'source_review':'scm_reload_check loads model/intent/store/tokenizer; raw ask calls runtime.ask, learned correction calls ingest, every saved session calls restore/run after the correction. No entity/relation/history fields route a raw ask. Expected checkpoint answers are compared only after inference.'}
# Enumerate every committed historical Cedar view plus the two actual interleaved writes.
chains={};committed=[]
for x in groups['normal','development']:
 if x['kind']!='ingest':continue
 c=assertion(x['text'])
 if not c or not c[4]:continue
 chains.setdefault((x['scope'],c[0],c[1]),[]).append({'value':c[2],'continues':c[3]})
 committed.append(evaluate(chains,'alpha','Cedar',0,'Current',8)[:3])
for text in ["Cedar's office became Fen","Oren's office became Office Park"]:
 c=assertion(text);chains.setdefault(('alpha',c[0],c[1]),[]).append({'value':c[2],'continues':c[3]});committed.append(evaluate(chains,'alpha','Cedar',0,'Current',8)[:3])
pin=r['interventions']['pinned_view'];out['fractured_read']={'pin':pin['pin'],'pinned_answer':pin['answer'],'unpinned_answer':pin['unpinned_answer'],'all_committed_cedar_answers':committed,'last_three_committed_answers':committed[-3:],'unpinned_answer_absent_from_all_committed_cedar_views':all(term!='Complete' or value!=pin['unpinned_answer'] for term,value,depth in committed),'source_review':'After captured Cedar->Oren, source genuinely commits Cedar->Fen first, then Oren->Office Park. Pinned continuation emits old Annex, unpinned consumes old link with new terminal. Office Park is never a committed Cedar answer.'}
out['remaining_qualifications']=['Corrected exposed replay, not fresh held-out draw; original final was lexical renaming with shared fitted forms and exposed event structure.','38/38 primary means 34 learned text asks and four explicit API calls, with two expected non-answer outcomes. Child is 14 learned text asks plus two API controls against the final serialized store.','Historical continuation convention is first-hop requested history and downstream Current at the answer pin, not reconstruction of the world at the historical assertion timestamp.','Immutable-history validation, owned capture and bounded payload retention do not bound total tombstone metadata or establish authenticated adversarial tamper resistance.','No H4 predictive advantage, broad language/reasoning, whole-path D0-b or energy qualification is established.','Scope comes from a host-authenticated tag. The finite authored language surface and training forms remain narrow.']
out['pass']=all(not s['errors'] and not s['missing'] and not s['unlisted'] for s in out['seals']) and all(s['commit_matches'] and s['working_file_matches'] for s in out['provenance']['source']) and out['provenance']['executable']['matches'] and all(x['unchanged_from_submission'] for x in out['artifacts'].values()) and not diff and not out['ownership']['errors'] and not out['independent_semantics']['errors'] and not child_errors and out['restart']['resumed_exact_answer_matches'] and out['fractured_read']['unpinned_answer_absent_from_all_committed_cedar_views'] and out['provenance']['root2_rows_identical'] and out['capacity']['identical_inputs'] and not out['restart']['raw_query_semantic_fields'] and out['restart']['post_restart_write']
old_intent=load(base/'scoped-memory-principal-3/artifacts/intent.json');new_intent=load(root/'artifacts/intent.json')
out['versioned_comparison']={'old_intent_version':old_intent['version'],'new_intent_version':new_intent['version'],'new_statement_classes':new_intent['statement_classes'],'question_parameters_unchanged':old_intent['question']==new_intent['question'],'statement_parameters_unchanged_with_zero_fourth_coordinate':old_intent['statement']==[[k,v[:3]] for k,v in new_intent['statement']] and all(v[3]==0 for _,v in new_intent['statement']),'session_format':4,'legacy_byte_identity_pass':out['pass'],'interpretation':'The retained-mode replay has identical compared row behavior but explicitly different artifact/session formats. Old global pass predicate includes byte-identical old intent and serialized rows; it remains false. Semantic and provenance checks below are distinct.'}
out['semantic_provenance_verified']=all(not s['errors'] and not s['missing'] and not s['unlisted'] for s in out['seals']) and all(s['commit_matches'] and s['working_file_matches'] for s in out['provenance']['source']) and out['provenance']['executable']['matches'] and not out['original_preservation']['differences'] and not out['ownership']['errors'] and not out['independent_semantics']['errors'] and not out['restart']['independent_child_answer_errors'] and out['restart']['resumed_exact_answer_matches'] and out['fractured_read']['unpinned_answer_absent_from_all_committed_cedar_views'] and all(out['versioned_comparison'][k] for k in ['question_parameters_unchanged','statement_parameters_unchanged_with_zero_fourth_coordinate'])
pathlib.Path(a.output).write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'output':a.output,'pass':out['pass'],'counts':out['counts'],'restart':out['restart'],'fractured_read':out['fractured_read']},indent=2))
