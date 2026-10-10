"""Independent saved-evidence reader. No model, tokenizer, optimizer or inference calls.
Usage: python3 reviewer-resume175.py SEALED_RUN [COMPARATOR_ROOT]
Prints JSON; caller may save it as reviewer-resume175.json outside the sealed run.
"""
import collections
import hashlib
import json
import math
import sys
import struct
from pathlib import Path

ROOT = Path(sys.argv[1])
LAB = Path(__file__).resolve().parent
COMPARATORS = Path(sys.argv[2]) if len(sys.argv) > 2 else LAB.parent / 'm2-cross-state-20261010/comparators'
BASE = LAB.parent
INPUT_SHA = 'b9661606b280884217a64e0a5b643f8324a90390e47ade7241da0889a5f7c86a'
LABEL_SHA = '84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155'
PARENT_REPORT_SHA = 'a1277d2be4ff962100f857d0da553257ad5596ac8bc72d9257299e74d2f2e278'
PARENT_MANIFEST_SHA = 'b37e2ca588bf1cc4dc5971fa5da97b9e83c90a94f4ea0bfdb0655b8f92bf94ca'
ACTIVE = 'continuation.cross_state'
OBJECTIVE = 'equal-episode-logmeanexp-unweighted-native-token-CE/1;temperature1;all-targets-including-EOS;existing-raw-identity-STE;no-phase-weighting'
PHASE_POLICY = 'none;all-target-token-logmeanexp'
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
check(report['source_commit'] == '25b36a074c682935d7319f04049ff3831afd8d26', 'reviewed producer identity')
attempt = read(ROOT / 'attempt.json')
config_path = Path(attempt['argv'][1])
config = read(config_path)
check(Path(config['out']).resolve() == ROOT.resolve(), 'config/run path')
manifest = read(ROOT / 'manifest.json')
check(report['status'] == 'COMPLETED' and report['mode'] == config['mode'] == 'cross_state_continuation', 'terminal mode/status')
check(config['cross_state_bottleneck'] is True and report['cross_state_bottleneck'] is True, 'explicit bottleneck objective admission')
check(report['training_objective'] == OBJECTIVE and report['phase_policy'] == PHASE_POLICY, 'reported bottleneck objective/no-phase weighting')
check(report['updates'] == config['updates'] == 256 and report['batch'] == 8, 'fixed dose')
check(report['cache_positions'] == 6664 and report['shared_coefficients'] == 115200, 'positions/coefficients')
ACCEPTED_REPORT_SHA = '9632d32651d0bcce82b2cda2ba70dee7d7a7878ff99577f88002442e23acb572'
ACCEPTED_MANIFEST_SHA = '1a6216a307a739a804f81bf94e93fd89638a896fda1904986dd3aee0a280fccc'
ACCEPTED_FIELD_SHA = 'c82c376df10f14d5c1c9af970fd3b1fe239fb3560e2ccd0020dd2805d8fb773e'
resume_config = config['cross_state_resume']
accepted_root = Path(resume_config['root'])
accepted = read(accepted_root / 'report.json', ACCEPTED_REPORT_SHA)
read(accepted_root / 'manifest.json', ACCEPTED_MANIFEST_SHA)
check(accepted['final']['complete'] == 175, 'pinned accepted headline')
check(accepted['cross_state_bottleneck'] is True and accepted['training_objective'] == OBJECTIVE and accepted['phase_policy'] == PHASE_POLICY, 'parent bottleneck objective identity')
check(accepted['updates'] == 256 and accepted['prior_updates'] == 320 and accepted['lineage_step'] == 576, 'parent local/cumulative lineage')
check(accepted['cross_state_resume']['report_sha256'] == 'e3a7e3ebd93c38253afb67f77ad3bb6b64a5e0ac50561f0204e1b83d22b8f4a5' and accepted['cross_state_resume']['prior_step'] == 256 and accepted['cross_state_resume']['prior_lineage_step'] == 320 and accepted['cross_state_resume']['prior_complete'] == 145, 'parent saved145 ancestry')
check(resume_config['expected_report_sha256'] == ACCEPTED_REPORT_SHA and resume_config['expected_manifest_sha256'] == ACCEPTED_MANIFEST_SHA and resume_config['expected_field_sha256'] == ACCEPTED_FIELD_SHA, 'resume config pins')
check(report['prior_updates'] == 576 and report['lineage_step'] == 832 and report['fresh_adam'], 'fresh Adam/local versus lineage steps')
check(report['training_row_draws'] == 2048 and report['target_position_draws'] == 26656, 'reported draw counts')
resume = report['cross_state_resume']
check(resume['report_sha256'] == ACCEPTED_REPORT_SHA and resume['manifest_sha256'] == ACCEPTED_MANIFEST_SHA and resume['field_sha256'] == ACCEPTED_FIELD_SHA and resume['prior_step'] == 256 and resume['prior_lineage_step'] == 576 and resume['prior_complete'] == 175, 'resume provenance')
prior_receipt_path = accepted_root / 'checkpoint-0256/receipt.json'
prior_receipt = read(prior_receipt_path, resume['checkpoint_receipt_sha256'])
check(prior_receipt == accepted['final_receipt'] and resume['source_master_inventory'] == prior_receipt['parameters'], 'prior receipt/master inventory')

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
for rec, local_step in [(initial_receipt, 0), (final_receipt, 256)]:
    check(rec['lineage_step'] == local_step + 576 and rec['cross_state_resume'] == resume, 'checkpoint lineage/provenance')
