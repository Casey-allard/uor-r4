#!/usr/bin/env python3
"""Read-only PR1337 saved-evidence audit: no builds, fits or model predictions."""
import collections, hashlib, json, pathlib, struct, subprocess
import blake3
P=pathlib.Path
BASE=P('/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20')
REPO=P('/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4')
HEAD='b22ecb197f9b3cd54a64ff2be884cc8105cbd7f3'
NAMES=['geometric-computation-1','geometric-computation-2','relation-composition-1','relation-composition-5']
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(p.read_text())
rs={n:load(BASE/n/'result.json') for n in NAMES}
rr={n:[json.loads(x) for x in (BASE/n/'rows.jsonl').read_text().splitlines()] for n in NAMES}
o={'schema':'uor-r4.geometric-computation-principal-evidence-audit/1','scope':'Read-only seal/source/parameter inspection; counts and feature histograms reconstructed only from retained rows and parameter bytes; source-derived fixture reconstruction. No model prediction, build, fit, new corpus or sealed-root mutation.','review_revision':HEAD,'integrity':{},'attempt_comparison':{},'row_recomputation':{},'composition':{}}
exe=P('/Users/casey.allard/uor-r4/target/release/competitive-reader'); exsha=sha(exe.read_bytes()) if exe.exists() else None
for n,r in rs.items():
 root=BASE/n;m=load(root/'manifest.json');actual={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}-{'manifest.json'}; listed={x['path'] for x in m['files']}
 o['integrity'][n]={'listed_files':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'seal_errors':[x['path'] for x in m['files'] if len((root/x['path']).read_bytes())!=x['bytes'] or blake3.blake3((root/x['path']).read_bytes()).hexdigest()!=x['blake3']],'result_sha256':sha((root/'result.json').read_bytes()),'rows_sha256':sha((root/'rows.jsonl').read_bytes()),'artifact_hashes_match':{a['arm']:{'params':sha((root/f"artifacts/{a['arm']}.rlrc").read_bytes())==a['params_sha256'],'residual':sha((root/f"artifacts/{a['arm']}.rlce").read_bytes())==a['residual_sha256']} for a in r['artifacts']},'submitted_source_matches':{x['path']:sha(subprocess.check_output(['git','show',HEAD+':'+x['path']],cwd=REPO))==x['sha256'] for x in r['running_source']['source_files']},'declared_git_revision':r['running_source']['git_rev'],'declared_git_dirty':r['running_source']['git_dirty'],'declared_executable_sha256':r['running_source']['executable_sha256'],'retained_current_executable_sha256':exsha,'current_executable_matches':exsha==r['running_source']['executable_sha256'],'claimed_at':load(root/'attempt.json')['claimed_at'],'sealed_at':m['sealed_at']}
for a,b in [('geometric-computation-1','geometric-computation-2'),('relation-composition-1','relation-composition-5')]:
 o['attempt_comparison'][a+'_vs_'+b]={'rows_identical':rr[a]==rr[b],'same_fields':{k:rs[a].get(k)==rs[b].get(k) for k in ['instrument','validity','learning','comparisons','generated','artifacts','screen']},'elapsed_s':[rs[a]['elapsed_s'],rs[b]['elapsed_s']]}
def parse_rc(p):
 b=p.read_bytes(); ver,nt=struct.unpack_from('<II',b,4); c=12;t=list(b[c:c+nt]);c+=nt;nd,=struct.unpack_from('<I',b,c);c+=4;domain=list(struct.unpack_from('<'+'I'*nd,b,c));c+=4*nd;nc,=struct.unpack_from('<I',b,c);c+=4;codes=list(b[c:c+nc]);c+=nc
 return {'version':ver,'bytes':len(b),'consumed':c,'transport':t,'value_domain':domain,'value_codes':codes,'injective':len(set(codes))==len(codes),'changes_from_index_initialization':[(i,x) for i,x in enumerate(codes) if i!=x]}
