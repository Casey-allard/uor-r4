#!/usr/bin/env python3
"""PR1339 read-only independent audit of retained events/bytes/supervision coverage. No model inference."""
import collections, hashlib, json, pathlib, shutil, struct, subprocess
import blake3
P=pathlib.Path
BASE=P('/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20')
REPO=P('/Users/casey.allard/uor-r4/.worktrees/shared-transition')
ROOT=BASE/'shared-transition-6'
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(p.read_text())
out={'schema':'uor-r4.pr1339-independent-evidence-audit/1','scope':'Read-only saved event recount, serialized-parameter inspection, source review, hashes and development-transition support audit. No training, model inference, Rust build or sealed-root mutation.','submitted_head':'832c1c33','roots':{},'counts':{},'artifacts':{}}

def decoder(b):
 c=0
 def take(n):
  nonlocal c
  v=b[c:c+n]; assert len(v)==n;c+=n;return v
 def u32():return struct.unpack('<I',take(4))[0]
 assert take(4)==b'RLRD';version=u32(); n=u32();states=[]
 for _ in range(n):
  s,valid=take(2);k=struct.unpack('<H',take(2))[0];ts=[u32() for _ in range(k)];scores=[struct.unpack('<i',take(4))[0] for _ in range(k)];j=min(range(k),key=lambda i:(-scores[i],ts[i])) if k else None
  states.append({'state':s,'valid':bool(valid),'top_token':ts[j] if j is not None and valid else None,'tokens':ts,'scores':scores})
 assert c==len(b)
 return states

def rlst(p):
 b=p.read_bytes();c=0
 def take(n):
  nonlocal c
  v=b[c:c+n];assert len(v)==n;c+=n;return v
 def u32():return struct.unpack('<I',take(4))[0]
 assert take(4)==b'RLST';version=u32();cyclic=bool(take(1)[0]);dom=[];codes=[]
 for _ in range(2):n=u32();dom.append([u32() for _ in range(n)])
 for _ in range(2):n=u32();codes.append(list(take(n)))
 n=u32();stop=[list(take(2)) for _ in range(n)];n=u32();st=decoder(take(n));assert c==len(b)
 return {'sha256':sha(b),'bytes':len(b),'version':version,'cyclic':cyclic,'value_domain':dom[0],'action_domain':dom[1],'value_codes':codes[0],'action_codes':codes[1],'stop_table':stop,'decoder_states':st,'actual_decoder_state_count':len(st),'action_aliases':{str(code):[tok for tok,co in zip(dom[1],codes[1]) if co==code] for code in sorted(set(codes[1])) if codes[1].count(code)>1}}

def groups(rows):
 g=collections.defaultdict(list)
 for x in rows:g[x.get('item_id',(x['split'],x['value']))].append(x)
 for xs in g.values():xs.sort(key=lambda x:x['primitive_index'])
 return g

def recount(g):
 d={}
 for split in sorted({xs[0]['split'] for xs in g.values()}):
  items=[xs for xs in g.values() if xs[0]['split']==split]
  complete=lambda xs:all(x['emitted']==x['expected'] and x['stopped'] for x in xs)
  correct=lambda xs:xs[0]['read'] and xs[0]['source_abs']==1 and xs[0]['selected_payload']==xs[0]['value']
  seqg=collections.defaultdict(list)
  for xs in items:seqg[tuple(x['primitive'] for x in xs)].append(xs)
  d[split]={'items':len(items),'unique_primitive_sequences':len(seqg),'complete':sum(map(complete,items)),'token_hits':sum(x['emitted']==x['expected'] for xs in items for x in xs),'token_total':sum(map(len,items)),'stopped_correct_length':sum(all(x['emitted'] is not None and x['stopped'] for x in xs) for xs in items),'correct_source_and_value':sum(map(correct,items)),'complete_with_correct_source':sum(complete(xs) and correct(xs) for xs in items),'complete_with_wrong_source':sum(complete(xs) and not correct(xs) for xs in items),'reads':sum(xs[0]['read'] for xs in items),'source_abs_counts':dict(collections.Counter(str(xs[0]['source_abs']) for xs in items)),'step_kind_counts':dict(collections.Counter(str(x['step_kind']) for xs in items for x in xs)),'sequences_complete_for_all_four_values':sum(all(map(complete,v)) for v in seqg.values()),'sequence_complete_count_histogram':dict(collections.Counter(str(sum(map(complete,v))) for v in seqg.values())),'primitive_domains_by_position':{str(j):sorted({s[j] for s in seqg if len(s)>j}) for j in range(max(map(len,seqg)))},'by_value':{str(v):{'items':sum(xs[0]['value']==v for xs in items),'complete':sum(xs[0]['value']==v and complete(xs) for xs in items)} for v in sorted({xs[0]['value'] for xs in items})}}
 return d

