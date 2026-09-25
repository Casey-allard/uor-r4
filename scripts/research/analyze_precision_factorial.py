#!/usr/bin/env python3
"""Arithmetic and endpoint verification over sealed Rust evaluation outputs.

Run first with --endpoints-only, then without it after QF/FQ finish. The supplied
Rust executable is invoked ONLY as `verify ROOT`; no inference or fitting occurs.
Outputs are exclusive new files in --root (default: beside this script), never
below sealed input roots. --prior and --worktree locate retained evidence and
the tracked evaluator/input receipt; identities remain the frozen authority.
"""

import sys
sys.dont_write_bytecode = True

import argparse
from array import array
import datetime as dt
import hashlib
import itertools
import json
import math
from pathlib import Path
import re
import struct
import subprocess
import uuid

ROOT = Path(__file__).resolve().parent
WORK = Path('/Users/casey.allard/uor-r4-worktrees/precision-factorial-20260925')
PRIOR = ROOT.parent / 'projected-recurrent-20260925'
ARMS = ('quaternion', 'householder_pair')
MODES = ('FF', 'QF', 'FQ', 'QQ')
RECORD = struct.Struct('<QIIddffi')
FIELDS = ('input_offset', 'target', 'predicted', 'target_probability', 'nll_nats',
          'no_read_mass', 'copy_gate', 'top_read_position')
N, TUNE, COMPARISON, CONTEXT = 249856, 16384, 233472, 256
PARTITIONS = {'tune': (0, TUNE), 'comparison': (TUNE, N), 'full': (0, N)}
QUARTERS = {f'comparison_quarter_{i + 1}': (TUNE + i * 58368, TUNE + (i + 1) * 58368) for i in range(4)}
EVALUATOR_SHA = 'd2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62'
OLD_SOURCE = '66349cdb69883dd7d438c76394c22f4e19000c73'
OLD_EXE_SHA = 'b14698807c4ab2635ca1de777f2c882d75b1d1ca96ebe61005581f6bcf360cec'
FORMULAS = {
    'parameter_cost_float_interfaces': 'QF - FF',
    'parameter_cost_quantized_interfaces': 'QQ - FQ',
    'interface_cost_float_parameters': 'FQ - FF',
    'interface_cost_quantized_parameters': 'QQ - QF',
    'interaction': 'QQ - QF - FQ + FF',
    'total_quantization_cost': 'QQ - FF',
}
EVIDENCE, VERIFIED, PARENTS = {}, {}, {}
EXE = EXE_SHA = SOURCE = DEV = EVALUATOR_DOCUMENT = None


class AnalysisFailure(Exception):
    def __init__(self, message, **evidence):
        super().__init__(message)
        self.evidence = {'message': message, **evidence}


def require(condition, message, **evidence):
    if not condition:
        raise AnalysisFailure(message, **evidence)


def reject_constant(value):
    raise AnalysisFailure('Nonfinite JSON constant', constant=value)


def identity(path):
    path = Path(path).resolve(strict=True)
    before = path.stat()
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    after = path.stat()
    require((before.st_ino, before.st_size, before.st_mtime_ns) == (after.st_ino, after.st_size, after.st_mtime_ns),
            'Input changed while hashing', path=str(path))
    result = {'path': str(path), 'bytes': after.st_size, 'sha256': digest.hexdigest()}
    EVIDENCE[str(path)] = result
    return result


def read_bytes(path):
    expected = identity(path)
    value = Path(path).read_bytes()
    require(hashlib.sha256(value).hexdigest() == expected['sha256'], 'Input changed during read', path=str(path))
    return value


def read_json(path):
    return json.loads(read_bytes(path), parse_constant=reject_constant)


def verify(root):
    root = Path(root).resolve(strict=True)
    if str(root) not in VERIFIED:
        checked = subprocess.run([str(EXE), 'verify', str(root)], capture_output=True, text=True, timeout=60)
        require(checked.returncode == 0, 'Rust report seal verification failed', root=str(root),
                exit_code=checked.returncode, stdout=checked.stdout, stderr=checked.stderr)
        VERIFIED[str(root)] = identity(root / 'manifest.json')


def strip_elapsed(value):
    if isinstance(value, dict):
        return {key: strip_elapsed(child) for key, child in value.items() if key != 'elapsed_seconds'}
    if isinstance(value, list):
        return [strip_elapsed(child) for child in value]
    return value


def first_json_difference(new, old, path='$'):
    if type(new) is not type(old):
        return {'json_path': path, 'new_type': type(new).__name__, 'old_type': type(old).__name__, 'new': new, 'old': old}
    if isinstance(new, dict):
        if new.keys() != old.keys():
            return {'json_path': path, 'only_new_keys': sorted(new.keys() - old.keys()), 'only_old_keys': sorted(old.keys() - new.keys())}
        for key in sorted(new):
            found = first_json_difference(new[key], old[key], path + '/' + key.replace('~', '~0').replace('/', '~1'))
            if found is not None:
                return found
    elif isinstance(new, list):
        if len(new) != len(old):
            return {'json_path': path, 'new_length': len(new), 'old_length': len(old)}
        for i, (a, b) in enumerate(zip(new, old)):
            found = first_json_difference(a, b, path + '/' + str(i))
            if found is not None:
                return found
    elif new != old:
        return {'json_path': path, 'new': new, 'old': old}
    return None


