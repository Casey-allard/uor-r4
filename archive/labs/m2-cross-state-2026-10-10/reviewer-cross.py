"""Independent saved-evidence reader. No model, tokenizer, optimizer or inference calls.
Usage: python3 reviewer-cross.py SEALED_RUN [COMPARATOR_ROOT]
Prints JSON; caller may save it as reviewer-cross.json outside the sealed run.
"""
import collections
import hashlib
import json
import math
import sys
from pathlib import Path

ROOT = Path(sys.argv[1])
LAB = Path(__file__).resolve().parent
COMPARATORS = Path(sys.argv[2]) if len(sys.argv) > 2 else LAB / 'comparators'
BASE = LAB.parent
INPUT_SHA = 'b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a'
LABEL_SHA = '84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155'
PARENT_REPORT_SHA = 'a1277d2be4ff962100f857d0da553257ad5596ac8bc72d9257299e74d2f2e278'
PARENT_MANIFEST_SHA = 'b37e2ca588bf1cc4dc5971fa5da97b9e83c90a94f4ea0bfdb0655b8f92bf94ca'
ACTIVE = 'continuation.cross_state'
ORIGINAL = [0, 1, 4, 5, 8, 9, 12, 13]

def check(ok, message):
    if not ok:
        raise ValueError(message)

def digest(raw):
    return hashlib.sha256(raw).hexdigest()

def read(path, expected=None):
    raw = Path(path).read_bytes()
    if expected is not None:
        check(digest(raw) == expected, 'SHA256 differs: ' + str(path))
    return json.loads(raw)

report = read(ROOT / 'report.json')
attempt = read(ROOT / 'attempt.json')
config_path = Path(attempt['argv'][1])
config = read(config_path)
check(Path(config['out']).resolve() == ROOT.resolve(), 'config/run path')
manifest = read(ROOT / 'manifest.json')
check(report['status'] == 'COMPLETED' and report['mode'] == config['mode'] == 'cross_state_continuation', 'terminal mode/status')
check(report['updates'] == config['updates'] == 64 and report['batch'] == 8, 'fixed dose')
check(report['cache_positions'] == 6664 and report['shared_coefficients'] == 115200, 'positions/coefficients')
for key in ['training_inputs', 'development_inputs']:
    read(config[key], INPUT_SHA)
for key in ['training_labels', 'development_labels']:
    labels_doc = read(config[key], LABEL_SHA)
labels = {x['id']: x['answers']['accepted'] for x in labels_doc['cases']}
check(len(labels) == 512, 'labels coverage')
# Report sealing itself remains the Rust producer's evidence; additionally check
# the listed file set and sizes here, without pretending SHA256 verifies BLAKE3.
listed = {x['path']: x for x in manifest['files']}
check(len(listed) == len(manifest['files']), 'duplicate manifest path')
for path, rec in listed.items():
    check((ROOT / path).is_file() and (ROOT / path).stat().st_size == rec['bytes'], 'manifest size/file: ' + path)
check({str(p.relative_to(ROOT)) for p in ROOT.rglob('*') if p.is_file()} == set(listed) | {'manifest.json'}, 'manifest complete file set')

initial_receipt = report['initial_receipt']
final_receipt = report['final_receipt']
for key in ['parent', 'generate_sha256', 'frozen_model_root', 'frozen_model_report_sha256', 'frozen_model_manifest_sha256', 'frozen_parent_receipt', 'upstream_training']:
    check(initial_receipt[key] == final_receipt[key], 'frozen parent receipt changed: ' + key)
for receipt, step in [(initial_receipt, 0), (final_receipt, 64)]:
    check(receipt == read(ROOT / f'checkpoint-{step:04}/receipt.json'), 'checkpoint receipt/report mismatch')
    check(receipt['step'] == step and receipt['active_parameter_names'] == [ACTIVE], 'checkpoint step/active family')
    check(set(receipt['parameters']) == {ACTIVE} and receipt['shared_coefficients'] == 115200, 'master inventory')
    check(receipt['native_independently_reloaded'] and receipt['masters_independently_reloaded'], 'reload receipt missing')
    check(receipt['frozen_model_report_sha256'] == PARENT_REPORT_SHA and receipt['frozen_model_manifest_sha256'] == PARENT_MANIFEST_SHA, 'parent identity')
