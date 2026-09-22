#!/usr/bin/env python3
"""Read-only saved-data audit of authored ordinary forms; no model implementation, fitting, inference or builds."""
import collections,copy,hashlib,json,pathlib,subprocess
import blake3
P=pathlib.Path;REPO=P('/Users/casey.allard/uor-r4');BASE=REPO/'.uor-models/realtext-prior-2026-09-20';HEAD='5d0a770c642855c54afab8fe7319f28ee4755eba';ROOT=BASE/'ordinary-form-argument-binding-8'
load=lambda p:json.loads(p.read_text());sha=lambda b:hashlib.sha256(b).hexdigest();compact=lambda v:json.dumps(v,separators=(',',':'),ensure_ascii=False).encode()
out={'schema':'uor-r4.ordinary-form-independent-audit/1','scope':__doc__,'submitted_head':HEAD,'roots':{}}
for i in range(1,9):
 root=BASE/f'ordinary-form-argument-binding-{i}';fs=[p for p in root.rglob('*')if p.is_file()];m=load(root/'manifest.json');r0=load(root/'result.json');listed={f['path']for f in m['files']};actual={str(p.relative_to(root))for p in fs}-{'manifest.json'}
 out['roots'][str(i)]={'root':str(root),'claimed_at':load(root/'attempt.json')['claimed_at'],'sealed_at':m['sealed_at'],'files':len(listed),'bytes':sum(p.stat().st_size for p in fs),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'size_errors':[f['path']for f in m['files']if(root/f['path']).stat().st_size!=f['bytes']],'blake3_errors':[f['path']for f in m['files']if blake3.blake3((root/f['path']).read_bytes()).hexdigest()!=f['blake3']],'source':r0['running_source'],'arms':r0['arms'],'elapsed_s':r0['elapsed_s'],'rows_sha256':sha((root/'rows.jsonl').read_bytes()),'worlds_sha256':sha((root/'worlds.json').read_bytes()),'models_sha256':{p.name:sha(p.read_bytes())for p in(root/'artifacts').glob('*.json')}}
r=load(ROOT/'result.json');wd=load(ROOT/'worlds.json');rows=[json.loads(x)for x in(ROOT/'rows.jsonl').read_text().splitlines()];EOS=r['task']['eos'];VOCAB=4096;persons=wd['membership_entities'];names=['Mara','Ivo','Cedar','Oren']
models={arm:(ROOT/f'artifacts/ordinary_forms_{arm}.json').read_bytes()for arm in['categorical_order','h4_hybrid']};modelb=models[r['primary_arm']]
worlds={w['id']:w for split in['development','exposed_regression','exposed_regression_second_draw','final']for w in wd[split]}
vocab=load(REPO/'.uor-models/sources/smollm2-135m-instruct/tokenizer.json')['model']['vocab'];rev={v:k for k,v in vocab.items()if v<VOCAB}
# All used authored fixture pieces are ASCII plus the GPT2 alphabet encoding of exterior space.
def decode(ts):return ''.join(rev[t]for t in ts if t!=EOS).replace('Ġ',' ')
def aligned(c):return bool(c['byte_lengths'])and len(c['tokens'])==len(c['byte_lengths'])and sum(c['byte_lengths'])==len(c['text'].encode())
def lexical(c,s,n):
 if aligned(c):return[1]+list(c['text'].encode()[sum(c['byte_lengths'][:s]):sum(c['byte_lengths'][:s+n])].strip(b' \t\n\r\v\f'))
 return[0]+list(b''.join(t.to_bytes(4,'little')for t in c['tokens'][s:s+n]))
