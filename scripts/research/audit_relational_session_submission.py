#!/usr/bin/env python3
"""Read-only PR1341 saved-byte/event audit. No fitting, model execution or sealed-root writes."""
import collections, hashlib, json, pathlib, struct, subprocess
try:
    import blake3
except ImportError:
    blake3 = None
P = pathlib.Path
BASE=P('/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20')
REPO=P('/Users/casey.allard/uor-r4/.worktrees/learned-relation-control')
HEAD='84ba72cea9f1283c56e36471b59f348a6f8c06ba'
sha=lambda b: hashlib.sha256(b).hexdigest()
load=lambda p: json.loads(p.read_text())
out={'schema':'uor-r4.pr1341-independent-evidence-audit/1','scope':__doc__,'submitted_head':HEAD,'roots':{}}
for n in range(1,5):
    root=BASE/f'relational-session-{n}';r=load(root/'result.json');m=load(root/'manifest.json')
    actual={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}-{'manifest.json'}
    listed={f['path'] for f in m['files']}
    xs=[json.loads(line) for line in (root/'rows.jsonl').read_text().splitlines()]
    out['roots'][root.name]={
        'claimed_at':load(root/'attempt.json')['claimed_at'],'sealed_at':m['sealed_at'],
        'listed':len(listed),'missing':sorted(listed-actual),'unlisted':sorted(actual-listed),
        'byte_size_errors':[f['path'] for f in m['files'] if (root/f['path']).stat().st_size != f['bytes']],
        'blake3_errors':[f['path'] for f in m['files'] if blake3.blake3((root/f['path']).read_bytes()).hexdigest()!=f['blake3']] if blake3 else 'UNAVAILABLE_MODULE',
        'rows_sha256':sha((root/'rows.jsonl').read_bytes()),'result_sha256':sha((root/'result.json').read_bytes()),
        'retained_bytes':sum(p.stat().st_size for p in root.rglob('*') if p.is_file()),
        'reported_elapsed_s':r['elapsed_s'],'running_source':r['running_source'],
        'source_matches_submitted':{f['path']:sha(subprocess.check_output(['git','show',HEAD+':'+f['path']],cwd=REPO))==f['sha256'] for f in r['running_source']['source_files']},
        'artifact_hashes':{p.name:sha(p.read_bytes()) for p in (root/'artifacts').iterdir()},
        'primary_retained_rows':len(xs),'primary_correct_recount':sum(x['emitted']==x['expected_answer'] and x['terminal']=='Stop' for x in xs),
        'reported_arms':r['arms'],'interventions':r.get('interventions'),
    }
r=load(BASE/'relational-session-4/result.json')
xs=[json.loads(line) for line in (BASE/'relational-session-4/rows.jsonl').read_text().splitlines()]
exe=P('/tmp/uor-pr1341-original-competitive-reader')
out['provenance']={'preserved_executable':str(exe),'executable_sha256':sha(exe.read_bytes()),'matches_run4':sha(exe.read_bytes())==r['running_source']['executable_sha256'],'all_run4_source_hashes_match_submitted':all(out['roots']['relational-session-4']['source_matches_submitted'].values()),'source_note':'Git stamp is reviewed parent; source-file hashes identify actual new implementation.'}
out['exposure']={'all_four_retained_primary_row_sets_byte_identical':len({v['rows_sha256'] for v in out['roots'].values()})==1,'all_four_primary_artifacts_identical':len({v['artifact_hashes']['relational_primary.rlrm'] for v in out['roots'].values()})==1,'all_four_have_same_final_seed':'Source hard-codes REL_FINAL_SEED=0xC0F50101. All four rows identical. Compatibility-gate and intervention repairs followed final-panel exposure.','qualification':'Final split is development-disjoint in world assignment but repeatedly exposed during instrumentation/design; not an untouched final qualification.'}
def artifact(path):
    b=path.read_bytes();c=0
    def take(n):
        nonlocal c
        z=b[c:c+n];assert len(z)==n;c+=n;return z
    def u32():return struct.unpack('<I',take(4))[0]
    assert take(4)==b'RLRM';ver=u32();cyclic=bool(take(1)[0]);categorical=bool(take(1)[0]);domains=[];codes=[]
    for _ in range(2):domains.append([u32() for _ in range(u32())])
    for _ in range(2):codes.append(list(take(u32())))
    follow=[(u32(),u32()) for _ in range(u32())]
    weights=[struct.unpack('<i',take(4))[0] for _ in range(4)]
    policy=list(take(2));n=u32();cats=[list(take(8)) for _ in range(n)]
    assert c==len(b)
    return {'version':ver,'cyclic':cyclic,'categorical':categorical,'relation_domain':domains[0],'role_domain':domains[1],'relation_code':codes[0],'role_code':codes[1],'follow':follow,'weights':weights,'continue_policy':policy,'categorical_table':cats,'bytes':len(b),'sha256':sha(b)}
