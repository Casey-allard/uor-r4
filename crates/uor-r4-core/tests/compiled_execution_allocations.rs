//! Direct allocation census for complete prepared native requests and generation.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use uor_r4_core::native_geometric::learner::{
    state_lexical::SlFacts, tl_execution::Execution, transferable_lexical::*,
};
struct Count;
static ACTIVE: AtomicBool = AtomicBool::new(false);
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Count {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if ACTIVE.load(Ordering::Relaxed) {
            ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        if ACTIVE.load(Ordering::Relaxed) {
            ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        System.realloc(p, l, n)
    }
}
#[global_allocator]
static ALLOCATOR: Count = Count;
#[test]
fn whole_pre_tokenized_request_and_decode_are_allocation_free() {
    let m = if let Ok(p) = std::env::var("UOR_ALLOCATION_MODEL") {
        TlModel::from_bytes(&std::fs::read(p).unwrap()).unwrap()
    } else {
        TlTrainer::new(TlConfig::new(32), TlTrainConfig::default())
            .unwrap()
            .model()
            .unwrap()
    };
    let p = Execution::compile(&m).unwrap();
    let mut w = p.workspace();
    let owned = [1, 2, 3, 4, 5, 6, 7];
    let mut tok = [0; 64];
    let mut acts = [TlAction::Stop; 64];
    let before = ALLOCS.load(Ordering::SeqCst);
    ACTIVE.store(true, Ordering::SeqCst);
    let r = p.run(
        &owned,
        &[2, 3],
        SlFacts {
            history: 1,
            committed: true,
            prior_differs: true,
            ..SlFacts::default()
        },
        &[4, 5, 6],
        &owned,
        false,
        false,
        &mut w,
        &mut tok,
        &mut acts,
    );
    ACTIVE.store(false, Ordering::SeqCst);
    let after = ALLOCS.load(Ordering::SeqCst);
    assert!(r.is_ok());
    assert_eq!(after - before, 0, "complete request allocation");
    println!(
        "whole request allocations={} actions={}",
        after - before,
        r.unwrap().actions
    );
}
