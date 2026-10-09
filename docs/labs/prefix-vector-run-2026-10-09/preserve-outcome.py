#!/usr/bin/env python3
"""Cold-store completed prefix-vector evidence with exact archive/fresh-download checks.
No model, proposal, gradient, numerical audit or selection is performed here.
"""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import tarfile
import time

R = Path('/workspace/uor-r4/codex/sol-prefix-vector-20261009')
SOURCE = 'cc079744f4824e91a6ad739e2e7cf45c8d374527'
CONFIG_SHA = json.loads((R / 'evidence/input-verification.json').read_text())['config_sha256']
V = Path('/workspace/uor-r4/codex/sol-intermediate-word-boundary/publications/runtime-76e10b53f7-attempt2/native-reached-prefix-attribution')
VERIFIER_SHA = 'd90411116c702dc4149fc055706f34c276c1ead29c5b0ec95600c30cc976326d'
P = R / 'publication-prefix-vector-0001-attempt1'
FRESH = R / 'outcome-redownload'
RECEIPT = R / 'outcome-upload.json'


def need(value, message):
    if not value:
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


def write(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def main():
    need(len(sys.argv) == 1, 'preserver takes no overrides')
    need(not any(p.exists() for p in [P, FRESH, RECEIPT]), 'fresh publication/download/receipt required')
    start = time.monotonic()
    need(sha(V) == VERIFIER_SHA, 'normative verifier differs')
    config = R / 'configs/prefix-vector-0001-attempt1.json'
    need(sha(config) == CONFIG_SHA, 'admitted external config differs')
    model = R / 'runs/prefix-vector-0001-attempt1'
    observation = R / 'observations/prefix-vector-0001-attempt1'
    audit = R / 'audits/prefix-vector-0001-attempt1'
    report, result = read(model / 'report.json'), read(audit / 'report.json')
    need(report['status'] == 'COMPLETED' and report['source_commit'] == SOURCE and
         report['mode'] == 'coupled_episode_learning' and report['selected_model'] is False and
         report['policy']['donor_credit'] == 'full_pool_utility' and report['candidate_native_steps'] == 391,
         'completed source-bound unselected prefix-vector run required')
    need(result['status'] == 'PASS_SAVED_COMPARISON' and read(audit / 'supervised-execution.json')['exit_code'] == 0,
         'completed saved independent audit required')
    positive = report['finite_episode_positive']
    need(type(positive) is bool and report['final_gate']['passed'] is positive,
         'construction decision absent/conflicting')
    roots = [model, observation, audit]
    decision = read(R / 'evidence/actual9-config-decision.json')
    if positive:
        need(decision['status'] == 'CONFIG_PREPARED_NOT_EVALUATED', 'positive qualification authority absent')
        roots += [R / 'qualifications/actual9-0001-attempt1', R / 'observations/actual9-0001-attempt1']
        qualification_scope = 'Actual9 reports retained; outcome/selection not inferred by preservation.'
    else:
        need(decision['status'] == 'NOT_RUN_CONSTRUCTION_NEGATIVE' and
             not (R / 'qualifications/actual9-0001-attempt1').exists(),
             'negative construction must retain actual9 NOT_RUN')
        qualification_scope = 'NOT_RUN_CONSTRUCTION_NEGATIVE'
    verification = []
    for root in roots:
        began = time.monotonic()
        argv = [str(V), 'verify-report', str(root)]
        checked = subprocess.run(argv, check=True, capture_output=True, text=True)
        verification.append({'root': str(root), 'argv': argv, 'exit_code': checked.returncode,
                             'seconds': time.monotonic() - began,
                             'manifest_sha256': sha(root / 'manifest.json'),
                             'stdout': checked.stdout, 'stderr': checked.stderr})
    runtime, upload = read(R / 'runtime/runtime-identity.json'), read(R / 'runtime-upload.json')
    need(runtime['source_commit'] == upload['source_commit'] == SOURCE and upload['status'] == 'PASS' and
         upload['all_fresh_download_verified'] is True and
         runtime['config_sha256'] == upload['config_sha256'] == CONFIG_SHA,
         'preserved runtime authority differs')
    leaves = ['runs', 'observations', 'audits', 'configs', 'runtime', 'evidence',
              'runtime-upload.json', 'execution-admission.json', 'execution-result.json',
              'build-runtime.py', 'verify-inputs.py', 'preserve-runtime.py',
              'prepare-prefix-vector-actual9.py', 'run-prefix-vector.py', 'preserve-outcome.py']
    if (R / 'qualifications').exists():
        leaves.append('qualifications')
    # Retain every root-level operational script, receipt and setup log, including
    # later-added audit helpers. Exclude only this not-yet-created output receipt.
    for path in sorted(R.iterdir()):
        if path.is_file() and path.name != RECEIPT.name:
            leaves.append(path.name)
    files = {}
    for leaf in sorted(set(leaves)):
        path = R / leaf
        need(path.exists() and not path.is_symlink(), 'missing/symlink publication component ' + leaf)
        candidates = sorted(path.rglob('*')) if path.is_dir() else [path]
        for member in candidates:
            need(not member.is_symlink(), 'publication symlink unsupported ' + str(member))
            if member.is_dir():
                continue
            need(member.is_file(), 'unsupported publication entry ' + str(member))
            name = str(member.relative_to(R))
            files[name] = {'sha256': sha(member), 'bytes': member.stat().st_size}
    total = sum(entry['bytes'] for entry in files.values())
    need(total < 1 << 30, 'uncompressed full-member package must be below 1GiB')
    P.mkdir()
    archive = P / 'outcome.tar.gz'
    with tarfile.open(archive, 'w:gz', compresslevel=6) as tar:
        for name in sorted(files):
            tar.add(R / name, arcname=name, recursive=False)
    seen = set()
    with tarfile.open(archive, 'r:gz') as tar:
        for member in tar:
            need(member.isfile() and member.name in files and member.name not in seen,
                 'archive member set/type differs')
            seen.add(member.name)
            stream = tar.extractfile(member)
            need(stream is not None, 'archive member stream absent')
            digest = hashlib.sha256()
            for chunk in iter(lambda: stream.read(1 << 20), b''):
                digest.update(chunk)
            need(digest.hexdigest() == files[member.name]['sha256'] and
                 member.size == files[member.name]['bytes'], 'archive member content differs')
    need(seen == set(files), 'archive inventory incomplete')
    identity = {'schema': 'uor-r4.prefix-vector-outcome/1', 'source_commit': SOURCE,
                'config_sha256': CONFIG_SHA, 'donor_credit': 'full_pool_utility', 'prefix_transaction':'gradient_vector_prefix', 'files': files,
                'files_count': len(files), 'uncompressed_bytes': total,
                'archive_sha256': sha(archive), 'archive_bytes': archive.stat().st_size,
                'complete_member_inventory_verified': True, 'verification': verification,
                'run_report_sha256': sha(model / 'report.json'),
                'run_manifest_sha256': sha(model / 'manifest.json'),
                'audit_result_sha256': sha(audit / 'report.json'),
                'audit_manifest_sha256': sha(audit / 'manifest.json'),
                'runtime_revision': upload['revision'], 'selected_model': False,
                'finite_episode_positive': positive, 'actual9_scope': qualification_scope,
                'selection': 'NOT_PERFORMED_BY_PRESERVATION'}
    write(P / 'identity.json', identity)
    from huggingface_hub import HfApi, hf_hub_download
    repo = 'caseyallard/uor-r4-store'
    prefix = 'codex/sol-prefix-vector-20261009/outcome-prefix-vector-0001-attempt1'
    commit = HfApi().upload_folder(repo_id=repo, repo_type='dataset', folder_path=str(P),
                                  path_in_repo=prefix,
                                  commit_message='Preserve complete prefix-vector run, saved audit, setup logs and conditional qualification')
    for path in sorted(P.iterdir()):
        fresh = Path(hf_hub_download(repo, prefix + '/' + path.name, repo_type='dataset',
                                    revision=commit.oid, local_dir=FRESH, force_download=True))
        need(sha(path) == sha(fresh) and path.stat().st_size == fresh.stat().st_size,
             'forced fresh pinned outcome roundtrip differs')
    receipt = {'status': 'PASS', 'source_commit': SOURCE, 'revision': commit.oid, 'prefix': prefix,
               'all_fresh_download_verified': True, 'full_archive_inventory_verified': True,
               'files': {p.name: {'sha256': sha(p), 'bytes': p.stat().st_size} for p in P.iterdir()},
               'uncompressed_bytes': total, 'archived_files': len(files),
               'seconds': time.monotonic() - start, 'selected_model': False,
               'scope': qualification_scope, 'selection': 'NOT_PERFORMED_BY_PRESERVATION'}
    write(RECEIPT, receipt)
    print(json.dumps(receipt), flush=True)


if __name__ == '__main__':
    main()