out['artifacts']={p.stem:artifact(p) for p in (BASE/'relational-session-4/artifacts').iterdir()}
M=out['artifacts']['relational_primary'];relations=M['relation_domain'];roles=M['role_domain'];entities=r['task']['entities'];literals=r['task']['literals']
# Reconstruct only the authored source fixture and independent target lookup, not the model.
MASK=(1<<64)-1
def fixture(seed,version):
    st=seed|1;rows=[]
    def rand():
        nonlocal st
        st^=(st<<13)&MASK;st^=st>>7;st^=(st<<17)&MASK;return st
    for e in entities:
        for ri,role in enumerate(roles):
            if ri<2:
                v=entities[rand()%len(entities)]
                if v==e:v=entities[(entities.index(v)+1)%len(entities)]
            else:v=literals[rand()%len(literals)]
            rows.append((role,e,v))
    return rows
worlds=[fixture(0xC0F50101+i*0x9E37,100+i) for i in range(4)]
def expected(world,relation,entity):
    ri=relations.index(relation);role=roles[ri]
    first=next((i for i,(r,k,v) in enumerate(world) if r==role and k==entity),None)
    if first is None:return None
    value=world[first][2]
    if value in entities and ri<2:
        second=next((i for i,(r,k,v) in enumerate(world) if r==roles[ri+2] and k==value),None)
        if second is None:return None
        return {'answer':world[second][2],'depth':2,'selected_abs':[first,second],'retained':[value,world[second][2]],'selected_roles':[role,roles[ri+2]]}
    return {'answer':value,'depth':1,'selected_abs':[first],'retained':[value],'selected_roles':[role]}
checks=[]
for x in xs:
    e=expected(worlds[x['world']],x['relation'],x['entity'])
    checks.append({'expected_matches':e['answer']==x['expected_answer'] and e['depth']==x['expected_depth'],'emitted_correct':e['answer']==x['emitted'] and x['terminal']=='Stop','exact_occurrences_match':e['selected_abs']==x['selected_abs'],'retained_matches':e['retained']==x['retained'],'roles_match':e['selected_roles']==x['selected_roles'],'flags_match':x['correct']==(e['answer']==x['emitted'] and x['terminal']=='Stop')})