def canonical_hash(value):
    encoded = json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':'), allow_nan=False).encode()
    return hashlib.sha256(encoded).hexdigest()


def parent(arm):
    if arm in PARENTS:
        return PARENTS[arm]
    root = PRIOR / f'fit-{arm}-projected-1/checkpoint-final'
    verify(root)
    checkpoint = read_json(root / 'checkpoint.json')
    campaign = read_json(root / 'campaign.json')
    config = read_json(root / 'config.json')
    require(checkpoint['optimizer_step'] == checkpoint['next_data_step'] == 8348
            and checkpoint['source_commit'] == OLD_SOURCE and checkpoint['sampled_target_visits'] == 34193408,
            'Retained projected parent step/source differs', arm=arm)
    require(checkpoint['model_sha256'] == identity(root / 'model.safetensors')['sha256']
            and checkpoint['model_config_sha256'] == identity(root / 'config.json')['sha256'],
            'Retained parent weights/configuration binding differs', arm=arm)
    require(campaign['model']['transport'] == arm and campaign['batch'] == 16 and campaign['context'] == 256
            and campaign['projection_transition']['parent_optimizer_step'] == 7836
            and campaign['projection_transition']['policy'] == 'fixed_representable_range',
            'Retained parent configuration differs', arm=arm)
    require(config['quantization'] == checkpoint['quantization'], 'Parent config/checkpoint quantizer differs', arm=arm)
    result = {'root': root, 'checkpoint': checkpoint, 'campaign': campaign, 'config': config,
              'checkpoint_identity': identity(root / 'checkpoint.json'),
              'campaign_identity': identity(root / 'campaign.json')}
    PARENTS[arm] = result
    return result


def load_evaluator():
    global DEV, EVALUATOR_DOCUMENT
    path = WORK / 'docs/integration/reference-evaluator-v2.json'
    require(identity(path)['sha256'] == EVALUATOR_SHA, 'Evaluator identity differs')
    evaluator = read_json(path)
    EVALUATOR_DOCUMENT = evaluator
    source = evaluator['dev_source']
    require(identity(source['path']) == source, 'Development token store identity differs')
    tokens = read_bytes(source['path'])
    require(len(tokens) == 500000, 'Expected exactly250000 u16 inputs')
    DEV = array('H')
    DEV.frombytes(tokens)
    if sys.byteorder != 'little':
        DEV.byteswap()
    for source in [evaluator['prompt_source']] + [entry for entry in evaluator['reference_inputs'] if entry['path'].endswith('/tokenizer.json')]:
        require(identity(source['path']) == source, 'Tokenizer or prompt identity differs', path=source['path'])
    return evaluator


