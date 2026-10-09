#!/usr/bin/env python3
"""Check saved vector projections/decisions and compare retained full-donor credit.

No model, encoder, score regeneration, native reducer, gradient or proposal runs.
Caller must normatively verify candidate and baseline complete sealed roots before
invocation and Rust-claim --output. This checker authenticates each used byte.
"""
import argparse
import hashlib
import json
import math
import resource
import struct
import sys
import time
from pathlib import Path

SOURCE = 'cc079744f4824e91a6ad739e2e7cf45c8d374527'
BASELINE_SOURCE = '4f7eee35b250b6d5bf9e250fc0ea00d356998695'
BASELINE_PINS = {'report.json': '56faa224da29b5da1db6fe82e21c4d19b6474be1316fef7fcb4716013cdd01a8',
                 'manifest.json': '9c19b82392b9e8f2263257992bf321ef5ed939e3b6e7788b783aa21328e45726'}
SOURCE_FILES = {'coupled_episode_learning.rs': '3f0daf51b15e6567f9e4195e61d470cf2572f195eb39eddf7db28e68ffdefe81',
                'gradient_vector_prefix.rs': '9871e03c64321a0e74d102e53e8419078b29ef25aa7be0f76ec1ffa08be599c1',
                'generate_episode_learning.rs': '30ac874b799a41cbfc57893944ac82cf7ac0c4a2681d2d95747da08017b2c769'}
FAMILIES = ('prefix.coefficients', 'generate.unary')
RADII = (1, 2, 4, 7)
USED = []


def need(ok, message):
    if not ok:
        raise ValueError(message)


def raw(path, expected=None):
    data = path.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    need(expected is None or digest == expected, f'input SHA mismatch: {path}')
    USED.append({'path': str(path), 'sha256': digest, 'bytes': len(data)})
    return data


def read(path, expected=None):
    return json.loads(raw(path, expected))


def f32(x):
    return struct.unpack('<f', struct.pack('<f', x))[0]


def bits(x):
    return struct.unpack('<I', struct.pack('<f', x))[0]


def total_key(x):
    b = struct.unpack('<Q', struct.pack('<d', x))[0]
    return (~b & ((1 << 64) - 1)) if b >> 63 else b | (1 << 63)


def code(x):
    need(math.isfinite(x) and -1.75 <= x <= 1.75, 'finite admitted fractional master')
    y = f32(4.0 * x)
    return int(math.copysign(math.floor(abs(y) + .5), y))


def vector_file(root, item):
    leaf = item['file']
    need(Path(leaf).name == leaf and item['shape'] == [960] and item['bytes'] == 3840,
         'raw960 filename/shape/bytes')
    data = raw(root / leaf, item['sha256'])
    need(len(data) == 3840, 'raw960 actual bytes')
    values = list(struct.unpack('<960f', data))
    need(all(map(math.isfinite, values)), 'nonfinite raw960')
    return values, data


def credit(root, transaction):
    receipt = read(root / 'coupled-gradient-receipt.json')
    need(receipt['physical_backward_calls'] == 31 and receipt['weighted_roles'] == 32
         and receipt['extracted_families'] == list(FAMILIES)
         and receipt['donor_credit'] == 'full_pool_utility'
         and receipt.get('prefix_transaction', 'coordinate_adjacent') == transaction,
         'fresh original31/full-pool/family/transaction receipt')
    terms = receipt['perterm']
    need(len(terms) == 31, '31 actual terms')
    arrays, encoded = {}, {}
    sums = {family: [0.] * 960 for family in FAMILIES}
    for i, term in enumerate(terms):
        need(term['physical_index'] == i and math.isfinite(term['weight']) and term['weight'] > 0,
             'physical term order/weight')
        family_files = {f['family']: f for f in term['families']}
        need(len(term['families']) == 2 and set(family_files) == set(FAMILIES), 'raw term families')
        for family, item in family_files.items():
            need(item['status'] == 'PRESENT' and item['missing_gradient_filled_zero'] is False,
                 'actual gradient present')
            values, data = vector_file(root, item)
            need(item['all_zero'] == all(x == 0. for x in values), 'saved zero classification')
            arrays[family, i], encoded[family, i] = values, data
            sums[family] = [f32(a + b) for a, b in zip(sums[family], values)]
    files = {(f['family'], f['kind']): f for f in receipt['files']}
    need(len(receipt['files']) == 4 and set(files) == {(f, k) for f in FAMILIES
         for k in ('gradient', 'initial-master')}, 'aggregate/master inventory')
    masters = {}
    for family in FAMILIES:
        aggregate, data = vector_file(root, files[family, 'gradient'])
        need(struct.pack('<960f', *sums[family]) == data, 'ordered weighted31 f32 aggregate')
        arrays[family, 'gradient'], encoded[family, 'gradient'] = aggregate, data
        masters[family], encoded[family, 'initial-master'] = vector_file(root, files[family, 'initial-master'])
    return receipt, terms, arrays, encoded, masters


