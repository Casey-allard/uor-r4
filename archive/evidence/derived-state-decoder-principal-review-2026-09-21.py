#!/usr/bin/env python3
"""Read-only PR1338 retained evidence audit; no model inference or training."""
import collections,hashlib,json,pathlib,struct,subprocess
import blake3
P=pathlib.Path; BASE=P('/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20');REPO=P('/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4');HEAD='16b7148e0db64e823309675e213e696cbcba79dd'
sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads(p.read_text()); majority=lambda c:max(c,key=lambda t:(c[t],t))
names=[f'derived-state-decoder-{i}' for i in range(1,6)];rs={n:load(BASE/n/'result.json') for n in names if (BASE/n/'result.json').exists()};rr={n:[json.loads(s) for s in (BASE/n/'rows.jsonl').read_text().splitlines()] for n in rs}
o={'schema':'uor-r4.derived-state-decoder-principal-evidence-audit/1','review_revision':HEAD,'scope':'Retained seal/hash/parameter and saved-row inspection, comparator recounts, source-derived fixture reconstruction. No model predictions, builds, fitting, optimization or sealed-root mutations.','integrity':{},'artifacts':{},'recount':{}}
exe=P('/Users/casey.allard/uor-r4/target/release/competitive-reader');exsha=sha(exe.read_bytes()) if exe.exists() else None
for n in names:
 root=BASE/n;paths={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}
 if n not in rs:
  o['integrity'][n]={'status':'INCOMPLETE_UNSEALED','files':sorted(paths),'file_hashes':{p:sha((root/p).read_bytes()) for p in sorted(paths)},'quality_evidence':'No result/rows/seal, not a completed run'};continue
 r=rs[n];m=load(root/'manifest.json');listed={p['path'] for p in m['files']};actual=paths-{'manifest.json'}
 aa=r['association_panel']['artifacts']+r['composition_panel']['artifacts']
 o['integrity'][n]={'listed_files':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'seal_errors':[p['path'] for p in m['files'] if len((root/p['path']).read_bytes())!=p['bytes'] or blake3.blake3((root/p['path']).read_bytes()).hexdigest()!=p['blake3']],'result_sha256':sha((root/'result.json').read_bytes()),'rows_sha256':sha((root/'rows.jsonl').read_bytes()),'artifact_hashes_match':{a['arm']:sha((root/f"artifacts/{a['arm']}.rlds").read_bytes())==a['artifact_sha256'] for a in aa},'submitted_source_matches':{x['path']:sha(subprocess.check_output(['git','show',HEAD+':'+x['path']],cwd=REPO))==x['sha256'] for x in r['running_source']['source_files']},'declared_source':r['running_source'],'current_executable_sha256':exsha,'current_executable_matches':r['running_source']['executable_sha256']==exsha,'claimed_at':load(root/'attempt.json')['claimed_at'],'sealed_at':m['sealed_at']}
def parse_model(path):
 b=path.read_bytes();c=0
 def take(n):
  nonlocal c
  q=b[c:c+n];assert len(q)==n;c+=n;return q
 def u32():return struct.unpack('<I',take(4))[0]
 assert take(4)==b'RLDS';version=u32();nt=u32();transport=list(take(nt));cyclic=bool(take(1)[0]);domains=[];codes=[]
 for _ in range(2):n=u32();domains.append([u32() for _ in range(n)])
 for _ in range(2):n=u32();codes.append(list(take(n)))
 dl=u32();end=c+dl;assert take(4)==b'RLRD';dv=u32();ns=u32();states=[]
 for _ in range(ns):
  s,valid=take(2);nk=struct.unpack('<H',take(2))[0];ts=[u32() for _ in range(nk)];sc=[struct.unpack('<i',take(4))[0] for _ in range(nk)];idx=min(range(nk),key=lambda j:(-sc[j],ts[j])) if nk else None
  states.append({'state':s,'valid':bool(valid),'tokens':ts,'scores':sc,'top_token':ts[idx] if idx is not None and valid else None,'top_score':sc[idx] if idx is not None and valid else 0})
 assert c==end==len(b)
 return {'sha256':sha(b),'bytes':len(b),'version':version,'cyclic':cyclic,'transport':transport,'op_domain':domains[0],'value_domain':domains[1],'op_codes':codes[0],'value_codes':codes[1],'decoder_version':dv,'states':states,'state_count':len(states)}
