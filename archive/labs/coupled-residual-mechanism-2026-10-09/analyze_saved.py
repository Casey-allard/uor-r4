#!/usr/bin/env python3
"""Read authenticated saved observations; never run a model or propose coefficients."""
import argparse
import collections
import hashlib
import json
import pathlib
import resource
import subprocess
import time

RUN_MANIFEST = 'e7667312b7d18b502e72b0fc41bf2d7d52a4977a020501ecd527da6ef628f9aa'
RUN_REPORT = '0aa9b3c0c2d1defaaa6d93611a78fb348be0016aefe5d7548563250b9a687b8c'
CLAIM = pathlib.Path('/workspace/uor-r4/codex/sol-prefix-margin-boundary/claim-report')
CLAIM_SHA = '394ad004859ae8d9baaf04510f230fb725bbc2f7caabe3c7abe26b1cf70ea208'
SEAL = pathlib.Path('/root/codex/prototype-target/release/examples/native_historical_version')
SEAL_SHA = '2691e33e5b1ce9c29b12ddbf5e30a3d857d30185d69db49402ed1a53f7c4320a'
VERIFIER_SHA = 'd90411116c702dc4149fc055706f34c276c1ead29c5b0ec95600c30cc976326d'

def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1048576), b''):
            h.update(block)
    return h.hexdigest()

def read(path):
    return json.loads(path.read_bytes())

def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()