def parse_res(p):
 b=p.read_bytes();ver,width,shift,nr=struct.unpack_from('<IIII',b,4);c=20;emb=list(struct.unpack_from('<'+'b'*(nr*width),b,c));c+=nr*width;nw,=struct.unpack_from('<I',b,c);c+=4;w=list(struct.unpack_from('<'+'b'*(nw*width),b,c));c+=nw*width;active=[i for i in range(nw) if any(w[i*width:(i+1)*width])]
 return {'version':ver,'bytes':len(b),'consumed':c,'width':width,'shift':shift,'r_rows':nr,'output_rows_stored':nw,'active_output_rows':active,'nonzero_coefficients':sum(x!=0 for x in w),'all_coefficients_ternary':all(-1<=x<=1 for x in emb+w),'embedding_rows':[emb[i*width:(i+1)*width] for i in range(nr)]}
art={n:{a['arm']:{'params':parse_rc(BASE/n/f"artifacts/{a['arm']}.rlrc"),'residual':parse_res(BASE/n/f"artifacts/{a['arm']}.rlce")} for a in r['artifacts']} for n,r in rs.items()}
o['artifact_parameters']=art
r=rs['geometric-computation-2'];rows={s:[x for x in rr['geometric-computation-2'] if x['split']==s] for s in ['dev','tune','final']}
majority=lambda c:max(c,key=lambda t:(c[t],t))
const=majority(collections.Counter(x['target'] for x in rows['dev'])); c=collections.defaultdict(collections.Counter)
for x in rows['dev']:c[x['selected_payload']][x['target']]+=1
cat={v:majority(h) for v,h in c.items()}
for s,xs in rows.items():
 def stats(fn):
  h=[fn(x)==x['target'] for x in xs]; pairs=collections.defaultdict(list)
  for x,y in zip(xs,h):pairs[x['pair']].append(y)
  return {'hits':sum(h),'pairs_both':sum(len(a)==2 and all(a) for a in pairs.values()),'correct_exact_occurrence_hits':sum(y and x['correct_exact_occurrence'] for x,y in zip(xs,h)),'correct_value_hits':sum(y and x['correct_payload_value'] for x,y in zip(xs,h))}
 sig=collections.defaultdict(collections.Counter)
 for x in xs:sig[(x['q0'],x['relation'],x['selected_payload'])][x['target']]+=1
 o['row_recomputation'][s]={'positions':len(xs),'reads':sum(x['read'] for x in xs),'correct_exact_occurrence_saved_flags':sum(x['correct_exact_occurrence'] for x in xs),'correct_value_recomputed':sum(x['selected_payload']==x['value'] for x in xs),'correct_value_flags_valid':all(x['correct_payload_value']==(x['selected_payload']==x['value']) for x in xs),'categorical':stats(lambda x:cat.get(x['selected_payload'],const)),'constant':stats(lambda x:const),'update_signatures':len(sig),'ambiguous_signatures':sum(len(v)>1 for v in sig.values()),'distinct_targets_summed_over_ambiguous_signatures':sum(len(v) for v in sig.values() if len(v)>1),'positions_on_ambiguous_signatures':sum(sum(v.values()) for v in sig.values() if len(v)>1),'signature_table_with_same_split_labels_best_hits':sum(max(v.values()) for v in sig.values()),'missing_geometric_predictions_and_refs':not any(k in xs[0] for k in ['emitted','h4_emitted','selected_abs','payload_abs','selected_ref'])}
