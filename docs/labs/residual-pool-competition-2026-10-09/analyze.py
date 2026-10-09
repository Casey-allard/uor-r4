#!/usr/bin/env python3
"""Saved endpoint atom arithmetic and finite feature incidence; no model replay.
Output must already be exclusively claimed by native Rust report_output.
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
EXP_SHA = '79485d6e63cc28f5e01d98c5d73abe021db33fa7fef368142ae6592d06b4817f'
TARGET, RIVAL = 324, 307
CLIP = 8 << 24


def need(ok, reason):
    if not ok:
        raise ValueError(reason)


def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1 << 20), b''):
            digest.update(block)
    return digest.hexdigest()


def read(path):
    with path.open(encoding='utf-8') as stream:
        return json.load(stream)


def signed_nibble(packed, index):
    need(0 <= index < len(packed) * 2, 'packed index domain')
    code = (packed[index // 2] >> (4 * (index % 2))) & 15
    code = code - 16 if code >= 8 else code
    need(-7 <= code <= 7, 'admitted signed Q4 domain')
    return code


def weight(raw, reference, table):
    clipped = max(-CLIP, min(CLIP, raw))
    need(-CLIP <= reference <= CLIP and reference >= clipped, 'native reference domain')
    gap = reference - clipped
    index, fraction = gap >> 16, gap & 65535
    need(index + 1 < len(table), 'canonical gap table boundary')
    a, b = table[index:index + 2]
    return a - (((a - b) * fraction) >> 16)


def contrast(target, rival):
    counts = collections.Counter(target)
    counts.subtract(rival)
    return [{'key': key, 'signed_multiplicity': count} for key, count in sorted(counts.items()) if count]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['run', 'identity', 'evidence', 'output']:
        parser.add_argument('--' + name, type=pathlib.Path, required=True)
    args = parser.parse_args()
    need(args.output.is_dir() and set(p.name for p in args.output.iterdir()) == {'attempt.json'} and
         read(args.output / 'attempt.json')['schema'] == 'uor-r4.report-attempt/1',
         'already native-claimed fresh output required')
    began = time.monotonic()
    need(sha(args.identity) == IDENTITY, 'pinned complete preservation identity')
    identity = read(args.identity)
    need(identity['source_commit'] == SOURCE and identity['files_count'] == len(identity['files']) == 608 and
         identity['complete_member_inventory_verified'] is True, 'complete source-bound recovery')
    used = {}

    def auth(path):
        need(path.is_file() and not path.is_symlink(), 'regular used input required')
        relative = path.resolve().relative_to(args.evidence.resolve()).as_posix()
        pin = identity['files'][relative]
        need(path.stat().st_size == pin['bytes'] and sha(path) == pin['sha256'], 'used file identity: ' + relative)
        used[relative] = pin
        return path

    def value(leaf):
        return read(auth(args.run / leaf))

    report = value('report.json')
    need(sha(auth(args.run / 'report.json')) == REPORT and sha(auth(args.run / 'manifest.json')) == MANIFEST and
         report['status'] == 'COMPLETED' and report['source_commit'] == SOURCE and report['selected_model'] is False and
         report['policy']['donor_credit'] == 'full_pool_utility' and report['candidate_native_steps'] == 391,
         'completed unselected corrected endpoint')
    snapshot = value('candidate-row-0245-position-07.json')
    need(snapshot['input_index'] == 245 and snapshot['position'] == 7 and snapshot['target_label_only'] == TARGET and
         snapshot['actual_prefix_ids'] == [617, 2097, 315, 1057, 267, 307, 397], 'fixed conditional endpoint')
    n = snapshot['native']
    gen, ids, copy, u = n['generate_q24'], n['copy_ids'], n['copy_q24'], n['continuation']['delta_scores_q24']
    need(len(gen) == len(u) == 4096 and len(ids) == len(copy) == 16 and
         n['schema'] == 'uor-r4.native-prefix-trajectory-pool/1' and
         n['pool_action_trace'].startswith('OMITTED_RECONSTRUCTIBLE_FROM_COMPLETE_SCORES'), 'complete saved score scope')
    exp_path = auth(args.run / 'checkpoint-0001/native/consumer/exp-q31.bin')
    exp_bytes = exp_path.read_bytes()
    need(len(exp_bytes) == 32776 and sha(exp_path) == EXP_SHA, 'canonical native exponential bytes')
    table = struct.unpack('<8194I', exp_bytes)
    need(table[0] == 1 << 31 and table[4097] > 0 and
         all(a >= b >= 0 for a, b in zip(table, table[1:])), 'canonical table monotonicity/positive admitted gap')
    reference = max(max(max(-CLIP, min(CLIP, x)) for x in gen), max(max(-CLIP, min(CLIP, x)) for x in copy))
    generate_weights = [weight(x, reference, table) for x in gen]
    copy_weights = [weight(x, reference, table) for x in copy]
    masses = generate_weights.copy()
    for token, mass in zip(ids, copy_weights):
        masses[token] += mass
    saved_masses = n['pool']['token_masses']
    need(len(saved_masses) == 4096 and {m['token_id'] for m in saved_masses} == set(range(4096)), 'all legal saved IDs')
    for entry in saved_masses:
        token = entry['token_id']
        need(entry['weight_q31'] == masses[token] and entry['generate_weight_q31'] == generate_weights[token] and
             entry['copy_weight_q31'] == masses[token] - generate_weights[token], 'exact saved native token mass parity')
    summary = n['pool']['summary']
    winner = min(range(4096), key=lambda t: (-masses[t], t))
    need(winner == summary['chosen_token_id'] == RIVAL and reference == summary['max_score_q24'] and
         sum(masses) == summary['total_weight_q31'] and masses[winner] == summary['chosen_weight_q31'] and
         sum(generate_weights) == summary['generate_weight_q31'] and sum(copy_weights) == summary['copy_weight_q31'] and
         summary['legal_generate_actions'] == 4096 and summary['copy_actions'] == 16 and
         summary['clipped_low_actions'] == sum(x < -CLIP for x in gen + copy) and
         summary['clipped_high_actions'] == sum(x > CLIP for x in gen + copy), 'complete common-reference pool parity')
    base_copy = [raw - u[token] for token, raw in zip(ids, copy)]
    donor = min(range(len(base_copy)), key=lambda i: (-base_copy[i], i))
    need(donor == n['bridge']['selected_ordinal'] and
         n['bridge']['selected_candidate'] == n['bank_trace']['cue_bank']['bank']['candidates'][donor],
         'earliest physical BASE Copy donor before U/alias pooling')
    prefix = n['bank_trace']['prefix']
    bank = n['bank_trace']['cue_bank']['bank']
    cue = n['bank_trace']['cue_bank']['carrier']
    packed_prefix = auth(args.run / 'checkpoint-0001/prefix/prefix-q4.bin').read_bytes()
    need(len(packed_prefix) == 480 and len(prefix['angular_indices']) == 8 and
         all(len(lane) == 16 for lane in prefix['angular_indices']), 'complete Prefix lane/occurrence scope')
    aliases = {token: [i for i, t in enumerate(ids) if t == token] for token in [TARGET, RIVAL]}
    key_sets, tokens = {}, {}
    source_actions = {a['action']['Copy']['source_offset']: a for a in bank['actions']['actions'] if 'Copy' in a['action']}
    for token, ordinals in aliases.items():
        rows = []
        for ordinal in ordinals:
            candidate = bank['candidates'][ordinal]
            need(candidate['bank_index'] == ordinal and candidate['occurrence']['token_id'] == token,
                 'physical alias identity')
            keys = [lane * 120 + prefix['angular_indices'][lane][ordinal] for lane in range(8)]
            codes = [signed_nibble(packed_prefix, key) for key in keys]
            prefix_heads = [prefix['copy_q24'][h][ordinal] for h in range(2)]
            need(prefix_heads == [sum(codes[h * 4:h * 4 + 4]) << 22 for h in range(2)],
                 'Prefix packed gather equals saved head adjustments')
            combined = [h['scores_q24'][ordinal] for h in bank['heads']]
            cue_heads = [h[ordinal] for h in cue['copy_q24']]
            source_heads = [a - b - c for a, b, c in zip(combined, cue_heads, prefix_heads)]
            need(sum(combined) + u[token] == copy[ordinal] and
                 sum(combined) == source_actions[ordinal]['score_q24'] and
                 source_actions[ordinal]['token_id'] == token and
                 all(bank['actions']['head_scores'][h]['copy_q24'][ordinal] == combined[h]
                     for h in range(2)),
                 'adjusted bank heads/actions and post-U native Copy authority')
            rows.append({'ordinal': ordinal, 'candidate': candidate,
                         'prefix_source_index': prefix['candidate_source_indices'][ordinal],
                         'prefix_source_offset': prefix['candidate_offsets'][ordinal],
                         'prefix_keys': keys, 'prefix_current_signed_codes': codes,
                         'source_head_q24': source_heads,
                         'source_head_scope': 'DERIVED_BY_SUBTRACTING_SAVED_CUE_AND_PREFIX; NOT_INDEPENDENT_SOURCE_REPLAY',
                         'bank_action_scope': 'POST_CUE_AND_PREFIX_PRE_U',
                         'bank_action_q24': source_actions[ordinal]['score_q24'], 'cue_head_q24': cue_heads,
                         'prefix_head_q24': prefix_heads, 'combined_head_q24': combined,
                         'U_q24': u[token], 'Copy_plus_U_raw_q24': copy[ordinal],
                         'Copy_clipped_q24': max(-CLIP, min(CLIP, copy[ordinal])),
                         'native_atom_weight_q31': copy_weights[ordinal]})
        key_sets[token] = [key for row in rows for key in row['prefix_keys']]
        tokens[token] = {'token_id': token, 'Generate_plus_U_q24': gen[token],
                         'U_q24': u[token], 'base_Generate_q24': gen[token] - u[token],
                         'generate_mass_q31': generate_weights[token],
                         'copy_mass_q31': sum(copy_weights[i] for i in ordinals),
                         'total_mass_q31': masses[token], 'physical_aliases': rows}
    g = value('checkpoint-0001/generate.bin')
    payload, metadata = g['payload'], g['metadata']
    algebra, energy = payload['algebra'], payload['energy']
    need(energy['lanes'] == 8 and len(energy['edges']) == 4 and len(payload['prototypes']) == 4096 * 8 and
         len(algebra['product']) == 120 * 128 and len(algebra['inverse']) == 120 and
         len(n['post_state']) == 8 and all(0 <= x < 120 for x in n['post_state']), 'admitted finite Generate geometry shape')
    factor_keys = {}
    for token in [TARGET, RIVAL]:
        relative = [algebra['product'][algebra['inverse'][n['post_state'][lane]] * 128 +
                                      payload['prototypes'][token * 8 + lane]] for lane in range(8)]
        need(all(0 <= r < 120 for r in relative), 'directed finite relative roots')
        unary_keys = [lane * 120 + r for lane, r in enumerate(relative)]
        pair_keys = [960 + edge * 14400 + relative[e['left']] * 120 + relative[e['right']]
                     for edge, e in enumerate(energy['edges'])]
        factor_keys[token] = {'relative_codes': relative, 'unary_keys': unary_keys, 'ordered_pair_keys': pair_keys,
                             'current_unary_signed_codes': [signed_nibble(energy['unary_packed'], lane * 128 + r)
                                                            for lane, r in enumerate(relative)]}
    focused = set(key_sets[TARGET] + key_sets[RIVAL])
    journal = value('coupled-construction.json')
    need(journal['policy'] == report['policy'] and len(journal['coordinate_records']) == 1920,
         'complete frozen journal policy')
    decisions = []
    for rec in journal['coordinate_records']:
        if rec['family'] == 'prefix.coefficients' and rec['index'] in focused:
            decisions.append({key: rec[key] for key in ['order', 'index', 'original_master', 'original_master_bits',
                              'gradient', 'gradient_bits', 'priority', 'rank_code', 'rank_status',
                              'incumbent_epoch', 'epoch_after', 'selected']})
            current_code = signed_nibble(packed_prefix, rec['index'])
            selected = rec['selected']
            original_q = int(math.copysign(math.floor(abs(rec['original_master'] * 4) + .5), rec['original_master']))
            need(current_code == (selected['code'] if selected['status'] == 'committed' else original_q),
                 'focused final packed code equals recorded selection')
    need(len(decisions) == len(focused), 'each focused Prefix key has one recorded decision')
    source_root = pathlib.Path(__file__).resolve().parents[3]
    source_pins = {
        'crates/uor-r4-integer/src/geometric_vocabulary_actions.rs': '053d2cf430e501862b3d1dcbd3f261bdb86bb85e2c66e87104d8ac2444de51ce',
        'crates/uor-r4-integer/src/geometric_source_realizer.rs': 'b72f3e91ba825dfccd66015b52597a1e1a14ad5c1e1c0554cfd6f0eaa65f9b48',
        'crates/uor-r4-integer/src/stack/kernels.rs': '9566d450c7a9ef5932ecd553d6fe3a06bd319a9d572091e10384418f42f48b88',
        'crates/uor-r4-integer/src/geometric_prefix_transport.rs': '3d6949ae43faf9435be35d31749db77cb294cdff33519361e0e4bff564326de5',
        'crates/uor-r4-core/src/native_geometric/learner/geometric_generate.rs': '30cb4b8c405799a020530b6b53a527bbda8a7e2ddd4a64a32cdc480c06662a49',
        'crates/uor-r4-training/examples/geometric_frozen_map_fit/context_cue_coadapt.rs': '2efd3f31ca6192fed70edc5056550db2656dad1c9f27e177bb5493eda16773a5',
        'crates/uor-r4-training/examples/geometric_frozen_map_fit/coupled_episode_learning.rs': '0350c3e1ca574741a283cfc3727ccf36522533c89a710ce3717c559cc39d6923'}
    need(all(sha(source_root / path) == digest for path, digest in source_pins.items()),
         'reused executed numerical/source module identity')
    result = {'schema': 'uor-r4.saved-residual-pool-competition/1', 'status': 'PASS_SAVED_COMPETITION',
              'source_commit': SOURCE, 'report_sha256': REPORT, 'manifest_sha256': MANIFEST,
              'identity_sha256': IDENTITY, 'script_sha256': sha(pathlib.Path(__file__)),
              'used_files': used, 'source_files': source_pins,
              'input_index': 245, 'position': 7, 'actual_prefix_ids': snapshot['actual_prefix_ids'],
              'scope': 'Final canonical-conditional endpoint; not actual ownfeedback arrival or original-selected endpoint.',
              'common_pool_reference_q24': reference, 'native_total_mass_q31': sum(masses),
              'target': tokens[TARGET], 'rival': tokens[RIVAL], 'factual_bridge': n['bridge'],
              'earliest_BASE_Copy_donor': donor, 'BASE_Copy_q24': base_copy, 'post_state': n['post_state'], 'continuation_state_codes': n['continuation']['state_codes'],
              'Prefix_signed_contrast': contrast(key_sets[TARGET], key_sets[RIVAL]),
              'Generate_features': factor_keys,
              'Generate_unary_signed_contrast': contrast(factor_keys[TARGET]['unary_keys'], factor_keys[RIVAL]['unary_keys']),
              'Generate_pair_signed_contrast': contrast(factor_keys[TARGET]['ordered_pair_keys'], factor_keys[RIVAL]['ordered_pair_keys']),
              'focused_Prefix_recorded_decisions': decisions,
              'arithmetic': 'Canonical retained exp table/integer interpolation reconstructed all4096 Generate atoms+16physicalCopy aliases at saved common reference; exact token mass and summary parity. Prefix gathers recomputed from current packed codes. Generate incidence uses inverse(post)*prototype finite table lookup ONLY; no Generate score/energy replay.',
              'superseded_attempt1_boundary': 'Failed assertion incorrectly compared derived Source-only sum with bank.actions; actual trace is post-Cue/Prefix. Attempt1 remains a retained setup/schema failure, not model evidence.',
              'limits': ['No alternate pool, proposal, gradient, model encoder, optimizer or new native inference.',
                         'Distinct feature keys are not feasibility or capacity evidence; no all-guard sharing analysis.',
                         'Generate+U saved raw vector is separated by exact saved U subtraction; bank heads AND bank action trace include Cue/Prefix and precede U; Source-only residual is derived, not independently replayed.',
                         'Recorded Prefix decisions concern the one completed policy and epoch; no new sweep is admitted.'],
              'elapsed_seconds': time.monotonic() - began,
              'peak_rss_bytes': resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * (1 if sys.platform == 'darwin' else 1024)}
    with (args.output / 'report.json').open('x') as stream:
        json.dump(result, stream, indent=2, allow_nan=False)
        stream.write('\n')
    execution = {'status': 'PASS', 'argv': sys.argv, 'script_sha256': result['script_sha256'],
                 'report_sha256': sha(args.output / 'report.json'), 'elapsed_seconds': result['elapsed_seconds'],
                 'peak_rss_bytes': result['peak_rss_bytes'], 'new_model_calls': 0,
                 'Generate_score_regeneration': False, 'new_proposals_or_gradients': 0,
                 'normative_seal_and_verify': 'PENDING_PARENT'}
    with (args.output / 'execution.json').open('x') as stream:
        json.dump(execution, stream, indent=2)
        stream.write('\n')
    print(json.dumps(execution), flush=True)


if __name__ == '__main__':
    main()
