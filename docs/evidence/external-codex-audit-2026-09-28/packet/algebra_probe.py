#!/usr/bin/env python3
"""Exact, bounded research probe. Not UOR model code or a language benchmark.

Independent SL(2,F5) construction, sparse/direct finite-group read equality,
rolling insertion/eviction, and an information-loss counterexample.
"""
from __future__ import annotations
import hashlib,itertools,json,pathlib,platform,sys,time
from datetime import datetime, timezone
import numpy as np
OUT=pathlib.Path(__file__).resolve().parent

def main():
    t0=time.perf_counter()
    E=[t for t in itertools.product(range(5),repeat=4) if (t[0]*t[3]-t[1]*t[2])%5==1]
    assert len(E)==120
    ix={t:i for i,t in enumerate(E)}; e=ix[(1,0,0,1)]; G=len(E); ids=np.arange(G)
    mul=np.empty((G,G),dtype=np.int16); inv=np.empty(G,dtype=np.int16)
    for i,a in enumerate(E):
        inv[i]=ix[(a[3],-a[1]%5,-a[2]%5,a[0])]
        for j,b in enumerate(E):
            mul[i,j]=ix[((a[0]*b[0]+a[1]*b[2])%5,(a[0]*b[1]+a[1]*b[3])%5,
                         (a[2]*b[0]+a[3]*b[2])%5,(a[2]*b[1]+a[3]*b[3])%5)]
    assert np.array_equal(mul[e],ids) and np.array_equal(mul[:,e],ids)
    assert np.all(mul[ids,inv]==e) and np.all(mul[inv,ids]==e)
    assoc=np.array_equal(mul[mul[:,:,None],ids[None,None,:]],mul[ids[:,None,None],mul[None,:,:]])
    assert assoc
    # rho(d)v(x)=v(d^-1 x). Every action is a coordinate permutation.
    perm=mul[inv[:,None],ids[None,:]]
    rep=np.array_equal(perm[mul],perm[ids[None,:,None],perm[:,None,:]])
    assert rep
    relative=mul[inv[:,None],ids[None,:]]
    left=np.all(mul[inv[mul][:,:,None],mul[:,None,:]]==relative[None,:,:])
    assert left
    def buckets(keys,values):
        sums=np.zeros((G,G),dtype=np.int64);counts=np.zeros(G,dtype=np.int64)
        np.add.at(sums,keys,values);np.add.at(counts,keys,1)
        return sums,counts
    records=[]; comparisons=0; zero=0
    for seed in [20260928,20260929]:
        rng=np.random.default_rng(seed);support=rng.choice(G,8,replace=False);shifts=rng.integers(0,4,8)
        weights=np.zeros(G,dtype=np.int64);weights[support]=np.left_shift(1,shifts)
        for n in [0,1,32,256,1024]:
            keys=rng.integers(0,G,n);values=rng.integers(-7,8,(n,G),dtype=np.int64)
            sums,counts=buckets(keys,values)
            for q in ids:
                r=mul[inv[q],keys]
                moved=values[np.arange(n)[:,None],perm[r]]
                direct=(weights[r,None]*moved).sum(axis=0);den=int(weights[r].sum())
                sparse=np.zeros(G,dtype=np.int64);sden=0
                for d,s in zip(support,shifts):
                    a=mul[q,d];sparse+=np.left_shift(sums[a,perm[d]],int(s));sden+=int(counts[a])<<int(s)
                assert np.array_equal(direct,sparse) and den==sden
                comparisons+=1;zero+=den==0
            records.append({'seed':seed,'events':n,'queries':G,'support':8,'numerators_exact':True,'denominators_exact':True})
    rng=np.random.default_rng(42);keys=rng.integers(0,G,256);values=rng.integers(-7,8,(256,G),dtype=np.int64)
    sums=np.zeros((G,G),dtype=np.int64);counts=np.zeros(G,dtype=np.int64)
    for i in range(256):
        if i>=32:
            sums[keys[i-32]]-=values[i-32];counts[keys[i-32]]-=1
        sums[keys[i]]+=values[i];counts[keys[i]]+=1
        a,b=buckets(keys[max(0,i-31):i+1],values[max(0,i-31):i+1])
        assert np.array_equal(a,sums) and np.array_equal(b,counts)
    blue=np.zeros(G,dtype=np.int64);blue[0]=1
    green=np.zeros(G,dtype=np.int64);green[1]=1
    a,ac=buckets(np.array([e,e]),np.array([blue,green]))
    b,bc=buckets(np.array([e,e]),np.array([green,blue]))
    assert np.array_equal(a,b) and np.array_equal(ac,bc)
    # A nontrivial overwrite map cannot be an invertible group action.
    overwrite={0:0,1:0};assert len(set(overwrite.values()))<len(overwrite)
    out={'schema':'uor.geometric-attention.algebra-probe.v1',
         'timestamp_utc':datetime.now(timezone.utc).isoformat(),
         'repo_context_pin':'15d2ce3fac9f66ddcc945a8097e5c6216352c453',
         'group':'SL(2,F5); independent IDs, no claimed mapping to repository 2I IDs',
         'checks':{'elements':G,'products':G**2,'associativity_all_triples':bool(assoc),
                   'associativity_triples':G**3,'permutation_representation_all_cases':bool(rep),
                   'representation_cases':G**3,'common_left_frame_all_triples':bool(left),
                   'left_frame_triples':G**3,'noncommuting_ordered_pairs':int(np.count_nonzero(mul!=mul.T)),
                   'sparse_direct_read_comparisons':comparisons,'no_read_zero_denominators':zero,
                   'insert_evict_checks':256},
         'runs':records,
         'counterexamples':{
             'binding':{'A':{'Alice':'blue','Bob':'green'},'B':{'Alice':'green','Bob':'blue'},
                        'bucket_summaries_identical':True,'correct_Alice_answers_differ':True},
             'norm_removal':{'query':[1,0,0,0],'A':[4,3,0,0],'B':[6,8,0,0],
                             'dot_scores':[4,6],'cosine_scores':[0.8,0.6],'rank_reversed':True},
             'overwrite_not_group_action':{'map':overwrite,'injective':False}},
         'illustrative_storage':{'G120_D120_int32_sums_bytes':57600,'int32_counts_bytes':480,
                                'unpadded_u8_group_table_bytes':14400,'S8_D120_coordinates_read':960,
                                'excludes':'encoders, identity/version records, index postings, type/gain bins, output head'},
         'limitations':['No learned model, natural text or repository checkpoint used.',
                        'Direct/sparse equality is for the defined finite kernel, not preservation of an existing LM.',
                        'Exact numerator and denominator only; normalization and native execution are not implemented.',
                        'Sparse equivalence requires identical weights within each eligible key bucket and linear transport.',
                        'Global common-left-frame invariance is not independent local gauge invariance or semantic equivariance.',
                        'No native instruction, throughput, energy or priority/novelty certification.',
                        'int64 bounds adequate for these finite cases only.'],
         'runtime_seconds':time.perf_counter()-t0,'python':sys.version,'numpy':np.__version__,'platform':platform.platform(),
         'script_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()}
    (OUT/'algebra_probe_results.json').write_text(json.dumps(out,indent=2)+'\n')
    print(json.dumps(out,indent=2))
if __name__=='__main__':main()
