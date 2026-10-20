#!/usr/bin/env python3
"""Independent saved Generate ordered-pair audit, not a model implementation.
Reuses the immutable fixture-verified integer pool kernel. No encoder/backward/
proposal/native execution. Parent separately claims/seals/verifies the audit and
all model roots with normative Rust tools. Run only after exact-source review.
"""
import argparse, array, hashlib, importlib.util, json, math, pathlib, resource, struct, sys, time
KERNEL = pathlib.Path('/workspace/uor-r4/codex/sol-generate-episode-learning/generate-independent-reader.py')
KERNEL_SHA = '8e5383e5ac93133c071ab540cf85cece0edc6e70d95367a4230d8574906ec8f4'
NAME='generate.pair'; COUNT=57600; N=4096; UNIT=1<<20

def need(ok,why):
    if not ok: raise ValueError(why)
def sha(path):
    h=hashlib.sha256()
    with pathlib.Path(path).open('rb') as f:
        for b in iter(lambda:f.read(1<<20),b''): h.update(b)
    return h.hexdigest()
def read_json(path): return json.loads(pathlib.Path(path).read_bytes())
def raw_f32(path):
    b=pathlib.Path(path).read_bytes();need(len(b)==COUNT*4,'full57600 rawf32 bytes')
    v=list(struct.unpack('<57600f',b));need(all(math.isfinite(x) for x in v),'finite raw57600')
    return b,v
def authenticate_kernel(path):
    need(sha(path)==KERNEL_SHA,'immutable sparse integer kernel hash')
    s=importlib.util.spec_from_file_location('pair_immutable_kernel',path)
    g=importlib.util.module_from_spec(s);s.loader.exec_module(g);return g

def packed_master(m):
    need(math.isfinite(m) and -1.75<=m<=1.75,'native admitted actual fractional master')
    z=m*4.;return math.floor(z+.5) if z>=0. else math.ceil(z-.5)
def total_key(x):
    need(math.isfinite(x),'finite ranking utility');return (x,0 if math.copysign(1.,x)<0 else 1)
def pair_order(masters,gradients,maximum):
    need(len(masters)==len(gradients)==COUNT and 1<=maximum<=960,'complete family/bounded selected population')
    values=[]
    for i,(m,g) in enumerate(zip(masters,gradients)):
        q=packed_master(m)
        potential=min((g*(c*.25-m) for c in range(-7,8) if c!=q),key=total_key)
        values.append((i,potential))
    values.sort(key=lambda x:(total_key(x[1]),x[0]));return [x[0] for x in values[:maximum]]
def pair_incidence(pools,algebra):
    need(len(pools)==391 and len(algebra.e['edges'])==4,'391 states/four ordered edges')
    buckets=[array.array('I') for _ in range(COUNT)]
    proto,product,inverse=algebra.p['prototypes'],algebra.a['product'],algebra.a['inverse']
    for row,pool in enumerate(pools):
        need(len(pool['post'])==8,'eight-lane post')
        for token in range(N):
            relative=[product[(inverse[s]<<7)|proto[token*8+l]] for l,s in enumerate(pool['post'])]
            for edge,e in enumerate(algebra.e['edges']):
                key=edge*14400+relative[e['left']]*120+relative[e['right']]
                buckets[key].append((row<<12)|token)
    return buckets