def clausebytes(cs):return compact([{'seg':c['seg'],'tokens':c['tokens'],'text':c['text'],'byte_lengths':c['byte_lengths']}for c in cs])
def control(arm):return{'membership_continuation':{'Membership':{'entities':persons}},'maximum_two_read':{'ReadCap':{'max_reads':2}},'reads_disabled':'ReadsDisabled'}.get(arm,'Normal')
def expected(w,person,goal,arm='categorical_order'):
 cs={c['seg']:c for c in w['clauses']};visited=[];q=[1]+list(names[person].encode())
 def result(ans,term):return{'answer':ans,'reads':len(visited),'selected_segments':[v[0]for v in visited],'terminal':term}
 if arm=='reads_disabled':return result([],'Unresolved')
 for _ in range(6):
  if arm=='maximum_two_read'and len(visited)>=2:return result([],'Exhausted')
  matches=[]
  for l in w['labels_for_evaluation_only']:
   c=cs[l['seg']];s,n=l['subject']
   if l['goal']==goal and lexical(c,s,n)==q:matches.append((c,l))
  if not matches:return result([],'Unresolved')
  assert len(matches)==1
  c,l=matches[0];s,n=l['object'];span=[c['seg'],s,n]
  if span in visited:return result([],'Exhausted')
  visited.append(span);tokens=c['tokens'][s:s+n];q=lexical(c,s,n)
  if arm=='membership_continuation':emit=tokens not in persons
  elif arm=='analytical_lexical_membership':emit=q not in[[1]+list(x.encode())for x in names]
  else:emit=l['action']=='Emit'
  if emit:return result(tokens,'Stop')
 return result([],'Exhausted')
errors=[];snaperrors=[];counts=collections.Counter();phases=collections.Counter();keytypes=collections.Counter();arms=collections.defaultdict(list)
def framecheck(f,w,arm):
 er=[];doc=sha(clausebytes(w['clauses']));b=f['binding']
 if f['version']!=3:er.append('session version')
 wants={'model_sha256':sha(modelb),'tokenizer_sha256':r['inputs']['tokenizer_derived'],'world_id':w['id'],'world_version':w['version'],'doc_sha256':doc,'control_sha256':sha(compact(control(arm))),'max_vocab':VOCAB,'eos':EOS}
 for k,v in wants.items():
  if b[k]!=v:er.append('binding '+k)
 if f['goal']!=f['query']['goal']:er.append('query goal')
 if f['reads']!=len(f['visited'])or len({tuple(v)for v in f['visited']})!=len(f['visited']):er.append('visited history')
 cap=f['captured']
 if cap:
  c=next((c for c in w['clauses']if c['seg']==cap['segment']),None)
  if c is None or c['tokens'][cap['start']:cap['start']+cap['len']]!=f['captured_tokens']:er.append('source payload')
  if cap['doc_sha256']!=doc or cap['world_id']!=w['id']or cap['world_version']!=w['version']:er.append('capture identity')
  if f['captured_span']!=[cap['segment'],cap['start'],cap['len']]or f['visited'][-1]!=f['captured_span']:er.append('capture extent')
  if c and cap['key']!=lexical(c,cap['start'],cap['len']):er.append('capture key')
 elif f['reads']or f['captured_tokens']or f['captured_span']is not None:er.append('missing capture')
 want=f['captured_tokens'][:f['cursor']]+([EOS]if f['terminal']=='Stop'else[])
 if f['emitted']!=want:er.append('emitted prefix/EOS')
 if f['terminal']=='Stop'and(not f['captured_tokens']or f['cursor']!=len(f['captured_tokens'])):er.append('incomplete Stop')
 if f['terminal']in['Unresolved','Exhausted']and f['emitted']:er.append('failed emission')
 return er

