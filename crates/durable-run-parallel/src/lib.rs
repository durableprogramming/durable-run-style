//! # Durable Run Parallel
//!
//! A high-performance parallel execution library for running commands concurrently.
//!
//! This crate provides:
//! - Async runtime management with Tokio
//! - Concurrent command execution with configurable parallelism
//! - Thread-safe job queues for task management
//! - Parallelism controllers for resource management
//! - Template-based command generation from inputs
//!
//! ## Features
//!
//! - **Async Runtime**: Wrapper around Tokio runtime for easy async execution
//! - **Concurrent Execution**: Execute commands in parallel with semaphore-based limits
//! - **Job Queues**: Async queues for managing command execution tasks with generic types
//! - **Parallelism Control**: Atomic counters for managing concurrent task limits
//! - **Error Handling**: Comprehensive error handling with custom error types
//! - **Queued Execution**: Enqueue commands and execute them asynchronously
//!
//! ## Example
//!
//! ```rust,no_run
//! use durable_run_parallel::ConcurrentExecutor;
//!
//! #[tokio::main]
//! async fn main() {
//!     let executor = ConcurrentExecutor::new(4);
//!
//!     let commands = vec![
//!         "echo 'hello'".to_string(),
//!         "echo 'world'".to_string(),
//!     ];
//!
//!     let results = executor.execute_batch(commands).await;
//!     for result in results {
//!         println!("Result: {:?}", result);
//!     }
//! }
//! ```

pub mod runtime;
pub mod executor;
pub mod queue;
pub mod control;
pub mod error;

// Re-export main types for convenience
pub use runtime::AsyncRuntime;
pub use executor::{CommandExecutor, ConcurrentExecutor, MockExecutor, ExecutionMetrics, OutputCallback};
pub use queue::CommandQueue;

/// Type alias for a command queue containing strings (commands).
pub type StringCommandQueue = CommandQueue<String>;
pub use control::ParallelismController;
pub use error::{ParallelError, Result};

/// Convenience function to execute commands generated from inputs.
///
/// This function demonstrates integration with input processing systems
/// where inputs are transformed into commands for parallel execution.
/// It uses advanced templating and input processing capabilities.
///
/// # Arguments
///
/// * `inputs` - Vector of input strings
/// * `command_template` - Template string with `{0}` placeholder for input (positional)
/// * `max_concurrent` - Maximum number of concurrent executions
///
/// # Returns
///
/// Vector of results, one for each input
///
/// # Example
///
/// ```rust,no_run
/// use durable_run_parallel::execute_from_inputs;
///
/// #[tokio::main]
/// async fn main() {
///     let inputs = vec!["file1.txt".to_string(), "file2.txt".to_string()];
///     let results = execute_from_inputs(inputs, "wc -l {0}", 4).await;
///     for result in results {
///         println!("Result: {:?}", result);
///     }
/// }
/// ```
pub async fn execute_from_inputs(
    inputs: Vec<String>,
    command_template: &str,
    max_concurrent: usize,
) -> Vec<Result<String>> {
    use durable_run_template::{TemplateEngine, Context};

    // Generate commands using template engine for consistency
    let mut engine = TemplateEngine::new();
    let template = match engine.parse(command_template) {
        Ok(t) => t,
        Err(e) => return vec![Err(ParallelError::Template(Box::new(e)))],
    };

    let mut commands = Vec::new();
    for input in inputs {
        let mut context = Context::new().with_arg(0, input);
        match template.render(&mut context) {
            Ok(command) => commands.push(command),
            Err(e) => return vec![Err(ParallelError::Template(Box::new(e)))],
        }
    }

    let executor = ConcurrentExecutor::new(max_concurrent);
    executor.execute_batch(commands).await
}

/// Advanced function to execute commands from input sources with processing.
///
/// This function integrates with durable-run-input for advanced input processing
/// and durable-run-template for sophisticated templating.
///
/// # Arguments
///
/// * `input_source` - Source of inputs (file, stdin, etc.)
/// * `input_transformation` - Processing to apply to inputs (filtering, mapping, etc.)
/// * `command_template` - Template string for command generation
/// * `max_concurrent` - Maximum number of concurrent executions
///
/// # Returns
///
/// Vector of results, one for each processed input
///
/// # Example
///
/// ```rust,no_run
/// use durable_run_parallel::execute_from_source;
/// use durable_run_input::{FileSource, InputTransformation};
///
/// #[tokio::main]
/// async fn main() {
///     let source = FileSource::new("input.txt".to_string());
///     let transform = InputTransformation::new().with_filter("*.txt".to_string());
///     let results = execute_from_source(&source, transform, "wc -l {0}", 4).await;
///     for result in results {
///         println!("Result: {:?}", result);
///     }
/// }
/// ```
pub async fn execute_from_source(
    input_source: &dyn durable_run_input::InputSource,
    input_transformation: durable_run_input::InputTransformation,
    command_template: &str,
    max_concurrent: usize,
) -> Vec<Result<String>> {
    use durable_run_template::{TemplateEngine, Context};
    use durable_run_input::ProcessingPipeline;

    // Create processing pipeline
    let mut pipeline = ProcessingPipeline::new(input_transformation);

    // Process inputs
    let inputs = match pipeline.process_from_source(input_source).await {
        Ok(inputs) => inputs,
        Err(e) => return vec![Err(ParallelError::Input(Box::new(e)))],
    };

    // Generate commands using template engine
    let mut engine = TemplateEngine::new();
    let template = match engine.parse(command_template) {
        Ok(t) => t,
        Err(e) => return vec![Err(ParallelError::Template(Box::new(e)))],
    };

    let mut commands = Vec::new();
    for input in inputs {
        let mut context = Context::new().with_arg(0, input);
        match template.render(&mut context) {
            Ok(command) => commands.push(command),
            Err(e) => return vec![Err(ParallelError::Template(Box::new(e)))],
        }
    }

    let executor = ConcurrentExecutor::new(max_concurrent);
    executor.execute_batch(commands).await
}