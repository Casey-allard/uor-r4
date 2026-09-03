#!/usr/bin/env python3
"""Independent #1102 audit of retained evidence; no project/model imports."""
import base64
import datetime
import hashlib
import json
import math
import os
from pathlib import Path
import struct

RUN = Path('/Users/casey.allard/.codex/uor/issue-1102-native-comparison')
OUT = Path('/Users/casey.allard/.codex/uor/issue-1102-result-review')
REPO = Path('/Users/casey.allard/.codex/worktrees/r4-native-bridge/uor-r4')
SHAPES = {'role_attention': [1, 5, 3, 13], 'role_vectors': [1, 5, 3, 64],
          'binding_attention': [1, 5], 'logits': [1, 4096]}
NAMES = ('execution-reference', 'execution-native', 'replay-reference', 'replay-native')
PARSED = ('inputs', 'lengths', 'token_spans', 'clause_spans',
          'raw_text_sha256', 'derived_input_sha256')
ERRORS = ('CONTAINER_LIMIT', 'INVALID_CONTAINER', 'ARTIFACT_IDENTITY_MISMATCH',
          'UNSUPPORTED_MANIFEST', 'UNSUPPORTED_PROFILE', 'SOURCE_BINDING_MISMATCH',
          'INVALID_COMPONENT', 'INVALID_TENSOR', 'INVALID_CODEC_POLICY',
          'INVALID_FRAME_TABLE', 'STATE_IDENTITY_MISMATCH')

def sha(data):
    return hashlib.sha256(data).hexdigest()

def canonical(value, lf=True):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'),
                       ensure_ascii=False, allow_nan=False) + ('\n' if lf else '')).encode()

def read_json(path):
    return json.loads(Path(path).read_bytes())

def bound(rec):
    path = Path(rec['path'])
    assert path.is_file() and path.resolve(strict=True) == path, path
    raw = path.read_bytes()
    assert len(raw) == rec['bytes'] and sha(raw) == rec['sha256'], path
    return raw

def file_record(path):
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': sha(data)}

def put_string(h, value):
    data = value.encode('ascii')
    h.update(struct.pack('<I', len(data)))
    h.update(data)

def coordinates(flat, shape):
    out = []
    for n in reversed(shape):
        out.append(flat % n)
        flat //= n
    assert flat == 0
    return list(reversed(out))

release = read_json(RUN / 'release.json')
release_sha = sha((RUN / 'release.json').read_bytes())
assert release_sha == '2c3c2f73eb6cf804eb69b2afb0f979ae623a512ca0492e47df2af70d6cbaca8b'
admission = read_json(RUN / 'independent-review.json')
assert admission['release_sha256'] == release_sha
assert admission['status'] == 'ACCEPTED_FOR_ONE_NATIVE_BRIDGE_ATTEMPT'
assert sha((RUN / 'independent-review.json').read_bytes()) == '88c8f8b4223ab83cca072b263c6a4b2febe542173c040fbf9f73bbc6143f4647'
contract = read_json(REPO / 'docs/r4_native_reference_1086_contract.json')
assert sha((REPO / 'docs/r4_native_reference_1086_contract.json').read_bytes()) == release['contract_sha256']
assert release['budgets'] == contract['budgets']
for rec in release['launch_files']:
    bound(rec)
original = read_json(REPO / 'docs/r4_retained_assembly_1094_evidence/bindings.json')
assert release['reference']['bindings'] == original
operator = read_json(RUN / 'operator-admission.json')
expected_command = [part.replace('{release_sha256}', release_sha).replace(
    '{review_sha256}', sha((RUN / 'independent-review.json').read_bytes()))
    for part in release['launch_contract']['coordinator_command_template']]
assert operator['command'] == expected_command
assert operator['environment'] == release['launch_contract']['coordinator_environment']
assert operator['cwd'] == str(REPO) and operator['model_work_before_supervise'] == 0