o['association_generation']=[{'answer':g['answer'],'emitted':g['emitted'],'decoded':g['decoded'],'first_correct':g['emitted'][0]==g['answer'],'steps_match':g['emitted']==[s['emitted'] for s in g['steps']]} for g in r['generated']]
for n in ['relation-composition-1','relation-composition-5']:
 r=rs[n]; xs=rr[n]; d=[x for x in xs if x['split']=='dev']; t=[x for x in xs if x['split']!='dev'];R=art[n]['h4_composition']['residual']['embedding_rows']
 feature=lambda x:tuple(b-a for a,b in zip(R[x['q0']],R[x['q1']]))
 def feature_report(z):
  z=[x for x in z if x['read'] and x['correct_source']];m=collections.defaultdict(collections.Counter)
  for x in z:m[feature(x)][x['target']]+=1
  return {'positions':len(z),'distinct_features':len(m),'conflicting_features':sum(len(v)>1 for v in m.values()),'positions_on_conflicting_features':sum(sum(v.values()) for v in m.values() if len(v)>1),'in_split_oracle_table_hits':sum(max(v.values()) for v in m.values())},m
 fd,md=feature_report(d);ft,mt=feature_report(t)
 c=collections.Counter(x['target'] for x in d);constant=majority(c);pay=collections.defaultdict(collections.Counter);two=collections.defaultdict(collections.Counter)
 for x in d:pay[x['selected_payload']][x['target']]+=1;two[(x['relation'],x['selected_payload'])][x['target']]+=1
 pmap={key:majority(v) for key,v in pay.items()};tmap={key:majority(v) for key,v in two.items()}; fmap={key:majority(v) for key,v in md.items()}
 def measures(z):
  return {'positions':len(z),'correct_source_saved_flags':sum(x['correct_source'] for x in z),'correct_value_recomputed':sum(x['selected_payload']==r['instrument']['values'][x['value_index']] for x in z),'relation_counts':dict(collections.Counter(x['relation'] for x in z)),'correct_source_relation_counts':dict(collections.Counter(x['relation'] for x in z if x['correct_source'])),'unique_operation_value_cells':len({(x['op'],x['value_index']) for x in z}),'unique_targets':sorted({x['target'] for x in z}),'unique_source_values':sorted({r['instrument']['values'][x['value_index']] for x in z}),'unique_selected_values':sorted({x['selected_payload'] for x in z}),'constant_hits':sum(x['target']==constant for x in z),'payload_table_hits':sum(x['target']==pmap.get(x['selected_payload'],constant) for x in z),'relation_payload_table_hits':sum(x['target']==tmap.get((x['relation'],x['selected_payload']),constant) for x in z),'dev_feature_table_seen_features':sum(feature(x) in fmap for x in z if x['correct_source']),'dev_feature_table_hits_seen_only':sum(fmap.get(feature(x))==x['target'] for x in z if x['correct_source']),'dev_feature_table_hits_with_constant_fallback':sum(fmap.get(feature(x),constant)==x['target'] for x in z if x['correct_source'])}
 o['composition'][n]={'modulus':r['instrument'].get('class_modulus',10),'declared_rule':r['instrument']['declared_rule'],'dev':measures(d),'test':measures(t),'feature_dev':fd,'feature_test':ft,'correct_source_features_shared_dev_test':len(set(md)&set(mt)),'all_features_shared_dev_test':len({feature(x) for x in d}&{feature(x) for x in t}),'source_values_shared_dev_test':len({x['value_index'] for x in d}&{x['value_index'] for x in t}),'heldout_value_codes':{a:{v:code for v,code in zip(z['params']['value_domain'],z['params']['value_codes']) if v in {r['instrument']['values'][x['value_index']] for x in t}} for a,z in art[n].items()},'class_formula_matches_recorded_rows':all(x['class']==(x['op']+x['value_index'])%r['instrument'].get('class_modulus',10) for x in xs),'rows_fields':sorted(xs[0]),'generation':'NOT_RUN: no generation entries in report or generation loop in composition source'}
# Independent source-derived composition fixtures: no learned selection or logits recomputed.
bank=load(BASE/'competitive-reader-1/population.json')['banks']; mask=(1<<64)-1
class Rng:
 def __init__(self,s):self.s=s
 def next(self):
  s=self.s;s^=(s<<13)&mask;s^=s>>7;s^=(s<<17)&mask;self.s=s&mask;return self.s