for key in ['parent', 'generate_sha256', 'frozen_model_report_sha256', 'frozen_model_manifest_sha256', 'frozen_parent_receipt']:
    check(initial_receipt[key] == prior_receipt[key], 'accepted upstream binding changed: ' + key)
check(initial_receipt['parameters'] == prior_receipt['parameters'], 'initial fractional master inventory differs')
master_rel = Path('continuation-source') / (ACTIVE + '.f32le')
prior_master_bytes = (accepted_root / 'checkpoint-0256' / master_rel).read_bytes()
initial_master_bytes = (ROOT / 'checkpoint-0000' / master_rel).read_bytes()
check(prior_master_bytes == initial_master_bytes, 'initial fractional master bytes differ from saved576')
check(digest(prior_master_bytes) == 'c59f6eb8898a7b7f3ebef0a877a7a26fef3ab3eb681a55e8c6e02cf701d11f10', 'pinned prior fractional bytes')
for rec, step in [(initial_receipt, 0), (final_receipt, 256)]:
    raw = (ROOT / f'checkpoint-{step:04}' / master_rel).read_bytes()
    identity = rec['parameters'][ACTIVE]
    check(identity['shape'] == [8, 14400] and identity['bytes'] == len(raw) == 460800 and identity['sha256'] == digest(raw), 'master shape/bytes/hash')
    check(all(math.isfinite(x[0]) and abs(x[0]) <= 1.75 for x in struct.iter_unpack('<f', raw)), 'master finite/range')
check((ROOT / 'checkpoint-0000/continuation-field.bin').read_bytes() == (accepted_root / 'checkpoint-0256/continuation-field.bin').read_bytes(), 'initial native field byte equality')

for key in ['parent', 'generate_sha256', 'frozen_model_root', 'frozen_model_report_sha256', 'frozen_model_manifest_sha256', 'frozen_parent_receipt', 'upstream_training']:
    check(initial_receipt[key] == final_receipt[key], 'frozen parent receipt changed: ' + key)