raw_rows = [json.loads(line) for line in bound(release['fixtures']['raw']).splitlines()]
refs = [json.loads(line) for line in bound(release['fixtures']['reference']).splitlines()]
assert len(raw_rows) == len(refs) == 336
for i, (raw, ref) in enumerate(zip(raw_rows, refs)):
    assert raw['row_id'] == ref['row_id']
    assert raw['partition'] == ref['partition'] == 'authoring'
    assert raw['kind'] == ref['kind'] == ('valid' if i < 320 else 'refusal')
    assert sha(base64.b64decode(raw['text_base64'], validate=True)) == ref['raw_text_sha256']
assert len({x['row_id'] for x in refs}) == 336

# Parse only the retained container framing, metadata and codec/frame tables.
# Parameter payloads are hashed as opaque bytes; no model is instantiated.
artifact = (RUN / 'exports/a.r4lr').read_bytes()
assert artifact == (RUN / 'exports/b.r4lr').read_bytes()
assert artifact[:8] == b'R4LR0001'
manifest_len = struct.unpack_from('<I', artifact, 8)[0]
manifest_raw = artifact[12:12 + manifest_len]
manifest = json.loads(manifest_raw)
assert manifest_raw == canonical(manifest, lf=False)
payload_len = struct.unpack_from('<Q', artifact, 12 + manifest_len)[0]
payload = artifact[20 + manifest_len:]
assert payload_len == len(payload) == contract['container']['payload_bytes']
assert len(artifact) <= contract['container']['maximum_bytes']
assert manifest_len <= contract['container']['maximum_manifest_bytes']
expected = read_json(RUN / 'exports/expected.json')
assert sha(artifact) == expected['artifact_sha256'] == '2c209590a64cae16a4140fd43adc1cb1f87b357c02e3d4959f1e37f4ab8cd5ab'
assert expected['export_release_sha256'] == release_sha
assert manifest['contract_sha256'] == expected['contract_sha256'] == release['contract_sha256']
assert manifest['source_binding'] == expected['accepted_binding']
accepted = manifest['source_binding']
for name, rec in original['assets'].items():
    assert accepted['assets'][name] == {k: rec[k] for k in ('bytes', 'cid', 'sha256')}
for key in ('reader_state_cid', 'core_state_cid', 'frame_tree_cid', 'policy_sha256'):
    assert accepted[key] == original[key]
provenance = manifest['export_provenance']
for key in ('source_revision', 'exporter_revision', 'exporter_sources',
            'exporter_runtime', 'exporter_lock_sha256'):
    assert provenance[key] == release['export'][key]
assert provenance['release_sha256'] == release_sha
assert manifest['tied_aliases'] == contract['tied_aliases'] == {'core.lm_head.weight': 'core.embedding.weight'}
parts = {}
state = hashlib.sha256()
put_string(state, 'uor-r4.native-reference-state/1')
state.update(struct.pack('<I', len(manifest['components'])))
offset = 0
assert len(manifest['components']) == len(contract['components']) == 21
for comp, template in zip(manifest['components'], contract['components']):
    assert {k: v for k, v in comp.items() if k != 'sha256'} == template
    assert comp['offset'] == offset
    data = payload[offset:offset + comp['bytes']]
    assert len(data) == comp['bytes'] and sha(data) == comp['sha256']
    parts[comp['name']] = data
    offset += len(data)
    for key in ('name', 'kind', 'dtype'):
        put_string(state, comp[key])
    state.update(struct.pack('<I', len(comp['shape'])))
    for dimension in comp['shape']:
        state.update(struct.pack('<Q', dimension))
    state.update(struct.pack('<Q', comp['bytes']))
    state.update(data)
assert offset == payload_len
state.update(struct.pack('<I', manifest['identity_index']))
put_string(state, manifest['operator_profile'])
assert state.hexdigest() == manifest['native_state_sha256'] == '4f453da12a9346356e64b6c16abfbaad1ca99e3966173cd79e9ddbc8c2d9341b'
for comp_name, asset_name in (('vocabulary.json', 'vocabulary'), ('h4-frames.json', 'h4_frames'), ('token-frames.json', 'token_frames')):
    assert sha(parts[comp_name]) == accepted['assets'][asset_name]['sha256']