def audit_gradient(run,frames,original,p):
    receipt=read_json(run/'generate-gradient-receipt.json')
    need(receipt['active_parameter_names']==[NAME] and receipt['shape']==[4,120,120] and
         receipt['coefficient_backward_calls']==31 and receipt['weighted_roles']==32 and
         len(receipt['perterm'])==31,'fresh full-family31 pair gradients/32 coalesced roles')
    total=[0.]*COUNT
    for i,(f,t) in enumerate(zip(frames,receipt['perterm'])):
        rr=f['row'];leaf=f'generate-gradient-term-{i:02}.f32le'
        need(t['physical_index']==i and t['file']==leaf and t['bytes']==COUNT*4 and
             t['status']=='PRESENT' and t['missing_gradient_filled_zero'] is False,'exact raw inventory/no missing fill')
        need(all(t[k]==rr[k] for k in ['input_index','position','id','weight']) and
             t['target']==rr['target_label_only'],'weighted term state and role identity')
        raw,v=raw_f32(run/leaf);need(hashlib.sha256(raw).hexdigest()==t['sha256'],'rawterm hash')
        norm=math.sqrt(sum(float(x)**2 for x in v))
        need(t['all_zero']==all(x==0. for x in v) and abs(t['l2_norm']-norm)<=1e-11*(1+norm),'raw present-zero/norm')
        need(t['termination_target']==(t['target']==1) and t['period_target']==(t['target']==16),'terminal term identity')
        total=[p.f32(a+b) for a,b in zip(total,v)]
    agg=receipt['aggregate'];raw,_=raw_f32(run/'generate-gradient.f32le')
    need(agg['file']=='generate-gradient.f32le' and agg['bytes']==COUNT*4 and
         hashlib.sha256(raw).hexdigest()==agg['sha256'] and raw==struct.pack('<57600f',*total),'ordered31 f32 aggregate including signed zeros')
    meta=read_json(original/'generate-source/metadata.json')['parameters']
    frozen=['generate.unary','generate.bias','generate.prototype_choices']
    for k in meta:
        f=original/'generate-source'/f'{k}.f32le'
        need(f.stat().st_size==meta[k]['bytes'] and sha(f)==meta[k]['sha256'],'original actual master authority '+k)
    need(receipt['frozen_other_generate_masters']=={k:meta[k]['sha256'] for k in frozen},'frozen master receipts')
    return total,receipt