r=rs['relation-composition-5'];seen=set();pairs=[]
for a in r['instrument']['relation_exposure']:
 if a['relation'] not in seen:seen.add(a['relation']);pairs.append((a['qrole'],a['source_role']))
rg=Rng(0xC0F10001);vals=r['instrument']['values'];out=r['instrument']['output_bank'];k=r['instrument']['class_modulus'];key=bank['keys'][0];items=[]
for op,(qr,sr) in enumerate(pairs):
 for vi,value in enumerate(vals):
  for repeat in range(4):
   blocks=[]
   for j in range(2):
    other=(op+1+rg.next()%(len(pairs)-1))%len(pairs);dr=pairs[other][1];dv=vals[rg.next()%len(vals)];blocks.append((dr,key,dv))
   pos=rg.next()%3;blocks.insert(pos,(sr,key,value));prefix=[t for b in blocks for t in b]+[qr,key]
   items.append({'op':op,'vi':vi,'test':vi>=6,'repeat':repeat,'target':out[(op+vi)%k],'prefix':prefix,'source_abs':pos*3+2})
d=[x for x in items if not x['test']];t=[x for x in items if x['test']]
rec={'rows_agree_with_fixture_labels':all(all(a[ak]==b[bk] for ak,bk in [('op','op'),('vi','value_index'),('test','test'),('target','target')]) for a,b in zip(d+t,rr['relation-composition-5'])),'target_absent_from_all_prefixes':all(x['target'] not in x['prefix'] for x in items),'development_prefix_contains_heldout_values':{v:sum(v in x['prefix'] for x in d) for v in vals[6:]},'dev_prefix_distinct':len({tuple(x['prefix']) for x in d}),'test_prefix_distinct':len({tuple(x['prefix']) for x in t}),'dev_test_prefix_overlap':len({tuple(x['prefix']) for x in d}&{tuple(x['prefix']) for x in t}),'canonical_fixture_sha256':sha(json.dumps(items,sort_keys=True,separators=(',',':')).encode()),'same_prefix_except_one_token_with_changed_target':[]}
for i,a in enumerate(items):
 for j,b in enumerate(items[i+1:],i+1):
  diffs=[k for k,(aa,bb) in enumerate(zip(a['prefix'],b['prefix'])) if aa!=bb]
  if len(diffs)==1 and a['target']!=b['target']:rec['same_prefix_except_one_token_with_changed_target'].append({'a':i,'b':j,'changed_abs':diffs[0],'changed_relevant_value':diffs==[a['source_abs']] and a['source_abs']==b['source_abs'],'changed_query_role':diffs==[len(a['prefix'])-2]})
o['composition']['fixture_reconstruction']=rec
o['association_attempt_scope']={'same_rows_as_previous_pr1336':rr['geometric-computation-2']==[json.loads(x) for x in (BASE/'consistent-emission-2/rows.jsonl').read_text().splitlines()],'same_output_feature_embedding_as_previous':sha((BASE/'consistent-emission-2/artifacts/h4_emission.rlce').read_bytes()[20:20+120*16])==sha((BASE/'geometric-computation-2/artifacts/h4_emission.rlce').read_bytes()[20:20+120*16])}
o['reported_losses']={n:r['learning'] for n,r in rs.items()}
o['learning_receipt_consistency']={n:{a:{'stage_losses_monotone':all(u>=v for u,v in zip([z['nll_bits'][k] for k in ['initial','served_output_before_map_search','served_after_map_search','served_final']], [z['nll_bits'][k] for k in ['served_output_before_map_search','served_after_map_search','served_final']])),'refit_incumbent_equals_after_maps':z['nll_bits']['post_map_refit_incumbent']==z['nll_bits']['served_after_map_search'],'active_rows_match_artifact':z['output_rows']==len(art[n]['h4_emission' if a=='h4' else 'cyclic_c120_emission']['residual']['active_output_rows']) if n.startswith('geometric') else z['output_rows']==len(art[n]['h4_composition' if a=='h4' else 'cyclic_c120_composition']['residual']['active_output_rows']),'final_refit_hash_matches_delivered':z['output_after_maps_refit_sha256']==next(v['residual_sha256'] for v in r['artifacts'] if v['algebra']==('signed_h4' if a=='h4' else 'cyclic_c120'))} for a,z in r['learning'].items()} for n,r in rs.items()}
r=rs['geometric-computation-2']
inputs=[('E',BASE/'head-projection-3/corrected/empirical.cpl2',r['inputs']['E_sha256']),('S',BASE/'s-attribution-3/corrected/separable_older_query_read.cpx3',r['inputs']['S_sha256']),('raw_tokenizer',BASE.parent/'sources/smollm2-135m-instruct/tokenizer.json','9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c'),('reader_parent',BASE/'relational-learning-4/artifacts/relational_ctx.rlr2',r['inputs']['selector_sha256'])]
o['inputs']={label:{'path':str(p),'sha256':sha(p.read_bytes()),'matches_declared':sha(p.read_bytes())==h} for label,p,h in inputs}

