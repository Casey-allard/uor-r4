#!/usr/bin/env python3
"""Already weighted saved gradient attribution; no native scoring or proposals."""
import argparse, hashlib, json, math, pathlib, resource, struct, sys, time
from analyze import need, sha, read, static_potential

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--run',type=pathlib.Path,required=True)
    p.add_argument('--prior',type=pathlib.Path,required=True)
    p.add_argument('--output',type=pathlib.Path,required=True)
    a=p.parse_args()
    need(a.output.is_dir() and (a.output/'attempt.json').is_file() and not (a.output/'report.json').exists(),'fresh Rust-claimed root required')
    need(read(a.output/'attempt.json')['schema']=='uor-r4.report-attempt/1','normative claim')
    start=time.monotonic(); prior=read(a.prior)
    need(sha(a.prior)=='9d8886ef565e8bf49b6be6202c8460d6d487dc9adfd23d127399c7a31280f9e4' and prior['status']=='PASS','prior frozen attribution')
    pins={}
    def auth(leaf):
        path=a.run/leaf
        key='runs/pair-0001-attempt1/'+leaf
        pin=prior['authenticated_inputs'][key]
        need(path.stat().st_size==pin['bytes'] and sha(path)==pin['sha256'],'input pin '+leaf)
        pins[leaf]=pin
        return path
    def floats(leaf):
        data=auth(leaf).read_bytes();need(len(data)==57600*4,'full pair shape')
        x=struct.unpack('<57600f',data);need(all(map(math.isfinite,x)),'finite gradients');return x
    masters=floats('generate-initial-masters.f32le');aggregate=floats('generate-gradient.f32le')
    receipt=read(auth('generate-gradient-receipt.json')); order=read(auth('generate-coordinate-order.json'))
    terms=[]
    for i,t in enumerate(receipt['perterm']):
        need(t['physical_index']==i and t['status']=='PRESENT' and not t['missing_gradient_filled_zero'],'physical weighted term authority')
        terms.append(floats(t['file']))
    need(len(terms)==31,'31 physical weighted terms')
    target=receipt['perterm'][4];need((target['input_index'],target['position'],target['target'],target['weight'])==(245,4,267,1/15),'weighted target role')
    selected=set(order);need(len(selected)==960,'selected frozen subset')
    metrics={name:{'coordinates':0,'available_descent_coordinates':0,'potential':0.0,'helps':{'count':0,'potential':0.0},'opposes':{'count':0,'potential':0.0},'zero':{'count':0,'potential':0.0}} for name in ('selected960','remaining56640')}
    max_residual=0.;l1_residual=0.;mismatch=0;max_order_error=0.;exact_other_opposes=0.;rounded_other_opposes=0.
    for i,m in enumerate(masters):
        values=[t[i] for t in terms]
        exact=math.fsum(values);ordered=0.
        for v in values: ordered=struct.unpack('<f',struct.pack('<f',ordered+v))[0]
        error=aggregate[i]-ordered;max_order_error=max(max_order_error,abs(error));mismatch+=struct.pack('<f',aggregate[i])!=struct.pack('<f',ordered)
        residual=aggregate[i]-exact;max_residual=max(max_residual,abs(residual));l1_residual+=abs(residual)
        group=metrics['selected960' if i in selected else 'remaining56640'];group['coordinates']+=1
        h,delta,_=static_potential(terms[4][i],m);b=max(0.,-h)
        if b>0:
            group['available_descent_coordinates']+=1;group['potential']+=b
            utility=aggregate[i]*delta
            sign='helps' if utility<0 else 'opposes' if utility>0 else 'zero'
            group[sign]['count']+=1;group[sign]['potential']+=b
            exact_other=math.fsum(values[:4]+values[5:])*delta
            rounded_other=(aggregate[i]-terms[4][i])*delta
            exact_other_opposes+=max(0.,exact_other);rounded_other_opposes+=max(0.,rounded_other)
    need(mismatch==0,'ordered f32 aggregate bit parity')
    total=sum(g['potential'] for g in metrics.values());opposed=sum(g['opposes']['potential'] for g in metrics.values())
    rss=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss;rss=rss if sys.platform=='darwin' else rss*1024
    need(rss<=1<<30,'one GiB RSS cap')
    out={'schema':'uor-r4.pair-direction-attribution/1','status':'PASS','prior_report_sha256':sha(a.prior),'script_sha256':sha(pathlib.Path(__file__)),'source_commit':prior['source_commit'],'producer_source_sha256':prior['producer_source_sha256'],'authenticated_inputs':pins,'scope':'Saved already-weighted 31 physical gradients only; static legal-displacement attribution, no native scores or proposals','groups':metrics,'total_available_descent_potential':total,'aggregate_opposed_available_descent_potential':opposed,'aggregate_opposed_share':opposed/total,'selected_descent_share':metrics['selected960']['potential']/total,'ordered_f32_sum':{'bit_mismatches':mismatch,'maximum_error':max_order_error},'rounded_aggregate_minus_exact_widened_sum':{'maximum_absolute_residual':max_residual,'l1_absolute_residual':l1_residual},'other_terms_at_target_best_destination':{'exact_widened_other_term_positive_utility':exact_other_opposes,'rounded_aggregate_minus_target_positive_utility':rounded_other_opposes,'interpretation':'Positive other-term utility is not aggregate opposition: it may oppose target credit without reversing the aggregate descent direction.'},'new_native_scores':0,'new_gradients':0,'new_proposals':0,'workers':1,'elapsed_seconds':time.monotonic()-start,'peak_RSS_bytes':rss,'limits':'Initial local linearized potential is not hard winner feasibility; remaining56640 not native evaluated; no automatic wider sweep.'}
    with (a.output/'report.json').open('x') as f:json.dump(out,f,indent=2,sort_keys=True);f.write('\n')
    print(json.dumps({'status':'PASS','report_sha256':sha(a.output/'report.json'),'groups':metrics,'opposed_share':out['aggregate_opposed_share'],'ordered_sum':out['ordered_f32_sum'],'seconds':out['elapsed_seconds'],'RSS':rss}))
if __name__=='__main__':main()
