#!/usr/bin/env python3
"""Read-only PR1342 audit of saved artifacts, counts and authored target reconstruction; no model run."""
import collections,copy,hashlib,json,pathlib,subprocess
import blake3
P=pathlib.Path;BASE=P('/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20');REPO=P('/Users/casey.allard/uor-r4/.worktrees/observed-text-session');HEAD='b9eca5d2759d86664e519a9fb4c875b8127cb914'
sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads(p.read_text())
out={'schema':'uor-r4.pr1342-independent-evidence-audit/1','scope':__doc__,'submitted_head':HEAD,'roots':{}}
for n in range(1,9):
 root=BASE/f'observed-text-session-{n}';r=load(root/'result.json');m=load(root/'manifest.json');xs=[json.loads(x) for x in (root/'rows.jsonl').read_text().splitlines()]
 listed={f['path'] for f in m['files']};actual={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}-{'manifest.json'}
 out['roots'][root.name]={'claimed_at':load(root/'attempt.json')['claimed_at'],'sealed_at':m['sealed_at'],'listed':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'size_errors':[f['path'] for f in m['files'] if (root/f['path']).stat().st_size!=f['bytes']],'blake3_errors':[f['path'] for f in m['files'] if blake3.blake3((root/f['path']).read_bytes()).hexdigest()!=f['blake3']],'rows_sha256':sha((root/'rows.jsonl').read_bytes()),'result_sha256':sha((root/'result.json').read_bytes()),'reported_arms':r['arms'],'primary_recount_correct':sum(x['emitted']==x['expected_answer']+[r['task']['eos']] and x['terminal']=='Stop' for x in xs),'primary_recount_depth':sum(x['reads']==x['expected_depth'] for x in xs),'correct_flags_match':all(x['correct']==(x['emitted']==x['expected_answer']+[r['task']['eos']] and x['terminal']=='Stop') for x in xs),'source_matches_submitted':{f['path']:sha(subprocess.check_output(['git','show',HEAD+':'+f['path']],cwd=REPO))==f['sha256'] for f in r['running_source']['source_files']},'running_source':r['running_source'],'model_sha256':sha((root/'artifacts/observed_text_model.json').read_bytes()),'elapsed_s':r['elapsed_s'],'retained_bytes':sum(p.stat().st_size for p in root.rglob('*') if p.is_file()),'interventions':r['interventions']}
ROOT=BASE/'observed-text-session-8';r=load(ROOT/'result.json');task=r['task'];xs=[json.loads(x) for x in (ROOT/'rows.jsonl').read_text().splitlines()];model=load(ROOT/'artifacts/observed_text_model.json');EOS=task['eos'];persons=task['persons'];markers=task['markers'];offices=task['offices'];projects=task['projects'];exe=P('/tmp/uor-pr1342-original-competitive-reader')
out['provenance']={'preserved_executable':str(exe),'executable_bytes':exe.stat().st_size,'executable_sha256':sha(exe.read_bytes()),'matches_run8':sha(exe.read_bytes())==r['running_source']['executable_sha256'],'all_run8_source_hashes_match':all(out['roots']['observed-text-session-8']['source_matches_submitted'].values()),'note':'Git stamp names the reviewed parent; individual source-file hashes bind the submitted new implementation.'}
MASK=(1<<64)-1
def world(wid,ver,ochains,pchains,target_key,seed):
 st=seed|1;clauses=[];labels=[]
 def rand():
  nonlocal st
  st^=(st<<13)&MASK;st^=st>>7;st^=(st<<17)&MASK;return st
 for goal,chains in [('Office',ochains),('Project',pchains)]:
  for chain in chains:
   candidates=persons if target_key and goal=='Office' else offices if goal=='Office' else projects;target=candidates[rand()%len(candidates)]
   for hop,p in enumerate(chain):
    last=hop+1==len(chain);role=(0 if goal=='Office' else 2)+(0 if last else 1);obj=target if last else persons[chain[hop+1]];seg=len(clauses);sub=persons[p];tokens=sub+markers[role]+obj;clauses.append({'seg':seg,'tokens':tokens});labels.append({'seg':seg,'subject':[0,len(sub)],'marker':[len(sub),2],'object':[len(sub)+2,len(obj)],'role':role,'goal':goal,'action':'Emit' if last else 'Continue'})
 return {'id':wid,'version':ver,'clauses':clauses,'labels':labels}
