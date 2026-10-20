#!/usr/bin/env python3
"""Independent saved-data audit of corrected observed-text runtime; no fitting, model run or build."""
import collections,copy,hashlib,json,pathlib,sys
import blake3
P=pathlib.Path
ROOT=P(sys.argv[1] if len(sys.argv)>1 else '/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/observed-text-session-principal-1')
REPO=P('/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4')
sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads(p.read_text());compact=lambda v:json.dumps(v,separators=(',',':'),ensure_ascii=False).encode()
r=load(ROOT/'result.json');worlddata=load(ROOT/'worlds.json');rows=[json.loads(s) for s in (ROOT/'rows.jsonl').read_text().splitlines()];m=load(ROOT/'manifest.json');modelb=(ROOT/'artifacts/observed_text_model.json').read_bytes();model=json.loads(modelb);EOS=r['task']['eos'];VOCAB=r['task']['max_vocab'];persons=r['task']['persons'];markers=r['task']['markers']
listed={f['path'] for f in m['files']};actual={str(p.relative_to(ROOT)) for p in ROOT.rglob('*') if p.is_file()}-{'manifest.json'}
out={'schema':'uor-r4.pr1342-corrected-independent-audit/1','scope':__doc__,'root':str(ROOT),'seal':{'listed':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'size_errors':[f['path'] for f in m['files'] if (ROOT/f['path']).stat().st_size!=f['bytes']],'blake3_errors':[f['path'] for f in m['files'] if blake3.blake3((ROOT/f['path']).read_bytes()).hexdigest()!=f['blake3']]},'provenance':{'result_sha256':sha((ROOT/'result.json').read_bytes()),'model_sha256':sha(modelb),'model_bytes':len(modelb),'source_matches_review':{f['path']:sha((REPO/f['path']).read_bytes())==f['sha256'] for f in r['running_source']['source_files']},'running_source':r['running_source']}}
exe=P(load(ROOT/'attempt.json')['argv'][0]);out['provenance']['actual_executable_matches']=sha(exe.read_bytes())==r['running_source']['executable_sha256']
worlds={w['id']:w for split in ['development','exposed_regression'] for w in worlddata[split]}
def clausebytes(clauses):return compact([{'seg':c['seg'],'tokens':c['tokens']} for c in clauses])
def control(arm):
 return {'membership_continuation':{'Membership':{'entities':persons}},'read_cap_two':{'ReadCap':{'max_reads':2}},'reads_disabled':'ReadsDisabled'}.get(arm,'Normal')
def expected(world,query,goal):
 clauses={c['seg']:c for c in world['clauses']};visited=[];roles=[];current=query;tokens=[]
 for _ in range(6):
  found=None
  for label in world['labels']:
   if label['seg'] not in clauses:continue
   c=clauses[label['seg']];ss,sl=label['subject']
   if label['goal']==goal and c['tokens'][ss:ss+sl]==current:found=(c,label);break
  if found is None:return {'answer':[],'terminal':'Unresolved','reads':len(visited),'selected_segments':[s[0] for s in visited],'roles':roles}
  c,l=found;start,length=l['object'];span=[c['seg'],start,length]
  if span in visited:return {'answer':[],'terminal':'Exhausted','reads':len(visited),'selected_segments':[s[0] for s in visited],'roles':roles}
  visited.append(span);roles.append(l['role']);tokens=c['tokens'][start:start+length]
  if l['action']=='Emit':return {'answer':tokens,'terminal':'Stop','reads':len(visited),'selected_segments':[s[0] for s in visited],'roles':roles}
  current=tokens
 return {'answer':[],'terminal':'Exhausted','reads':len(visited),'selected_segments':[s[0] for s in visited],'roles':roles}