out['primary_recount']={'rows':len(xs),'independent_world_draws':len(worlds),'by_world':dict(collections.Counter(x['world'] for x in xs)),'by_relation':{str(rel):{'requests':sum(x['relation']==rel for x in xs),'depths':sorted({x['hops'] for x in xs if x['relation']==rel})} for rel in relations},'depth_counts':dict(collections.Counter(x['hops'] for x in xs)),'decoded_output_counts':dict(collections.Counter(x['decoded'] for x in xs)),'token_counts':dict(collections.Counter(x['emitted'] for x in xs)),'checks_true_counts':{k:sum(x[k] for x in checks) for k in checks[0]},'all_reported_primary_correct_flags_match':all(x['flags_match'] for x in checks),'action_event_count':sum(len(x['actions']) for x in xs),'selection_count':sum(len(x['selected_abs']) for x in xs),'read_action_event_count':sum(x['actions'].count('Read') for x in xs),'note':'Actions omit the second Read event; only primary final rows retained, no development/comparator/lesion row files.'}
w=worlds[0];before=expected(w,relations[0],entities[0]);edited=list(w);new=entities[1] if w[0][2]!=entities[1] else entities[2];edited[0]=(w[0][0],w[0][1],new);after=expected(edited,relations[0],entities[0])
out['intervention_reconstruction']={'first_source_edit_changed_record_indices':[i for i,(a,b) in enumerate(zip(w,edited)) if a!=b],'before':before,'after':after,'answer_changes':before['answer']!=after['answer'],'second_occurrence_changes':before['selected_abs'][1]!=after['selected_abs'][1],'follow_relation_before_after':[M['follow'][0][1],M['follow'][0][1]],'query_entity_before_after':[before['retained'][0],after['retained'][0]],'note':'Source edit changes second-query entity, not follow-up relation. Downstream records fixed. Saved intervention lacks selected occurrence traces; reconstructed from pinned fixture source.','missing_source':'Original incomplete-world intervention removes first required fact only; not a missing second entity record or same-value/version overwrite test.'}
out['claim_audit']={
 'learned_components':['relation/role code assignment or categorical compatibility table','relation-indexed follow map from development intermediate supervision'],
 'scorer_weights':'Fixed [1,1,0,0]; no weight fit. Across admitted candidates exact_key constant, content_is_key passed as a single per-query value; relation match is the only discriminant.',
 'continuation_policy':'Initialized to already correct [false,true]. Training derives both feature and target from e.depth>=2 instead of observing world key membership. The policy does not change. policy_initial=1 counts true entries, policy_final=128 counts examples: incomparable quantities.',
 'depth_confound':'Every relation0/1 is always two reads, every relation2/3 always one. A relation-only fixed-depth policy perfectly predicts all observed depths, so same-request content-dependent scheduling advantage remains untested.',
 'geometric_identity':'H4 code tests relative element0, but canonical identity is1; code0 is central -1. Learned antipodal codes compensate. Cyclic identity0 is correct. Classification scores hold, claimed identity interpretation needs repair.',
 'geometric_scope':'Testing whether a^-1*z equals one fixed group element is a relabeled equality relation. All three arms tie by construction-level expressive equivalence on this fixture; no noncommutative/order geometry used.',
 'no_relation_lesion':'Always selects candidate0 while retaining learned compatibility gate. Requests that could survive first selection terminate on second incompatible relation. The 0/128 is whole ranking-plus-gate intervention, not evidence that group structure is necessary.',
 'resume':'rel_serve always constructs fresh RelFrame. resume_probe serializes/deserializes after capture but retains host-loop program counter, chosen.value, relation and trace vectors; subsequent control uses chosen.value rather than restored payload. No snapshot bytes or resumed trajectories retained, no external continuation entry exercised.',
 'interleave':'Two frame structs receive manual field mutations; no interleaved predictor or alternate model/world resume. Checks only retained/emitted fields.',
 'ownership':'CapturedPayload fields public; live-origin diagnostic compares scalar payload only, ignores seq/abs/version. Same-value overwrite would be falsely live. No model/geometry/tokenizer/session identity binding in model or frame.',
 'loaded_serving':'All three actual loaded objects used for evaluated serving. Full-predictor pre/post parity256 requests primary only; categorical/cyclic parameter roundtrip equality plus loaded serving, no explicit pre/post full-predictor parity.',
 'output_scope':'One literal token among four, decoded strings ame/able/ong/ear. These are complete answers only for supplied typed-world task; no natural-language parsing/prose.',
 'raw_evidence_scope':'Complete primary final rows are retained. Development/comparator/lesion outcomes summarized only. Raw worlds reconstructed from constant seeds/source; no event-level all-arm audit possible from stored reports alone.'}
out['resources']={'all_four_saved_run_elapsed_s':sum(v['reported_elapsed_s'] for v in out['roots'].values()),'retained_report_bytes':sum(v['retained_bytes'] for v in out['roots'].values()),'model_runs_by_auditor':0,'builds_by_auditor':0,'ledger_charge_claim_ms':14000000,'component_estimates_sum_ms':14400000,'estimate_note':'Listed 900+1800+3600+4200+1500+2400 seconds total14400, vs14000 charged; no trustworthy measured breakdown to reconcile overlap. Preserve as estimated charge, not measured wall time.','energy':'UNAVAILABLE'}
out['script_sha256']=sha(P(__file__).read_bytes())
dest=P('/tmp/uor-pr1341-evidence-audit.json');dest.write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'output':str(dest),'provenance':out['provenance'],'exposure':out['exposure'],'primary_recount':out['primary_recount'],'intervention':out['intervention_reconstruction'],'resources':out['resources']},indent=2))
