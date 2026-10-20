#!/usr/bin/env python3
"""Admitted original-parent full-pool donor run and immediate positive-only actual9.
This orchestrator calls the pinned supervisor; it contains no model implementation.
"""
import datetime
import importlib.util
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import time

BASE = pathlib.Path('/workspace/uor-r4/codex/sol-full-donor-20261009')
SOURCE = '4f7eee35b250b6d5bf9e250fc0ea00d356998695'
BINARY = BASE / 'runtime/geometric-frozen-map-fit'
CONFIG = BASE / 'configs/full-donor-0001-attempt1.json'
FACTORY = BASE / 'prepare-full-donor-actual9.py'
MODEL = BASE / 'runs/full-donor-0001-attempt1'
OBSERVATION = BASE / 'observations/full-donor-0001-attempt1'
ACTUAL_OBSERVATION = BASE / 'observations/actual9-0001-attempt1'
SUPERVISOR = pathlib.Path('/workspace/uor-r4/codex/sol-sequence-progress-causal/capture-supervise.py')
SUPERVISOR_SHA = '393532acefc51e80d52952f3a1a367785523fade6a755b16439ffc902d836115'


def main():
    if len(sys.argv) != 1:
        raise ValueError('runner takes no overrides')
    with (BASE / 'runtime/runtime-identity.json').open() as stream:
        identity = json.load(stream)
    entry = identity['files'][FACTORY.name]
    if (identity['source_commit'] != SOURCE or
        hashlib.sha256(FACTORY.read_bytes()).hexdigest() != entry['sha256'] or
        FACTORY.stat().st_size != entry['bytes']):
        raise ValueError('preserved factory identity differs before import')
    spec = importlib.util.spec_from_file_location('full_donor_actual9', FACTORY)
    if spec is None or spec.loader is None:
        raise ValueError('qualification factory cannot be loaded')
    factory = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(factory)
    need, sha, read, write = factory.need, factory.sha, factory.read, factory.write_exclusive
    runtime, upload = factory.admitted_runtime()
    need(runtime['source_commit'] == SOURCE and sha(SUPERVISOR) == SUPERVISOR_SHA,
         'source/supervisor identity differs')
    config_sha, binary_sha = runtime['config_sha256'], runtime['binary_sha256']
    config = read(CONFIG)
    learning = config['coupled_episode_learning']
    need(learning['donor_credit'] == 'full_pool_utility' and
         learning.get('retained_gradient') is None and learning.get('retained_export') is None and
         config.get('generate_episode_learning') is None and config.get('prefix_artifact_check') is None and
         pathlib.Path(config['out']) == MODEL and config['mode'] == 'joint_continuation' and
         config['updates'] == 1 and config['loss_scope'] == 'all',
         'fresh original-parent coupled policy differs')
    input_verification = read(BASE / 'evidence/input-verification.json')
    need(input_verification['status'] == 'PASS' and input_verification['config_sha256'] == config_sha,
         'authenticated input recovery incomplete')
    need(type(config['maximum_report_bytes']) is int and 0 < config['maximum_report_bytes'] <= 512 << 20,
         'model report cap differs')
    # Whole-task storage admission is recorded by the owner before invocation.
    # This additional live check protects the next report, temporary files and stop margin.
    required_free = config['maximum_report_bytes'] + (256 << 20) + (128 << 20)
    need(shutil.disk_usage(BASE).free > required_free, 'live report/temp/stop-margin disk boundary')
    need(not any(p.exists() for p in [MODEL, OBSERVATION, ACTUAL_OBSERVATION,
         factory.OUTCONFIG, factory.OUTPUT, factory.DECISION, BASE / 'execution-result.json']),
         'fresh learning/qualification/report roots required')
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    start = time.monotonic()
    write(BASE / 'execution-admission.json', {
        'source_commit': SOURCE, 'binary_sha256': binary_sha, 'config_sha256': config_sha,
        'runtime_revision': upload['revision'],
        'runtime_identity_sha256': sha(BASE / 'runtime/runtime-identity.json'),
        'factory_sha256': sha(FACTORY), 'runner_sha256': sha(BASE / 'run-full-donor.py'),
        'supervisor_sha256': SUPERVISOR_SHA, 'started_utc': started,
        'donor_credit': 'full_pool_utility',
        'active_parameter_names': ['prefix.coefficients', 'generate.unary'],
        'active_scalars_per_family': 960, 'physical_backward_calls': 31,
        'inherited_science': False, 'cpu_threads': 2,
        'model_process_ram_projection_bytes': 4 << 30,
        'model_report_cap_bytes': config['maximum_report_bytes'],
        'live_required_free_bytes': required_free,
        'actual_model_fit': 'ADMITTED_NOT_YET_EXECUTED'})
    subprocess.run(['python3', str(SUPERVISOR), str(BINARY), str(CONFIG), binary_sha,
                    config_sha, 'cuda-learning', str(OBSERVATION)], check=True)
    subprocess.run(['python3', str(FACTORY)], check=True)
    decision = read(factory.DECISION)
    actual = 'NOT_RUN_CONSTRUCTION_NEGATIVE'
    if decision['status'] == 'CONFIG_PREPARED_NOT_EVALUATED':
        actual_config = pathlib.Path(decision['config'])
        need(actual_config == factory.OUTCONFIG and sha(actual_config) == decision['config_sha256'],
             'qualification config path/bytes differ')
        subprocess.run(['python3', str(SUPERVISOR), str(BINARY), str(actual_config), binary_sha,
                        decision['config_sha256'], 'cpu-artifact', str(ACTUAL_OBSERVATION)], check=True)
        actual = 'EXECUTED_SEPARATE_TYPED_REPORT'
    else:
        need(decision['status'] == 'NOT_RUN_CONSTRUCTION_NEGATIVE', 'unknown construction decision')
    result = {'status': 'MODEL_AND_CONDITIONAL_QUALIFICATION_FINISHED',
              'source_commit': SOURCE, 'started_utc': started, 'seconds': time.monotonic() - start,
              'construction_report_sha256': sha(MODEL / 'report.json'),
              'construction_manifest_sha256': sha(MODEL / 'manifest.json'),
              'actual9': actual, 'actual9_config_decision': decision['status'],
              'independent_saved_audit': 'NOT_RUN', 'full512': 'NOT_RUN'}
    write(BASE / 'execution-result.json', result)
    print(json.dumps(result), flush=True)


if __name__ == '__main__':
    main()