event_errors=[];snapshot_errors=[];phasecounts=collections.Counter();totals=collections.Counter();byarm=collections.defaultdict(list)
def check_frame(frame,world,arm):
 errors=[];binding=frame['binding'];doc=sha(clausebytes(world['clauses']));ctrlsha=sha(compact(control(arm)))
 for field,want in [('model_sha256',sha(modelb)),('tokenizer_sha256',r['inputs']['tokenizer_derived']),('world_id',world['id']),('world_version',world['version']),('doc_sha256',doc),('control_sha256',ctrlsha),('max_vocab',VOCAB),('eos',EOS)]:
  if binding[field]!=want:errors.append('binding:'+field)
 if frame['goal']!=frame['query']['goal']:errors.append('goal changed')
 if frame['reads']!=len(frame['visited']) or len({tuple(v) for v in frame['visited']})!=len(frame['visited']):errors.append('read history')
 if any(t>=VOCAB for t in frame['captured_tokens']+frame['query']['tokens']):errors.append('ordinary token outside vocab')
 cap=frame['captured']
 if cap is not None:
  c=next((c for c in world['clauses'] if c['seg']==cap['segment']),None)
  if c is None or c['tokens'][cap['start']:cap['start']+cap['len']]!=frame['captured_tokens']:errors.append('owned exact source span')
  if cap['doc_sha256']!=doc or cap['world_id']!=world['id'] or cap['world_version']!=world['version']:errors.append('capture identity')
  if frame['captured_span']!=[cap['segment'],cap['start'],cap['len']] or frame['visited'][-1]!=frame['captured_span']:errors.append('capture extent/history')
 elif frame['captured_tokens'] or frame['reads'] or frame['captured_span'] is not None:errors.append('capture absent with content')
 stop=frame['terminal']=='Stop';expectedem=frame['captured_tokens'][:frame['cursor']]+([EOS] if stop else [])
 if frame['emitted']!=expectedem:errors.append('emitted prefix/EOS')
 if stop and (not frame['captured_tokens'] or frame['cursor']!=len(frame['captured_tokens'])):errors.append('incomplete Stop')
 if frame['terminal'] in ['Unresolved','Exhausted'] and frame['emitted']:errors.append('partial failure emission')
 return errors
def check_events(events,world,arm,tag):
 errs=[];reads=0;attempts=0;emissions=[];selected=[]
 for i,e in enumerate(events):
  b=e['before'];a=e['after'];eff=e['effect'];errs.extend([[i,'before:'+err] for err in check_frame(b,world,arm)]);errs.extend([[i,'after:'+err] for err in check_frame(a,world,arm)])
  if i and b!=events[i-1]['after']:errs.append([i,'frame discontinuity'])
  if b['goal']!=a['goal']:errs.append([i,'active goal mutation'])
  if a['reads']>b['reads']:
   reads+=1;selected.append(a['captured']['segment'])
   if eff['action']!='Read' or a['reads']!=b['reads']+1 or eff['selected_segment']!=a['captured']['segment']:errs.append([i,'successful read effect'])
  elif eff['action']=='Read' and eff['selected_segment'] is not None:attempts+=1
  if eff['emitted'] is not None:emissions.append(eff['emitted'])
  if eff['terminal']!=a['terminal']:errs.append([i,'effect terminal'])
  if eff['next_action']!=(a['pending'] if a['terminal'] is None else None):errs.append([i,'next phase'])
 if events and emissions!=events[-1]['after']['emitted']:errs.append(['final','effect emissions disagree'])
 if errs:event_errors.append({'tag':tag,'errors':errs})
 totals['events']+=len(events);totals['successful_reads']+=reads;totals['repeated_read_attempts']+=attempts
 return reads,selected
