#!/usr/bin/env python3
"""Read-only saved evidence audit: PR1347. Integer Hamilton arithmetic is an
independent expectation calculator, not a model implementation. Does not build,
fit, run inference, mutate artifacts, or import the Rust product table."""
import argparse, collections, hashlib, json, pathlib, re, subprocess
import blake3
ap=argparse.ArgumentParser()
ap.add_argument('--base', default='/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20')
ap.add_argument('--source', default='/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4')
ap.add_argument('--exe', default='/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/consumed-geometric-state-principal-delivery/competitive-reader-submitted-a94c9dec')
ap.add_argument('--output', default='/tmp/uor-consumed-submission-audit.json')
a=ap.parse_args(); base=pathlib.Path(a.base); source=pathlib.Path(a.source)
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(pathlib.Path(p).read_text())
def rows(n):
 p=base/f'consumed-geometric-state-{n}'/'rows.jsonl'
 return [json.loads(s)for s in p.read_text().splitlines()]if p.exists()else[]
result={'schema':'uor-r4.consumed-submission-independent-audit/1','method':'Saved artifact/receipt audit. Independent integer Hamilton products over explicitly declared signed-axis labels; no model execution. Source audit fixed at a94c9dec.','roots':[]}
for n in range(1,17):
 p=base/f'consumed-geometric-state-{n}';d={'root':n,'exists':p.exists(),'sealed':(p/'manifest.json').exists()};result['roots'].append(d)
 if not p.exists():continue
 actual={str(x.relative_to(p))for x in p.rglob('*')if x.is_file()}
 if d['sealed']:
  m=load(p/'manifest.json');expected={f['path']for f in m['files']};errors=[]
  for f in m['files']:
   b=(p/f['path']).read_bytes()
   if len(b)!=f['bytes']or blake3.blake3(b).hexdigest()!=f['blake3']:errors.append(f['path'])
  d.update(bad_seals=errors,missing=sorted(expected-actual),unlisted=sorted(actual-expected-{'manifest.json'}),sealed_at=m['sealed_at'])
 d['artifact_sha256']={str(x.relative_to(p)):sha(x.read_bytes())for x in (p/'artifacts').glob('*.json')}
 rr=rows(n)
 if rr:d.update(rows=len(rr),rows_sha256=sha((p/'rows.jsonl').read_bytes()),final_rows_sha256=sha(json.dumps([r for r in rr if r['panel']=='final'],sort_keys=True).encode()))
 if(p/'result.json').exists():
  r=load(p/'result.json');d.update(panels=r['panels'],checks=r['checks'],running_source=r['running_source'])
p=base/'consumed-geometric-state-16';r=load(p/'result.json');rr=rows(16)
result['source_bindings']=[]
for sf in r['running_source']['source_files']:
 b=subprocess.check_output(['git','show','a94c9dec:'+sf['path']],cwd=source)
 result['source_bindings'].append({'path':sf['path'],'recorded_sha256':sf['sha256'],'committed_matches':sha(b)==sf['sha256']})
exe=pathlib.Path(a.exe);result['executable']={'path':str(exe),'exists':exe.exists(),'recorded_sha256':r['running_source']['executable_sha256']}
if exe.exists():result['executable'].update(actual_sha256=sha(exe.read_bytes()),matches=sha(exe.read_bytes())==r['running_source']['executable_sha256'])
# Declared eight label vectors in first eight canonical roots; each vector is exactly
# one signed coordinate. We do NOT read the learned artifact to determine targets.
labels=['Alma','Bert','Cora','Dane','Elin','Frey','Gwen','Holt']
axes=[tuple(s if j==axis else 0 for j in range(4))for axis in range(4)for s in(-1,1)]
def hamilton(a,b):
 w,x,y,z=a;v,i,j,k=b
 return(w*v-x*i-y*j-z*k,w*i+x*v+y*k-z*j,w*j-x*k+y*v+z*i,w*k+x*j-y*i+z*v)