def load_evaluation(arm, mode, old=False):
    suffix = {'FF': 'shadow', 'QQ': 'hard'}[mode] if old else mode
    root = PRIOR / f'evaluate-{arm}-{suffix}-read-1' if old else ROOT / f'evaluate-{arm}-{mode}-1'
    label = f'{arm}/{"retained" if old else "new"}/{mode}'
    verify(root)
    report = read_json(root / 'evaluation-report.json')
    generations = read_json(root / 'generations.json')
    probes = read_json(root / 'story-probes.json')
    p = parent(arm)
    artifact, binding, evaluation = report['artifact'], report['checkpoint_binding'], report['evaluation']
    require(report['source_commit'] == (OLD_SOURCE if old else SOURCE)
            and report['executable_sha256'] == (OLD_EXE_SHA if old else EXE_SHA), 'Source/executable differs', label=label)
    require(report['evaluator_sha256'] == EVALUATOR_SHA and report['campaign'] == p['campaign'], 'Data or training parent campaign differs', label=label)
    if old and mode == 'QQ':
        packed = PRIOR / f'final-packed-{arm}-1'
        verify(packed)
        packed_sha = identity(packed / 'hard-model.json')['sha256']
        expected = dict(p['checkpoint'])
        expected['model_sha256'] = packed_sha
        expected['artifact_kind'] = 'packed_quantized_parameters_with_float_numerical_emulator'
        expected.pop('model_config_sha256')
        require(binding == expected and artifact['kind'] == 'packed_hard_export'
                and artifact['parent_checkpoint_sha256'] == p['checkpoint_identity']['sha256']
                and artifact['parent_campaign_sha256'] == p['campaign_identity']['sha256']
                and artifact['parent_checkpoint_binding'] == p['checkpoint']
                and artifact['hard_model_manifest_sha256'] == packed_sha,
                'Retained packed QQ is not exported from this exact floating parent', label=label)
    else:
        require(binding == p['checkpoint'] and artifact['kind'] == 'training_checkpoint'
                and artifact['checkpoint_sha256'] == p['checkpoint_identity']['sha256']
                and artifact['campaign_sha256'] == p['campaign_identity']['sha256'],
                'Evaluation did not load the exact floating parent', label=label)
    if old:
        require(report['mode'] == ('joint-evaluate-shadow' if mode == 'FF' else 'joint-evaluate-hard'), 'Wrong retained endpoint mode', label=label)
    else:
        require(report['mode'] == 'joint-evaluate-precision', 'Wrong new evaluation operation', label=label)
        switches = report['precision_intervention']
        require(switches == artifact['precision_intervention'] and switches['mode'] == mode
                and switches['quantize_parameters'] is (mode[0] == 'Q')
                and switches['quantize_interfaces'] is (mode[1] == 'Q')
                and switches['optimizer_updates'] == 0 and switches['parameters_mutated'] is False
                and switches['scales_recalibrated'] is False and report['parameter_projection_during_evaluation'] is False,
                'Numerical intervention declaration differs', label=label)
        require(report['executed_quantization'] == p['config']['quantization'], 'Retained quantizer metadata changed', label=label)
        require(read_json(root / 'evaluator.json') == EVALUATOR_DOCUMENT, 'Per-root evaluator document differs', label=label)
    if mode in ('QF', 'FQ'):
        require('evaluation_quantization_strength' not in report, 'Mixed mode must not claim one scalar precision strength', label=label)
    else:
        require(report.get('evaluation_quantization_strength') == {'FF': 0.0, 'QQ': 1.0}[mode], 'Precision strength metadata differs', label=label)
    require(evaluation['schema'] == 'uor-r4.joint-population-evaluation/1' and evaluation['complete'] is True
            and evaluation['mode'] == 'enabled' and evaluation['context'] == evaluation['stride'] == CONTEXT
            and evaluation['requested_batch_size'] == 16 and evaluation['completed_blocks'] == 976
            and evaluation['input_token_count'] == 250000 and evaluation['unscored_tail_targets'] == 143
            and evaluation['optimizer_steps'] == 0 and evaluation['gradients_tracked'] is False,
            'Evaluation population, batching, history or no-gradient contract differs', label=label)
    require(math.isfinite(evaluation['maximum_probability_sum_error'])
            and 0 <= evaluation['maximum_probability_sum_error'] <= 0.0002, 'Invalid normalization report', label=label)
    require(report['targets_format']['record_bytes'] == RECORD.size == 44
            and report['targets_format']['endianness'] == 'little'
            and report['targets_format']['fields'] == ['input_offset:u64', 'target:u32', 'predicted:u32',
                'target_probability:f64', 'nll_nats:f64', 'no_read_mass:f32', 'copy_gate:f32', 'top_read_position:i32(-1=None)'],
            'Target record layout differs', label=label)
    raw = read_bytes(root / 'targets.bin')
    require(len(raw) == N * RECORD.size and hashlib.sha256(raw).hexdigest() == report['targets_sha256'], 'Target size or report hash differs', label=label)
    columns = {'predicted': array('I'), 'probability': array('d'), 'nll': array('d'),
               'no_read': array('f'), 'gate': array('f'), 'read_index': array('i')}
    for i, row in enumerate(RECORD.iter_unpack(raw)):
        offset, target, predicted, probability, nll, no_read, gate, read_index = row
        require(offset == i and target == DEV[i + 1] and 0 <= predicted < 4096, 'Target alignment or prediction identity differs', label=label, record_index=i, record=dict(zip(FIELDS, row)))
        require(math.isfinite(probability) and 0 < probability <= 1.0002 and math.isfinite(nll)
                and abs(nll + math.log(probability)) < 1e-10, 'Probability/NLL invalid', label=label, record_index=i)
        require(all(math.isfinite(v) and -0.0002 <= v <= 1.0002 for v in (no_read, gate))
                and (read_index == -1 or 0 <= read_index < i % CONTEXT), 'Read metadata invalid or noncausal', label=label, record_index=i)
        for name, value in zip(columns, (predicted, probability, nll, no_read, gate, read_index)):
            columns[name].append(value)
    metrics = {}
    for name, (lo, hi) in {**PARTITIONS, **QUARTERS}.items():
        mean = math.fsum(columns['nll'][lo:hi]) / (hi - lo)
        correct = sum(columns['predicted'][i] == DEV[i + 1] for i in range(lo, hi))
        metrics[name] = {'input_offsets': [lo, hi], 'end_exclusive': True, 'targets': hi - lo,
                         'blocks': (hi - lo) // CONTEXT, 'mean_nll_nats': mean,
                         'correct': correct, 'top1_accuracy': correct / (hi - lo)}
        if name in PARTITIONS:
            stored = evaluation[name]
            require(stored['scored_targets'] == hi - lo and stored['correct'] == correct
                    and abs(mean - stored['mean_nll_nats']) < 1e-10, 'Recomputed partition differs from Rust summary', label=label, partition=name)
    require(len(generations) == 5 and len(probes) == 16, 'Generation or source-probe panel count differs', label=label)
    return {'label': label, 'root': root, 'report': report, 'raw': raw, 'columns': columns,
            'generations': generations, 'probes': probes, 'metrics': metrics,
            'binding': {'root': str(root), 'manifest': VERIFIED[str(root.resolve())],
                        'targets': identity(root / 'targets.bin'), 'generations': identity(root / 'generations.json'),
                        'story_probes': identity(root / 'story-probes.json'), 'report': identity(root / 'evaluation-report.json'),
                        'parent_checkpoint': p['checkpoint_identity'], 'source_commit': report['source_commit'],
                        'executable_sha256': report['executable_sha256'], 'artifact_kind': artifact['kind']}}