for n in names:
 o['artifacts'][n]={p.stem:parse_model(p) for p in sorted((BASE/n/'artifacts').glob('*.rlds'))}
for n,r in rs.items():
 xs=rr[n];dev=[x for x in xs if x['split']=='dev_cells'];test=[x for x in xs if x['split']=='held_out_cells'];h4=o['artifacts'][n]['composition_derived_state_h4'];states={x['state']:x for x in h4['states']};counter=collections.Counter(x['target'] for x in dev);constant=majority(counter);pay=collections.defaultdict(collections.Counter);two=collections.defaultdict(collections.Counter)
 for x in dev:pay[x['selected_payload']][x['target']]+=1;two[(x['query_role'],x['selected_payload'])][x['target']]+=1
 pm={k:majority(v) for k,v in pay.items()};tm={k:majority(v) for k,v in two.items()}
 panes={}
 for split,z in [('dev_cells',dev),('held_out_cells',test)]:
  slices={}
  for name,zz in [('all',z),('known_operation', [x for x in z if x['query_role'] in h4['op_domain']]),('unknown_operation',[x for x in z if x['query_role'] not in h4['op_domain']])]:
   slices[name]={'positions':len(zz),'h4_emitted_hits':sum(x['emitted']==x['target'] for x in zz),'decoder_noread':sum(x['decoder_token'] is None for x in zz),'correct_occurrences':sum(x['selected_payload_abs']==x['intended_payload_abs'] for x in zz),'correct_values':sum(x['selected_payload']==x['intended_payload'] for x in zz)}
  actual=collections.defaultdict(set)
  for x in z:actual[(x['op'],x['value_index'])].add(x['emitted'])
  panes[split]={'positions':len(z),'unique_cells':len(actual),'h4_emitted_hits':sum(x['emitted']==x['target'] for x in z),'h4_decoder_hits':sum(x['decoder_token']==x['target'] for x in z),'h4_reads':sum(x['read'] for x in z),'h4_decoder_noread':sum(x['decoder_token'] is None for x in z),'exact_source_flags_match':all(x['correct_source']==(x['selected_payload_abs']==x['intended_payload_abs']) for x in z),'value_flags_match':all(x['correct_value']==(x['selected_payload']==x['intended_payload']) for x in z),'source_is_payload_predecessor':all(x['selected_payload_abs']==x['selected_source_abs']+1 for x in z if x['read']),'decoder_table_matches_saved_token':all(x['decoder_token']==states.get(x['s'],{}).get('top_token') for x in z),'decoder_table_matches_saved_score':all(x['decoder_score']==states.get(x['s'],{}).get('top_score',0) for x in z),'payload_table_hits':sum(pm.get(x['selected_payload'],constant)==x['target'] for x in z),'operation_payload_table_hits':sum(tm.get((x['query_role'],x['selected_payload']),constant)==x['target'] for x in z),'constant_hits':sum(constant==x['target'] for x in z),'slices':slices,'class_zero_states':sorted({x['s'] for x in z if x['class']==0 and x['correct_source']}),'all_distinct_states':len({x['s'] for x in z}),'distinct_cells_with_all_four_members_correct':sum(all(x['emitted']==x['target'] for x in z if (x['op'],x['value_index'])==key) for key in actual),'missing_rows_for_other_arms':True}
 defined=set(h4['op_domain']);actualroles={x['op']:x['query_role'] for x in xs}
 o['recount'][n]={'panels':panes,'operation_domain_missing':{op:role for op,role in actualroles.items() if role not in defined},'all_cells_primitive_coverage':{'dev_operations':sorted({x['op'] for x in dev}),'dev_values':sorted({x['value_index'] for x in dev}),'test_operations':sorted({x['op'] for x in test}),'test_values':sorted({x['value_index'] for x in test})},'saved_row_fields':sorted(xs[0]),'state_intersection':len({x['s'] for x in dev}&{x['s'] for x in test}),'reported_h4_counts_match':all(panes[a['split']]['h4_emitted_hits']==a['hits'] for a in r['composition_panel']['arms'] if a['arm']=='h4_derived_state'),'retained_generated':r['composition_panel']['generated'],'retained_interventions':r['composition_panel']['interventions']}