def outcomecheck(row,w,arm,tag,check_snapshots=True):
 es=row['events'];emitted=[];selected=[];roles=[];attempts=[];er=[]
 for i,e in enumerate(es):
  b,a,eff=e['before'],e['after'],e['effect'];er.extend([i,'before',x]for x in framecheck(b,w,arm));er.extend([i,'after',x]for x in framecheck(a,w,arm))
  if i and b!=es[i-1]['after']:er.append([i,'continuity'])
  if b['goal']!=a['goal']:er.append([i,'goal mutation'])
  if a['reads']>b['reads']:
   selected.append(a['captured']['segment']);roles.append(eff['selected_role']);c=next(c for c in w['clauses']if c['seg']==a['captured']['segment']);keytypes['lexical_capture'if aligned(c)else'token_fallback_capture']+=1
   if eff['action']!='Read'or a['reads']!=b['reads']+1 or eff['selected_segment']!=a['captured']['segment']:er.append([i,'read effect'])
  elif eff['action']=='Read'and eff['selected_segment']is not None:attempts.append(eff['selected_segment'])
  if eff['emitted']is not None:emitted.append(eff['emitted'])
  if eff['terminal']!=a['terminal']:er.append([i,'terminal effect'])
  if eff['next_action']!=(a['pending']if a['terminal']is None else None):er.append([i,'next action'])
 if emitted!=row['emitted']or len(selected)!=row['reads']or es[-1]['after']!=row['final_frame']:er.append(['aggregate output'])
 if selected!=row['selected_segments']or roles!=row['selected_roles']:er.append(['selected refs'])
 if er:errors.append({'tag':tag,'errors':er})
 seen=[]
 for s in row['snapshots']:
  i=s['checkpoint_index'];seen.append(i);f=json.loads(s['snapshot_utf8']);se=[];phases['terminal:'+f['terminal']if f['terminal']else f['pending']]+=1
  if sha(s['snapshot_utf8'].encode())!=s['snapshot_sha256']:se.append('SHA')
  if f!=(es[i]['before']if i<len(es)else row['final_frame']):se.append('boundary')
  if s['resumed_suffix']!=es[i:]or s['resumed_final_frame']!=row['final_frame']or not s['identical']:se.append('continuation')
  se.extend(framecheck(f,w,arm))
  if se:snaperrors.append({'tag':tag,'index':i,'errors':se})
 if check_snapshots and (seen!=list(range(len(es)+1))or row['resume_identical']!=len(seen)):snaperrors.append({'tag':tag,'errors':['coverage']})
 counts.update({'outcomes':1,'events':len(es),'snapshots':len(seen),'successful_reads':len(selected),'rejected_read_attempts':len(attempts)})


failures=[];clauseerrors=[];formats=collections.Counter();splitstats={}
for w in worlds.values():
 for c in w['clauses']:
  if decode(c['tokens'])!=c['text']or c['byte_lengths']!=[len(decode([t]).encode())for t in c['tokens']]:clauseerrors.append([w['id'],c['seg']])
for row in rows:
 arm=row['arm'];w=worlds[row['world']];modelb=models[arm if arm in models else r['primary_arm']];tag=[row['split'],arm,row['world'],row['person'],row['goal']];ex=expected(w,row['person'],row['goal']);cx=expected(w,row['person'],row['goal'],arm);ok=row['emitted']==ex['answer']+[EOS]and row['terminal']=='Stop';want=cx['answer']+([EOS]if cx['terminal']=='Stop'else[])
 arms[row['split']+'/'+arm].append({'correct':ok,'depth':row['reads']==ex['reads'],'flag':ok==row['correct'],'oracle':row['independent_expected']==ex and row['expected_answer']==ex['answer']and row['expected_depth']==ex['reads'],'decoded':row['emitted_text_without_eos']==decode(row['emitted']),'control':row['emitted']==want and row['terminal']==cx['terminal']and row['reads']==cx['reads']and row['selected_segments']==cx['selected_segments']})
 outcomecheck(row,w,arm,tag)
 if arm in models and not ok:failures.append(tag)
 if row['events'][0]['before']['query']['key']!=[1]+list(names[row['person']].encode()):errors.append({'tag':tag,'errors':['query identity']})
 if row['question']['text']!=decode(row['question']['tokens'])or row['question_text']!=row['question']['text']:clauseerrors.append(['question',tag])