def endpoint_check(new, old):
    if new['raw'] != old['raw']:
        byte = next((i for i, (a, b) in enumerate(zip(new['raw'], old['raw'])) if a != b), min(len(new['raw']), len(old['raw'])))
        record = byte // RECORD.size
        raise AnalysisFailure('Endpoint targets.bin is not byte-exact', endpoint=new['label'], retained=old['label'],
            byte_offset=byte, record_index=record,
            new_record=dict(zip(FIELDS, RECORD.unpack_from(new['raw'], record * RECORD.size))),
            retained_record=dict(zip(FIELDS, RECORD.unpack_from(old['raw'], record * RECORD.size))),
            new_record_hex=new['raw'][record * RECORD.size:(record + 1) * RECORD.size].hex(),
            retained_record_hex=old['raw'][record * RECORD.size:(record + 1) * RECORD.size].hex(),
            new_identity=new['binding']['targets'], retained_identity=old['binding']['targets'])
    hashes = {}
    for key, filename in [('generations', 'generations.json'), ('probes', 'story-probes.json')]:
        actual, expected = strip_elapsed(new[key]), strip_elapsed(old[key])
        difference = first_json_difference(actual, expected)
        require(difference is None, 'Endpoint complete decoded-output structure differs after removing only elapsed_seconds',
                endpoint=new['label'], retained=old['label'], filename=filename, first_difference=difference)
        hashes[filename] = canonical_hash(actual)
    return {'endpoint': new['label'], 'retained': old['label'], 'targets_byte_exact': True,
            'complete_generation_structures_equal': True, 'complete_story_probe_structures_equal': True,
            'ignored_json_keys': ['elapsed_seconds (recursively; no other field ignored)'],
            'canonical_output_sha256_after_elapsed_removal': hashes,
            'new_binding': new['binding'], 'retained_binding': old['binding'],
            'scope': 'Numerical/output endpoint identity. Floating checkpoint and packed export metadata are validated separately and are intentionally not required to be identical.'}


def verify_frozen_inputs():
    path = WORK / 'docs/evidence/precision-factorial-inputs-2026-09-25.json'
    receipt = read_json(path)
    require(receipt['schema'] == 'uor-r4.precision-factorial-inputs/1' and set(receipt['arms']) == set(ARMS), 'Input receipt schema/arms differ')
    checked = []
    for arm in ARMS:
        roots = receipt['arms'][arm]['roots']
        require(set(roots) == {'shadow_parent', 'packed_parent', 'FF_reference', 'QQ_reference'}, 'Frozen input root set differs', arm=arm)
        relative_roots = {'shadow_parent': f'fit-{arm}-projected-1/checkpoint-final',
                          'packed_parent': f'final-packed-{arm}-1',
                          'FF_reference': f'evaluate-{arm}-shadow-read-1',
                          'QQ_reference': f'evaluate-{arm}-hard-read-1'}
        for label, binding in roots.items():
            # Root relocation changes no frozen file identity or model metadata.
            root = PRIOR / relative_roots[label]
            verify(root)
            actual_paths = {str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}
            require(actual_paths == set(binding['complete_file_set']), 'Frozen input complete file set changed', arm=arm, label=label,
                    only_actual=sorted(actual_paths - set(binding['complete_file_set'])),
                    only_bound=sorted(set(binding['complete_file_set']) - actual_paths))
            for relative, expected in binding['complete_file_set'].items():
                actual = identity(root / relative)
                require(actual['sha256'] == expected['sha256'] and actual['bytes'] == expected['bytes'],
                        'Frozen input file changed after evaluation', arm=arm, label=label, relative_path=relative,
                        expected=expected, actual=actual)
            checked.append({'arm': arm, 'kind': label, 'root': str(root), 'files': len(actual_paths),
                            'original_bound_root': binding['path'],
                            'manifest': identity(root / 'manifest.json')})
    require(len(checked) == 8, 'Expected eight complete retained input roots')
    return {'receipt': identity(path), 'complete_roots_checked': checked,
            'all_bound_file_sets_and_sha256_unchanged': True, 'optimizer_updates': 0}


def record_witness(item, index):
    row = RECORD.unpack_from(item['raw'], index * RECORD.size)
    return {**dict(zip(FIELDS, row)), 'target_offset': index + 1,
            'block_index': index // CONTEXT, 'position_in_block': index % CONTEXT,
            'record_hex': item['raw'][index * RECORD.size:(index + 1) * RECORD.size].hex()}


def delta_summary(values, lo, hi):
    subset = values[lo:hi]
    mean = math.fsum(subset) / len(subset)
    blocks = [math.fsum(values[i:i + CONTEXT]) / CONTEXT for i in range(lo, hi, CONTEXT)]
    block_mean = math.fsum(blocks) / len(blocks)
    variance = math.fsum((value - block_mean) ** 2 for value in blocks) / (len(blocks) - 1)
    minimum = min(range(lo, hi), key=values.__getitem__)
    maximum = max(range(lo, hi), key=values.__getitem__)
    return {'targets': hi - lo, 'blocks': len(blocks), 'mean_delta_nats': mean,
            'descriptive_block_paired_se_nats': math.sqrt(variance / len(blocks)),
            'negative_target_deltas': sum(value < 0 for value in subset),
            'zero_target_deltas': sum(value == 0 for value in subset),
            'positive_target_deltas': sum(value > 0 for value in subset),
            'mean_absolute_target_delta_nats': math.fsum(abs(value) for value in subset) / len(subset),
            'minimum_target_delta': {'input_offset': minimum, 'target': int(DEV[minimum + 1]), 'delta_nats': values[minimum]},
            'maximum_target_delta': {'input_offset': maximum, 'target': int(DEV[maximum + 1]), 'delta_nats': values[maximum]}}