for root in sorted(BASE.glob('shared-transition-*')):
 r=load(root/'result.json'); m=load(root/'manifest.json');attempt=load(root/'attempt.json'); actual={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}-{'manifest.json'};listed={x['path'] for x in m['files']}; rows=[json.loads(l) for l in (root/'rows.jsonl').read_text().splitlines()]
 checks=[]
 for x in m['files']:
  b=(root/x['path']).read_bytes()
  if len(b)!=x['bytes'] or blake3.blake3(b).hexdigest()!=x['blake3']:checks.append(x['path'])
 out['roots'][root.name]={'listed_files':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'seal_errors':checks,'retained_bytes':sum(p.stat().st_size for p in root.rglob('*') if p.is_file()),'result_sha256':sha((root/'result.json').read_bytes()),'rows_sha256':sha((root/'rows.jsonl').read_bytes()),'elapsed_s':r['elapsed_s'],'claimed_at':attempt['claimed_at'],'sealed_at':m['sealed_at'],'source_receipt':r['running_source'],'actual_source_hash_matches_submitted':{x['path']:sha(subprocess.check_output(['git','show','832c1c33:'+x['path']],cwd=REPO))==x['sha256'] for x in r['running_source']['source_files']},'artifact_hashes_match':{a['arm']:sha((root/'artifacts'/(a['arm']+('.rlds' if a['arm']=='value_only_lexical' else '.rlst'))).read_bytes())==a['artifact_sha256'] for a in r['artifacts']},'reported_arms':r['arms'],'reported_recount':r['recount'],'independent_h4_counts':recount(groups(rows)) if all('item_id' in x for x in rows) else 'NOT_RECONSTRUCTED: original rows lack unique item id'}

r=load(ROOT/'result.json');rows=[json.loads(l) for l in (ROOT/'rows.jsonl').read_text().splitlines()]; g=groups(rows);out['counts']=recount(g); out['all_h4_counts_match']=all(out['counts'][a['split']]['complete']==a['complete'] and out['counts'][a['split']]['token_hits']==a['token_hits'] and out['counts'][a['split']]['stopped_correct_length']==a['stopped_correctly'] for a in r['arms'] if a['arm']=='h4_shared_transition')
for p in (ROOT/'artifacts').glob('*.rlst'):out['artifacts'][p.stem]=rlst(p)
vo=ROOT/'artifacts/value_only_lexical.rlds';out['artifacts']['value_only_lexical']={'bytes':vo.stat().st_size,'sha256':sha(vo.read_bytes()),'reported_reload_identical':True}
h4=out['artifacts']['shared_transition_h4']; table={x['state']:x['top_token'] for x in h4['decoder_states']}
out['event_quality']={'rows':len(rows),'items':len(g),'all_primitive_indices_contiguous':all([x['primitive_index'] for x in xs]==list(range(len(xs))) for xs in g.values()),'all_item_fields_consistent':all(all(len({json.dumps(x[k]) for x in xs})==1 for k in ['split','value','read','selected_payload','source_abs','stopped']) for xs in g.values()),'emitted_rows_match_saved_artifact_decoder':all(x['emitted']==table.get(x['state']) for x in rows if x['step_kind']=='Emit'),'actual_emission_rows':sum(x['step_kind']=='Emit' for x in rows),'synthetically_recomputed_state_rows_after_unknown_value':sum(x['selected_payload'] not in h4['value_domain'] for x in rows),'comparison_arm_event_rows':0,'limits':['Only H4 saved events. No independent row-level C120, dictionary, value-only or local counts; cannot reconstruct paired intervals against comparators.','Rows omit full prefix, source sequence/version, actual pre-state, scores, learned control input/action, terminal Stop event.','State is recomputed separately with initial_state(...).unwrap_or(0) and apply(...).unwrap_or(state), including after actual serve stopped on UnknownValue; 16 state fields are counterfactual recomputation, not executed trajectory.']}

# Supervision-support audit, not a predictor: classify saved gold transitions by development support.
items=list(g.values()); dev=[xs for xs in items if xs[0]['split']=='dev'];initial=collections.defaultdict(set);trans=collections.defaultdict(set)
for xs in dev:
 initial[xs[0]['value'],xs[0]['primitive']].add(xs[0]['expected'])
 for prev,nxt in zip(xs,xs[1:]):trans[prev['expected'],nxt['primitive']].add(nxt['expected'])