def audit_constructor(run, pop, states, mapping, roles, masters, gradient, csr, maximum):
    construction = read_json(run / "generate-construction.json")
    rank = pair_order(masters, gradient, maximum)
    need(read_json(run / "generate-coordinate-order.json") == rank and
         len(construction["coordinate_records"]) == maximum, "frozen bounded full-family legal order")
    baseline = objective(pop, states, mapping, roles, {})
    current = baseline
    current_masters = masters[:]
    epoch = accepted = row_patches = evaluated = 0
    max_copy = max(len(s.copy_ids) for s in states)
    for ordinal, (index, record) in enumerate(zip(rank, construction["coordinate_records"])):
        incumbent = packed_master(current_masters[index])
        need(record["order"] == ordinal and record["index"] == index and
             record["incumbent_epoch"] == epoch and record["incumbent_code"] == incumbent,
             "coordinate incumbent identity/epoch")
        need(struct.pack("<f", record["original_master"]) == struct.pack("<f", masters[index]) and
             struct.pack("<f", record["gradient"]) == struct.pack("<f", gradient[index]) and
             record["initial_zero_gradient"] == (gradient[index] == 0),
             "original fractional rank evidence")
        by_row = {}
        for atom in csr[index]:
            row, token = atom >> 12, atom & 4095
            by_row.setdefault(row, []).append(token)
        expected_codes = [q for q in range(-7, 8) if q != incumbent]
        need([a["code"] for a in record["alternatives"]] == expected_codes,
             "exact ALL14 legal alternatives including countergradient")
        best = None
        for code, alternative in zip(expected_codes, record["alternatives"]):
            delta = (code - incumbent) * UNIT
            stages = {}
            def stage(row):
                if row in by_row and row not in stages:
                    state = states[row]
                    stages[row] = state.stage(
                        ((t, state.scores[t] + delta) for t in by_row[row]),
                        pop[row]["row"]["target_label_only"])
            for row in mapping:
                stage(row)
            value = objective(pop, states, mapping, roles, stages)
            compare_objective(alternative, value)
            objective_gate = (improves(current["combined"], value["combined"]) and
                              value["correct_reference_frames"] == 17)
            checked = []
            first = None
            if objective_gate:
                for row in range(380):
                    if row in by_row:
                        stage(row)
                        checked.append(row)
                    chosen = masses_view(row, states, pop[row]["row"]["target_label_only"],
                                         stages)["chosen_token_id"]
                    if chosen != pop[row]["row"]["target_label_only"]:
                        rr = pop[row]["row"]
                        first = {"row": row, "input_index": rr["input_index"],
                                 "position": rr["position"],
                                 "required": rr["target_label_only"], "chosen": chosen}
                        break
            feasible = objective_gate and first is None
            status = ("NOT_RUN_OBJECTIVE_INELIGIBLE" if not objective_gate else
                      "VETO" if first is not None else "PASS")
            digest_rows = []
            for row in sorted(stages):
                s = stages[row]["summary"]
                digest_rows.append(dict(row=row, **{k: s[k] for k in
                    ["reference_q24", "total_weight_q31", "chosen_token_id",
                     "chosen_weight_q31", "target_mass"]}))
            need(alternative["delta_code"] == code - incumbent and
                 alternative["actual_master_delta"] == code * .25 - masters[index] and
                 alternative["objective_gate"] == objective_gate and
                 alternative["guard_status"] == status and
                 alternative["checked_guard_ids"] == checked and
                 alternative["first_guard_failure"] == first and
                 alternative["feasible"] == feasible, "native alternative gate/traversal")
            need(alternative["staged_summary_digest"] == object_digest(digest_rows) and
                 alternative["changed_atoms"] == len(csr[index]) and
                 alternative["potential_affected_rows"] == len(by_row) and
                 alternative["row_patches"] == len(stages),
                 "all-incidence changedatoms and compact staged math digest")
            for flag in ["used_full_reduction_count", "maximum_scans", "winner_scans"]:
                need(0 <= alternative[flag] <= len(stages), "bounded producer optimization witness")
            row_patches += len(stages)
            evaluated += len(stages)
            if feasible and (best is None or (value["combined"], code) < (best[0], best[1])):
                # Hold at most incumbent-best pending and current temporary patch.
                best = (value["combined"], code, stages, value)
        selected = record["selected"]
        if best is None:
            need(selected == {"code": incumbent, "status": "unchanged", "restaged_rows": 0},
                 "no feasible alternative leaves coordinate unchanged")
        else:
            ce, code, stages, current = best
            need(selected["status"] == "committed" and selected["code"] == code and
                 selected["restaged_rows"] == 0 and
                 abs(selected["combined"] - ce) <= 1e-12 * (1 + abs(ce)),
                 "best nativeCE/signedcode choice at same epoch")
            need(set(stages) == set(by_row), "feasible pending covers all affected union rows")
            commit_batch((states[row], stages[row]) for row in sorted(stages))
            current_masters[index] = code * .25
            epoch += 1
            accepted += 1
        need(record["epoch_after"] == epoch, "one atomic coordinate commit epoch")
    summary = construction["summary"]
    need(summary["coordinates"] == maximum and summary["alternatives"] == maximum * 14 and
         summary["accepted_coordinates"] == accepted and summary["row_patches"] == row_patches and
         summary["revisited_coordinates"] == summary["selected_pending_restage_count"] == 0,
         "complete all-alternative constructor counters")
    compare_objective(summary["initial"], baseline)
    compare_objective(summary["final"], current)
    return current_masters, baseline, current, {
        "accepted": accepted, "alternative_codes": maximum * 14, "row_patches": evaluated,
        "maximum_copy_aliases": max_copy, "revisited": 0}