for receipt, step in [(initial_receipt, 0), (final_receipt, 256)]:
    check(receipt == read(ROOT / f'checkpoint-{step:04}/receipt.json'), 'checkpoint receipt/report mismatch')
    check(receipt['cross_state_bottleneck'] is True and receipt['training_objective'] == OBJECTIVE, 'checkpoint objective binding')
    check(receipt['step'] == step and receipt['active_parameter_names'] == [ACTIVE], 'checkpoint step/active family')
    check(set(receipt['parameters']) == {ACTIVE} and receipt['shared_coefficients'] == 115200, 'master inventory')
    check(receipt['native_independently_reloaded'] and receipt['masters_independently_reloaded'], 'reload receipt missing')
    check(receipt['frozen_model_report_sha256'] == PARENT_REPORT_SHA and receipt['frozen_model_manifest_sha256'] == PARENT_MANIFEST_SHA, 'parent identity')
parent = Path(config['saved_fit'])
read(parent / 'report.json', PARENT_REPORT_SHA)
read(parent / 'manifest.json', PARENT_MANIFEST_SHA)
admission = read(ROOT / 'admission.json')
check(admission['source_commit'] == report['source_commit'], 'producer source receipt mismatch')
check(admission['cross_state_bottleneck'] is True and admission['training_objective'] == OBJECTIVE and admission['phase_policy'] == PHASE_POLICY, 'admitted bottleneck objective/no-phase weighting')
check(admission['objective_accumulation'] == 'max-shifted detached streaming STE gradient numerator; one token graph plus episode/batch gradients; no retained episode graphs', 'streamed gradient policy')
check(0 < admission['maximum_episode_target_positions'] <= 32, 'target position admission cap')
check(admission['cross_state_resume'] == resume and admission['prior_updates'] == 576 and admission['local_updates'] == 256 and admission['final_lineage_step'] == 832 and admission['fresh_adam'], 'admission lineage')
check(admission['initial_active_masters'] == {ACTIVE: digest(initial_master_bytes)}, 'initial master identity receipt')
check(report['final_active_masters'] == {ACTIVE: final_receipt['parameters'][ACTIVE]['sha256']}, 'final master identity receipt')
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
check(len(updates) == 256, 'update coverage')
for i, update in enumerate(updates):
    check(update['step'] == i + 1 and update['indices'] == order[(i%64)*8:((i%64)+1)*8], 'actual batch schedule')
    check(update['active_gradient_names'] == [ACTIVE], 'active gradient family')
    check(update['training_objective'] == OBJECTIVE and update['phase_balanced_loss'] is None, 'per-update objective/no-phase weighting')
    check(math.isfinite(update['objective_loss']) and update['objective_loss'] >= 0 and update['objective_loss'] == update['bottleneck_logmeanexp_loss'], 'true bottleneck objective scalar')
    check(update['lineage_step'] == i + 577, 'actual update lineage')
    n = update['global_active_gradient_norm']
    check(math.isfinite(n) and n >= 0 and (i != 0 or n > 0), 'gradient norm')
check(sum(x['answer_positions_including_eos'] for x in updates) == 26656, 'actual target draws')

cache = read(ROOT / 'fixed-position-cache.json')
prior_cache = read(accepted_root / 'fixed-position-cache.json')
check(cache['cases'] == 512 and cache['positions'] == 6664 and cache['rows'] == prior_cache['rows'], 'frozen cache packet/prefix/state provenance changed')
# Cache records carry provenance, not cached score tensors; zero-score separation
# is established by reviewed source and producer native-zero parity admission.
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
        check(target and target[-1] == 1 and len(target) <= 32, 'canonical EOS identity/target cap')
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
accepted22 = read(BASE / 'm2-cross-state-20261010/runs/cross-0064-attempt1/report.json', '738649541df34641ffe0eb6e45a151d612d485ff8902b2d9d8d1e812844a2cde')
original_ids = {accepted22['initial']['rows'][i]['id'] for i in ORIGINAL}
check(len(original_ids) == 8, 'original-eight baseline indices')
check(sets['initial'] == {x['id'] for x in accepted['final']['rows'] if x['complete']} and len(sets['initial']) == 175, 'saved175 success set')
for old, now in zip(accepted['final']['rows'], report['initial']['rows']):
    for key in ['id', 'generated_ids', 'complete', 'eos']:
        check(old[key] == now[key], 'all512 exact saved175 baseline: ' + key)
