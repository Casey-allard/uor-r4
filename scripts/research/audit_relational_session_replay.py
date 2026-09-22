#!/usr/bin/env python3
"""Independent saved-report/event audit of corrected PR1341. No model execution or fitting."""
import collections,hashlib,json,pathlib,struct,sys
import blake3
P=pathlib.Path
ROOT=P(sys.argv[1] if len(sys.argv)>1 else '/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/relational-session-principal-2')
REPO=P('/Users/casey.allard/.codex/worktrees/takeover-research-reconciliation/uor-r4')
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(p.read_text())
r=load(ROOT/'result.json');manifest=load(ROOT/'manifest.json')
listed={f['path'] for f in manifest['files']};actual={str(p.relative_to(ROOT)) for p in ROOT.rglob('*') if p.is_file()}-{'manifest.json'}
out={'schema':'uor-r4.pr1341-corrected-independent-audit/1','scope':__doc__,'report_root':str(ROOT),'seal':{'listed_files':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),'size_errors':[f['path'] for f in manifest['files'] if (ROOT/f['path']).stat().st_size!=f['bytes']],'blake3_errors':[f['path'] for f in manifest['files'] if blake3.blake3((ROOT/f['path']).read_bytes()).hexdigest()!=f['blake3']]},'provenance':{'result_sha256':sha((ROOT/'result.json').read_bytes()),'source_matches_review_worktree':{f['path']:sha((REPO/f['path']).read_bytes())==f['sha256'] for f in r['running_source']['source_files']},'running_source':r['running_source']}}
exe=P(load(ROOT/'attempt.json')['argv'][0]);out['provenance']['executable_matches']=sha(exe.read_bytes())==r['running_source']['executable_sha256']
def parse_model(path):
    b=path.read_bytes();c=0
    def take(n):
        nonlocal c
        v=b[c:c+n];assert len(v)==n;c+=n;return v
    def u32():return struct.unpack('<I',take(4))[0]
    assert take(4)==b'RLRM';version=u32();cyclic=bool(take(1)[0]);categorical=bool(take(1)[0]);domains=[];codes=[]
    for _ in range(2):domains.append([u32() for _ in range(u32())])
    for _ in range(2):codes.append(list(take(u32())))
    follow=[(u32(),u32()) for _ in range(u32())];weights=[struct.unpack('<i',take(4))[0] for _ in range(4)];policy=list(take(2));cats=[list(take(8)) for _ in range(u32())];assert c==len(b)
    return {'version':version,'cyclic':cyclic,'categorical':categorical,'relation_domain':domains[0],'role_domain':domains[1],'relation_codes':codes[0],'role_codes':codes[1],'follow':follow,'weights':weights,'policy':policy,'categorical_table':cats,'sha256':sha(b),'bytes':len(b)}
out['artifacts']={p.stem:parse_model(p) for p in (ROOT/'artifacts').iterdir()}
model=out['artifacts']['relational_primary'];relations=model['relation_domain'];roles=model['role_domain'];follow=dict(model['follow'])
def oracle(world,relation,entity):
    ri=relations.index(relation);rs=world['records'];first=next((i for i,(role,key,v) in enumerate(rs) if role==roles[ri] and key==entity),None)
    if first is None:return {'answer':None,'terminal':'Unresolved','selected_abs':[]}
    value=rs[first][2]
    if value in world['entities'] and ri<2:
        second=next((i for i,(role,key,v) in enumerate(rs) if role==roles[ri+2] and key==value),None)
        if second is None:return {'answer':None,'terminal':'Unresolved','selected_abs':[first]}
        return {'answer':rs[second][2],'terminal':'Stop','selected_abs':[first,second]}
    return {'answer':value,'terminal':'Stop','selected_abs':[first]}
rows=r['all_arm_rows'];worlds=r['worlds'];counts={};event_errors=[]
out['rows_file_matches_embedded_primary']=[json.loads(s) for s in (ROOT/'rows.jsonl').read_text().splitlines()]==rows['exposed_primary']
def audit_events(xs,world,label):
    es=xs['events'];errors=[]
    for i,ev in enumerate(es):
        b=ev['before'];a=ev['after']
        if i and b!=es[i-1]['after']:errors.append([i,'broken frame continuity'])
        if a['hops']>b['hops']:
            cp=a['captured'];record=world['records'][cp['abs']]
            if a['hops']!=b['hops']+1 or b['pending']!='Read' or record[2]!=cp['payload'] or record[1]!=b['entity'] or cp['seq']!=world['version'] or a['retained']!=record[2]:errors.append([i,'capture mismatch'])
        if a['binding']!=b['binding']:errors.append([i,'binding changed'])
        if bytes(a['binding']['model']).hex() not in {m['sha256'] for m in out['artifacts'].values()}:errors.append([i,'unknown model binding'])
        if bytes(a['binding']['tokenizer']).hex()!=r['inputs']['tokenizer_derived']:errors.append([i,'tokenizer binding mismatch'])
        if a['binding']['world_namespace']!=world['version']:errors.append([i,'world namespace mismatch'])
        if a['captured'] is not None and a['retained']!=a['captured']['payload']:errors.append([i,'owned retained mismatch'])
    if es:
        last=es[-1]['after']
        if last['emitted']!=([xs['emitted']] if xs['emitted'] is not None else []):errors.append(['final','emitted trace mismatch'])
        if last['terminal']!=xs['terminal'] or last['hops']!=xs['hops']:errors.append(['final','terminal trace mismatch'])
        captures=[ev['after']['captured']['abs'] for ev in es if ev['after']['hops']>ev['before']['hops']]
        if captures!=xs['selected_abs']:errors.append(['final','selected indices mismatch'])
    return errors