dev=[world(0,10,[[0,1],[2,3]],[[0],[1,2],[3]],False,0xC0F60001),world(1,11,[[0],[1,2],[3]],[[0,1],[2,3]],False,0xC0F60001+0x9E37),world(2,12,[[0,1],[2,3]],[[0,2],[1,3]],False,0xC0F60001+2*0x9E37)]
final=[world(100,300,[[0,1,2],[3]],[[0],[1,2,3]],False,0xC0F60101),world(101,301,[[0],[1,2,3]],[[0,1,2],[3]],False,0xC0F60101+0x9E37),world(102,302,[[0,1,2],[3]],[[0],[1,2,3]],True,0xC0F60101+2*0x9E37)]
def expected(w,p,goal):
 query=persons[p];visited=[];subjects=[];rolepath=[]
 for depth in range(1,7):
  found=None
  for c,l in zip(w['clauses'],w['labels']):
   ss,sl=l['subject']
   if l['goal']==goal and c['tokens'][ss:ss+sl]==query:found=(c,l);break
  if found is None:return {'answer':[],'terminal':'Unresolved','reads':len(visited),'segments':visited,'subjects':subjects,'roles':rolepath}
  c,l=found;os,ol=l['object'];obj=c['tokens'][os:os+ol]
  if c['seg'] in visited:return {'answer':[],'terminal':'Exhausted','reads':len(visited),'segments':visited,'subjects':subjects,'roles':rolepath}
  visited.append(c['seg']);subjects.append(query);rolepath.append(l['role'])
  if l['action']=='Emit':return {'answer':obj,'terminal':'Stop','reads':depth,'segments':visited,'subjects':subjects,'roles':rolepath}
  query=obj
 return {'answer':[],'terminal':'Exhausted','reads':6,'segments':visited,'subjects':subjects,'roles':rolepath}
checks=[]
for x in xs:
 ex=expected(final[x['world']-100],x['person'],x['goal']);subject_events=[]
 for subject in ex['subjects'][:-1]:subject_events += [subject[0]]*2
 subject_events += [ex['subjects'][-1][0]]*(1+len(ex['answer']))
 checks.append({'independent_answer_matches':x['expected_answer']==ex['answer'],'emitted_matches':x['emitted']==ex['answer']+[EOS] and x['terminal']==ex['terminal'],'depth_matches':x['reads']==ex['reads']==x['expected_depth'],'selected_role_path_matches':x['selected_roles']==ex['roles'],'subject_event_array_matches':x['selected_subjects']==subject_events,'eos_exactly_once_at_end':x['emitted'].count(EOS)==1 and x['emitted'][-1]==EOS})
out['primary_recount']={'rows':len(xs),'distinct_worlds':3,'checks_true_counts':{k:sum(x[k] for x in checks) for k in checks[0]},'depth_counts':dict(collections.Counter(x['reads'] for x in xs)),'decoded_answer_counts':dict(collections.Counter(x['emitted_text'] for x in xs)),'answer_token_lengths':dict(collections.Counter(len(x['emitted'])-1 for x in xs)),'decoded_outputs_with_whitespace':sum(any(c.isspace() for c in x['emitted_text']) for x in xs),'eos':EOS,'terminal_frames_with_out_of_vocab_emission':sum(any(t>=4096 for t in x['emitted']) for x in xs),'note':'All24 rows are correct multi-token spans for the declared token task. Nine two-token spans decode as single no-space strings, not demonstrated multiword answers.'}
novel=[]
for p in range(4):
 for goal in ['Office','Project']:
  dd=[expected(w,p,goal)['reads'] for w in dev];fd=[expected(w,p,goal)['reads'] for w in final]
  novel.append({'person':p,'goal':goal,'question_tokens':persons[p]+markers[0 if goal=='Office' else 2],'development_depths':dd,'final_depths':fd,'all_three_depths_across_both':set(dd+fd)=={1,2,3},'all_three_depths_within_final':set(fd)=={1,2,3}})
