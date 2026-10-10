#!/usr/bin/env python3
"""Layer the fixed numerical admission onto the pinned observation copy only."""
import hashlib,json,pathlib,shutil,difflib
P=pathlib.Path(__file__).resolve().parent
R=P.parents[2]
D=R/'local/microlp-0.6.0'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def replace(s,a,b):
    if s.count(a)!=1:raise ValueError('exclusive patch anchor differs: '+a[:80])
    return s.replace(a,b)
def main():
    inv=json.loads((R/'local/observer-install.json').read_text())
    for f,h in inv['changed'].items():
        if sha(D/f)!=h:raise ValueError('observed backend differs: '+f)
    backup=R/'local/microlp-observed-before-repair'
    shutil.copytree(D,backup)
    names=['src/lu.rs','src/sparse.rs','src/lib.rs','src/diagnostics.rs']
    before={f:(D/f).read_text() for f in names}
    s=before['src/lu.rs']
    s=replace(s,'    let mat_nnz =','''    for c in 0..size {
        let (rows, vals) = get_col(c);
        if rows.len()!=vals.len() || rows.iter().any(|&r|r>=size) || vals.iter().any(|v|!v.is_finite()) {
            return Err(Error::NonFiniteFactorization);
        }
    }
    let mat_nnz =''')
    s=replace(s,'                    scratch.rhs.values[orig_r] -= x_val * coeff;','''                    let product = x_val * coeff;
                    let updated = scratch.rhs.values[orig_r] - product;
                    if !product.is_finite() || !updated.is_finite() { return Err(Error::NonFiniteFactorization); }
                    scratch.rhs.values[orig_r] = updated;''')
    s=replace(s,'    for i_col in 0..size {','    let mut min_chosen_pivot_over_tau=f64::INFINITY;\n    let mut max_column_growth: f64=0.0;\n    for i_col in 0..size {')
    s=replace(s,'        let pivot_orig_r = {','''        let original_scale = mat_col.1.iter().copied().map(f64::abs).fold(0.0,f64::max);
        let mut residual_scale: f64 = 0.0;
        for &r in &scratch.rhs.nonzero {
            let v=scratch.rhs.values[r];
            if !v.is_finite() { return Err(Error::NonFiniteFactorization); }
            residual_scale=residual_scale.max(v.abs());
        }
        let tau=crate::repair::pivot_threshold(size,original_scale,residual_scale)?;
        let pivot_orig_r = {''')
    s=replace(s,'            if max_abs < EPS {','''            if !max_abs.is_finite() || max_abs==0.0 || max_abs <= tau {
                crate::diagnostics::repair_event(format!("pivot_rejection column={} original_scale_bits={:016x} residual_scale_bits={:016x} tau_bits={:016x} pmax_bits={:016x}",i_col,original_scale.to_bits(),residual_scale.to_bits(),tau.to_bits(),max_abs.to_bits()));''')
    s=replace(s,'            assert!(max_abs.is_normal());','')
    s=replace(s,'                    i_col,col_perm.new2orig[i_col],max_abs.to_bits(),EPS.to_bits(),stability_coeff.to_bits(),','                    i_col,col_perm.new2orig[i_col],max_abs.to_bits(),tau.to_bits(),stability_coeff.to_bits(),')
    s=s.replace('max_abs_bits={:016x} EPS_bits={:016x} stability_bits=', 'max_abs_bits={:016x} tau_bits={:016x} stability_bits=')
    s=replace(s,'        let pivot_val = scratch.rhs.values[pivot_orig_r];','''        let pivot_val = scratch.rhs.values[pivot_orig_r];
        let growth=if original_scale>0.0 {residual_scale/original_scale} else {0.0};
        if !growth.is_finite() {return Err(Error::NonFiniteFactorization);}
        max_column_growth=max_column_growth.max(growth);
        if tau>0.0 {min_chosen_pivot_over_tau=min_chosen_pivot_over_tau.min(pivot_val.abs()/tau);}
        if !pivot_val.is_finite() || pivot_val==0.0 || pivot_val.abs()<=tau {
            crate::diagnostics::singular("selected_pivot_threshold",size,&get_col,format!("column={i_col} chosen_pivot_bits={:016x} tau_bits={:016x} original_scale_bits={:016x} residual_scale_bits={:016x}",pivot_val.to_bits(),tau.to_bits(),original_scale.to_bits(),residual_scale.to_bits()));
            return Err(Error::SingularMatrix);
        }''')
    s=replace(s,'                Ordering::Greater => lower.push(orig_r, val / pivot_val),','''                Ordering::Greater => {
                    let coefficient=val/pivot_val;
                    if !coefficient.is_finite() {return Err(Error::NonFiniteFactorization);}
                    lower.push(orig_r,coefficient)
                },''')
    s=replace(s,'    Ok(res)','    crate::diagnostics::repair_event(format!("factor_pivots min_chosen_pivot_over_tau={:?} max_column_growth={:?}",min_chosen_pivot_over_tau,max_column_growth));\n    if let Err(error)=crate::repair::verify(size,&get_col,&res) {\n        crate::diagnostics::singular("fresh_factor_residual_verification",size,&get_col,format!("verification_error={error}"));\n        return Err(error);\n    }\n    Ok(res)')
    s=replace(s,'    rhs[col] = x_val;','    crate::repair::observed(x_val);\n    rhs[col] = x_val;')
    s=replace(s,'        rhs[r] -= x_val * coeff;','''        let product=crate::repair::observed(x_val*coeff);
        rhs[r]=crate::repair::observed(rhs[r]-product);''')
    (D/'src/lu.rs').write_text(s)
    s=before['src/sparse.rs'];s=replace(s,'    SingularMatrix,','    SingularMatrix,\n    NonFiniteFactorization,\n    UnreliableFactorization,')
    s=replace(s,'            Error::SingularMatrix => "Singular matrix",','''            Error::SingularMatrix => "Singular matrix",
            Error::NonFiniteFactorization => "Nonfinite factorization arithmetic",
            Error::UnreliableFactorization => "Original-basis residual verification failed",''');(D/'src/sparse.rs').write_text(s)
    (D/'src/lib.rs').write_text(before['src/lib.rs']+'\n/// Offline numerical admission utility; not a serving dependency.\n#[allow(missing_docs)]\npub mod repair;\n')
    (D/'src/diagnostics.rs').write_text(before['src/diagnostics.rs']+'''
pub(crate) fn repair_event(detail: String) {
    change(|s| {
        let Some(root)=&s.root else {return;};
        let result=(|| -> io::Result<()> {
            let path=root.join("repair-events.log");
            let bytes=std::fs::metadata(&path).map(|m|m.len()).unwrap_or(0);
            if bytes>32*1024*1024 {return Err(io::Error::other("repair aggregate log cap exceeded"));}
            let mut f=OpenOptions::new().create(true).append(true).open(path)?;
            writeln!(f,"factor={} phase={} lp_phase={} iteration={} {}",s.sequence,s.phase,s.lp_phase,s.iteration,detail)
        })();
        if let Err(e)=result {s.errors.push(e.to_string());}
    });
}
''')
    shutil.copyfile(P/'repair.rs',D/'src/repair.rs')
    (D/'Cargo.toml').write_text((D/'Cargo.toml').read_text()+'\n[workspace]\n')
    patch=''.join(line for f,old in before.items() for line in difflib.unified_diff(old.splitlines(True),(D/f).read_text().splitlines(True),fromfile='observed/'+f,tofile='repaired/'+f))
    (R/'local/repair.patch').write_text(patch)
    files={f:sha(D/f) for f in [*names,'src/repair.rs','Cargo.toml','Cargo.lock'] if (D/f).exists()}
    (R/'local/repair-install.json').write_text(json.dumps({'observer':inv,'changed':files,'repair_source_sha256':sha(P/'repair.rs'),'apply_sha256':sha(P/'apply-repair.py'),'patch_sha256':sha(R/'local/repair.patch'),'policy':'scale-aware-pivot-and-all-unit-original-basis-residual/1'},indent=2)+'\n')
if __name__=='__main__':main()