parent = Path(config['saved_fit'])
read(parent / 'report.json', PARENT_REPORT_SHA)
read(parent / 'manifest.json', PARENT_MANIFEST_SHA)
admission = read(ROOT / 'admission.json')
check(admission['source_commit'] == report['source_commit'], 'producer source receipt mismatch')
check(admission['active_parameter_names'] == [ACTIVE] and admission['learning_rate'] == 0.03, 'admitted optimizer')
check(set(report['final_active_masters']) == {ACTIVE}, 'final active families')
check(config['seed'] == report['order_seed'] == 1001, 'seed')
order_doc = read(ROOT / 'order.json')
order = order_doc['order']
expected = list(range(512)); state = 1001; mask = (1 << 64) - 1
for i in range(511, 0, -1):
    state = (state + 0x9e3779b97f4a7c15) & mask
    z = state
    z = ((z ^ (z >> 30)) * 0xbf58476d1ce4e5b9) & mask
    z = ((z ^ (z >> 27)) * 0x94d049bb133111eb) & mask
    z ^= z >> 31
    j = z % (i + 1)
    expected[i], expected[j] = expected[j], expected[i]
check(order == expected, 'seed1001 full512 shuffle')
updates = read(ROOT / 'updates.json')
check(len(updates) == 64, 'update coverage')
for i, update in enumerate(updates):
    check(update['step'] == i + 1 and update['indices'] == order[i*8:(i+1)*8], 'actual batch schedule')
    check(update['active_gradient_names'] == [ACTIVE], 'active gradient family')
    n = update['global_active_gradient_norm']
    check(math.isfinite(n) and n >= 0 and (i != 0 or n > 0), 'gradient norm')
check(sum(x['answer_positions_including_eos'] for x in updates) == 6664, 'actual target draws')

sets = {}; outputs = {}; summaries = {}; canonical_total = {}
for phase in ['initial', 'final']:
    evaluation = report[phase]
    check(evaluation['continuation_sha256'] == report[phase + '_receipt']['continuation_sha256'], 'endpoint/checkpoint field identity')
    refs = evaluation['rows']
    check(len(refs) == 512, 'endpoint coverage')
    seen = set(); good_ids = set(); out = {}; count = collections.Counter(); strata = {}
    for i, ref in enumerate(refs):
        rid = ref['id']
        check(rid in labels and rid not in seen, 'endpoint ID identity/uniqueness'); seen.add(rid)
        row = read(ROOT / ref['row_file'], ref['row_sha256'])
        check(row['id'] == rid, 'row identity')
        ids = row['generated_ids']; target = row['canonical_target_ids_labels_only']
        check(target and target[-1] == 1, 'canonical EOS identity')
        check(ids == ref['generated_ids'] and len(ids) == len(row['generation']), 'generated chain coverage')
        for t, trace in enumerate(row['generation']):
            check(trace['actual_prefix_ids'] == ids[:t] and trace['pool']['summary']['chosen_token_id'] == ids[t], 'own-prefix chain')
            check(trace['continuation']['actual_prefix_tokens'] == t, 'continuation prefix length')
        eos = bool(ids) and ids[-1] == 1
        good = eos and row['decoded'] in labels[rid]
        check(eos == row['eos'] == ref['eos'] and good == row['complete'] == ref['complete'], 'EOS/frozen membership')
        check(row['continuation_sha256'] == evaluation['continuation_sha256'], 'row field identity')
        canonical = row['canonical']; check(len(canonical) == len(target), 'canonical coverage')
        teacher = 0
        for token, trace in zip(target, canonical):
            check(trace['target_label_only'] == token, 'canonical target trace')
            teacher += trace['native']['pool']['summary']['chosen_token_id'] == token
        count.update({'complete': good, 'eos': eos, 'entry_correct': bool(ids) and ids[0] == target[0], 'teacher_all_tokens_correct': teacher == len(target), 'teacher_correct_tokens': teacher, 'target_positions': len(target)})
        stratum = rid.split('-')[2]
        sc = strata.setdefault(stratum, {'cases': 0, 'complete': 0})
        sc['cases'] += 1; sc['complete'] += good
        if good: good_ids.add(rid)
        out[rid] = ids
    check(seen == set(labels) and count['complete'] == evaluation['complete'], 'endpoint summary')
    check(count['target_positions'] == 6664, 'canonical target coverage')
    metrics = report[phase + '_metrics']
    check(count['entry_correct'] == metrics['entry_own_correct'], 'entry metric')
    pairs = sum(all(refs[i]['id'] in good_ids for i in pair['indices']) for pair in metrics['pairs'])
    count['complete_swap_pairs'] = pairs
    summaries[phase] = dict(count, complete_ids=sorted(good_ids), strata=strata)
    sets[phase] = good_ids; outputs[phase] = out
