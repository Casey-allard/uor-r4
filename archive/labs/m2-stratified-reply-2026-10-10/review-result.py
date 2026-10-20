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
assert report['updates']==96 and report['training_row_draws']==768 and report['target_position_draws']==9280
result={'status':'PASS_SAVED_ROWS','source_commit':report['source_commit'],'report_sha256':hashlib.sha256((root/'report.json').read_bytes()).hexdigest(),'endpoints':summary,'gained':gained,'lost':lost,'retained':retained,'changed_outputs':sum(outputs['initial'][k]!=outputs['final'][k] for k in labels),'keep':bool(gained) and not lost,'checked_rows':1024,'scope':'Saved row hashes, row-ID set, actual-prefix feedback and chosen-token chain, EOS and exact frozen membership reproduced. Native scores/backwards/tokenizer decoding not independently regenerated. Complete Rust report seal verification is producer evidence, separate from this reader.'}
# Check repeated supervised distribution and the separate subset decision.
from collections import Counter
indices=[0,1,4,5,8,9,12,13,130,131,138,139,256,257,264,265,386,387,394,395,448,449,456,457]
order=json.loads((root/'order.json').read_text())['order'];assert len(order)==768
assert Counter(order)==Counter({i:32 for i in indices})
for start in range(0,768,24):assert order[start:start+24]==order[:24]
q=report['qualification']
for phase in ['initial','final']:
 selected=q[phase+'_rows'];assert [r['index'] for r in selected]==indices
 counts={'complete':0,'teacher_all_tokens_correct':0,'entry_correct':0,'eos':0}
 for rec in selected:
  ref=report[phase+'_evaluation']['rows'][rec['index']];assert rec['id']==ref['id']
  row=json.loads((root/ref['row_file']).read_text());target=row['canonical_target_ids_labels_only']
  teacher=sum(s['native']['pool']['summary']['chosen_token_id']==tok for s,tok in zip(row['canonical'],target));assert len(row['canonical'])==len(target)
  actual={'complete':row['complete'],'teacher_all_tokens_correct':teacher==len(target),'entry_correct':bool(row['generated_ids']) and row['generated_ids'][0]==target[0],'eos':row['eos']}
  for k,v in actual.items():assert rec[k]==v;counts[k]+=v
  assert rec['teacher_correct_tokens']==teacher and rec['teacher_total_tokens']==len(target)
 for k,v in counts.items():assert q[phase][k]==v
assert q['qualified_fit']==(q['final']['complete']==24)
assert q['model_keep']==result['keep']
result['qualification']=q
result['schedule_check']='PASS: fixed24 repeated32epochs,768draws,9280targetpositions'
# Prior rejected candidate is an independently authenticated retained report.
prior_root=Path('/workspace/uor-r4/codex/m2-reply-20261009/runs/reply-0064-attempt1')
raw=(prior_root/'report.json').read_bytes();assert hashlib.sha256(raw).hexdigest()=='678f4ff4d8b97559260076607f6168b7d5f4317cc1472bd9fe5e253ca76bdbb7'
prior=json.loads(raw)['final_evaluation']['rows'];assert {r['id'] for r in prior}==set(labels)
pc={r['id'] for r in prior if r['complete']}
result['versus_rejected_2141']={'complete_before':len(pc),'complete_after':len(sets['final']),'gained':sorted(sets['final']-pc),'lost':sorted(pc-sets['final']),'changed_outputs':sum(r['generated_ids']!=outputs['final'][r['id']] for r in prior)}
print(json.dumps(result,indent=2))
