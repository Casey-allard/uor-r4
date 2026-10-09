#!/usr/bin/env python3
"""Saved credit comparison only: no model, encoder, backward or native pool replay.

The caller must first perform normative Rust complete-file/seal verification of
the three roots. SHA checks below authenticate used bytes, not BLAKE3 manifests.
Native integer masses are observations; this script recomputes their floating CE.
"""
import argparse
import hashlib
import json
import math
import struct
from pathlib import Path

BASE = Path('/workspace/uor-r4/codex')
OLD = BASE / 'sol-coupled-episode-learning/runs/coupled-episode-0001-attempt2'
EXPORTED = BASE / 'sol-coupled-episode-learning/runs/coupled-export-completion-0001-attempt3'
CANDIDATE = BASE / 'sol-full-donor-20261009/runs/full-donor-0001-attempt1'
OLD_PINS = {
    'report.json': 'd3a5b210d6627296e63aabf08ddb09fd16db45ca7cfad52ba819ce6a6d618cf8',
    'manifest.json': '11079e130773e8fd467fe10ec78c7f5205fec1157a87bd893eed086c9f17f9a8',
    'coupled-gradient-receipt.json': '285e2ca562387afc8a986a2507cb4efda04b37de3ff45bf31711e217cd89b4e5',
    'coupled-coordinate-order.json': 'd35d1b00bd653eff90e2efa2b334052a0b3b1ca05af6c93948caf2dfc4f25eb6',
    'coupled-forward-parity.json': '0fa8c8fd799067d21e29f4456947db1b2b889a948e4b0b6a7ece32d5637e39f4',
}
EXPORT_PINS = {
    'report.json': '0aa9b3c0c2d1defaaa6d93611a78fb348be0016aefe5d7548563250b9a687b8c',
    'manifest.json': 'e7667312b7d18b502e72b0fc41bf2d7d52a4977a020501ecd527da6ef628f9aa',
}
OLD_CONFIG_SHA = '4c33cf1bb35f5bc984f47a224d2c07ef1ba56393c7b2732ea48439baf56c2497'
FAMILIES = ('prefix.coefficients', 'generate.unary')
USED = []

def need(ok, message):
    if not ok:
        raise ValueError(message)

def raw(path, expected=None):
    data = Path(path).read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    need(expected is None or digest == expected, f'SHA identity: {path}')
    USED.append({'path': str(path), 'bytes': len(data), 'sha256': digest})
    return data

def read(path, expected=None):
    return json.loads(raw(path, expected))

def f32(x):
    return struct.unpack('<f', struct.pack('<f', x))[0]

def array(root, receipt):
    leaf = receipt['file']
    need(Path(leaf).name == leaf, 'raw gradient leaf')
    need(receipt['shape'] == [960] and receipt['bytes'] == 3840, 'raw960 shape/bytes')
    data = raw(root / leaf, receipt['sha256'])
    need(len(data) == 3840, 'raw960 length')
    values = list(struct.unpack('<960f', data))
    need(all(math.isfinite(x) for x in values), 'nonfinite raw960')
    return values, data

def credit(root, mode):
    receipt = read(root / 'coupled-gradient-receipt.json')
    need(receipt['physical_backward_calls'] == 31 and receipt['weighted_roles'] == 32,
         '31 physical/32 roles')
    need(receipt['extracted_families'] == list(FAMILIES), 'two active families')
    need(receipt.get('donor_credit', 'state_tangent') == mode, 'credit mode')
    terms = receipt['perterm']
    need(len(terms) == 31, '31 term receipts')
    vectors = {n: [] for n in FAMILIES}
    sums = {n: [0.0] * 960 for n in FAMILIES}
    for i, term in enumerate(terms):
        need(term['physical_index'] == i and math.isfinite(term['weight'])
             and term['weight'] > 0, 'physical term identity/weight')
        files = {x['family']: x for x in term['families']}
        need(set(files) == set(FAMILIES) and len(term['families']) == 2, 'raw family inventory')
        for name in FAMILIES:
            item = files[name]
            need(item['status'] == 'PRESENT' and item['missing_gradient_filled_zero'] is False,
                 'no missing-gradient fabrication')
            values, _ = array(root, item)
            need(item['all_zero'] == all(x == 0 for x in values), 'zero-gradient receipt')
            vectors[name].append(values)
            sums[name] = [f32(a + b) for a, b in zip(sums[name], values)]
    files = {(x['family'], x['kind']): x for x in receipt['files']}
    need(set(files) == {(n, k) for n in FAMILIES for k in ('gradient', 'initial-master')},
         'aggregate/master inventory')
    masters = {}
    for name in FAMILIES:
        aggregate, encoded = array(root, files[name, 'gradient'])
        need(struct.pack('<960f', *sums[name]) == encoded, 'ordered weighted f32 sum')
        masters[name], _ = array(root, files[name, 'initial-master'])
        sums[name] = aggregate
    return receipt, terms, vectors, sums, masters