out['novelty']={'requests':novel,'development_read_counts':dict(collections.Counter(expected(w,p,g)['reads'] for w in dev for p in range(4) for g in ['Office','Project'])),'final_read_counts':dict(collections.Counter(expected(w,p,g)['reads'] for w in final for p in range(4) for g in ['Office','Project'])),'depth3_absent_development':all(expected(w,p,g)['reads']<3 for w in dev for p in range(4) for g in ['Office','Project']),'familiar_marker_vocabulary':True,'note':'Depth3 is held out of fit supervision, but not kept unexposed through design. Final chain structures are explicitly authored arrays; only terminal assignments use fixed PRNG draws.'}
out['exposure']={'run_primary_final_correct':[out['roots'][f'observed-text-session-{i}']['primary_recount_correct'] for i in range(1,9)],'roots5_through8_rows_identical':len({out['roots'][f'observed-text-session-{i}']['rows_sha256'] for i in range(5,9)})==1,'membership_reported_final':[next(a['complete'] for a in out['roots'][f'observed-text-session-{i}']['reported_arms'] if a['arm']=='membership_continuation') for i in range(1,9)],'qualification':'Repeated exposed development diagnostics. Final output improved7->18->24 while source/instrument/model changed; membership control repaired24->20 after primary perfect result. Fresh final confirmation not established.'}
# Count gold observation-label statistics independently; this verifies artifact receipts, not fitted serving.
tv=collections.defaultdict(lambda:[0,0,0,0]);edges=collections.Counter();actionlabels=[];goal_by_role=collections.defaultdict(set);annotated=0
for w in dev:
 cls=copy.deepcopy(w['clauses']);labs=copy.deepcopy(w['labels'])
 for p in range(4):
  for g in ['Office','Project']:
   role=0 if g=='Office' else 2;cls.append({'seg':999,'tokens':persons[p]+markers[role]});labs.append({'subject':[0,1],'marker':[1,2],'object':None,'role':role,'goal':g,'action':'Emit'})
 for c,l in zip(cls,labs):
  annotated+=1;ms,ml=l['marker']
  for t in c['tokens'][ms:ms+ml]:tv[t][l['role']]+=1
  ss,sl=l['subject'];edges.update(c['tokens'][ss:ss+sl])
  if ss>0:edges[c['tokens'][ss-1]]-=1
  if l['object']:
   os,ol=l['object'];edges.update(c['tokens'][os:os+ol]);edges[c['tokens'][os-1]]-=1;actionlabels.append(l['action'])
  goal_by_role[l['role']].add(l['goal'])