o['findings']=[
 'No geometric model predictions or full logits are retained in either row schema; complete H4/C120 hits, pairs, CE and parity are reported/source-inspected, not independently reconstructed by this read-only audit. Correct-occurrence booleans likewise lack exact occurrence refs.',
 'Feature separability is an empirical histogram on each split with that split target labels and ground-truth correct-source filter. It is an in-sample oracle ceiling; it does not measure a development-fitted feature decoder generalizing to test, nor establish linear infeasibility or optimizer convergence.',
 'The association artifact repeated exactly on exposed populations. Its fully separating feature report refutes observed feature equality collisions on each eligible population only; training failure below a lookup ceiling does not prove any linear map impossible.',
 'Composition holds out value indices6,7 for every operation: these are unseen correct-source operands, not novel combinations of operands each learned in other operation cells. Tokens do occur as distractors. Both held-out value codes retain initialization6,7 in both arms.',
 'Composition5 uses class=(op+vi) mod7. A witnessed order10 element cannot prove the asserted q0*h^class modulo7 factorization. A group of order120 has no order7 element. This refutes that proposed power witness, not every alternative code+decoder implementation of the finite task.',
 'The two intended relation values80,85 across14 in-scope directed role pairs are a scoped descriptor/interface collapse, not total geometric group capacity. Composition5 observes a third relation14 from two wrong source reads; correct-source positions use exactly80,85.',
 'Composition5 held-out feature oracle is16/16 using held-out targets, but zero held-out features occur in correct-source development. A development-fitted feature table sees0/16 and obtains4/16 only from constant fallback, matching the ordinary lookup controls.',
 'The report calls ambiguous signature distinct-label total21/17 a count of positions. Actual affected position counts must be taken from the row reconstruction; equality of residual update input cannot prove whole-predictor aliasing when local logits can differ.',
 'Composition generation, paired controlled changed-source/change-operation experiments, absent-source fixtures, changed-distractor controls, independent new final data and natural-text integration were not executed in this instrument. Association has three three-token loaded traces,1/3 correct first answers, no useful prose.'
]
o['cost_scope']={'retained_attempt_elapsed_s':{n:r['elapsed_s'] for n,r in rs.items()},'retained_attempt_total_s':sum(r['elapsed_s'] for r in rs.values()),'new_model_inference_in_audit':False,'physical_energy':'UNAVAILABLE'}
o['script_sha256']=sha(P(__file__).read_bytes());dest=P('/tmp/uor-pr1337-evidence-review.json');dest.write_text(json.dumps(o,indent=2)+'\n')
print(json.dumps({'output':str(dest),'integrity':o['integrity'],'attempt_comparison':o['attempt_comparison'],'row_recomputation':o['row_recomputation'],'composition':o['composition']},indent=2))
