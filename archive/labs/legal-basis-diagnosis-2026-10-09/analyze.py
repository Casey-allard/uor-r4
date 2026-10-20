#!/usr/bin/env python3
"""Decode exact diagnostic bit receipts; no solver, training or model evaluation."""
from pathlib import Path
import ast,hashlib,json,re,struct,sys
import blake3
w=Path(__file__).resolve().parents[3]
attempt=int(sys.argv[1]) if len(sys.argv)==2 else 1
assert attempt in [1,2]
suffix='' if attempt==1 else '-0002'
root=w/f'local/replay-{attempt:04}';out=w/f'local/analysis{suffix}';out.mkdir(exist_ok=False)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
def f64(bits):return struct.unpack('<d',struct.pack('<Q',bits))[0]
manifest=read(root/'manifest.json');paths=set()
for f in manifest['files']:
 p=root/f['path'];assert p.stat().st_size==f['bytes'] and blake3.blake3(p.read_bytes()).hexdigest()==f['blake3'];paths.add(f['path'])
assert {str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}==paths|{'manifest.json'}
model=read(root/'model-mapping.json');guards=read(w/'local/input/guard-map.json');events=[]
for p in sorted((root/'backend-observations').glob('*/context.txt')):
 text=p.read_text();d=dict(line.split('=',1) for line in text.splitlines() if '=' in line)
 arrays={k:ast.literal_eval(d[k]) for k in ['basic_vars','col_new2orig','row_new2orig','row_orig2new','residual_all_original_row_bits','residual_stored_nonzero_original_rows','eligible_original_rows','prior_upper_diagonal_bits','row_scale_bits']}
 pivot={k:int(v,16) if k.endswith('_bits') else int(v) for k,v in re.findall(r'(pivot_column|original_column|max_abs_bits|EPS_bits|stability_bits)=([0-9a-f]+)',text)}
 n=len(arrays['basic_vars']);assert n==len(model['constraints'])==440 and len(arrays['row_scale_bits'])==n
 assert arrays['col_new2orig'][pivot['pivot_column']]==pivot['original_column']
 assert len(arrays['prior_upper_diagonal_bits'])==pivot['pivot_column']
 variable=arrays['basic_vars'][pivot['original_column']]
 mapping=model['variables'][variable] if variable<1960 else {'backend_variable':variable,'component':'slack','model_constraint':variable-1960}
 eligible=set(arrays['eligible_original_rows']);stored=set(arrays['residual_stored_nonzero_original_rows']);candidates=[]
 for r in sorted(eligible):
  bits=arrays['residual_all_original_row_bits'][r];value=f64(bits)
  if r in stored:
   candidates.append({'internal_row':r,'stored':True,'value':value,'bits':f'{bits:016x}','constraint':model['constraints'][r]})
 max_abs=max([abs(x['value']) for x in candidates],default=0.)
 assert max_abs==f64(pivot['max_abs_bits']) and 0<max_abs<f64(pivot['EPS_bits'])
 cols=[];entries=0
 for line in (p.parent/'basis-csc.tsv').read_text().splitlines():
  if line.startswith('COLUMN\t'):cols.append(int(line.split('\t')[2]))
  if line.startswith('ENTRY\t'):entries+=1
 assert len(cols)==n and sum(cols)==entries
 events.append({'event':p.parent.name,'kind':d['kind'],'factor_sequence':int(d['factor_sequence']),'phase':d['phase'],'lp_phase':d['lp_phase'],'lp_iterations':int(d['lp_iterations']),'factor_callsite':d['factor_callsite'],'eta_updates_before_reset':int(d['eta_updates_before_reset']),'basis_dimension':n,'basis_stored_entries':entries,'basis_empty_stored_columns':sum(x==0 for x in cols),'pivot_column':pivot['pivot_column'],'original_basis_column':pivot['original_column'],'backend_variable':mapping,'max_abs_pivot':max_abs,'max_abs_pivot_bits':f"{pivot['max_abs_bits']:016x}",'EPS':f64(pivot['EPS_bits']),'EPS_bits':f"{pivot['EPS_bits']:016x}",'eligible_rows':sorted(eligible),'stored_eligible_residuals':candidates,'computed_nonzero_candidates':sum(x['value']!=0 for x in candidates),'context_sha256':sha(p),'basis_sha256':sha(p.parent/'basis-csc.tsv')})
process=read(w/f'local/replay{suffix}-process.json');report=read(root/'report.json')
summary={'schema':'uor-r4.legal-basis-result/1','decision':'KEEP diagnosis; constructor NOT YET PROMOTED; no numerical repair in this unit','source_commit':process['source_commit'],'binary_sha256':process['binary_sha256'],'input_sha256':process['input_sha256'],'report_sha256':sha(root/'report.json'),'manifest_sha256':sha(root/'manifest.json'),'sealed_files_verified':len(paths),'factorizations_observed':report['factorizations_observed'],'singular_events':report['singular_events'],'backend_termination':report['backend_termination'],'events':events,'elapsed_seconds':process['elapsed_seconds'],'peak_child_rss_bytes':process['peak_child_rss_bytes'],'new_backward_calls':0,'native_score_calls':0,'returned_assignments':0,'model_quality':'UNAVAILABLE','exact_rank':'NOT_MEASURED','global_feasibility':'NOT_ESTABLISHED','accepted_development_complete_replies':'8/512 unchanged','separate_conditional_artifact':'9/15 unchanged'}
(out/'result.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({k:v for k,v in summary.items() if k!='events'},indent=2));print(json.dumps(events,indent=2))
