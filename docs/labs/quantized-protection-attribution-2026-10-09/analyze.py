#!/usr/bin/env python3
"""Authenticate and decompose four saved displacements. No model or candidate search."""
from pathlib import Path
import hashlib, json, math, struct, sys, time
import blake3

DIM = 1920
MANIFEST = 'cdadd7c5cbbbdca2225a71006ab58f70129e8df9e015b92f35b19359028a7167'
REPORT = 'b1d27f7b15e65d0aa8ea7e0a995b8bc76979610f97dbcb3055c8bbb434d0ac9b'
STAGES = ['intended', 'addition_subtraction_residual', 'clamp', 'f32_cast', 'quarter_round', 'preserved_fractional']

def need(value, message):
    if not value:
        raise ValueError(message)

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def f32(x):
    return struct.unpack('<f', struct.pack('<f', x))[0]

def bits(x):
    return struct.unpack('<I', struct.pack('<f', x))[0]

def code(x):
    need(math.isfinite(x) and -1.75 <= x <= 1.75, 'master range')
    y = f32(4.0 * x)
    return int(math.copysign(math.floor(abs(y) + 0.5), y))

def dot(a, b):
    need(len(a) == len(b), 'dot shape')
    return sum(x * y for x, y in zip(a, b))

def vector(v, n=DIM):
    need(len(v) == n and all(math.isfinite(x) for x in v), 'finite vector shape')
    return v

def decompose(m, eta, d):
    s = eta * d
    x = m + s
    c = min(1.75, max(-1.75, x))
    f = f32(c)
    q = code(f)
    original = code(m)
    z = q * 0.25
    v = m if q == original else z
    stages = [s, (x - m) - s, c - x, f - c, z - f, v - z]
    return {'original_bits': bits(m), 'destination_bits': bits(v), 'original_code': original,
            'destination_code': q, 'actual_delta': v - m, 'stages': dict(zip(STAGES, stages)),
            'closure': (v - m) - sum(stages), 'clamped': x != c,
            'unchanged_code': q == original, 'exact_half_tie': abs(f32(4.0*f)) % 1.0 == 0.5}

def self_test():
    need(code(f32(0.125)) == 1 and code(f32(-0.125)) == -1, 'ties away')
    n = decompose(-0.0, 1.0, 0.0)
    need(n['destination_bits'] == 0x80000000 and n['unchanged_code'], 'signed zero retention')
    n = decompose(f32(0.12), 1.0, 0.001)
    need(n['actual_delta'] == 0 and n['stages']['preserved_fractional'] != 0, 'fractional no-op')
    n = decompose(1.75, 1.0, 0.5)
    need(n['clamped'] and n['actual_delta'] == 0 and n['stages']['clamp'] == -0.5, 'clamp')
    n = decompose(0.0, 1.0, -0.125)
    need(n['destination_bits'] == bits(-0.25), 'negative changed tie')
    try:
        dot([1], [1, 2])
    except ValueError:
        pass
    else:
        raise ValueError('shape fail-open')
    print('PASS: ties-away, signed-zero, fractional no-op, clamp, changed tie, shape rejection')

