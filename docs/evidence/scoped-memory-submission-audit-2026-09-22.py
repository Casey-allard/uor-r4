#!/usr/bin/env python3
"""Read-only independent audit of submitted scoped-correction-memory roots 1..14.
No model execution. Uses raw authored text to reconstruct reference histories,
BLAKE3 to verify seals, and source/executable byte hashes to verify receipts.
"""
import argparse, pathlib, json, hashlib, collections, re, subprocess
import blake3
ap=argparse.ArgumentParser();ap.add_argument('--base',default='/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20');ap.add_argument('--source',default='/Users/casey.allard/uor-r4/.worktrees/scoped-correction-memory');ap.add_argument('--output',default='/tmp/uor-scoped-submitted-audit.json');args=ap.parse_args()
base=pathlib.Path(args.base);wt=pathlib.Path(args.source)
def sha(b):return hashlib.sha256(b).hexdigest()
def readrows(n):return [json.loads(l) for l in (base/f'scoped-correction-memory-{n}'/'rows.jsonl').read_text().splitlines()]
out={'schema':'uor-r4.scoped-submitted-independent-audit/1','method':'Read-only files/source audit; no build or model execution','roots':[]}
for n in range(1,15):
 p=base/f'scoped-correction-memory-{n}';d={'root':n,'exists':p.exists()};out['roots'].append(d)
 if not p.exists():continue
 actual={str(x.relative_to(p)) for x in p.rglob('*') if x.is_file()};d['files']=sorted(actual)
 d['sealed']=(p/'manifest.json').exists()
 if d['sealed']:
  manifest=json.loads((p/'manifest.json').read_text());expected={f['path'] for f in manifest['files']};bad=[]
  for f in manifest['files']:
   data=(p/f['path']).read_bytes()
   if len(data)!=f['bytes'] or blake3.blake3(data).hexdigest()!=f['blake3']:bad.append(f['path'])
  d.update(bad_seal_files=bad,unlisted=sorted(actual-expected-{'manifest.json'}),missing=sorted(expected-actual),sealed_at=manifest['sealed_at'])
 d['artifacts']={name:sha((p/name).read_bytes()) for name in ('artifacts/binder.json','artifacts/intent.json') if (p/name).exists()}
 if not(p/'result.json').exists():continue
 result=json.loads((p/'result.json').read_text());rows=readrows(n);d['source']=result['running_source'];d['learning']=result['learning'];d['primary']=[x for x in result['arms'] if x['arm']=='primary'];d['rows']=len(rows);d['kind_counts']=dict(collections.Counter(x['kind'] for x in rows));d['rows_sha256']=sha((p/'rows.jsonl').read_bytes())
latest=base/'scoped-correction-memory-14';result=json.loads((latest/'result.json').read_text());rows=readrows(14)
out['source_bindings']=[]
for sf in result['running_source']['source_files']:
 committed=subprocess.check_output(['git','show',f'c4bd440a:{sf["path"]}'],cwd=wt)
 out['source_bindings'].append({'path':sf['path'],'recorded_sha256':sf['sha256'],'commit_matches':sha(committed)==sf['sha256'],'current_matches':sha((wt/sf['path']).read_bytes())==sf['sha256']})
exe=base/'scoped-memory-principal-delivery/competitive-reader-submitted-c4bd440a';out['executable']={'path':str(exe),'recorded_sha256':result['running_source']['executable_sha256'],'exists':exe.exists()}
if exe.exists():out['executable'].update(actual_sha256=sha(exe.read_bytes()),matches=sha(exe.read_bytes())==result['running_source']['executable_sha256'])
groups=collections.defaultdict(list)
for row in rows:groups[row['control'],row['script']].append(row)
out['groups']=[]
for (control,script),rr in groups.items():
 asks=[x for x in rr if x['kind'] in ('ask','ask_view')];ings=[x for x in rr if x['kind']=='ingest'];out['groups'].append({'control':control,'script':script,'rows':len(rr),'language_questions':sum(x['kind']=='ask' for x in asks),'exact_api_queries':sum(x['kind']=='ask_view' for x in asks),'matches':sum(x['matched'] for x in asks),'terminal_counts':dict(collections.Counter(x['terminal'] for x in asks)),'branch_matches':sum(x['learned_is_question']==x['declared_is_question'] for x in ings),'branch_total':len(ings),'intent_matches':sum(x['learned_intent']==x['declared_intent'] for x in ings),'no_write_matches':sum(not x['wrote'] for x in ings if not x['expected_write']),'no_write_total':sum(not x['expected_write'] for x in ings)})