def require(ok, reason):
    if not ok:
        raise ValueError(reason)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('run', type=pathlib.Path)
    parser.add_argument('verifier', type=pathlib.Path)
    parser.add_argument('output', type=pathlib.Path)
    args = parser.parse_args()
    started = time.monotonic()
    require(sha(CLAIM) == CLAIM_SHA, 'Rust exclusive claimant identity')
    subprocess.run([str(CLAIM), str(args.output)], check=True)
    require(sha(SEAL) == SEAL_SHA, 'Rust sealer identity')
    require(sha(args.run/'manifest.json') == RUN_MANIFEST, 'pinned completed manifest')
    require(sha(args.run/'report.json') == RUN_REPORT, 'pinned completed report')
    require(sha(args.verifier) == VERIFIER_SHA, 'actual Rust report verifier')
    subprocess.run([str(args.verifier), 'verify-report', str(args.run)], check=True)
    report = read(args.run/'report.json')
    authority = read(args.run/'episode-objective-authority.json')
    targets = [r['target'] for r in authority['physical_frames'][:15]]
    require([r['position'] for r in authority['physical_frames'][:15]] == list(range(15)), 'all task positions in order')
    phases = []
    post_groups = collections.defaultdict(list)
    inputs = []
    files = sorted(args.run.glob('candidate-row-*-position-*.json'))
    require(len(files) == 391, 'all saved native rows')
    for path in files:
        saved = read(path)
        n = saved['native']
        ident = [saved['input_index'], saved['position'], saved['target_label_only']]
        post_groups[tuple(n['post_state'])].append(ident)
        inputs.append({'file': path.name, 'sha256': sha(path), 'bytes': path.stat().st_size})
        if saved['input_index'] != 245:
            continue
        pos = saved['position']; target = targets[pos]
        require(saved['target_label_only'] == target, 'task label authority')
        masses = {r['token_id']: r for r in n['pool']['token_masses']}
        winner = n['pool']['summary']['chosen_token_id']
        require(sum(v['weight_q31'] for v in masses.values()) == n['pool']['summary']['total_weight_q31'], 'complete pool mass sum')
        require(masses[winner]['weight_q31'] == max(v['weight_q31'] for v in masses.values()), 'actual largest saved mass')
        keys = n['bank_trace']['prefix']['angular_indices']
        target_aliases = [i for i,t in enumerate(n['copy_ids']) if t == target]
        rival_aliases = [i for i,t in enumerate(n['copy_ids']) if t == winner]
        prefix_differences = [{'target_ordinal': i, 'rival_ordinal': j,
                               'distinct_lanes': sum(row[i] != row[j] for row in keys)}
                              for i in target_aliases for j in rival_aliases]
        delta = n['continuation']['delta_scores_q24']
        phases.append({'position':pos, 'target':target, 'winner':winner, 'correct':target == winner,
                       'target_mass': masses[target], 'winner_mass':masses[winner],
                       'native_total_mass':n['pool']['summary']['total_weight_q31'],
                       'donor_ordinal':n['bridge']['selected_ordinal'], 'post_state':n['post_state'],
                       'generate_vector_sha256':digest(n['generate_q24']),
                       'response_state':n['bank_trace']['prefix']['response'],
                       'target_copy_ordinals':target_aliases, 'rival_copy_ordinals':rival_aliases,
                       'prefix_target_rival_distinct_lanes':prefix_differences,
                       'Generate_plus_U_target_minus_rival_q24':n['generate_q24'][target]-n['generate_q24'][winner],
                       'base_Generate_target_minus_rival_q24':n['generate_q24'][target]-n['generate_q24'][winner]-(delta[target]-delta[winner]),
                       'token_winner_changed_by_clip':n['pool']['summary']['token_winner_changed_by_clip'],
                       'U_target_minus_rival_q24':delta[target]-delta[winner],
                       'clipped_low':n['pool']['summary']['clipped_low_actions'],
                       'clipped_high':n['pool']['summary']['clipped_high_actions']})
    phases.sort(key=lambda x:x['position'])
    require([r['position'] for r in phases] == list(range(15)), 'exact complete episode')
    residuals = [p['position'] for p in phases if not p['correct']]
    require(residuals == [7,8,9,10,13,14], 'declared residual set')
    for p in phases:
        p['same_post_rows'] = post_groups[tuple(p['post_state'])]
    journal = read(args.run/'coupled-construction.json')
    stats = {p:{'winning_alternatives':0, 'winning_feasible':0,
                'winning_objective_rejected':0, 'winning_guard_vetoed':0,
                'winning_CE_not_strict':0, 'winning_reference_fail':0, 'first_veto_counts':{},
                'committed_gains':[], 'committed_losses':[], 'feasible_but_not_selected':[],
                'first_winning_alternative':None} for p in residuals}
    counters = collections.Counter()
    current = journal['summary']['initial']
    best_count = sum(current['objective_masses'][p][2] == targets[p] for p in range(15))
    best_examples = []
    for rec in journal['coordinate_records']:
        family=rec['family']; counters[family+':'+rec['rank_status']] += 1
        selected=rec['selected']; chosen=None
        for alt in rec['alternatives']:
            counters[family+':alternative:'+alt['guard_status']] += 1
            objective=alt['objective']
            correct=[p for p in range(15) if objective['objective_masses'][p][2] == targets[p]]
            event={'order':rec['order'],'family':family,'index':rec['index'],'code':alt['code'],
                   'incumbent_epoch':rec['incumbent_epoch'],'combined_ce':objective['combined'],
                   'current_ce':current['combined'],'correct_positions':correct,
                   'feasible':alt['feasible'],'guard_status':alt['guard_status'],
                   'first_guard_failure':alt['first_failure']}
            is_selected=selected['status']=='committed' and selected['code']==alt['code']
            if is_selected: chosen=alt
            if alt['feasible'] and len(correct)>best_count:
                best_count=len(correct);best_examples=[event]
            elif alt['feasible'] and len(correct)==best_count and len(best_examples)<5:
                best_examples.append(event)
            for p in residuals:
                if p not in correct: continue
                st=stats[p];st['winning_alternatives']+=1
                if st['first_winning_alternative'] is None:st['first_winning_alternative']=event
                if alt['feasible']:
                    st['winning_feasible']+=1
                    if not is_selected and len(st['feasible_but_not_selected'])<5:st['feasible_but_not_selected'].append(event)
                elif alt['guard_status']=='FIRST_VETO':
                    st['winning_guard_vetoed']+=1
                    key=str(alt['first_failure']['guard_index'])
                    st['first_veto_counts'][key]=st['first_veto_counts'].get(key,0)+1
                else:
                    st['winning_objective_rejected']+=1
                    st['winning_CE_not_strict']+=int(current['combined']-objective['combined'] <= 1e-10*(1+abs(current['combined'])))
                    st['winning_reference_fail']+=int(objective['correct_reference_frames'] != 17)
        if chosen:
            for p in residuals:
                before=current['objective_masses'][p][2]==targets[p]
                after=chosen['objective']['objective_masses'][p][2]==targets[p]
                if before != after:
                    stats[p]['committed_gains' if after else 'committed_losses'].append({
                        'order':rec['order'],'family':family,'index':rec['index'],'code':chosen['code'],
                        'before_winner':current['objective_masses'][p][2],
                        'after_winner':chosen['objective']['objective_masses'][p][2],
                        'before_ce':current['combined'],'after_ce':chosen['objective']['combined']})
            current=chosen['objective']
    require(journal['summary']['final']==report['candidate_objective'], 'two complete endpoint reports agree')
    require(all(current[k] == report['candidate_objective'][k] for k in current), 'every selected endpoint numeric field matches')
    require([{'chosen':p['winner'], 'position':p['position'], 'target':p['target'],
              'target_mass':p['target_mass']['weight_q31'],
              'total_mass':p['native_total_mass']} for p in phases]
            == report['candidate_objective']['phases'], 'native per-phase endpoint fields match')
    require(sum(v for k,v in counters.items() if ':alternative:' in k)==14292, 'all saved alternatives counted')
    result={'schema':'uor-r4.saved-residual-diagnosis/1','status':'PASS',
            'measured_model_source_commit':report['source_commit'],
            'analyzer_sha256':sha(pathlib.Path(__file__)), 'input_report_sha256':RUN_REPORT,
            'input_manifest_sha256':RUN_MANIFEST,'rust_file_verification':'PASS',
            'snapshot_inputs':inputs, 'journal_sha256':sha(args.run/'coupled-construction.json'),
            'phases':phases,'residual_history':stats,'counts':dict(counters),
            'maximum_correct_in_recorded_feasible_alternative':best_count,
            'maximum_examples':best_examples,
            'scope':'Saved-data attribution only. Reuses prior full numerical audit; no new model, gradient, proposal or encoder. Equal current post states constrain Generate at that donor only; different states or Prefix keys do not prove global feasibility. Rejected-objective guards may be unexamined. Per-position feasible alternatives cannot be combined across epochs.',
            'elapsed_seconds':time.monotonic()-started,'peak_rss_kib':resource.getrusage(resource.RUSAGE_SELF).ru_maxrss}
    (args.output/'report.json').write_text(json.dumps(result,indent=2)+'\n')
    subprocess.run([str(SEAL), 'seal', str(args.output)], check=True)
    subprocess.run([str(args.verifier), 'verify-report', str(args.output)], check=True)
    print(json.dumps({'status':'PASS', 'output_rust_seal_and_verify':'PASS','seconds':result['elapsed_seconds'], 'max_correct':best_count,
                      'residuals':{p:{k:v for k,v in s.items() if not isinstance(v,(list,dict))} for p,s in stats.items()}}))

if __name__=='__main__':main()