coverage={'initial_pairs':len(initial),'state_label_primitive_pairs':len(trans),'initial_conflicts':sum(len(v)>1 for v in initial.values()),'state_transition_conflicts':sum(len(v)>1 for v in trans.values()),'output_labels':sorted({x['expected'] for xs in dev for x in xs}),'heldout_support':{}}
for split in out['counts']:
 xs=[z for z in items if z[0]['split']==split]; allsupported=[]
 for z in xs:
  support=[z[0]['expected'] in initial.get((z[0]['value'],z[0]['primitive']),set())]+[b['expected'] in trans.get((a['expected'],b['primitive']),set()) for a,b in zip(z,z[1:])]
  allsupported.append((z,support))
 coverage['heldout_support'][split]={'items':len(xs),'all_steps_supported_by_consistent_dev_labels':sum(all(v) for _,v in allsupported),'supported_steps':sum(sum(v) for _,v in allsupported),'total_steps':sum(len(v) for _,v in allsupported),'supported_with_correct_actual_source':sum(all(v) and z[0]['source_abs']==1 and z[0]['selected_payload']==z[0]['value'] for z,v in allsupported)}
coverage['interpretation']='A development-fitted ordinary finite-state transducer is identified by 32 input-value/first-primitive transitions plus 64 output-label/next-primitive transitions, all consistent. By induction it can reconstruct every authored gold-operand trajectory; through actual reader, potential exact support is124/128 length4 and64/64 reversal. This is a supervision-coverage argument, not a newly executed comparator or fresh performance result. The submitted whole-sequence dictionary cannot test this shared-state alternative.'
out['development_transition_support']=coverage
cal=[xs for xs in dev if xs[0]['item_id']%2==0];probe=[xs for xs in dev if xs[0]['item_id']%2==1];seq=lambda xs:tuple(x['primitive'] for x in xs);ds={seq(xs) for xs in dev};fs={seq(xs) for xs in cal};ps={seq(xs) for xs in probe}
out['population_and_exposure']={'calibration_items':len(cal),'probe_items':len(probe),'calibration_values':sorted({xs[0]['value'] for xs in cal}),'probe_values':sorted({xs[0]['value'] for xs in probe}),'calibration_unique_sequences':len(fs),'probe_unique_sequences':len(ps),'shared_calibration_probe_sequences':len(fs&ps),'is_sequence_disjoint':not bool(fs&ps),'heldout_sequence_overlap_dev':{split:len({seq(xs) for xs in items if xs[0]['split']==split}&ds) for split in ['held_out_length4','held_out_reversal']},'same_sealed_final_rows_sha_runs_3_through_6':len({out['roots']['shared-transition-'+str(i)]['rows_sha256'] for i in range(3,7)})==1,'scope':'Exact heldout primitive sequences do not appear in development, but all primitive transition supervision is covered. Fixed population was repeatedly exposed across design changes: NOT_FRESH_FINAL. The purported sequence-disjoint development probe is actually a value split with all72 sequences shared.'}

out['generation']=[]
for z in r['generated']:
 matches=[xs for xs in items if xs[0]['split']==z['split'] and xs[0]['value']==z['value'] and seq(xs)==tuple(z['primitives'])]; exact=z['stopped'] and z['emitted']==z['expected']
 out['generation'].append({'value':z['value'],'primitives':z['primitives'],'emitted':z['emitted'],'expected':z['expected'],'steps':z['steps'],'complete_recounted':exact,'flag_matches':z['complete_correct']==exact,'matches_saved_evaluation':len(matches)==1 and [x['emitted'] for x in matches[0]]==z['emitted'],'all_states_match_saved_evaluation':len(matches)==1 and [x['state'] for x in matches[0]]==z['states']})
