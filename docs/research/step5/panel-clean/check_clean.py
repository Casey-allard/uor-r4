#!/usr/bin/env python3
"""Check the panel-clean labels and tabulate chat-grade reports on the clean subsets.

Tabulation only: no model runs. Usage:

  check_clean.py [--panel-dir DIR] [--write-ids] [REPORT.json ...]

* Verifies panel-clean-232.tsv covers exactly the 232 open-panel ids (everyday-32 +
  heldout-200-a + heldout-200-b, in panel order) when the panel files are present.
* Verifies every row Step 0a flagged ill-posed (ill=1 in either 0a label file) is
  status `ill` here (this file is a superset of the 0a flags).
* --write-ids rewrites clean-ids.txt (status clean) and wellposed-ids.txt
  (status clean or underspecified).
* For each chat-grade report.json, prints acceptable counts (fluent and relevant)
  on all 232 rows, on the well-posed subset and on the clean subset.
"""
import argparse
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
STEP0A = os.path.join(HERE, '..', '..', 'step0', '0a-panel-taxonomy')


def load_labels():
    rows = []
    with open(os.path.join(HERE, 'panel-clean-232.tsv'), encoding='utf-8') as handle:
        header = handle.readline().rstrip('\n').split('\t')
        assert header == ['id', 'status', 'reason', 'note'], header
        for line in handle:
            rid, status, reason, note = line.rstrip('\n').split('\t')
            assert status in ('clean', 'ill', 'underspecified'), (rid, status)
            assert (status == 'clean') == (reason == '-'), (rid, status, reason)
            rows.append((rid, status, reason))
    return rows


def step0a_ill():
    flagged = set()
    for name in ('labels-96m-pass1.tsv', 'labels-29m.tsv'):
        with open(os.path.join(STEP0A, name), encoding='utf-8') as handle:
            for line in handle:
                parts = line.rstrip('\n').split('\t')
                if len(parts) >= 5 and parts[4] == '1':
                    flagged.add(parts[0])
    return flagged


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--panel-dir', default=os.path.expanduser('~/uor-r4-local/ladder/panel'))
    parser.add_argument('--write-ids', action='store_true')
    parser.add_argument('reports', nargs='*')
    args = parser.parse_args()

    rows = load_labels()
    ids = [r[0] for r in rows]
    assert len(ids) == 232 and len(set(ids)) == 232, len(ids)
    files = ['everyday-32.json', 'heldout-200-a.json', 'heldout-200-b.json']
    if all(os.path.exists(os.path.join(args.panel_dir, f)) for f in files):
        panel = []
        for f in files:
            with open(os.path.join(args.panel_dir, f), encoding='utf-8') as handle:
                panel += [row['id'] for row in json.load(handle)]
        assert panel == ids, 'label rows differ from the panel ids or their order'
        print('panel ids: match (232, in order)')
    else:
        print('panel files not found; id order not checked')

    status = {rid: st for rid, st, _ in rows}
    missing = sorted(i for i in step0a_ill() if status.get(i) != 'ill')
    assert not missing, f'0a ill rows not marked ill here: {missing}'
    print(f'0a ill-flagged rows: {len(step0a_ill())}, all marked ill here')

    counts = {}
    for _, st, reason in rows:
        counts.setdefault(st, {}).setdefault(reason, 0)
        counts[st][reason] += 1
    for st in ('clean', 'ill', 'underspecified'):
        total = sum(counts.get(st, {}).values())
        detail = ', '.join(f'{k} {v}' for k, v in sorted(counts.get(st, {}).items()) if k != '-')
        print(f'{st}: {total}' + (f' ({detail})' if detail else ''))

    clean = [rid for rid, st, _ in rows if st == 'clean']
    wellposed = [rid for rid, st, _ in rows if st != 'ill']
    if args.write_ids:
        for name, subset in (('clean-ids.txt', clean), ('wellposed-ids.txt', wellposed)):
            with open(os.path.join(HERE, name), 'w', encoding='utf-8') as handle:
                handle.write('\n'.join(subset) + '\n')

    for path in args.reports:
        with open(path, encoding='utf-8') as handle:
            report = json.load(handle)
        ok = {row['id']: bool(row['grades']['fluent'] and row['grades']['relevant'])
              for row in report['rows']}
        assert set(ok) == set(ids), f'{path}: rows differ from the 232 panel ids'
        line = [f'{sum(ok.values())}/232 all']
        for name, subset in (('well-posed', wellposed), ('clean', clean)):
            line.append(f'{sum(ok[i] for i in subset)}/{len(subset)} {name}')
        print(f'{path}: ' + ', '.join(line))
    return 0


if __name__ == '__main__':
    sys.exit(main())