mul=lambda a,b:axes.index(hamilton(axes[a],axes[b]))
ops={'i':2,'j':4,'e':1} # first noncommuting pair is (-i,-j), plus +1.
assert mul(2,4)!=mul(4,2)
assert all(mul(1,x)==x and mul(x,1)==x for x in range(8))
dev_people=['Mara','Ivo','Cedar','Oren'];final_people=['Una','Pia','Soren','Kestrel']
dev_dests=['Bramble','Quarry','Vale','Marsh','Tarn','Ledge','Ridge','Stone'];final_dests=['Cobalt','Mica','Dune','Basalt','Flint','Slate','Amber','Onyx']
assignments={0:[6,1,4,3],1:[6,1,4,3],4:[0,5,2,7],5:[6,1,4,3]}
redirects={0:5,1:2,4:3,5:6}
def world_records(w):
 people=final_people if w>=4 else dev_people;dests=final_dests if w>=4 else dev_dests
 recs=[]
 for person,label in zip(people,assignments[w]):recs.append({'entity':person,'value':labels[label],'continues':False})
 for k,label in enumerate(labels):recs.append({'entity':label,'value':labels[(k+1)%8]if k==redirects[w]else dests[k],'continues':k==redirects[w]})
 for i,rec in enumerate(recs):rec.update(id=i+1,commit=i+1)
 return recs
errors=[];expectation_checks=0;match_checks=0;read_checks=0;emission_checks=0;arithmetic_checks=0;consumption_checks=0
artifact=load(p/'artifacts/artifact.json')
for rowidx,row in enumerate(rr):
 w=row['world'];people=final_people if w>=4 else dev_people;person=row['person'];pos=people.index(person);recs=world_records(w);bykey={x['entity']:x for x in recs};state=assignments[w][pos]
 expected_key=None
 for op in row['ops']:state=mul(ops[op],state)
 if row['ops']:
  expected_key=labels[state];record=bykey[expected_key];expected_reads=2
  while record['continues']:record=bykey[record['value']];expected_reads+=1
 else:record=recs[pos];expected_reads=1
 expected=record['value'];expectation_checks+=1
 if row['expected']!='Answer('+expected+')':errors.append([rowidx,'declared expected differs from integer Hamilton oracle',expected,row['expected']])
 matched=row['terminal']=='Complete'and row['answer']==expected and row['reads']==expected_reads;match_checks+=1
 if matched!=row['matched']:errors.append([rowidx,'match flag differs from independent oracle'])
 effects=row['effects'];frame=row['final_frame'];reads=[e for e in effects if e['action']=='Read'and e['selected_record']is not None]
 for e in reads:
  rec=recs[e['selected_record']-1];read_checks+=1
  if rec['commit']!=e['selected_commit']or rec['value'].encode()!=bytes(e['selected_value']):errors.append([rowidx,'selected record commit/value mismatch'])
 if row['terminal']=='Complete':
  emission_checks+=1;cap=frame['captured'];emitted=[e['emitted']for e in effects if e['emitted']is not None]
  if frame['emitted']!=cap['payload']+[frame['eos']]or frame['emitted']!=emitted:errors.append([rowidx,'emitted bytes not owned token payload plus EOS'])
  if bytes(cap['value']).decode()!=row['answer']:errors.append([rowidx,'reported answer differs from captured lexical value'])
 if row['arm']=='primary_signed'and row['ops']:
  c=row['computed'];arithmetic_checks+=1
  if bytes(c['derived_key']).decode()!=expected_key or c['derived_label']!=state:errors.append([rowidx,'geometric grounded result differs from independent Hamilton oracle'])
  applied=artifact['payload_state'][assignments[w][pos]]
  oe=[e for e in effects if e['action']=='Apply'and e['op_label']is not None]
  if len(oe)!=len(row['ops']):errors.append([rowidx,'wrong actual primitive Apply count'])
  for op,e in zip(row['ops'],oe):
   applied=mul(ops[op],applied)
   if applied!=e['computed_state']:errors.append([rowidx,'intermediate state differs from independent Hamilton product'])
  if applied!=c['state']:errors.append([rowidx,'final state differs from arithmetic trace'])
  if c['source_record']!=pos+1 or c['source_commit']!=pos+1 or bytes(c['source_entity']).decode()!=person:errors.append([rowidx,'computation source provenance mismatch'])
  if not c['consumed']or len(reads)<2 or recs[reads[1]['selected_record']-1]['entity']!=expected_key:errors.append([rowidx,'computed result not actual second read address'])
  consumption_checks+=1
result['independent_audit']={'rows':len(rr),'expectations_checked':expectation_checks,'match_flags_checked':match_checks,'selected_reads_checked':read_checks,'complete_owned_emissions_checked':emission_checks,'primary_computation_arithmetic_checked':arithmetic_checks,'primary_consumed_addresses_checked':consumption_checks,'errors':errors,'oracle_note':'Expected labels from signed-axis Hamilton arithmetic and authored world, not learned factorization or row.expected. Intermediate-state checks use only artifact initial gauge and integer products.'}
result['arm_counts']=[]
for arm in sorted(set(x['arm']for x in rr)):
 ar=[x for x in rr if x['arm']==arm];cr=[x for x in ar if x['ops']]
 result['arm_counts'].append({'arm':arm,'rows':len(ar),'matches':sum(x['matched']for x in ar),'ordinary':len(ar)-len(cr),'computation':len(cr),'computation_matches':sum(x['matched']for x in cr),'read_depths_for_computation':dict(collections.Counter(x['reads']for x in cr)),'actual_primitive_apply_events':sum(e['action']=='Apply'and e['op_label']is not None for x in ar for e in x['effects'])})
