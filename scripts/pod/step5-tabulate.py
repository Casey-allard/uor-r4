#!/usr/bin/env python3
"""Tabulate the Step 5 knowledge A/B (#820) and apply its frozen decision rule.

Tabulation only (reads sealed JSON reports; no model runs):

  step5-tabulate.py RUN_ROOT [--clean DIR] [--json OUT.json]

RUN_ROOT is the `R` of scripts/pod/step5-knowledge.sh. Arms: A (current mix) and
K (current mix with 25% knowledge corpus), seeds 1 and 2. It reads
  base-{A,K}-s{1,2}/report.json                 base train report (final ts-valid NLL)
  eval-knowledge-{base,ft}-{A,K}-s{1,2}/evaluation.json   knowledge held-out NLL
  session-{sieve,off}-{A,K}-s{1,2}/report.json  D19 grounded sessions
  grade-{A,K}-s{1,2}/report.json                open 232-request panel grades
and the clean subsets in docs/research/step5/panel-clean (clean-ids.txt and
wellposed-ids.txt).

Decision rule (frozen in docs/research/step5/knowledge-ab.md before any run),
applied in this order; the first outcome reached is the decision:
  0. Completeness (matched tokens). Every input the rule reads is present: the
     four base train reports with completed_steps == 70609 and stopped_early
     false, the four Arm C fine-tune reports with completed_steps == 4000, base
     knowledge held-out NLL for all four, the four panel grades and the four
     sieve-on session reports. Otherwise INCOMPLETE (a run capped by
     max_seconds is not the matched-token arm the rule was frozen for).
  1. Manipulation check: K's knowledge held-out NLL (base) is at least 0.10 nats
     below A's on both seeds; otherwise INSTRUMENT (the corpus did not land;
     the panel is not read).
  Panel delta: d_s = acceptable(K, s) - acceptable(A, s) on all 232 rows;
    mean_d = (d_1 + d_2) / 2; pooled exact McNemar over the 464 (request, seed)
    pairs (decisive); per-seed exact McNemar over 232 pairs (reported only).
  Panel criterion: d_1 > 0 and d_2 > 0, mean_d >= 10, p < 0.05, and the clean
    guard (d_s on the 143-row clean subset > 0 on both seeds).
  Instruction guard: pooled D19 Instruction passes (sieve on, 2 x 98 turns) of K
    >= pooled A - max(3, |A_s1 - A_s2|).
  2. PROMOTE  panel criterion and instruction guard hold.
  3. BLOCKED  panel criterion holds, instruction guard fails.
  4. NULL     mean_d < 5, or d_1 and d_2 have opposite signs (d_1 * d_2 < 0).
  5. WEAK     anything else.
  The panel criterion requires both seed deltas positive and mean_d >= 10, so
  PROMOTE/BLOCKED and NULL cannot both hold; the order is stated anyway.
"""
import argparse
import json
import math
import os
import sys

ARMS = ('A', 'K')
SEEDS = (1, 2)
BASE_STEPS = 70609      # x batch 16 x context 384 = 433,821,696 tokens per base
FT_STEPS = 4000
HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_CLEAN = os.path.join(HERE, '..', '..', 'docs', 'research', 'step5', 'panel-clean')


def load(path):
    if not os.path.exists(path):
        return None
    with open(path, encoding='utf-8') as handle:
        return json.load(handle)


def ids(path):
    with open(path, encoding='utf-8') as handle:
        return [line.strip() for line in handle if line.strip()]