def main(root, out):
    start = time.monotonic()
    need(root.is_dir() and not out.exists(), 'existing input and exclusive output required')
    out.mkdir(parents=True)
    def read(name):
        return json.loads((root / name).read_text())
    need(sha(root/'manifest.json') == MANIFEST and sha(root/'report.json') == REPORT, 'pinned source seals')
    manifest = read('manifest.json')
    names = {f['path'] for f in manifest['files']}
    need({str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()} == names | {'manifest.json'}, 'complete file set')
    for f in manifest['files']:
        p = root / f['path']
        need(p.resolve().is_relative_to(root.resolve()), 'manifest path escape')
        b = p.read_bytes()
        need(len(b) == f['bytes'] and blake3.blake3(b).hexdigest() == f['blake3'], 'file seal '+f['path'])
    report = read('report.json'); g = read('coupled-gradient-receipt.json'); margin = read('protected-margin-receipt.json'); journal = read('coupled-construction.json')
    need(report['status'] == 'COMPLETED' and journal['selected']['status'] == 'unchanged', 'completed unchanged source')
    need(report['new_backward_calls'] == 411 and report['new_training_graph_forwards'] == 822, 'measured source scope')
    need(sha(root/'protected-margin-receipt.json') == g['protected_margin_receipt_sha256'] == journal['protected_margin_receipt_sha256'], 'margin chain')
    def raw(entry, n):
        p = root / entry['file']; b = p.read_bytes()
        need(sha(p) == entry['sha256'] and len(b) == n*4, 'raw identity')
        return vector(list(struct.unpack('<'+'f'*n, b)), n)
    files = {(x['family'], x['kind']): x for x in g['files']}
    masters=[]; gradient=[]
    for family in ['prefix.coefficients', 'generate.unary']:
        masters += raw(files[(family, 'initial-master')], 960)
        gradient += raw(files[(family, 'gradient')], 960)
    direction = vector(journal['projection']['direction'])
    terms = margin['terms']; need(len(terms) == 380, '380 rows')
    jacobians = [raw(t, DIM) for t in terms]
    norms = [math.sqrt(dot(j,j)) for j in jacobians]
    units = [[x/n for x in j] if n else [0.0]*DIM for j,n in zip(jacobians,norms)]
    need(journal['projection']['final_residuals'] == [dot(j,direction) for j in units], 'continuous saved residuals')
    records=[]; all_coordinates=[]; all_guards=[]; contribution_rows=[]
    need([t['radius'] for t in journal['joint_vectors']] == [1,2,4,7], 'four original radii')
    for trial in journal['joint_vectors']:
        radius=trial['radius']; eta=trial['eta']
        need(eta == radius*0.25/max(abs(x) for x in direction), 'eta policy')
        coords=[decompose(m, eta, d) for m,d in zip(masters,direction)]
        delta=vector(trial['actual_delta'])
        need([c['destination_bits'] for c in coords] == trial['destination_master_bits'], 'all destination bits')
        need([c['actual_delta'] for c in coords] == delta, 'all actual displacements')
        stage_vectors={name:[c['stages'][name] for c in coords] for name in STAGES}
        intended=stage_vectors['intended']; error=[x-y for x,y in zip(delta,intended)]
        tolerance=1e-10*math.sqrt(dot(delta,delta)); residuals=[dot(j,delta) for j in units]
        need(residuals == trial['quantized_margin_residuals'] and tolerance == trial['quantized_tolerance'], 'saved quantized residuals')
        need(dot(gradient,delta) == trial['actual_CE_linear_delta'], 'objective dot')
        violated=[i for i,x in enumerate(residuals) if x < -tolerance]
        pressure=[0.0]*DIM; relief=[0.0]*DIM; stage_adverse={n:0.0 for n in STAGES}; stage_helpful={n:0.0 for n in STAGES}
        rows=[]
        for i,(j,u,t) in enumerate(zip(jacobians,units,terms)):
            raw_delta=dot(j,delta); original_margin=math.log(t['winner_mass']/t['rival_mass'])
            parts={n:dot(j,stage_vectors[n]) for n in STAGES}
            family_parts={family:{n:dot(j[lo:hi],stage_vectors[n][lo:hi]) for n in STAGES} for family,lo,hi in [('Prefix',0,960),('Generate',960,DIM)]}
            row={'guard':i,'source_term':{k:v for k,v in t.items() if k not in ['file','sha256','utility_file','utility_sha256','original_masses_file','original_masses_sha256']},'violated':i in violated,'unit_residual':residuals[i],'tolerance':tolerance,'raw_J_norm':norms[i],'raw_J_delta':raw_delta,'raw_J_intended':parts['intended'],'raw_stage_contributions':parts,'raw_family_contributions':family_parts,'dot_closure':raw_delta-sum(parts.values()),'original_margin_ln_mass_ratio':original_margin,'linearized_margin':original_margin+raw_delta,'original_token_tie':t['winner_mass']==t['rival_mass'],'linear_diagnostic_only':True}
            rows.append(row)
            if i in violated:
                for k,(uj,e) in enumerate(zip(u,error)):
                    v=uj*e;pressure[k]+=max(0.0,-v);relief[k]+=max(0.0,v)
                for n in STAGES:
                    v=dot(u,stage_vectors[n]);stage_adverse[n]+=max(0.0,-v);stage_helpful[n]+=max(0.0,v)
                signed=[j[k]*error[k] for k in range(DIM)]
                order=sorted(range(DIM),key=lambda k:(signed[k],k))
                contribution_rows.append({'radius':radius,'guard':i,'scope':'raw J times actual-minus-intended; attribution only, no coordinate removal','most_negative':[{'coordinate':k,'family':'Prefix' if k<960 else 'Generate','contribution':signed[k]} for k in order[:8]],'most_positive':[{'coordinate':k,'family':'Prefix' if k<960 else 'Generate','contribution':signed[k]} for k in order[-8:][::-1]]})
        total=sum(pressure);order=sorted(range(DIM),key=lambda k:(-pressure[k],k))
        summary={'radius':radius,'eta':eta,'violated_rows':len(violated),'violated_guard_indices':violated,'objective_dot_actual':dot(gradient,delta),'objective_dot_intended':dot(gradient,intended),'native_proposal_scope':trial['native'],'component_adverse_unit_row_dots_over_violated':stage_adverse,'component_helpful_unit_row_dots_over_violated':stage_helpful,'coordinate_pressure_scope':'sum max(0,-unitJ*(actual-intended)) over violated rows; descriptive concentration, not objective or candidate search','coordinate_adverse_total':total,'coordinate_relief_total':sum(relief),'top_pressure_share':{str(n):sum(pressure[k] for k in order[:n])/total if total else 0 for n in [1,10,50,100,500]},'family_adverse':{'Prefix':sum(pressure[:960]),'Generate':sum(pressure[960:])},'top_pressure_coordinates':[{'coordinate':k,'adverse':pressure[k],'relief':relief[k]} for k in order[:20]],'coordinate_counts':{family:{'clamped':sum(c['clamped'] for c in cs),'unchanged_code':sum(c['unchanged_code'] for c in cs),'changed_code':sum(not c['unchanged_code'] for c in cs),'nonzero_preservation':sum(c['stages']['preserved_fractional']!=0 for c in cs),'exact_half_ties':sum(c['exact_half_tie'] for c in cs)} for family,cs in [('Prefix',coords[:960]),('Generate',coords[960:])]},'max_coordinate_closure':max(abs(c['closure']) for c in coords),'max_dot_closure':max(abs(r['dot_closure']) for r in rows),'violated_original_margin_zero':sum(rows[i]['original_token_tie'] for i in violated),'violated_linearized_margin_positive':sum(rows[i]['linearized_margin']>0 for i in violated),'violated_linearized_margin_nonpositive':sum(rows[i]['linearized_margin']<=0 for i in violated),'minimum_linearized_margin':min(r['linearized_margin'] for r in rows)}
        records.append(summary);all_coordinates.append({'radius':radius,'coordinates':coords});all_guards.append({'radius':radius,'guards':rows})
    result={'schema':'uor-r4.saved-quantized-protection-attribution/1','status':'PASS_SAVED_RECONSTRUCTION','report_sha256':REPORT,'manifest_sha256':MANIFEST,'complete_source_files_verified':len(names),'analysis_sha256':sha(Path(__file__)),'scope':'four measured saved displacements,1920coordinates,380rawJacobian rows; no model/backward/candidate/native scoring or changedpolicy','coordinate_bits_verified':4*DIM,'trials':records,'all_radius_violated_guard_intersection':sorted(set.intersection(*(set(t['violated_guard_indices']) for t in records))),'seconds':time.monotonic()-start}
    for name,data in [('result',result),('coordinates',all_coordinates),('guards',all_guards),('top-contributions',contribution_rows)]:
        with (out/(name+'.json')).open('x') as f:json.dump(data,f,indent=2);f.write('\n')
    print(json.dumps({k:v for k,v in result.items() if k!='trials'}))
    for t in records:print(json.dumps({k:v for k,v in t.items() if k not in ['violated_guard_indices','top_pressure_coordinates']}))

if __name__ == '__main__':
    if sys.argv[1:] == ['--self-test']:self_test()
    else:
        need(len(sys.argv)==3, 'usage: analyze.py MODEL_ROOT NEW_OUTPUT')
        main(Path(sys.argv[1]),Path(sys.argv[2]))
