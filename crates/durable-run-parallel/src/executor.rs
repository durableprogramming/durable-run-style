use async_trait::async_trait;
use futures::future::join_all;
use tokio::process::Command as TokioCommand;
use tokio::task;
use tokio::time::Instant;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::{CommandQueue, ParallelismController, error::{ParallelError, Result}};

/// Callback type for receiving streaming output updates
/// Arguments: (command_index, new_output_chunk)
pub type OutputCallback = Arc<dyn Fn(usize, String) + Send + Sync>;

/// Performance metrics for command execution
#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    /// Total number of commands executed
    pub commands_executed: usize,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u128,
    /// Average execution time per command in milliseconds
    pub average_execution_time_ms: f64,
    /// Number of successful executions
    pub successful_executions: usize,
    /// Number of failed executions
    pub failed_executions: usize,
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            commands_executed: 0,
            total_execution_time_ms: 0,
            average_execution_time_ms: 0.0,
            successful_executions: 0,
            failed_executions: 0,
        }
    }
}

/// Trait for executing commands asynchronously.
///
/// This trait allows for different implementations of command execution,
/// such as local execution, remote execution, or mocked execution for testing.
#[async_trait]
pub trait CommandExecutor {
    /// Executes a single command and returns its output as a string.
    ///
    /// # Arguments
    ///
    /// * `command` - The command string to execute
    ///
    /// # Returns
    ///
    /// A Result containing the command output or an error
    async fn execute(&self, command: &str) -> Result<String>;
}

/// Executes commands concurrently with configurable parallelism limits.
///
/// This executor uses a queue to manage command execution tasks and a parallelism controller
/// to limit the number of concurrent executions, preventing system resource exhaustion
/// while maximizing throughput.
#[derive(Clone)]
pub struct ConcurrentExecutor {
    controller: ParallelismController,
    queue: Arc<CommandQueue<String>>,
    metrics: Arc<std::sync::Mutex<ExecutionMetrics>>,
}

impl ConcurrentExecutor {
    /// Creates a new executor with the specified maximum concurrency.
    ///
    /// # Panics
    ///
    /// Panics if `max_concurrent` is 0.
    pub fn new(max_concurrent: usize) -> Self {
        assert!(max_concurrent > 0, "max_concurrent must be greater than 0");
        Self {
            controller: ParallelismController::new(max_concurrent),
            queue: Arc::new(CommandQueue::new()),
            metrics: Arc::new(std::sync::Mutex::new(ExecutionMetrics::default())),
        }
    }

    /// Executes a batch of commands concurrently, respecting the concurrency limit.
    ///
    /// Commands are executed in parallel up to the maximum concurrency limit.
    /// Returns a vector of results in the same order as the input commands.
    ///
    /// # Arguments
    ///
    /// * `commands` - Vector of command strings to execute
    ///
    /// # Returns
    ///
    /// A vector of Results, one for each command, in the same order as input
    pub async fn execute_batch(&self, commands: Vec<String>) -> Vec<Result<String>> {
        let start_time = Instant::now();
        let controller = self.controller.clone();

        let handles: Vec<_> = commands.into_iter().map(|cmd| {
            let controller = controller.clone();
            let metrics = Arc::clone(&self.metrics);
            task::spawn(async move {
                // Wait until we can start a task
                while !controller.start_task() {
                    task::yield_now().await;
                }

                let result = Self::execute_command(&cmd).await;

                // Mark task as finished
                controller.finish_task();

                // Update metrics
                let mut m = metrics.lock().unwrap();
                m.commands_executed += 1;
                match &result {
                    Ok(_) => m.successful_executions += 1,
                    Err(_) => m.failed_executions += 1,
                }

                result
            })
        }).collect();

        let results = join_all(handles).await.into_iter().map(|res| res.unwrap()).collect::<Vec<_>>();
        let total_time = start_time.elapsed().as_millis();

        // Update total execution time
        let mut m = self.metrics.lock().unwrap();
        m.total_execution_time_ms += total_time;
        if m.commands_executed > 0 {
            m.average_execution_time_ms = m.total_execution_time_ms as f64 / m.commands_executed as f64;
        }

        results
    }