original_ids = {report['initial']['rows'][i]['id'] for i in ORIGINAL}
check(sets['initial'] == original_ids and len(original_ids) == 8, 'exact original-eight baseline')
gained = sorted(sets['final'] - sets['initial']); lost = sorted(sets['initial'] - sets['final']); retained = sorted(sets['final'] & sets['initial'])
outcome = report['cross_state_outcome']
for key, value in [('gained_complete_ids', gained), ('lost_complete_ids', lost), ('retained_complete_ids', retained)]:
    check(sorted(outcome[key]) == value, 'outcome IDs: ' + key)
keep = len(sets['final']) > 8
check(outcome['keep'] == keep and outcome['decision'] == ('KEEP' if keep else 'REJECT'), 'prospective net-complete decision')
check(outcome['final_complete'] == len(sets['final']), 'outcome headline')
check(set(outcome['changed_output_ids']) == {k for k in labels if outputs['initial'][k] != outputs['final'][k]}, 'changed-output IDs')
for stratum, counts in outcome['by_stratum_cases_initial_final'].items():
    check(counts == [summaries['initial']['strata'][stratum]['cases'], summaries['initial']['strata'][stratum]['complete'], summaries['final']['strata'][stratum]['complete']], 'stratum outcome')

fields = []; code_arrays = []
for step, receipt in [(0, initial_receipt), (64, final_receipt)]:
    path = ROOT / f'checkpoint-{step:04}/continuation-field.bin'
    field = read(path, receipt['continuation_sha256']); metadata = field['metadata']
    check(metadata['schema'] == 'uor-r4.native-continuation-field/3' and metadata['score_shift'] == 22, 'native schema/units')
    check(metadata['lanes'] == 8 and not field['packed_unary'], 'native dimensions')
    packed = bytes(field['packed_cross_state']); check(len(packed) == 65536, 'packed size')
    check(digest(packed) == metadata['payload_sha256'], 'native payload hash')
    codes = []
    for lane in range(8):
        for factual in range(128):
            for local in range(128):
                byte = packed[(lane << 13) | (factual << 6) | (local >> 1)]
                nibble = (byte >> ((local & 1) * 4)) & 15
                check(nibble != 8, 'forbidden Q4 -8')
                value = nibble if nibble < 8 else nibble - 16
                if factual < 120 and local < 120: codes.append(value)
                else: check(value == 0, 'nonzero128-stride padding')
    check(len(codes) == 115200, 'compact coefficient count')
    fields.append(field); code_arrays.append(codes)
check(all(x == 0 for x in code_arrays[0]), 'initial field is not zero')
for key in fields[0]['metadata']:
    if key != 'payload_sha256': check(fields[0]['metadata'][key] == fields[1]['metadata'][key], 'frozen native binding/policy: ' + key)
crossings = sum(x != y for x, y in zip(*code_arrays))

