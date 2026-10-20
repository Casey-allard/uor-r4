"""Step 0a tabulation: counts annotator labels already fixed in TSV files. No model work."""
import hashlib, json, math, collections

R = '/Users/casey.allard/uor-r4-local/step0/a-taxonomy'
G = '/Users/casey.allard/uor-r4-local/ladder/grades'
LABELS = ['R', 'K', 'N', 'I', 'T', 'O']
NAMES = {'R': 'recall-anaphora-consistency', 'K': 'knowledge-absent', 'N': 'incoherent',
         'I': 'instruction', 'T': 'truncation', 'O': 'other'}
THRESHOLD = 0.15


def sha(path):
    return hashlib.sha256(open(path, 'rb').read()).hexdigest()


def wilson(k, n, z=1.959964):
    p = k / n
    den = 1 + z * z / n
    c = (p + z * z / (2 * n)) / den
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / den
    return [round(c - h, 4), round(c + h, 4)]


def load(path):
    out = {}
    for line in open(path).read().strip().split('\n'):
        f = line.split('\t')
        out[f[0]] = {'label': f[1], 'ctx': int(f[2]), 'r_any': int(f[3]), 'ill': int(f[4]), 'note': f[5]}
    return out


def summarize(model_dir, labels_path):
    rep_path = f'{G}/{model_dir}/report.json'
    rep = json.load(open(rep_path))
    rows = {r['id']: r for r in rep['rows']}
    failing = [i for i, r in rows.items() if not (r['grades']['fluent'] and r['grades']['relevant'])]
    lab = load(labels_path)
    assert set(lab) == set(failing), set(lab) ^ set(failing)
    for i, l in lab.items():
        assert l['label'] in LABELS, (i, l)
    n = len(failing)
    cnt = collections.Counter(l['label'] for l in lab.values())
    by_cat = collections.defaultdict(collections.Counter)
    for i, l in lab.items():
        by_cat[rows[i]['category']][l['label']] += 1
    well = [l for l in lab.values() if not l['ill']]
    cnt_well = collections.Counter(l['label'] for l in well)
    r_any = sum(l['r_any'] for l in lab.values())
    return {
        'model_dir': model_dir,
        'report_sha256': sha(rep_path),
        'model_sha256': rep['model_sha256'],
        'parameters': rep['parameters'],
        'grader': rep['grader']['model'],
        'panel_rows': len(rows),
        'acceptable': rep['actual']['acceptable'],
        'failing_rows': n,
        'labels_file_sha256': sha(labels_path),
        'counts': {NAMES[k]: cnt.get(k, 0) for k in LABELS},
        'shares': {NAMES[k]: round(cnt.get(k, 0) / n, 4) for k in LABELS},
        'recall_share_wilson95': wilson(cnt.get('R', 0), n),
        'recall_upper_bound_r_any': {'rows': r_any, 'share': round(r_any / n, 4)},
        'context_dependent_failing_rows': sum(l['ctx'] for l in lab.values()),
        'ill_posed_failing_rows': n - len(well),
        'counts_excluding_ill_posed': {NAMES[k]: cnt_well.get(k, 0) for k in LABELS},
        'recall_share_excluding_ill_posed': round(cnt_well.get('R', 0) / len(well), 4),
        'counts_by_panel_category': {c: {NAMES[k]: v.get(k, 0) for k in LABELS} for c, v in sorted(by_cat.items())},
        'recall_rows': sorted(i for i, l in lab.items() if l['label'] == 'R'),
        'r_any_rows': sorted(i for i, l in lab.items() if l['r_any']),
    }


def kappa(a, b):
    ids = sorted(a)
    n = len(ids)
    po = sum(a[i] == b[i] for i in ids) / n
    ca, cb = collections.Counter(a.values()), collections.Counter(b.values())
    pe = sum(ca[k] * cb[k] for k in LABELS) / (n * n)
    return po, pe, (po - pe) / (1 - pe)


m96 = summarize('chat-100m-C-powered-7b', f'{R}/labels-96m-pass1.tsv')
m29 = summarize('chat-29m-B-lr5e-4-powered-7b', f'{R}/labels-29m.tsv')

p1 = {k: v['label'] for k, v in load(f'{R}/labels-96m-pass1.tsv').items()}
p2 = dict(l.split('\t') for l in open(f'{R}/labels-96m-pass2-blind.tsv').read().strip().split('\n'))
subset = open(f'{R}/blind-subset-ids.txt').read().split()
assert sorted(subset) == sorted(p2)
first = {i: p1[i] for i in subset}
po, pe, k = kappa(first, p2)
disagreements = sorted([i, first[i], p2[i]] for i in subset if first[i] != p2[i])

r_share = m96['shares']['recall-anaphora-consistency']
outcome = ('retrieval cannot move the open panel; panel lever = corpus/knowledge (Step 5)'
           if r_share < THRESHOLD else 'retrieval can move the panel')
result = {
    'schema': 'uor-r4.step0a-panel-taxonomy/1',
    'instrument': 'Step 0a failure taxonomy, open 232-request panel, annotator = Claude (Opus 5.5), training-free',
    'rubric_sha256': sha(f'{R}/rubric.md'),
    'precedence': 'N > R > I > K > T > O',
    'models': {'96m_C': m96, '29m_B_lr5e-4': m29},
    'self_agreement': {
        'subset_rows': len(subset), 'subset_seed': 20261005, 'subset_ids_sha256': sha(f'{R}/blind-subset-ids.txt'),
        'pass2_sha256': sha(f'{R}/labels-96m-pass2-blind.tsv'),
        'observed_agreement': round(po, 4), 'chance_agreement': round(pe, 4), 'cohen_kappa': round(k, 4),
        'disagreements': disagreements,
        'caveat': 'same annotator, same session; ids visible; a self-consistency check, not inter-rater reliability',
    },
    'decision_rule': {'threshold_recall_share': THRESHOLD, 'observed_96m_recall_share': r_share,
                      'observed_96m_recall_upper_bound': m96['recall_upper_bound_r_any']['share'],
                      'outcome': outcome},
    'tabulator_sha256': sha(f'{R}/tabulate.py'),
}
json.dump(result, open(f'{R}/taxonomy.json', 'w'), indent=1, sort_keys=True)
print(json.dumps({k: result[k] for k in ['decision_rule', 'self_agreement']}, indent=1))
for name, m in result['models'].items():
    print(name, m['failing_rows'], m['counts'], m['recall_share_wilson95'], m['recall_upper_bound_r_any'],
          'ill', m['ill_posed_failing_rows'], 'ctx', m['context_dependent_failing_rows'])
    print('  excl ill', m['counts_excluding_ill_posed'], m['recall_share_excluding_ill_posed'])
    for c, v in m['counts_by_panel_category'].items():
        print('  ', c, v)