# Source-derived fixture reconstruction, not reader/model replay.
banks=load(BASE/'competitive-reader-1/population.json')['banks'];values=banks['values_fit'][:8];mask=(1<<64)-1
class Rng:
 def __init__(self,s):self.s=s
 def next(self):
  s=self.s;s^=(s<<13)&mask;s^=s>>7;s^=(s<<17)&mask;self.s=s&mask;return self.s
 def pick(self,z):return z[self.next()%len(z)]
def testcell(op,vi):return vi in ([1,4] if op%2==0 else [2,5])
def comp_fixture(nops,seed):
 rg=Rng(seed|1);pairs=banks['role_pairs'][:nops];key=banks['keys'][0];outs=list(range(4095,4085,-1));items=[]
 for op,(qr,sr) in enumerate(pairs):
  for vi,value in enumerate(values):
   for rep in range(4):
    blocks=[]
    for _ in range(2):
     other=(op+1+rg.next()%(nops-1))%nops;blocks.append((pairs[other][1],key,values[rg.next()%8]))
    pos=rg.next()%3;blocks.insert(pos,(sr,key,value));prefix=[v for b in blocks for v in b]+[qr,key];items.append({'op':op,'vi':vi,'test':testcell(op,vi),'repeat':rep,'target':outs[(op+vi)%10],'source_abs':pos*3+2,'prefix':prefix})
 return items
items=comp_fixture(8,0xC0F10002);d=[x for x in items if not x['test']];t=[x for x in items if x['test']];saved=rr['derived-state-decoder-5']
o['composition_fixture']={'source_seed':0xC0F10002,'report_seed':rs['derived-state-decoder-5']['composition_panel']['seed'],'repository_evidence_json_seed':json.loads(subprocess.check_output(['git','show',HEAD+':docs/evidence/derived-state-decoder-2026-09-21.json'],cwd=REPO))['panels']['composition']['seed'],'labels_refs_match_saved_rows':all(a['op']==b['op'] and a['vi']==b['value_index'] and a['target']==b['target'] and a['source_abs']==b['intended_payload_abs'] for a,b in zip(d+t,saved)),'all_answers_absent':all(x['target'] not in x['prefix'] for x in items),'dev_test_exact_prefix_overlap':len({tuple(x['prefix']) for x in d}&{tuple(x['prefix']) for x in t}),'unique_dev_cells':len({(x['op'],x['vi']) for x in d}),'unique_test_cells':len({(x['op'],x['vi']) for x in t}),'first_three_generation_cells':[(x['op'],x['vi'],x['repeat']) for x in t[:3]],'every_test_primitive_seen_in_dev':{k:{x[k] for x in t}<={x[k] for x in d} for k in ['op','vi','target']},'canonical_sha256':sha(json.dumps(items,sort_keys=True,separators=(',',':')).encode())}
def ce_fixture(seed,n):
 rg=Rng(seed);outs=list(range(4095,4087,-1));items=[]
 for pid in range(n):
  key=rg.pick(banks['keys']);fa,fb=rg.pick(banks['role_pairs']);qr,sr=(fa,fb) if rg.next()&1==0 else (fb,fa);i0=rg.next()%8;i1=rg.next()%8
  if i1==i0:i1=(i0+1)%8
  pool=[p for p in banks['role_pairs'] if p!=[fa,fb] and p!=[fb,fa]];blocks=[]
  for _ in range(3):
   a,b=rg.pick(pool);role=a if rg.next()&1==0 else b;value=rg.pick(values);blocks.append((role,key,value))
  pos=rg.next()%3
  for member,vi in enumerate([i0,i1]):
   bs=blocks.copy();bs.insert(pos,(sr,key,values[vi]));prefix=[v for b in bs for v in b]+[qr,key];items.append({'pair':pid,'member':member,'target':outs[vi],'value':values[vi],'source_abs':3*pos+2,'prefix':prefix})
 return items