out['row_counts']={'rows':len(rows),'kinds':dict(collections.Counter(x['kind'] for x in rows)),'events':sum(len(x.get('effects',[])) for x in rows),'read_events':sum(e['action']=='Read' for x in rows for e in x.get('effects',[])),'successful_read_events':sum(e.get('selected_record') is not None for x in rows for e in x.get('effects',[])),'terminal_counts':dict(collections.Counter(x['terminal'] for x in rows if x['kind']!='ingest'))}
# Verify every selected record was actually written and captured output belongs to it.
errors=[];selected=0;complete_captures=0
for group,rr in groups.items():
 records={}
 for row in rr:
  if row['kind']=='ingest' and row['wrote']:
   records[row['outcome']['Wrote']['id']]={**row,**row['outcome']['Wrote']}
  if row['kind']=='ingest':continue
  for event in row['effects']:
   if event.get('selected_record') is None:continue
   selected+=1;rec=records.get(event['selected_record'])
   if rec is None:errors.append([group,row['turn'],'record not written']);continue
   if rec['commit']!=event['selected_commit'] or rec['value_key'].encode()!=bytes(event['selected_value']):errors.append([group,row['turn'],'source value or commit mismatch'])
   if rec['commit']>row['pin_view'] and group[0]!='unpinned':errors.append([group,row['turn'],'future record'])
  frame=row['final_frame'];cap=frame['captured']
  if cap and row['terminal']=='Complete':
   complete_captures+=1
   if frame['emitted']!=cap['payload']+[frame['eos']]:errors.append([group,row['turn'],'emission not captured payload'])
out['ownership']={'checked_selected_reads':selected,'checked_complete_captures':complete_captures,'errors':errors}
# Independent authored-language parser and semantic oracle, never consume row.expected.
def clause(s):
 m=re.fullmatch(r"(.+?)'s (office|project) (is|became|might be) (.+)",s)
 if m:return (m[1],int(m[2]=='project'),m[4],False,m[3])
 m=re.fullmatch(r'(.+?) (office|project) follows (.+)',s)
 if m:return(m[1],int(m[2]=='project'),m[3],True,'is')
 if s.startswith('What '):return None
 raise ValueError(s)
def query(row):
 s=row['text']
 if row['kind']=='ask_view':
  m=re.fullmatch(r'exact (\w+) of "(.+)" relation (\d)',s);return(m[2],int(m[3]),m[1])
 m=re.fullmatch(r"What (?:is|was) (.+?)'s (office|project)( before| originally)?\?",s)
 return(m[1],int(m[2]=='project'),{None:'Current',' before':'PreviousAssertion',' originally':'Initial'}[m[3]])
def answer(chains,scope,entity,relation,history,capacity):
 visited=[];hops=0
 while True:
  if entity in visited:return ('Cycle','',hops)
  chain=chains.get((scope,entity,relation),[])
  if not chain:return('Unresolved','',hops)
  index=len(chain)-1
  if not hops:
   if history=='Initial':index=0
   elif history=='PreviousAssertion':index-=1
   elif history=='PreviousDistinctValue':
    index-=1
    while index>=0 and chain[index][0]==chain[-1][0]:index-=1
  if index<0:return('NoHistory','',hops)
  if index<len(chain)-capacity:return('Evicted','',hops)
  value,continues=chain[index]
  if not continues:return('Complete',value,hops+1)
  visited.append(entity);entity=value;hops+=1
  if hops>8:return('Exhausted','',hops)