for name,xs in rows.items():
    ws=worlds['development'] if name=='development_primary' else worlds['exposed_original_final']
    cs=[]
    for x in xs:
        w=ws[x['world']];ex=oracle(w,x['relation'],x['entity']);ok=x['emitted']==ex['answer'] and x['terminal']==ex['terminal']
        cs.append((ok,x['hops']==len(ex['selected_abs']),x['correct']==ok))
        errs=audit_events(x,w,name)
        if errs:event_errors.append({'arm':name,'request':[x['world'],x['relation'],x['entity']],'errors':errs})
    counts[name]={'rows':len(xs),'correct':sum(v[0] for v in cs),'correct_depth':sum(v[1] for v in cs),'reported_flags_match':all(v[2] for v in cs),'event_count':sum(len(x['events']) for x in xs),'successful_captures':sum(len(x['selected_abs']) for x in xs),'read_action_count':sum(x['actions'].count('Read') for x in xs)}
out['all_arm_recount']=counts;out['event_errors']=event_errors
reported_map={'development_primary':('learned_relational','development'),'exposed_primary':('learned_relational','final'),'categorical':('categorical_control','final'),'cyclic':('cyclic_control','final'),'relation_disabled':('relation_rank_and_gate_disabled','final'),'always_continue':('continuation_frozen_continue','final'),'read_disabled':('reads_disabled','final')}
out['reported_arm_count_matches']={name:any(a['arm']==arm and a['split']==split and a['complete']==counts[name]['correct'] and a['total']==counts[name]['rows'] for a in r['arms']) for name,(arm,split) in reported_map.items()}
original=[json.loads(s) for s in (ROOT.parent/'relational-session-4/rows.jsonl').read_text().splitlines()]
out['original_primary_preservation']={'original_rows':len(original),'matching_request_output_selection_depth':sum(all(a[k]==b[k] for k in ['world','relation','entity','emitted','hops','selected_abs','retained','terminal']) for a,b in zip(original,rows['exposed_primary']))}
devtypes=[]
for w in worlds['development']:
    for relation in relations:
        for entity in w['entities']:
            ex=oracle(w,relation,entity);first=w['records'][ex['selected_abs'][0]][2];devtypes.append((first in w['entities'],len(ex['selected_abs'])>=2))
out['learning_recount']={'examples':len(devtypes),'observed_entity_first_values':sum(f for f,y in devtypes),'initial_all_false_policy_correct':sum(not y for f,y in devtypes),'fitted_policy_correct':sum(bool(model['policy'][int(f)])==y for f,y in devtypes),'artifact_policy':model['policy'],'fixed_score_coefficients':model['weights'],'relation_role_codes_equal':model['relation_codes']==model['role_codes'],'reports':r['learning'],'note':'Feature derived from supplied entity registry, target from separately declared oracle action label. Original fixture still correlates type/depth with request relation.'}
ints=r['interventions'];snapshots=[]
primary_index={(worlds['exposed_original_final'][x['world']]['version'],x['relation'],x['entity']):x for x in rows['exposed_primary']}
for item in ints['saved_snapshots']:
    b=(ROOT/item['path']).read_bytes();assert b[:8]==b'RLRF\x02\0\0\0';frame=json.loads(b[8:]);row=primary_index[item['world_namespace'],item['relation'],item['entity']]
    snapshots.append({'path':item['path'],'sha_size_match':sha(b)==item['sha256'] and len(b)==item['bytes'],'matches_after_first_read':frame==row['events'][0]['after'],'matches_next_transition_before':frame==row['events'][1]['before'],'phase_is_continue':frame['pending']=='Continue','hops':frame['hops'],'model_sha_matches_primary':bytes(frame['binding']['model']).hex()==model['sha256'],'bytes':len(b)})
