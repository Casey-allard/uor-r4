#!/usr/bin/env python3
"""Saved proposal/gradient attribution only; no scores or proposals are evaluated.
The parent must claim the output with existing Rust report_output before invocation,
and seal/verify it after completion. This reader never fabricates a Rust seal.
"""
import argparse
import collections
import hashlib
import json
import math
import pathlib
import resource
import struct
import sys
import time


def need(value, message):
    if not value:
        raise ValueError(message)


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b''):
            h.update(chunk)
    return h.hexdigest()


def read(path):
    with path.open('r', encoding='utf-8') as stream:
        return json.load(stream)


def total_key(value):
    bits = struct.unpack('<Q', struct.pack('<d', value))[0]
    return (~bits & ((1 << 64) - 1)) if bits >> 63 else bits | (1 << 63)


def native_code(value):
    need(math.isfinite(value) and -1.75 <= value <= 1.75, 'original master Q4 range')
    x = value * 4
    return int(math.copysign(math.floor(abs(x) + 0.5), x))


def static_potential(gradient, master):
    q = native_code(master)
    choices = [(gradient * (code * 0.25 - master), code * 0.25 - master, code)
               for code in range(-7, 8) if code != q]
    return min(choices, key=lambda x: (total_key(x[0]), x[2]))


def ratio_gt(a, b):
    return a[0] * b[1] > b[0] * a[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', type=pathlib.Path, required=True)
    parser.add_argument('--identity', type=pathlib.Path, required=True)
    parser.add_argument('--evidence', type=pathlib.Path, required=True)
    parser.add_argument('--output', type=pathlib.Path, required=True)
    args = parser.parse_args()
    # Boundary supplied by parent; no input/model file is read before this check.
    need(args.output.is_dir() and (args.output / 'attempt.json').is_file(),
         'existing Rust-claimed exclusive output required')
    need(not (args.output / 'report.json').exists(), 'fresh result required')
    attempt = read(args.output / 'attempt.json')
    need(attempt['schema'] == 'uor-r4.report-attempt/1', 'normative claim schema')
    start = time.monotonic()
    identity = read(args.identity)
    need(identity['complete_member_inventory_verified'] is True and
         identity['files_count'] == len(identity['files']) == 546, 'recovered inventory scope')
    pins = {}

    def authenticated(path):
        rel = path.relative_to(args.evidence).as_posix()
        pin = identity['files'][rel]
        need(path.is_file() and path.stat().st_size == pin['bytes'] and sha(path) == pin['sha256'],
             'recovered byte receipt differs: ' + rel)
        pins[rel] = pin
        return path

    def value(leaf):
        return read(authenticated(args.run / leaf))

    def floats(leaf):
        data = authenticated(args.run / leaf).read_bytes()
        need(len(data) == 57600 * 4, 'full pair-family raw shape')
        x = struct.unpack('<57600f', data)
        need(all(math.isfinite(v) for v in x), 'finite raw family')
        return x

    report = value('report.json')
    population = value('generate-population.json')
    gradients = value('generate-gradient-receipt.json')
    journal = value('generate-construction.json')
    binding = value('generate-original-master-binding.json')
    order = value('generate-coordinate-order.json')
    audit = read(authenticated(args.evidence / 'audits/pair-0001-attempt1/result.json'))
    source = pathlib.Path(__file__).resolve().parents[3] / 'crates/uor-r4-training/examples/geometric_frozen_map_fit/generate_episode_learning.rs'
    need(sha(source) == audit['source_file_sha256'] and audit['status'] == 'PASS' and
         audit['report_sha256'] == identity['run_report_sha256'] == sha(args.run / 'report.json') and
         audit['manifest_sha256'] == identity['run_manifest_sha256'] == sha(args.run / 'manifest.json') and
         audit['source_commit'] == report['source_commit'] == identity['source_commit'],
         'prior independent audit/source binding')
    need(report['status'] == 'COMPLETED' and report['mode'] == 'generate_pair_episode_learning' and
         report['selected_model'] is False and report['policy']['coordinate_count'] == 960 and
         report['policy']['full_family_coordinate_count'] == 57600, 'bounded pair scope')
    rows = population['rows']; mapping = population['objective_row_map']
    focus = [row for row in rows if row['input_index'] == 245 and row['position'] == 4]
    need(len(focus) == 1 and focus[0]['target_label_only'] == 267, 'focus physical identity')
    focus = focus[0]; slot = mapping.index(focus['row'])
    need(slot == 4 and focus['row'] == 380, 'authenticated physical/objective mapping')
    reference_slots = []
    for role in population['roles']:
        if not role['task']:
            matched = [i for i, union in enumerate(mapping) if rows[union]['input_index'] == role['input'] and rows[union]['position'] == role['position']]
            need(len(matched) == 1, 'reference role mapping')
            reference_slots.append(matched[0])
    need(len(reference_slots) == 17 and len(mapping) == 31, '31 coalesced frames/17 unchanged references')
    records = journal['coordinate_records']
    need(len(records) == len(order) == 960 and journal['summary']['alternatives'] == 13440,
         'complete bounded alternative population')
    incumbent = journal['summary']['initial']['objective_masses'][slot]
    original = list(incumbent)
    ce = journal['summary']['initial']['combined']
    epoch = 0
    classes = collections.Counter(); gates = collections.Counter(); probability = collections.Counter()
    winner_values = collections.Counter(); best = None; crossings = []; winning_commits = []
    lost_commits = []; first_veto_counts = collections.Counter(); accepted = 0
    for ordinal, record in enumerate(records):
        need(record['order'] == ordinal and record['index'] == order[ordinal] and record['incumbent_epoch'] == epoch,
             'actual coordinate/epoch sequence')
        alts = record['alternatives']
        need(len(alts) == 14 and {a['code'] for a in alts} == set(range(-7, 8)) - {record['incumbent_code']}, 'all14 same-incumbent alternatives')
        selected = record['selected']
        for alt in alts:
            masses = alt['objective_masses']; triple = masses[slot]
            need(len(masses) == 31 and triple[0] > 0 and triple[1] > 0, 'saved mass tuple')
            refs = all(masses[i][2] == rows[mapping[i]]['target_label_only'] for i in reference_slots)
            descent = alt['combined'] < ce - 1e-10 * (1 + abs(ce))
            need(alt['objective_gate'] == (refs and descent), 'saved CE/reference gate classification')
            chosen = selected['status'] == 'committed' and selected['code'] == alt['code']
            if not alt['objective_gate']:
                category = ('CE_AND_REFERENCE_REJECTION' if not refs and not descent else
                            'REFERENCE_REJECTION' if not refs else 'CE_REJECTION')
            elif alt['first_guard_failure'] is not None:
                category = 'FIRST_ORIGINAL_GUARD_VETO'
                first_veto_counts[alt['first_guard_failure']['row']] += 1
            else:
                category = 'ACCEPTED' if chosen else 'FEASIBLE_NOT_SELECTED'
            need(alt['feasible'] == (alt['objective_gate'] and alt['first_guard_failure'] is None), 'saved feasibility flags')
            classes[category] += 1; gates[alt['guard_status']] += 1; winner_values[triple[2]] += 1
            relation = 'INCREASE' if ratio_gt(triple, incumbent) else 'DECREASE' if ratio_gt(incumbent, triple) else 'EQUAL'
            probability[relation] += 1
            observed = {'order': ordinal, 'index': record['index'], 'incumbent_epoch': epoch,
                        'code': alt['code'], 'classification': category,
                        'target_mass': triple[0], 'total_mass': triple[1], 'chosen': triple[2],
                        'incumbent_mass': incumbent[0], 'incumbent_total': incumbent[1],
                        'probability_relation_to_incumbent': relation,
                        'objective_gate': alt['objective_gate'], 'strict_ce_descent': descent,
                        'all17_reference_winners': refs, 'first_guard_failure': alt['first_guard_failure']}
            if best is None or ratio_gt(triple, [best['target_mass'], best['total_mass']]):
                best = observed
            if triple[2] == 267:
                crossings.append(observed)
        if selected['status'] == 'committed':
            selected_alt = next(a for a in alts if a['code'] == selected['code'])
            next_triple = selected_alt['objective_masses'][slot]
            if next_triple[2] == 267: winning_commits.append(ordinal)
            if incumbent[2] == 267 and next_triple[2] != 267: lost_commits.append(ordinal)
            incumbent = list(next_triple); ce = selected_alt['combined']; epoch += 1; accepted += 1
        else:
            need(selected['status'] == 'unchanged', 'closed noncommitted status')
        need(record['epoch_after'] == epoch, 'actual epoch after selection')
    need(accepted == journal['summary']['accepted_coordinates'] and
         incumbent == journal['summary']['final']['objective_masses'][slot], 'final saved mass/accepted consistency')
    masters = floats('generate-initial-masters.f32le')
    need(sha(args.run / 'generate-initial-masters.f32le') == binding['parameters']['generate.pair'], 'original actual master hash')
    aggregate = floats('generate-gradient.f32le')
    need(sha(args.run / 'generate-gradient.f32le') == gradients['aggregate']['sha256'], 'aggregate receipt SHA')
    terms = gradients['perterm']; need(len(terms) == 31, 'weighted physical terms')
    target_terms = [t for t in terms if t['input_index'] == 245 and t['position'] == 4 and t['target'] == 267]
    need(len(target_terms) == 1 and target_terms[0]['physical_index'] == slot and target_terms[0]['weight'] == 1/15,
         'already-weighted target raw term')
    target_gradient = floats(target_terms[0]['file'])
    sum_abs = [0.0] * 57600
    for i, term in enumerate(terms):
        need(term['physical_index'] == i and term['status'] == 'PRESENT' and not term['missing_gradient_filled_zero'], 'raw physical term presence/order')
        values = floats(term['file'])
        need(sha(args.run / term['file']) == term['sha256'], 'raw term receipt SHA')
        for k, v in enumerate(values): sum_abs[k] += abs(v)
    frozen = [static_potential(g, m) for g, m in zip(aggregate, masters)]
    ranking = sorted(range(57600), key=lambda i: (total_key(frozen[i][0]), i))
    need(ranking[:960] == order, 'declared original frozen ranking reconstruction')
    rank_pos = [0] * 57600
    for rank, index in enumerate(ranking): rank_pos[index] = rank
    selected_indices = set(order)
    metrics = {}
    for label, indices in [('selected960', order), ('remaining56640', ranking[960:])]:
        magnitude = 0.; norm2 = 0.; potential = 0.; nonzero = 0; aligned = 0; opposite = 0
        cancel_num = 0.; cancel_den = 0.; opposition = 0.; helped = 0
        support_ranks = []
        for i in indices:
            tg = target_gradient[i]; h, delta, _ = static_potential(tg, masters[i])
            magnitude += abs(tg); norm2 += tg*tg; potential += max(0., -h)
            if tg != 0:
                nonzero += 1; support_ranks.append(rank_pos[i]); cancel_num += abs(aggregate[i]); cancel_den += sum_abs[i]
                if tg * frozen[i][1] < 0: aligned += 1
                elif tg * frozen[i][1] > 0: opposite += 1
                if h < 0:
                    other_effect = (aggregate[i] - tg) * delta
                    if other_effect > 0: opposition += other_effect
                    elif other_effect < 0: helped += -other_effect
        metrics[label] = {'coordinates': len(indices), 'target_term_nonzero': nonzero,
                          'target_term_l1': magnitude, 'target_term_l2': math.sqrt(norm2),
                          'negative_legal_displacement_potential': potential,
                          'aggregate_rank_direction_helping_target_count': aligned,
                          'aggregate_rank_direction_opposing_target_count': opposite,
                          'aggregate_abs_over_all_term_abs_on_target_support': cancel_num/cancel_den if cancel_den else None,
                          'other_terms_opposing_target_best_destination_positive_utility': opposition,
                          'other_terms_helping_target_best_destination_negative_utility_magnitude': helped,
                          'target_support_rank_min': min(support_ranks) if support_ranks else None,
                          'target_support_rank_max': max(support_ranks) if support_ranks else None}
    total_potential = sum(m['negative_legal_displacement_potential'] for m in metrics.values())
    coverage = metrics['selected960']['negative_legal_displacement_potential']/total_potential if total_potential else None
    rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    rss_bytes = rss if sys.platform == 'darwin' else rss*1024
    result = {'schema': 'uor-r4.pair-proposal-attribution/1', 'status': 'PASS',
              'scope': 'Source-bound saved receipt attribution only; prior independent arithmetic audit reused, not rerun',
              'focus': dict(focus, objective_slot=slot), 'source_commit': report['source_commit'],
              'producer_source_sha256': sha(source), 'script_sha256': sha(pathlib.Path(__file__)),
              'recovered_identity_sha256': sha(args.identity), 'authenticated_inputs': pins,
              'prior_audit_result_sha256': sha(args.evidence / 'audits/pair-0001-attempt1/result.json'),
              'original_master_authority': {'file': 'generate-initial-masters.f32le', 'sha256': sha(args.run / 'generate-initial-masters.f32le'),
                                            'binding': 'generate-original-master-binding.parameters.generate.pair', 'scope': 'ALL57600 original fractional values; no reconstruction from negative checkpoint'},
              'alternatives': {'count': sum(classes.values()), 'classification_counts': dict(classes),
                               'guard_status_counts': dict(gates), 'chosen_token_counts': dict(winner_values),
                               'probability_relation_to_actual_incumbent_counts': dict(probability),
                               'best_observed_target_probability': dict(best, probability=best['target_mass']/best['total_mass']),
                               'first_guard_veto_row_counts': dict(first_veto_counts),
                               'target_winner_crossings': crossings, 'accepted_target_winner_orders': winning_commits,
                               'accepted_then_lost_orders': lost_commits,
                               'winner_outcome': 'NO_WINNING_SAVED_PROPOSAL' if not crossings else 'SEE_CATEGORIZED_CROSSINGS'},
              'baseline': {'mass': original[0], 'total': original[1], 'chosen': original[2], 'probability': original[0]/original[1]},
              'final': {'mass': incumbent[0], 'total': incumbent[1], 'chosen': incumbent[2], 'probability': incumbent[0]/incumbent[1]},
              'gradient_attribution': {'physical_weight': 1/15, 'physical_terms': 31, 'conceptual_roles': 32,
                                       'reweighting': False, 'p3_task_reference_separate_gradient': 'UNAVAILABLE_COALESCED_PHYSICAL_TERM',
                                       'groups': metrics, 'selected_share_of_target_negative_legal_potential': coverage,
                                       'utility_scope': 'Original frozen first-order gradient times actual legal fractional displacement; not native score/proposal/feasibility evaluation',
                                       'remaining_coordinates': 'NOT_NATIVE_EVALUATED'},
              'accepted_coordinates': accepted, 'elapsed_seconds': time.monotonic()-start, 'peak_RSS_bytes': rss_bytes,
              'workers': 1, 'new_scores': 0, 'new_gradients': 0, 'new_native_proposals': 0, 'model_execution': False,
              'limits': 'No feasibility or capacity conclusion for56640 unexamined coordinates; no automatic larger sweep; gradient utility is local first-order attribution, not hard loss improvement'}
    need(sum(classes.values()) == 13440 and rss_bytes <= 1024**3, 'bounded attribution count/RAM')
    with (args.output / 'report.json').open('x', encoding='utf-8') as out:
        json.dump(result, out, indent=2); out.write('\n')
    print(json.dumps({'status': 'PASS', 'report_sha256': sha(args.output / 'report.json'),
                      'classification_counts': dict(classes), 'winner_counts': dict(winner_values),
                      'target_potential_coverage': coverage, 'gradient_groups': metrics,
                      'seconds': result['elapsed_seconds'], 'peak_RSS_bytes': rss_bytes}))


if __name__ == '__main__':
    main()