assert sha(parts['policy.json']) == accepted['policy_sha256']
vocabulary = json.loads(parts['vocabulary.json'])['vocabulary']
assert len(vocabulary) == 4096
multiplication = struct.unpack('<14400q', parts['multiplication'])
leaves = struct.unpack('<8192q', parts['token_leaves'])

gate = read_json(RUN / 'results-gate.json')
frozen = read_json(RUN / 'mutations/frozen.json')
assert len(gate) == 12 and len(frozen) == 11
for i, fixture in enumerate(frozen):
    bound(fixture['artifact']); bound(fixture['expected_binding'])
    assert gate[i] == read_json(RUN / 'mutations' / f'gate-{i:02d}.json')
    assert gate[i]['error'] == fixture['expected_error']
    assert gate[i]['error']['tag'] == ERRORS[i]
    assert gate[i]['logical_forwards'] == gate[i]['model_loads'] == 0
    assert gate[i]['validation_audit']['stages'] == list(ERRORS[:i + 1])
    assert all(value == 0 for value in gate[i]['fpcr_after'].values())
good = gate[-1]
assert good == read_json(RUN / 'mutations/gate-11.json')
assert good['error'] is None and good['logical_forwards'] == 0 and good['model_loads'] == 2
assert good['validation_audit']['stages'] == list(ERRORS)
assert good['missing_qualification']['tag'] == 'UNAVAILABLE_NATIVE_QUALIFICATION'
assert good['capability']['native_behavior'] == 'NOT_RUN'
assert good['native_state_sha256'] == state.hexdigest()
assert good['owned_artifact_bytes'] == len(artifact)
partial_states = sum(len(g['validation_audit']['partial_model_states']) for g in gate[:-1])
assert partial_states == 7 <= 22