pops={s:ce_fixture(seed,n) for s,seed,n in [('dev',0xC0F20011,90),('tune',0xC0F20012,60),('final',0xC0F20021,60)]};old={s:ce_fixture(seed,n) for s,seed,n in [('dev',0xC0F00011,90),('tune',0xC0F00012,60),('final',0xC0F00021,60)]}
o['association_population']={'seeds_hex':['0xC0F20011','0xC0F20012','0xC0F20021'],'old_seeds_hex':['0xC0F00011','0xC0F00012','0xC0F00021'],'same_ordered_panel_as_prior':{s:pops[s]==old[s] for s in pops},'exact_prefix_overlap_against_prior_same_split':{s:len({tuple(x['prefix']) for x in pops[s]}&{tuple(x['prefix']) for x in old[s]}) for s in pops},'answers_absent':all(x['target'] not in x['prefix'] for v in pops.values() for x in v),'label_mapping_same_as_prior':True,'value_only_artifact_retained':any('value_only' in x for n in o['artifacts'].values() for x in n),'association_prediction_rows_retained':sum(x.get('panel')=='association' for x in saved),'claimed_new_vs_old_numbers_are_unpaired':True,'counts_scope':'161/108/104 dictionary/value-only counts are report-only. No association rows or value-only artifact retained. Full-factor association artifact exists; loaded parity420 runs, but headline counts use model_hits rather than saved dsd_step prediction records.'}
# Exact perturbations in submitted runner, reconstructed without inference.
key0,key1=banks['keys'][:2];q0,s0=banks['role_pairs'][0];q1,s1=banks['role_pairs'][1];drole=banks['role_pairs'][2][1];v0,v1=values[:2]
base=lambda q,s,v:[s,key0,v,q,key0]
a=base(q0,s0,v0);b=base(q1,s1,v0)
o['intervention_audit']={'operation_named_fixed_evidence':{'prefix_a':a,'prefix_b':b,'changed_positions':[i for i,(x,y) in enumerate(zip(a,b)) if x!=y],'both_source_role_and_query_role_change':a[0]!=b[0] and a[3]!=b[3]},'payload_pair':{'prefix_a':a,'prefix_b':base(q0,s0,v1),'changed_positions':[2],'expected_targets':[4095,4094]},'distractor_pair':{'prefix_a':[drole,key0,values[2]]+a,'prefix_b':[drole,key0,values[3]]+a,'changed_positions':[2]},'absent_fixture':[drole,key1,values[0],q0,key0],'equals_local_not_independently_evaluated':True,'identity_check':'Uses class(op,vi)==0 and tests decoder_token.is_some(), not computed s == actual group identity. Source canonical identity is1; saved correct-source class0 cell(0,0) emits from state25 in primary run.','read_update_disabled_scope':'Only one prefix; checks booleans, no full-logit parity against corresponding local baseline.'}
o['comparability']={'association_artifacts_equal_across_completed_roots':len({o['artifacts'][n]['association_derived_state']['sha256'] for n in rs})==1,'composition_4_5_same_inputs':all({k:x[k] for k in ['op','value_index','query_role','target','selected_payload','selected_payload_abs','intended_payload_abs','relation']}=={k:y[k] for k in ['op','value_index','query_role','target','selected_payload','selected_payload_abs','intended_payload_abs','relation']} for x,y in zip(rr['derived-state-decoder-4'],rr['derived-state-decoder-5'])),'source_changes_between4and5':[a['path'] for a,b in zip(rs['derived-state-decoder-4']['running_source']['source_files'],rs['derived-state-decoder-5']['running_source']['source_files']) if a['sha256']!=b['sha256']],'attempt4_heldout_was_exposed_before_objective_order_changed':True}
r=rs['derived-state-decoder-5'];paths=[('E',BASE/'head-projection-3/corrected/empirical.cpl2','E_sha256'),('S',BASE/'s-attribution-3/corrected/separable_older_query_read.cpx3','S_sha256'),('selector',BASE/'relational-learning-4/artifacts/relational_ctx.rlr2','selector_sha256')];o['input_hashes']={label:{'path':str(p),'sha256':sha(p.read_bytes()),'matches':sha(p.read_bytes())==r['inputs'][key]} for label,p,key in paths}
o['findings']=[
 'Completed roots1,2,4,5 are sealed, six listed files each. Root3 is partial and unsealed with association artifact only; preserve it as an incomplete attempt, not an empty directory or model result.',
 'The useful value-only104/120 claim is a report-only association decoder-count result on new seeds, not a paired improvement over old residual33/120. Neither association rows nor the value-only artifact is retained.',
 'The primary H4 composition103/192 development and8/64 heldout are independently recounted from actual saved emitted-token rows. These rows save only H4; C120 totals are reported, while ordinary dictionary controls can be reconstructed from shared saved read data.',
 'Artifact op_domain contains only6 fit operation tokens in roots4/5. Probe operations6/7 are absent and both map to the same identity fallback. Probe labels participate directly in coordinate-search scoring, so probe27/48 is training/selection performance, not out-of-sample generalization.',
 'The final heldout cells were evaluated before changing objective ordering from attempt4 to5. They remain exploratory/exposed across the investigation even though each individual fit uses development labels only.',
 'The operation intervention changes the source role as well as query role, so fixed evidence is false. The primary changed-source and changed-operation pairs each fail both_expected. Preserving a wrong answer on a distractor change is sensitivity evidence only.',
 'The identity intervention mistakes target class0 for actual computed group identity. Primary saved class0 at operation0,value0 has state25; group identity is1 by canonical root construction. Unit testing decode(0) is not an actual identity test either.',
 'The absence equals_local flag is tautologically true whenever !rem.read; it is not a separately measured full-logit equality. The served branch structure supports fallback, but its diagnostic assertion is weaker than claimed.',
 'Three generated prompts are repeated contexts for the same heldout cell(operation0,value1), all emit4087 rather than target4094 in the primary run. No complete answer+EOS, broad prose, language preservation, energy or matched cost evidence.',
 'The learned decoder uses intended-source correctness only in offline supervision, which is allowed but must be declared. Serving knows only whether any read occurred and whether the resulting state has an entry; a valid decoder state does not certify the selected source was correct.',
 'Whole heldout cells are not independent free parameters: operation and value maps are shared. An identifiability impossibility claim needs a concrete pair of observationally equivalent parameterizations or another proof, not poor optimization alone. The missing probe operation domains are a direct implementation obstruction to address first.',
 'The screen text requires paired intervention correctness but met computes only h4_test > control_max. It also reintroduces requiring H4 to beat C120 on a cyclic task, contrary to the prior principal competence/advantage separation.'
]
o['not_run_or_unavailable']=['New paired residual-vs-decoder comparison on identical association prefixes','Retained value-only artifact and serving/export parity for that arm','Association emitted-token prediction rows and changed-source pair accuracy','Independent final selection-free evaluation after objective choice','Operation-only intervention with fixed evidence','Actual learned identity-state intervention','Full-logit NoRead/UpdateDisabled/local parity','Complete generated answer+EOS','Natural text/ordinary language preservation','Measured serving latency and physical energy']
o['cost_scope']={'retained_complete_run_seconds':{n:r['elapsed_s'] for n,r in rs.items()},'total_complete_run_seconds':sum(r['elapsed_s'] for r in rs.values()),'incomplete_run3_seconds':'UNAVAILABLE','new_inference_or_build':False}
# Keep the delivered audit compact; retain digests and decision-relevant artifact structure.
for ms in o['artifacts'].values():
 for m in ms.values():
  m['transport_nonidentity_rows']={i:v for i,v in enumerate(m.pop('transport')) if v!=1}
  full_states=m.pop('states');m['decoder_parameter_sha256']=sha(json.dumps(full_states,sort_keys=True,separators=(',',':')).encode())
  m['decoder_top_entries']=[[x['state'],x['top_token'],x['top_score']] for x in full_states]
o['script_sha256']=sha(P(__file__).read_bytes());out=P('/tmp/uor-pr1338-evidence-review.json');out.write_text(json.dumps(o,indent=2)+'\n');print(json.dumps({'output':str(out),'bytes':out.stat().st_size,'integrity':{n:{k:v for k,v in q.items() if k not in ['declared_source']} for n,q in o['integrity'].items()},'recount':{n:{k:v for k,v in q.items() if k not in ['retained_generated','retained_interventions']} for n,q in o['recount'].items()},'fixture':o['composition_fixture'],'comparability':o['comparability']},indent=2))
