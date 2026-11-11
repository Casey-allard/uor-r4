use one_model_end_to_end_panel::reply_control_controller::control_reply;
use one_model_end_to_end_panel::{
    apply_flow, Decision, EosTrace, MemoryMoveTrace, MemoryTrace, NearestTrace, ReadTrace,
    TraceRow, EOS, REPLY_CAP,
};

fn row(token: u32) -> TraceRow {
    TraceRow {
        case: 0,
        reply_position: 0,
        token,
        nearest: NearestTrace { margin: 1 },
        eos: EosTrace { rank: 8 },
        memory: MemoryTrace { movement: None },
        reads: ReadTrace {
            copy_hits: 0,
            type_hits: 0,
        },
        short_cycle_period: None,
        class: "reflective".to_owned(),
    }
}

#[test]
fn empty_input_has_empty_output() {
    assert!(control_reply(&[]).is_empty());
}

#[test]
fn served_eos_is_kept_and_terminal() {
    let rows = [row(5), row(EOS), row(9)];
    assert_eq!(control_reply(&rows), vec![5, EOS]);
}

#[test]
fn injected_eos_is_terminal() {
    let rows = [row(5), row(6), row(7)];
    let ids = apply_flow(&rows, false, |_, out| {
        if out.is_empty() {
            Decision::Keep
        } else {
            Decision::EmitEos
        }
    });
    assert_eq!(ids, vec![5, EOS]);
}

#[test]
fn current_cycle_stop_uses_the_observed_cycle_mark() {
    let mut third = row(7);
    third.short_cycle_period = Some(1);
    let rows = [row(9), row(7), row(7), third, row(8)];
    assert_eq!(control_reply(&rows), vec![9, 7, 7, 7]);
}

#[test]
fn output_is_only_served_tokens_or_eos() {
    let rows = [row(5), row(8), row(EOS)];
    assert_eq!(control_reply(&rows), vec![5, 8, EOS]);
}

#[test]
fn output_never_exceeds_the_reply_cap() {
    let rows = (0..REPLY_CAP + 5)
        .map(|index| row(u32::try_from(index + 3).unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(control_reply(&rows).len(), REPLY_CAP);
}

#[test]
fn flow_emits_nothing_after_a_stop() {
    let rows = [row(EOS), row(9)];
    assert_eq!(control_reply(&rows), vec![EOS]);
}

#[test]
fn controller_receives_only_trace_features() {
    let mut permitted = row(4);
    permitted.memory.movement = Some(MemoryMoveTrace { stored: Some(9) });
    permitted.reads.copy_hits = 1;
    permitted.class = "permeable".to_owned();
    assert_eq!(control_reply(&[permitted]), vec![4]);
}
