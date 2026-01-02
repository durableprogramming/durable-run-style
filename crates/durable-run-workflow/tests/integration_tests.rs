use durable_run_workflow::*;
use durable_run_parallel::{MockExecutor, CommandExecutor, ParallelError};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A mock executor that fails a configurable number of times before succeeding
struct FailingMockExecutor {
    fail_count: AtomicUsize,
    max_failures: usize,
}

impl FailingMockExecutor {
    fn new(max_failures: usize) -> Self {
        Self {
            fail_count: AtomicUsize::new(0),
            max_failures,
        }
    }
}

#[async_trait]
impl CommandExecutor for FailingMockExecutor {
    async fn execute(&self, command: &str) -> durable_run_parallel::Result<String> {
        let current_fail = self.fail_count.fetch_add(1, Ordering::SeqCst);
        if current_fail < self.max_failures {
            Err(ParallelError::CommandFailed {
                command: command.to_string(),
                exit_code: Some(1),
                stderr: format!("Mock failure {} for command: {}", current_fail + 1, command),
            })
        } else {
            Ok(format!("Mock executed after {} failures: {}", self.max_failures, command))
        }
    }
}

#[tokio::test]
async fn test_simple_pipeline_execution() {
    let mut pipeline = Pipeline::new();

    let task1 = CommandTask::new(
        "task1".to_string(),
        "echo 'hello'".to_string(),
        MockExecutor,
    );

    let task2 = CommandTask::new(
        "task2".to_string(),
        "echo 'world'".to_string(),
        MockExecutor,
    ).with_dependencies(vec!["task1".to_string()]);

    pipeline.add_task(task1).add_task(task2);

    let result = pipeline.execute().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_conditional_execution() {
    let mut pipeline = Pipeline::new();

    let task1 = CommandTask::new(
        "task1".to_string(),
        "echo 'success'".to_string(),
        MockExecutor,
    );

    let conditional = ConditionalExecution::new("result.task1 == \"Mock executed: echo 'success'\"".to_string());
    let task2 = CommandTask::new(
        "task2".to_string(),
        "echo 'conditional'".to_string(),
        MockExecutor,
    ).with_conditional(conditional).with_dependencies(vec!["task1".to_string()]);

    pipeline.add_task(task1).add_task(task2);

    let result = pipeline.execute().await;
    assert!(result.is_ok());

    // Verify that both tasks executed (condition was met)
    let context = pipeline.get_context().lock().unwrap();
    assert!(context.get_result("task1").is_some());
    assert!(context.get_result("task2").is_some());
    assert_eq!(context.get_result("task1").unwrap().as_str().unwrap(), "Mock executed: echo 'success'");
    assert_eq!(context.get_result("task2").unwrap().as_str().unwrap(), "Mock executed: echo 'conditional'");
}

#[tokio::test]
async fn test_conditional_execution_skip() {
    let mut pipeline = Pipeline::new();

    let task1 = CommandTask::new(
        "task1".to_string(),
        "echo 'failure'".to_string(),
        MockExecutor,
    );

    let conditional = ConditionalExecution::new("result.task1 == \"Mock executed: echo 'success'\"".to_string());
    let task2 = CommandTask::new(
        "task2".to_string(),
        "echo 'conditional'".to_string(),
        MockExecutor,
    ).with_conditional(conditional);

    pipeline.add_task(task1).add_task(task2);

    let result = pipeline.execute().await;
    assert!(result.is_ok());

    // Verify that task2 was skipped (condition was not met)
    let context = pipeline.get_context().lock().unwrap();
    assert!(context.get_result("task1").is_some());
    assert!(context.get_result("task2").is_none()); // task2 should not have executed
    assert_eq!(context.get_result("task1").unwrap().as_str().unwrap(), "Mock executed: echo 'failure'");
}

#[tokio::test]
async fn test_retry_mechanism() {
    let retry = RetryMechanism::new(3, 10); // Short delay for testing
    let mut pipeline = Pipeline::new();

    // Use FailingMockExecutor that fails twice then succeeds
    let failing_executor = FailingMockExecutor::new(2);
    let task = CommandTask::new(
        "retry_task".to_string(),
        "eventually_succeeds".to_string(),
        failing_executor,
    ).with_retry(retry);

    pipeline.add_task(task);

    let result = pipeline.execute().await;
    assert!(result.is_ok());

    // Verify the task succeeded after retries
    let context = pipeline.get_context().lock().unwrap();
    assert!(context.get_result("retry_task").is_some());
    assert_eq!(context.get_result("retry_task").unwrap().as_str().unwrap(), "Mock executed after 2 failures: eventually_succeeds");
}

#[tokio::test]
async fn test_retry_mechanism_exhaustion() {
    let retry = RetryMechanism::new(2, 10); // Only 2 attempts
    let mut pipeline = Pipeline::new();

    // Use FailingMockExecutor that fails 3 times (more than retry attempts)
    let failing_executor = FailingMockExecutor::new(3);
    let task = CommandTask::new(
        "retry_task_fail".to_string(),
        "always_fails".to_string(),
        failing_executor,
    ).with_retry(retry);

    pipeline.add_task(task);

    let result = pipeline.execute().await;
    assert!(result.is_err()); // Should fail after exhausting retries
}

#[tokio::test]
async fn test_circular_dependency_detection() {
    let mut pipeline = Pipeline::new();

    let task1 = CommandTask::new(
        "task1".to_string(),
        "echo 'task1'".to_string(),
        MockExecutor,
    ).with_dependencies(vec!["task2".to_string()]);

    let task2 = CommandTask::new(
        "task2".to_string(),
        "echo 'task2'".to_string(),
        MockExecutor,
    ).with_dependencies(vec!["task1".to_string()]);

    pipeline.add_task(task1).add_task(task2);

    let result = pipeline.execute().await;
    assert!(result.is_err());
    if let WorkflowError::DependencyNotMet(msg) = result.unwrap_err() {
        assert!(msg.contains("Circular dependency detected"));
    } else {
        panic!("Expected DependencyNotMet error with circular dependency message");
    }
}

#[tokio::test]
async fn test_missing_dependency() {
    let mut pipeline = Pipeline::new();

    let task = CommandTask::new(
        "task".to_string(),
        "echo 'task'".to_string(),
        MockExecutor,
    ).with_dependencies(vec!["nonexistent".to_string()]);

    pipeline.add_task(task);

    let result = pipeline.execute().await;
    assert!(result.is_err());
    if let WorkflowError::DependencyNotMet(msg) = result.unwrap_err() {
        assert!(msg.contains("not found"));
    } else {
        panic!("Expected DependencyNotMet error for missing dependency");
    }
}

#[tokio::test]
async fn test_workflow_orchestrator() {
    let mut orchestrator = WorkflowOrchestrator::new();

    let mut pipeline1 = Pipeline::new();
    let task = CommandTask::new(
        "pipeline1_task".to_string(),
        "echo 'pipeline1'".to_string(),
        MockExecutor,
    );
    pipeline1.add_task(task);
    orchestrator.add_pipeline("pipeline1".to_string(), pipeline1);

    let result = orchestrator.execute_pipeline("pipeline1").await;
    assert!(result.is_ok());

    let result = orchestrator.execute_pipeline("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_template_rendering() {
    let mut pipeline = Pipeline::new();

    // Set up context variables
    {
        let mut context = pipeline.get_context().lock().unwrap();
        context.set_variable("env".to_string(), serde_json::json!("production"));
        context.set_variable("version".to_string(), serde_json::json!("1.2.3"));
    }

    let task1 = CommandTask::new(
        "setup".to_string(),
        "echo 'setup complete'".to_string(),
        MockExecutor,
    );

    let task2 = CommandTask::new(
        "deploy".to_string(),
        "deploy {env} {version}".to_string(),
        MockExecutor,
    )
    .with_template("deploy {env} v{version}".to_string())
    .with_dependencies(vec!["setup".to_string()]);

    pipeline.add_task(task1).add_task(task2);

    let result = pipeline.execute().await;
    assert!(result.is_ok());

    // Verify template was rendered correctly
    let context = pipeline.get_context().lock().unwrap();
    assert!(context.get_result("deploy").is_some());
    let result_str = context.get_result("deploy").unwrap().as_str().unwrap();
    assert!(result_str.contains("deploy production v1.2.3"));
}

#[tokio::test]
async fn test_parallel_execution() {
    let mut pipeline = Pipeline::new();

    // Create multiple independent tasks
    let task1 = CommandTask::new(
        "task1".to_string(),
        "echo 'task1'".to_string(),
        MockExecutor,
    );

    let task2 = CommandTask::new(
        "task2".to_string(),
        "echo 'task2'".to_string(),
        MockExecutor,
    );

    let task3 = CommandTask::new(
        "task3".to_string(),
        "echo 'task3'".to_string(),
        MockExecutor,
    ).with_dependencies(vec!["task1".to_string(), "task2".to_string()]);

    pipeline.add_task(task1).add_task(task2).add_task(task3);

    let result = pipeline.execute().await;
    assert!(result.is_ok());

    // Verify all tasks executed
    let context = pipeline.get_context().lock().unwrap();
    assert!(context.get_result("task1").is_some());
    assert!(context.get_result("task2").is_some());
    assert!(context.get_result("task3").is_some());
}

#[tokio::test]
async fn test_task_failure_propagation() {
    let mut pipeline = Pipeline::new();

    // Use FailingMockExecutor that always fails
    let failing_executor = FailingMockExecutor::new(10);
    let task = CommandTask::new(
        "failing_task".to_string(),
        "always fails".to_string(),
        failing_executor,
    );

    pipeline.add_task(task);

    let result = pipeline.execute().await;
    assert!(result.is_err());
    if let WorkflowError::TaskFailed(msg) = result.unwrap_err() {
        assert!(msg.contains("Mock failure"));
    } else {
        panic!("Expected TaskFailed error");
    }
}

#[tokio::test]
async fn test_variable_conditions() {
    let mut pipeline = Pipeline::new();

    // Set up context variables
    {
        let mut context = pipeline.get_context().lock().unwrap();
        context.set_variable("status".to_string(), serde_json::json!("ready"));
    }

    let conditional = ConditionalExecution::new("var.status == \"ready\"".to_string());
    let task = CommandTask::new(
        "conditional_task".to_string(),
        "echo 'executed'".to_string(),
        MockExecutor,
    ).with_conditional(conditional);

    pipeline.add_task(task);

    let result = pipeline.execute().await;
    assert!(result.is_ok());

    // Verify task executed
    let context = pipeline.get_context().lock().unwrap();
    assert!(context.get_result("conditional_task").is_some());
}

#[tokio::test]
async fn test_workflow_orchestrator_multiple_pipelines() {
    let mut orchestrator = WorkflowOrchestrator::new();

    // Pipeline 1
    let mut pipeline1 = Pipeline::new();
    let task1 = CommandTask::new(
        "pipeline1_task".to_string(),
        "echo 'pipeline1'".to_string(),
        MockExecutor,
    );
    pipeline1.add_task(task1);
    orchestrator.add_pipeline("pipeline1".to_string(), pipeline1);

    // Pipeline 2 with dependencies
    let mut pipeline2 = Pipeline::new();
    let task2a = CommandTask::new(
        "pipeline2_task_a".to_string(),
        "echo 'pipeline2a'".to_string(),
        MockExecutor,
    );
    let task2b = CommandTask::new(
        "pipeline2_task_b".to_string(),
        "echo 'pipeline2b'".to_string(),
        MockExecutor,
    ).with_dependencies(vec!["pipeline2_task_a".to_string()]);
    pipeline2.add_task(task2a).add_task(task2b);
    orchestrator.add_pipeline("pipeline2".to_string(), pipeline2);

    // Test execution
    let result1 = orchestrator.execute_pipeline("pipeline1").await;
    assert!(result1.is_ok());

    let result2 = orchestrator.execute_pipeline("pipeline2").await;
    assert!(result2.is_ok());

    // Test listing
    let pipelines = orchestrator.list_pipelines();
    assert_eq!(pipelines.len(), 2);
    assert!(pipelines.contains(&"pipeline1".to_string()));
    assert!(pipelines.contains(&"pipeline2".to_string()));

    // Test nonexistent
    let result_nonexistent = orchestrator.execute_pipeline("nonexistent").await;
    assert!(result_nonexistent.is_err());
}

#[tokio::test]
async fn test_pipeline_performance() {
    let mut pipeline = Pipeline::new();

    // Create a pipeline with multiple tasks
    for i in 0..10 {
        let task = CommandTask::new(
            format!("task_{}", i),
            format!("echo 'task {}'", i),
            MockExecutor,
        );
        pipeline.add_task(task);
    }

    let start = std::time::Instant::now();
    let result = pipeline.execute().await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    // Should complete quickly (MockExecutor is instant)
    assert!(duration.as_millis() < 100);
}