primary=[x for x in rr if x['arm']=='primary_signed']
result['novelty']={'first_recorded_final_root':7,'root7_final_complete':load(base/'consumed-geometric-state-7/result.json')['panels'][1],'root7_development_complete':load(base/'consumed-geometric-state-7/result.json')['panels'][0],'root15_root16_rows_byte_identical':(base/'consumed-geometric-state-15/rows.jsonl').read_bytes()==(p/'rows.jsonl').read_bytes(),'whole_final_people_dests_disjoint':not(set(dev_people+dev_dests)&set(final_people+final_dests)),'fitting_operation_orders':[['i','j'],['j','i'],['i','e']],'final_operation_orders':[['i','j','i'],['j','i','j']],'qualification':'Unseen order combinations and length3 at first exposure root7; same final population was then repeatedly evaluated while identity and comparator changed. Root16 is exposed regression, not fresh post-selection acceptance.'}
result['worlds']={'assignments':assignments,'redirects':redirects,'world_assignment_bug':'cgs_worlds tests people.len()==CGS_FINAL_PEOPLE.len(); both constants length4, so first assignment branch always overwritten. Development scopes share operand assignment; beta differs in redirect position.'}
result['claims_requiring_qualification']=[
 {'id':'same_artifact_preservation_missing','severity':'blocker_for_completion','finding':'Binder/intent refit from 48 Cgs forms, not preserved artifacts. cgs_ordinary is only two scoped assertions, one NonAsserting question and two asks. Corrections/history/same-value/initial/previous/capacity/in-flight pin lifecycle not exercised with this artifact.'},
 {'id':'raw_memory_bypass','severity':'blocker_for_completion','finding':'cgs_memory calls Memory.write directly with authored relation0 and continues booleans. Five primary third-hop cases demonstrate deterministic continuation over host-authored links, not learned interpretation of raw documents.'},
 {'id':'restart_scope','severity':'blocker_for_completion','finding':'One same-store in-memory snapshot after first Apply; disk reload restarts from request; fresh process restarts serialized Clause from beginning. No child phase restore, no before/after consumption restore, no origin mutation/eviction, no exact provenance comparison in child success gate.'},
 {'id':'exposed_final','severity':'qualification','finding':'Final24/24 first exposed root7 before dev16 failures were corrected; root16 exact replay of15. Retain score but call exposed composition panel.'},
 {'id':'control_gate','severity':'repair','finding':'consume_disabled_changes_answer only checks consumed=false; it does not compare answers or verify unchanged computed state. Saved witness does change answer, but predicate fails to enforce it.'},
 {'id':'unscoped_control','severity':'qualification','finding':'Control empties lookup scope while fixture records all have named scopes. 0/40 is missing-address behavior, not competitive cross-scope value selection.'},
 {'id':'comparator_persistence','severity':'qualification','finding':'Only signed artifact persisted/reloaded. Fitted, tabulated and folded arms use in-memory clones; no saved finite transition artifact/provenance permits standalone reproduction of their rows.'},
 {'id':'finite_decoder','severity':'qualification','finding':'Fitted shared-transition arm inverts value_state to decode instead of using its trained decoder. Source result acknowledges mismatch;26/40 cannot attribute the gap to lack of geometry. Tabulated finite32/32 computation control ties signed.'},
 {'id':'counts','severity':'qualification','finding':'Primary80 requests comprise16 ordinary and64 compute;40-row arms comprise8 ordinary and32 compute. Apply effect tally584 includes publication steps with no operator; actual primitive Apply events360 across all arms.'},
 {'id':'world_assignment','severity':'repair','finding':'All length4 people slices trigger final assignment branch, collapsing intended development world assignment diversity.'},
 {'id':'artifact_version','severity':'qualification','finding':'New Session version3 invalidates serialized version2 sessions even memory-only; old retained code unchanged does not establish persisted artifact compatibility.'}
]
pathlib.Path(a.output).write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'output':a.output,'roots':len(result['roots']),'sealed':sum(x['sealed']for x in result['roots']),'independent_audit':result['independent_audit'],'arm_counts':result['arm_counts'],'executable':result['executable']},indent=2))