gained = sorted(sets['final'] - sets['initial']); lost = sorted(sets['initial'] - sets['final']); retained = sorted(sets['final'] & sets['initial'])
outcome = report['cross_state_outcome']
for key, value in [('gained_complete_ids', gained), ('lost_complete_ids', lost), ('retained_complete_ids', retained)]:
    check(sorted(outcome[key]) == value, 'outcome IDs: ' + key)
keep = len(sets['final']) > 175
check(outcome['keep'] == keep and outcome['decision'] == ('KEEP' if keep else 'REJECT'), 'prospective net-complete decision')
check(outcome['initial_complete'] == 175 and outcome['final_complete'] == len(sets['final']), 'outcome headline')
check(set(outcome['changed_output_ids']) == {k for k in labels if outputs['initial'][k] != outputs['final'][k]}, 'changed-output IDs')
for stratum, counts in outcome['by_stratum_cases_initial_final'].items():
    check(counts == [summaries['initial']['strata'][stratum]['cases'], summaries['initial']['strata'][stratum]['complete'], summaries['final']['strata'][stratum]['complete']], 'stratum outcome')

check(initial_receipt['parameters'][ACTIVE]['sha256'] != final_receipt['parameters'][ACTIVE]['sha256'], 'candidate master unchanged')
fields = []; code_arrays = []
for step, receipt in [(0, initial_receipt), (256, final_receipt)]:
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
check(fields[0] == read(accepted_root / 'checkpoint-0256/continuation-field.bin', ACCEPTED_FIELD_SHA), 'initial field differs from accepted175')
for key in fields[0]['metadata']:
    if key != 'payload_sha256': check(fields[0]['metadata'][key] == fields[1]['metadata'][key], 'frozen native binding/policy: ' + key)
crossings = sum(x != y for x, y in zip(*code_arrays))
check(crossings > 0, 'candidate native coefficients unchanged')

early = read(ROOT / 'endpoint-prior175.json')
check(early['cases'] == 175 and len(early['rows']) == 175 and early['continuation_sha256'] == final_receipt['continuation_sha256'], 'early175 endpoint identity')
check({x['id'] for x in early['rows']} == sets['initial'], 'early175 selected success IDs')
early_count = 0
full_refs = {x['id']: x for x in report['final']['rows']}
for ref in early['rows']:
    row = read(ROOT / ref['row_file'], ref['row_sha256'])
    check(row['id'] == ref['id'] and row['canonical'] == 'NOT_RUN', 'early175 actual-only identity')
    for key in ['generated_ids', 'complete', 'eos']:
        check(row[key] == ref[key] == full_refs[ref['id']][key], 'early/full endpoint agreement: ' + key)
    check(row['continuation_sha256'] == final_receipt['continuation_sha256'], 'early175 field identity')
    early_count += row['complete']
