"""Inspect saved native traces only; no model calls or new predictions."""
from pathlib import Path
from collections import defaultdict
from fractions import Fraction
import hashlib,json,sys
R=Path(sys.argv[1]);P=Path(sys.argv[2]);O=Path(sys.argv[3])
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def fraction(n,d):
 f=Fraction(n,d);return {'numerator':f.numerator,'denominator':f.denominator}
report=read(R/'report.json');prep=read(P/'report.json');inputs=read(P/'inputs.json')['cases'];labels=read(P/'labels.json')['cases']
assert report['status']==prep['status']=='COMPLETED' and report['cases']==4
assert prep['changed_token_index']==0 and prep['model_calls']==0
assert len(report['rows'])==len(prep['rows'])==len(inputs)==len(labels)==4
for kind in ('inputs','labels'):
 assert report[kind+'_sha256']==prep[kind+'_sha256']==sha(P/(kind+'.json'))
ids=[x['id'] for x in inputs];assert len(set(ids))==4
for i in range(4):
 assert ids[i]==labels[i]['id']==prep['rows'][i]['id']==report['rows'][i]['id']
rows=[read(R/x['row_file']) for x in report['rows']]
for i,(x,row) in enumerate(zip(report['rows'],rows)):
 assert row['id']==ids[i]
 assert row['entry_correct']==(row['generated_ids'][0]==prep['rows'][i]['target_first_token_id'])
 assert x['row_sha256']==sha(R/x['row_file'])
 assert 'continuation_sha256' not in row
 for step in row['steps']:
  assert 'continuation' not in step
  pool=step['actions'];sums=defaultdict(lambda:[0,0])
  for a in pool['actions']:
   sums[a['token_id']][int('Copy' in a['action'])]+=a['weight_q31']
  assert len(pool['token_masses'])==4096
  for t in pool['token_masses']:
   g,c=sums[t['token_id']]
   assert (g,c,g+c)==(t['generate_weight_q31'],t['copy_weight_q31'],t['weight_q31'])
  s=pool['summary'];assert s['total_weight_q31']==sum(sum(x) for x in sums.values())
  winner=min(sums,key=lambda t:(-sum(sums[t]),t))
  assert winner==s['chosen_token_id']
 assert row['generated_ids']==[s['actions']['summary']['chosen_token_id'] for s in row['steps']]
a,b=(rows[i]['steps'][0] for i in (2,3));ta,tb=(s['bank_trace'] for s in (a,b));ba,bb=(t['cue_bank']['bank'] for t in (ta,tb));ca,cb=(t['cue_bank']['carrier'] for t in (ta,tb))
assert inputs[2]['segments']==inputs[3]['segments'] and inputs[2]['actual_prefix_ids']==inputs[3]['actual_prefix_ids']==[]
assert inputs[2]['query_ids'][1:]==inputs[3]['query_ids'][1:] and inputs[2]['query_ids'][0]!=inputs[3]['query_ids'][0]
ra,rb=ba['context'],bb['context'];diff=[i for i,(x,y) in enumerate(zip(ra['tokens'],rb['tokens'])) if x!=y];assert len(ra['tokens'])==len(rb['tokens']) and len(diff)==1
start=len(ra['tokens'])-len(inputs[2]['query_ids']);assert diff==[start]
assert ra['tokens'][start:]==inputs[2]['query_ids'] and rb['tokens'][start:]==inputs[3]['query_ids']
lanes=ra['heads']*ra['lanes_per_head'];assert lanes==rb['heads']*rb['lanes_per_head']
def changed(x,y):
 assert len(x)==len(y)
 return [i for i,(a,b) in enumerate(zip(x,y)) if a!=b]
state_diff=[{'query_position':i-start,'different_state_lanes':changed(ra['states'][i],rb['states'][i]),'different_observed_lanes':changed(ra['codes'][i*lanes:(i+1)*lanes],rb['codes'][i*lanes:(i+1)*lanes])} for i in range(start,len(ra['tokens']))]
assert ra['states'][:start]==rb['states'][:start]
assert ba['candidates']==bb['candidates']
targets=[prep['rows'][i]['target_first_token_id'] for i in (2,3)];assert targets[0]!=targets[1]
entry=[]
for i,row in enumerate(rows):
 s=row['steps'][0];pool=s['actions'];m={t['token_id']:t for t in pool['token_masses']};total=pool['summary']['total_weight_q31'];target=prep['rows'][i]['target_first_token_id']
 entry.append({'id':row['id'],'query_text':prep['rows'][i]['query_text'],'expected_answers':labels[i]['answers'],'target_first_id':target,'chosen_id':row['generated_ids'][0],'entry_correct':row['entry_correct'],'complete':row['complete'],'eos':row['eos'],'decoded':row['decoded'],'generated_ids':row['generated_ids'],'target_masses':{str(t):dict(m[t],probability=fraction(m[t]['weight_q31'],total)) for t in targets},'total_mass':total,'source_candidates':len(s['copy_token_ids']),'selected_donor':s['bridge']['selected_candidate'] if s['bridge'] else None})
summary={'status':'PASS','scope':'saved trace integrity and paired numerical comparison; no independent forward or capability promotion','report_sha256':sha(R/'report.json'),'preparation_report_sha256':sha(P/'report.json'),'rows':entry,'pair':{'changed_absolute_context_position':start,'query_tokens':len(inputs[2]['query_ids']),'context_tokens':len(ra['tokens']),'state_width':lanes,'source_candidates':len(ba['candidates']),'query_state_trajectory':state_diff,'cue_state_changed_lanes':changed(ca['query']['states'],cb['query']['states']),'cue_observation_changed_lanes':changed(ca['query']['codes'],cb['query']['codes']),'cue_copy_changed_heads':[changed(x,y) for x,y in zip(ca['copy_q24'],cb['copy_q24'])],'bank_score_changed_heads':[changed(x['scores_q24'],y['scores_q24']) for x,y in zip(ba['heads'],bb['heads'])],'prefix_trace_identical':ta['prefix']==tb['prefix'],'donor_identical':a['bridge']['selected_candidate']==b['bridge']['selected_candidate'],'post_state_changed_lanes':changed(a['post_state_codes'],b['post_state_codes']),'generate_score_changed_tokens':changed(a['generate_raw_scores_q24'],b['generate_raw_scores_q24']),'copy_score_changed_positions':changed(a['copy_raw_scores_q24'],b['copy_raw_scores_q24']),'full_pool_identical':a['actions']==b['actions']},'all_saved_steps_alias_aggregation_and_winner':'PASS','continuation_U':'ABSENT'}
with O.open('x') as f:json.dump(summary,f,indent=2)
print(json.dumps({'status':summary['status'],'rows':entry,'pair':summary['pair']},indent=2))