def audit_export(cp,original,masters,base,p,source):
    for folder in ['native','source','cue','prefix']:
        need(base.directory(cp/folder)==base.directory(original/folder),'frozen entire '+folder)
    for folder in ['cue-source','prefix-source']:
        if (original/folder).exists(): need(base.directory(cp/folder)==base.directory(original/folder),'frozen '+folder)
    for leaf in ['read-state-bridge.bin','read-state-bridge-categorical.bin']:
        need((cp/leaf).read_bytes()==(original/leaf).read_bytes(),'frozen bridge '+leaf)
    base.frozen_raw(original/'read-state-bridge-source',cp/'read-state-bridge-source')
    base.frozen_raw(original/'continuation-source',cp/'continuation-source')
    old=read_json(original/'generate.bin');new=read_json(cp/'generate.bin')
    expected=json.loads(json.dumps(old['payload']));energy=expected['energy'];packed=energy['pair_packed'][:]
    need(len(energy['edges'])==4 and energy['lanes']==8 and len(packed)==4*8192,'native pair physical layout')
    for edge in range(4):
        for left in range(120):
            for right in range(120):
                logical=edge*14400+left*120+right
                physical=edge*16384+left*128+right;shift=(physical%2)*4
                packed[physical//2]=(packed[physical//2]&~(15<<shift))|((packed_master(masters[logical])&15)<<shift)
    energy['pair_packed']=packed
    need(new['payload']==expected,'only logical pair nibbles changed; all padding and other payload frozen')
    em=dict(old['metadata']);em['payload_sha256']=new['metadata']['payload_sha256']
    need(new['metadata']==em and len(em['payload_sha256'])==64 and
         all(x in '0123456789abcdef' for x in em['payload_sha256']),'frozen Generate format/geometry/binding metadata')
    om=read_json(original/'generate-source/metadata.json');nm=read_json(cp/'generate-source/metadata.json')
    need(set(om['parameters'])==set(nm['parameters'])=={NAME,'generate.unary','generate.bias','generate.prototype_choices'},'exact four master families')
    for name,info in nm['parameters'].items():
        f=cp/'generate-source'/f'{name}.f32le'
        need(f.stat().st_size==info['bytes'] and sha(f)==info['sha256'],'candidate f32 master hash/bytes '+name)
        if name==NAME:
            need(info['shape']==[4,120,120] and info['bytes']==COUNT*4 and
                 f.read_bytes()==struct.pack('<57600f',*masters),'final accepted and unselected fractional pair bits')
        else:
            need(info==om['parameters'][name] and f.read_bytes()==(original/'generate-source'/f.name).read_bytes(),'frozen actual master '+name)
    em=dict(om);em['parameters']=nm['parameters'];need(nm==em,'Generate master metadata only pair identity changed')
    ou=read_json(original/'continuation-field.bin');nu=read_json(cp/'continuation-field.bin')
    eu=json.loads(json.dumps(ou));eu['metadata']['generate_sha256']=sha(cp/'generate.bin');eu['metadata']['generate_metadata']=new['metadata']
    need(nu==eu,'whole frozen U artifact with only new Generate metadata rebind')
    receipt=read_json(cp/'receipt.json')
    need(receipt==read_json(cp/'continuation-source/metadata.json') and receipt['mode']=='generate_pair_episode_learning' and
         receipt['active_parameter_names']==[NAME] and receipt['fresh_adam'] is False and
         receipt['optimizer_updates']==0 and receipt['new_gradients']==1 and receipt['coefficient_backward_calls']==31 and
         receipt['candidate_native_steps']==391 and receipt['generate_sha256']==sha(cp/'generate.bin'),'fresh pair export provenance')
    return receipt

def audit_resource(run,retained,cfg,maximum):
    v=read_json(run/'generate-resource-projection.json');prior=read_json(retained/'trajectory-resource-projection.json')
    pre=read_json(retained/'episode-pregradient-resource-projection.json')
    need(v['prior_projection']==prior and v['prior_pregradient_projection']==pre,'original measured projection authority')
    csr=391*N*4*4;offset=(COUNT-960)*8*3;scratch=(COUNT-960)*4*4
    numeric=prior['numeric_upper_bound']-prior['donor_cache_cap']-2*prior['actual_retained_guard_pool_bytes']+csr+3*(391*N*8*3)+(24<<20)+offset+scratch
    cache=csr+3*(391*N*8*3)+(8<<20)+offset
    retained_bytes=sum(f.stat().st_size for f in retained.rglob('*') if f.is_file())
    removed=(retained/'prefix-construction.json').stat().st_size
    report=retained_bytes-removed+(80<<20)+33*4*(COUNT-960)
    process=pre['process_ram_projection_bytes']+(64<<20)+scratch+offset
    need(v['numeric_upper_bound']==numeric<=512<<20 and v['cache_upper_bound']==cache<=256<<20 and
         v['process_ram_projection_bytes']==process<=4<<30 and v['report_projection_bytes']==report and
         report+(1<<20)<cfg['maximum_report_bytes'],'independent pair resource formulas and caps')
    need(v['selected_family']=='pair' and v['full_family_coordinates']==COUNT and v['selected_coordinate_cap']==maximum and
         v['all31_pair_gradient_bytes']==31*COUNT*4 and v['CSR_bytes']==csr and
         v['offset_count_cursor_extra_bytes']==offset and v['gradient_scratch_extra_bytes']==scratch and
         v['projected_alternatives']==maximum*14 and v['fresh_backwards_completed']==0,
         'bounded full-family gradient/CSR/trial projection before backward')
    need(v['numeric_cap']==v['report_cap']==512<<20 and v['cache_cap']==256<<20 and
         v['process_ram_cap']==4<<30,'exact unchanged owner caps')
    return v

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('run',type=pathlib.Path)
    for k in ['config','binary','execution','source-file']:
        ap.add_argument('--'+k,type=pathlib.Path,required=True)
    for k in ['config-sha256','binary-sha256','expected-source','source-sha256']:
        ap.add_argument('--'+k,required=True)
    ap.add_argument('--kernel',type=pathlib.Path,default=KERNEL)
    ap.add_argument('--qualification',type=pathlib.Path)
    ap.add_argument('--qualification-report-sha256');ap.add_argument('--qualification-manifest-sha256')
    args=ap.parse_args();start=time.monotonic();g=authenticate_kernel(args.kernel)
    # Reuse immutable arithmetic and schema helpers; do not invoke their main.
    global objective,compare_objective,improves,masses_view,commit_batch,object_digest
    objective=g.objective;compare_objective=g.compare_objective;improves=g.improves
    masses_view=g.masses_view;commit_batch=g.commit_batch;object_digest=g.object_digest
    e,base,p=g.immutable_helpers();run=args.run;cfg=read_json(args.config)
    need(sha(args.config)==args.config_sha256 and sha(args.binary)==args.binary_sha256 and
         sha(args.source_file)==args.source_sha256,'external raw config, preserved binary and reviewed source bytes')
    need(len(args.expected_source)==40 and all(x in '0123456789abcdef' for x in args.expected_source),'exact source commit')
    need(pathlib.Path(cfg['out'])==run and read_json(run/'config.json')==cfg,'semantic config equality; external bytes separately pinned')
    attempt=read_json(run/'attempt.json');launch=read_json(args.execution.with_name('launch.json'));execution=read_json(args.execution)
    need(attempt['schema']=='uor-r4.report-attempt/1' and len(attempt['argv'])==2 and
         pathlib.Path(attempt['argv'][1])==args.config and launch['argv']==attempt['argv'] and
         launch['binary']==attempt['argv'][0] and launch['pid']==attempt['pid'],'canonical launch argv/PID, preserved binary relocation allowed')
    need(launch['binary_sha256']==execution['binary_sha256']==args.binary_sha256 and
         launch['config_sha256']==execution['config_sha256']==args.config_sha256 and
         launch['started_utc']==execution['started_utc'] and execution['exit_code']==0,'terminal launch/execution identity')
    need(read_json(run/'external-config-binding.json')=={'path':str(args.config.resolve()),'sha256':args.config_sha256,
         'bytes':args.config.stat().st_size,'attempt_argv':attempt['argv']},'exact external byte-binding receipt')
    report=read_json(run/'report.json')
    need(report['schema']=='uor-r4.generate-episode-learning-report/1' and report['status']=='COMPLETED' and
         report['mode']=='generate_pair_episode_learning' and report['source_commit']==args.expected_source,'exact completed pair producer')
    c=cfg['generate_episode_learning'];maximum=c['maximum_coordinates']
    need(c['family']=='pair' and isinstance(maximum,int) and not isinstance(maximum,bool) and 1<=maximum<=960 and
         cfg['credit']=='raw_identity' and cfg['updates']==1 and cfg['loss_scope']=='all' and
         cfg['mode']=='joint_continuation','prospective generic pair bounded family and scalar objective')
    retained=pathlib.Path(c['retained_episode_root']);original_inputs=c['original_inputs']
    need(c['expected_report_sha256']==g.RETAINED_REPORT==sha(retained/'report.json') and
         c['expected_manifest_sha256']==g.RETAINED_SEAL==sha(retained/'manifest.json') and
         read_json(retained/'config.json')['prefix_fragment_learning']==original_inputs,'original episode authority/spec')
    inventories={'model':p.inventory(run),'original_episode':p.inventory(retained)}
    parent=pathlib.Path(original_inputs['retained_intermediate_root']);original=parent/'checkpoint-0001'
    need(pathlib.Path(cfg['checkpoint'])==original and sha(original/'native/metadata.json')==e.SOURCE_SHA and
         sha(original/'generate.bin')==e.G_SHA and sha(original/'continuation-field.bin')==e.U_SHA,'selected original seed Source/G/U')
    for field,pin in [('training_inputs','b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a'),
                      ('training_labels','84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155')]:
        need(sha(pathlib.Path(cfg[field]))==pin,'fixed authored panel '+field)
    roots=[(parent,base.P_REPORT,base.P_SEAL),(pathlib.Path(original_inputs['retained_probe_root']),base.B_REPORT,base.B_SEAL),
           (pathlib.Path(original_inputs['episode']['retained_supplement_root']),e.SUP_REPORT,e.SUP_SEAL)]
    roots.extend((pathlib.Path(w['capture']['root']),w['capture']['expected_report_sha256'],w['capture']['expected_manifest_sha256']) for w in original_inputs['episode']['phases'])
    rp=original_inputs['episode']['retained_projection'];roots.append((pathlib.Path(rp['root']),rp['expected_report_sha256'],rp['expected_manifest_sha256']))
    for root,rh,mh in roots:
        need(sha(root/'report.json')==rh and sha(root/'manifest.json')==mh,'original witness report/seal')
        if str(root) not in inventories: inventories[str(root)]=p.inventory(root)
    pop,pools,mapping,roles,objective_frames,original,ar=g.load_original_corpus(retained,original_inputs,e,base)
    native=read_json(original/'generate.bin');energy=native['payload']['energy']
    need(native['metadata']['lanes']==8 and native['metadata']['vocab_size']==4096 and
         native['metadata']['score_shift']==20 and len(energy['edges'])==4,'native full4096/eight lanes/four ordered edges')
    raw,masters=raw_f32(run/'generate-initial-masters.f32le')
    meta=read_json(original/'generate-source/metadata.json')['parameters']
    need(raw==(original/'generate-source/generate.pair.f32le').read_bytes() and
         meta[NAME]['shape']==[4,120,120] and meta[NAME]['bytes']==COUNT*4 and
         hashlib.sha256(raw).hexdigest()==meta[NAME]['sha256'],'all57600 original fractional pair master bytes')
    packed=[p.nib(energy['pair_packed'],edge*16384+left*128+right) for edge in range(4) for left in range(120) for right in range(120)]
    need([packed_master(x) for x in masters]==packed,'all57600 native pair quantization')
    gradient,gradient_receipt=audit_gradient(run,objective_frames,original,p)
    binding=read_json(run/'generate-original-master-binding.json')
    need(binding=={'source':str(original/'generate-source'),'parameters':{k:v['sha256'] for k,v in meta.items()},
         'native_sha256':e.G_SHA,'all_original_f32_restored_before_graph':True,'active_parameter_names':[NAME]},'all actual Generate master restore binding')
    need(read_json(run/'generate-forward-parity.json')=={'physical_frames':31,'weighted_roles':32,'all_before_any_backward':True,
         'native_raw_generate_copy_full_alias_pool':True,'frozen_poststate':True,'relaxed_transport':'NOT_RUN','device':'cuda'},'31 factual parity-before-backward producer witness')
    projection=audit_resource(run,retained,cfg,maximum)
    population=read_json(run/'generate-population.json');rows=[]
    for row,(f,pool) in enumerate(zip(pop,pools)):
        rr=f['row'];rows.append({'row':row,'input_index':rr['input_index'],'position':rr['position'],'id':rr['id'],
            'actual_prefix_ids':rr['actual_prefix_ids'],'target_label_only':rr['target_label_only'],
            'zero_weight_guard':row<380,'guard_weight':0.,'coalesced_objective_weight':rr['weight'],'post_state':pool['post'],'donor':pool['donor']})
    need(population['rows']==rows and population['objective_row_map']==mapping and population['roles']==roles and
         population['guards']==380 and population['unique_union']==391 and population['objective_physical_frames']==31 and
         population['weighted_roles']==32 and population['guard_population']==read_json(retained/'trajectory-protected-population.json'),'complete original391/31/32/380 population')
    csr=pair_incidence(pools,ar.a);offsets=[0];h=hashlib.sha256()
    for bucket in csr:
        offsets.append(offsets[-1]+len(bucket));b=array.array('I',bucket)
        if sys.byteorder!='little':b.byteswap()
        h.update(b.tobytes())
    incidence=read_json(run/'generate-incidence.json')
    need(incidence['legal_generate_ids']==list(range(N)) and incidence['postings']==391*N*4==offsets[-1] and
         incidence['offsets']==offsets and incidence['packed_atoms_sha256']==h.hexdigest() and
         incidence['generate_sha256']==e.G_SHA and incidence['source_binding']==read_json(original/'receipt.json')['parent'],'independent ordered-pair full-domain CSR')
    need(incidence['key_encoding']=='edge*14400+left_relative*120+right_relative; declared ordered edges; core keys[8..8+edges] minus960' and
         incidence['atom_encoding']=='row<<12|token; legal admitted IDs only, full raw4096 arrays retained','exact logical pair encoding')
    weights=g.IntegerWeights(ar.exp);states=[g.PoolState(o['generate'],f['ids'],o['copy'],weights) for f,o in zip(pop,pools)]
    baseline=objective(pop,states,mapping,roles,{});compare_objective(report['baseline_objective'],baseline)
    initial=read_json(run/'initial-original-objective.json')
    for key in ['combined','task','reference']:
        need(abs(initial[key]-baseline[key])<=1e-12*(1+abs(baseline[key])),'original native baseline '+key)
    need(len(initial['phases'])==15 and initial['correct_reference_frames']==17,'initial15/17 authority')
    for a,b in zip(initial['phases'],baseline['phases']):
        need(a['position']==b['position'] and a['target']==b['target'] and a['target_mass']==b['target_mass'] and
             a['total_mass']==b['total_mass'] and a['pool']['chosen_token_id']==b['chosen'],'initial phase mass/winner authority')
    need(abs(gradient_receipt['native_combined_ce']-baseline['combined'])<1e-12 and
         abs(gradient_receipt['float_graph_combined_ce']-baseline['combined'])<1e-5 and
         gradient_receipt['loss_anchor_tolerance']==1e-5,'saved weighted hard-anchor CE witness; backward not independently replicated')
    policy=report['policy'];need(policy['active_parameter_names']==[NAME] and policy['full_family_coordinate_count']==COUNT and
         policy['coordinate_count']==maximum and policy['selected_family']=='pair','generic full-family frozen ranking policy')
    need(read_json(run/'generate-construction.json')['policy']==policy,'journal policy equality')
    ar.cache.clear();ar.a.generate_cache.clear()
    current_masters,baseline,current,counters=audit_constructor(run,pop,states,mapping,roles,masters,gradient,csr,maximum)
    del csr
    compare_objective(report['candidate_objective'],current);compare_objective(read_json(run/'final-objective.json'),current)
    guards=all(states[i].current['chosen_token_id']==pop[i]['row']['target_label_only'] for i in range(380))
    gate=g.final_gate(baseline,current,guards)
    need(report['final_gate']==gate and report['finite_episode_positive']==gate['passed'] and
         guards and report['all_original380_preserved'] is True,'original15/17/380 final gate')
    fg=read_json(run/'final-trajectory-guards.json');need(fg['guards']==380 and fg['all_original_winners'] is True and len(fg['terms'])==380,'final380 receipt')
    for i,t in enumerate(fg['terms']):
        rr=pop[i]['row'];need(t['guard_index']==i and t['input_index']==rr['input_index'] and t['position']==rr['position'] and
             t['required_original_winner']==t['chosen']==rr['target_label_only'],'guard identity/winner')
        for key in ['reference_q24','total_weight_q31','chosen_token_id','chosen_weight_q31']:
            need(t['pool'][key]==states[i].current[key],'guard pool '+key)
    cp=run/'checkpoint-0001';receipt=audit_export(cp,original,current_masters,base,p,args.expected_source)
    need(report['candidate_receipt']==receipt,'export/report receipt agreement')
    snapshots=g.audit_native_snapshots(run,cp,pop,pools,states,e,base,p)
    need(report['selected_model'] is False and report['useful_candidate'] is False and report['weighted_roles']==32 and
         report['unique_objective_frames']==31 and report['candidate_native_steps']==391 and report['full512']=='NOT_RUN' and
         report['prefix_backward_calls']==0 and report['generate_backward_calls']==31 and report['new_generate_gradients']==1 and
         report['optimizer_updates']==0 and report['parent_master_bits_restored'] is True and
         report['autoregressive_rollout']=='NOT_RUN_SEPARATE_ARTIFACT_CHECK' and report['actual9']=='NOT_RUN','bounded pair model/restoration scope')
    if gate['passed']:
        need(args.qualification is not None and args.qualification_report_sha256 and args.qualification_manifest_sha256,'positive construction needs actual9 evidence')
        qualification=g.audit_actual_qualification(args,run,parent,cp,p,cfg,report)
    else:
        need(args.qualification is None,'negative construction has no undeclared actual9')
        qualification={'status':'NOT_RUN','reason':'complete conditional construction negative'}
    rss=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss*(1 if sys.platform=='darwin' else 1024)
    need(rss<=4<<30,'independent audit4GiB ceiling')
    print(json.dumps({'schema':'uor-r4.generate-pair-saved-audit/1','status':'PASS','source_commit':args.expected_source,
        'source_file_sha256':args.source_sha256,'reader_sha256':sha(pathlib.Path(__file__)),'kernel_sha256':KERNEL_SHA,
        'report_sha256':sha(run/'report.json'),'manifest_sha256':sha(run/'manifest.json'),'inventories':inventories,
        'family_coordinates':COUNT,'selected_coordinates':maximum,'raw_terms':31,'weighted_roles':32,'constructor':counters,
        'snapshots':snapshots,'final_gate':gate,'actual_qualification':qualification,'elapsed_seconds':time.monotonic()-start,
        'peak_RSS_bytes':rss,'scope':'Independent saved finite pool/gradient-sum/ranking/transaction/export arithmetic; no backward or encoder replication. Normative Rust full-file/BLAKE verification is separate parent evidence. No serving or general-language claim.'},sort_keys=True))

if __name__=='__main__':main()
