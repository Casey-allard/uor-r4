#!/usr/bin/env python3
"""Positive-only pair config factory. Runs only normative report verification.
No model, gradient, proposal, arithmetic audit, or evaluator is invoked here.
"""
import copy
import hashlib
import json
import pathlib
import subprocess
import sys
import time

BASE = pathlib.Path('/workspace/uor-r4/codex/sol-bounded-pair-20261009')
CONFIG = BASE / 'configs/pair-0001-attempt1.json'
CONFIG_SHA = '7c15dd4956770c0f008644f8ac4b99d60b6b2bf085847377d0fadbfa41c45740'
SOURCE = 'a66ecef50848cc11160fc0c5a1f188355731ff98'
BINARY = BASE / 'runtime/geometric-frozen-map-fit'
BINARY_SHA = 'f37b39134e865f6892c8cf0e5229376d1bcd9440ff3b6b89405b2ba73f702d63'
MODEL = BASE / 'runs/pair-0001-attempt1'
OBSERVATION = BASE / 'observations/pair-0001-attempt1'
CHECKPOINT = MODEL / 'checkpoint-0001'
OUTCONFIG = BASE / 'configs/actual9-0001-attempt1.json'
OUTPUT = BASE / 'qualifications/actual9-0001-attempt1'
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


def main():
    need(len(sys.argv) == 1, 'factory takes no overrides')
    need(not OUTCONFIG.exists() and not OUTPUT.exists() and not DECISION.exists(),
         'fresh actual9 config/output/decision required')
    need(sha(CONFIG) == CONFIG_SHA and sha(BINARY) == BINARY_SHA and
         sha(VERIFIER) == VERIFIER_SHA, 'admitted config/binary/verifier changed')
    config = read(CONFIG)
    learning = config['generate_episode_learning']
    need(learning['family'] == 'pair' and type(learning['maximum_coordinates']) is int and
         learning['maximum_coordinates'] == 960 and pathlib.Path(config['out']) == MODEL and
         config['mode'] == 'joint_continuation' and config['updates'] == 1 and
         config['loss_scope'] == 'all', 'prospective pair policy changed')
    runtime = read(BASE / 'runtime/runtime-identity.json')
    need(runtime['source_commit'] == SOURCE and runtime['binary_sha256'] == BINARY_SHA and
         runtime['config_sha256'] == CONFIG_SHA, 'runtime identity differs')
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
    need(report['status'] == 'COMPLETED' and report['mode'] == 'generate_pair_episode_learning' and
         report['source_commit'] == SOURCE and report['selected_model'] is False and
         report['weighted_roles'] == 32 and report['unique_objective_frames'] == 31 and
         report['generate_backward_calls'] == 31 and report['prefix_backward_calls'] == 0 and
         report['candidate_native_steps'] == 391 and report['optimizer_updates'] == 0 and
         report['all_original380_preserved'] is True and report['parent_master_bits_restored'] is True,
         'completed pair construction/reload scope differs')
    need(report['policy']['selected_family'] == 'pair' and
         report['policy']['coordinate_count'] == 960 and
         report['policy']['full_family_coordinate_count'] == 57600 and
         report['policy']['active_parameter_names'] == ['generate.pair'],
         'completed pair family/budget differs')
    decision = {'schema': 'uor-r4.pair-actual9-config-decision/1',
                'candidate_report_sha256': sha(MODEL / 'report.json'),
                'candidate_manifest_sha256': sha(MODEL / 'manifest.json'),
                'model_source_commit': SOURCE, 'binary_sha256': BINARY_SHA,
                'model_config_sha256': CONFIG_SHA, 'verification': verification,
                'verifier_sha256': VERIFIER_SHA, 'actual9_executed': False,
                'canonical': 'NOT_RUN', 'full512': 'NOT_RUN',
                'arithmetic_audit_prerequisite': False}
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
    master = read(CHECKPOINT / 'generate-source/metadata.json')['parameters']['generate.pair']
    master_file = CHECKPOINT / 'generate-source/generate.pair.f32le'
    need(master['shape'] == [4, 120, 120] and master['bytes'] == 230400 and
         master_file.stat().st_size == 230400 and sha(master_file) == master['sha256'],
         'pair actual fractional master identity differs')
    pins = {'family': 'pair', 'maximum_coordinates': 960,
            'expected_report_sha256': decision['candidate_report_sha256'],
            'expected_manifest_sha256': decision['candidate_manifest_sha256'],
            'expected_generate_sha256': sha(CHECKPOINT / 'generate.bin'),
            'expected_pair_master_sha256': master['sha256'],
            'expected_continuation_sha256': sha(CHECKPOINT / 'continuation-field.bin')}
    actual = copy.deepcopy(config)
    actual.pop('generate_episode_learning')
    actual['checkpoint'] = str(CHECKPOINT)
    actual['out'] = str(OUTPUT)
    actual['maximum_report_bytes'] = 256 * 1024**2
    actual['prefix_artifact_check'] = {
        'retained_candidate_root': str(MODEL),
        'retained_intermediate_root': learning['original_inputs']['retained_intermediate_root'],
        'generate_episode_candidate': pins}
    # Every input/oracle/control setting remains byte-value-identical to the admitted config.
    # Rust artifact dispatch validates complete family/rank/frozen-master authority before inference.
    write_exclusive(OUTCONFIG, actual)
    decision.update(status='CONFIG_PREPARED_NOT_EVALUATED', cheap9_config_written=True,
                    config=str(OUTCONFIG), config_sha256=sha(OUTCONFIG), output=str(OUTPUT),
                    authority=pins, qualification='original8 typed complete/EOS retained AND task typed wholeanswer/EOS',
                    evaluator_source=SOURCE)
    write_exclusive(DECISION, decision)
    print(json.dumps(decision))


if __name__ == '__main__':
    main()
