#!/usr/bin/env python3
"""Install a pinned local dependency copy; never modifies the Cargo registry."""
import argparse,difflib,hashlib,json,pathlib,shutil
P=pathlib.Path(__file__).resolve().parent
R=P.parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def replace(text,a,b,n=1):
    if text.count(a)!=n:raise ValueError(f'patch anchor count {text.count(a)} != {n}: {a[:80]}')
    return text.replace(a,b)
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--upstream',type=pathlib.Path,required=True);args=ap.parse_args()
    src=args.upstream.resolve(); inv=json.loads((P/'upstream-identity.json').read_text())
    for f,v in inv.items():
        if sha(src/f)!=v:raise ValueError('upstream identity differs: '+f)
    dst=R/'local/microlp-0.6.0'
    if dst.exists():raise ValueError('exclusive dependency destination exists')
    shutil.copytree(src,dst)
    before={f:(src/f).read_text() for f in ['src/lib.rs','src/lu.rs','src/solver.rs','src/mip/mod.rs']}
    s=before['src/lib.rs'];s+='\n/// Offline observation adapter; no solver arithmetic changes.\n#[allow(missing_docs)]\npub mod diagnostics;\n';(dst/'src/lib.rs').write_text(s)
    s=before['src/lu.rs'];s=replace(s,'    // Implementation of the Gilbert-Peierls algorithm:', '    crate::diagnostics::factor_begin();\n    // Implementation of the Gilbert-Peierls algorithm:')
    s=replace(s,'    let col_perm = super::ordering::order_simple(size, |c| get_col(c).0)?;', '''    let col_perm = match super::ordering::order_simple(size, |c| get_col(c).0) {
        Ok(p) => p,
        Err(e) => {
            crate::diagnostics::singular("symbolic_order_simple", size, &get_col,
                format!("empty_stored_columns={:?}; EPS_bits={:016x}; stability_bits={:016x}",
                    (0..size).filter(|&c|get_col(c).0.is_empty()).collect::<Vec<_>>(), EPS.to_bits(),stability_coeff.to_bits()));
            return Err(e);
        }
    };''')
    s=replace(s,'''            if max_abs < EPS {
                return Err(Error::SingularMatrix);
            }''','''            if max_abs < EPS {
                crate::diagnostics::singular("numeric_lu_pivot", size, &get_col,
                    format!("pivot_column={} original_column={} max_abs_bits={:016x} EPS_bits={:016x} stability_bits={:016x}\\ncol_new2orig={:?}\\nrow_new2orig={:?}\\nrow_orig2new={:?}\\nresidual_all_original_row_bits={:?}\\nresidual_stored_nonzero_original_rows={:?}\\neligible_original_rows={:?}\\nprior_upper_diagonal_bits={:?}",
                    i_col,col_perm.new2orig[i_col],max_abs.to_bits(),EPS.to_bits(),stability_coeff.to_bits(),
                    col_perm.new2orig,new2orig_row,orig2new_row,crate::diagnostics::bits(&scratch.rhs.values),scratch.rhs.nonzero,
                    (0..size).filter(|&r|orig2new_row[r]>=i_col).collect::<Vec<_>>(),crate::diagnostics::bits(&upper_diag)));
                return Err(Error::SingularMatrix);
            }''');(dst/'src/lu.rs').write_text(s)
    s=before['src/solver.rs']
    s=replace(s,'        let lu_factors = lu_factorize(','''        crate::diagnostics::context(format!("factor_callsite=initial_constructor\\nstructural_vars={}\\noriginal_rhs_bits={:?}\\nrow_scale_bits={:?}\\noriginal_lower_bits={:?}\\noriginal_upper_bits={:?}\\nnonbasic_vars={:?}",num_vars,crate::diagnostics::bits(&orig_rhs),crate::diagnostics::bits(&row_scales),crate::diagnostics::bits(&orig_var_mins),crate::diagnostics::bits(&orig_var_maxs),nb_vars));
        crate::diagnostics::mapping(&basic_vars);
        let lu_factors = lu_factorize(''')
    # Every reset records current complete variable and row mapping before the existing operation.
    import re
    pat=r'(self\.basis_solver\s*\.reset\(&self\.orig_constraints_csc, &self\.basic_vars\)\?;)'
    if len(re.findall(pat,s))!=4:raise ValueError('expected four existing reset callsites')
    s=re.sub(pat,r'self.diagnostic_context();\n                        \1',s)
    anchor='    pub(crate) fn initial_solve(&mut self) -> Result<StopReason, Error> {'
    helper='''    fn diagnostic_context(&self) {
        crate::diagnostics::context(format!("factor_callsite=basis_reset\\nstructural_vars={}\\noriginal_rhs_bits={:?}\\nrow_scale_bits={:?}\\ncurrent_lower_bits={:?}\\ncurrent_upper_bits={:?}\\nnonbasic_vars={:?}\\nbasic_value_bits={:?}\\neta_updates_before_reset={}",self.num_vars,crate::diagnostics::bits(&self.orig_rhs),crate::diagnostics::bits(&self.row_scales),crate::diagnostics::bits(&self.orig_var_mins),crate::diagnostics::bits(&self.orig_var_maxs),self.nb_vars,crate::diagnostics::bits(&self.basic_var_vals),self.basis_solver.eta_matrices.len()));
        crate::diagnostics::mapping(&self.basic_vars);
        crate::diagnostics::iteration(self.lp_iterations);
    }
'''
    s=replace(s,anchor,helper+anchor)
    s=replace(s,'    fn optimize(&mut self) -> Result<StopReason, Error> {','    fn optimize(&mut self) -> Result<StopReason, Error> {\n        crate::diagnostics::lp_phase("optimization");')
    s=replace(s,'    fn restore_feasibility(&mut self) -> Result<StopReason, Error> {','    fn restore_feasibility(&mut self) -> Result<StopReason, Error> {\n        crate::diagnostics::lp_phase("feasibility");')
    s=replace(s,'            self.lp_iterations += 1;','            self.lp_iterations += 1;\n            crate::diagnostics::iteration(self.lp_iterations);',2)
    (dst/'src/solver.rs').write_text(s)
    s=before['src/mip/mod.rs']
    s=replace(s,'    if state.solver.initial_solve()? == StopReason::Limit {','    crate::diagnostics::phase("root_initial_solve");\n    if state.solver.initial_solve()? == StopReason::Limit {')
    s=replace(s,'fn solve_node_lp(state: &mut MipState) -> Result<NodeLp, Error> {','fn solve_node_lp(state: &mut MipState) -> Result<NodeLp, Error> {\n    crate::diagnostics::phase(&format!("branch_lp_nodes_solved={}",state.stats.nodes_solved));')
    s=s.replace('    let slack = state.solver.slack_basis();','    crate::diagnostics::phase(&format!("integral_candidate_slack_retry_nodes_solved={}",state.stats.nodes_solved));\n    let slack = state.solver.slack_basis();',1)
    # Bind the internal-error fallback in its exact function, not the first slack call.
    begin=s.index('fn solve_node_lp(state:')
    end=s.index('/// Pop policy:',begin)
    body=s[begin:end]
    body=replace(body,'    let slack = state.solver.slack_basis();',
        '    crate::diagnostics::phase(&format!("branch_slack_retry_nodes_solved={}",state.stats.nodes_solved));\n    let slack = state.solver.slack_basis();')
    s=s[:begin]+body+s[end:]
    s=replace(s,'fn visit_node(state: &mut MipState, node: Node, domains: &[VarDomain]) -> Result<NodeVisit, Error> {','fn visit_node(state: &mut MipState, node: Node, domains: &[VarDomain]) -> Result<NodeVisit, Error> {\n    crate::diagnostics::phase(&format!("branch_basis_load_nodes_solved={} depth={} bound_changes={:?}",state.stats.nodes_solved,node.depth,node.bound_changes));')
    # Scope warm-start subsolver explicitly at its entry; no bound handling is altered.
    start=s.index('fn try_warm_start(');pos=s.index(' {',start)+2
    s=s[:pos]+'\n    crate::diagnostics::phase("advisory_warm_start");'+s[pos:]
    (dst/'src/mip/mod.rs').write_text(s)
    shutil.copyfile(P/'diagnostics.rs',dst/'src/diagnostics.rs')
    patches=[]
    for f,old in before.items():patches.extend(difflib.unified_diff(old.splitlines(keepends=True),(dst/f).read_text().splitlines(keepends=True),fromfile='upstream/'+f,tofile='observed/'+f))
    (P/'observation.patch').write_text(''.join(patches))
    changed={f:sha(dst/f) for f in [*before,'src/diagnostics.rs']}
    (R/'local/observer-install.json').write_text(json.dumps({'upstream':inv,'changed':changed,'observer_sha256':sha(P/'diagnostics.rs'),'patch_sha256':sha(P/'observation.patch')},indent=2)+'\n')
if __name__=='__main__':main()
