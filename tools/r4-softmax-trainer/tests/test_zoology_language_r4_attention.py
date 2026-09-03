"""Focused synthetic transport checks; no retained artifacts or candidate scoring."""

from __future__ import annotations

import unittest
from unittest.mock import patch

import torch
from test_zoology_r4_attention import _synthetic_frames

from r4_softmax_trainer.zoology_compound_binding.model import CompoundBindingModel
from r4_softmax_trainer.zoology_compound_r4.attention import _gauge_attention
from r4_softmax_trainer.zoology_language_interface.model import (
    LanguageInterfaceModel,
    LearnedRoleReader,
)
from r4_softmax_trainer.zoology_language_r4.attention import (
    AUDIT_COUNTS,
    EXECUTIONS,
    R4LanguageInterfaceInference,
    _pool_roles,
    frame_assignment,
    work_counts,
)


def _inputs() -> tuple[torch.Tensor, torch.Tensor]:
    inputs = torch.arange(11, 61).reshape(2, 5, 5)
    lengths = torch.tensor([[2, 4, 3, 5, 4], [5, 3, 1, 4, 5]])
    valid = torch.arange(5) < lengths.unsqueeze(-1)
    return inputs.masked_fill(~valid, -999), lengths


def _model() -> LanguageInterfaceModel:
    torch.manual_seed(1079)
    return (
        LanguageInterfaceModel(CompoundBindingModel(), LearnedRoleReader())
        .eval()
        .requires_grad_(False)
    )