out['snapshot_audit']={'count':len(snapshots),'sha_size_match':sum(x['sha_size_match'] for x in snapshots),'matches_after_first_read':sum(x['matches_after_first_read'] for x in snapshots),'matches_next_transition_before':sum(x['matches_next_transition_before'] for x in snapshots),'continue_after_one_read':sum(x['phase_is_continue'] and x['hops']==1 for x in snapshots),'model_bindings_correct':sum(x['model_sha_matches_primary'] for x in snapshots),'min_bytes':min(x['bytes'] for x in snapshots),'max_bytes':max(x['bytes'] for x in snapshots),'total_bytes':sum(x['bytes'] for x in snapshots),'resume_identical_reported':ints['pause_resume_identical'],'scope':'Saved checkpoints equal the exact frame at the continuation boundary and next recorded transition. Source executes independent rel_finish after dropping live frame; no extra model runs by auditor.'}
interleave=[]
for i,(rel,ent) in enumerate([(relations[0],r['task']['entities'][0]),(relations[2],r['task']['entities'][1])]):
    isolated=primary_index[worlds['exposed_original_final'][0]['version'],rel,ent]
    interleave.append(ints['interleaved_events'][i]==isolated['events'])
out['interleave_audit']={'recorded_interleaved_traces_match_isolated':interleave,'reported_independent':ints['interleaved_sessions_independent']}
ci=ints['one_position_source_edit'];wb=ci['world_before'];wa=ci['world_after'];eb=oracle(wb,ci['relation'],ci['entity']);ea=oracle(wa,ci['relation'],ci['entity'])
out['source_intervention']={'changed_record_indices':[i for i,(a,b) in enumerate(zip(wb['records'],wa['records'])) if a!=b],'world_other_fields_same':all(wb[k]==wa[k] for k in wb if k!='records'),'before_expected':eb,'after_expected':ea,'saved_occurrences_match':eb['selected_abs']==ci['selected_abs_before'] and ea['selected_abs']==ci['selected_abs_after'],'answers_match':eb['answer']==ci['emitted_before'] and ea['answer']==ci['emitted_after'],'follow_relation_unchanged':follow[ci['relation']],'query_entities':[ci['retained_before'][0],ci['retained_after'][0]]}
diag=[]
for case in ints['same_request_content_diagnostic']:
    ex=oracle(case['world'],case['request']['relation'],case['request']['entity']);cs=[]
    for x in case['rows']:
        ok=x['emitted']==ex['answer'] and x['terminal']==ex['terminal'];cs.append({'arm':x['arm'],'correct':ok,'flag_matches':x['correct']==ok,'emitted':x['emitted'],'hops':x['hops'],'terminal':x['terminal'],'event_errors':audit_events(x,case['world'],'diagnostic')})
    diag.append({'case':case['case'],'request':case['request'],'independent_expected':ex,'rows':cs})
out['same_request_diagnostic']={'cases':diag,'same_request_in_all_cases':len({json.dumps(x['request'],sort_keys=True) for x in diag})==1,'by_arm':{arm:sum(x['correct'] for c in diag for x in c['rows'] if x['arm']==arm) for arm in ['learned_type_policy','relation_only_fixed_depth']},'scope':'Three authored diagnostic cases after design: normal two-read entity, one-read literal, unresolved orphan entity. Not a fresh population or statistical advantage claim.'}
out['remaining_boundaries']=['Relation-role operation is equality encoding; group/categorical/cyclic equivalence on this task remains.', 'Entity type registry, record layout and follow convention supplied; no natural-language parsing/prose.', 'Session runtime remains in competitive-reader runner; exact origin comparison is a descriptor diagnostic, not independently verified persistent ring service.', 'Primary only explicit pre/post full-predictor parity256; categorical/cyclic actual loaded serving audited through retained rows.', 'Serializing bound snapshots does not prove bounded serving allocations or optimized energy/cost.', 'Repeatedly exposed original panels and three new authored cases are development diagnostics.']
out['resources']={'reported_elapsed_s':r['elapsed_s'],'retained_report_bytes':sum(p.stat().st_size for p in ROOT.rglob('*') if p.is_file()),'model_runs_by_auditor':0,'builds_by_auditor':0}
out['all_material_checks_pass']=not(out['seal']['missing'] or out['seal']['unlisted'] or out['seal']['size_errors'] or out['seal']['blake3_errors'] or event_errors) and all(out['provenance']['source_matches_review_worktree'].values()) and out['provenance']['executable_matches'] and out['rows_file_matches_embedded_primary'] and all(v['reported_flags_match'] for v in counts.values()) and all(out['reported_arm_count_matches'].values()) and all(all(x[k] for k in ['sha_size_match','matches_after_first_read','matches_next_transition_before','model_sha_matches_primary']) for x in snapshots) and all(interleave) and all(x['flag_matches'] and not x['event_errors'] for c in diag for x in c['rows'])
out['script_sha256']=sha(P(__file__).read_bytes());dest=P('/tmp/uor-pr1341-corrected-audit.json');dest.write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'output':str(dest),'all_material_checks_pass':out['all_material_checks_pass'],'counts':counts,'snapshots':out['snapshot_audit'],'diagnostic':out['same_request_diagnostic'],'resources':out['resources']},indent=2))