def mode_pair_witness(left, right):
    names = ('record_bytes_different', 'predicted_token_different', 'target_probability_different',
             'nll_different', 'no_read_mass_different', 'copy_gate_different', 'top_read_position_different')
    counts = {partition: dict.fromkeys(names, 0) for partition in PARTITIONS}
    first_record = first_prediction = None
    a, b = left['columns'], right['columns']
    left_bytes, right_bytes = memoryview(left['raw']), memoryview(right['raw'])
    for i in range(N):
        changes = [left_bytes[i * 44:(i + 1) * 44] != right_bytes[i * 44:(i + 1) * 44]]
        changes.extend(a[field][i] != b[field][i] for field in ('predicted', 'probability', 'nll', 'no_read', 'gate', 'read_index'))
        for partition in ('full', 'tune' if i < TUNE else 'comparison'):
            for key, changed in zip(names, changes):
                counts[partition][key] += int(changed)
        if changes[0] and first_record is None:
            first_record = i
        if changes[1] and first_prediction is None:
            first_prediction = i
    witness = lambda index: None if index is None else {'input_offset': index,
        'left': record_witness(left, index), 'right': record_witness(right, index)}
    return {'left_mode': left['label'].split('/')[-1], 'right_mode': right['label'].split('/')[-1],
            'counts': counts, 'first_different_record': witness(first_record),
            'first_different_prediction': witness(first_prediction),
            'observed_record_difference': first_record is not None,
            'observed_prediction_difference': first_prediction is not None,
            'scope': 'Observed full-prefix forward records. Identical predictions would not imply identical probabilities, and a declared switch alone is not treated as an observed effect.'}


def factorial_metrics(items):
    values = {name: array('d') for name in FORMULAS}
    max_residual = 0.0
    for ff, qf, fq, qq in zip(*(items[mode]['columns']['nll'] for mode in MODES)):
        row = (qf - ff, qq - fq, fq - ff, qq - qf, math.fsum((qq, -qf, -fq, ff)), qq - ff)
        for name, value in zip(values, row):
            values[name].append(value)
        max_residual = max(max_residual, abs(row[5] - math.fsum((row[0], row[2], row[4]))))
    effects = {name: {'formula': FORMULAS[name], 'partitions': {split: delta_summary(deltas, lo, hi)
               for split, (lo, hi) in PARTITIONS.items()},
               'comparison_contiguous_quartiles': {split: delta_summary(deltas, lo, hi)
               for split, (lo, hi) in QUARTERS.items()}} for name, deltas in values.items()}
    return {'mode_metrics': {mode: item['metrics'] for mode, item in items.items()}, 'effects': effects,
            'decomposition': {'identity': 'QQ - FF = (QF - FF) + (FQ - FF) + (QQ - QF - FQ + FF)',
                              'maximum_absolute_per_target_F64_residual_nats': max_residual,
                              'scope': 'Algebraic decomposition; recorded residual reflects F64 subtraction/summation rounding, not an acceptance threshold.'},
            'mode_pair_record_and_prediction_witnesses': {a + '_vs_' + b: mode_pair_witness(items[a], items[b]) for a, b in itertools.combinations(MODES, 2)},
            'se_scope': 'Descriptive sample standard deviation of paired256-target block-mean differences divided by sqrt(number of blocks). These contiguous exposed-corpus blocks are not assumed independent; no confidence interval, significance test or population generalization is asserted.',
            'per_target_scope': 'Every aligned target contributes to each effect and change count. Original44-byte prediction records are retained and hash-bound for exact recomputation. No rounded or duplicate large per-target export is created.'}


def compact_generation(generation):
    return {key: generation[key] for key in ('schema', 'prompt', 'mode', 'seed', 'tokenizer_cid', 'sampler_policy',
        'bos_policy', 'prompt_token_ids', 'generated_token_ids', 'raw_decoded_bytes', 'raw_decoded',
        'response_text', 'utf8_decodable', 'stop', 'max_new_tokens', 'context_capacity', 'fresh_session',
        'whole_prompt_read_mode', 'gradients_tracked')}


