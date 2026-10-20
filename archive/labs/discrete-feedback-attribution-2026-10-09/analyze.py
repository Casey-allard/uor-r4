#!/usr/bin/env python3
"""Saved displacement attribution only; no solver, model, score or candidate replay."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import resource
import struct
import time
import blake3

DIM, HALF, GUARDS = 1920, 960, 380
REPORT = '3bbcc79268cd5f13ccfb63fe244bf99ff97d43820ef7dbe5ed4c5544d3440cc6'
MANIFEST = 'f616174ce7c4b244b331ea48b9d61340201662e75faef086c2d6835a3ee9ef74'
CONFIG = '63b954032e10c7d5b6c3c2a1db26150b2b53d50ab098e9bf3c9e73840652617b'
SOURCE = '7381d670735acddb0fac1f7abc0d83498aed04fc'
OUTPUT_CAP = 50 * 1024 * 1024
FAMILIES = ['prefix.coefficients', 'generate.unary']


def need(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for b in iter(lambda: f.read(1024 * 1024), b''):
            h.update(b)
    return h.hexdigest()


def read(path):
    with path.open() as f:
        return json.load(f)


def finite(v, n):
    need(isinstance(v, list) and len(v) == n and all(type(x) in (int, float) and math.isfinite(x) for x in v), 'finite vector shape')
    return v


def dot(a, b):
    need(len(a) == len(b), 'dot shape')
    total = 0.0
    for x, y in zip(a, b):
        total += x * y
        need(math.isfinite(total), 'nonfinite dot')
    return total


def norm(v):
    return math.sqrt(dot(v, v))


def f32(x):
    return struct.unpack('<f', struct.pack('<f', x))[0]


def bits(x):
    return struct.unpack('<I', struct.pack('<f', x))[0]


def native_code(x):
    need(math.isfinite(x) and -1.75 <= x <= 1.75, 'master domain')
    y = f32(4.0 * x)
    return int(math.copysign(math.floor(abs(y) + 0.5), y))


def displacement(masters, destination_bits):
    need(len(destination_bits) == len(masters), 'destination shape')
    dest, delta = [], []
    for m, b in zip(masters, destination_bits):
        need(type(b) is int and 0 <= b <= 0xffffffff, 'destination bit domain')
        x = struct.unpack('<f', struct.pack('<I', b))[0]
        q = native_code(x)
        need((q == native_code(m) and b == bits(m)) or
             (q != native_code(m) and b == bits(q * 0.25)), 'noncanonical changed code or lost fractional bits')
        dest.append(x)
        delta.append(float(x) - float(m))
    return dest, delta


def first_duplicate(seen, signature, index):
    old = seen.get(signature)
    if old is None:
        seen[signature] = index
    return old


def signed_pressure(unit, delta):
    need(len(unit) == len(delta), 'pressure shape')
    signed = [a * b for a, b in zip(unit, delta)]
    return signed, [max(0.0, -x) for x in signed], [max(0.0, x) for x in signed]


def self_test():
    signed, adverse, helpful = signed_pressure([1., -2., 0.], [-3., -2., 4.])
    need(signed == [-3., 4., 0.] and sum(helpful) - sum(adverse) == dot([1., -2., 0.], [-3., -2., 4.]), 'signed closure')
    need(dot([1., 2., 3., 4.], [2., -1., 1., -2.]) == dot([1., 2.], [2., -1.]) + dot([3., 4.], [1., -2.]), 'family closure fixture')
    seen = {}
    need(first_duplicate(seen, (1, 2), 0) is None and first_duplicate(seen, (1, 2), 1) == 0, 'dedup first identity')
    m = f32(.12)
    need(displacement([m, -0.0], [bits(m), bits(-0.0)])[1] == [0., 0.], 'fractional and signed zero')
    for operation in [lambda: finite([math.nan], 1), lambda: finite([1.], 2),
                      lambda: displacement([m], [bits(.0)]), lambda: dot([1.], [])]:
        try:
            operation()
        except ValueError:
            pass
        else:
            raise ValueError('malformed/tampered input accepted')
    print('PASS: signed/family closure, first-bitwise dedup, fractional retention, nonfinite/shape/tamper rejection')


def verify_seal(root):
    need(sha(root/'report.json') == REPORT and sha(root/'manifest.json') == MANIFEST, 'fixed completed producer identity')
    manifest = read(root/'manifest.json')
    entries = manifest['files']
    names = [e['path'] for e in entries]
    need(len(names) == len(set(names)), 'duplicate seal entries')
    actual = set()
    for p in root.rglob('*'):
        need(not p.is_symlink(), 'symlink in sealed input')
        if p.is_file():
            actual.add(str(p.relative_to(root)))
    need(actual == set(names) | {'manifest.json'}, 'complete normative file inventory')
    for e in entries:
        p = root/e['path']
        need(p.resolve().is_relative_to(root.resolve()) and p.is_file(), 'sealed path escape')
        h = blake3.blake3()
        with p.open('rb') as f:
            for b in iter(lambda: f.read(1024*1024), b''):
                h.update(b)
        need(p.stat().st_size == e['bytes'] and h.hexdigest() == e['blake3'], 'normative seal '+e['path'])
    return len(entries)


def analyze(root, out):
    started = time.monotonic()
    root, out = root.resolve(), out.resolve()
    need(root.is_dir() and out.is_dir() and not out.is_relative_to(root), 'separate existing input and claimed output required')
    need({p.name for p in out.iterdir()} == {'attempt.json'}, 'output must be fresh Rust-claimed root containing only attempt.json')
    count = verify_seal(root)
    report, gradient, margins, journal, config = [read(root/n) for n in ['report.json', 'coupled-gradient-receipt.json', 'protected-margin-receipt.json', 'coupled-construction.json', 'config.json']]
    need(sha(root/'config.json') == CONFIG and report['policy'] == journal['policy'], 'config and report/journal policy identity')
    direction_policy=journal['policy']['protected_direction']
    need(direction_policy['outer_rounds']==32 and direction_policy['inner_passes']==32 and direction_policy['rho']==1 and direction_policy['coordinates']==DIM and direction_policy['maximum_alternatives']==32, 'fixed feedback policy')
    need(report['status'] == 'COMPLETED' and report['source_commit'] == SOURCE and report['new_backward_calls'] == 0 and report['new_training_graph_forwards'] == 0, 'completed saved-credit scope')
    need(config['coupled_episode_learning']['prefix_transaction'] == 'protected_discrete_feedback' and gradient['prefix_transaction'] == 'protected_joint_vector', 'new constructor and inherited derivative distinction')
    need(journal['schema'] == 'uor-r4.protected-discrete-feedback/1' and journal['selected']['status'] == 'unchanged', 'bounded unchanged constructor')
    need(sha(root/'protected-margin-receipt.json') == gradient['protected_margin_receipt_sha256'] == journal['protected_margin_receipt_sha256'], 'margin hash chain')
    need(gradient['physical_backward_calls'] == 31 and gradient['weighted_roles'] == 32 and margins['backward_calls'] == 380, 'inherited derivative populations')
    used = {}
    def raw(entry, n):
        leaf = entry['file']; p = root/leaf
        need(p.resolve().is_relative_to(root) and sha(p) == entry['sha256'] and p.stat().st_size == n*4, 'raw file identity')
        used[leaf] = {'sha256': entry['sha256'], 'bytes': n*4}
        return finite(list(struct.unpack('<'+'f'*n, p.read_bytes())), n)
    files = {(e['family'], e['kind']): e for e in gradient['files']}
    need(len(files) == 4, 'four unique aggregate/master arrays')
    masters, g = [], []
    for family in FAMILIES:
        masters.extend(raw(files[family, 'initial-master'], HALF))
        g.extend(raw(files[family, 'gradient'], HALF))
    need(len(gradient['perterm']) == 31, '31 physical weighted terms')
    for family, lo in zip(FAMILIES, [0, HALF]):
        aggregate = [0.]*HALF
        for i, term in enumerate(gradient['perterm']):
            need(term['physical_index'] == i, 'ordered physical derivative terms')
            entries = [e for e in term['families'] if e['family'] == family]
            need(len(entries) == 1 and entries[0]['status'] == 'PRESENT' and not entries[0]['missing_gradient_filled_zero'], 'present weighted family gradient')
            values = raw(entries[0], HALF)
            aggregate = [f32(a+b) for a,b in zip(aggregate, values)]
        need([bits(x) for x in aggregate] == [bits(x) for x in g[lo:lo+HALF]], 'ordered f32 aggregate')
    terms = margins['terms']; need(len(terms) == GUARDS, '380 protected terms')
    js, units, lengths, identities = [], [], [], []
    for i, term in enumerate(terms):
        need(term['guard_index'] == i and term['status'] == 'PRESENT', 'protected row order/presence')
        j = raw(term, DIM); length = norm(j)
        need(term['all_zero'] == (length == 0.), 'protected zero presence')
        js.append(j); lengths.append(length); units.append([x/length for x in j] if length else j[:])
        identities.append({k: term[k] for k in ['guard_index', 'input_index', 'position', 'id', 'actual_prefix_ids', 'winner', 'rival', 'winner_mass', 'rival_mass']})
        need(term['winner_mass'] > 0 and term['rival_mass'] > 0 and term['winner'] != term['rival'], 'fixed positive original contrast')
    trials = journal['joint_vectors']
    need(len(trials) == 32 and [t['round'] for t in trials] == list(range(32)), 'ordered32 rounds')
    seen, summaries, violation_sets, unique_indices = {}, [], [], []
    coord_file = (out/'coordinates.jsonl').open('x')
    guard_file = (out/'guards.jsonl').open('x')
    output_bytes = 0
    def line(stream, value):
        nonlocal output_bytes
        data = json.dumps(value, separators=(',', ':'), allow_nan=False)+'\n'
        output_bytes += len(data.encode())
        need(output_bytes < OUTPUT_CAP-2*1024*1024, 'detailed output cap')
        stream.write(data)
    previous_bits, previous_violations = None, None
    try:
        for trial in trials:
            r = trial['round']; destination_bits = trial['destination_master_bits']
            dest, delta = displacement(masters, destination_bits)
            need(delta == finite(trial['actual_delta'], DIM), 'exact widened displacement')
            signature = tuple(destination_bits); duplicate = first_duplicate(seen, signature, r)
            need(duplicate == trial['duplicate_of'], 'first bitwise duplicate')
            if duplicate is None: unique_indices.append(r)
            residuals = [dot(j, delta) for j in units]; tolerance = 1e-10*norm(delta)
            gd = dot(g, delta)
            need(residuals == finite(trial['quantized_margin_residuals'], GUARDS) and tolerance == trial['quantized_tolerance'] and gd == trial['actual_CE_linear_delta'], 'exact saved dots/residuals/tolerance')
            passed = all(x >= -tolerance for x in residuals)
            eligible = passed and gd < 0 and any(x != 0 for x in delta)
            expected_native_status='NOT_CHECKED_DUPLICATE' if duplicate is not None else 'NOT_CHECKED_LINEAR_INELIGIBLE'
            need(not eligible and trial['native']['guard_status']==expected_native_status and trial['native']['feasible'] is False, 'saved unscored duplicate/ineligible boundary')
            need(trial['quantized_constraints_passed'] == passed and trial['eligible'] == eligible and trial['incumbent_epoch'] == 0, 'actual displacement screen/epoch')
            need(math.isfinite(trial['feedback_norm']) and math.isfinite(trial['primal_norm']), 'saved norms finite')
            violated = {i for i,x in enumerate(residuals) if x < -tolerance}; violation_sets.append(violated)
            adverse, helpful = [0.]*DIM, [0.]*DIM
            guards = []
            for i, (j, u, term) in enumerate(zip(js, units, terms)):
                raw_delta = dot(j, delta); prefix_raw = dot(j[:HALF], delta[:HALF]); generate_raw = dot(j[HALF:], delta[HALF:])
                unit_prefix = dot(u[:HALF], delta[:HALF]); unit_generate = dot(u[HALF:], delta[HALF:])
                original_margin = math.log(term['winner_mass']/term['rival_mass'])
                row = {'round':r, 'guard':identities[i], 'violated':i in violated, 'unit_residual':residuals[i], 'tolerance':tolerance, 'raw_J_norm':lengths[i], 'raw_J_delta':raw_delta, 'raw_family_contribution':{'Prefix':prefix_raw,'Generate':generate_raw}, 'unit_family_contribution':{'Prefix':unit_prefix,'Generate':unit_generate}, 'family_raw_sum_residual':raw_delta-(prefix_raw+generate_raw), 'family_unit_sum_residual':residuals[i]-(unit_prefix+unit_generate), 'original_margin_ln_mass_ratio':original_margin, 'linearized_fixed_pair_margin':original_margin+raw_delta, 'diagnostic_only_not_native_pool_replay':True}
                line(guard_file, row); guards.append(row)
                if i in violated:
                    for k, (uj, d) in enumerate(zip(u, delta)):
                        contribution = uj*d; adverse[k] += max(0., -contribution); helpful[k] += max(0., contribution)
            order = sorted(range(DIM), key=lambda k:(-adverse[k], k)); total = sum(adverse)
            for k,(m,x,d) in enumerate(zip(masters,dest,delta)):
                line(coord_file, {'round':r,'coordinate':k,'family':'Prefix' if k<HALF else 'Generate','family_index':k%HALF,'original_bits':bits(m),'destination_bits':destination_bits[k],'original_code':native_code(m),'destination_code':native_code(x),'actual_delta':d,'objective_gradient':g[k],'objective_contribution':g[k]*d,'adverse_pressure_on_violated_unit_rows':adverse[k],'helpful_pressure_on_violated_unit_rows':helpful[k]})
            transition = None if previous_violations is None else {'retained':sorted(violated & previous_violations),'new':sorted(violated-previous_violations),'resolved':sorted(previous_violations-violated),'jaccard':len(violated & previous_violations)/len(violated | previous_violations) if violated | previous_violations else 1.}
            summaries.append({'round':r,'duplicate_of':duplicate,'violated_guard_indices':sorted(violated),'changed_coordinates':sum(d!=0 for d in delta),'changed_by_family':{'Prefix':sum(d!=0 for d in delta[:HALF]),'Generate':sum(d!=0 for d in delta[HALF:])},'changed_bits_since_previous':None if previous_bits is None else sum(a!=b for a,b in zip(previous_bits,destination_bits)),'objective_dot':gd,'objective_dot_family':{'Prefix':dot(g[:HALF],delta[:HALF]),'Generate':dot(g[HALF:],delta[HALF:])},'objective_family_sum_residual':gd-(dot(g[:HALF],delta[:HALF])+dot(g[HALF:],delta[HALF:])),'minimum_unit_residual':min(residuals),'tolerance':tolerance,'constraints_passed':passed,'eligible':eligible,'native_scope':trial['native'],'saved_feedback_norm':trial['feedback_norm'],'saved_primal_norm':trial['primal_norm'],'consecutive_violated_sets':transition,'coordinate_adverse_total':total,'coordinate_helpful_total':sum(helpful),'signed_pressure_net':sum(helpful)-total,'sum_violated_unit_residuals':sum(residuals[i] for i in sorted(violated)),'pressure_sum_closure_residual':(sum(helpful)-total)-sum(residuals[i] for i in sorted(violated)),'violated_family_sign_counts':{name:sum(guards[i]['unit_family_contribution']['Prefix']*ps>0 and guards[i]['unit_family_contribution']['Generate']*gs>0 for i in violated) for name,ps,gs in [('both_adverse',-1,-1),('Prefix_adverse_Generate_helpful',-1,1),('Prefix_helpful_Generate_adverse',1,-1),('both_helpful',1,1)]},'family_pressure':{name:{'adverse':sum(adverse[lo:hi]),'helpful':sum(helpful[lo:hi])} for name,lo,hi in [('Prefix',0,HALF),('Generate',HALF,DIM)]},'top_adverse_fraction':{str(n):sum(adverse[k] for k in order[:n])/total if total else 0. for n in [10,100]},'top_adverse_coordinates':[{'coordinate':k,'adverse':adverse[k],'helpful':helpful[k]} for k in order[:20]],'violated_linearized_fixed_margin_positive':sum(guards[i]['linearized_fixed_pair_margin']>0 for i in violated),'violated_linearized_fixed_margin_nonpositive':sum(guards[i]['linearized_fixed_pair_margin']<=0 for i in violated)})
            previous_bits, previous_violations = destination_bits, violated
    finally:
        coord_file.close(); guard_file.close()
    need(len(unique_indices)==28 and all(not s['eligible'] for s in summaries), 'fixed completed outcome28unique/zeroeligible')
    def frequency(indices):
        sets=[violation_sets[i] for i in indices]
        return {'round_indices':indices,'union':sorted(set.union(*sets)),'intersection':sorted(set.intersection(*sets)), 'frequencies':[dict(identities[i],violated_rounds=sum(i in s for s in sets)) for i in range(GUARDS)]}
    def coordinate_sequence(indices):
        changed_sets=[{k for k,b in enumerate(trials[i]['destination_master_bits']) if b!=bits(masters[k])} for i in indices]
        return {'round_indices':indices,'changed_coordinate_union':sorted(set.union(*changed_sets)), 'changed_coordinate_intersection':sorted(set.intersection(*changed_sets)), 'changed_round_frequency':[sum(k in changed for changed in changed_sets) for k in range(DIM)]}
    result = {'schema':'uor-r4.saved-discrete-feedback-attribution/1','status':'PASS_SAVED_ATTRIBUTION','input_report_sha256':REPORT,'input_manifest_sha256':MANIFEST,'source_commit':SOURCE,'config_sha256':sha(root/'config.json'),'script_sha256':sha(Path(__file__)),'complete_input_files_verified':count,'scope':'saved32round displacements and inherited31objective/380protected derivative arrays; no solver/inner-loop/search/model/native score replay','pressure_scope':'sum adverse/helpful signed unitJ_coordinate*actual_delta on each round violated set; descriptive concentration, not fraction of native failure','linearized_margin_scope':'original fixed winner/rival ln integer mass ratio plus rawJdelta; no candidate score or feasibility inference','rounds':summaries,'distinct_round_indices':unique_indices,'all32_coordinate_sequence':coordinate_sequence(list(range(32))),'unique28_coordinate_sequence':coordinate_sequence(unique_indices),'all32_guard_sequence':frequency(list(range(32))),'unique28_guard_sequence':frequency(unique_indices),'fixed_representatives':{'first_round':summaries[0],'last_round':summaries[-1]},'raw_input_files':used,'saved_norm_scope':'reported scalar feedback/primal norms only; z/u/inner iterations not reconstructed','native_proposals':journal['summary']['evaluated_alternatives'],'elapsed_seconds':time.monotonic()-started}
    need(result['native_proposals']==0, 'no measured native offers')
    with (out/'report.json').open('x') as f: json.dump(result,f,indent=2,allow_nan=False);f.write('\n')
    return result


def main():
    parser=argparse.ArgumentParser(); parser.add_argument('--self-test',action='store_true'); parser.add_argument('--run',type=Path);parser.add_argument('--output',type=Path);args=parser.parse_args()
    if args.self_test: self_test();return
    need(args.run is not None and args.output is not None,'--run and --output required')
    need(args.output.is_dir() and {p.name for p in args.output.iterdir()}=={'attempt.json'} and not args.output.resolve().is_relative_to(args.run.resolve()), 'fresh separate Rust-claimed output precondition')
    started=time.monotonic(); status='FAILED';error=None
    try:
        result=analyze(args.run,args.output);status=result['status']
    except Exception as exc:
        error=str(exc)
        if args.output.is_dir() and (args.output/'attempt.json').is_file() and not (args.output/'report.json').exists():
            with (args.output/'report.json').open('x') as f:json.dump({'status':'FAILED','error':error,'script_sha256':sha(Path(__file__))},f,indent=2)
        raise
    finally:
        if args.output.is_dir() and (args.output/'attempt.json').is_file():
            execution={'status':status,'error':error,'elapsed_seconds':time.monotonic()-started,'max_rss_platform_units':resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,'rss_units':'bytes on macOS; KiB on Linux','workers':1,'model_calls':0,'native_score_calls':0,'gradient_calls':0,'solver_replay_calls':0,'script_sha256':sha(Path(__file__))}
            with (args.output/'execution.json').open('x') as f:json.dump(execution,f,indent=2)
            files={p.name:{'sha256':sha(p),'bytes':p.stat().st_size} for p in args.output.iterdir() if p.is_file()}
            need(sum(e['bytes'] for e in files.values())<OUTPUT_CAP,'complete output cap')
            with (args.output/'output-files.json').open('x') as f:json.dump({'files':files,'scope':'hash inventory excluding this file; normative seal supplied externally'},f,indent=2)
    print(json.dumps({'status':status,'rounds':32,'unique_destinations':28,'output':str(args.output)}))

if __name__=='__main__': main()