out['learning_audit']={'annotated_clauses':annotated,'statement_actions':len(actionlabels),'all_emit_initial_correct':actionlabels.count('Emit'),'gold_action_counts':dict(collections.Counter(actionlabels)),'marker_votes_equal_label_counts':model['token_votes']==[[t,v] for t,v in sorted(tv.items())],'boundary_votes_equal_label_counts':model['edge_votes']==[[t,v] for t,v in sorted(edges.items()) if v],'action_table':[3,2,3,2],'artifact_action_table_matches':model['action_per_role']==[3,2,3,2],'goal_table_matches':model['goal_per_role']==[0,0,1,1],'negative_edge_tokens':[t for t,v in model['edge_votes'] if v<0],'negative_edges_are_already_removed_marker_tails':all(t in [m[-1] for m in markers] for t,v in model['edge_votes'] if v<0),'actual_filler_boundary_cases':0,'marker_codebook_scope':'All four roles use disjoint marker-token pairs. Gold marker extents, subject/object boundaries, role, action and goal supervise offline counts. Serving sees a fitted categorical codebook; no unseen wording or competing same-goal syntax tested.'}
w=final[0];edit=copy.deepcopy(w);edit['clauses'][0]['tokens'][3:]=persons[3];edit['labels'][0]['object']=[3,1];before=expected(w,0,'Office');after=expected(edit,0,'Office');changed=[(i,j,a,b) for i,(ca,cb) in enumerate(zip(w['clauses'],edit['clauses'])) for j,(a,b) in enumerate(zip(ca['tokens'],cb['tokens'])) if a!=b]
removed=copy.deepcopy(w);del removed['clauses'][4];del removed['labels'][4]
out['interventions_audit']={'changed_observed_token_positions':changed,'before':before,'after':after,'saved_answer_after_matches':r['interventions']['one_position_source_edit']['emitted_after']==after['answer']+[EOS],'saved_depths_match':r['interventions']['one_position_source_edit']['reads_before']==before['reads'] and r['interventions']['one_position_source_edit']['reads_after']==after['reads'],'goal_invariant_receipt':'Recorded boolean is only first action==Continue, not an audited goal trace. Source Continue explicitly copies self.goal, but no frames retained.','selected_receipt':'Arrays contain subject first-token IDs repeated per read/continue/emit effect, not unique exact occurrence references. Reconstructed underlying segment path [0,1,2]->[0,3].','removed_test_original_path':expected(w,0,'Project'),'removed_test_result':expected(removed,0,'Project'),'removed_is_later_fact':False,'cycle_and_unknown':'Cycle and unknown subject supported in module tests only, not retained in run8 report.'}
out['interface_and_controls']={'fixed_depth_two':'Control enforces at most2 reads only when pendingRead andreads>=2; it retains learned earlier Emit.18/24 therefore max-two-read truncation, not a pure fixed-depth baseline.','membership':'Corrected after run6; replaces action after each read based on declared registry. Only aggregate20/24,22correctdepth retained; comparator rows absent.','reads_disabled':'Overrides pendingRead toStop, causingEOS emission and terminalStop with no answer tokens, rather than explicit unresolved. Report0complete is correct but no-read semantics need repair.','goal_input':'ob_serve receives supplied Goal, uses it to create the question and initialize session; observed qgoal is checked against it. No evidence of a mismatched goal changing a correct prediction, but claimed raw-input-only API is not implemented.','binding':'Model/tokenizer/document digests are literal bound placeholders; only worldid/version are real. Foreign model/tokenizer/document rejection is not established by this runner.','step_validation':'step does not validate expected binding or frame invariants; validate is called at start and a single resume point. EOS4294967294 outside4096 makes every completed emitted frame fail validate.','resume':'Inline serde roundtrip after step1 in the same loop; no saved snapshots, frame events, independent continuation entry, midphrase or postEOS restoration. Counts compare only emitted andterminal, not reads/control/selection.','loaded_model':'Artifact JSON independently deserialized, equal to original, and loaded object used by serving; no pre/post full-predictor comparison beyond parameter equality.','all_arm_rows':'Only24 primary final rows retained; development/control/resume full rows not retained.','span_identity':'Observed object_start derives totaltrim count rather than leadingtrim only, so trailing fillers shift origin incorrectly; current fixture has no actual filler case. CapturedPayload.abs lacks segment but captured_span stores segment.','step_budget':'TextSession::run uses fixed2*MAX_READS+4 cap unrelated to phrase length and returnsOk possibly nonterminal. Wrapper maps nonterminal toUnresolved even with partial emission. Current1-2token answers do not expose this.','prose_and_teacher_forcing':'Fixture constructs token arrays directly from banks; decoded strings are arbitrary fragments. Runner performs complete rollout exact-answer comparison, not a separate teacher-forced predictive-language test.'}
out['artifact']={'path':str(ROOT/'artifacts/observed_text_model.json'),'bytes':(ROOT/'artifacts/observed_text_model.json').stat().st_size,'sha256':sha((ROOT/'artifacts/observed_text_model.json').read_bytes()),'model':model}
out['resources']={'eight_saved_run_elapsed_s':sum(x['elapsed_s'] for x in out['roots'].values()),'retained_report_bytes':sum(x['retained_bytes'] for x in out['roots'].values()),'reported_charge_ms':18000000,'listed_component_estimates_sum_ms':18000000,'charge_scope':'Approximate measured-plus-estimated whole work charge, not model-run wall time. No independent cleanup manifest audited here.','model_runs_by_auditor':0,'builds_by_auditor':0,'energy':'UNAVAILABLE'}
out['script_sha256']=sha(P(__file__).read_bytes());dest=P('/tmp/uor-pr1342-evidence-audit.json');dest.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'output':str(dest),'provenance':out['provenance'],'primary_recount':out['primary_recount'],'novelty':out['novelty'],'exposure':out['exposure'],'learning_audit':out['learning_audit'],'interventions':out['interventions_audit'],'resources':out['resources']},indent=2))