def response_panels(items):
    headers = ('schema', 'prompt', 'mode', 'seed', 'tokenizer_cid', 'sampler_policy', 'bos_policy',
               'prompt_token_ids', 'max_new_tokens', 'context_capacity', 'fresh_session',
               'whole_prompt_read_mode', 'gradients_tracked')
    reference = items['FF']
    for mode, item in items.items():
        for i, generation in enumerate(item['generations']):
            require({key: generation[key] for key in headers} == {key: reference['generations'][i][key] for key in headers},
                    'Free-generation protocol differs across modes', mode=mode, prompt_index=i)
            require(generation['seed'] == 2014 + i and generation['fresh_session'] is True
                    and generation['gradients_tracked'] is False and generation['mode'] == 'enabled',
                    'Generation seed/fresh-state contract differs', mode=mode, prompt_index=i)
        require([row['probe'] for row in item['probes']] == [row['probe'] for row in reference['probes']],
                'Source-probe definitions/order differ across modes', mode=mode)
    rows = []
    for i, probe in enumerate(reference['probes']):
        for variant in ('original', 'edited'):
            observed = {}
            for mode, item in items.items():
                source = item['probes'][i]
                require(source['scope'] == probe['scope'] and source['stop_policy'] == probe['stop_policy']
                        and source['mode'] == 'enabled', 'Source-probe policy differs', mode=mode, probe=probe['probe']['id'])
                judged = source[variant]
                require(judged['generation']['prompt'] == probe['probe'][variant]['prompt'], 'Source prompt differs from judged generation')
                observed[mode] = {key: judged[key] for key in ('first_noun_correct', 'complete_correct')}
                observed[mode]['generation'] = compact_generation(judged['generation'])
            rows.append({'probe_id': probe['probe']['id'], 'variant': variant, **probe['probe'][variant], 'modes': observed})
    pairs = {'QF_minus_FF': ('QF', 'FF'), 'QQ_minus_FQ': ('QQ', 'FQ'), 'FQ_minus_FF': ('FQ', 'FF'),
             'QQ_minus_QF': ('QQ', 'QF'), 'QQ_minus_FF': ('QQ', 'FF')}
    changes = {}
    for name, (candidate, control) in pairs.items():
        item = {'candidate': candidate, 'control': control}
        for key in ('first_noun_correct', 'complete_correct'):
            item[key] = {
                'gains': [{'probe_id': row['probe_id'], 'variant': row['variant']} for row in rows
                          if row['modes'][candidate][key] and not row['modes'][control][key]],
                'losses': [{'probe_id': row['probe_id'], 'variant': row['variant']} for row in rows
                           if row['modes'][control][key] and not row['modes'][candidate][key]],
            }
        item['different_complete_response_rows'] = [{'probe_id': row['probe_id'], 'variant': row['variant']} for row in rows
            if row['modes'][candidate]['generation']['response_text'] != row['modes'][control]['generation']['response_text']]
        changes[name] = item
    return {'free_generations': {mode: [compact_generation(row) for row in item['generations']] for mode, item in items.items()},
            'source_probe_counts': {mode: {key: sum(row['modes'][mode][key] for row in rows) for key in ('first_noun_correct', 'complete_correct')} for mode in MODES},
            'source_probe_rows': rows, 'source_probe_changes': changes,
            'scope': 'Complete decoded responses and retained Rust oracle judgments on the same fixed prompts. This is descriptive transfer behavior, not a serving qualification or a new selection gate.'}


def code_block(text):
    fence = '`' * max(3, 1 + max((len(match.group()) for match in re.finditer(r'`+', text)), default=0))
    return [fence + 'text', text, fence]


def render(result):
    lines = ['# Fixed-parent precision factorial', '',
             'First letter: parameters; second letter: all declared interfaces. F uses the retained floating shadows/continuous interfaces; Q uses the already frozen grids.', '',
             'All modes replay complete prefixes with their own endogenous recurrent states and read/write histories. The differences measure the whole forward intervention; they do not isolate a local layer error or hold hidden trajectories fixed.', '',
             'Costs below are changes in NLL (nats per target); negative means lower NLL. Interaction is a signed departure from additivity. No mixed mode is a serving candidate or qualification.', '',
             'Comparison quartiles are contiguous corpus segments of228 blocks/58,368 targets, after the64 tune blocks. They are not within-window position quarters.', '',
             'Block-paired standard errors are descriptive only: contiguous corpus blocks are not assumed independent, and no population inference or new acceptance threshold is applied.']
    for arm, data in result['arms'].items():
        lines += ['', '## ' + arm, '', '| Mode | Tune NLL | Comparison NLL | Full NLL | Comparison top1 |',
                  '|---|---:|---:|---:|---:|']
        for mode in MODES:
            metrics = data['factorial']['mode_metrics'][mode]
            lines.append(f"| {mode} | {metrics['tune']['mean_nll_nats']:.12f} | {metrics['comparison']['mean_nll_nats']:.12f} | {metrics['full']['mean_nll_nats']:.12f} | {metrics['comparison']['top1_accuracy']:.9f} |")
        lines += ['', '| Effect | Formula | Comparison delta | Descriptive block SE |', '|---|---|---:|---:|']
        for name, effect in data['factorial']['effects'].items():
            metric = effect['partitions']['comparison']
            lines.append(f"| {name} | {effect['formula']} | {metric['mean_delta_nats']:+.12f} | {metric['descriptive_block_paired_se_nats']:.12f} |")
        lines += ['', '| Contiguous quarter | FF | QF | FQ | QQ | Interaction |', '|---|---:|---:|---:|---:|---:|']
        for quarter in QUARTERS:
            means = [data['factorial']['mode_metrics'][mode][quarter]['mean_nll_nats'] for mode in MODES]
            interaction = data['factorial']['effects']['interaction']['comparison_contiguous_quartiles'][quarter]['mean_delta_nats']
            lines.append('| ' + quarter + ' | ' + ' | '.join(f'{value:.12f}' for value in means) + f' | {interaction:+.12f} |')
        lines += ['', '### Actual mode differences', '', '| Pair | Different comparison records | Different predicted IDs | Different target probabilities |', '|---|---:|---:|---:|']
        for pair, witness in data['factorial']['mode_pair_record_and_prediction_witnesses'].items():
            counts = witness['counts']['comparison']
            lines.append(f"| {pair} | {counts['record_bytes_different']} | {counts['predicted_token_different']} | {counts['target_probability_different']} |")
        panels = data['responses']
        lines += ['', '### Complete free generations', '', 'The text below is the complete recorded raw decoded continuation. Stop reasons and token IDs are retained; no output repair or truncation is applied.']
        for index in range(5):
            lines += ['', f'#### Prompt {index + 1}', '']
            lines += code_block(panels['free_generations']['FF'][index]['prompt'])
            for mode in MODES:
                generation = panels['free_generations'][mode][index]
                lines += ['', f"**{mode}**, seed{generation['seed']}, stop={json.dumps(generation['stop'], ensure_ascii=False)}, UTF8={generation['utf8_decodable']}", '']
                lines += code_block(generation['raw_decoded'])
                lines += ['', 'Generated token IDs: `' + json.dumps(generation['generated_token_ids']) + '`']
                if generation['response_text'] != generation['raw_decoded']:
                    lines += ['', 'Recorded response_text (as separately stored by the evaluator):', ''] + code_block(generation['response_text'])
        lines += ['', '### Source-probe rows', '', '| Mode | First noun correct /32 | Complete correct /32 |', '|---|---:|---:|']
        for mode in MODES:
            counts = panels['source_probe_counts'][mode]
            lines.append(f"| {mode} | {counts['first_noun_correct']} | {counts['complete_correct']} |")
        for row in panels['source_probe_rows']:
            lines += ['', f"#### {row['probe_id']} / {row['variant']}", '',
                      'Accepted continuations: `' + json.dumps(row['accepted_continuations'], ensure_ascii=False) + '`', '']
            lines += code_block(row['prompt'])
            for mode in MODES:
                observed = row['modes'][mode]
                generation = observed['generation']
                lines += ['', f"**{mode}**: first_noun_correct={observed['first_noun_correct']}; complete_correct={observed['complete_correct']}; stop={json.dumps(generation['stop'], ensure_ascii=False)}", '']
                lines += code_block(generation['raw_decoded'])
                if generation['response_text'] != generation['raw_decoded']:
                    lines += ['', 'Recorded response_text:', ''] + code_block(generation['response_text'])
    lines += ['', '## Evidence and limits', '',
              'The JSON result binds every original prediction file and provides per-target sign counts, extrema, first mode differences, source-row gains/losses and block-paired summaries. Original targets remain the exact recomputation source; no lossy duplicate per-target export is created.', '',
              'FF and QQ must reproduce the retained shadow and packed endpoints exactly in targets.bin and in complete generations/probe JSON after removing only elapsed_seconds. Metadata for a floating parent and packed export is validated according to its own artifact type.', '',
              'This evaluation uses the exposed development population, fixed final step8348 and one paired seed. It performs no retraining, recalibration or checkpoint selection. Results do not qualify integer serving, general language, geometric advantage or energy savings.']
    return '\n'.join(lines) + '\n'