out['generation_scope']={'examples':4,'unique_primitive_sequences':len({tuple(z['primitives']) for z in r['generated']}),'complete_recounted':sum(z['complete_recounted'] for z in out['generation']),'lexical_text_not_retained':True,'stop_token_is_emitted':False,'note':'Four input values on one ordered primitive sequence. Typed response returned all tokens at once, no separate model-step lifecycle and no emitted-feedback perturbation.'}
out['interventions_reported']=r['interventions']
out['intervention_audit']={'payload_changed':'Calls h4.serve on fixture gold values directly. Valid computation-only two-value control, bypasses actual reader/occurrence selection.','primitive_order':'Both named order controls use h4.serve directly with intended operand. Source-prefix repl constructed but unused. Population reversal all64 source-correct, so its low score is not explained by reader errors.','retained_state':'Two identical h4.serve calls; proves repeatability, not an intervention. Source confirms local state variable retained within a serve call and emissions unused as input.','absent_source':'Calls st_read_and_serve with use_reader=false on unchanged source-present prefix. ReadDisabled, not true source-removal or eviction.','learned_stop':'fit_stop_policy authors supervision with remaining==0, unobserved remaining defaults Continue, and serve hard-stops when j==n even if learned table did not stop. No learned scheduling/termination advantage can be inferred. All arms inherit this policy; differing stop counts arise from earlier missing-state/unknown exits.','noncommuting_witness':'Learned-code state72 versus90 supports representational order sensitivity. Saved comparison lacks fixture targets for this specifically chosen witness, so it is not standalone correct order-reversal generalization.','valid_identity':'No actual identity-versus-absence control in this run.','dependent_read':'NOT_RUN','irrelevant_distractor':'NOT_RUN','actual_read_disabled_full_local_logits':'NOT_RUN','unknown_value_and_primitive':'Direct computation typed outcomes recorded; no serialized event record for independent replay.'}
exe=P(load(ROOT/'attempt.json')['argv'][0]);out['primary_provenance']={'executable_path':str(exe),'executable_exists':exe.exists(),'actual_executable_sha256':sha(exe.read_bytes()) if exe.exists() else None,'reported_executable_sha256':r['running_source']['executable_sha256'],'executable_matches':sha(exe.read_bytes())==r['running_source']['executable_sha256'] if exe.exists() else False,'all_source_files_match_submitted_head':all(out['roots'][ROOT.name]['actual_source_hash_matches_submitted'].values()),'stale_git_stamp':'All running git stamps claim clean86623138 although new files/changes are absent at that revision. Source-file hashes bind submitted832c1c33 for primary; old git stamp is not an honest complete source identity.'}
inputs={'E_sha256':BASE/'head-projection-3/corrected/empirical.cpl2','S_sha256':BASE/'s-attribution-3/corrected/separable_older_query_read.cpx3','selector_sha256':BASE/'relational-learning-4/artifacts/relational_ctx.rlr2'}
out['input_hashes']={k:{'path':str(p),'actual_sha256':sha(p.read_bytes()),'matches':sha(p.read_bytes())==r['inputs'][k]} for k,p in inputs.items()}
ledger=P('/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json'); disk=shutil.disk_usage('/Users/casey.allard/uor-r4');elapsed=sum(x['elapsed_s'] for x in out['roots'].values());out['resources']={'live_ledger_at_audit':load(ledger),'measured_six_run_elapsed_s':elapsed,'retained_root_bytes':sum(x['retained_bytes'] for x in out['roots'].values()),'reported_increment_ms':10000000,'reported_charge_ms':9000000,'ledger_arithmetic_note':'Approximate context, implementation, repair, documentation and build components sum9Mms, with41.6768565s model elapsed separately listed. Given approximate components, do not infer a definite undercharge; retain9M recorded debit and label mixed estimates.','live_filesystem_free_bytes':disk.free,'physical_reserve_bytes':36766079385,'stop_margin_bytes':134217728,'free_above_reserve_and_margin_bytes':disk.free-36766079385-134217728,'scope':'Ledger charge combines estimated authoring/recovery/build work with ~42s measured model runs. Estimates are not measured model time, latency or energy; audit made no inference/build/deletion.'}
out['decision_scope']=['Useful artifact-bound learned shared-action recurrence and lexicalization exist; recorded H4 emits244 exact complete authored responses across480 fixed items.','Do not promote48vs24vs0 as geometry/ordinary-state advantage. C120 counts are report-only; whole-sequence cache is an incompetent comparator for shared-state transfer; development determines a fully supported ordinary transition table.','Do not blame reversed-order failure on reader selection in the saved primary population:64/64 correct sources,10/64 exact responses.','The bottleneck is not established as solely source ownership: length4 already124/128 correct sources and48 complete. Improve ordinary shared-state comparator plus action/state learning, while separating explicit source ownership from observed control is still necessary for general instruction interfaces.','Learned scheduler, dependent read, true missing-source test, emission perturbation, fresh independent final, ordinary language and energy remain NOT_RUN/UNAVAILABLE.']
out['script_sha256']=sha(P(__file__).read_bytes());dest=P('/tmp/uor-pr1339-evidence-audit.json');dest.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'output':str(dest),'all_h4_counts_match':out['all_h4_counts_match'],'counts':out['counts'],'development_transition_support':coverage,'population_and_exposure':out['population_and_exposure'],'primary_provenance':out['primary_provenance'],'resources':out['resources']},indent=2))