class LanguageR4AttentionTests(unittest.TestCase):
    def test_continuous_native_fold_skips_padding_and_preserves_prefixes(self) -> None:
        inputs, lengths = _inputs()
        frames = _synthetic_frames()
        tokens, clauses = frame_assignment(inputs, lengths, frames)
        valid = torch.arange(5) < lengths.unsqueeze(-1)
        for row in range(2):
            source = inputs[row][valid[row]]
            expected = frames.cumulative_frame_indices(source[None])[0]
            self.assertTrue(torch.equal(tokens[row][valid[row]], expected))
            self.assertTrue(
                torch.equal(clauses[row], expected[lengths[row].cumsum(0) - 1])
            )
            self.assertTrue(torch.equal(expected, source.cumsum(0) % 120))
        self.assertTrue(bool((tokens[~valid] == frames.identity_index).all()))
        changed = inputs.masked_fill(~valid, 8192)
        changed_tokens, changed_clauses = frame_assignment(changed, lengths, frames)
        self.assertTrue(torch.equal(changed_tokens, tokens))
        self.assertTrue(torch.equal(changed_clauses, clauses))
        changed[:, 4, 0] = 56
        later_tokens, later_clauses = frame_assignment(changed, lengths, frames)
        self.assertTrue(torch.equal(later_tokens[:, :4], tokens[:, :4]))
        self.assertTrue(torch.equal(later_clauses[:, :4], clauses[:, :4]))
        self.assertFalse(torch.equal(later_clauses[:, 4], clauses[:, 4]))

    def test_plain_covariance_full_roles_and_no_parameter_or_rng_changes(self) -> None:
        model, frames = _model(), _synthetic_frames()
        inputs, lengths = _inputs()
        before = {name: value.clone() for name, value in model.state_dict().items()}
        parameters = [id(value) for value in model.parameters()]
        rng = torch.get_rng_state().clone()
        wrappers = {
            name: R4LanguageInterfaceInference(model, frames, name)
            for name in EXECUTIONS
        }
        with torch.inference_mode():
            expected = model(inputs, lengths)
        with patch.object(model, "forward", wraps=model.forward) as source:
            plain = wrappers["plain"](inputs, lengths)
            source.assert_called_once_with(inputs, lengths)
        embedded_tokens = []
        handle = model.core.embedding.register_forward_pre_hook(
            lambda _module, args: embedded_tokens.append(args[0].clone())
        )
        try:
            coherent = wrappers["r4"](inputs, lengths)
        finally:
            handle.remove()
        self.assertEqual(
            sum(value.numel() for value in embedded_tokens), int(lengths.sum())
        )
        self.assertTrue(all(bool((value >= 0).all()) for value in embedded_tokens))
        for key in expected:
            torch.testing.assert_close(plain[key], expected[key], rtol=0, atol=0)
        torch.testing.assert_close(
            coherent["role_attention"], plain["role_attention"], rtol=0, atol=0
        )
        for key in ("role_vectors", "binding_attention", "logits"):
            torch.testing.assert_close(coherent[key], plain[key], rtol=1e-5, atol=1e-6)
        self.assertEqual(coherent["role_vectors"].shape, (2, 5, 3, 64))
        self.assertEqual(coherent["binding_attention"].shape, (2, 5))
        valid = torch.arange(5) < lengths.unsqueeze(-1)
        with torch.inference_mode():
            # Uniform reader coefficients require every token value, including
            # the query-location role that is computed but unused downstream.
            weights = (
                valid[:, :, None].expand(-1, -1, 3, -1).float()
                / lengths[:, :, None, None]
            )
            token_frames, clause_frames = frame_assignment(inputs, lengths, frames)
            uniform = _pool_roles(
                model,
                inputs,
                lengths,
                weights,
                token_frames,
                clause_frames,
                frames,
                permute_token_frames=False,
            )
            values = (
                model.core.embedding(inputs.masked_fill(~valid, 0)) * valid[..., None]
            )
            average = values.double().sum(2) / lengths[..., None]
            torch.testing.assert_close(
                uniform,
                average[:, :, None].expand(-1, -1, 3, -1).float(),
                rtol=1e-5,
                atol=1e-7,
            )
        self.assertTrue(torch.equal(rng, torch.get_rng_state()))
        self.assertEqual(parameters, [id(value) for value in model.parameters()])
        self.assertTrue(
            all(
                torch.equal(before[name], value)
                for name, value in model.state_dict().items()
            )
        )
        self.assertIs(model.core.embedding.weight, model.core.lm_head.weight)
        self.assertFalse(coherent["logits"].requires_grad)

    def test_each_control_changes_only_its_connection_and_has_equal_work(self) -> None:
        model, frames = _model(), _synthetic_frames()
        inputs, lengths = _inputs()
        wrappers = {
            name: R4LanguageInterfaceInference(model, frames, name)
            for name in EXECUTIONS
        }
        output = {name: wrapper(inputs, lengths) for name, wrapper in wrappers.items()}
        coherent = output["r4"]
        token_control = output["token_source_frame_permuted"]
        fact_control = output["fact_source_frame_permuted"]
        for controlled in (token_control, fact_control):
            self.assertTrue(
                torch.equal(controlled["role_attention"], coherent["role_attention"])
            )
            self.assertFalse(torch.equal(controlled["logits"], coherent["logits"]))
        self.assertFalse(
            torch.equal(token_control["role_vectors"], coherent["role_vectors"])
        )
        self.assertTrue(
            torch.equal(fact_control["role_vectors"], coherent["role_vectors"])
        )
        token_indices, clause_indices = frame_assignment(inputs, lengths, frames)
        with torch.inference_mode():
            # Independent model-coordinate token control: F_next F_true^T E.
            for row in range(2):
                for clause in range(5):
                    size = int(lengths[row, clause])
                    true = frames.frame_matrices[token_indices[row, clause, :size]]
                    corruption = true.roll(-1, 0) @ true.transpose(-1, -2)
                    embedded = (
                        model.core.embedding(inputs[row, clause, :size])
                        .double()
                        .reshape(size, 16, 4)
                    )
                    changed = torch.einsum("tij,tdj->tdi", corruption, embedded)
                    expected = (
                        torch.einsum(
                            "rt,tdi->rdi",
                            coherent["role_attention"][row, clause, :, :size].double(),
                            changed,
                        )
                        .reshape(3, 64)
                        .float()
                    )
                    torch.testing.assert_close(
                        token_control["role_vectors"][row, clause],
                        expected,
                        rtol=1e-5,
                        atol=1e-7,
                    )
            # Feed the coherent role vectors into the existing isolated fact
            # control. This also carries the complete learned-null mixture.
            rv = coherent["role_vectors"]
            core = model.core
            query = core.query_projection(
                core.compound_norm(rv[:, 4, :2].reshape(2, 1, 128))
            )
            keys = core.key_projection(
                core.compound_norm(rv[:, :4, :2].reshape(2, 4, 128))
            )
            values = core.value_projection(core.location_norm(rv[:, :4, 2]))
            keys = torch.cat((keys, core.null_key.expand(2, 1, -1)), 1)
            values = torch.cat((values, core.null_value.expand(2, 1, -1)), 1)
            sources = torch.cat(
                (clause_indices[:, :4], torch.zeros(2, 1, dtype=torch.long)), 1
            )
            context, attention = _gauge_attention(
                query,
                keys,
                values,
                frames.frame_matrices[clause_indices[:, 4]],
                frames.frame_matrices[sources],
                permute_source_frames=True,
            )
            expected = core.lm_head(core.output_norm(core.output_projection(context)))[
                :, 0
            ]
            torch.testing.assert_close(expected, fact_control["logits"], rtol=0, atol=0)
            torch.testing.assert_close(
                attention[:, 0, 0], fact_control["binding_attention"], rtol=0, atol=0
            )
        geometric_counts = work_counts(inputs, lengths, "r4")
        for execution in EXECUTIONS[1:]:
            self.assertEqual(work_counts(inputs, lengths, execution), geometric_counts)
            self.assertEqual(
                {k: wrappers[execution].audit[k] for k in AUDIT_COUNTS},
                geometric_counts,
            )
        self.assertEqual(geometric_counts["role_outputs"], 30)
        self.assertEqual(geometric_counts["role_blocks_decoded"], 480)
        self.assertEqual(
            geometric_counts["token_blocks_encoded"], 16 * int(lengths.sum())
        )
        self.assertEqual(geometric_counts["padding_blocks_encoded"], 0)
        self.assertEqual(geometric_counts["padding_blocks_transported"], 0)
        self.assertEqual(
            wrappers["token_source_frame_permuted"].audit[
                "fact_source_frame_matrices_changed"
            ],
            0,
        )
        self.assertEqual(
            wrappers["fact_source_frame_permuted"].audit[
                "token_source_frame_matrices_changed"
            ],
            0,
        )
        self.assertGreater(
            wrappers["token_source_frame_permuted"].audit[
                "token_source_frame_matrices_changed"
            ],
            0,
        )
        self.assertGreater(
            wrappers["fact_source_frame_permuted"].audit[
                "fact_source_frame_matrices_changed"
            ],
            0,
        )

    def test_padding_labels_inference_validation_and_audit_reset(self) -> None:
        model, frames = _model(), _synthetic_frames()
        inputs, lengths = _inputs()
        valid = torch.arange(5) < lengths.unsqueeze(-1)
        changed = inputs.masked_fill(~valid, 999999)
        for execution in EXECUTIONS:
            wrapper = R4LanguageInterfaceInference(model, frames, execution)
            first, second = wrapper(inputs, lengths), wrapper(changed, lengths)
            for key in first:
                torch.testing.assert_close(first[key], second[key], rtol=0, atol=0)
            self.assertEqual(wrapper.audit["rows"], 4)
            self.assertIn(frames.identity_index, wrapper.audit["reached_frame_indices"])
            with self.assertRaises(TypeError):
                wrapper(inputs, lengths, targets=torch.zeros(2, dtype=torch.long))
            with self.assertRaisesRegex(ValueError, "only control"):
                wrapper(inputs, lengths, control="value_cycle")
            wrapper.reset_audit()
            self.assertTrue(all(wrapper.audit[key] == 0 for key in AUDIT_COUNTS))
            self.assertEqual(wrapper.audit["reached_frame_indices"], [])
        with self.assertRaisesRegex(ValueError, "execution"):
            R4LanguageInterfaceInference(model, frames, "both_controls")
        with self.assertRaisesRegex(ValueError, "lengths"):
            frame_assignment(inputs, lengths * 0, frames)
        model.train()
        with self.assertRaisesRegex(RuntimeError, "eval"):
            R4LanguageInterfaceInference(model, frames)


if __name__ == "__main__":
    unittest.main()