    /// Executes a batch of commands with streaming output support.
    ///
    /// This method executes commands concurrently while streaming output in real-time
    /// via the provided callback function. The callback is invoked for each line of output
    /// as it becomes available.
    ///
    /// # Arguments
    ///
    /// * `commands` - Vector of command strings to execute
    /// * `output_callback` - Callback function to receive streaming output (command_index, output_line)
    ///
    /// # Returns
    ///
    /// A vector of Results, one for each command, in the same order as input
    pub async fn execute_batch_streaming(&self, commands: Vec<String>, output_callback: OutputCallback) -> Vec<Result<String>> {
        let start_time = Instant::now();
        let controller = self.controller.clone();

        let handles: Vec<_> = commands.into_iter().enumerate().map(|(index, cmd)| {
            let controller = controller.clone();
            let metrics = Arc::clone(&self.metrics);
            let callback = Arc::clone(&output_callback);
            task::spawn(async move {
                // Wait until we can start a task
                while !controller.start_task() {
                    task::yield_now().await;
                }

                let result = Self::execute_command_streaming(&cmd, index, callback).await;

                // Mark task as finished
                controller.finish_task();

                // Update metrics
                let mut m = metrics.lock().unwrap();
                m.commands_executed += 1;
                match &result {
                    Ok(_) => m.successful_executions += 1,
                    Err(_) => m.failed_executions += 1,
                }

                result
            })
        }).collect();

        let results = join_all(handles).await.into_iter().map(|res| res.unwrap()).collect::<Vec<_>>();
        let total_time = start_time.elapsed().as_millis();

        // Update total execution time
        let mut m = self.metrics.lock().unwrap();
        m.total_execution_time_ms += total_time;
        if m.commands_executed > 0 {
            m.average_execution_time_ms = m.total_execution_time_ms as f64 / m.commands_executed as f64;
        }

        results
    }

    /// Executes a single command and returns its stdout output.
    ///
    /// This is a helper method that handles the low-level command execution.
    async fn execute_command(command: &str) -> Result<String> {
        let output = TokioCommand::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await?;

        if output.status.success() {
            String::from_utf8(output.stdout).map_err(ParallelError::from)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code();
            Err(ParallelError::CommandFailed {
                command: command.to_string(),
                exit_code,
                stderr,
            })
        }
    }

    /// Executes a single command with streaming output support.
    ///
    /// Output is streamed line-by-line via the callback function as it becomes available.
    async fn execute_command_streaming(command: &str, index: usize, callback: OutputCallback) -> Result<String> {
        let mut child = TokioCommand::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().expect("stdout should be piped");
        let stderr = child.stderr.take().expect("stderr should be piped");

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let mut full_output = String::new();
        let mut full_stderr = String::new();

        // Read stdout and stderr concurrently
        loop {
            tokio::select! {
                line_result = stdout_reader.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            let output_line = format!("{}\n", line);
                            full_output.push_str(&output_line);
                            callback(index, full_output.clone());
                        }
                        Ok(None) => break,
                        Err(e) => return Err(ParallelError::Io(e)),
                    }
                }
                line_result = stderr_reader.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            let output_line = format!("{}\n", line);
                            full_stderr.push_str(&output_line);
                            // Also send stderr through callback
                            let combined = format!("{}{}", full_output, full_stderr);
                            callback(index, combined);
                        }
                        Ok(None) => {}
                        Err(e) => return Err(ParallelError::Io(e)),
                    }
                }
            }
        }

        // Read any remaining stderr
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            full_stderr.push_str(&format!("{}\n", line));
        }

        // Wait for the command to complete
        let status = child.wait().await?;

        if status.success() {
            Ok(full_output)
        } else {
            let exit_code = status.code();
            Err(ParallelError::CommandFailed {
                command: command.to_string(),
                exit_code,
                stderr: full_stderr,
            })
        }
    }

    /// Returns the maximum concurrency setting.
    pub fn max_concurrent(&self) -> usize {
        self.controller.max_parallel()
    }

    /// Returns the current execution metrics.
    pub fn metrics(&self) -> ExecutionMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Enqueues a command for later execution.
    ///
    /// Commands can be executed later using `execute_queued`.
    pub fn enqueue_command(&self, command: String) {
        self.queue.enqueue(command);
    }

    /// Executes all queued commands concurrently, respecting the concurrency limit.
    ///
    /// Returns a vector of results in arbitrary order (as commands complete).
    pub async fn execute_queued(&self) -> Vec<Result<String>> {
        let controller = self.controller.clone();
        let queue = Arc::clone(&self.queue);
        let mut results = Vec::new();

        // Mark the queue as finished so workers know when to stop
        queue.finish();

        // Spawn workers to process the queue
        let mut handles = Vec::new();
        for _ in 0..self.controller.max_parallel() {
            let controller = controller.clone();
            let queue = Arc::clone(&queue);
            let handle = task::spawn(async move {
                let mut local_results = Vec::new();
                while let Some(cmd) = queue.dequeue().await {
                    // Wait until we can start a task
                    while !controller.start_task() {
                        task::yield_now().await;
                    }

                    let result = Self::execute_command(&cmd).await;

                    // Mark task as finished
                    controller.finish_task();

                    local_results.push(result);
                }
                local_results
            });
            handles.push(handle);
        }

        // Wait for all workers
        let all_results: Vec<Vec<_>> = join_all(handles).await.into_iter().map(|res| res.unwrap()).collect();

        // Flatten results
        for mut worker_results in all_results {
            results.append(&mut worker_results);
        }

        results
    }

    /// Returns the number of commands currently queued.
    pub fn queued_count(&self) -> usize {
        self.queue.len()
    }

    /// Clears all queued commands.
    pub fn clear_queue(&self) {
        self.queue.clear();
    }
}