out['arms']={k:{'rows':len(xs),'complete':sum(x['correct']for x in xs),'depth_correct':sum(x['depth']for x in xs),'flags_oracles_text_controls_match':all(x['flag']and x['oracle']and x['decoded']and x['control']for x in xs)}for k,xs in arms.items()};out['primary_failures']=failures;out['rows_counts']=dict(counts);out['rows_phase_counts']=dict(phases);out['row_capture_key_modes']=dict(keytypes)
# Empirical surface support and overlap, not a claim that labels were inference inputs.
dev_texts={c['text']for w in wd['development']for c in w['clauses']};earlier_texts={c['text']for split in['development','exposed_regression','exposed_regression_second_draw']for w in wd[split]for c in w['clauses']};fitquestions=[c for c in wd['fit_clauses']if c['seg']>=900000]
for split in['development','exposed_regression','exposed_regression_second_draw','final']:
 ws=wd[split];cs=[c for w in ws for c in w['clauses']];ls=[l for w in ws for l in w['labels_for_evaluation_only']];reordered=sum(l['object'][0]<l['subject'][0]for l in ls);adjunct=sum(c['tokens'][l['object'][0]+l['object'][1]:]!=[] and l['object'][0]>l['subject'][0]for w in ws for c,l in zip(w['clauses'],w['labels_for_evaluation_only']))
 splitstats[split]={'worlds':len(ws),'statement_occurrences':len(cs),'unique_statements':len({c['text']for c in cs}),'object_before_subject':reordered,'post_object_trailing_material':adjunct,'statements_seen_in_development':sum(c['text']in dev_texts for c in cs),'statements_seen_in_prior_panels':sum(c['text']in earlier_texts for c in cs)}
finalquestions={x['question_text']for x in rows if x['split']=='final'}
out['split_scope']={'panels':splitstats,'unique_fit_questions':len({c['text']for c in fitquestions}),'fit_question_occurrences':len(fitquestions),'final_questions_seen_in_fit':len(finalquestions&{c['text']for c in fitquestions}),'unique_final_questions':len(finalquestions),'construction':'Authored deterministic chain/style/target cycling, not randomized population sampling. Three-read compositions absent from development worlds; vocabulary and form families shared.','draw7_vs8':{'same_worlds_bytes':out['roots']['7']['worlds_sha256']==out['roots']['8']['worlds_sha256'],'same_rows_bytes':out['roots']['7']['rows_sha256']==out['roots']['8']['rows_sha256'],'same_models':out['roots']['7']['models_sha256']==out['roots']['8']['models_sha256']},'root6_vs7_models_identical':out['roots']['6']['models_sha256']==out['roots']['7']['models_sha256'],'claim':'Final composition panel first appears in root7 after root6 fitted models; root8 is a provenance replay of the same final draw, not another fresh qualification. No preregistered independent random sampling is claimed by this audit.'}
# Both fitted models must be bound to their own rows; geometry arm adds information/parameters.
cat=json.loads(models['categorical_order']);h4=json.loads(models['h4_hybrid']);kinds=collections.Counter()
for key,vals in h4['segment_weights']:
 if any(vals):kinds[str(key>>40)]+=1
out['model_comparison']={'reported_learning':r['learning'],'models':{a:{'sha256':sha(b),'bytes':len(b),'version':json.loads(b)['version'],'role_feature_rows':len(json.loads(b)['feature_weights']),'segment_feature_rows':len(json.loads(b)['segment_weights']),'token_code_rows':len(json.loads(b)['token_elements'])}for a,b in models.items()},'h4_nonzero_segment_feature_kinds':dict(kinds),'all_h4_codes_still_token_mod120':all(e==t%120 for t,e in h4['token_elements']),'scope':'H4 hybrid adds ordered finite-group potentials to order-aware categorical features; equal authored final answers, not equality of capacity or proof of no geometric advantage. Code-search accepts strictly higher exact-fit count after already96/96, so 0 accepted moves is ceiling-imposed.'}
# Matched lexical-membership semantics are an analytic counterfactual, not another executed model.
memrows=[x for x in rows if x['arm']=='membership_continuation'];lcorrect=0;different=[]
for row in memrows:
 ex=expected(worlds[row['world']],row['person'],row['goal'],'analytical_lexical_membership');want=ex['answer']+([EOS]if ex['terminal']=='Stop'else[]);lcorrect+=want==row['expected_answer']+[EOS]and ex['terminal']=='Stop'
 if want!=row['emitted']or ex['reads']!=row['reads']:different.append({'world':row['world'],'person':row['person'],'goal':row['goal'],'token_registry_result':{'emitted':row['emitted'],'reads':row['reads']},'lexical_membership_gold_trace':ex})