def ranking(gradients, masters):
    out = []
    for family in FAMILIES:
        for i, (g, m) in enumerate(zip(gradients[family, 'gradient'], masters[family])):
            q = code(m)
            if family == FAMILIES[0]:
                direction = -1 if g > 0 else 1 if g < 0 else 0
                destination = q + direction
                status = 'zero_gradient' if not direction else 'saturated' if not -7 <= destination <= 7 else 'eligible'
                if status != 'eligible':
                    destination = q
                delta = destination * .25 - m if status == 'eligible' else 0.
            else:
                destination = min((c for c in range(-7, 8) if c != q),
                                  key=lambda c: (total_key(g * (c * .25 - m)), c))
                status, delta = 'all14', destination * .25 - m
            out.append({'family': family, 'index': i, 'gradient': g, 'original_master': m,
                        'rank_code': destination, 'original_code': q, 'actual_master_delta': delta,
                        'priority': g * delta, 'status': status})
    return sorted(out, key=lambda r: (total_key(r['priority']), r['family'], r['index']))


def projected(masters, gradient, radius):
    maxabs = max(abs(g) for g in gradient)
    eta = radius * .25 / maxabs if maxabs else 0.
    destination, changed, linear = [], [], 0.
    for i, (m, g) in enumerate(zip(masters, gradient)):
        value = m
        if maxabs:
            x = f32(max(-1.75, min(1.75, m - eta * g)))
            q = code(x)
            if q != code(m):
                value = f32(q * .25)
        if bits(value) != bits(m):
            changed.append(i)
        destination.append(value)
        linear += g * (value - m)
    status = ('zero_gradient' if not maxabs else 'unchanged_native_codes' if not changed
              else 'eligible' if linear < 0 else 'nonnegative_actual_linear_delta')
    return {'radius': radius, 'max_abs_gradient': maxabs, 'eta': eta, 'actual_linear_delta': linear,
            'status': status, 'changed_indices': changed, 'destination_master_bits': [bits(x) for x in destination]}, destination


def objective(value, population):
    masses = value['objective_masses']
    mapping, rows = population['objective_row_map'], population['rows']
    need(len(mapping) == len(masses) == 31 and len(set(mapping)) == 31, '31 unique objective masses')
    lookup = {(rows[r]['input_index'], rows[r]['position']): i for i, r in enumerate(mapping)}
    task, reference, correct, phases = 0., 0., 0, []
    for role in population['roles']:
        slot = lookup[role['input'], role['position']]
        mass, total, chosen = masses[slot]
        target = rows[mapping[slot]]['target_label_only']
        need(type(mass) is int and type(total) is int and 0 < mass <= total
             and type(chosen) is int, 'integer native target mass/denominator/winner')
        ce = -math.log(mass / total) * role['weight']
        if role['task']:
            task += ce
            phases.append({'input_index': role['input'], 'position': role['position'], 'target': target, 'chosen': chosen})
        else:
            reference += ce
            correct += chosen == target
    need(len(phases) == 15 and len(population['roles']) == 32, '15 task/17 reference roles')
    for key, expected in [('task', task), ('reference', reference), ('combined', task + reference)]:
        need(math.isfinite(value[key]) and abs(value[key] - expected) <= 1e-12 * (1 + abs(expected)),
             'saved role CE vs native mass arithmetic')
    need(value['correct_reference_frames'] == correct and
         value['all_phase_winners'] == all(p['target'] == p['chosen'] for p in phases), 'saved winner-role count')
    return phases


def gate(before, after):
    return (after['combined'] < before['combined'] - 1e-10 * (1 + abs(before['combined']))
            and after['correct_reference_frames'] == 17)