#[async_trait]
impl CommandExecutor for ConcurrentExecutor {
    async fn execute(&self, command: &str) -> Result<String> {
        Self::execute_command(command).await
    }
}

/// A mock executor for testing purposes that doesn't execute real commands.
pub struct MockExecutor;

#[async_trait]
impl CommandExecutor for MockExecutor {
    async fn execute(&self, command: &str) -> Result<String> {
        // Return a mock success response
        Ok(format!("Mock executed: {}", command))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_executor_creation() {
        let executor = ConcurrentExecutor::new(2);
        assert_eq!(executor.max_concurrent(), 2);
    }

    #[tokio::test]
    async fn test_concurrent_executor_execute_batch() {
        let executor = ConcurrentExecutor::new(2);
        let commands = vec![
            "echo 'hello'".to_string(),
            "echo 'world'".to_string(),
        ];
        let results = executor.execute_batch(commands).await;
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert_eq!(results[0].as_ref().unwrap().trim(), "hello");
        assert_eq!(results[1].as_ref().unwrap().trim(), "world");
    }

    #[tokio::test]
    async fn test_concurrent_executor_single_execute() {
        let executor = ConcurrentExecutor::new(1);
        let result = executor.execute("echo 'test'").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "test");
    }

    #[tokio::test]
    async fn test_concurrent_executor_error_handling() {
        let executor = ConcurrentExecutor::new(1);
        let result = executor.execute("false").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_executor() {
        let executor = MockExecutor;
        let result = executor.execute("test command").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Mock executed: test command");
    }

    #[tokio::test]
    async fn test_concurrent_executor_concurrency_limit() {
        let executor = ConcurrentExecutor::new(1);
        let commands = vec![
            "sleep 0.1 && echo 'first'".to_string(),
            "echo 'second'".to_string(),
        ];
        let results = executor.execute_batch(commands).await;
        assert_eq!(results.len(), 2);
        // Both should succeed, but the second should complete after the first
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_executor_enqueue_and_execute_queued() {
        let executor = ConcurrentExecutor::new(2);

        // Enqueue commands
        executor.enqueue_command("echo 'first'".to_string());
        executor.enqueue_command("echo 'second'".to_string());
        assert_eq!(executor.queued_count(), 2);

        // Execute queued commands
        let results = executor.execute_queued().await;
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        // Queue should be empty after execution
        assert_eq!(executor.queued_count(), 0);
    }

    #[test]
    fn test_concurrent_executor_clear_queue() {
        let executor = ConcurrentExecutor::new(2);
        executor.enqueue_command("echo 'test'".to_string());
        assert_eq!(executor.queued_count(), 1);
        executor.clear_queue();
        assert_eq!(executor.queued_count(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_executor_metrics() {
        let executor = ConcurrentExecutor::new(2);
        let commands = vec![
            "echo 'hello'".to_string(),
            "echo 'world'".to_string(),
        ];

        let _results = executor.execute_batch(commands).await;
        let metrics = executor.metrics();

        assert_eq!(metrics.commands_executed, 2);
        assert_eq!(metrics.successful_executions, 2);
        assert_eq!(metrics.failed_executions, 0);
        assert!(metrics.total_execution_time_ms > 0);
        assert!(metrics.average_execution_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_execute_from_inputs() {
        let inputs = vec!["hello".to_string(), "world".to_string()];
        let results = crate::execute_from_inputs(inputs, "echo {0}", 2).await;
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert_eq!(results[0].as_ref().unwrap().trim(), "hello");
        assert_eq!(results[1].as_ref().unwrap().trim(), "world");
    }

    #[test]
    #[should_panic(expected = "max_concurrent must be greater than 0")]
    fn test_concurrent_executor_zero_concurrency() {
        let _executor = ConcurrentExecutor::new(0);
    }

    #[tokio::test]
    async fn test_execute_batch_streaming() {
        use std::sync::{Arc, Mutex};

        let executor = ConcurrentExecutor::new(2);
        let stream_updates = Arc::new(Mutex::new(Vec::new()));
        let stream_updates_clone = stream_updates.clone();

        let callback: super::OutputCallback = Arc::new(move |index, output| {
            stream_updates_clone.lock().unwrap().push((index, output.len()));
        });

        let commands = vec![
            "echo 'line1'; sleep 0.1; echo 'line2'".to_string(),
            "echo 'job2'".to_string(),
        ];

        let results = executor.execute_batch_streaming(commands, callback).await;

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        // Verify that streaming updates were received
        let updates = stream_updates.lock().unwrap();
        assert!(updates.len() > 0, "Should have received streaming updates");
    }
}