out['membership_boundary']={'actual_token_registry_complete':38,'analytical_lexical_registry_correct':lcorrect,'total':len(memrows),'different_rows':different,'scope':'Analytic traversal of saved typed facts, not model execution; actual token-membership comparator is boundary-mismatched and is not a matched lexical role comparator.'}
modelb=models[r['primary_arm']];ints=r['interventions'];base=ints['changed_terminal_source'];w0=worlds[100];details={};oracleerrors=[];editdetails=[]
checks=[('base',base['before'],base['before_clauses'],base['before_expected'],'Office',None),('changed_terminal_source',base['after'],base['after_clauses'],base['after_expected'],'Office',base['segment']),('cycle',ints['cycle']['outcome'],ints['cycle']['clauses'],ints['cycle']['expected'],'Office',ints['cycle']['edited_segment']),('valid_redirect_to_existing_person',ints['valid_redirect_to_existing_person']['outcome'],ints['valid_redirect_to_existing_person']['clauses'],ints['valid_redirect_to_existing_person']['expected'],'Office',ints['valid_redirect_to_existing_person']['edited_segment']),('required_terminal_fact_removed',ints['required_terminal_fact_removed']['outcome'],ints['required_terminal_fact_removed']['clauses'],ints['required_terminal_fact_removed']['expected'],'Office',None),('request_goal_change',ints['request_goal_change']['outcome'],w0['clauses'],ints['request_goal_change']['expected'],'Project',None)]
for name,row,cs,saved_expected,goal,edited_seg in checks:
 byseg={c['seg']:c for c in cs};labels=copy.deepcopy(w0['labels_for_evaluation_only']);labels=[l for l in labels if l['seg']in byseg]
 if edited_seg is not None:
  l=next(l for l in labels if l['seg']==edited_seg);start,oldlen=l['object'];oldc=next(c for c in w0['clauses']if c['seg']==edited_seg);c=byseg[edited_seg];l['object']=[start,len(c['tokens'])-start];editdetails.append({'name':name,'segment':edited_seg,'before':oldc['text'],'after':c['text'],'old_object':decode(oldc['tokens'][start:start+oldlen]),'new_object':decode(c['tokens'][start:]),'dropped_suffix':decode(oldc['tokens'][start+oldlen:]),'single_object_only':not oldc['tokens'][start+oldlen:]})
 w={'id':100,'version':300,'clauses':cs,'labels_for_evaluation_only':labels};ex=expected(w,0,goal);want=ex['answer']+([EOS]if ex['terminal']=='Stop'else[]);coherent=all(aligned(c)and decode(c['tokens'])==c['text']and c['byte_lengths']==[len(decode([t]).encode())for t in c['tokens']]for c in cs);outcomecheck(row,w,r['primary_arm'],name);match=ex==saved_expected and row['emitted']==want and row['reads']==ex['reads']and row['terminal']==ex['terminal']and row['selected_segments']==ex['selected_segments']
 if not match or not coherent:oracleerrors.append(name)
 details[name]={'expected':ex,'actual':{k:row[k]for k in['emitted','reads','selected_segments','terminal']},'decoded_answer':decode(row['emitted']),'coherent':coherent,'matches':match}