for idx,row in enumerate(rows):
 w=worlds[row['world']];arm=row['arm'];tag=[arm,row['split'],row['world'],row['person'],row['goal']];ex=expected(w,persons[row['person']],row['goal']);want=ex['answer']+([EOS] if ex['terminal']=='Stop' else []);ok=row['emitted']==want and row['terminal']==ex['terminal'];byarm[arm,row['split']].append({'ok':ok,'depth':row['reads']==ex['reads'],'flag':ok==row['correct'],'expected':row['expected_answer']==ex['answer'] and row['expected_depth']==ex['reads']})
 actualreads,selected=check_events(row['events'],w,arm,tag)
 if row['question']['tokens']!=persons[row['person']]+markers[0 if row['goal']=='Office' else 2] or row['events'][0]['before']['query']['tokens']!=persons[row['person']] or row['events'][0]['before']['goal']!=row['goal']:event_errors.append({'tag':tag,'errors':['observed question/initial objective mismatch']})
 if actualreads!=row['reads'] or row['final_frame']!=row['events'][-1]['after'] or row['final_frame']['emitted']!=row['emitted'] or row['final_frame']['terminal']!=row['terminal']:event_errors.append({'tag':tag,'errors':['row/finalframe/read count mismatch']})
 if row['selected_segments']!=selected:
  # Legacy source may expose repeated cycle Read attempts as selections; keep separate from read count.
  attempted=[e['effect']['selected_segment'] for e in row['events'] if e['effect']['action']=='Read' and e['effect']['selected_segment'] is not None]
  if row['selected_segments']!=attempted:event_errors.append({'tag':tag,'errors':['selected segments mismatch']})
 for s in row['snapshots']:
  at=s['checkpoint_index'];bs=s['snapshot_utf8'].encode();f=json.loads(bs);phasecounts['terminal:'+f['terminal'] if f['terminal'] else f['pending']]+=1;errs=[]
  if sha(bs)!=s['snapshot_sha256']:errs.append('snapshot SHA')
  expectedframe=row['events'][at]['before'] if at<len(row['events']) else row['final_frame']
  if f!=expectedframe:errs.append('snapshot boundary')
  if s['resumed_suffix']!=row['events'][at:] or s['resumed_final_frame']!=row['final_frame'] or not s['identical']:errs.append('resumed continuation')
  errs+=check_frame(f,w,arm)
  if errs:snapshot_errors.append({'tag':tag,'checkpoint':at,'errors':errs})
  totals['snapshots']+=1
 if len(row['snapshots'])!=len(row['events'])+1 or row['resume_identical']!=len(row['snapshots']):snapshot_errors.append({'tag':tag,'errors':['missing phase checkpoint/count']})