def traversal(stage, targets, expected=None):
    affected, checked = stage['affected_guard_indices'], stage['checked_guard_indices']
    need(affected == sorted(set(affected)) and all(type(i) is int and 0 <= i < 380 for i in affected), 'affected guard domain')
    if expected is not None:
        need(affected == expected, 'declared changed-key affected union')
    need(checked == affected[:len(checked)], 'ascending first-veto traversal')
    eligible, failure = stage['strict_current_CE_and17'], stage['first_failure']
    if not eligible:
        need(not checked and failure is None and stage['guard_status'] == 'NOT_CHECKED_OBJECTIVE_GATE_FALSE', 'unmeasured guard scope')
    elif failure is not None:
        need(checked and failure['guard_index'] == checked[-1] and failure['required'] == targets[checked[-1]]
             and failure['chosen'] != failure['required'] and stage['guard_status'] == 'FIRST_VETO', 'first original guard veto')
    else:
        need(checked == affected and stage['guard_status'] == 'FULL_PASS', 'complete affected guard pass')
    need(stage['feasible'] == (eligible and failure is None), 'feasible gate composition')


def compact(value):
    return {k: value[k] for k in ('combined', 'task', 'reference', 'correct_reference_frames', 'all_phase_winners', 'objective_masses')}


def constructor(root, population, arrays, masters, report):
    ranked = read(root / 'coupled-coordinate-order.json')
    expected_rank = ranking(arrays, masters)
    need(len(ranked) == len(expected_rank) == 1920, 'full1920 rank authority')
    for saved, expected in zip(ranked, expected_rank):
        need(all(saved[k] == expected[k] for k in ('family', 'index', 'rank_code', 'original_code', 'status'))
             and bits(saved['gradient']) == bits(expected['gradient'])
             and bits(saved['original_master']) == bits(expected['original_master'])
             and total_key(saved['priority']) == total_key(expected['priority'])
             and total_key(saved['actual_master_delta']) == total_key(expected['actual_master_delta']),
             'frozen1920 order/actual fractional bits and displacement')
    journal = read(root / 'coupled-construction.json')
    need(journal['schema'] == 'uor-r4.gradient-vector-prefix-construction/1' and journal['policy'] == report['policy']
         and journal['policy']['prefix_transaction'] == 'gradient_vector_prefix'
         and journal['policy']['maximum_alternatives'] == 13444, 'vector schema/policy binding')
    summary, vector = journal['summary'], journal['prefix_vector']
    need(summary['coordinates'] == 960 and summary['maximum_alternatives'] == 13444 and summary['revisited'] == 0
         and vector['radii'] == list(RADII) and vector['all_radius_incumbent_epochs'] == 0
         and len(vector['alternatives']) == 4, 'finite fixed vector/G population')
    objective(summary['initial'], population)
    need(compact(summary['initial']) == compact(report['baseline_objective']), 'original baseline/report identity')
    targets = [r['target_label_only'] for r in population['rows']]
    unions = [r['prefix_key_union'] for r in population['rows']]
    need(len(unions) == 391 and all(k == sorted(set(k)) and k and all(type(x) is int and 0 <= x < 960 for x in k) for k in unions), '391 sorted key unions')
    best, evaluated, reductions = None, 0, 0
    vector_results = []
    for radius, alternative in zip(RADII, vector['alternatives']):
        projection, destination = projected(masters[FAMILIES[0]], arrays[FAMILIES[0], 'gradient'], radius)
        need(alternative['projection'] == projection, 'exact f64/f32 vector projection and fractional NOOP')
        stage = alternative['native_stage']
        if projection['status'] != 'eligible':
            need(stage == 'NOT_RUN_LINEAR_FILTER' and alternative['incumbent_epoch'] == 0, 'linear-filter NOOP')
            vector_results.append({'radius': radius, 'status': projection['status'], 'native': 'NOT_RUN_LINEAR_FILTER'})
            continue
        evaluated += 1
        need(stage['incumbent_epoch'] == 0 and stage['radius'] == radius, 'same original radius epoch')
        objective(stage['objective'], population)
        need(stage['strict_current_CE_and17'] == gate(summary['initial'], stage['objective']), 'vector strict current CE/reference gate')
        affected = [i for i, keys in enumerate(unions) if set(keys).intersection(projection['changed_indices'])]
        need(stage['affected_rows'] == affected, 'union includes cancellation rows')
        traversal(stage, targets, [i for i in affected if i < 380])
        changed_rows = stage['changed_rows']
        staged = sorted(set(population['objective_row_map']).intersection(affected) | set(stage['checked_guard_indices']))
        need([r[0] for r in changed_rows] == staged and stage['staged_rows_count'] == len(staged), 'actually staged objective/visited guard rows')
        for row, old_donor, donor, post, rebuilt, digest in changed_rows:
            need(old_donor == population['rows'][row]['donor'] and type(donor) is int and donor >= 0
                 and len(post) == 8 and all(type(x) is int and 0 <= x < 120 for x in post)
                 and type(rebuilt) is bool and len(digest) == 64, 'saved physical donor/post/incidence identity')
        reductions += stage['staged_rows_count']
        vector_results.append({'radius': radius, 'actual_linear_delta': projection['actual_linear_delta'],
                               'changed_coordinates': len(projection['changed_indices']), 'objective': stage['objective'],
                               'guard_status': stage['guard_status'], 'first_failure': stage['first_failure'],
                               'feasible': stage['feasible'], 'staged_rows': len(staged),
                               'donor_changes': sum(r[1] != r[2] for r in changed_rows),
                               'post_incidence_rebuilds': sum(r[4] for r in changed_rows)})
        if stage['feasible'] and (best is None or (stage['objective']['combined'], radius) < best[:2]):
            best = (stage['objective']['combined'], radius, stage, destination)
    selected = vector['selected']
    prefix_commit = int(best is not None)
    need(vector['proposal_stage_pool_reductions'] == reductions and vector['selected_restage_count'] == prefix_commit
         and summary['selected_restage_count'] == prefix_commit and summary['accepted_prefix'] == prefix_commit, 'stage/restage/Prefix commit counts')
    if best:
        need(selected['status'] == 'committed' and selected['incumbent_epoch'] == 0 and selected['epoch_after'] == 1
             and selected['radius'] == best[1] and selected['selected_restage_exact'] is True
             and selected['selected_receipt'] == best[2]
             and vector['selected_restage_pool_reductions'] == best[2]['staged_rows_count'], 'minimum feasible radius and exact retained restage receipt')
        current, final_prefix = best[2]['objective'], best[3]
    else:
        need(selected == {'status': 'unchanged', 'incumbent_epoch': 0, 'epoch_after': 0}
             and vector['selected_restage_pool_reductions'] == 0, 'no feasible vector means unchanged original')
        current, final_prefix = compact(summary['initial']), masters[FAMILIES[0]][:]
    epoch, accepted_g = prefix_commit, 0
    final_generate = masters[FAMILIES[1]][:]
    records, g_order = journal['coordinate_records'], [r for r in ranked if r['family'] == FAMILIES[1]]
    need(len(records) == len(g_order) == 960, 'Generate complete relative order')
    gate_counts = {'NOT_CHECKED_OBJECTIVE_GATE_FALSE': 0, 'FIRST_VETO': 0, 'FULL_PASS': 0}
    for ordinal, (record, rank) in enumerate(zip(records, g_order)):
        index = rank['index']
        need(record['order'] == ordinal and record['family'] == FAMILIES[1] and record['index'] == index
             and record['incumbent_epoch'] == epoch and record['original_master_bits'] == bits(rank['original_master'])
             and record['gradient_bits'] == bits(rank['gradient']) and record['priority_bits'] == struct.unpack('<Q', struct.pack('<d', rank['priority']))[0]
             and record['rank_code'] == rank['rank_code'] and record['rank_status'] == rank['status'], 'frozen G sequence/raw rank/epoch')
        alts = record['alternatives']
        need(len(alts) == 14 and [a['code'] for a in alts] == [q for q in range(-7, 8) if q != code(final_generate[index])], 'all14 legal codes once')
        passing = []
        for alt in alts:
            need(alt['incumbent_epoch'] == epoch and alt['incumbent_code'] == code(final_generate[index]), 'same G incumbent')
            need(alt['delta_code'] == alt['code'] - alt['incumbent_code']
                 and alt['actual_master_delta_from_original'] == alt['code'] * .25 - masters[FAMILIES[1]][index],
                 'G native code vs actual original fractional displacement')
            objective(alt['objective'], population)
            need(alt['strict_current_CE_and17'] == gate(current, alt['objective']), 'G strict CE/reference gate')
            traversal(alt, targets)
            gate_counts[alt['guard_status']] += 1
            if alt['feasible']:
                passing.append(alt)
        chosen = min(passing, key=lambda a: (a['objective']['combined'], a['code'])) if passing else None
        if chosen:
            need(record['selected'] == {'status': 'committed', 'code': chosen['code'], 'staged_summary_digest': chosen['staged_summary_digest']}, 'G minimum feasible selected identity')
            current = chosen['objective']
            final_generate[index] = f32(chosen['code'] * .25)
            epoch += 1
            accepted_g += 1
        else:
            need(record['selected'] == {'status': 'unchanged'}, 'no feasible G code means unchanged')
        need(record['epoch_after'] == epoch, 'G epoch advances only on commit')
        evaluated += len(alts)
    need(summary['accepted_generate'] == accepted_g and summary['accepted_epoch'] == epoch
         and summary['evaluated_alternatives'] == evaluated and compact(summary['final']) == current
         and compact(report['candidate_objective']) == current, 'terminal objective/counters/report chain')
    for leaf, values in [('prefix/prefix-source-f32.bin', final_prefix), ('generate-source/generate.unary.f32le', final_generate)]:
        need(raw(root / 'checkpoint-0001' / leaf) == struct.pack('<960f', *values), 'final committed actual fractional masters')
    guards = read(root / 'final-trajectory-guards.json')
    need(guards['all_original_winners'] is True and len(guards['terms']) == 380
         and all(r['chosen'] == r['required_original_winner'] for r in guards['terms']), 'recorded final380 original winners')
    before, final = summary['initial'], summary['final']
    improves = lambda a,b: b < a - 1e-10 * (1 + abs(a))
    gate_expected = {'strict_combined_descent': improves(before['combined'], final['combined']),
                     'strict_episode_descent': improves(before['task'], final['task']),
                     'all15_conditional_winners': final['all_phase_winners'],
                     'all17_references': final['correct_reference_frames'] == 17,
                     'all380_original_winners': True}
    gate_expected['passed'] = all(gate_expected.values())
    need(all(report['final_gate'][key] == value for key,value in gate_expected.items())
         and report['finite_episode_positive'] == gate_expected['passed'], 'final declared conditional gate')
    return {'four_vectors': vector_results, 'selected': selected, 'Generate_alternative_gate_counts': gate_counts,
            'summary': summary, 'new_native_score_replay': False, 'staged_digest_recomputed': False}