pert=ints['subword_order_perturbation'];pert_counts_before=dict(counts)
if 'events'in pert['outcome']:outcomecheck(pert['outcome'],w0,r['primary_arm'],'subword_perturbation',False)
out['interventions']={'reported_checks':ints['checks'],'all_expected_reported':r['interventions_all_expected'],'details':details,'oracle_errors':oracleerrors,'edits':editdetails,'changed_payload_same_path_but_suffix_also_deleted':any(e['name']=='changed_terminal_source'and not e['single_object_only']for e in editdetails),'subword_perturbation':{'text':pert['decoded'],'emitted_text':decode(pert['outcome'].get('emitted',[])),'snapshots':len(pert['outcome'].get('snapshots',[])),'scope':pert['scope']},'scope':'Saved intervention outcomes valid for their actual inputs; terminal-source edit changes payload AND removes downtown. Does not isolate payload-only causal effect. ob_edit_object truncates suffix, unsafe for object-before-subject forms.'}
out['counts_before_uncheckpointed_perturbation']=pert_counts_before;out['all_saved_counts_including_perturbation']=dict(counts);out['frame_errors']=errors;out['snapshot_errors']=snaperrors;out['clause_text_errors']=clauseerrors
# Directly verify reported cross-boundary source and query spans using actual bytes.
identity=r['identity'];joins=[]
for j in identity['chain_joins']:
 pc=next(c for c in w0['clauses']if c['seg']==j['from_segment']);nc=next(c for c in w0['clauses']if c['seg']==j['to_segment']);pl=next(l for l in w0['labels_for_evaluation_only']if l['seg']==j['from_segment']);nl=next(l for l in w0['labels_for_evaluation_only']if l['seg']==j['to_segment']);ps,pn=pl['object'];ns,nn=nl['subject'];ok=j['object_surface']==pc['tokens'][ps:ps+pn]and j['subject_surface']==nc['tokens'][ns:ns+nn]and lexical(pc,ps,pn)==lexical(nc,ns,nn)and j['object_surface']!=j['subject_surface'];joins.append({'from':j['from_segment'],'to':j['to_segment'],'independent_match':ok})
qjoin=identity['query_join'];qrow=next(x for x in rows if x['arm']==r['primary_arm']and x['world']==100 and x['person']==0 and x['goal']=='Office');qf=qrow['events'][0]['before'];queryok=qf['query']['tokens']==qjoin['query_subject_surface']and qf['query']['key']==[1]+list(qjoin['query_key'].encode())and decode(qjoin['source_subject_surface']).strip()==qjoin['query_key']and qjoin['source_subject_surface']!=qjoin['query_subject_surface']
out['cross_bpe']={'query_verified':queryok,'chain_joins':joins,'actual_successful_base_answer':details['base']['decoded_answer'],'qualification':'Real different token sequences join through exact lexical identity in executed query and dependent chain.'}
exe=BASE/'ordinary-form-argument-binding-delivery/competitive-reader-ordinary-form-binding-8';out['provenance']={'source_matches_submitted':{f['path']:sha(subprocess.check_output(['git','show',HEAD+':'+f['path']],cwd=REPO))==f['sha256']for f in r['running_source']['source_files']},'preserved_executable':str(exe),'executable_matches':sha(exe.read_bytes())==r['running_source']['executable_sha256'],'source_receipt':r['running_source']}
out['reported_counts_match']={a['split']+'/'+a['arm']:out['arms'][a['split']+'/'+a['arm']]['rows']==a['total']and out['arms'][a['split']+'/'+a['arm']]['complete']==a['complete']and(a['depth_correct']is None or out['arms'][a['split']+'/'+a['arm']]['depth_correct']==a['depth_correct'])for a in r['arms']+r['controls']}
out['saved_evidence_consistency_pass']=not errors and not snaperrors and not clauseerrors and not failures and not oracleerrors and all(out['provenance']['source_matches_submitted'].values())and out['provenance']['executable_matches']and all(out['reported_counts_match'].values())and all(x['flags_oracles_text_controls_match']for x in out['arms'].values())and queryok and all(j['independent_match']for j in joins)and all(not v[k]for v in out['roots'].values()for k in['missing','unlisted','size_errors','blake3_errors'])
out['qualified_conclusion']='Bounded authored ordinary-form argument parsing and dependent lexical joins demonstrated; source-edit instrument not isolated, token-membership comparator mismatched, root8 repeats root7 final draw. No geometric predictive advantage or broad-language claim.'
P('/tmp/uor-ordinary-review-audit.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'consistent':out['saved_evidence_consistency_pass'],'arms':out['arms'],'counts':out['counts_before_uncheckpointed_perturbation'],'frame_errors':len(errors),'snapshot_errors':len(snaperrors),'split_scope':out['split_scope'],'membership':{'actual':38,'analytical_lexical':lcorrect},'out':'/tmp/uor-ordinary-review-audit.json'},indent=2))