def save_new(path, content):
    require(path.parent.resolve() == ROOT, 'Analysis output must stay outside sealed input roots')
    with path.open('x') as stream:
        stream.write(content)


def main():
    global EXE, EXE_SHA, SOURCE, ROOT, PRIOR, WORK
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--endpoints-only', action='store_true')
    parser.add_argument('--root', type=Path, default=ROOT,
                        help='Existing unsealed directory containing the new evaluations and receiving exclusive outputs')
    parser.add_argument('--prior', type=Path, default=PRIOR,
                        help='Retained projected campaign directory; complete frozen file identities remain required')
    parser.add_argument('--worktree', type=Path, default=WORK,
                        help='Worktree containing the tracked evaluator and factorial input receipt')
    args = parser.parse_args()
    ROOT, PRIOR, WORK = (path.resolve(strict=True) for path in (args.root, args.prior, args.worktree))
    require(all(path.is_dir() for path in (ROOT, PRIOR, WORK)), 'All three evidence paths must be existing directories')
    require(not (ROOT / 'manifest.json').exists(), 'Analysis output root must not be sealed', root=str(ROOT))
    EXE = args.exe.resolve(strict=True)
    EXE_SHA = identity(EXE)['sha256']
    SOURCE = args.source_commit
    require(re.fullmatch(r'[0-9a-f]{40}', SOURCE) is not None, 'A full lowercase source commit SHA is required')
    endpoint_path = ROOT / 'precision-factorial-endpoints.json'
    result_path = ROOT / 'precision-factorial-result.json'
    markdown_path = ROOT / 'precision-factorial-generations-and-probes.md'
    expected_new = [endpoint_path] if args.endpoints_only else [result_path, markdown_path]
    require(not any(path.exists() for path in expected_new), 'Refusing to overwrite an existing analysis output', paths=[str(path) for path in expected_new])
    if not args.endpoints_only:
        require(endpoint_path.is_file(), 'Run --endpoints-only successfully before the full factorial analysis')
    evaluator = load_evaluator()
    inputs = verify_frozen_inputs()
    fresh, endpoints = {}, []
    for arm in ARMS:
        fresh[arm] = {}
        for mode in ('FF', 'QQ'):
            fresh[arm][mode] = load_evaluation(arm, mode)
            endpoints.append(endpoint_check(fresh[arm][mode], load_evaluation(arm, mode, old=True)))
    endpoint_receipt = {'schema': 'uor-r4.precision-factorial-endpoints/1',
        'status': 'PASS_EXACT_RETAINED_ENDPOINTS', 'recorded_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
        'source_commit': SOURCE, 'verification_executable': identity(EXE), 'analysis_script': identity(Path(__file__)),
        'targets_per_endpoint': N, 'checks': endpoints, 'retained_inputs': inputs,
        'scope': 'Endpoint integrity only; no mixed-mode result or model-quality acceptance is inferred.'}
    if args.endpoints_only:
        save_new(endpoint_path, json.dumps(endpoint_receipt, indent=2, allow_nan=False) + '\n')
        print(json.dumps({'endpoint_receipt': str(endpoint_path), 'status': endpoint_receipt['status'], 'checked_endpoints': len(endpoints)}))
        return
    prior = read_json(endpoint_path)
    require(prior['status'] == 'PASS_EXACT_RETAINED_ENDPOINTS' and prior['source_commit'] == SOURCE
            and prior['verification_executable'] == identity(EXE) and prior['checks'] == endpoints,
            'Endpoints changed since the prior gate', prior_receipt=str(endpoint_path))
    result = {'schema': 'uor-r4.precision-factorial-analysis/1', 'status': 'DESCRIPTIVE_ANALYSIS_COMPLETE',
              'principal_interpretation': 'REQUIRED; no automatic candidate selection or serving promotion',
              'recorded_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
              'source_commit': SOURCE, 'verification_executable': identity(EXE), 'analysis_script': identity(Path(__file__)),
              'endpoint_receipt': identity(endpoint_path), 'rechecked_endpoints': endpoints,
              'retained_inputs': inputs, 'evaluator_sha256': EVALUATOR_SHA, 'dev_source': evaluator['dev_source'],
              'targets_format': {'struct': '<QIIddffi', 'record_bytes': 44, 'fields': list(FIELDS)},
              'partitions': {name: {'input_offsets': [lo, hi], 'end_exclusive': True, 'targets': hi - lo} for name, (lo, hi) in PARTITIONS.items()},
              'comparison_contiguous_quartiles': {name: {'input_offsets': [lo, hi], 'end_exclusive': True, 'targets': hi - lo, 'blocks': 228} for name, (lo, hi) in QUARTERS.items()},
              'formulas_nll_nats': FORMULAS,
              'mode_definition': {'first_letter': 'parameter precision', 'second_letter': 'all declared interface precision',
                                  'F': 'retained floating shadow / continuous interface', 'Q': 'unchanged frozen dyadic grid'},
              'intervention_scope': 'Both retained final projected parents; no optimizer updates, recalibration or checkpoint selection. Each mode replays its own complete endogenous recurrent/read/write history. Conditional costs include all resulting trajectory/attention/output feedback; they are not independent local error budgets.',
              'serving_scope': 'All four paths are floating numerical evaluations. Neither mixed mode is serving qualified. Exposed development and fixed response panels do not establish general language, geometric advantage or energy savings.',
              'arms': {}}
    for arm in ARMS:
        for mode in ('QF', 'FQ'):
            fresh[arm][mode] = load_evaluation(arm, mode)
        items = {mode: fresh[arm][mode] for mode in MODES}
        result['arms'][arm] = {'parent_checkpoint': parent(arm)['checkpoint_identity'],
                               'evaluations': {mode: item['binding'] for mode, item in items.items()},
                               'factorial': factorial_metrics(items), 'responses': response_panels(items)}
    # Check immutable controls/parents again after all analysis reads; no writes
    # have been made beneath those roots or to any model parameter file.
    result['retained_inputs_after_analysis'] = verify_frozen_inputs()
    result['verified_report_roots'] = list(VERIFIED.values())
    result['input_identities'] = list(EVIDENCE.values())
    markdown = render(result)
    encoded = json.dumps(result, indent=2, ensure_ascii=False, allow_nan=False) + '\n'
    # Claim both names exclusively before writing either result; partial output
    # on an I/O failure remains evidence and is never silently overwritten.
    with result_path.open('x') as json_file, markdown_path.open('x') as markdown_file:
        json_file.write(encoded)
        markdown_file.write(markdown)
    print(json.dumps({'result': str(result_path), 'complete_generation_panel': str(markdown_path),
                      'targets_per_mode': N, 'arms': {arm: {name: effect['partitions']['comparison']['mean_delta_nats']
                      for name, effect in result['arms'][arm]['factorial']['effects'].items()} for arm in ARMS}}, indent=2))


if __name__ == '__main__':
    try:
        main()
    except Exception as error:
        evidence = error.evidence if isinstance(error, AnalysisFailure) else {'message': str(error), 'type': type(error).__name__}
        failure = ROOT / ('precision-factorial-analysis-failure-' + uuid.uuid4().hex + '.json')
        receipt = {'schema': 'uor-r4.precision-factorial-analysis-failure/1', 'status': 'FAILED_NO_ATTRIBUTION',
                   'recorded_utc': dt.datetime.now(dt.timezone.utc).isoformat(), 'evidence': evidence,
                   'inputs_hashed_before_failure': list(EVIDENCE.values()),
                   'scope': 'Failure is retained; no mixed-mode interpretation or acceptance is authorized by this attempt.'}
        with failure.open('x') as stream:
            json.dump(receipt, stream, indent=2, allow_nan=False)
            stream.write('\n')
        print(json.dumps({'failure_receipt': str(failure), 'error': evidence}, allow_nan=False), file=sys.stderr)
        raise SystemExit(1)