errors=[];expected_verified=0;matches_verified=0
for group,rr in groups.items():
 chains={};capacity=2 if group[0]=='capacity_2' else 8
 for row in rr:
  if row['kind']=='ingest':
   c=clause(row['text'])
   if c and c[4]!='might be':chains.setdefault((row['scope'],c[0],c[1]),[]).append((c[2],c[3]))
   continue
  entity,relation,history=query(row);term,value,depth=answer(chains,row['scope'],entity,relation,history,capacity)
  expectation=f'Complete {{ value: "{value}", hops: {depth} }}' if term=='Complete' else term
  expected_verified+=1
  if expectation!=row['expected']:errors.append([group,row['turn'],'expected mismatch',expectation,row['expected']])
  actualmatch=row['terminal']==term and (term!='Complete' or (row['answer']==value and row['hop']+1==depth))
  matches_verified+=1
  if actualmatch!=row['matched']:errors.append([group,row['turn'],'matched flag mismatch'])
out['independent_semantics']={'rows_verified':expected_verified,'match_flags_verified':matches_verified,'errors':errors,'capacity_warning':'Independent oracle fixes repeated-oldest eviction, but submitted asks never probe the second-oldest after fourth write, so rows do not distinguish the bug.'}
# Fresh lexical population: new whole strings, existing form/control and exposed event skeleton.
training=set(result['task']['development_entities'])|set(sum(result['task']['development_values'].values(),[]));final=set(result['task']['final_entities'])|set(sum(result['task']['final_values'].values(),[]))
ex=groups['normal','exposed_regression'];fi=groups['normal','final'];mp=dict(zip(['Ova','Nia','Rin','Tessa','Bramble','Quarry','Vale','Marsh','Harbor','Ledge'],['Una','Pia','Soren','Kestrel','Cobalt','Mica','Ridge','Delta','Basalt','Dune']))
diffs=[]
for a,b in zip(ex,fi):
 mapped=re.sub('|'.join(mp),lambda m:mp[m.group()],a['text'])
 if mapped!=b['text']:diffs.append({'turn':a['turn'],'mapped_exposed':mapped,'final':b['text'],'kind':a['kind'],'expected_write':a.get('expected_write')})
out['novelty']={'training_strings':sorted(training),'final_strings':sorted(final),'overlap':sorted(training&final),'root13_first_new_lexical_population':True,'root13_root14_rows_identical':readrows(13)==rows,'root13_root14_models_identical':out['roots'][12]['artifacts']==out['roots'][13]['artifacts'],'final_and_exposed_event_shape_identical':[(x['kind'],x['scope'],x.get('declared_relation'),x.get('declared_intent')) for x in ex]==[(x['kind'],x['scope'],x.get('declared_relation'),x.get('declared_intent')) for x in fi],'renaming_differences':diffs,'qualification':'Fresh whole entity/value strings at root13; all clause forms shared with fit; interaction skeleton already exposed. Root14 is byte-identical replay, not second fresh draw.'}
out['defects']=[
 {'id':'capacity','finding':'Both runtime commit and ScmOracle::declared always evict chain[0]. Capacity2 with four writes retains three live payloads. Existing capacity asks do not probe the surviving second oldest.'},
 {'id':'history_control_denominator','finding':'history_control_differs compares capacity2 development matches16 to primary all-panels38; unequal denominators make this non-discriminating.'},
 {'id':'reload_semantic_bypass','finding':'Fresh child loads model but queries via ask_view from supplied entity/relation/history authored in scm_queries. No learned language query path in child, no subsequent write, no persisted in-flight session.'},
 {'id':'pin_witness','finding':'Only terminal Oren office changes between hops while Cedar->Oren remains unchanged. Unpinned Office Park exists in final committed view; no-single-view splice claim false.'},
 {'id':'fresh_final_scope','finding':'New whole lexical strings with the already-exposed operation structure. Root14 repeats root13 rows/models exactly.'},
 {'id':'answer_counts','finding':'Primary38/38 is oracle agreement:34 learned language questions+4 direct API queries;36 terminal Complete plus NoHistory and Unresolved. Intent/branch counts37/37 cover ingest operations, not all queries.'},
 {'id':'historical_continuation_missing','finding':'No submitted script asks previous/initial over a dependent redirect; history x continuation contract unexercised on language path.'}
]
pathlib.Path(args.output).write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'audit':args.output,'sealed_roots':sum(x['sealed'] for x in out['roots']),'rows':out['row_counts'],'ownership':out['ownership'],'semantics':out['independent_semantics'],'executable':out['executable'],'novelty':out['novelty']},indent=2))