comparisons = {}
def compare(name, evaluation):
    refs = evaluation['rows']; check(len(refs) == 512 and {x['id'] for x in refs} == set(labels), 'prior row coverage: ' + name)
    prior = {x['id'] for x in refs if x['complete']}
    check(len(prior) == evaluation['complete'], 'prior summary: ' + name)
    comparisons[name] = {'complete_before': len(prior), 'complete_after': len(sets['final']), 'gained': sorted(sets['final'] - prior), 'lost': sorted(prior - sets['final']), 'retained': sorted(prior & sets['final']), 'changed_outputs': sum(x['generated_ids'] != outputs['final'][x['id']] for x in refs)}
for name, folder, sha in [
    ('2141', 'm2-reply-20261009/runs/reply-0064-attempt1', '678f4ff4d8b97559260076607f6168b7d5f4317cc1472bd9fe5e253ca76bdbb7'),
    ('2143', 'm2-stratified-20261010/runs/qualification-0096-attempt1', '16d03a31fc1eb27ee270cec96a7d388c108522c79b2a87856a0ca94b232aab68'),
    ('2148', 'm2-prototype-reply-20261010/runs/prototype-0096-attempt1', '22fb69a89cfa6191ce9e05f840cf4afb790ec10b4bd2e0a07f6962f5a94d4ece'),
    ('2149', 'm2-joint-reply-20261010/runs/joint-0096-attempt1', 'bed3e8eec305f38cfbff7fcb54a1fc7fbb159b3657e4efd17709ddf74e8c8616')]:
    prior = read(BASE / folder / 'report.json', sha)
    compare(name, prior['final_evaluation'])
old = COMPARATORS / 'old-u64'
old_report = read(old / 'report.json', '612f8c8123f05b87112a13acd0028e2130436e2b65273f38c6ccb56feefaaf37')
read(old / 'manifest.json', '50a436ead5af213b0d71135ecdc5089833d74586565a8d6e5f92eaac93c37ca5')
old_initial = read(old / 'development-0000.json', '2ce43ce091dab5d6d160c5683c4a75fd4c66c630d4cf9e82d147df40f2348704')
old_final = read(old / 'development-0064.json', 'd2f0cb3e95870936a3b0464c2bead5c514f2a7866f8348c3fc4286e64822cebc')
check(old_report['mode'] == 'continuation_only' and old_report['updates'] == 64, 'oldU64 protocol')
check({x['id'] for x in old_initial['rows'] if x['complete']} == original_ids, 'oldU64 accepted baseline')
check(old_final['continuation_sha256'] == old_report['final_receipt']['continuation_sha256'], 'oldU64 endpoint binding')
compare('old_u64', old_final)
result = {'status': 'PASS_SAVED_EVIDENCE', 'source_commit': report['source_commit'], 'reader_sha256': digest(Path(__file__).read_bytes()), 'report_sha256': digest((ROOT / 'report.json').read_bytes()), 'manifest_sha256': digest((ROOT / 'manifest.json').read_bytes()), 'config_sha256': digest(config_path.read_bytes()), 'config_scope': 'external attempt-argv config; protocol/data/parent cross-checked against sealed receipts', 'checked_rows': 1024, 'endpoints': summaries, 'gained': gained, 'lost': lost, 'retained': retained, 'keep': keep, 'changed_outputs_vs_parent': sum(outputs['initial'][k] != outputs['final'][k] for k in labels), 'comparisons': comparisons, 'native_cross_state': {'compact_coefficients': 115200, 'packed_bytes': 65536, 'changed_codes': crossings, 'final_nonzero_codes': sum(x != 0 for x in code_arrays[1]), 'minimum_code': min(code_arrays[1]), 'maximum_code': max(code_arrays[1])}, 'schedule': 'PASS seed1001 full512 once,64x8,6664positions', 'frozen_parent_receipts': 'PASS', 'scope': 'Saved hashes/IDs/prefix chains/EOS/frozen membership and native Q4 payloads checked. No model, backward, tokenizer decoding or native score regeneration. Full BLAKE3 seal verification is producer evidence; reader checks file inventory/sizes. Prior comparisons use pinned saved reports, not fresh model evaluation.'}
print(json.dumps(result, indent=2))