def total_key(x):
    bits = struct.unpack('>Q', struct.pack('>d', x))[0]
    return (~bits & ((1 << 64) - 1)) if bits >> 63 else bits | (1 << 63)

def rank(gradients, masters):
    rows = []
    for name in FAMILIES:
        for i, (g, m) in enumerate(zip(gradients[name], masters[name])):
            need(abs(m) <= 1.75, 'original master Q4 range')
            q = int(math.copysign(math.floor(abs(m * 4) + 0.5), m))
            if name == FAMILIES[0]:
                d = -1 if g > 0 else 1 if g < 0 else 0
                destination = q + d if d and -7 <= q + d <= 7 else q
                delta = destination / 4 - m if destination != q else 0.0
            else:
                destination = min((x for x in range(-7, 8) if x != q),
                                  key=lambda x: (total_key(g * (x / 4 - m)), x))
                delta = destination / 4 - m
            rows.append((g * delta, name, i))
    return [(n, i) for _, n, i in sorted(rows, key=lambda x: (total_key(x[0]), x[1], x[2]))]

def check_rank(root, gradients, masters):
    saved = read(root / 'coupled-coordinate-order.json')
    need(isinstance(saved, list) and len(saved) == 1920, 'full frozen1920 ranking')
    observed = [(r['family'], r['index']) for r in saved]
    expected = rank(gradients, masters)
    need(observed == expected, 'saved ranking vs actual legal fractional displacement')
    return observed

def difference(old, new, atol, rtol):
    delta = [b - a for a, b in zip(old, new)]
    norm0 = math.sqrt(math.fsum(x*x for x in old))
    norm1 = math.sqrt(math.fsum(x*x for x in new))
    return {
        'bitwise_equal': struct.pack('<960f', *old) == struct.pack('<960f', *new),
        'within_declared_tolerance': all(abs(d) <= atol + rtol * max(abs(a), abs(b))
                                       for a, b, d in zip(old, new, delta)),
        'max_abs_difference': max(map(abs, delta)),
        'l2_difference': math.sqrt(math.fsum(d*d for d in delta)),
        'old_l2': norm0, 'new_l2': norm1,
        'cosine': math.fsum(a*b for a, b in zip(old, new)) / (norm0*norm1)
                  if norm0 and norm1 else None,
        'old_nonzero': sum(x != 0 for x in old), 'new_nonzero': sum(x != 0 for x in new),
        'strict_sign_reversals': sum(a*b < 0 for a, b in zip(old, new)),
    }