check(early_count == early['complete'], 'early175 complete summary')
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
rejected139 = read(BASE / 'm2-cross-145-20261010/runs/resume145-0256-attempt1/report.json', 'be3399c6321144453b7ba055f56aefa42973447fe739ecd8e9d54da248034078')
read(BASE / 'm2-cross-145-20261010/runs/resume145-0256-attempt1/manifest.json', 'e0631ef92be4e28f5f9f0a25bbcb54794bb44c80e0e1123b9928393bc6b36c06')
check(rejected139['final']['complete'] == 139, 'pinned rejected139 headline')
compare('rejected139', rejected139['final'])
accepted145 = read(BASE / 'm2-cross-resume-20261010/runs/resume-0256-attempt1/report.json', 'e3a7e3ebd93c38253afb67f77ad3bb6b64a5e0ac50561f0204e1b83d22b8f4a5')
check(accepted145['final']['complete'] == 145, 'pinned accepted145 headline')
compare('accepted145', accepted145['final'])
compare('accepted175', accepted['final'])
compare('accepted22', accepted22['final'])
compare('original8', accepted22['initial'])
old = COMPARATORS / 'old-u64'
old_report = read(old / 'report.json', '612f8c8123f05b87112a13acd0028e2130436e2b65273f38c6ccb56feefaaf37')
read(old / 'manifest.json', '50a436ead5af213b0d71135ecdc5089833d74586565a8d6e5f92eaac93c37ca5')
old_initial = read(old / 'development-0000.json', '2ce43ce091dab5d6d160c5683c4a75fd4c66c630d4cf9e82d147df40f2348704')
old_final = read(old / 'development-0064.json', 'd2f0cb3e95870936a3b0464c2bead5c514f2a7866f8348c3fc4286e64822cebc')
check(old_report['mode'] == 'continuation_only' and old_report['updates'] == 64, 'oldU64 protocol')
check({x['id'] for x in old_initial['rows'] if x['complete']} == original_ids, 'oldU64 accepted baseline')
check(old_final['continuation_sha256'] == old_report['final_receipt']['continuation_sha256'], 'oldU64 endpoint binding')
compare('old_u64', old_final)
result = {'status': 'PASS_SAVED_EVIDENCE', 'source_commit': report['source_commit'], 'reader_sha256': digest(Path(__file__).read_bytes()), 'report_sha256': digest((ROOT / 'report.json').read_bytes()), 'manifest_sha256': digest((ROOT / 'manifest.json').read_bytes()), 'config_sha256': digest(config_path.read_bytes()), 'config_scope': 'external attempt-argv config; protocol/data/parent cross-checked against sealed receipts', 'checked_rows': 1024, 'endpoints': summaries, 'gained': gained, 'lost': lost, 'retained': retained, 'keep': keep, 'changed_outputs_vs_parent': sum(outputs['initial'][k] != outputs['final'][k] for k in labels), 'comparisons': comparisons, 'resume_verification': {'initial_fractional_master_sha256': digest(initial_master_bytes), 'initial_native_field_sha256': ACCEPTED_FIELD_SHA, 'exact_all512_baseline': True, 'frozen_cache_provenance_equal': True, 'fresh_adam': True, 'local_updates': 256, 'lineage_step': 832, 'early175_complete': early_count}, 'native_cross_state': {'compact_coefficients': 115200, 'packed_bytes': 65536, 'changed_codes': crossings, 'final_nonzero_codes': sum(x != 0 for x in code_arrays[1]), 'minimum_code': min(code_arrays[1]), 'maximum_code': max(code_arrays[1])}, 'objective_verification': {'training_objective': OBJECTIVE, 'phase_policy': PHASE_POLICY, 'update_scalars_finite': True, 'first_batch_objective': updates[0]['objective_loss'], 'last_batch_objective': updates[-1]['objective_loss'], 'minimum_batch_objective': min(u['objective_loss'] for u in updates), 'maximum_batch_objective': max(u['objective_loss'] for u in updates), 'scope': 'Policy/checkpoint/update metadata and finite saved objective scalars checked; training objective and gradient not recomputed; first/last are different scheduled batches'}, 'schedule': 'PASS seed1001 full512 repeated4,256x8,26656positions', 'frozen_parent_receipts': 'PASS', 'scope': 'Saved hashes/IDs/prefix chains/EOS/frozen membership and native Q4 payloads checked. No model, backward, tokenizer decoding or native score regeneration. Full BLAKE3 seal verification is producer evidence; reader checks file inventory/sizes. Prior comparisons use pinned saved reports, not fresh model evaluation.'}
print(json.dumps(result, indent=2))