out['arm_recounts']={arm+'/'+split:{'rows':len(v),'correct':sum(x['ok'] for x in v),'correct_depth':sum(x['depth'] for x in v),'all_flags_match':all(x['flag'] for x in v),'all_expected_targets_match':all(x['expected'] for x in v)} for (arm,split),v in byarm.items()}
out['reported_counts_match']={a['arm']+'/'+a['split']:out['arm_recounts'][a['arm']+'/'+a['split']]['correct']==a['complete'] and out['arm_recounts'][a['arm']+'/'+a['split']]['rows']==a['total'] and out['arm_recounts'][a['arm']+'/'+a['split']]['correct_depth']==a['depth_correct'] for a in r['arms']}
out['row_event_totals']=dict(totals)
out['snapshot_counts_by_arm']={arm+'/'+split:sum(len(x['snapshots']) for x in rows if x['arm']==arm and x['split']==split) for arm,split in byarm}
primary_snapshot_total=sum(len(x['snapshots']) for x in rows if x['arm']=='observed_text_session')
out['reported_primary_resume_count_matches']=r['interventions']['resume_identical']=={'identical':primary_snapshot_total,'total':primary_snapshot_total}
out['snapshot_phase_counts']=dict(phasecounts);out['event_errors']=event_errors;out['snapshot_errors']=snapshot_errors
original=[json.loads(s) for s in (ROOT.parent/'observed-text-session-8/rows.jsonl').read_text().splitlines()];primary={(x['world'],x['person'],x['goal']):x for x in rows if x['arm']=='observed_text_session' and x['split']=='exposed_regression'}
out['original_preservation']={'rows':len(original),'matching_outputs_depth_roles':sum(all(a[k]==primary[a['world'],a['person'],a['goal']][k] for k in ['emitted','reads','selected_roles','terminal']) for a in original)}
ints=r['interventions'];ci=ints['one_position_source_edit'];base=worlds[ci['world']];bw=copy.deepcopy(base);bw['clauses']=ci['before_clauses'];aw=copy.deepcopy(base);aw['clauses']=ci['after_clauses'];be=expected(bw,persons[ci['person']],ci['goal']);ae=expected(aw,persons[ci['person']],ci['goal']);changed=[{'segment':a['seg'],'offset':i,'before':x,'after':y} for a,b in zip(bw['clauses'],aw['clauses']) for i,(x,y) in enumerate(zip(a['tokens'],b['tokens'])) if x!=y]
check_events(ci['before_events'],bw,'observed_text_session','source before');check_events(ci['after_events'],aw,'observed_text_session','source after')
out['source_intervention']={'changed_tokens':changed,'before_expected':be,'after_expected':ae,'answers_match':ci['emitted_before']==be['answer']+[EOS] and ci['emitted_after']==ae['answer']+[EOS],'selection_depth_match':ci['selected_before']==be['selected_segments'] and ci['selected_after']==ae['selected_segments'] and ci['reads_before']==be['reads'] and ci['reads_after']==ae['reads'],'goal_invariant_verified':all(e[k]['goal']==ci['goal'] and e[k]['query']['goal']==ci['goal'] for e in ci['after_events'] for k in ['before','after'])}
rm=ints['required_later_fact_removed'];rw=copy.deepcopy(base);rw['clauses']=rm['after_clauses'];rex=expected(rw,persons[0],'Office');check_events(rm['outcome']['events'],rw,'observed_text_session','later fact removed');out['downstream_absence']={'removed_segment':rm['removed_segment'],'original_expected_path':be['selected_segments'],'is_terminal_fact_after_two_redirects':len(be['selected_segments'])==3 and rm['removed_segment']==be['selected_segments'][-1],'expected':rex,'reported_successful_reads':rm['successful_reads_before_absence'],'outcome_matches':rm['outcome']['emitted']==rex['answer'] and rm['outcome']['terminal']==rex['terminal'] and rm['successful_reads_before_absence']==rex['reads']}
cy=ints['cycle'];cw=copy.deepcopy(base);cw['clauses']=cy['clauses'];cex=expected(cw,persons[0],'Office');cyreads,cysel=check_events(cy['events'],cw,'observed_text_session','cycle');out['cycle']={'expected':cex,'actual_successful_reads':cyreads,'terminal':cy['terminal'],'emitted':cy['emitted'],'matches':cy['terminal']==cex['terminal'] and cy['emitted']==cex['answer'] and cyreads==cex['reads']}
out['totals']=dict(totals)
out['remaining_scope']=['Repeatedly exposed symbolic BPE-token grammar; no untouched final language qualification.', 'Three-hop composition absent from fit, but exposed during eight original design attempts.', 'Nine two-token answers do not establish multiword prose; familiar disjoint marker codebook remains.', 'Categorical observation baseline; no geometric predictive advantage or whole-path D0-b/energy claim.', 'Source-bound immutable runtime rejects changed documents rather than implementing mutable persistent-memory capture reuse.']
out['resources']={'reported_elapsed_s':r['elapsed_s'],'retained_report_bytes':sum(p.stat().st_size for p in ROOT.rglob('*') if p.is_file()),'model_runs_by_auditor':0,'builds_by_auditor':0}
out['all_material_checks_pass']=not any(out['seal'][k] for k in ['missing','unlisted','size_errors','blake3_errors']) and all(out['provenance']['source_matches_review'].values()) and out['provenance']['actual_executable_matches'] and not event_errors and not snapshot_errors and all(out['reported_counts_match'].values()) and out['reported_primary_resume_count_matches'] and all(v['all_flags_match'] and v['all_expected_targets_match'] for v in out['arm_recounts'].values()) and out['source_intervention']['answers_match'] and out['source_intervention']['selection_depth_match'] and out['source_intervention']['goal_invariant_verified'] and out['downstream_absence']['outcome_matches'] and out['cycle']['matches']
out['script_sha256']=sha(P(__file__).read_bytes());dest=P('/tmp/uor-pr1342-corrected-audit.json');dest.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({k:out[k] for k in ['all_material_checks_pass','seal','arm_recounts','snapshot_phase_counts','totals','source_intervention','downstream_absence','cycle','resources']},indent=2))
