#!/usr/bin/env python3
"""Build a single-source-of-truth canonical manifest for proof artifacts."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def derive_obligation_statuses(requirement_table_path: str) -> Dict[str, str]:
    """Derive O1..O5 statuses from the strict requirement closure table."""
    default_statuses = {"O1": "open", "O2": "open", "O3": "open", "O4": "open", "O5": "open"}
    table = Path(requirement_table_path)
    if not table.exists():
        return default_statuses
    try:
        payload = json.loads(table.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return default_statuses

    summary = payload.get("summary_by_obligation", {})
    statuses = dict(default_statuses)
    for oid in statuses:
        data = summary.get(oid)
        if not isinstance(data, dict):
            continue
        total = int(data.get("total", 0))
        theorem_open = int(data.get("theorem_open", total))
        if total > 0 and theorem_open == 0:
            statuses[oid] = "theorem_closed"
    return statuses


def main() -> None:
    ap = argparse.ArgumentParser(description="Create canonical proof manifest.")
    ap.add_argument("--profile", default="rh_bridge_candidate_b")
    ap.add_argument("--lemma-a", default="research/output/lemma_a_channel_derivation_check_none_z64_n1m.json")
    ap.add_argument("--lemma-b", default="research/output/lemma_b_truncation_program_gauss100_ref512.json")
    ap.add_argument("--lemma-c-triangle", default="research/output/lemma_c_triangle_transfer_refresh_2026-02-17.json")
    ap.add_argument("--lemma-c-ineq", default="research/output/lemma_c_inequality_probe_refresh_2026-02-17.json")
    ap.add_argument("--lemma-d", default="research/output/lemma_d_base_uniformity_probe_refresh_2026-02-17.json")
    ap.add_argument("--lemma-e", default="research/output/lemma_e_endpoint_probe_refresh_2026-02-17.json")
    ap.add_argument("--a1", default="research/output/a1_smoothing_uplift_pack_refresh_2026-02-17.json")
    ap.add_argument("--a2", default="research/output/a2_infinite_tail_uplift_refresh_2026-02-17_sf3p5.json")
    ap.add_argument("--a3", default="research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3.json")
    ap.add_argument("--a3-stress", default="research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3_stress_2026-02-17.json")
    ap.add_argument("--a3-sign-sensitive-fallback", default="research/output/a3_sign_sensitive_constant_replacement_pack_2026-02-17_deterministic_k1.json")
    ap.add_argument("--a4", default="research/output/a4_uniform_assumption_check_refresh_2026-02-17.json")
    ap.add_argument("--theorem-pack", default="research/output/uplift_theorem_pack_refresh_2026-02-17.json")
    ap.add_argument("--proof-closure-tracker", default="research/output/proof_closure_tracker_2026-02-17.json")
    ap.add_argument("--proof-requirement-closure-table", default="research/output/proof_requirement_closure_table_2026-02-17.json")
    ap.add_argument("--formal-proof-manuscript", default="research/output/formal_proof_manuscript_2026-02-17.md")
    ap.add_argument("--formal-proof-dependency-map", default="research/output/formal_proof_dependency_map_2026-02-17.json")
    ap.add_argument("--formal-proof-packet-index", default="research/output/formal_proof_packet_index_2026-02-17.json")
    ap.add_argument("--proof-grade-manuscript", default="research/output/proof_grade_manuscript_2026-02-17.md")
    ap.add_argument("--proof-citation-audit-json", default="research/output/proof_citation_audit_2026-02-17.json")
    ap.add_argument("--proof-citation-audit-md", default="research/output/proof_citation_audit_2026-02-17.md")
    ap.add_argument("--proof-formalization-index", default="research/output/proof_formalization_index_2026-02-17.json")
    ap.add_argument("--proof-lean-scaffold", default="research/formal/lean/PrimeRiemannBridge.lean")
    ap.add_argument("--proof-lean-mathlib", default="research/formal/lean/PrimeRiemannBridgeMathlib.lean")
    ap.add_argument("--proof-lean-completion-kernel", default="research/formal/lean/PrimeRiemannBridgeCompletionKernel.lean")
    ap.add_argument(
        "--proof-lean-oscillatory-reduction",
        default="research/formal/lean/PrimeRiemannBridgeOscillatoryReduction.lean",
    )
    ap.add_argument(
        "--proof-lean-concrete-pack-instantiation",
        default="research/formal/lean/PrimeRiemannBridgeConcretePackInstantiation.lean",
    )
    ap.add_argument(
        "--proof-lean-w2b-final-slot",
        default="research/formal/lean/PrimeRiemannBridgeW2bFinalSlot.lean",
    )
    ap.add_argument(
        "--proof-lean-w2b-imported-instance",
        default="research/formal/lean/PrimeRiemannBridgeW2bImportedInstance.lean",
    )
    ap.add_argument(
        "--proof-lean-ingham-imported-slot",
        default="research/formal/lean/PrimeRiemannBridgeInghamImportedSlot.lean",
    )
    ap.add_argument("--proof-constant-provenance-json", default="research/output/proof_constant_provenance_2026-02-17.json")
    ap.add_argument("--proof-constant-provenance-md", default="research/output/proof_constant_provenance_2026-02-17.md")
    ap.add_argument("--proof-equation-citation-map-json", default="research/output/proof_equation_citation_map_2026-02-17.json")
    ap.add_argument("--proof-equation-citation-map-md", default="research/output/proof_equation_citation_map_2026-02-17.md")
    ap.add_argument("--lean-contract-bridge-json", default="research/output/lean_contract_bridge_2026-02-17.json")
    ap.add_argument("--lean-contract-bridge-md", default="research/output/lean_contract_bridge_2026-02-17.md")
    ap.add_argument("--lean-contract-satisfiability-json", default="research/output/lean_contract_satisfiability_2026-02-17.json")
    ap.add_argument("--lean-contract-satisfiability-md", default="research/output/lean_contract_satisfiability_2026-02-17.md")
    ap.add_argument("--formal-axiom-audit-json", default="research/output/formal_axiom_audit_2026-02-17.json")
    ap.add_argument("--formal-axiom-audit-md", default="research/output/formal_axiom_audit_2026-02-17.md")
    ap.add_argument("--formal-compile-report", default="research/output/formal_compile_report_2026-02-17.json")
    ap.add_argument("--formal-verify-script", default="research/formal/lean/verify_formal_proof.sh")
    ap.add_argument("--formal-compile-report-mathlib", default="research/output/formal_compile_report_mathlib_2026-02-17.json")
    ap.add_argument("--formal-verify-script-mathlib", default="research/formal/lean/verify_formal_proof_mathlib.sh")
    ap.add_argument("--formal-compile-report-completion-kernel", default="research/output/formal_compile_report_completion_kernel_2026-02-17.json")
    ap.add_argument("--formal-verify-script-completion-kernel", default="research/formal/lean/verify_formal_proof_completion_kernel.sh")
    ap.add_argument("--proof-remaining-obstacles", default="research/output/proof_remaining_obstacles_2026-02-17.json")
    ap.add_argument("--proof-step-update-remaining-obstacles", default="research/output/proof_step_update_2026-02-17_remaining_obstacles.md")
    ap.add_argument("--final-rh-proof-attempt", default="research/output/final_rh_proof_attempt_2026-02-17.md")
    ap.add_argument("--final-rh-proof-status", default="research/output/final_rh_proof_status_2026-02-17.json")
    ap.add_argument("--repo-derived-rh-program", default="research/output/repo_derived_rh_elimination_program_2026-02-18.md")
    ap.add_argument("--repo-derived-rh-status", default="research/output/repo_derived_rh_status_2026-02-18.json")
    ap.add_argument(
        "--final-pack-instantiation-contract",
        default="research/output/final_pack_instantiation_contract_2026-02-18.md",
    )
    ap.add_argument("--proof-truth-snapshot", default="research/output/proof_truth_snapshot_2026-02-18.json")
    ap.add_argument("--proof-resume-checkpoint", default="research/output/proof_resume_checkpoint_2026-02-18.json")
    ap.add_argument(
        "--proof-step-update-resume-checkpoint",
        default="research/output/proof_step_update_2026-02-18_resume_checkpoint.md",
    )
    ap.add_argument(
        "--proof-step-update-rh-from-provider-bridge",
        default="research/output/proof_step_update_2026-02-18_rh_from_provider_bridge.md",
    )
    ap.add_argument(
        "--proof-step-update-external-formalization-scan",
        default="research/output/proof_step_update_2026-02-18_external_formalization_scan.md",
    )
    ap.add_argument(
        "--proof-step-update-oscillatory-reduction",
        default="research/output/proof_step_update_2026-02-18_oscillatory_reduction.md",
    )
    ap.add_argument(
        "--proof-step-update-loglinear-bridge",
        default="research/output/proof_step_update_2026-02-18_loglinear_bridge.md",
    )
    ap.add_argument(
        "--proof-step-update-vonkoch-global-decomposition",
        default="research/output/proof_step_update_2026-02-18_vonkoch_global_decomposition.md",
    )
    ap.add_argument(
        "--proof-step-update-linear-phase-only-endpoints",
        default="research/output/proof_step_update_2026-02-18_linear_phase_only_endpoints.md",
    )
    ap.add_argument(
        "--proof-step-update-linear-phase-witness-route",
        default="research/output/proof_step_update_2026-02-18_linear_phase_witness_route.md",
    )
    ap.add_argument(
        "--proof-step-update-loglinear-to-witness-bridge",
        default="research/output/proof_step_update_2026-02-18_loglinear_to_witness_bridge.md",
    )
    ap.add_argument(
        "--proof-step-update-w2b-explicit-pintz-chain",
        default="research/output/proof_step_update_2026-02-18_w2b_explicit_pintz_chain.md",
    )
    ap.add_argument(
        "--proof-step-update-ingham-boundary",
        default="research/output/proof_step_update_2026-02-18_ingham_boundary.md",
    )
    ap.add_argument(
        "--proof-step-update-ingham-imported-slot",
        default="research/output/proof_step_update_2026-02-18_ingham_imported_slot.md",
    )
    ap.add_argument(
        "--proof-step-update-asymptotic-to-ingham-bridge",
        default="research/output/proof_step_update_2026-02-18_asymptotic_to_ingham_bridge.md",
    )
    ap.add_argument(
        "--linear-phase-witness-lemma-queue-json",
        default="research/output/linear_phase_witness_lemma_queue_2026-02-18.json",
    )
    ap.add_argument(
        "--linear-phase-witness-lemma-queue-md",
        default="research/output/linear_phase_witness_lemma_queue_2026-02-18.md",
    )
    ap.add_argument(
        "--w2b-import-contract-json",
        default="research/output/w2b_import_contract_2026-02-18.json",
    )
    ap.add_argument(
        "--w2b-import-contract-md",
        default="research/output/w2b_import_contract_2026-02-18.md",
    )
    ap.add_argument(
        "--w2b-source-lineage-json",
        default="research/output/w2b_source_lineage_2026-02-18.json",
    )
    ap.add_argument(
        "--w2b-source-lineage-md",
        default="research/output/w2b_source_lineage_2026-02-18.md",
    )
    ap.add_argument("--proof-skeleton", default="research/output/rh_bridge_candidate_b_proof_skeleton.md")
    ap.add_argument("--draft", default="research/output/rh_assumption_theorem_draft.md")
    ap.add_argument("--o1-skeleton", default="research/output/o1_ref_residual_asymptotic_scaffold_2026-02-17.json")
    ap.add_argument("--o1-theorem-budget-check", default="research/output/o1_theorem_budget_endpoint_check_2026-02-17.json")
    ap.add_argument("--o1-u-residual-handoff", default="research/output/o1_u_residual_o1_handoff_2026-02-17.json")
    ap.add_argument("--o1-theorem-closure-pack", default="research/output/o1_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o2-skeleton", default="research/output/a2_infinite_tail_lemma_skeleton_2026-02-17.json")
    ap.add_argument("--o2-sum-integral-checker", default="research/output/a2_sum_to_integral_domination_checker_2026-02-17.json")
    ap.add_argument("--o2-theorem-unconditionalization-pack", default="research/output/o2_theorem_unconditionalization_pack_2026-02-17.json")
    ap.add_argument("--o2-theorem-closure-pack", default="research/output/o2_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o3-skeleton", default="research/output/a3_analytic_closure_lemma_skeleton_2026-02-17.json")
    ap.add_argument("--o3-ref-skeleton", default="research/output/o3_ref_bridge_asymptotic_scaffold_2026-02-17.json")
    ap.add_argument("--o3-derived-bridge", default="research/output/o3_ref_bridge_from_e2_closure_2026-02-17.json")
    ap.add_argument("--o3-unconditionalization-contract", default="research/output/o3_unconditionalization_contract_2026-02-17.json")
    ap.add_argument("--o3-direct-envelope-pack", default="research/output/o3_direct_envelope_derivation_pack_2026-02-17.json")
    ap.add_argument("--o3-l-offabs-direct-draft", default="research/output/o3_l_offabs_direct_draft_2026-02-17.json")
    ap.add_argument("--o3-l-offabs-theorem-closure", default="research/output/o3_l_offabs_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-l-offsign-direct-draft", default="research/output/o3_l_offsign_direct_draft_2026-02-17.json")
    ap.add_argument("--o3-l-offsign-theorem-closure", default="research/output/o3_l_offsign_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-l-diag-direct-draft", default="research/output/o3_l_diag_direct_draft_2026-02-17.json")
    ap.add_argument("--o3-l-diag-theorem-closure", default="research/output/o3_l_diag_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-l-asm-direct-draft", default="research/output/o3_l_asm_direct_draft_2026-02-17.json")
    ap.add_argument("--o3-l-asm-theorem-closure", default="research/output/o3_l_asm_theorem_closure_2026-02-17.json")
    ap.add_argument("--o3-u-rem-handoff", default="research/output/o3_u_rem_o2_handoff_2026-02-17.json")
    ap.add_argument("--o3-u-offdiag-handoff", default="research/output/o3_u_offdiag_o3_handoff_2026-02-17.json")
    ap.add_argument("--o3-u-diag-handoff", default="research/output/o3_u_diag_o3_handoff_2026-02-17.json")
    ap.add_argument("--o3-u-uniformity-handoff", default="research/output/o3_u_uniformity_o4_handoff_2026-02-17.json")
    ap.add_argument("--o3-theorem-budget", default="research/output/o3_sign_sensitive_theorem_budget_2026-02-17.json")
    ap.add_argument("--o3-sign-quantization", default="research/output/o3_sign_cancellation_quantization_2026-02-17.json")
    ap.add_argument("--o3-asymptotic-draft", default="research/output/o3_asymptotic_inequality_draft_2026-02-17.json")
    ap.add_argument("--o3-sign-impact-audit", default="research/output/o3_sign_cancellation_impact_audit_2026-02-17.json")
    ap.add_argument("--o3-e2-target-pack", default="research/output/o3_e2_theorem_target_pack_2026-02-17.json")
    ap.add_argument("--o3-e2-lemma", default="research/output/o3_e2_asymptotic_lemma_scaffold_2026-02-17.json")
    ap.add_argument("--o3-e2-full-proof-draft", default="research/output/o3_e2_full_proof_draft_2026-02-17.json")
    ap.add_argument("--o3-e2-offdiag-sign-draft", default="research/output/o3_e2_offdiag_sign_symbolic_draft_2026-02-17.json")
    ap.add_argument("--o3-e2-offdiag-closure", default="research/output/o3_e2_offdiag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--o3-e2-diag-draft", default="research/output/o3_e2_diag_symbolic_draft_2026-02-17.json")
    ap.add_argument("--o3-e2-diag-closure", default="research/output/o3_e2_diag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--o3-e2-remainder-draft", default="research/output/o3_e2_remainder_symbolic_draft_2026-02-17.json")
    ap.add_argument("--o3-e2-remainder-closure", default="research/output/o3_e2_remainder_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--o3-e2-glue-draft", default="research/output/o3_e2_glue_symbolic_draft_2026-02-17.json")
    ap.add_argument("--o3-e2-glue-closure", default="research/output/o3_e2_glue_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--o4-skeleton", default="research/output/o4_uniformity_asymptotic_scaffold_2026-02-17.json")
    ap.add_argument("--o4-theorem-closure-pack", default="research/output/o4_theorem_closure_pack_2026-02-17.json")
    ap.add_argument("--o5-skeleton", default="research/output/o5_rh_equivalent_implication_skeleton_2026-02-17.json")
    ap.add_argument("--o5-mref-lemma", default="research/output/o5_mref_transfer_lemma_scaffold_2026-02-17.json")
    ap.add_argument("--o5-integrated-draft", default="research/output/o5_integrated_implication_draft_2026-02-17.json")
    ap.add_argument("--o5-theorem-budget-check", default="research/output/o5_theorem_budget_endpoint_check_2026-02-17.json")
    ap.add_argument("--o5-theorem-closure", default="research/output/o5_theorem_closure_2026-02-17.json")
    ap.add_argument("--obligation-ledger", default="research/output/proof_obligation_ledger.json")
    ap.add_argument("--pipeline-manifest", default="research/output/hx_batch_manifest.json")
    ap.add_argument("--output", default="research/output/proof_canonical_manifest.json")
    args = ap.parse_args()

    canonical: Dict[str, str] = {
        "lemma_a": args.lemma_a,
        "lemma_b": args.lemma_b,
        "lemma_c_triangle": args.lemma_c_triangle,
        "lemma_c_ineq": args.lemma_c_ineq,
        "lemma_d": args.lemma_d,
        "lemma_e": args.lemma_e,
        "a1": args.a1,
        "a2": args.a2,
        "a3": args.a3,
        "a3_stress": args.a3_stress,
        "a3_sign_sensitive_fallback": args.a3_sign_sensitive_fallback,
        "a4": args.a4,
        "theorem_pack": args.theorem_pack,
        "proof_closure_tracker": args.proof_closure_tracker,
        "proof_requirement_closure_table": args.proof_requirement_closure_table,
        "formal_proof_manuscript": args.formal_proof_manuscript,
        "formal_proof_dependency_map": args.formal_proof_dependency_map,
        "formal_proof_packet_index": args.formal_proof_packet_index,
        "proof_grade_manuscript": args.proof_grade_manuscript,
        "proof_citation_audit_json": args.proof_citation_audit_json,
        "proof_citation_audit_md": args.proof_citation_audit_md,
        "proof_formalization_index": args.proof_formalization_index,
        "proof_lean_scaffold": args.proof_lean_scaffold,
        "proof_lean_mathlib": args.proof_lean_mathlib,
        "proof_lean_completion_kernel": args.proof_lean_completion_kernel,
        "proof_lean_oscillatory_reduction": args.proof_lean_oscillatory_reduction,
        "proof_lean_concrete_pack_instantiation": args.proof_lean_concrete_pack_instantiation,
        "proof_lean_w2b_final_slot": args.proof_lean_w2b_final_slot,
        "proof_lean_w2b_imported_instance": args.proof_lean_w2b_imported_instance,
        "proof_lean_ingham_imported_slot": args.proof_lean_ingham_imported_slot,
        "proof_constant_provenance_json": args.proof_constant_provenance_json,
        "proof_constant_provenance_md": args.proof_constant_provenance_md,
        "proof_equation_citation_map_json": args.proof_equation_citation_map_json,
        "proof_equation_citation_map_md": args.proof_equation_citation_map_md,
        "lean_contract_bridge_json": args.lean_contract_bridge_json,
        "lean_contract_bridge_md": args.lean_contract_bridge_md,
        "lean_contract_satisfiability_json": args.lean_contract_satisfiability_json,
        "lean_contract_satisfiability_md": args.lean_contract_satisfiability_md,
        "formal_axiom_audit_json": args.formal_axiom_audit_json,
        "formal_axiom_audit_md": args.formal_axiom_audit_md,
        "formal_compile_report": args.formal_compile_report,
        "formal_verify_script": args.formal_verify_script,
        "formal_compile_report_mathlib": args.formal_compile_report_mathlib,
        "formal_verify_script_mathlib": args.formal_verify_script_mathlib,
        "formal_compile_report_completion_kernel": args.formal_compile_report_completion_kernel,
        "formal_verify_script_completion_kernel": args.formal_verify_script_completion_kernel,
        "proof_remaining_obstacles": args.proof_remaining_obstacles,
        "proof_step_update_remaining_obstacles": args.proof_step_update_remaining_obstacles,
        "final_rh_proof_attempt": args.final_rh_proof_attempt,
        "final_rh_proof_status": args.final_rh_proof_status,
        "repo_derived_rh_program": args.repo_derived_rh_program,
        "repo_derived_rh_status": args.repo_derived_rh_status,
        "final_pack_instantiation_contract": args.final_pack_instantiation_contract,
        "proof_truth_snapshot": args.proof_truth_snapshot,
        "proof_resume_checkpoint": args.proof_resume_checkpoint,
        "proof_step_update_resume_checkpoint": args.proof_step_update_resume_checkpoint,
        "proof_step_update_rh_from_provider_bridge": args.proof_step_update_rh_from_provider_bridge,
        "proof_step_update_external_formalization_scan": args.proof_step_update_external_formalization_scan,
        "proof_step_update_oscillatory_reduction": args.proof_step_update_oscillatory_reduction,
        "proof_step_update_loglinear_bridge": args.proof_step_update_loglinear_bridge,
        "proof_step_update_vonkoch_global_decomposition": args.proof_step_update_vonkoch_global_decomposition,
        "proof_step_update_linear_phase_only_endpoints": args.proof_step_update_linear_phase_only_endpoints,
        "proof_step_update_linear_phase_witness_route": args.proof_step_update_linear_phase_witness_route,
        "proof_step_update_loglinear_to_witness_bridge": args.proof_step_update_loglinear_to_witness_bridge,
        "proof_step_update_w2b_explicit_pintz_chain": args.proof_step_update_w2b_explicit_pintz_chain,
        "proof_step_update_ingham_boundary": args.proof_step_update_ingham_boundary,
        "proof_step_update_ingham_imported_slot": args.proof_step_update_ingham_imported_slot,
        "proof_step_update_asymptotic_to_ingham_bridge": args.proof_step_update_asymptotic_to_ingham_bridge,
        "linear_phase_witness_lemma_queue_json": args.linear_phase_witness_lemma_queue_json,
        "linear_phase_witness_lemma_queue_md": args.linear_phase_witness_lemma_queue_md,
        "w2b_import_contract_json": args.w2b_import_contract_json,
        "w2b_import_contract_md": args.w2b_import_contract_md,
        "w2b_source_lineage_json": args.w2b_source_lineage_json,
        "w2b_source_lineage_md": args.w2b_source_lineage_md,
        "proof_skeleton": args.proof_skeleton,
        "draft": args.draft,
        "o1_skeleton": args.o1_skeleton,
        "o1_theorem_budget_check": args.o1_theorem_budget_check,
        "o1_u_residual_handoff": args.o1_u_residual_handoff,
        "o1_theorem_closure_pack": args.o1_theorem_closure_pack,
        "o2_skeleton": args.o2_skeleton,
        "o2_sum_integral_checker": args.o2_sum_integral_checker,
        "o2_theorem_unconditionalization_pack": args.o2_theorem_unconditionalization_pack,
        "o2_theorem_closure_pack": args.o2_theorem_closure_pack,
        "o3_skeleton": args.o3_skeleton,
        "o3_ref_skeleton": args.o3_ref_skeleton,
        "o3_derived_bridge": args.o3_derived_bridge,
        "o3_unconditionalization_contract": args.o3_unconditionalization_contract,
        "o3_direct_envelope_pack": args.o3_direct_envelope_pack,
        "o3_l_offabs_direct_draft": args.o3_l_offabs_direct_draft,
        "o3_l_offabs_theorem_closure": args.o3_l_offabs_theorem_closure,
        "o3_l_offsign_direct_draft": args.o3_l_offsign_direct_draft,
        "o3_l_offsign_theorem_closure": args.o3_l_offsign_theorem_closure,
        "o3_l_diag_direct_draft": args.o3_l_diag_direct_draft,
        "o3_l_diag_theorem_closure": args.o3_l_diag_theorem_closure,
        "o3_l_asm_direct_draft": args.o3_l_asm_direct_draft,
        "o3_l_asm_theorem_closure": args.o3_l_asm_theorem_closure,
        "o3_u_rem_handoff": args.o3_u_rem_handoff,
        "o3_u_offdiag_handoff": args.o3_u_offdiag_handoff,
        "o3_u_diag_handoff": args.o3_u_diag_handoff,
        "o3_u_uniformity_handoff": args.o3_u_uniformity_handoff,
        "o3_theorem_budget": args.o3_theorem_budget,
        "o3_sign_quantization": args.o3_sign_quantization,
        "o3_asymptotic_draft": args.o3_asymptotic_draft,
        "o3_sign_impact_audit": args.o3_sign_impact_audit,
        "o3_e2_target_pack": args.o3_e2_target_pack,
        "o3_e2_lemma": args.o3_e2_lemma,
        "o3_e2_full_proof_draft": args.o3_e2_full_proof_draft,
        "o3_e2_offdiag_sign_draft": args.o3_e2_offdiag_sign_draft,
        "o3_e2_offdiag_closure": args.o3_e2_offdiag_closure,
        "o3_e2_diag_draft": args.o3_e2_diag_draft,
        "o3_e2_diag_closure": args.o3_e2_diag_closure,
        "o3_e2_remainder_draft": args.o3_e2_remainder_draft,
        "o3_e2_remainder_closure": args.o3_e2_remainder_closure,
        "o3_e2_glue_draft": args.o3_e2_glue_draft,
        "o3_e2_glue_closure": args.o3_e2_glue_closure,
        "o4_skeleton": args.o4_skeleton,
        "o4_theorem_closure_pack": args.o4_theorem_closure_pack,
        "o5_skeleton": args.o5_skeleton,
        "o5_mref_lemma": args.o5_mref_lemma,
        "o5_integrated_draft": args.o5_integrated_draft,
        "o5_theorem_budget_check": args.o5_theorem_budget_check,
        "o5_theorem_closure": args.o5_theorem_closure,
        "obligation_ledger": args.obligation_ledger,
        "pipeline_manifest": args.pipeline_manifest,
    }

    obligation_statuses = derive_obligation_statuses(args.proof_requirement_closure_table)

    payload: Dict[str, Any] = {
        "schema_version": "1.0",
        "updated_utc": datetime.now(timezone.utc).isoformat(),
        "profile": args.profile,
        "purpose": "Single source of truth for canonical proof and pipeline artifacts.",
        "canonical": canonical,
        "policy": {
            "use_manifest_defaults": True,
            "override_via_cli_allowed": True,
            "branching_rule": "When branch-testing alternatives, write new artifacts but keep canonical pointers stable until explicitly promoted.",
            "proof_gate_rule": "Every proof step must update proof_obligation_ledger with exactly one O1-O5 target, removed assumption, and evidence.",
        },
        "proof_obligations": obligation_statuses,
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# Proof Canonical Manifest",
        "",
        f"- Profile: `{payload['profile']}`",
        f"- Updated (UTC): `{payload['updated_utc']}`",
        "",
        "## Canonical Artifact Pointers",
        "",
        "| key | path |",
        "|---|---|",
    ]
    for k, v in canonical.items():
        lines.append(f"| {k} | `{v}` |")
    lines += [
        "",
        "## Obligations",
        "",
        "| id | status |",
        "|---|---|",
    ]
    for k, v in payload["proof_obligations"].items():
        lines.append(f"| {k} | {v} |")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
