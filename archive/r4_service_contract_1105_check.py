"""Named #1105 metadata check. Reads source/JSON only; never imports model code."""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parent.parent
START = time.monotonic()


def unique(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError('duplicate JSON key: ' + key)
        value[key] = item
    return value


def read_json(path):
    return json.loads(path.read_bytes(), object_pairs_hook=unique,
                      parse_constant=lambda x: (_ for _ in ()).throw(ValueError(x)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='Explicit receipt destination; omitted prints only')
    args = parser.parse_args()
    contract_path = ROOT / 'docs/r4_service_contract_1105.json'
    manifest_path = ROOT / 'docs/r4_service_contract_1105_source_manifest.json'
    c, m = read_json(contract_path), read_json(manifest_path)
    assert c['issue'] == m['issue'] == 1105
    assert c['base_revision'] == m['base_revision']
    verified = []
    cache = {}
    for row in m['sources'] + m['local_evidence'] + m['service_sources']:
        key = (row['repo'], row['revision'], row['path'])
        if key not in cache:
            if row['repo'] == 'Casey-allard/uor-r4-wasm-chat':
                data = subprocess.check_output(['git', '-C', '/Users/casey.allard/Downloads/uor-r4-project',
                    'show', row['revision'] + ':' + row['path']], timeout=20)
            elif row['repo'] == 'UOR-Foundation/uor-r4':
                data = subprocess.check_output(['git', '-C', str(ROOT), 'show',
                    row['revision'] + ':' + row['path']], timeout=20)
            else:
                data = Path(row['local_path']).read_bytes()
            cache[key] = data
        data = cache[key]
        assert len(data) == row['bytes'], key
        assert hashlib.sha256(data).hexdigest() == row['sha256'], key
        if 'git_blob_sha1' in row:
            blob = b'blob ' + str(len(data)).encode() + b'\0' + data
            assert hashlib.sha1(blob).hexdigest() == row['git_blob_sha1'], key
        verified.append({'repo': row['repo'], 'revision': row['revision'],
                         'path': row['path'], 'sha256': row['sha256']})

    h = read_json(ROOT / 'docs/r4_native_bridge_1102_evidence/qualification-handoff.json')
    assert c['identity']['artifact_sha256'] == h['artifact']['sha256']
    assert c['identity']['artifact_bytes'] == h['artifact']['bytes']
    assert c['identity']['historical_binary_sha256'] == h['native_binary']['sha256']
    assert c['identity']['historical_qualification_sha256'] == h['qualification_sha256']
    assert c['identity']['original_export_release_sha256'] == h['trusted_binding']['export_release_sha256']
    q = read_json(ROOT / 'docs/r4_native_bridge_1102_evidence/qualification.json')
    original_contract = read_json(ROOT / 'docs/r4_native_reference_1086_contract.json')
    assert len(q) == 12
    assert set(q) == set(original_contract['exact_fields']['qualification_binding'])
    assert c['identity']['native_state_sha256'] == q['native_state_sha256']
    assert c['identity']['contract_sha256'] == q['contract_sha256']
    assert c['identity']['historical_receipt_applies_to_new_host'] is False

    native = (ROOT / 'crates/uor-r4-core/src/learned_reference/mod.rs').read_text()
    adapter = (ROOT / 'crates/uor-r4-core/src/learned_reference/adapter.rs').read_text()
    variants = re.search(r'pub enum NativeErrorTag \{(.*?)\n\}', native, re.S).group(1)
    variants = [v.strip().rstrip(',') for v in variants.strip().splitlines()]
    tags = {re.sub(r'(?<!^)(?=[A-Z])', '_', v).upper() for v in variants}
    assert tags == set(c['enums']['NativeErrorTag'])
    refusal_tags = set(re.findall(r'Refusal::at\("([A-Z_]+)"', adapter))
    refusal_tags |= set(re.findall(r'status: "([A-Z_]+)"', adapter))
    assert refusal_tags == set(c['enums']['RefusalStatus'])
    for name in ['ModelToken', 'Refusal', 'NativeError']:
        source = adapter if name == 'Refusal' else native
        body = re.search(r'pub struct ' + name + r' \{(.*?)\n\}', source, re.S).group(1)
        fields = set(re.findall(r'pub (\w+):', body))
        assert fields == set(c['types']['definitions'][name]), name

    definitions = c['types']['definitions']
    enums = c['enums']
    primitives = c['types']['primitives']
    unions = c['types']['unions']

    def known(typ):
        for part in typ.split('|'):
            if part.endswith('[]'):
                known(part[:-2])
            else:
                assert part.startswith('literal:') or part in definitions or part in enums or part in primitives or part in unions, part

    for fields in definitions.values():
        for typ in fields.values():
            known(typ)
    for route in c['routes']:
        assert route['response'] in definitions
        assert route['request'] is None or route['request'] in definitions
        assert set(route.get('admit_from', [])) <= set(enums['ModelState'])
    assert len({(r['method'], r['path']) for r in c['routes']}) == len(c['routes']) == 7
    for row in c['state_transitions']:
        assert set(row['from']) <= set(enums['ModelState'])
        assert row['to'] in enums['ModelState']
        assert row.get('job_terminal', 'completed') in enums['JobState']

    def valid(value, typ):
        if '|' in typ:
            return any(valid(value, part) for part in typ.split('|'))
        if typ in unions:
            return any(valid(value, part) for part in unions[typ])
        if typ.endswith('[]'):
            return isinstance(value, list) and all(valid(v, typ[:-2]) for v in value)
        if typ in definitions:
            return isinstance(value, dict) and set(value) == set(definitions[typ]) and all(valid(value[k], t) for k, t in definitions[typ].items())
        if typ in enums:
            return value in enums[typ]
        if typ.startswith('literal:'):
            literal = typ[8:]
            if literal in ['true', 'false'] or literal.isdecimal():
                expected = json.loads(literal)
                return type(value) is type(expected) and value == expected
            return value == literal
        if typ == 'null': return value is None
        if typ == 'boolean': return type(value) is bool
        if typ == 'uint53': return type(value) is int and 0 <= value <= 9007199254740991
        if typ == 'port': return type(value) is int and 1024 <= value <= 65535
        if typ == 'zero_or_one': return type(value) is int and value in [0, 1]
        if typ == 'token_id': return type(value) is int and 0 <= value <= 4095
        if typ == 'string': return isinstance(value, str)
        if typ == 'bounded_string_512': return isinstance(value, str) and len(value.encode()) <= 512
        if typ in ['hex64', 'hex32']: return isinstance(value, str) and re.fullmatch('[0-9a-f]{' + typ[3:] + '}', value) is not None
        if typ == 'configured_model_id': return value == c['identity']['configured_model_id']
        if typ == 'blake3_cid': return isinstance(value, str) and re.fullmatch('blake3:[0-9a-f]{64}', value) is not None
        if typ == 'JobId': return isinstance(value, str) and re.fullmatch('[1-9][0-9]{0,15}', value) is not None and int(value) <= 9007199254740991
        if typ == 'FrozenAcceptedBinding': return value == h['trusted_binding']['accepted_binding']
        if typ == 'canonical_base64':
            try: return isinstance(value, str) and base64.b64encode(base64.b64decode(value, validate=True)).decode() == value
            except (ValueError, UnicodeError): return False
        raise ValueError(typ)

    for example in c['wire_examples']:
        assert example['status'] == 'ILLUSTRATIVE_NOT_EXECUTED'
        assert valid(example['value'], example['type']), example['id']
    assert len({r['id'] for r in c['scenarios']}) == 16
    assert all(n == 0 for n in c['runtime_work_in_this_issue'].values())
    for file in ['docs/r4_service_contract_1105_sources.md', 'docs/r4_service_contract_1105_service_audit.md', c['adr']]:
        assert (ROOT / file).is_file()
    elapsed = time.monotonic() - START
    assert elapsed <= 60
    result = {'schema': 'uor-r4.service-contract-metadata-check/1', 'issue': 1105,
        'observed_utc': datetime.now(timezone.utc).isoformat(), 'status': 'SPECIFICATION_METADATA_CONSISTENT',
        'contract_sha256': hashlib.sha256(contract_path.read_bytes()).hexdigest(),
        'source_manifest_sha256': hashlib.sha256(manifest_path.read_bytes()).hexdigest(),
        'checker_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'source_rows_verified': len(verified), 'unique_original_sources_verified': len(cache),
        'source_identities': verified, 'definition_count': len(definitions), 'routes': len(c['routes']),
        'illustrative_wire_objects_checked': len(c['wire_examples']), 'scenario_ids_checked': len(c['scenarios']),
        'exact_native_error_tags_match_source': True, 'exact_refusal_tags_match_source': True,
        'original_result_object_fields_match_source': True, 'historical_qualification_not_reassigned': True,
        'elapsed_seconds': elapsed, 'budget_seconds': 60,
        'scope': 'JSON unique keys, type/reference closure, illustrative object shape and actual pinned source/identity consistency only. No lifecycle/model implementation was executed or proved.',
        'runtime_work': c['runtime_work_in_this_issue']}
    if args.output is not None:
        args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({k: v for k, v in result.items() if k != 'source_identities'}, indent=2))


if __name__ == '__main__':
    main()