def mcnemar_exact(b, c):
    """Two-sided exact McNemar p (the chat-grade definition)."""
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    tail = sum(math.comb(n, i) for i in range(k + 1)) / 2 ** n
    return min(1.0, 2 * tail)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('root')
    parser.add_argument('--clean', default=DEFAULT_CLEAN)
    parser.add_argument('--json')
    args = parser.parse_args()
    R = args.root
    clean = ids(os.path.join(args.clean, 'clean-ids.txt'))
    wellposed = ids(os.path.join(args.clean, 'wellposed-ids.txt'))
    out = {'root': os.path.abspath(R), 'arms': {}}

    ok = {}
    for arm in ARMS:
        for seed in SEEDS:
            key = f'{arm}-s{seed}'
            row = {}
            base = load(os.path.join(R, f'base-{key}', 'report.json'))
            row['base_ts_valid_nll'] = (base or {}).get('final', {}).get('nll')
            row['base_completed_steps'] = (base or {}).get('completed_steps')
            row['base_stopped_early'] = (base or {}).get('stopped_early')
            ft = load(os.path.join(R, f'ft-C-{key}', 'report.json'))
            row['ft_completed_steps'] = (ft or {}).get('completed_steps')
            row['ft_stopped_early'] = (ft or {}).get('stopped_early')
            for stage in ('base', 'ft'):
                ev = load(os.path.join(R, f'eval-knowledge-{stage}-{key}', 'evaluation.json'))
                row[f'knowledge_nll_{stage}'] = ev['full']['nll'] if ev else None
            for recall in ('sieve', 'off'):
                session = load(os.path.join(R, f'session-{recall}-{key}', 'report.json'))
                if session:
                    cats = session['arms']['default']['by_category']
                    row[f'session_{recall}'] = {c: v['pass'] for c, v in cats.items()}
                    row[f'session_{recall}_total'] = sum(v['pass'] for v in cats.values())
                    row[f'session_{recall}_of'] = sum(v['of'] for v in cats.values())
            grade = load(os.path.join(R, f'grade-{key}', 'report.json'))
            if grade:
                ok[key] = {r['id']: bool(r['grades']['fluent'] and r['grades']['relevant'])
                           for r in grade['rows']}
                row['panel_all'] = sum(ok[key].values())
                row['panel_wellposed'] = sum(ok[key][i] for i in wellposed)
                row['panel_clean'] = sum(ok[key][i] for i in clean)
            out['arms'][key] = row

    print(f"{'arm':6} {'tsNLL':>7} {'kNLLb':>7} {'kNLLft':>7} {'panel':>6} {'well':>5} {'clean':>5} "
          f"{'D19 sieve':>10} {'D19 off':>8} {'Instr':>5}")
    for key, row in out['arms'].items():
        def f(v, fmt='{:.4f}'):
            return '-' if v is None else fmt.format(v)
        print(f"{key:6} {f(row.get('base_ts_valid_nll')):>7} {f(row.get('knowledge_nll_base')):>7} "
              f"{f(row.get('knowledge_nll_ft')):>7} {f(row.get('panel_all'), '{}'):>6} "
              f"{f(row.get('panel_wellposed'), '{}'):>5} {f(row.get('panel_clean'), '{}'):>5} "
              f"{f(row.get('session_sieve_total'), '{}'):>10} {f(row.get('session_off_total'), '{}'):>8} "
              f"{f((row.get('session_sieve') or {}).get('Instruction'), '{}'):>5}")

    need = [f'{a}-s{s}' for a in ARMS for s in SEEDS]
    a = out['arms']
    missing = []
    for key in need:
        row = a[key]
        if row['base_completed_steps'] != BASE_STEPS or row['base_stopped_early'] is not False:
            missing.append(f"base-{key}: completed_steps {row['base_completed_steps']}, "
                           f"stopped_early {row['base_stopped_early']} (need {BASE_STEPS}, false)")
        if row['ft_completed_steps'] != FT_STEPS:
            missing.append(f"ft-C-{key}: completed_steps {row['ft_completed_steps']} (need {FT_STEPS})")
        if row.get('knowledge_nll_base') is None:
            missing.append(f'eval-knowledge-base-{key}: missing')
        if key not in ok:
            missing.append(f'grade-{key}: missing')
        if (row.get('session_sieve') or {}).get('Instruction') is None:
            missing.append(f'session-sieve-{key}: missing (or no Instruction category)')
    out['completeness'] = {'base_steps': BASE_STEPS, 'ft_steps': FT_STEPS, 'problems': missing}
    if missing:
        for problem in missing:
            print('incomplete:', problem)
        decision = 'INCOMPLETE'
    else:
        manip = all(a[f'A-s{s}']['knowledge_nll_base'] - a[f'K-s{s}']['knowledge_nll_base'] >= 0.10
                    for s in SEEDS)
        rows = sorted(ok['A-s1'])
        d = {s: a[f'K-s{s}']['panel_all'] - a[f'A-s{s}']['panel_all'] for s in SEEDS}
        dc = {s: a[f'K-s{s}']['panel_clean'] - a[f'A-s{s}']['panel_clean'] for s in SEEDS}
        per_seed = {}
        for s in SEEDS:
            bs = sum(ok[f'K-s{s}'][i] and not ok[f'A-s{s}'][i] for i in rows)
            cs = sum(ok[f'A-s{s}'][i] and not ok[f'K-s{s}'][i] for i in rows)
            per_seed[s] = {'K_only': bs, 'A_only': cs, 'p': mcnemar_exact(bs, cs)}
        b = sum(v['K_only'] for v in per_seed.values())
        c = sum(v['A_only'] for v in per_seed.values())
        p = mcnemar_exact(b, c)
        mean_d = (d[1] + d[2]) / 2
        instr = {arm: [a[f'{arm}-s{s}']['session_sieve']['Instruction'] for s in SEEDS]
                 for arm in ARMS}
        tol = max(3, abs(instr['A'][0] - instr['A'][1]))
        instr_ok = sum(instr['K']) >= sum(instr['A']) - tol
        clean_ok = dc[1] > 0 and dc[2] > 0
        both_positive = d[1] > 0 and d[2] > 0
        panel_ok = both_positive and mean_d >= 10 and p < 0.05 and clean_ok
        # Frozen order: the first outcome reached is the decision.
        if not manip:
            decision = 'INSTRUMENT: knowledge corpus did not land (manipulation check failed); panel not read'
        elif panel_ok and instr_ok:
            decision = 'PROMOTE'
        elif panel_ok:
            decision = 'BLOCKED (instruction guard failed)'
        elif mean_d < 5 or d[1] * d[2] < 0:
            decision = 'NULL'
        else:
            decision = 'WEAK'
        out['comparison'] = {
            'manipulation_check': manip, 'delta_all': d, 'delta_clean': dc, 'mean_delta_all': mean_d,
            'both_seed_deltas_positive': both_positive, 'panel_criterion': panel_ok,
            'mcnemar_pooled': {'K_only': b, 'A_only': c, 'p': p,
                               'note': 'decisive; treats the two seeds as independent pairs '
                                       '(anti-conservative), so the per-seed p is reported too'},
            'mcnemar_per_seed': per_seed,
            'instruction_sieve': instr, 'instruction_tolerance': tol,
            'instruction_guard': instr_ok, 'clean_guard': clean_ok,
        }
        print(f"manipulation check (knowledge NLL A-K >= 0.10 on both seeds): {manip}")
        print(f"panel delta K-A: s1 {d[1]:+d}, s2 {d[2]:+d}, mean {mean_d:+.1f}; clean s1 {dc[1]:+d}, s2 {dc[2]:+d}")
        print(f"pooled McNemar over {2 * len(rows)} pairs (decisive): K-only {b}, A-only {c}, p = {p:.4g}")
        for s in SEEDS:
            v = per_seed[s]
            print(f"  seed {s} McNemar over {len(rows)} pairs (reported): K-only {v['K_only']}, "
                  f"A-only {v['A_only']}, p = {v['p']:.4g}")
        print(f"D19 Instruction (sieve) A {instr['A']} K {instr['K']} (tolerance {tol}): guard {instr_ok}")
        print(f"panel criterion (d_1 > 0, d_2 > 0, mean_d >= 10, p < 0.05, clean guard): {panel_ok}")
    out['decision'] = decision
    print(f'DECISION: {decision}')
    if args.json:
        with open(args.json, 'x', encoding='utf-8') as handle:
            json.dump(out, handle, indent=1)
    return 0


if __name__ == '__main__':
    sys.exit(main())
