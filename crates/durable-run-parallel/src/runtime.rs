use tokio::runtime::{Builder, Runtime};
use std::sync::Arc;

/// Builder for creating customized AsyncRuntime instances.
pub struct AsyncRuntimeBuilder {
    builder: Builder,
}

impl AsyncRuntimeBuilder {
    /// Creates a new builder with default multi-threaded configuration.
    pub fn new() -> Self {
        Self {
            builder: Builder::new_multi_thread(),
        }
    }

    /// Creates a new builder for a current-thread runtime.
    pub fn new_current_thread() -> Self {
        Self {
            builder: Builder::new_current_thread(),
        }
    }

    /// Sets the number of worker threads for the runtime.
    ///
    /// This is only relevant for multi-threaded runtimes.
    pub fn worker_threads(mut self, threads: usize) -> Self {
        self.builder.worker_threads(threads);
        self
    }

    /// Sets the maximum number of blocking threads.
    pub fn max_blocking_threads(mut self, threads: usize) -> Self {
        self.builder.max_blocking_threads(threads);
        self
    }

    /// Enables all Tokio features (timer, io, etc.).
    pub fn enable_all(mut self) -> Self {
        self.builder.enable_all();
        self
    }

    /// Builds the AsyncRuntime.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new runtime or an error if creation fails.
    pub fn build(mut self) -> Result<AsyncRuntime, std::io::Error> {
        let runtime = self.builder.build()?;
        Ok(AsyncRuntime {
            inner: Arc::new(runtime),
        })
    }
}

impl Default for AsyncRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper around Tokio's async runtime for managing asynchronous execution.
///
/// This provides a convenient interface for creating and managing a Tokio runtime,
/// allowing for blocking operations on futures and proper resource management.
pub struct AsyncRuntime {
    inner: Arc<Runtime>,
}

impl AsyncRuntime {
    /// Creates a new multi-threaded async runtime with all features enabled.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the new runtime or an error if creation fails.
    pub fn new() -> Result<Self, std::io::Error> {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()?;
        Ok(Self {
            inner: Arc::new(runtime),
        })
    }

    /// Returns a handle to the runtime for spawning tasks.
    pub fn handle(&self) -> tokio::runtime::Handle {
        self.inner.handle().clone()
    }

    /// Blocks the current thread until the given future completes.
    ///
    /// This is useful for integrating async code into synchronous contexts.
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.inner.block_on(future)
    }

    /// Spawns a new asynchronous task on this runtime.
    ///
    /// # Arguments
    ///
    /// * `future` - The future to spawn as a task
    ///
    /// # Returns
    ///
    /// A `JoinHandle` that can be used to await the result of the task
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.inner.spawn(future)
    }

    /// Shuts down the runtime, waiting for at most `duration` for all spawned tasks to complete.
    ///
    /// # Arguments
    ///
    /// * `duration` - Maximum time to wait for tasks to complete
    pub fn shutdown_timeout(self, duration: std::time::Duration) {
        if let Ok(runtime) = Arc::try_unwrap(self.inner) {
            runtime.shutdown_timeout(duration);
        }
        // If there are other references, shutdown is not possible
    }

    /// Shuts down the runtime in the background.
    ///
    /// This method starts the shutdown process and returns immediately. The runtime will
    /// continue shutting down in the background. This is useful when you want to start
    /// shutdown but don't need to wait for it to complete.
    pub fn shutdown_background(self) {
        if let Ok(runtime) = Arc::try_unwrap(self.inner) {
            runtime.shutdown_background();
        }
        // If there are other references, we can't shut down, but that's okay
    }


}

impl Clone for AsyncRuntime {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl std::fmt::Debug for AsyncRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncRuntime")
            .field("inner", &"Arc<Runtime>")
            .finish()
    }
}

impl Default for AsyncRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create default async runtime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_runtime_creation() {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        // Test that we can get a handle
        let _handle = runtime.handle();
    }

    #[test]
    fn test_async_runtime_block_on() {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        let result = runtime.block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_async_runtime_default() {
        let _runtime = AsyncRuntime::default();
    }

    #[test]
    fn test_async_runtime_spawn() {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        let result = runtime.block_on(async {
            let handle = runtime.spawn(async { 42 });
            handle.await.expect("Task failed")
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_async_runtime_builder() {
        let runtime = AsyncRuntimeBuilder::new()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to build runtime");
        let _handle = runtime.handle();
    }

    #[test]
    fn test_async_runtime_builder_current_thread() {
        let runtime = AsyncRuntimeBuilder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build runtime");
        let _handle = runtime.handle();
    }

    #[test]
    fn test_async_runtime_shutdown_background() {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        runtime.shutdown_background();
        // Runtime is shut down, but we can't test much here
    }

    #[test]
    fn test_async_runtime_shutdown_timeout() {
        let runtime = AsyncRuntime::new().expect("Failed to create runtime");
        runtime.shutdown_timeout(std::time::Duration::from_millis(100));
        // Runtime is shut down
    }


}