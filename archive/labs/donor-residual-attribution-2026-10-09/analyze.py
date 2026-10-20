#!/usr/bin/env python3
"""Attribute authenticated saved constructor observations, without evaluating scores.
The caller claims output with Rust report_output and seals/verifies it afterward.
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

SOURCE = '4f7eee35b250b6d5bf9e250fc0ea00d356998695'
REPORT = '56faa224da29b5da1db6fe82e21c4d19b6474be1316fef7fcb4716013cdd01a8'
MANIFEST = '9c19b82392b9e8f2263257992bf321ef5ed939e3b6e7788b783aa21328e45726'
IDENTITY = 'a687437f618815bac8a6e482dfcc62630e6b6878ced8eb2ceda0f4b75ed46afa'
RESIDUALS = [7, 8, 9, 10, 13, 14]
FIELDS = ['combined', 'task', 'reference', 'correct_reference_frames', 'all_phase_winners', 'objective_masses']


def need(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b''):
            digest.update(chunk)
    return digest.hexdigest()


def read(path):
    with path.open(encoding='utf-8') as stream:
        return json.load(stream)


def compact(value):
    return {key: value[key] for key in FIELDS}


def improves(before, after):
    return after < before - 1e-10 * (1.0 + abs(before))


def native_code(value):
    need(math.isfinite(value) and -1.75 <= value <= 1.75, 'original Q4 master domain')
    return int(math.copysign(math.floor(abs(value * 4) + .5), value))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['run', 'identity', 'evidence', 'output']:
        parser.add_argument('--' + name, type=pathlib.Path, required=True)
    args = parser.parse_args()
    need(args.output.is_dir() and set(p.name for p in args.output.iterdir()) == {'attempt.json'},
         'fresh already Rust-claimed output containing only attempt.json required')
    need(read(args.output / 'attempt.json')['schema'] == 'uor-r4.report-attempt/1', 'Rust claim schema')
    start = time.monotonic()
    need(sha(args.identity) == IDENTITY, 'pinned preservation identity')
    identity = read(args.identity)
    need(identity['complete_member_inventory_verified'] is True and
         identity['files_count'] == len(identity['files']) == 608 and identity['source_commit'] == SOURCE,
         'complete preservation inventory scope')
    used = {}

    def auth(path):
        path = path.resolve()
        name = path.relative_to(args.evidence.resolve()).as_posix()
        pin = identity['files'][name]
        need(path.is_file() and not path.is_symlink() and path.stat().st_size == pin['bytes'] and
             sha(path) == pin['sha256'], 'authenticated used file differs: ' + name)
        used[name] = pin
        return path

    def value(name):
        return read(auth(args.run / name))

    need(sha(auth(args.run / 'report.json')) == identity['run_report_sha256'] == REPORT and
         sha(auth(args.run / 'manifest.json')) == identity['run_manifest_sha256'] == MANIFEST,
         'pinned model report/manifest')
    report = value('report.json')
    population = value('coupled-population.json')
    authority = value('episode-objective-authority.json')
    order = value('coupled-coordinate-order.json')
    journal = value('coupled-construction.json')
    need(report['status'] == 'COMPLETED' and report['source_commit'] == SOURCE and
         report['policy']['donor_credit'] == 'full_pool_utility' and report['candidate_native_steps'] == 391 and
         report['selected_model'] is False, 'corrected native391 run scope')
    need(journal['policy'] == report['policy'], 'journal policy identity')
    need(population['guards'] == 380 and population['unique_union'] == 391 and
         population['physical_frames'] == 31 and population['weighted_roles'] == 32 and
         len(population['rows']) == 391 and len(population['roles']) == 32,
         'complete guard/objective/role population')
    rows, mapping = population['rows'], population['objective_row_map']
    need(len(mapping) == len(set(mapping)) == 31 and len(authority['physical_frames']) == 31,
         'unique physical objective map')
    slots = {}
    for slot, row_index in enumerate(mapping):
        row, frame = rows[row_index], authority['physical_frames'][slot]
        need(row['row'] == row_index and frame['physical_index'] == slot and
             (frame['input'], frame['position'], frame['target']) ==
             (row['input_index'], row['position'], row['target_label_only']), 'physical frame identity')
        slots[(row['input_index'], row['position'])] = slot
    task_roles = [r for r in population['roles'] if r['task'] is True]
    refs = [r for r in population['roles'] if r['task'] is False]
    need(len(task_roles) == 15 and len(refs) == 17 and
         sorted(r['position'] for r in task_roles) == list(range(15)) and
         len(set(r['input'] for r in task_roles)) == 1, 'explicit full episode/reference roles')
    phase_slots = {r['position']: slots[(r['input'], r['position'])] for r in task_roles}
    targets = {pos: rows[mapping[slot]]['target_label_only'] for pos, slot in phase_slots.items()}
    ref_slots = [slots[(r['input'], r['position'])] for r in refs]
    guards = population['guard_population']['terms']
    need(len(guards) == 380, 'all guard identities')
    for index, guard in enumerate(guards):
        row = rows[index]
        need((guard['id'], guard['input_index'], guard['position'], guard['actual_prefix_ids'], guard['required_original_winner']) ==
             (row['id'], row['input_index'], row['position'], row['actual_prefix_ids'], row['target_label_only']),
             'guard ordering and identity')

    def check_objective(v):
        masses = v['objective_masses']
        need(len(masses) == 31 and all(len(m) == 3 and all(type(x) is int for x in m) and
             0 < m[0] <= m[1] and 0 <= m[2] < 4096 for m in masses), 'saved objective mass domain')
        need(all(math.isfinite(v[k]) for k in ['combined', 'task', 'reference']), 'finite saved CE')
        goodrefs = sum(masses[s][2] == rows[mapping[s]]['target_label_only'] for s in ref_slots)
        correct = [pos for pos, slot in sorted(phase_slots.items()) if masses[slot][2] == targets[pos]]
        need(v['correct_reference_frames'] == goodrefs and v['all_phase_winners'] is (len(correct) == 15),
             'saved winner/reference flags agree role mapping')
        return correct

    summary = journal['summary']
    current = compact(summary['initial'])
    need(current == compact(report['baseline_objective']), 'baseline endpoints')
    baseline_correct = check_objective(current)
    epoch = 0
    records = journal['coordinate_records']
    need(len(records) == len(order) == 1920, 'full frozen coordinate population')
    counts = collections.Counter()
    stats = {p: {'winning_alternatives': 0, 'objective_reference_rejected': 0, 'CE_only_rejected': 0,
                 'reference_only_rejected': 0, 'CE_and_reference_rejected': 0, 'first_guard_veto': 0,
                 'feasible_unselected': 0, 'selected_winning': 0, 'committed_gains': [],
                 'committed_losses': [], 'winning_event_indices': []} for p in RESIDUALS}
    events, commits, seen = [], [], set()
    maximum_feasible = -1
    maximum_examples = []
    accepted = collections.Counter()
    for ordinal, rec in enumerate(records):
        family, index = rec['family'], rec['index']
        need(rec['order'] == ordinal and rec['incumbent_epoch'] == epoch and
             family in ['prefix.coefficients', 'generate.unary'] and 0 <= index < 960 and
             (family, index) not in seen, 'coordinate order/domain/epoch')
        seen.add((family, index))
        ranked = order[ordinal]
        for key in ['family', 'index', 'original_master', 'gradient', 'priority', 'rank_code']:
            need(rec[key] == ranked[key], 'frozen rank field differs: ' + key)
        need(rec['rank_status'] == ranked['status'], 'frozen rank status')
        need(struct.unpack('<I', struct.pack('<f', rec['original_master']))[0] == rec['original_master_bits'],
             'original fractional master bits')
        need(struct.unpack('<I', struct.pack('<f', rec['gradient']))[0] == rec['gradient_bits'] and
             struct.unpack('<Q', struct.pack('<d', rec['priority']))[0] == rec['priority_bits'] and
             native_code(rec['original_master']) == ranked['original_code'],
             'frozen gradient/priority bits and original native code')
        alternatives = rec['alternatives']
        if family == 'generate.unary':
            incumbent_code = native_code(rec['original_master'])
            need([a['code'] for a in alternatives] == [q for q in range(-7, 8) if q != incumbent_code],
                 'all14 signed legal alternatives, no re-evaluation at another incumbent')
        else:
            need(len(alternatives) == int(rec['rank_status'] == 'eligible') and
                 (not alternatives or alternatives[0]['code'] == rec['rank_code']), 'preferred adjacent/NOOP count')
        selected = rec['selected']
        eligible = []
        for alt in alternatives:
            counts['alternatives'] += 1
            need(alt['incumbent_epoch'] == epoch and type(alt['code']) is int and -7 <= alt['code'] <= 7,
                 'alternative same-incumbent identity')
            v = alt['objective']
            correct = check_objective(v)
            ce = improves(current['combined'], v['combined'])
            reference = v['correct_reference_frames'] == 17
            gate = ce and reference
            need(alt['strict_current_CE_and17'] is gate, 'source strict combined CE/reference predicate')
            affected, checked = alt['affected_guard_indices'], alt['checked_guard_indices']
            need(affected == sorted(set(affected)) and all(0 <= x < 380 for x in affected) and
                 checked == affected[:len(checked)], 'truthful ascending guard traversal')
            failure = alt['first_failure']
            if not gate:
                status = 'NOT_CHECKED_OBJECTIVE_GATE_FALSE'
                need(not checked and failure is None, 'ineligible alternative did not examine guards')
            elif failure is not None:
                status = 'FIRST_VETO'
                need(checked and failure['guard_index'] == checked[-1] and
                     failure['required'] == rows[checked[-1]]['target_label_only'] and
                     failure['chosen'] != failure['required'], 'first measured guard veto identity')
            else:
                status = 'FULL_PASS'
                need(checked == affected, 'complete affected guard traversal')
            need(alt['guard_status'] == status and alt['feasible'] is (status == 'FULL_PASS'),
                 'guard status/feasibility agreement')
            counts[family + ':' + status] += 1
            if alt['feasible']:
                eligible.append(alt)
                if len(correct) > maximum_feasible:
                    maximum_feasible, maximum_examples = len(correct), []
                if len(correct) == maximum_feasible and len(maximum_examples) < 12:
                    maximum_examples.append({'order': ordinal, 'family': family, 'index': index,
                                             'code': alt['code'], 'epoch': epoch, 'correct_positions': correct})
            winning = [p for p in RESIDUALS if p in correct]
            is_selected = selected['status'] == 'committed' and selected['code'] == alt['code']
            if winning:
                classification = ('selected_winning' if is_selected else 'feasible_unselected') if alt['feasible'] else (
                    'first_guard_veto' if status == 'FIRST_VETO' else 'objective_reference_rejected')
                event = {'order': ordinal, 'family': family, 'index': index, 'code': alt['code'],
                         'incumbent_epoch': epoch, 'winning_residual_positions': winning,
                         'correct_positions': correct, 'classification': classification,
                         'CE_descent_passed': ce, 'references_passed': reference,
                         'current_combined_CE': current['combined'], 'proposed_combined_CE': v['combined'],
                         'guard_status': status, 'affected_guard_indices': affected,
                         'checked_guard_indices': checked, 'first_failure': failure,
                         'unvisited_affected_guard_count': len(affected) - len(checked),
                         'staged_summary_digest': alt['staged_summary_digest'],
                         'residual_masses': {str(p): v['objective_masses'][phase_slots[p]] for p in winning}}
                event_index = len(events)
                events.append(event)
                for p in winning:
                    st = stats[p]
                    st['winning_alternatives'] += 1
                    st[classification] += 1
                    st['winning_event_indices'].append(event_index)
                    if classification == 'objective_reference_rejected':
                        st['CE_only_rejected' if not ce and reference else
                           'reference_only_rejected' if ce and not reference else 'CE_and_reference_rejected'] += 1
        best = min(eligible, key=lambda a: (a['objective']['combined'], a['code'])) if eligible else None
        if best is not None:
            need(selected['status'] == 'committed' and selected['code'] == best['code'] and
                 selected['staged_summary_digest'] == best['staged_summary_digest'],
                 'selected feasible min-CE/code identity')
            after = compact(best['objective'])
            for p in RESIDUALS:
                before_win = current['objective_masses'][phase_slots[p]][2] == targets[p]
                after_win = after['objective_masses'][phase_slots[p]][2] == targets[p]
                if before_win != after_win:
                    stats[p]['committed_gains' if after_win else 'committed_losses'].append({
                        'order': ordinal, 'family': family, 'index': index, 'code': best['code'],
                        'epoch_before': epoch, 'epoch_after': epoch + 1,
                        'before_mass': current['objective_masses'][phase_slots[p]],
                        'after_mass': after['objective_masses'][phase_slots[p]]})
            current = after
            epoch += 1
            accepted[family] += 1
            commits.append({'order': ordinal, 'epoch': epoch, 'family': family, 'index': index,
                            'code': best['code'], 'correct_positions': check_objective(current)})
        else:
            expected = 'noop' if family == 'prefix.coefficients' and rec['rank_status'] != 'eligible' else 'unchanged'
            need(selected['status'] == expected and 'code' not in selected and
                 (expected != 'noop' or selected['reason'] == rec['rank_status']), 'noncommitted original bits/NOOP semantics')
        need(rec['epoch_after'] == epoch, 'commit epoch increment or unchanged epoch')
    need(counts['alternatives'] == summary['evaluated_alternatives'] == 14292 and
         accepted['prefix.coefficients'] == summary['accepted_prefix'] and
         accepted['generate.unary'] == summary['accepted_generate'] and
         epoch == summary['accepted_epoch'], 'complete constructor counts')
    need(current == compact(summary['final']) == compact(report['candidate_objective']), 'final selected endpoint')
    final_correct = check_objective(current)
    need([p for p in range(15) if p not in final_correct] == RESIDUALS, 'fixed residual population')
    for pos, slot in sorted(phase_slots.items()):
        row = rows[mapping[slot]]
        snapshot = value('candidate-row-%04d-position-%02d.json' % (row['input_index'], row['position']))
        need((snapshot['input_index'], snapshot['position'], snapshot['target_label_only']) ==
             (row['input_index'], row['position'], row['target_label_only']), 'final native snapshot identity')
        native = snapshot['native']
        masses = {m['token_id']: m['weight_q31'] for m in native['pool']['token_masses']}
        endpoint = [masses[targets[pos]], native['pool']['summary']['total_weight_q31'],
                    native['pool']['summary']['chosen_token_id']]
        need(endpoint == current['objective_masses'][slot], 'native final phase masses/winner equal selected endpoint')
    for p, st in stats.items():
        st['target'] = targets[p]
        st['physical_slot'] = phase_slots[p]
        st['union_row'] = mapping[phase_slots[p]]
        st['no_winning_proposal'] = st['winning_alternatives'] == 0
        st['baseline_mass'] = summary['initial']['objective_masses'][phase_slots[p]]
        st['final_mass'] = current['objective_masses'][phase_slots[p]]
        st['final_probability_increased'] = (st['final_mass'][0] * st['baseline_mass'][1] >
                                            st['baseline_mass'][0] * st['final_mass'][1])
    result = {'schema': 'uor-r4.saved-full-donor-residual-attribution/1', 'status': 'PASS_SAVED_ATTRIBUTION',
              'source_commit': SOURCE, 'input_report_sha256': REPORT, 'input_manifest_sha256': MANIFEST,
              'preservation_identity_sha256': IDENTITY, 'analyzer_sha256': sha(pathlib.Path(__file__)),
              'used_files': used, 'full_population': {'guards': 380, 'union': 391, 'roles': 32,
              'physical_objectives': 31, 'episode_phases': 15, 'coordinates': 1920,
              'alternatives': counts['alternatives'], 'accepted_epochs': epoch},
              'source_threshold': 'next.combined < current.combined - 1e-10*(1+abs(current.combined)); references17',
              'baseline_correct_positions': baseline_correct, 'final_correct_positions': final_correct,
              'residual_history': stats, 'winning_events': events, 'committed_epoch_winners': commits,
              'counts': dict(counts), 'maximum_correct_in_single_feasible_alternative': maximum_feasible,
              'maximum_examples': maximum_examples,
              'limits': ['Saved observation attribution, not a numerical audit or model replay.',
              'Objective-ineligible guards are UNAVAILABLE; FIRST_VETO leaves later guards unexamined.',
              'Maximum feasible correctness is per single offered alternative, never across-epoch composition.',
              'No unexamined coefficient, capacity, whole-answer, or serving claim; probability gain differs from winner crossing.'],
              'elapsed_seconds': time.monotonic() - start,
              'peak_rss_bytes': resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * (1 if sys.platform == 'darwin' else 1024)}
    with (args.output / 'report.json').open('x', encoding='utf-8') as stream:
        json.dump(result, stream, indent=2, allow_nan=False)
        stream.write('\n')
    execution = {'status': 'PASS', 'argv': sys.argv, 'script_sha256': result['analyzer_sha256'],
                 'report_sha256': sha(args.output / 'report.json'), 'elapsed_seconds': result['elapsed_seconds'],
                 'peak_rss_bytes': result['peak_rss_bytes'], 'new_model_calls': 0, 'new_gradients': 0,
                 'new_scores_or_proposals': 0, 'seal_and_normative_verification': 'PENDING_PARENT'}
    with (args.output / 'execution.json').open('x', encoding='utf-8') as stream:
        json.dump(execution, stream, indent=2, allow_nan=False)
        stream.write('\n')
    print(json.dumps(execution), flush=True)


if __name__ == '__main__':
    main()
