//! Exclusive storage reuse. Completion stays inside the lock, even on error.
use std::sync::Mutex;

pub(super) struct ScratchCache<T> {
    state: Mutex<(Option<T>, bool)>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum CacheError<E> {
    Unavailable,
    Operation(E),
    Completion { operation: E, completion: E },
}

impl<T> ScratchCache<T> {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new((None, true)),
        }
    }

    pub(super) fn with<R, E>(
        &self,
        operation: impl FnOnce(&mut Option<T>) -> Result<R, E>,
        complete: impl FnOnce() -> Result<(), E>,
    ) -> Result<R, CacheError<E>> {
        let mut state = self.state.lock().map_err(|_| CacheError::Unavailable)?;
        if !state.1 {
            return Err(CacheError::Unavailable);
        }
        let result = operation(&mut state.0);
        if let Err(error) = complete() {
            // Keep the allocation alive, but never submit work using it again.
            state.1 = false;
            return Err(match result {
                Ok(_) => CacheError::Operation(error),
                Err(operation) => CacheError::Completion {
                    operation,
                    completion: error,
                },
            });
        }
        if result.is_err() {
            state.0 = None;
        }
        result.map_err(CacheError::Operation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_calls_reuse_storage_and_complete_before_return() {
        let cache = ScratchCache::new();
        let allocations = std::cell::Cell::new(0);
        let completions = std::cell::Cell::new(0);
        for _ in 0..3 {
            cache
                .with(
                    |slot| {
                        let value = slot.get_or_insert_with(|| {
                            allocations.set(allocations.get() + 1);
                            vec![7u8; 8]
                        });
                        assert_eq!(value.len(), 8);
                        Ok::<_, &str>(())
                    },
                    || {
                        completions.set(completions.get() + 1);
                        Ok(())
                    },
                )
                .unwrap();
        }
        assert_eq!(allocations.get(), 1);
        assert_eq!(completions.get(), 3);
    }

    #[test]
    fn failed_completion_quarantines_storage_and_refuses_reuse() {
        let cache = ScratchCache::new();
        assert_eq!(
            cache.with(
                |slot| {
                    *slot = Some(vec![1u8]);
                    Ok(())
                },
                || Err("stream failed")
            ),
            Err(CacheError::Operation("stream failed"))
        );
        assert_eq!(
            cache.with(
                |_| -> Result<(), &str> { panic!("quarantined storage must not be borrowed") },
                || panic!("no new operation can be submitted")
            ),
            Err(CacheError::Unavailable)
        );
    }

    #[test]
    fn operation_error_completes_before_evicting_storage() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };
        struct Pending(Arc<AtomicBool>);
        impl Drop for Pending {
            fn drop(&mut self) {
                assert!(self.0.load(Ordering::SeqCst));
            }
        }
        let finished = Arc::new(AtomicBool::new(false));
        let cache = ScratchCache::new();
        assert_eq!(
            cache.with(
                |slot| {
                    *slot = Some(Pending(Arc::clone(&finished)));
                    Err::<(), _>("launch refused")
                },
                || {
                    finished.store(true, Ordering::SeqCst);
                    Ok(())
                }
            ),
            Err(CacheError::Operation("launch refused"))
        );
        cache
            .with(
                |slot| {
                    assert!(slot.is_none(), "failed call must evict completed scratch");
                    Ok::<_, &str>(())
                },
                || Ok(()),
            )
            .unwrap();
    }

    #[test]
    fn simultaneous_operation_and_completion_errors_preserve_both() {
        let cache = ScratchCache::<()>::new();
        assert_eq!(
            cache.with(|_| Err::<(), _>("launch refused"), || Err("sync failed")),
            Err(CacheError::Completion {
                operation: "launch refused",
                completion: "sync failed"
            })
        );
    }

    #[test]
    fn concurrent_call_waits_for_completion() {
        use std::sync::{mpsc, Arc};
        let cache = Arc::new(ScratchCache::<()>::new());
        let (completing, wait_completing) = mpsc::channel();
        let (release, wait_release) = mpsc::channel();
        let (entered, wait_entered) = mpsc::channel();
        let first = Arc::clone(&cache);
        let worker = std::thread::spawn(move || {
            first.with(
                |_| Ok::<_, &str>(()),
                || {
                    completing.send(()).unwrap();
                    wait_release.recv().unwrap();
                    Ok(())
                },
            )
        });
        wait_completing.recv().unwrap();
        let second = Arc::clone(&cache);
        let (started, wait_started) = mpsc::channel();
        let next = std::thread::spawn(move || {
            started.send(()).unwrap();
            second.with(
                |_| {
                    entered.send(()).unwrap();
                    Ok::<_, &str>(())
                },
                || Ok(()),
            )
        });
        wait_started.recv().unwrap();
        assert!(wait_entered
            .recv_timeout(std::time::Duration::from_millis(30))
            .is_err());
        release.send(()).unwrap();
        worker.join().unwrap().unwrap();
        next.join().unwrap().unwrap();
        wait_entered.recv().unwrap();
    }

    #[test]
    fn panic_poisoning_refuses_any_later_operation() {
        let cache = ScratchCache::<()>::new();
        let interrupted = std::panic::catch_unwind(|| {
            cache.with(
                |_| -> Result<(), &str> { panic!("interrupted before completion") },
                || Ok(()),
            )
        });
        assert!(interrupted.is_err());
        assert_eq!(
            cache.with(
                |_| -> Result<(), &str> { panic!("must not reuse") },
                || Ok(())
            ),
            Err(CacheError::Unavailable)
        );
    }
}
