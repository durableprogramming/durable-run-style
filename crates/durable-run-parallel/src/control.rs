use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Controls the maximum number of parallel tasks that can execute concurrently.
///
/// This controller uses atomic operations to safely manage task counts across threads,
/// providing methods to check availability, start tasks, and finish tasks.
#[derive(Clone)]
pub struct ParallelismController {
    max_parallel: usize,
    active_count: Arc<AtomicUsize>,
}

impl ParallelismController {
    /// Creates a new controller with the specified maximum parallelism.
    pub fn new(max_parallel: usize) -> Self {
        Self {
            max_parallel,
            active_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Returns true if a new task can be started without exceeding the maximum parallelism.
    pub fn can_start(&self) -> bool {
        self.active_count.load(Ordering::SeqCst) < self.max_parallel
    }

    /// Attempts to start a new task.
    ///
    /// Returns true if the task was successfully started, false if the maximum
    /// parallelism would be exceeded.
    pub fn start_task(&self) -> bool {
        loop {
            let current = self.active_count.load(Ordering::SeqCst);
            if current >= self.max_parallel {
                return false;
            }
            if self.active_count.compare_exchange(current, current + 1, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                return true;
            }
        }
    }

    /// Marks a task as finished, decrementing the active task count.
    pub fn finish_task(&self) {
        self.active_count.fetch_sub(1, Ordering::SeqCst);
    }

    /// Returns the current number of active tasks.
    pub fn active_tasks(&self) -> usize {
        self.active_count.load(Ordering::SeqCst)
    }

    /// Updates the maximum parallelism limit.
    ///
    /// Note: This does not affect currently running tasks.
    pub fn set_max_parallel(&mut self, max: usize) {
        self.max_parallel = max;
    }

    /// Returns the maximum allowed parallelism.
    pub fn max_parallel(&self) -> usize {
        self.max_parallel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallelism_controller_new() {
        let controller = ParallelismController::new(2);
        assert_eq!(controller.max_parallel(), 2);
        assert_eq!(controller.active_tasks(), 0);
        assert!(controller.can_start());
    }

    #[test]
    fn test_parallelism_controller_start_task() {
        let controller = ParallelismController::new(2);

        // Start first task
        assert!(controller.start_task());
        assert_eq!(controller.active_tasks(), 1);
        assert!(controller.can_start());

        // Start second task
        assert!(controller.start_task());
        assert_eq!(controller.active_tasks(), 2);
        assert!(!controller.can_start());

        // Try to start third task (should fail)
        assert!(!controller.start_task());
        assert_eq!(controller.active_tasks(), 2);
    }

    #[test]
    fn test_parallelism_controller_finish_task() {
        let controller = ParallelismController::new(2);

        controller.start_task();
        controller.start_task();
        assert_eq!(controller.active_tasks(), 2);

        controller.finish_task();
        assert_eq!(controller.active_tasks(), 1);
        assert!(controller.can_start());

        controller.finish_task();
        assert_eq!(controller.active_tasks(), 0);
        assert!(controller.can_start());
    }

    #[test]
    fn test_parallelism_controller_set_max_parallel() {
        let mut controller = ParallelismController::new(1);
        assert_eq!(controller.max_parallel(), 1);

        controller.set_max_parallel(3);
        assert_eq!(controller.max_parallel(), 3);
        assert!(controller.can_start());
    }

    #[test]
    fn test_parallelism_controller_edge_cases() {
        let controller = ParallelismController::new(0);
        assert!(!controller.can_start());
        assert!(!controller.start_task());
        assert_eq!(controller.active_tasks(), 0);
    }

    #[tokio::test]
    async fn test_parallelism_controller_concurrent_access() {
        let controller = ParallelismController::new(2);
        let mut handles = vec![];

        // Spawn 4 tasks that try to start
        for _ in 0..4 {
            let controller = controller.clone();
            let handle = tokio::spawn(async move {
                if controller.start_task() {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    controller.finish_task();
                    true
                } else {
                    false
                }
            });
            handles.push(handle);
        }

        let results: Vec<bool> = futures::future::join_all(handles).await.into_iter().map(|r| r.unwrap()).collect();
        let successful_starts = results.iter().filter(|&&b| b).count();

        // Should have exactly 2 successful starts
        assert_eq!(successful_starts, 2);
        assert_eq!(controller.active_tasks(), 0);
    }
}