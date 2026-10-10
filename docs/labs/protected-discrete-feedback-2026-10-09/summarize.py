#!/usr/bin/env python3
"""Extract bounded result fields from sealed Rust reports; does not score a model."""
from pathlib import Path
import sys
import hashlib,json
W=Path(__file__).resolve().parents[3];R=W/'local/discrete-feedback'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
c=read(Path(sys.argv[1]) if len(sys.argv)>1 else R/'config.json');root=Path(c['out']);r=read(root/'report.json');assert r['status']=='COMPLETED'
j=read(root/'coupled-construction.json');rounds=j['joint_vectors'];obs=R/'observations'/root.name
result=dict(source_commit=r['source_commit'],config_sha256=sha(root/'config.json'),report_sha256=sha(root/'report.json'),manifest_sha256=sha(root/'manifest.json'),binary_sha256=read(obs/'runtime.json')['binary_sha256'],construction=j['summary'],final_gate=r['final_gate'],candidate_receipt=r['candidate_receipt'],new_backward_calls=r['new_backward_calls'],new_training_graph_forwards=r['new_training_graph_forwards'],candidate_native_steps=r['candidate_native_steps'],feedback_status=j['projection']['feedback_status'],rounds=len(rounds),distinct_destinations=len({tuple(x['destination_master_bits']) for x in rounds}),eligible_rounds=sum(x['eligible'] for x in rounds),distinct_eligible=sum(x['eligible'] and x['duplicate_of'] is None for x in rounds),round_summary=[dict(round=x['round'],violated_guards=sum(v < -x['quantized_tolerance'] for v in x['quantized_margin_residuals']),minimum_residual=min(x['quantized_margin_residuals']),tolerance=x['quantized_tolerance'],gradient_dot=x['actual_CE_linear_delta'],feedback_norm=x['feedback_norm'],primal_norm=x['primal_norm'],duplicate_of=x['duplicate_of'],eligible=x['eligible'],native=x['native']['guard_status']) for x in rounds],execution={k:v for k,v in read(obs/'execution.json').items() if k!='rust_files'},qualification=read(R/f'qualification-decision-{root.name}.json'),scope='Fixed original-parent development245 conditional15 includingEOS,17reference roles,380guards; accepted8/512 unchanged unless separately qualified')
with (R/'result.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps({k:result[k] for k in ['rounds','distinct_destinations','eligible_rounds','distinct_eligible','final_gate','new_backward_calls']}))
