#!/usr/bin/env python3
"""Positive-only protected joint construction config factory. Runs only normative report verification.
No model, gradient, proposal, arithmetic audit, or evaluator is invoked here.
"""
import copy
import hashlib
import json
import pathlib
import subprocess
import sys
import time

BASE = pathlib.Path('/workspace/uor-r4/codex/sol-protected-joint-20261009')
CONFIG = BASE / 'configs/protected-joint-0001-attempt3.json'
SOURCE = json.loads((BASE / 'source.json').read_text())['commit']
# The admission receipt is authored after exact input verification and before model execution.
EXPECTED_CONFIG_SHA = json.loads((BASE / 'evidence/input-verification.json').read_text())['config_sha256']
BINARY = BASE / 'runtime/geometric-frozen-map-fit'
MODEL = BASE / 'runs/protected-joint-0001-attempt3'
OBSERVATION = BASE / 'observations/protected-joint-0001-attempt3'
CHECKPOINT = MODEL / 'checkpoint-0001'
OUTCONFIG = BASE / 'configs/actual9-0001-attempt3.json'
OUTPUT = BASE / 'qualifications/actual9-0001-attempt3'
DECISION = BASE / 'evidence/actual9-config-decision.json'
VERIFIER = pathlib.Path('/workspace/uor-r4/codex/sol-intermediate-word-boundary/publications/runtime-76e10b53f7-attempt2/native-reached-prefix-attribution')
VERIFIER_SHA = 'd90411116c702dc4149fc055706f34c276c1ead29c5b0ec95600c30cc976326d'


def need(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b''):
            digest.update(chunk)
    return digest.hexdigest()


def read(path):
    with path.open('r', encoding='utf-8') as stream:
        return json.load(stream)