rows_by_arm = {}
tensor_values = {}
arm_summaries = {}
for name in NAMES:
    row_path = RUN / 'results' / (name + '.jsonl')
    rows = [json.loads(line) for line in row_path.read_bytes().splitlines()]
    stream = (RUN / 'results' / (name + '.f32')).read_bytes()
    assert len(rows) == 336 and len(stream) == 6727680
    rows_by_arm[name] = rows
    tensors = []
    cursor = 0
    roles_correct = answers = refusals = padding_zero_entries = 0
    for i, (row, ref) in enumerate(zip(rows, refs)):
        assert row['coordinator_row_index'] == i and row['coordinator_row_id'] == ref['row_id']
        assert row['kind'] == 'result'
        if i >= 320:
            assert row['logical_forwards'] == 0
            assert row['parsed'] is None and row['diagnostics'] is None and row['tensors'] == {}
            assert set(row['result']) == {'schema', 'status', 'byte_offset'}
            assert row['result']['schema'] == 'uor-r4.text-to-clauses-result/1'
            assert row['result']['status'] == ref['expected_status']
            if 'expected_byte_offset' in ref:
                assert row['result']['byte_offset'] == ref['expected_byte_offset']
            refusals += 1
            tensors.append({})
        else:
            assert row['logical_forwards'] == 1 and row['result']['status'] == 'MODEL_TOKEN'
            parsed = row['parsed']
            assert all(parsed[field] == ref[field] for field in PARSED)
            h = hashlib.sha256()
            for value in ('uor-r4.text-to-clauses-input/1', accepted['policy_sha256'],
                          accepted['assets']['vocabulary']['cid'], 'i64le'):
                put_string(h, value)
            h.update(struct.pack('<5I', 1, 5, 13, 1, 5))
            for clause in parsed['inputs'][0]:
                h.update(struct.pack('<13q', *clause))
            h.update(struct.pack('<5q', *parsed['lengths'][0]))
            assert h.hexdigest() == parsed['derived_input_sha256']
            assert set(row['tensors']) == set(SHAPES)
            decoded = {}
            for tensor_name, shape in SHAPES.items():
                rec = row['tensors'][tensor_name]
                count = math.prod(shape) * 4
                assert rec['shape'] == shape and rec['offset'] == cursor and rec['bytes'] == count
                data = stream[cursor:cursor + count]
                assert sha(data) == rec['sha256']
                values = struct.unpack('<' + 'f' * (count // 4), data)
                assert all(math.isfinite(v) for v in values)
                decoded[tensor_name] = values
                if tensor_name == 'role_attention':
                    role_argmax = []
                    for clause, length in enumerate(parsed['lengths'][0]):
                        for role in range(3):
                            start = (clause * 3 + role) * 13
                            role_argmax.append(max(range(length), key=lambda j: values[start + j]))
                            for j in range(length, 13):
                                assert data[(start+j)*4:(start+j+1)*4] == b'\0' * 4
                                padding_zero_entries += 1
                    assert row['diagnostics']['role_argmax'] == role_argmax
                    expected_roles = [v for clause in ref['role_positions'] for v in clause][:14]
                    assert len(expected_roles) == 14 and role_argmax[:14] == expected_roles
                    roles_correct += 14
                cursor += count
            predicted = max(range(4096), key=lambda j: decoded['logits'][j])
            assert predicted == row['result']['token_id'] == ref['target_id']
            assert row['result']['token'] == vocabulary[predicted]
            answers += 1
            current = manifest['identity_index']; token_frames = []; clause_frames = []
            for clause, length in zip(parsed['inputs'][0], parsed['lengths'][0]):
                for j, token in enumerate(clause):
                    if j < length:
                        current = multiplication[current * 120 + leaves[token]]
                        token_frames.append(current)
                    else:
                        token_frames.append(manifest['identity_index'])
                clause_frames.append(current)
            assert row['diagnostics']['token_frame_indices'] == token_frames
            assert row['diagnostics']['clause_frame_indices'] == clause_frames
            for key in ('raw_text_sha256', 'derived_input_sha256'):
                assert row['result'][key] == parsed[key]
            assert row['result']['policy_sha256'] == accepted['policy_sha256']
            assert row['result']['reader_file_cid'] == accepted['assets']['reader']['cid']
            assert row['result']['core_file_cid'] == accepted['assets']['core']['cid']
            assert row['result']['frame_tree_cid'] == accepted['frame_tree_cid']
            tensors.append(decoded)
        if name.endswith('native'):
            receipt = row['receipt']
            assert receipt['logical_forwards'] == row['logical_forwards'] and receipt['parameter_updates'] == 0
            assert receipt['raw_text_sha256'] == ref['raw_text_sha256']
            assert receipt['artifact_sha256'] == sha(artifact)
            assert receipt['native_state_sha256'] == state.hexdigest()
            assert receipt['native_binary_sha256'] == release['native']['runtime']['native_binary_sha256']
            assert receipt['runtime_receipt_sha256'] == release['native']['runtime']['runtime_receipt_sha256']
            assert receipt['contract_sha256'] == release['contract_sha256']
            assert receipt['result_sha256'] == sha(canonical(row['result'], lf=False))
    assert cursor == len(stream)
    tensor_values[name] = tensors
    ready = read_json(RUN / 'results' / (name + '.ready.json'))
    done = read_json(RUN / 'results' / (name + '.done.json'))
    assert ready['logical_forwards'] == 0 and ready['model_loads'] == done['model_loads'] == 2
    assert done['logical_forwards'] == 320 and done['valid_rows'] == 320 and done['refusal_rows'] == 16
    assert done['parameter_updates'] == 0
    if name.endswith('reference'):
        assert ready['states_before'] == done['states_before'] == done['states_after'] == {
            'core': accepted['core_state_cid'], 'reader': accepted['reader_state_cid']}
        assert ready['runtime'] == {'blas': 'accelerate', 'device': 'cpu', 'interop_threads': 1,
            'python': '3.12.14', 'threads': 4, 'torch': '2.7.1', 'workers': 1}
        assert all(ready['denied_probes'].values()) and len(ready['denied_probes']) == 4
    else:
        assert ready['native_state_sha256'] == done['native_state_sha256'] == state.hexdigest()
        assert all(v == 0 for v in ready['fpcr'].values())
        assert all(v == 0 for v in done['fpcr_after'].values())
    arm_summaries[name] = {'answers_correct': answers, 'consumed_roles_correct': roles_correct,
        'refusals_correct': refusals, 'reference_errors': [], 'all_finite': True,
        'positive_zero_padding_entries': padding_zero_entries,
        'full_head_and_all_role_argmax_recomputed': True, 'frame_indices_recomputed': True,
        'row_file': file_record(row_path), 'tensor_file': file_record(RUN / 'results' / (name + '.f32'))}

comparisons = {}
for phase, stored_name in (('execution', 'initial'), ('replay', 'replay')):
    aname, bname = phase + '-reference', phase + '-native'
    peaks = {name: {'max_abs': -1} for name in SHAPES}
    per_row = []
    for i in range(336):
        a, b = rows_by_arm[aname][i], rows_by_arm[bname][i]
        assert all(a[key] == b[key] for key in ('result', 'parsed', 'logical_forwards'))
        assert a['diagnostics'] == b['diagnostics']
        maxima = {}
        for tensor_name, shape in SHAPES.items():
            if i >= 320:
                continue
            av, bv = tensor_values[aname][i][tensor_name], tensor_values[bname][i][tensor_name]
            index = max(range(len(av)), key=lambda j: abs(av[j] - bv[j]))
            error = abs(av[index] - bv[index])
            assert error <= 1e-5
            maxima[tensor_name] = {'max_abs': error, 'flat_index': index}
            if error > peaks[tensor_name]['max_abs']:
                peaks[tensor_name] = {'max_abs': error, 'row': i, 'row_id': refs[i]['row_id'],
                    'flat_index': index, 'coordinates': coordinates(index, shape),
                    'reference_value': av[index], 'native_value': bv[index]}
        per_row.append({'row': i, 'maxima': maxima})
    stored = read_json(RUN / ('comparison-' + stored_name + '.json'))
    assert stored == {'errors': [], 'rows': per_row}
    comparisons[phase] = {'errors': [], 'peaks': peaks, 'rows': per_row}

replays = {}
for arm in ('reference', 'native'):
    for suffix in ('.jsonl', '.f32'):
        a = (RUN / 'results' / ('execution-' + arm + suffix)).read_bytes()
        b = (RUN / 'results' / ('replay-' + arm + suffix)).read_bytes()
        assert a == b
    first = read_json(RUN / 'results' / ('execution-' + arm + '.done.json'))
    second = read_json(RUN / 'results' / ('replay-' + arm + '.done.json'))
    normalize = lambda d: {k: v for k, v in d.items() if k not in ('phase', 'resources')}
    assert normalize(first) == normalize(second)
    replays[arm] = {'row_jsonl_byte_identical': True, 'full_tensor_file_byte_identical': True,
                    'completion_identical_excluding_phase_and_resources': True}

result = read_json(RUN / 'coordinator-result.json')
assert sha((RUN / 'coordinator-result.json').read_bytes()) == 'ae9c6fce9f50cab67c94ae9695c28ebfd735069b6d42833a2ad73666ab7e8263'
assert result['terminal'] == 'NATIVE_REFERENCE_PRESERVED'
details = result['details']
assert details['completed_logical_forwards'] == 1280 and details['parameter_updates'] == 0
assert details['successful_engine_loads'] == 5 and details['completed_model_state_loads'] == 10
assert details['loader_gate_calls'] == 12 and details['loader_rejected_partial_model_states'] == partial_states
assert details['artifact_sha256'] == sha(artifact) and details['native_state_sha256'] == state.hexdigest()
assert details['worker_failures'] == [] and details['initial_comparison_errors'] == details['replay_comparison_errors'] == []
completion = read_json(RUN / 'supervisor-completion.json')
wall = read_json(RUN / 'supervisor-wall.json')
assert completion['terminal'] == result['terminal'] and completion['child_exit_code'] == 0
assert completion['release_sha256'] == wall['release_sha256'] == release_sha
assert not (RUN / 'supervisor-stop.json').exists()
assert wall['elapsed_seconds_after_completion_write'] <= 360
assert all(v <= 120 for v in completion['phase_seconds'].values())
assert completion['rss_peak_bytes'] <= 3221225472
assert wall['ledger_bytes_including_wall_receipt'] <= 134217728
assert wall['wall_receipt_write_checked_before_return'] is True
total_tensor_bytes = sum(x['tensor_file']['bytes'] for x in arm_summaries.values())
assert total_tensor_bytes == 26910720
evidence = [file_record(p) for p in sorted(RUN.rglob('*')) if p.is_file()]
assert sum(e['bytes'] for e in evidence) + sum(e['bytes'] for e in release['fixtures'].values()) == wall['ledger_bytes_including_wall_receipt']
audit = {'schema': 'uor-r4.native-bridge-independent-result-audit/1', 'issue': 1102,
    'utc_reviewed': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'disposition': 'ACCEPTED_BOUNDED_NATIVE_REFERENCE_QUALIFICATION',
    'terminal': result['terminal'], 'release_sha256': release_sha,
    'admission_sha256': sha((RUN / 'independent-review.json').read_bytes()),
    'source_revision': release['source_revision'], 'contract_sha256': release['contract_sha256'],
    'native_binary_sha256': release['native']['runtime']['native_binary_sha256'],
    'runtime_receipt_sha256': release['native']['runtime']['runtime_receipt_sha256'],
    'artifact_sha256': sha(artifact), 'artifact_bytes': len(artifact),
    'native_state_sha256': state.hexdigest(), 'manifest_bytes': manifest_len,
    'launch_identity_count': len(release['launch_files']), 'original_reference_sources': 169,
    'both_exports_byte_identical': True, 'artifact_components_hashed': 21,
    'tied_head_evidence': 'Exact manifest alias plus previously independently reviewed source alias; no new engine/model load.',
    'arms': arm_summaries, 'comparisons': comparisons, 'replays': replays,
    'gate': {'calls': 12, 'expected_rejections': 11, 'successful_engines': 1,
             'partial_rejected_model_states': partial_states, 'logical_forwards': 0,
             'missing_qualification_refused': True, 'records': gate},
    'total_successful_engine_loads': 5, 'total_model_state_loads': 10,
    'logical_forwards': 1280, 'refusal_forwards': 0, 'parameter_updates': 0,
    'total_retained_tensor_bytes': total_tensor_bytes,
    'supervisor_completion': completion, 'supervisor_wall': wall,
    'sealed_evidence': evidence,
    'review_work': {'project_imports': 0, 'model_imports': 0, 'model_loads': 0,
        'logical_forwards': 0, 'exports': 0, 'builds': 0, 'fits': 0, 'replays': 0,
        'withheld_reads': 0, 'original_authoring_files_read_as_evaluator': 2,
        'retained_tensor_and_record_comparison': True,
        'retained_parameter_bytes_hashed_without_tensor_deserialization': True},
    'limits': ['Measured preservation only for the original 320 valid and 16 refusal authoring fixtures, B=1, this exact artifact/binary/runtime/profile.',
        'The reused authoring population is not a new independent semantic holdout.',
        'No universal mathematical or floating-point equivalence proof; cross-runtime tensors differ within the frozen 1e-5 absolute ceiling.',
        'No portability, general parsing/context/generation/reasoning/coding, geometry-superiority or final integer-kernel claim.',
        'Recorded timings are this one bounded campaign, not a throughput or comparative performance benchmark.',
        'No additional run or task is authorized by this retrospective review.']}
OUT.mkdir(parents=True, exist_ok=True)
with (OUT / 'audit.json').open('xb') as f:
    f.write(canonical(audit)); f.flush(); os.fsync(f.fileno()); os.fchmod(f.fileno(), 0o444)
print(json.dumps({'disposition': audit['disposition'], 'audit': str(OUT / 'audit.json'),
    'audit_sha256': sha((OUT / 'audit.json').read_bytes()),
    'peaks': comparisons['execution']['peaks'], 'arms': {k: {x: v[x] for x in (
        'answers_correct', 'consumed_roles_correct', 'refusals_correct')} for k, v in arm_summaries.items()},
    'evidence_files': len(evidence), 'complete_ledger_bytes': wall['ledger_bytes_including_wall_receipt']}))
