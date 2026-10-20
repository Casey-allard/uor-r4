#!/usr/bin/env python3
"""Read-only PR1348 evidence audit, independent of the Rust product/answer oracle.

Uses integer Hamilton multiplication and the declared authored worlds, replays raw
scoped statements in a separate history oracle, checks actual effect/capture fields,
and audits sealed files, source identities, exposure history and checkpoint frames.
This is an evidence calculator, not a Python model or product dependency.
"""
import argparse
import collections
import datetime
import hashlib
import json
import pathlib
import re
import subprocess

try:
    import blake3
except ImportError:
    blake3 = None

DEFAULT_BASE = '/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20'
SOURCE_COMMIT = 'de5991fd10fa9c50f7d3e95a3d94299fc3af45fe'
LABELS = ['Alma', 'Bert', 'Cora', 'Dane', 'Elin', 'Frey', 'Gwen', 'Holt']
PEOPLE = {
    'development': ['Mara', 'Ivo', 'Cedar', 'Oren'],
    'final': ['Una', 'Pia', 'Soren', 'Kestrel'],
    'fresh_withheld': ['Juno', 'Rhea', 'Silas', 'Perrin'],
}
DESTS = {
    'development': ['Bramble', 'Quarry', 'Vale', 'Marsh', 'Tarn', 'Ledge', 'Ridge', 'Stone'],
    'final': ['Cobalt', 'Mica', 'Dune', 'Basalt', 'Flint', 'Slate', 'Amber', 'Onyx'],
    'fresh_withheld': ['Larkspur', 'Nettle', 'Umber', 'Vellum', 'Wren', 'Yarrow', 'Zephyr', 'Cinder'],
}
# The first eight canonical signed-axis roots, and the first noncommuting pair.
# These declared coordinates do not read the fitted artifact to determine targets.
AXES = [tuple(s if j == axis else 0 for j in range(4)) for axis in range(4) for s in (-1, 1)]
OPS = {'i': 2, 'j': 4, 'e': 1}
REDIRECT = {0: 5, 1: 2, 4: 3, 5: 6, 8: 6, 9: 1}
load = lambda p: json.loads(pathlib.Path(p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
canonical = lambda x: json.dumps(x, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()


def multiply(a, b):
    w, x, y, z = AXES[a]
    v, i, j, k = AXES[b]
    return AXES.index((w*v-x*i-y*j-z*k, w*i+x*v+y*k-z*j,
                       w*j-x*k+y*v+z*i, w*k+x*j-y*i+z*v))


def world_records(world):
    panel = 'development' if world < 4 else ('final' if world < 8 else 'fresh_withheld')
    people, dests = PEOPLE[panel], DESTS[panel]
    assignments = [0, 5, 2, 7] if world == 4 else [6, 1, 4, 3]
    records = []
    for index, (person, label) in enumerate(zip(people, assignments)):
        continues = world >= 8 and index == 0
        records.append(dict(entity=person, value=people[1] if continues else LABELS[label], continues=continues))
    for index, label in enumerate(LABELS):
        continues = index == REDIRECT[world]
        records.append(dict(entity=label, value=LABELS[(index+1) % 8] if continues else dests[index], continues=continues))
    for index, record in enumerate(records):
        record.update(id=index+1, commit=index+1)
    return records


def expected_request(records, person, ops):
    by_key = {r['entity']: r for r in records}
    entity, done, reads, operand, result = person, False, [], None, None
    for _ in range(8):
        rec = by_key[entity]
        reads.append(rec['id'])
        if rec['continues']:
            entity = rec['value']
            continue
        if ops and not done:
            operand = rec
            state = LABELS.index(rec['value'])
            for op in ops:
                state = multiply(OPS[op], state)
            result = LABELS[state]
            entity, done = result, True
            continue
        return rec['value'], reads, operand, result
    raise ValueError('declared world exhausted')


def parse_statement(text):
    match = re.fullmatch(r"(.+?)'s (office|project) (is|became|might be) (.+)", text)
    if match:
        return dict(entity=match[1], relation=int(match[2] == 'project'), value=match[4],
                    continues=False, write=match[3] != 'might be',
                    intent={'is': 0, 'became': 1, 'might be': 2}[match[3]], is_question=False)
    match = re.fullmatch(r'(.+?) (office|project) follows (.+)', text)
    if match:
        return dict(entity=match[1], relation=int(match[2] == 'project'), value=match[3],
                    continues=True, write=True, intent=0, is_question=False)
    if text.startswith('What '):
        entity, relation, history = parse_question(dict(kind='ask', text=text))
        return dict(entity=entity, relation=relation, value=None, continues=False, write=False,
                    intent={'Current': 0, 'PreviousAssertion': 1, 'Initial': 2}[history], is_question=True)
    raise ValueError('unrecognized authored statement ' + text)


def parse_question(row):
    if row['kind'] == 'ask_view':
        m = re.fullmatch(r'exact (\w+) of "(.+)" relation (\d)', row['text'])
        return m[2], int(m[3]), m[1]
    m = re.fullmatch(r"What (?:is|was) (.+?)'s (office|project)( before| originally)?\?", row['text'])
    return m[1], int(m[2] == 'project'), {None: 'Current', ' before': 'PreviousAssertion', ' originally': 'Initial'}[m[3]]


def history_answer(chains, scope, entity, relation, history):
    visited, hops = [], 0
    while True:
        if entity in visited:
            return 'Cycle', '', hops
        chain = chains.get((scope, entity, relation), [])
        if not chain:
            return 'Unresolved', '', hops
        index = len(chain)-1
        if hops == 0:
            if history == 'Initial':
                index = 0
            elif history == 'PreviousAssertion':
                index -= 1
            elif history == 'PreviousDistinctValue':
                index -= 1
                while index >= 0 and chain[index][0] == chain[-1][0]:
                    index -= 1
        if index < 0:
            return 'NoHistory', '', hops
        if index < len(chain)-8:
            return 'Evicted', '', hops
        value, cont = chain[index]
        if not cont:
            return 'Complete', value, hops+1
        visited.append(entity)
        entity, hops = value, hops+1
        if hops > 8:
            return 'Exhausted', '', hops


def audit_preservation(item):
    groups = collections.defaultdict(list)
    for row in item['rows']:
        groups[row['script']].append(row)
    errors, failed_rows = [], []
    n = matches = ingest = ingest_matches = reported_errors = language = api = 0
    for script, rows in groups.items():
        chains = {}
        for row in rows:
            reported_errors += row.get('error') is not None
            if row['kind'] == 'ingest':
                ingest += 1
                st = parse_statement(row['text'])
                if st['write']:
                    chains.setdefault((row['scope'], st['entity'], st['relation']), []).append((st['value'], st['continues']))
                observed = row['observed']
                matched = row['error'] is None and row['wrote'] == st['write']
                matched = matched and all(observed.get(k) == st[k] for k in ('entity', 'relation', 'continues', 'intent', 'is_question'))
                matched = matched and (not st['write'] or observed.get('value') == st['value'])
                ingest_matches += matched
                if row['expected_write'] != st['write'] or row['observation_and_write_match'] != matched:
                    errors.append([script, row['turn'], 'ingest flag differs from independent raw-text oracle'])
                continue
            n += 1
            language += row['kind'] == 'ask'
            api += row['kind'] == 'ask_view'
            entity, rel, hist = parse_question(row)
            terminal, value, depth = history_answer(chains, row['scope'], entity, rel, hist)
            expected = f'Complete {{ value: "{value}", hops: {depth} }}' if terminal == 'Complete' else terminal
            matched = row['error'] is None and row['terminal'] == terminal
            matched = matched and (terminal != 'Complete' or (row['answer'] == value and row['final_frame']['hop']+1 == depth))
            if row['expected'] != expected or row['matched'] != matched:
                errors.append([script, row['turn'], 'question expectation or flag differs from raw-text oracle'])
            matches += matched
            if not matched:
                failed_rows.append(dict(script=script, turn=row['turn'], text=row['text'], expected=expected, answer=row['answer'], error=row['error']))
    counts = dict(questions=n, matched=matches, language_questions=language, api_questions=api,
                  ingest_total=ingest, ingest_observation_and_write_correct=ingest_matches, errors=reported_errors)
    return dict(recomputed=counts, reported_summary_matches=all(item[k] == v for k, v in counts.items()),
                independent_oracle_errors=errors, failed_queries=failed_rows)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--base', default=DEFAULT_BASE)
    parser.add_argument('--source', default=str(pathlib.Path(__file__).resolve().parents[2]))
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    base, source = pathlib.Path(args.base), pathlib.Path(args.source)
    root = base/'observed-computation-lifecycle-4'
    result, preserved = load(root/'result.json'), load(root/'preservation.json')
    rows = [json.loads(line) for line in (root/'rows.jsonl').read_text().splitlines()]
    out = dict(schema='uor-r4.observed-computation-lifecycle-principal-audit/1', report_root=str(root),
               method='Read-only saved evidence; independent integer Hamilton action and raw scoped-history oracle. No fitting or product execution.')
    seals, exposures = [], []
    artifact_sets = []
    for number in range(1, 5):
        p = base/f'observed-computation-lifecycle-{number}'
        manifest, attempt = load(p/'manifest.json'), load(p/'attempt.json')
        listed = {r['path'] for r in manifest['files']}
        actual = {str(x.relative_to(p)) for x in p.rglob('*') if x.is_file()}
        bad_sizes, bad_hashes = [], []
        for entry in manifest['files']:
            data = (p/entry['path']).read_bytes()
            if len(data) != entry['bytes']:
                bad_sizes.append(entry['path'])
            if blake3 and blake3.blake3(data).hexdigest() != entry['blake3']:
                bad_hashes.append(entry['path'])
        seals.append(dict(root=p.name, files=len(listed), missing=sorted(listed-actual),
                          unlisted=sorted(actual-listed-{'manifest.json'}), bad_sizes=bad_sizes,
                          blake3_status='VERIFIED' if blake3 else 'UNAVAILABLE: Python blake3 not installed', bad_hashes=bad_hashes))
        r = load(p/'result.json')
        artifact_sets.append({x.name: sha(x.read_bytes()) for x in (p/'artifacts').glob('*.json')})
        exposures.append(dict(root=p.name, claimed_at_utc=datetime.datetime.fromtimestamp(int(attempt['claimed_at']), datetime.timezone.utc).isoformat(),
                              sealed_at_utc=datetime.datetime.fromtimestamp(int(manifest['sealed_at']), datetime.timezone.utc).isoformat(),
                              panels=r['panels'], checks_all_expected=r['checks_all_expected'],
                              rows_sha256=sha((p/'rows.jsonl').read_bytes())))
    out['seals'] = seals
    out['exposure'] = dict(attempts=exposures, all_artifacts_roots1_through4_identical=all(x == artifact_sets[0] for x in artifact_sets),
                           roots2_through4_rows_byte_identical=len({x['rows_sha256'] for x in exposures[1:]}) == 1,
                           interpretation='The32-case population first passes in root2 before the mixed-session read-count oracle repair. Roots3 and4 replay identical model parameters and identical rows. Root4 is exposed replay, not a new final draw. The24-case final population was already exposed under PR1347.')
    out['sources'] = []
    for entry in result['running_source']['source_files']:
        data = subprocess.check_output(['git', 'show', SOURCE_COMMIT+':'+entry['path']], cwd=source)
        out['sources'].append(dict(path=entry['path'], recorded_sha256=entry['sha256'], committed_matches=sha(data) == entry['sha256']))
    executable = base/'observed-computation-lifecycle-delivery/competitive-reader-de5991fd'
    out['executable'] = dict(path=str(executable), sha256=sha(executable.read_bytes()),
                             recorded_matches=sha(executable.read_bytes()) == result['running_source']['executable_sha256'])
    errors, counts, selected, emissions, computations = [], collections.defaultdict(lambda: dict(requests=0, complete=0)), 0, 0, 0
    backend = load(root/'artifacts/artifact.json')
    for index, row in enumerate(rows):
        recs = world_records(row['world'])
        expected, ids, operand, derived = expected_request(recs, row['person'], row['ops'])
        matched = row['terminal'] == 'Complete' and row['answer'] == expected and row['reads'] == len(ids)
        if row['expected'] != 'Answer('+expected+')' or row['matched'] != matched:
            errors.append([index, 'independent Hamilton/world expectation or match flag differs'])
        key = row['panel']+'/'+row['arm']
        counts[key]['requests'] += 1
        counts[key]['complete'] += matched
        frame, effects = row['final_frame'], row['effects']
        read_events = [e for e in effects if e['action'] == 'Read' and e['selected_record'] is not None]
        for event in read_events:
            selected += 1
            rec = recs[event['selected_record']-1]
            if event['selected_commit'] != rec['commit'] or bytes(event['selected_value']).decode() != rec['value']:
                errors.append([index, 'selected record not declared observed world'])
        if row['terminal'] == 'Complete':
            emissions += 1
            cap = frame['captured']
            emitted = [e['emitted'] for e in effects if e['emitted'] is not None]
            if frame['emitted'] != cap['payload']+[frame['eos']] or emitted != frame['emitted'] or bytes(cap['value']).decode() != row['answer']:
                errors.append([index, 'reported answer not actual owned lexical emission'])
        if row['arm'] == 'primary_signed' and row['ops']:
            computations += 1
            c = row['computed']
            state = backend['payload_state'][LABELS.index(operand['value'])]
            applies = [e for e in effects if e['action'] == 'Apply' and e['op_label'] is not None]
            if len(applies) != len(row['ops']):
                errors.append([index, 'wrong primitive Apply count'])
            for op, event in zip(row['ops'], applies):
                state = multiply(OPS[op], state)
                if state != event['computed_state']:
                    errors.append([index, 'intermediate state not integer Hamilton product'])
            if c['state'] != state or bytes(c['derived_key']).decode() != derived or c['derived_label'] != LABELS.index(derived):
                errors.append([index, 'computed state or derived key mismatch'])
            if c['source_record'] != operand['id'] or c['source_commit'] != operand['commit'] or bytes(c['source_entity']).decode() != operand['entity']:
                errors.append([index, 'operand ownership mismatch'])
            if [e['selected_record'] for e in read_events] != ids or not c['consumed']:
                errors.append([index, 'actual consumed read path mismatch'])
    out['rows'] = dict(count=len(rows), counts=dict(counts), selected_reads_checked=selected,
                        complete_owned_emissions_checked=emissions, primary_computations_checked=computations,
                        errors=errors, oracle='Targets reconstructed from declared world plus signed-axis integer Hamilton algebra, not row.expected or learned table.')
    ingest_errors = []
    for index, row in enumerate(result['learned_ingest']['receipts']):
        st = parse_statement(row['text'])
        if not row['committed'] or not all(row[k] == st[k] for k in ('entity', 'value', 'relation', 'intent', 'continues')):
            ingest_errors.append(index)
    out['learned_ingest'] = dict(actual_receipts=len(result['learned_ingest']['receipts']), independent_errors=ingest_errors,
                                  correction='The result prose and user handoff say48; actual sealed root contains72 observed assertions across six worlds.')
    out['preservation'] = {arm: audit_preservation(preserved[arm]) for arm in ('candidate', 'prior_migrated', 'ablation_computation_forms_only')}
    prior = load(base/'consumed-geometric-state-principal-1/preservation.json')
    fields = ('kind', 'text', 'scope', 'error', 'expected', 'matched', 'answer', 'terminal', 'wrote', 'observation_and_write_match', 'observed')
    old = prior['candidate']['rows']
    narrow = preserved['ablation_computation_forms_only']['rows']
    out['ablation_baseline_parity'] = dict(rows=len(narrow), matching_rows=sum(all(a.get(f) == b.get(f) for f in fields) for a, b in zip(old, narrow)), identical_denominators=len(old) == len(narrow),
                                           scope='Actual saved baseline row observations/answers/errors match; narrow ablation parameter bytes were not separately saved.')
    checkpoints = [json.loads(bytes(x)) for x in load(root/'reload/checkpoints.json')]
    child = result['consumption']['reload']['child']
    out['restart'] = dict(checkpoints=[dict(index=i, pending=f['pending'], applied=f['computation'] and f['computation']['applied'],
                                             derived_key=f['computation'] and bytes(f['computation']['derived_key']).decode(),
                                             consumed=f['computation'] and f['computation']['consumed'],
                                             captured_hop=f['captured'] and f['captured']['read_hop']) for i, f in enumerate(checkpoints)],
                          independently_equal_final_frames=sum(r['final_frame'] == child['final_frame'] for r in child['resumed']),
                          phases_match_snapshots=[f['pending'] for f in checkpoints] == [r['phase'] for r in child['resumed']],
                          explicit_grounded_unconsumed_snapshots=sum(bool(f['computation'] and f['computation']['derived_key'] and not f['computation']['consumed']) for f in checkpoints),
                          scope='Ten saved checkpoints resume to one identical complete frame. The store for this separate-process check is typed cgs_memory; the request is raw text. Snapshot3 is after operations but before finalization; snapshot4 is after atomic grounding/consumption. No grounded-unconsumed boundary is serialized.')
    out['typed_comparator'] = dict(status='SUMMARY_ONLY', reason='The runner compares matched and answer in memory, but discards typed rows. Saved receipts support only reported counters, not independent replay of typed trace or record identity.')
    charge = load(base/'observed-computation-lifecycle-delivery/uor-observed-charge.json')
    out['submitted_accounting'] = dict(receipt=charge, initial_authoritative_ledger_observed_by_principal={'cumulative_ms': 337430864, 'limit_ms': 344500000},
                                        missing_submitted_debit_ms=2400000,
                                        interpretation='Receipt records the approximate charge but did not update the authoritative owner ledger. Principal must reconcile once, without recharging on audit replay.')
    out['unrelated_tests'] = dict(status='UNVERIFIED_SUBMITTED_ATTRIBUTION', reason='Preserved focused logs show27+23 module and25 runner passes/1ignored; no saved broad-failure log or parent execution proves the reported four failures pre-existing. Unchanged module source alone is insufficient attribution.')
    pathlib.Path(args.output).write_text(json.dumps(out, indent=2, sort_keys=True)+'\n')
    print(json.dumps(dict(output=args.output, row_errors=len(errors), preservation_errors={k: len(v['independent_oracle_errors']) for k, v in out['preservation'].items()}, ingest_errors=ingest_errors, blake3_available=bool(blake3))))


if __name__ == '__main__':
    main()
