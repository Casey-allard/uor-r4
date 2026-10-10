"""Read saved row evidence; no model, optimizer, tokenizer or inference code."""
from pathlib import Path
import json,hashlib,sys
root=Path(sys.argv[1]); config=json.loads((root/'config.json').read_text())
report=json.loads((root/'report.json').read_text());assert report['status']=='COMPLETED'
labels_path=Path(config['development_labels']); label_bytes=labels_path.read_bytes()
assert hashlib.sha256(label_bytes).hexdigest()=='84991e0657b5697c0e061eaa3fe86e4a0ec7ce6bc2be8371b62698c6b8126155'
labels={x['id']:x['answers']['accepted'] for x in json.loads(label_bytes)['cases']};assert len(labels)==512
summary={};sets={};outputs={}
for phase in ['initial','final']:
 evaluation=report[phase+'_evaluation']; refs=evaluation['rows'];assert len(refs)==512
 seen=set();complete=[];eos=entry=0;out={}
 for ref in refs:
  rid=ref['id'];assert rid not in seen and rid in labels;seen.add(rid)
  raw=(root/ref['row_file']).read_bytes();assert hashlib.sha256(raw).hexdigest()==ref['row_sha256']
  row=json.loads(raw);assert row['id']==rid
  ids=row['generated_ids']; target=row['canonical_target_ids_labels_only'];trace=row['generation'];assert len(ids)==len(trace)
  for t,(token,step) in enumerate(zip(ids,trace)):
   assert step['actual_prefix_ids']==ids[:t]
   assert step['pool']['summary']['chosen_token_id']==token
  stopped=bool(ids) and ids[-1]==target[-1];assert stopped==row['eos']==ref['eos']
  good=stopped and row['decoded'] in labels[rid];assert good==row['complete']==ref['complete']
  if good:complete.append(rid)
  eos+=stopped;entry+=bool(ids) and ids[0]==target[0];out[rid]=ids
 assert seen==set(labels) and len(complete)==evaluation['complete']
 sets[phase]=set(complete);outputs[phase]=out
 summary[phase]={'complete':len(complete),'eos':eos,'entry_correct':entry,'native_equal_episode_ce':evaluation['native_equal_episode_ce'],'complete_ids':complete}
gained=sorted(sets['final']-sets['initial']);lost=sorted(sets['initial']-sets['final']);retained=sorted(sets['initial']&sets['final'])
for key,actual in [('gained_complete_ids',gained),('lost_complete_ids',lost),('retained_complete_ids',retained)]:assert sorted(report['outcomes'][key])==actual
assert report['updates']==64 and report['training_row_draws']==512
result={'status':'PASS_SAVED_ROWS','source_commit':report['source_commit'],'report_sha256':hashlib.sha256((root/'report.json').read_bytes()).hexdigest(),'endpoints':summary,'gained':gained,'lost':lost,'retained':retained,'changed_outputs':sum(outputs['initial'][k]!=outputs['final'][k] for k in labels),'keep':bool(gained) and not lost,'checked_rows':1024,'scope':'Saved row hashes, row-ID set, actual-prefix feedback and chosen-token chain, EOS and exact frozen membership reproduced. Native scores/backwards/tokenizer decoding not independently regenerated. Complete Rust report seal verification is producer evidence, separate from this reader.'}
print(json.dumps(result,indent=2))
