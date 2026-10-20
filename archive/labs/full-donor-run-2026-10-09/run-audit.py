#!/usr/bin/env python3
"""Exclusive saved-credit comparison after normative verification; never runs a model."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import time

ROOT = Path('/workspace/uor-r4/codex/sol-full-donor-20261009')
SOURCE = '4f7eee35b250b6d5bf9e250fc0ea00d356998695'
CONFIG_SHA = 'f778b5affdb17e9da1c29ce517f280357e9333082618c34eee06a0015b7f6692'
READER_SHA = '6e6622bf41de4532b06cc13671963b19b744d678c570468ba8bea06c7a3f1e96'
READER = ROOT / 'compare-credit.py'
MODEL = ROOT / 'runs/full-donor-0001-attempt1'
OBS = ROOT / 'observations/full-donor-0001-attempt1'
AUDIT = ROOT / 'audits/full-donor-0001-attempt1'
OLD = Path('/workspace/uor-r4/codex/sol-coupled-episode-learning/runs/coupled-episode-0001-attempt2')
EXPORTED = OLD.parent / 'coupled-export-completion-0001-attempt3'
CLAIM = Path('/workspace/uor-r4/codex/sol-prefix-margin-boundary/claim-report')
VERIFIER = Path('/workspace/uor-r4/codex/sol-intermediate-word-boundary/publications/runtime-76e10b53f7-attempt2/native-reached-prefix-attribution')
SEAL = ROOT / 'runtime/native_historical_version'

def need(ok, message):
    if not ok:
        raise ValueError(message)

def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1 << 20), b''):
            h.update(block)
    return h.hexdigest()

def read(path):
    return json.loads(path.read_bytes())

def write(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write('\n')

def main():
    need(len(sys.argv) == 1, 'no audit overrides')
    need(sha(READER) == READER_SHA, 'reviewed comparison bytes differ')
    need(sha(CLAIM) == '394ad004859ae8d9baaf04510f230fb725bbc2f7caabe3c7abe26b1cf70ea208', 'claim tool identity')
    need(sha(VERIFIER) == 'd90411116c702dc4149fc055706f34c276c1ead29c5b0ec95600c30cc976326d', 'normative verifier identity')
    # Operational runtime identity only; no scientific/model inputs loaded yet.
    runtime = read(ROOT / 'runtime/runtime-identity.json')
    upload = read(ROOT / 'runtime-upload.json')
    need(runtime['source_commit'] == upload['source_commit'] == SOURCE
         and upload['status'] == 'PASS' and upload['all_fresh_download_verified'] is True
         and runtime['config_sha256'] == upload['config_sha256'] == CONFIG_SHA,
         'fresh preserved exact-source runtime')
    seal_entry = runtime['files']['native_historical_version']
    need(sha(SEAL) == seal_entry['sha256'] and SEAL.stat().st_size == seal_entry['bytes'],
         'actual runtime sealer differs from preserved identity')
    need(not AUDIT.exists(), 'exclusive fresh audit root')
    subprocess.run([str(CLAIM), str(AUDIT)], check=True)
    start = time.monotonic()
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    rc = None
    error = None
    try:
        config = ROOT / 'configs/full-donor-0001-attempt1.json'
        need(sha(config) == CONFIG_SHA, 'submitted config identity')
        binary = ROOT / 'runtime/geometric-frozen-map-fit'
        need(sha(binary) == runtime['binary_sha256'] == upload['binary_sha256'], 'preserved model binary identity')
        checks = []
        def verify(root):
            command = [str(VERIFIER), 'verify-report', str(root)]
            began = time.monotonic()
            result = subprocess.run(command, capture_output=True, text=True)
            checks.append({'root':str(root), 'argv':command, 'exit_code':result.returncode,
                           'seconds':time.monotonic()-began, 'stdout':result.stdout, 'stderr':result.stderr,
                           'manifest_sha256':sha(root/'manifest.json')})
            need(result.returncode == 0, 'normative input verification failed: '+str(root))
        for root in (MODEL, OBS, OLD, EXPORTED):
            verify(root)
        report = read(MODEL / 'report.json')
        need(report['status'] == 'COMPLETED' and report['source_commit'] == SOURCE
             and report['mode'] == 'coupled_episode_learning', 'terminal exact-source model authority')
        execution_result = read(ROOT / 'execution-result.json')
        need(execution_result['status'] == 'MODEL_AND_CONDITIONAL_QUALIFICATION_FINISHED'
             and execution_result['source_commit'] == SOURCE
             and execution_result['construction_report_sha256'] == sha(MODEL/'report.json')
             and execution_result['construction_manifest_sha256'] == sha(MODEL/'manifest.json'),
             'immediate qualification orchestration incomplete')
        qualification = {'status':'NOT_RUN_CONSTRUCTION_NEGATIVE'}
        if report['final_gate']['passed']:
            need(execution_result['actual9'] == 'EXECUTED_SEPARATE_TYPED_REPORT', 'positive candidate actual9 not finished')
            actual = ROOT / 'qualifications/actual9-0001-attempt1'
            verify(actual)
            verify(ROOT / 'observations/actual9-0001-attempt1')
            q = read(actual / 'report.json')
            need(q['status'] == 'COMPLETED', 'actual9 execution failure')
            qualification = {'status':'EXECUTED_SEPARATE_TYPED_REPORT',
                             'report_sha256':sha(actual/'report.json'), 'manifest_sha256':sha(actual/'manifest.json'),
                             'report':q}
        else:
            need(execution_result['actual9'] == 'NOT_RUN_CONSTRUCTION_NEGATIVE', 'negative construction qualification status')
        write(AUDIT/'normative-verification.json', {'status':'PASS','checks':checks})
        argv = ['python3', str(READER), '--config', str(config), '--config-sha256', CONFIG_SHA,
                '--candidate-report-sha256', sha(MODEL/'report.json'),
                '--candidate-manifest-sha256', sha(MODEL/'manifest.json'),
                '--expected-source', SOURCE, '--output', str(AUDIT/'result.json')]
        with (AUDIT/'stdout.log').open('x') as out, (AUDIT/'stderr.log').open('x') as err, (AUDIT/'telemetry.jsonl').open('x') as telemetry:
            process = subprocess.Popen(argv, stdout=out, stderr=err,
                                       preexec_fn=lambda: os.sched_setaffinity(0,{0,1}),
                                       env={**os.environ,'OMP_NUM_THREADS':'2','OPENBLAS_NUM_THREADS':'2'})
            write(AUDIT/'launch.json', {'argv':argv,'pid':process.pid,'started_utc':started,
                                      'reader_sha256':READER_SHA,'source_commit':SOURCE,'config_sha256':CONFIG_SHA,
                                      'sealer_sha256':seal_entry['sha256'],'qualification':qualification,
                                      'scope':'Saved evidence only; no model, encoder, backward or native reconstruction'})
            while True:
                alive = process.poll() is None
                row = {'pid':process.pid,'alive':alive,'seconds':time.monotonic()-start}
                try:
                    lines = Path(f'/proc/{process.pid}/status').read_text().splitlines()
                    for key in ('VmRSS','VmHWM'):
                        row[key] = int(next(x.split()[1] for x in lines if x.startswith(key+':')))
                except (FileNotFoundError, StopIteration):
                    pass
                telemetry.write(json.dumps(row)+'\n');telemetry.flush()
                if not alive:
                    break
                time.sleep(2)
            rc = process.wait()
        need(rc == 0, 'saved comparison failed; preserve stderr and exclusive attempt')
        need(read(AUDIT/'result.json')['status'] == 'PASS_SAVED_COMPARISON', 'comparison result status')
    except Exception as exc:
        error = str(exc)
        raise
    finally:
        write(AUDIT/'execution.json', {'exit_code':rc,'status':'PASS' if rc==0 and error is None else 'FAILED',
                                     'error':error,'started_utc':started,'seconds':time.monotonic()-start,
                                     'children_max_rss_kib':resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss})
        subprocess.run([str(SEAL),'seal',str(AUDIT)],check=True)
        subprocess.run([str(VERIFIER),'verify-report',str(AUDIT)],check=True)
    print(json.dumps({'audit':str(AUDIT),'status':'PASS_SAVED_COMPARISON','seconds':time.monotonic()-start}),flush=True)

if __name__ == '__main__':
    main()
