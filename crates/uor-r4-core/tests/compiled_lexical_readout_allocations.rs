//! Focused successful-call allocation census; construction is excluded.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
struct CountingAllocator;
thread_local! {
    static MEASURING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            let _ = MEASURING.try_with(|enabled| {
                if enabled.get() {
                    let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
                    let _ = BYTES.try_with(|count| count.set(count.get() + layout.size()));
                }
            });
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// Compilation and workspace ownership are outside the measured successful readout.
#[test]
fn compiled_lexical_readout_has_zero_allocations() {
    use uor_r4_core::native_geometric::learner::state_lexical::SlFacts;
    use uor_r4_core::native_geometric::learner::transferable_lexical::{
        TlConfig, TlReadPlan, TlTrainConfig, TlTrainer,
    };
    let mut cfg = TlConfig::new(32);
    cfg.h_dim = 8;
    let mut model = TlTrainer::new(cfg, TlTrainConfig::default())
        .unwrap()
        .model()
        .unwrap();
    model.wo.packed.fill(0x19);
    let plan = TlReadPlan::compile(&model).unwrap();
    let h = vec![3; cfg.h_dim];
    let m = vec![-2; cfg.h_dim];
    let f = model.typed_block(&[], &[], SlFacts::default());
    let mut scratch = vec![0; plan.workspace_len()];
    let mut out = vec![0; plan.output_len()];
    ALLOCATIONS.with(|n| n.set(0));
    BYTES.with(|n| n.set(0));
    MEASURING.with(|n| n.set(true));
    let mut valid = true;
    for i in 0..256 {
        valid &= plan
            .score_into(&h, &m, &f, i % 4, &mut scratch, &mut out)
            .is_ok();
        std::hint::black_box(&out);
    }
    MEASURING.with(|n| n.set(false));
    assert!(valid);
    assert_eq!(ALLOCATIONS.with(Cell::get), 0);
    assert_eq!(BYTES.with(Cell::get), 0);
    assert_eq!(out, model.readout(&h, &m, &f, 3));
}
