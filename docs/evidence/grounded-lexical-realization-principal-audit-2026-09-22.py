#!/usr/bin/env python3
"""Read-only PR1349 evidence audit; not a model or product implementation.

Reconstructs sealed populations, actual emitted bytes, fitted action rows,
historical statement semantics and legacy retention without trusting pass flags.
The independent PR1348 history/Hamilton calculator is reused explicitly.
No report root is mutated. Requires the optional local blake3 evidence library.
"""
import argparse
import collections
import datetime
import hashlib
import importlib.util
import json
import pathlib
import subprocess

import blake3

BASE = pathlib.Path('/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20')
SOURCE = '27fdc1798f0a8536238a72062ca269f976085e73'
TOKENIZER = pathlib.Path('/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json')
TOKENIZER_SHA = '9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c'
EOS = 4294967294
load = lambda p: json.loads(pathlib.Path(p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()


def decoder(path):
    raw = path.read_bytes()
    if sha(raw) != TOKENIZER_SHA:
        raise ValueError('tokenizer source identity differs')
    vocab = {i: t for t, i in json.loads(raw)['model']['vocab'].items() if i < 4096}
    # Inverse GPT byte-unicode alphabet; no alternate tokenization is performed.
    bs = list(range(33, 127)) + list(range(161, 173)) + list(range(174, 256))
    cs, n = list(bs), 0
    for b in range(256):
        if b not in bs:
            bs.append(b)
            cs.append(256 + n)
            n += 1
    inverse = dict(zip(map(chr, cs), bs))
    def decode(tokens):
        return bytes(inverse[c] for t in tokens if t != EOS for c in vocab[t]).decode('utf-8').strip()
    return decode


def key(relation, history, derived, prior_differs, copy_stage, emitted_bucket):
    mask, salt = (1 << 64) - 1, 0x9e3779b97f4a7c15
    h = salt
    for v in (relation, history, derived, prior_differs, copy_stage, emitted_bucket):
        h ^= (int(v) + salt) & mask
        h = (h + (h << 13)) & mask
        h ^= h >> 7
        h = (h + (h << 17)) & mask
        h = ((h << 29) | (h >> 35)) & mask
    return h


def normalize_legacy(value):
    v = json.loads(json.dumps(value))
    frame = v.get('final_frame')
    if isinstance(frame, dict):
        for k in ('contract', 'vocabulary_words', 'prelude_words', 'version'):
            frame.pop(k, None)
        for k in ('contract', 'realization_sha256'):
            frame.get('binding', {}).pop(k, None)
    for event in v.get('effects', []):
        event.pop('realization', None)
    return v


def divergence(a, b):
    aa = [e for e in a['effects'] if e.get('emitted') is not None]
    bb = [e for e in b['effects'] if e.get('emitted') is not None]
    prefix = 0
    for x, y in zip(aa, bb):
        if x['emitted'] != y['emitted']:
            break
        prefix += 1
    def side(events):
        if prefix >= len(events):
            return None
        e = events[prefix]
        return dict(token=e['emitted'], decision=e.get('realization'))
    return dict(same_request=a['request'] == b['request'], fixed_generated_prefix_tokens=prefix,
                left=side(aa), right=side(bb))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--base', type=pathlib.Path, default=BASE)
    ap.add_argument('--source', type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[2])
    ap.add_argument('--output', type=pathlib.Path, required=True)
    args = ap.parse_args()
    parent_path = args.source/'docs/evidence/observed-computation-lifecycle-principal-audit-2026-09-22.py'
    spec = importlib.util.spec_from_file_location('parent_audit', parent_path)
    parent = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(parent)
    decode = decoder(TOKENIZER)
    root = args.base/'grounded-lexical-realization-9'
    result = load(root/'result.json')
    section = result['grounded_realization']
    model = load(root/'artifacts/realization.json')
    rows = [json.loads(line) for line in (root/'rows.jsonl').read_text().splitlines()]
    out = dict(schema='uor-r4.grounded-lexical-realization-principal-audit/1',
               reviewed_source=SOURCE, report_root=str(root),
               method='Read-only independent byte decoding, saved action/row reconstruction, raw history oracle and seal checks. No model fitting or product execution.',
               calculator_dependency=dict(path=str(parent_path.relative_to(args.source)), sha256=sha(parent_path.read_bytes())),
               tokenizer=dict(path=str(TOKENIZER), sha256=TOKENIZER_SHA, vocabulary='Unchanged dense first4096 source IDs; independent byte decoding only.'))
    attempts = []
    for n in range(1, 10):
        p = args.base/f'grounded-lexical-realization-{n}'
        a = load(p/'attempt.json')
        item = dict(root=p.name, claimed_at_utc=datetime.datetime.fromtimestamp(int(a['claimed_at']), datetime.timezone.utc).isoformat())
        if not (p/'manifest.json').exists():
            item.update(status='UNSEALED_PARTIAL', files=sorted(str(x.relative_to(p)) for x in p.rglob('*') if x.is_file()))
        else:
            m, r = load(p/'manifest.json'), load(p/'result.json')
            listed = {x['path'] for x in m['files']}
            actual = {str(x.relative_to(p)) for x in p.rglob('*') if x.is_file()}
            errors = []
            for f in m['files']:
                data = (p/f['path']).read_bytes()
                if len(data) != f['bytes'] or blake3.blake3(data).hexdigest() != f['blake3']:
                    errors.append(f['path'])
            item.update(status='SEALED', listed_files=len(listed), missing=sorted(listed-actual), unlisted=sorted(actual-listed-{'manifest.json'}), seal_errors=errors,
                        realization_sha256=sha((p/'artifacts/realization.json').read_bytes()),
                        realized_outputs={c['case']:decode(c['emitted']) for c in r['grounded_realization']['cases']},
                        examples=r['grounded_realization']['artifact']['examples'], table_rows=r['grounded_realization']['artifact']['table_rows'],
                        root_reported_pass=r['checks_all_expected'], mixed_reported_pass=r['grounded_realization'].get('mixed_interaction',{}).get('ok'),
                        rows_sha256=sha((p/'rows.jsonl').read_bytes()), executable_sha256=r['running_source']['executable_sha256'],
                        source_hashes=r['running_source']['source_files'],
                        failed_reported_checks={k:v for k,v in r['grounded_realization']['checks'].items() if not v})
        attempts.append(item)
    out['attempts'] = attempts
    out['exposure'] = dict(first_complete_realized_population=3, final_realizer_first_exposed=7,
                           exact_final_realizer_sha256=sha((root/'artifacts/realization.json').read_bytes()),
                           rows_roots3_through9_byte_identical=len({x['rows_sha256'] for x in attempts if x['status']=='SEALED'})==1,
                           final_status='Development-exposed replay; root9 is not a new population or fresh withheld realization test.',
                           correction='Roots1 and2 are both partial claims; roots3 through9 are seven sealed attempts. Root5 mixed check fails but root pass is true; root8 context-report check fails; root9 repairs the report predicate.')
    source_checks = []
    for e in result['running_source']['source_files']:
        raw = subprocess.check_output(['git','show',SOURCE+':'+e['path']],cwd=args.source)
        source_checks.append(dict(path=e['path'], reported_sha256=e['sha256'], committed_sha256=sha(raw), matches=sha(raw)==e['sha256']))
    lexpath='crates/uor-r4-core/src/native_geometric/learner/lexical_realization.rs'
    out['source_provenance'] = dict(recorded=source_checks,
        omitted_new_module=dict(path=lexpath, committed_sha256=sha(subprocess.check_output(['git','show',SOURCE+':'+lexpath],cwd=args.source)), status='Not bound in saved running_source.source_files'),
        recorded_git_rev=result['running_source']['git_rev'], recorded_dirty=result['running_source']['git_dirty'], recorded_base=result['base'],
        qualification='Branch name plus dirty flag is not a commit identity. Existing per-file hashes are compared to reviewed commit; new module was omitted. Sealed report base is stale.')
    table = dict(model['table'])
    realization_rows, reconstruction_errors = [], []
    cases = section['cases']+[dict(section['derived_case'],case='derived')]
    for case in cases:
        trace = [e for e in case['effects'] if e.get('emitted') is not None]
        emitted = [e['emitted'] for e in trace]
        copied = [e['emitted'] for e in trace if (e.get('realization') or {}).get('action')=='Copy']
        inserts = [e for e in trace if isinstance((e.get('realization') or {}).get('action'),dict)]
        choices=[]
        cursor=words=0
        derived=case['case']=='derived'
        values=[x.get('value') for x in case['observed']] if not derived else []
        prior_differs=len(values)>1 and values[-1]!=values[-2]
        for event in case['effects']:
            dec=event.get('realization')
            if not dec:
                continue
            action=dec['action']
            stage=0 if cursor==0 else (2 if cursor==len(copied) else 1)
            actual_key=key(0,0,derived,prior_differs,stage,min(words,3))
            if actual_key!=dec['key']:
                reconstruction_errors.append([case['case'],'context key differs from independent causal trace'])
            scores=table.get(dec['key'])
            chosen=None if scores is None else max(range(len(scores)),key=lambda i:scores[i])
            expected='Copy' if chosen==0 else ('Stop' if chosen==1 else ({'Insert':chosen-2} if chosen is not None else None))
            if dec['from_table'] and action!=expected:
                reconstruction_errors.append([case['case'],'from_table action differs from fitted row'])
            if isinstance(action,dict) and event['emitted']!=model['slots'][action['Insert']]:
                reconstruction_errors.append([case['case'],'Insert slot/token mismatch'])
            choices.append(dict(action=action, from_table=dec['from_table'], key=dec['key'], table_argmax=expected))
            cursor+=int(action=='Copy' and event['emitted'] is not None)
            words+=int(isinstance(action,dict) and event['emitted'] is not None)
        answer=decode(emitted)
        selected=[e for e in case['effects'] if e.get('selected_value') is not None]
        last_value=bytes(selected[-1]['selected_value']).decode() if selected else None
        if emitted!=case['emitted'] or answer!=case['answer'] or decode(copied)!=last_value:
            reconstruction_errors.append([case['case'],'actual bytes/effect/copied value mismatch'])
        realization_rows.append(dict(case=case['case'],request=case['request'],decoded_answer=answer, copied_tokens=copied,
            copied_bytes=decode(copied), selected_final_value=last_value, inserted_tokens=[e['emitted'] for e in inserts],
            inserted_bytes=[decode([e['emitted']]) for e in inserts], decisions=choices,
            legacy_payload_equal=case.get('legacy_answer')==decode(copied) if 'legacy_answer'in case else None))
    out['realized_outputs']=dict(cases=realization_rows, reconstruction_errors=reconstruction_errors,
        slots_decode=[decode([t]) for t in model['slots']],
        class_values=sorted({e['realization']['evidence_class'] for c in cases for e in c['effects'] if e.get('realization')}))
    by_name={c['case']:c for c in cases}
    out['causal_pairs']={name:divergence(by_name[a],by_name[b]) for name,a,b in [('history','direct','superseded'),('unfamiliar_history','fresh_direct','fresh_superseded'),('computation','direct','derived'),('reassertion','direct','reasserted')]}
    out['causal_pairs']['qualification']='History pair is real causal information flow but uses current-value predecessor difference as a tense target. The consumed comparison changes the request and derived flag; it does not isolate computed value or operator-state content.'
    out['semantic_audit']=dict(
        current_query_contradictions=[dict(case=c['case'],request=c['request'],facts=[x['text'] for x in c['observed']],current_value=bytes([e for e in c['effects'] if e.get('selected_value') is not None][-1]['selected_value']).decode(),answer=decode(c['emitted'])) for c in section['cases'] if c['case'] in ('superseded','fresh_superseded')],
        mixed_record=section['mixed_interaction']['turns'][-1],
        interpretation='Quarry then Harbor makes Harbor current, not former; `it was Harbor` is not licensed by its differing predecessor. Initial Harbor after later corrections is historical but emits `it is Harbor`. The authored target rule confuses prior change with selected-answer temporal status. This is a supervision/semantic failure, not failure to learn the supplied labels.',
        expected_next='Correct the proposition/temporal feature and independent answer semantics before widening vocabulary. Hold exact payload/scope/version contracts separate from lexical tense.')
    # Reuse the independent algebra/history oracle; do not trust row.expected/matched.
    errors=[]; counts=collections.defaultdict(lambda:dict(requests=0,matched=0)); selected_count=complete_count=0
    for i,row in enumerate(rows):
        recs=parent.world_records(row['world'])
        expected,ids,operand,derived=parent.expected_request(recs,row['person'],row['ops'])
        matched=row['terminal']=='Complete' and row['answer']==expected and row['reads']==len(ids)
        if row['expected']!='Answer('+expected+')' or row['matched']!=matched:
            errors.append([i,'independent algebra target/match mismatch'])
        group=counts[row['panel']+'/'+row['arm']];group['requests']+=1;group['matched']+=matched
        for e in row['effects']:
            if e.get('selected_record') is not None:
                selected_count+=1;rec=recs[e['selected_record']-1]
                if e['selected_commit']!=rec['commit'] or bytes(e['selected_value']).decode()!=rec['value']:
                    errors.append([i,'selected record provenance mismatch'])
        if row['terminal']=='Complete':
            complete_count+=1
            if decode(row['final_frame']['emitted'])!=row['answer']:
                errors.append([i,'decoded answer mismatch'])
    priorrows=[json.loads(line) for line in (args.base/'observed-computation-lifecycle-4/rows.jsonl').read_text().splitlines()]
    preserved=load(root/'preservation.json')
    out['legacy_retention']=dict(rows=len(rows),independently_recomputed_counts=dict(counts),errors=errors,
        selected_records_checked=selected_count,complete_answers_independently_decoded=complete_count,
        parent_rows_equal_after_declared_schema_additions=sum(normalize_legacy(a)==normalize_legacy(b) for a,b in zip(rows,priorrows)),
        equality_exclusions='Session version4→5; added contract, vocabulary_words, prelude_words, binding.contract, binding.realization_sha256; added null realization effect.',
        realizer_empty_sha_frames=sum(r['final_frame']['binding']['realization_sha256']==sha(b'') for r in rows),
        legacy_contract_frames=sum(r['final_frame']['contract']==0 for r in rows),
        effects_with_realization=sum(e.get('realization') is not None for r in rows for e in r['effects']),
        qualification='All392 rows use load_with_computation with NO realization artifact. This is retained modified-runtime/component behavior, not a full RealizedV1 bundle control campaign. Five explicit short LegacyWords examples elsewhere do bind the realization artifact.',
        preservation={arm:parent.audit_preservation(preserved[arm]) for arm in ('candidate','prior_migrated','ablation_computation_forms_only')})
    snapshots=[json.loads(bytes(x)) for x in load(root/'realized_reload/checkpoints.json')]
    resumed=section['restart']['resumed']
    out['restart']=dict(saved_snapshots=len(snapshots),
        states=[dict(cursor=s['cursor'],pending=s['pending'],vocabulary_words=s['vocabulary_words'],emitted=s['emitted'],terminal=s['terminal']) for s in snapshots],
        inprocess_reported_final_token_terminal_equal=sum(r['emitted']==snapshots[-1]['emitted'] and r['terminal']==snapshots[-1]['terminal'] for r in resumed),
        saved_parent_final_frame=snapshots[-1],
        separate_process='SUMMARY_ONLY: runner computes full child equality, but saves only fresh_process_matches boolean; child full output is not in sealed report. New principal replay needed for independent child frame verification.',
        mixed='Only eight restart booleans/count are saved; actual mixed snapshots and resumed frames are not retained.',
        ownership='Pinned mixed answer starts before correction but capture/emission occur afterward. No saved realized mid-copy eviction/correction run; simple restart store is unchanged.')
    out['missing_measurements']=['No frozen E/S comparator run','No source-separated natural-text fit/evaluation or served likelihood','No fresh withheld realized population','No complete-response latency/RAM/parameter-traffic profile','No evidence-class ablation with variation: class1 everywhere and excluded from key','No saved clean-parent baseline/failing-test logs among sealed roots']
    out['resource_record']=load(args.source/'docs/evidence/grounded-lexical-realization-charge-2026-09-22.json')
    out['resource_qualification']='Charge is declared approximate wall-time accounting, not reconstructed stopwatch events. No second target tree does not imply low new disk usage; reported shared-cache growth is about9GiB and below-reserve free space is explicit.'
    args.output.write_text(json.dumps(out,indent=2,sort_keys=True)+'\n')
    print(json.dumps(dict(output=str(args.output),sealed=sum(x['status']=='SEALED' for x in attempts),row_errors=len(errors),realization_reconstruction_errors=len(reconstruction_errors),legacy_empty_realizer_frames=out['legacy_retention']['realizer_empty_sha_frames'])))


if __name__=='__main__':
    main()