def write_exclusive(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def admitted_runtime():
    runtime = read(BASE / 'runtime/runtime-identity.json')
    upload = read(BASE / 'runtime-preservation.json')
    need(runtime['source_commit'] == upload['source_commit'] == SOURCE and
         runtime['source_snapshot_verified'] is True and upload['status'] == 'PASS' and
         upload['canonical_network_volume'] is True,
         'durable exact-source runtime admission incomplete')
    need(runtime['config_sha256'] == EXPECTED_CONFIG_SHA and
         runtime['donor_credit'] == 'full_pool_utility' and runtime['coordinates'] == 1920 and
         runtime['prefix_transaction'] == 'protected_joint_vector' and
         runtime['active_families'] == ['prefix.coefficients', 'generate.unary'],
         'prospective original-parent credit/runtime policy differs')
    for key, path in [('binary_sha256', BINARY), ('config_sha256', CONFIG)]:
        need(runtime[key] == upload[key] == sha(path), 'runtime/config bytes differ')
    for name in ['prepare-actual9.py', 'run-protected-joint.py']:
        entry = runtime['files'][name]
        path = BASE / name
        need(entry['sha256'] == sha(path) and entry['bytes'] == path.stat().st_size,
             'preserved operational helper identity differs')
        need(sha(BASE / 'runtime' / name) == entry['sha256'],
             'preserved runtime helper bytes differ')
    need(sha(VERIFIER) == VERIFIER_SHA, 'immutable normative verifier differs')
    return runtime, upload


def main():
    need(len(sys.argv) == 1, 'factory takes no overrides')
    need(not OUTCONFIG.exists() and not OUTPUT.exists() and not DECISION.exists(),
         'fresh actual9 config/output/decision required')
    runtime, upload = admitted_runtime()
    CONFIG_SHA, BINARY_SHA = runtime['config_sha256'], runtime['binary_sha256']
    config = read(CONFIG)
    learning = config['coupled_episode_learning']
    need(learning['donor_credit'] == 'full_pool_utility' and
         learning['prefix_transaction'] == 'protected_joint_vector' and
         learning.get('retained_gradient') is None and learning.get('retained_export') is None and
         config.get('generate_episode_learning') is None and
         config.get('prefix_artifact_check') is None and
         pathlib.Path(config['out']) == MODEL and config['mode'] == 'joint_continuation' and
         config['updates'] == 1 and config['loss_scope'] == 'all',
         'fresh original-parent full-pool donor policy differs')
    verification = []
    for root in [MODEL, OBSERVATION]:
        start = time.monotonic()
        argv = [str(VERIFIER), 'verify-report', str(root)]
        result = subprocess.run(argv, check=True, capture_output=True, text=True)
        verification.append({'root': str(root), 'argv': argv, 'exit_code': result.returncode,
                             'seconds': time.monotonic() - start, 'stdout': result.stdout,
                             'stderr': result.stderr, 'manifest_sha256': sha(root / 'manifest.json')})
    need(sha(MODEL / 'config.json') == CONFIG_SHA, 'sealed model config bytes differ')
    report = read(MODEL / 'report.json')
    attempt = read(MODEL / 'attempt.json')
    binding = read(MODEL / 'external-config-binding.json')
    launch = read(OBSERVATION / 'launch.json')
    execution = read(OBSERVATION / 'execution.json')
    need(attempt['schema'] == 'uor-r4.report-attempt/1' and
         attempt['argv'] == launch['argv'] == [str(BINARY), str(CONFIG)] and
         attempt['pid'] == launch['pid'] and launch['binary'] == str(BINARY) and
         launch['started_utc'] == execution['started_utc'] and execution['exit_code'] == 0 and
         launch['binary_sha256'] == execution['binary_sha256'] == BINARY_SHA and
         launch['config_sha256'] == execution['config_sha256'] == CONFIG_SHA and
         launch['lane'] == execution['lane'] == 'cuda-learning', 'actual learning launch differs')
    need(binding == {'path': str(CONFIG.resolve()), 'sha256': CONFIG_SHA,
                     'bytes': CONFIG.stat().st_size, 'attempt_argv': attempt['argv']},
         'external config binding differs')
    need(report['status'] == 'COMPLETED' and report['mode'] == 'coupled_episode_learning' and
         report['source_commit'] == SOURCE and report['selected_model'] is False and
         report['weighted_roles'] == 32 and report['unique_objective_frames'] == 31 and
         report['physical_backward_calls'] == 31 and report['new_backward_calls'] == 411 and
         report['protected_margin_backward_calls'] == 380 and
         report['total_fresh_training_backward_calls'] == 411 and
         report['extracted_family_gradients'] == 2 and report['fresh_gradient_files'] == 62 and
         report['new_training_graph_forwards'] == 822 and report['new_constructor_calls'] == 1 and
         report['new_order_selection_calls'] == 1 and report['candidate_native_steps'] == 391 and
         report['expected_final_pool_reductions'] == 391 and report['optimizer_updates'] == 0 and
         report['all_original380_preserved'] is True and report['parent_master_bits_restored'] is True and
         report['constructor_restart_from_original'] is False and report['export_completion_only'] is False and
         report['inherited_learning'] is None and report['inherited_construction'] is None,
         'completed fresh coupled construction/reload scope differs')
    need(report['policy']['donor_credit'] == 'full_pool_utility' and
         report['policy']['prefix_transaction'] == 'protected_joint_vector' and
         report['policy']['active'] == ['prefix.coefficients', 'generate.unary'] and
         report['active_scalars_per_family'] == 960 and
         report['candidate_receipt']['native_independently_reloaded'] is True and
         report['candidate_receipt']['active_parameter_names'] == ['prefix.coefficients', 'generate.unary'],
         'completed credit/active-family/reload authority differs')
    gradient = read(MODEL / 'coupled-gradient-receipt.json')
    utility_file = MODEL / 'coupled-full-pool-donor-utilities.json'
    utility = read(utility_file)
    need(gradient['donor_credit'] == utility['donor_credit'] == 'full_pool_utility' and
         gradient['prefix_transaction'] == 'protected_joint_vector' and
         gradient['physical_backward_calls'] == 31 and gradient['weighted_roles'] == 32 and
         len(gradient['perterm']) == 31 and gradient['full_pool_utility_file'] == utility_file.name and
         gradient['full_pool_utility_sha256'] == sha(utility_file) and
         utility['physical_frames'] == 31 and utility['all_before_any_backward'] is True,
         'fresh full-pool utility/gradient authority differs')
    margin_file = MODEL / 'protected-margin-receipt.json'
    margin = read(margin_file)
    need(gradient['protected_margin_backward_calls'] == 380 and
         gradient['total_fresh_backward_calls'] == 411 and
         gradient['total_training_graph_forwards'] == 822 and
         gradient['protected_margin_receipt_sha256'] == sha(margin_file) and
         margin['schema'] == 'uor-r4.protected-margin-jacobians/1' and
         margin['backward_calls'] == 380 and margin['physical_gradients_shape'] == [380, 1920] and
         margin['protected_weight'] == 1 and margin['objective_guard_CE_weight'] == 0 and
         margin['all380_parity_before_any_backward'] is True and len(margin['terms']) == 380,
         'protected margin gradient authority differs')
    for index, term in enumerate(margin['terms']):
        leaf = f'protected-margin-{index:03}.f32le'
        need(term['guard_index'] == index and term['file'] == leaf and
             term['shape'] == [1920] and term['status'] == 'PRESENT' and
             (MODEL / leaf).stat().st_size == 7680 and sha(MODEL / leaf) == term['sha256'],
             'protected raw gradient identity differs')
    decision = {'schema': 'uor-r4.protected-joint-actual9-config-decision/1',
                'candidate_report_sha256': sha(MODEL / 'report.json'),
                'candidate_manifest_sha256': sha(MODEL / 'manifest.json'),
                'model_source_commit': SOURCE, 'binary_sha256': BINARY_SHA,
                'model_config_sha256': CONFIG_SHA, 'verification': verification,
                'verifier_sha256': VERIFIER_SHA, 'actual9_executed': False,
                'canonical': 'NOT_RUN', 'full512': 'NOT_RUN',
                'arithmetic_audit_prerequisite': False, 'donor_credit': 'full_pool_utility',
                'prefix_transaction': 'protected_joint_vector',
                'runtime_preservation_source': upload['source_commit'], 'runtime_identity_sha256': sha(BASE / 'runtime/runtime-identity.json')}
    positive = report['finite_episode_positive']
    need(type(positive) is bool and type(report['final_gate']['passed']) is bool and
         positive == report['final_gate']['passed'], 'construction decision absent/conflicting')
    if not positive:
        decision.update(status='NOT_RUN_CONSTRUCTION_NEGATIVE', cheap9_config_written=False)
        write_exclusive(DECISION, decision)
        print(json.dumps(decision))
        return
    gate = report['final_gate']
    need(all(gate[key] is True for key in ['strict_combined_descent', 'strict_episode_descent',
             'all15_conditional_winners', 'all17_references', 'all380_original_winners', 'passed']) and
         report['candidate_objective']['all_phase_winners'] is True and
         report['candidate_objective']['correct_reference_frames'] == 17,
         'positive complete episode/reference/guard gate differs')
    phases = report['candidate_objective']['phases']
    need(len(phases) == 15 and [phase['position'] for phase in phases] == list(range(15)) and
         all(phase['target'] == phase['chosen'] for phase in phases), 'positive fifteen-phase scope differs')
    master = read(CHECKPOINT / 'generate-source/metadata.json')['parameters']['generate.unary']
    master_file = CHECKPOINT / 'generate-source/generate.unary.f32le'
    need(master['shape'] == [8, 120] and master['bytes'] == 3840 and
         master_file.stat().st_size == 3840 and sha(master_file) == master['sha256'],
         'Generate unary actual fractional master identity differs')
    prefix_master = CHECKPOINT / 'prefix/prefix-source-f32.bin'
    prefix_packed = CHECKPOINT / 'prefix/prefix-q4.bin'
    need(prefix_master.stat().st_size == 3840 and prefix_packed.stat().st_size == 480,
         'Prefix actual master/packed dimensions differ')
    original = pathlib.Path(learning['original_inputs']['retained_intermediate_root']) / 'checkpoint-0001'
    u_leaf = pathlib.Path('continuation-source/continuation.unary.f32le')
    need((CHECKPOINT / u_leaf).stat().st_size == (original / u_leaf).stat().st_size and
         sha(CHECKPOINT / u_leaf) == sha(original / u_leaf), 'frozen actual U master differs')
    pins = {'donor_credit': 'full_pool_utility', 'prefix_transaction': 'protected_joint_vector',
            'expected_report_sha256': decision['candidate_report_sha256'],
            'expected_manifest_sha256': decision['candidate_manifest_sha256'],
            'expected_generate_sha256': sha(CHECKPOINT / 'generate.bin'),
            'expected_unary_master_sha256': master['sha256'],
            'expected_prefix_packed_sha256': sha(prefix_packed),
            'expected_prefix_master_sha256': sha(prefix_master),
            'expected_continuation_sha256': sha(CHECKPOINT / 'continuation-field.bin')}
    decision['frozen_u_master_sha256'] = sha(CHECKPOINT / u_leaf)
    actual = copy.deepcopy(config)
    actual.pop('coupled_episode_learning')
    actual['checkpoint'] = str(CHECKPOINT)
    actual['out'] = str(OUTPUT)
    actual['maximum_report_bytes'] = 256 * 1024**2
    actual['prefix_artifact_check'] = {
        'retained_candidate_root': str(MODEL),
        'retained_intermediate_root': learning['original_inputs']['retained_intermediate_root'],
        'coupled_episode_candidate': pins}
    # Every input/oracle/control setting remains byte-value-identical to the admitted config.
    # Rust artifact dispatch validates complete coupled credit/frozen-master authority before inference.
    write_exclusive(OUTCONFIG, actual)
    decision.update(status='CONFIG_PREPARED_NOT_EVALUATED', cheap9_config_written=True,
                    config=str(OUTCONFIG), config_sha256=sha(OUTCONFIG), output=str(OUTPUT),
                    authority=pins, qualification='original8 typed complete/EOS retained AND task typed wholeanswer/EOS',
                    evaluator_source=SOURCE)
    write_exclusive(DECISION, decision)
    print(json.dumps(decision))


if __name__ == '__main__':
    main()
