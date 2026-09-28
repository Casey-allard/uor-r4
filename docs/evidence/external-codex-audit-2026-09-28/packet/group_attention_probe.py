#!/usr/bin/env python3
"""Exact finite-group attention identity, plus explicit information-loss checks.
Standalone mathematical experiment, not a UOR runtime or a trained model.
"""
from __future__ import annotations
import hashlib
import itertools
import json
import pathlib
import platform
import sys
import time
from datetime import datetime, timezone
import numpy as np

OUT = pathlib.Path(__file__).resolve().parent

def run() -> dict:
    started = time.perf_counter()
    p = 5
    elements = [t for t in itertools.product(range(p), repeat=4)
                if (t[0]*t[3] - t[1]*t[2]) % p == 1]
    assert len(elements) == 120
    lookup = {a:i for i,a in enumerate(elements)}
    e = lookup[(1,0,0,1)]
    n = len(elements)
    table = np.empty((n,n), dtype=np.int16)
    inverse = np.empty(n, dtype=np.int16)
    for i,a in enumerate(elements):
        inverse[i] = lookup[(a[3],-a[1] % p,-a[2] % p,a[0])]
        for j,b in enumerate(elements):
            table[i,j] = lookup[((a[0]*b[0]+a[1]*b[2]) % p,
                                 (a[0]*b[1]+a[1]*b[3]) % p,
                                 (a[2]*b[0]+a[3]*b[2]) % p,
                                 (a[2]*b[1]+a[3]*b[3]) % p)]
    ids = np.arange(n)
    assert np.array_equal(table[e], ids) and np.array_equal(table[:,e], ids)
    assert np.all(table[ids,inverse] == e) and np.all(table[inverse,ids] == e)
    assoc = bool(np.array_equal(table[table[:,:,None],ids[None,None,:]],
                               table[ids[:,None,None],table[None,:,:]]))
    assert assoc
    # Regular permutation representation rho(d)v(x) = v(d^-1 x).
    perm = table[inverse[:,None],ids[None,:]]
    rep = bool(np.array_equal(perm[table],perm[ids[None,:,None],perm[:,None,:]]))
    assert rep
    relative = table[inverse[:,None],ids[None,:]]
    frame = bool(np.all(table[inverse[table][:,:,None],table[:,None,:]] == relative[None,:,:]))
    assert frame

    def bucket(keys, values):
        sums = np.zeros((n,n), dtype=np.int64)
        count = np.zeros(n, dtype=np.int64)
        np.add.at(sums,keys,values)
        np.add.at(count,keys,1)
        return sums,count

    rows = []
    no_read = 0
    checks = 0
    for seed in (20260928,20260929):
        rng = np.random.default_rng(seed)
        support = rng.choice(n, size=8, replace=False)
        # Nonnegative weights 1,2,4, all within a signed four-bit value range.
        shifts = rng.integers(0,3,size=8)
        weights = np.zeros(n,dtype=np.int64)
        weights[support] = np.left_shift(1,shifts)
        for events in (0,1,32,256,1024):
            keys = rng.integers(0,n,size=events)
            values = rng.integers(-7,8,size=(events,n),dtype=np.int64)
            sums,count = bucket(keys,values)
            for a in ids:
                rel = table[inverse[a],keys]
                transported = values[np.arange(events)[:,None],perm[rel]]
                direct_n = np.sum(weights[rel,None]*transported,axis=0)
                direct_z = int(np.sum(weights[rel]))
                grouped_n = np.zeros(n,dtype=np.int64)
                grouped_z = 0
                for d,shift in zip(support,shifts):
                    address = table[a,d]
                    grouped_n += np.left_shift(sums[address,perm[d]],int(shift))
                    grouped_z += int(count[address]) << int(shift)
                assert np.array_equal(direct_n,grouped_n)
                assert direct_z == grouped_z
                no_read += direct_z == 0
                checks += 1
            rows.append({'seed':seed,'events':events,'queries':n,'support':8,
                         'numerator_and_denominator_exact':True})
    rng = np.random.default_rng(42)
    keys = rng.integers(0,n,size=256)
    values = rng.integers(-7,8,size=(256,n),dtype=np.int64)
    sums = np.zeros((n,n),dtype=np.int64); counts = np.zeros(n,dtype=np.int64)
    for t in range(256):
        if t >= 32:
            sums[keys[t-32]] -= values[t-32]; counts[keys[t-32]] -= 1
        sums[keys[t]] += values[t]; counts[keys[t]] += 1
        fresh_s,fresh_c = bucket(keys[max(0,t-31):t+1],values[max(0,t-31):t+1])
        assert np.array_equal(sums,fresh_s) and np.array_equal(counts,fresh_c)
    # Minimal ownership/binding collision in a single aggregate bucket.
    blue = np.zeros(n,dtype=np.int64); blue[0] = 1
    green = np.zeros(n,dtype=np.int64); green[1] = 1
    s1,c1 = bucket(np.array([e,e]),np.array([blue,green]))
    s2,c2 = bucket(np.array([e,e]),np.array([green,blue]))
    assert np.array_equal(s1,s2) and np.array_equal(c1,c2)
    noncommuting = int(np.count_nonzero(table != table.T))
    assert noncommuting > 0
    qi,qj,minus_e = lookup[(0,1,4,0)],lookup[(2,0,0,3)],lookup[(4,0,0,4)]
    assert table[qi,qi] == table[qj,qj] == minus_e
    assert table[qi,qj] != table[qj,qi]
    assert table[minus_e,table[qi,qj]] == table[qj,qi]
    # A corrupted transition cannot pass the group-law check.
    bad = table.copy(); bad[e,e] = (e+1) % n
    negative = not np.array_equal(bad[e],ids)
    assert negative
    # Positive scaling preserves direction but can reverse dot-score ranking.
    q=np.array([1,0]); ka=np.array([4,3]); kb=np.array([6,8])
    assert q@ka < q@kb
    assert (q@ka)/np.linalg.norm(ka) > (q@kb)/np.linalg.norm(kb)
    return {
        'schema':'uor-r4.geometric-attention-algebra-probe.v2',
        'observed_utc':datetime.now(timezone.utc).isoformat(),
        'source_context':'Roadmap inspected at 15d2ce3fac9f66ddcc945a8097e5c6216352c453',
        'group':'SL(2,F_5), independently enumerated; no repository group-ID binding',
        'arithmetic':'exact finite-field group; bounded signed int64 synthetic values and accumulators',
        'checks':{'order':n,'identity_inverse':True,'associativity_triples':n**3,'associativity_pass':assoc,
                  'representation_composition_scalar_equalities':n**3,'representation_pass':rep,
                  'common_left_frame_triples':n**3,'common_left_frame_pass':frame,
                  'ordered_noncommuting_pairs':noncommuting,'Q8_signed_subgroup':True,
                  'corrupted_identity_negative_control_detected':negative,
                  'direct_vs_grouped_attention_queries':checks,'zero_mass_queries':int(no_read),
                  'rolling_window_insert_evictions':256},
        'attention_cases':rows,
        'binding_obstruction':{'history_A':{'Alice':'blue','Bob':'green'},
                               'history_B':{'Alice':'green','Bob':'blue'},
                               'same_bucket_sum_count':True,'different_Alice_answer':True},
        'normalization_counterexample':{'q':[1,0],'key_A':[4,3],'key_B':[6,8],
                                       'dots':[4,6],'unit_dots':[0.8,0.6]},
        'illustrative_one_head_storage_bytes':{'int32_summary_120_by_120':120*120*4,
                                               'int32_counts':120*4,'unpadded_u8_group_table':120*120},
        'limits':['Not a trained model; no corpus, checkpoint, learned encoder or generated language tested.',
                  'Sparse equality concerns the defined compact-support operator, not an existing softmax language model.',
                  'Exact output numerator/denominator; normalization and overflow-safe native implementation remain separate.',
                  'Regular permutation values have dimension 120; not a four-coordinate quaternion implementation.',
                  'Frame test is a common left action on discrete codes, not independent local gauges or quantizer equivariance.',
                  'No throughput, energy, whole-process opcode, novelty or semantic-advantage claim.',
                  'Counts and vector sums lose owner-specific bindings unless those records/keys are retained separately.'],
        'runtime_seconds':time.perf_counter()-started,
        'software':{'python':sys.version.split()[0],'numpy':np.__version__,'platform':platform.platform()},
        'script_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
    }

if __name__ == '__main__':
    result=run()
    (OUT/'group_attention_probe_results.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'checks':result['checks'],'runtime_seconds':result['runtime_seconds'],
                      'script_sha256':result['script_sha256']},indent=2))