def utilities(root, receipt, terms):
    need(receipt['full_pool_utility_file'] == 'coupled-full-pool-donor-utilities.json', 'utility leaf')
    utility = read(root / receipt['full_pool_utility_file'], receipt['full_pool_utility_sha256'])
    need(utility['donor_credit'] == 'full_pool_utility' and utility['physical_frames'] == 31
         and utility['all_before_any_backward'] is True, 'utility prebackward scope')
    need(len(utility['records']) == 31, 'utility records31')
    results = []
    for term, frame in zip(terms, utility['records']):
        for k in ('input_index', 'position', 'target'):
            need(term[k] == frame['target_label_only' if k == 'target' else k], 'utility term identity')
        need(term['weight'] == frame['weight_applied_by_outer_loss_once'], 'utility already-weighted identity')
        donors = frame['donors']
        need(len(donors) == frame['physical_copy_aliases'] and donors, 'physical donor inventory')
        losses = []
        for j, donor in enumerate(donors):
            need(donor['physical_ordinal'] == j, 'physical donor order')
            mass, total = donor['target_mass'], donor['total_mass']
            need(type(mass) is int and type(total) is int and 0 < mass <= total, 'native integer mass')
            ce = -math.log(mass / total)
            need(abs(ce - donor['native_ce_f64']) <= 1e-12, 'saved mass->CE tolerance')
            losses.append(ce)
        factual = frame['factual_physical_donor']
        need(type(factual) is int and 0 <= factual < len(donors), 'factual donor ordinal')
        contrasts = [f32(ce - losses[factual]) for ce in losses]
        groups = {}
        for donor, contrast in zip(donors, contrasts):
            saved = donor['adjoint_factual_centered_contrast_f32']
            need(abs(contrast - saved) <= 1e-7 + 1e-6 * abs(saved), 'centered contrast conversion tolerance')
            key = tuple(donor['post_state'])
            need(len(key) == 8 and all(type(x) is int and 0 <= x < 120 for x in key), 'poststate8')
            identity = (donor['base_generate_sha256'], donor['target_mass'], donor['total_mass'], donor['chosen'])
            need(key not in groups or groups[key] == identity, 'equal-post utility identity')
            groups[key] = identity
        results.append({'input_index': term['input_index'], 'position': term['position'], 'id':frame['id'],
                        'target': term['target'], 'weight': term['weight'],
                        'actual_prefix_ids': frame['actual_prefix_ids'], 'factual_donor': factual,
                        'physical_donors': len(donors), 'distinct_posts': len(groups),
                        'lower_CE_donor_ordinals': [i for i, x in enumerate(contrasts) if x < 0],
                        'centered_contrasts_from_masses_f32': contrasts,
                        'factual_native_ce_from_masses': losses[factual], 'donors': donors})
    return results

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--candidate', type=Path, default=CANDIDATE)
    parser.add_argument('--config', type=Path, required=True)
    parser.add_argument('--config-sha256', required=True)
    parser.add_argument('--candidate-report-sha256', required=True)
    parser.add_argument('--candidate-manifest-sha256', required=True)
    parser.add_argument('--expected-source', required=True)
    parser.add_argument('--gradient-atol', type=float, default=2e-7)
    parser.add_argument('--gradient-rtol', type=float, default=2e-5)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    need(args.gradient_atol >= 0 and args.gradient_rtol >= 0
         and math.isfinite(args.gradient_atol + args.gradient_rtol), 'finite nonnegative tolerances')
    for leaf, digest in OLD_PINS.items(): raw(OLD / leaf, digest)
    for leaf, digest in EXPORT_PINS.items(): raw(EXPORTED / leaf, digest)
    old_config_path = OLD.parent.parent / 'configs/coupled-episode-0001-attempt2.json'
    old_config = read(old_config_path, OLD_CONFIG_SHA)
    need(read(OLD / 'config.json') == old_config, 'legacy parsed configuration identity')
    config = read(args.config, args.config_sha256)
    need(read(args.candidate / 'config.json') == config, 'candidate parsed configuration identity')
    need(config['coupled_episode_learning']['donor_credit'] == 'full_pool_utility', 'explicit corrected mode')
    need(config['coupled_episode_learning']['original_inputs']
         == old_config['coupled_episode_learning']['original_inputs'], 'same original input authority')
    report = read(args.candidate / 'report.json', args.candidate_report_sha256)
    raw(args.candidate / 'manifest.json', args.candidate_manifest_sha256)
    need(report['status'] == 'COMPLETED' and report['source_commit'] == args.expected_source,
         'candidate completed/source authority')
    old_receipt, old_terms, old_vectors, old_sums, old_masters = credit(OLD, 'state_tangent')
    receipt, terms, vectors, sums, masters = credit(args.candidate, 'full_pool_utility')
    need([{k: t[k] for k in ('physical_index','input_index','position','target','weight')} for t in terms]
         == [{k: t[k] for k in ('physical_index','input_index','position','target','weight')} for t in old_terms],
         'exact31 frame/target/coalesced-weight identity')
    for n in FAMILIES:
        need(struct.pack('<960f', *masters[n]) == struct.pack('<960f', *old_masters[n]), 'original fractional master bits')
    old_order = check_rank(OLD, old_sums, old_masters)
    new_order = check_rank(args.candidate, sums, masters)
    donor_rows = utilities(args.candidate, receipt, terms)
    old_population = read(OLD / 'coupled-population.json')
    population = read(args.candidate / 'coupled-population.json')
    need(len(population['objective_row_map']) == len(old_population['objective_row_map']) == 31,
         'population objective31 map')
    for i, term in enumerate(terms):
        old_frame = old_population['rows'][old_population['objective_row_map'][i]]
        frame = population['rows'][population['objective_row_map'][i]]
        for key in ('input_index', 'position', 'id', 'actual_prefix_ids', 'target_label_only'):
            need(frame[key] == old_frame[key], 'exact original-frame identity/prefix')
        need(frame['input_index'] == term['input_index'] and frame['position'] == term['position']
             and frame['target_label_only'] == term['target'], 'term/population binding')
        need(donor_rows[i]['actual_prefix_ids'] == frame['actual_prefix_ids']
             and donor_rows[i]['id'] == frame['id'], 'utility frame identity')
    perframe = []
    for i, term in enumerate(terms):
        perframe.append({'physical_index':i, 'input_index':term['input_index'], 'position':term['position'],
                         'target':term['target'], 'weight':term['weight'],
                         'families':{n:difference(old_vectors[n][i],vectors[n][i],args.gradient_atol,args.gradient_rtol)
                                     for n in FAMILIES}, 'donor_utility': donor_rows[i]})
    old_positions = {key:i for i,key in enumerate(old_order)}
    displacements = [abs(i-old_positions[key]) for i,key in enumerate(new_order)]
    exported = read(EXPORTED / 'report.json')
    result = {'schema':'uor-r4.saved-full-donor-credit-comparison/1', 'status':'PASS_SAVED_COMPARISON',
              'scope':'Authenticated saved masses/gradients/ranks only; no backward, encoder or native reconstruction; normative full-file Rust verification is caller prerequisite.',
              'candidate_source':args.expected_source, 'candidate_root':str(args.candidate),
              'legacy_gradient_root':str(OLD), 'legacy_export_root':str(EXPORTED),
              'tolerances':{'gradient_atol':args.gradient_atol,'gradient_rtol':args.gradient_rtol,
                            'mass_CE_absolute':1e-12,'centered_f32_absolute':1e-7,'centered_f32_relative':1e-6,
                            'meaning':'Tolerance equivalence is not bitwise gradient replication; CUDA device differences are not causally attributed.'},
              'aggregate':{n:difference(old_sums[n],sums[n],args.gradient_atol,args.gradient_rtol) for n in FAMILIES},
              'ranking':{'full1920_same_order':old_order==new_order,
                         'mean_absolute_rank_displacement':math.fsum(displacements)/1920,
                         'maximum_absolute_rank_displacement':max(displacements),
                         'top_overlap':{str(k):len(set(old_order[:k])&set(new_order[:k])) for k in (32,64,128,256,960)},
                         'interpretation':'Full1920 coordinate-set overlap is tautological; only order/prefix overlap changes are informative.'},
              'perframe':perframe,
              'candidate_outcome':{'objective':report['candidate_objective'],'final_gate':report['final_gate'],
                                   'selected_model':report['selected_model']},
              'legacy_export_outcome':{'objective':exported['candidate_objective'],'final_gate':exported['final_gate'],
                                       'selected_model':exported['selected_model']},
              'limits':['Native alternative losses are recomputed from saved integer masses, not regenerated.',
                        'No legacy full-pool donor utility was saved; old tangent-versus-finite utility residual is UNAVAILABLE here.',
                        'Gradient/ranking differences establish changed credit, not cause of outcome or language gain.',
                        'Equal-post checks authenticate saved identities; no recurrence/semantic occurrence proof.'],
              'used_files':USED}
    with args.output.open('x') as handle:
        json.dump(result,handle,indent=2,allow_nan=False);handle.write('\n')

if __name__ == '__main__':
    main()