def correct(report):
    phases = report['candidate_objective']['phases']
    need(len(phases) == 15, '15 final conditional phases')
    return [p['position'] for p in phases if p['chosen'] == p['target']]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--candidate', type=Path, default=Path('/workspace/uor-r4/codex/sol-prefix-vector-20261009/runs/prefix-vector-0001-attempt1'))
    parser.add_argument('--baseline', type=Path, default=Path('/workspace/uor-r4/codex/sol-full-donor-20261009/runs/full-donor-0001-attempt1'))
    parser.add_argument('--config', type=Path, required=True)
    parser.add_argument('--config-sha256', required=True)
    parser.add_argument('--candidate-report-sha256', required=True)
    parser.add_argument('--candidate-manifest-sha256', required=True)
    parser.add_argument('--output', type=Path, required=True, help='fresh parent Rust-claimed report directory')
    args = parser.parse_args()
    start = time.monotonic()
    need(args.output.is_dir() and sorted(p.name for p in args.output.iterdir()) == ['attempt.json'], 'fresh exclusively claimed output')
    attempt = json.loads((args.output / 'attempt.json').read_text())
    need(attempt['schema'] == 'uor-r4.report-attempt/1', 'normative output claim sentinel')
    config = read(args.config, args.config_sha256)
    need(raw(args.candidate / 'config.json') == args.config.read_bytes(), 'candidate raw config exact bytes')
    learning = config['coupled_episode_learning']
    need(learning['donor_credit'] == 'full_pool_utility' and learning['prefix_transaction'] == 'gradient_vector_prefix'
         and learning.get('retained_gradient') is None and learning.get('retained_export') is None, 'fresh vector policy config')
    report = read(args.candidate / 'report.json', args.candidate_report_sha256)
    raw(args.candidate / 'manifest.json', args.candidate_manifest_sha256)
    baseline = read(args.baseline / 'report.json', BASELINE_PINS['report.json'])
    raw(args.baseline / 'manifest.json', BASELINE_PINS['manifest.json'])
    baseline_config = read(args.baseline / 'config.json')
    need(learning['original_inputs'] == baseline_config['coupled_episode_learning']['original_inputs'], 'same original typed input authority')
    for observed, source in [(report, SOURCE), (baseline, BASELINE_SOURCE)]:
        need(observed['status'] == 'COMPLETED' and observed['source_commit'] == source
             and observed['mode'] == 'coupled_episode_learning' and observed['selected_model'] is False
             and observed['candidate_native_steps'] == 391 and observed['all_original380_preserved'] is True, 'completed source/native391/unselected scope')
    receipt, terms, arrays, encoded, masters = credit(args.candidate, 'gradient_vector_prefix')
    old_receipt, old_terms, old_arrays, old_encoded, old_masters = credit(args.baseline, 'coordinate_adjacent')
    identity_keys = ('physical_index', 'input_index', 'position', 'target', 'weight')
    need([{k: t[k] for k in identity_keys} for t in terms] == [{k: t[k] for k in identity_keys} for t in old_terms], 'same31 physical weighted terms')
    for family in FAMILIES:
        need(encoded[family, 'initial-master'] == old_encoded[family, 'initial-master'], 'same original fractional master bits')
    population = read(args.candidate / 'coupled-population.json')
    old_population = read(args.baseline / 'coupled-population.json')
    need(population['guards'] == 380 and population['unique_union'] == 391
         and population['roles'] == old_population['roles']
         and population['objective_row_map'] == old_population['objective_row_map'], 'same32 roles/31 map/391 union')
    row_fields = ('row', 'input_index', 'position', 'id', 'actual_prefix_ids', 'target_label_only', 'donor', 'post_state')
    need([{k: r[k] for k in row_fields} for r in population['rows']] == [{k: r[k] for k in row_fields} for r in old_population['rows']], 'same391 original states and labels')
    result = constructor(args.candidate, population, arrays, masters, report)
    comparisons = {family: {'aggregate_bitwise_equal': encoded[family, 'gradient'] == old_encoded[family, 'gradient'],
                           'term_bitwise_equal': [encoded[family, i] == old_encoded[family, i] for i in range(31)],
                           'max_aggregate_absolute_difference': max(abs(a-b) for a,b in zip(arrays[family, 'gradient'], old_arrays[family, 'gradient']))} for family in FAMILIES}
    current_correct, old_correct = correct(report), correct(baseline)
    elapsed = time.monotonic() - start
    rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * (1 if sys.platform == 'darwin' else 1024)
    output = {'schema': 'uor-r4.saved-prefix-vector-comparison/1', 'status': 'PASS_SAVED_COMPARISON',
              'candidate_source': SOURCE, 'baseline_source': BASELINE_SOURCE, 'source_contract_sha256': SOURCE_FILES,
              'candidate_root': str(args.candidate), 'baseline_root': str(args.baseline), 'gradient_comparison': comparisons,
              'construction': result, 'conditional_outcomes': {'candidate_correct_positions': current_correct,
              'baseline_correct_positions': old_correct, 'gained': sorted(set(current_correct)-set(old_correct)),
              'lost': sorted(set(old_correct)-set(current_correct)), 'candidate_final_gate': report['final_gate'],
              'baseline_final_gate': baseline['final_gate']},
              'scope': {'normative_seal_verification': 'CALLER_PREREQUISITE', 'native_score_replication': 'NOT_RUN',
              'independent_backward_replication': 'NOT_RUN', '391_native_reload': 'PRODUCER_RECEIPT_ONLY',
              'selected_restage': 'EXACT_SAVED_RECEIPT_EQUALITY_ONLY', 'Generate_affected_incidence': 'SAVED_IDENTITIES_ONLY',
              'language_qualification': 'SEPARATE_ACTUAL9_REPORT', 'full512': 'NOT_RUN'},
              'used_files': USED, 'script_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'seconds': elapsed, 'peak_rss_bytes': rss}
    with (args.output / 'report.json').open('x') as stream:
        json.dump(output, stream, indent=2, allow_nan=False)
        stream.write('\n')
    with (args.output / 'execution.json').open('x') as stream:
        json.dump({'status': 'PASS', 'argv': sys.argv, 'seconds': elapsed, 'peak_rss_bytes': rss,
                   'script_sha256': output['script_sha256'], 'scope': 'saved-data arithmetic only; no model calls'}, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'status': output['status'], 'candidate_correct_positions': current_correct,
                      'baseline_correct_positions': old_correct, 'seconds': elapsed}), flush=True)


if __name__ == '__main__':
    main()
