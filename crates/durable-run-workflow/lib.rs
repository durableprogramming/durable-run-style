//! # Durable Run Workflow
//!
//! A flexible workflow orchestration library for executing tasks with dependencies,
//! conditionals, retries, and parallel execution.
//!
//! This crate provides:
//! - Task-based workflow execution with dependency management
//! - Conditional task execution based on results or variables
//! - Retry mechanisms with configurable backoff
//! - Template-based command generation
//! - Parallel task execution
//! - Circular dependency detection
//!
//! ## Features
//!
//! - **Task Dependencies**: Define task execution order with dependency graphs
//! - **Conditional Execution**: Skip tasks based on previous results or variables
//! - **Retry Logic**: Automatic retry with exponential backoff
//! - **Template Support**: Dynamic command generation using variables and results
//! - **Parallel Execution**: Concurrent task execution with proper dependency handling
//! - **Error Handling**: Comprehensive error types and propagation
//!
//! ## Example
//!
//! ```rust,no_run
//! use durable_run_workflow::*;
//! use durable_run_parallel::ConcurrentExecutor;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut pipeline = Pipeline::new();
//!
//!     let task1 = CommandTask::new(
//!         "build".to_string(),
//!         "cargo build".to_string(),
//!         ConcurrentExecutor::new(1),
//!     );
//!
//!     let task2 = CommandTask::new(
//!         "test".to_string(),
//!         "cargo test".to_string(),
//!         ConcurrentExecutor::new(1),
//!     ).with_dependencies(vec!["build".to_string()]);
//!
//!     pipeline.add_task(task1).add_task(task2);
//!
//!     let result = pipeline.execute().await;
//!     assert!(result.is_ok());
//! }
//! ```

use async_trait::async_trait;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::time::{sleep, Duration};
use durable_run_parallel::ConcurrentExecutor;
use durable_run_template::Context as TemplateContext;

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("Task failed: {0}")]
    TaskFailed(String),
    #[error("Dependency not met: {0}")]
    DependencyNotMet(String),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, WorkflowError>;

#[async_trait]
pub trait Task: Send + Sync {
    async fn execute(&self, context: Arc<Mutex<WorkflowContext>>) -> Result<()>;
    fn id(&self) -> &str;
    fn dependencies(&self) -> &[String];
}


pub struct WorkflowContext {
    pub variables: HashMap<String, serde_json::Value>,
    pub results: HashMap<String, serde_json::Value>,
}

impl Default for WorkflowContext {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            results: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, key: String, value: serde_json::Value) {
        self.variables.insert(key, value);
    }

    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }

    pub fn set_result(&mut self, task_id: String, result: serde_json::Value) {
        self.results.insert(task_id, result);
    }

    pub fn get_result(&self, task_id: &str) -> Option<&serde_json::Value> {
        self.results.get(task_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalExecution {
    pub condition: String, // Simple condition like "result.task1 == 'success'"
}

impl ConditionalExecution {
    pub fn new(condition: String) -> Self {
        Self { condition }
    }

    pub fn evaluate(&self, context: &WorkflowContext) -> bool {
        // Simple expression evaluator supporting basic comparisons
        let condition = self.condition.trim();

        // Handle equality: result.task1 == "success"
        if let Some(eq_pos) = condition.find("==") {
            let left = condition[..eq_pos].trim();
            let right = condition[eq_pos + 2..].trim().trim_matches('"');

            // Check if left is a result reference
            if let Some(task_id) = left.strip_prefix("result.") {
                if let Some(result) = context.get_result(task_id) {
                    if let Some(s) = result.as_str() {
                        return s == right;
                    }
                }
                return false; // Result not found or not a string
            }
        }

        // Handle inequality: result.task1 != "failed"
        if let Some(ne_pos) = condition.find("!=") {
            let left = condition[..ne_pos].trim();
            let right = condition[ne_pos + 2..].trim().trim_matches('"');

            if let Some(task_id) = left.strip_prefix("result.") {
                if let Some(result) = context.get_result(task_id) {
                    if let Some(s) = result.as_str() {
                        return s != right;
                    }
                }
                return true; // If result not found, consider it not equal
            }
        }

        // Handle variable checks: var.status == "ready"
        if let Some(eq_pos) = condition.find("==") {
            let left = condition[..eq_pos].trim();
            let right = condition[eq_pos + 2..].trim().trim_matches('"');

            if let Some(var_name) = left.strip_prefix("var.") {
                if let Some(var) = context.get_variable(var_name) {
                    if let Some(s) = var.as_str() {
                        return s == right;
                    }
                }
                return false;
            }
        }

        // Default to true if condition can't be evaluated
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryMechanism {
    pub max_attempts: usize,
    pub delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl RetryMechanism {
    pub fn new(max_attempts: usize, delay_ms: u64) -> Self {
        Self {
            max_attempts,
            delay_ms,
            backoff_multiplier: 1.0,
        }
    }

    pub async fn execute_with_retry<F, Fut>(&self, mut f: F) -> Result<()>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let mut delay = self.delay_ms;
        for attempt in 1..=self.max_attempts {
            match f().await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if attempt == self.max_attempts {
                        return Err(e);
                    }
                    sleep(Duration::from_millis(delay)).await;
                    delay = (delay as f64 * self.backoff_multiplier) as u64;
                }
            }
        }
        unreachable!()
    }
}

pub struct Pipeline {
    tasks: Vec<Arc<dyn Task + Send + Sync>>,
    context: Arc<Mutex<WorkflowContext>>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            context: Arc::new(Mutex::new(WorkflowContext::new())),
        }
    }

    pub fn add_task<T: Task + Send + Sync + 'static>(&mut self, task: T) -> &mut Self {
        self.tasks.push(Arc::new(task));
        self
    }

    pub async fn execute(&self) -> Result<()> {
        // First, detect circular dependencies
        self.detect_circular_dependencies()?;

        let completed = Arc::new(Mutex::new(std::collections::HashSet::new()));
        let mut pending: std::collections::HashSet<_> = self.tasks.iter().map(|t| t.id().to_string()).collect();
        let task_map: std::collections::HashMap<_, _> = self.tasks.iter()
            .map(|t| (t.id().to_string(), Arc::clone(t)))
            .collect();

        while !pending.is_empty() {
            let mut to_execute = Vec::new();
            let mut remaining = std::collections::HashSet::new();

            for task_id in &pending {
                if let Some(task) = task_map.get(task_id) {
                    let deps_met = task.dependencies().iter().all(|dep| completed.lock().unwrap().contains(dep));
                    if deps_met {
                        to_execute.push(Arc::clone(task));
                    } else {
                        remaining.insert(task_id.clone());
                    }
                }
            }

            if to_execute.is_empty() {
                return Err(WorkflowError::DependencyNotMet("Unmet dependencies detected".to_string()));
            }

            let futures = to_execute.into_iter().map(|task| {
                let context = Arc::clone(&self.context);
                let completed = Arc::clone(&completed);
                async move {
                    task.execute(Arc::clone(&context)).await?;
                    completed.lock().unwrap().insert(task.id().to_string());
                    Ok(())
                }
            });

            join_all(futures).await.into_iter().collect::<Result<Vec<_>>>()?;
            pending = remaining;
        }

        Ok(())
    }

    /// Get the current workflow context (for testing purposes)
    pub fn get_context(&self) -> &Arc<Mutex<WorkflowContext>> {
        &self.context
    }

    fn detect_circular_dependencies(&self) -> Result<()> {
        use std::collections::{HashMap, HashSet};

        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut all_tasks = HashSet::new();

        // Build graph
        for task in &self.tasks {
            let task_id = task.id().to_string();
            all_tasks.insert(task_id.clone());
            graph.entry(task_id).or_default();

            for dep in task.dependencies() {
                graph.entry(dep.clone()).or_default().push(task.id().to_string());
            }
        }

        // Add nodes with no dependencies
        for task in &self.tasks {
            for dep in task.dependencies() {
                if !all_tasks.contains(dep) {
                    return Err(WorkflowError::DependencyNotMet(format!("Dependency '{}' not found", dep)));
                }
            }
        }

        // Simple cycle detection using DFS
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();

        fn has_cycle(
            node: &str,
            graph: &HashMap<String, Vec<String>>,
            visiting: &mut HashSet<String>,
            visited: &mut HashSet<String>,
        ) -> bool {
            if visiting.contains(node) {
                return true;
            }
            if visited.contains(node) {
                return false;
            }

            visiting.insert(node.to_string());

            if let Some(neighbors) = graph.get(node) {
                for neighbor in neighbors {
                    if has_cycle(neighbor, graph, visiting, visited) {
                        return true;
                    }
                }
            }

            visiting.remove(node);
            visited.insert(node.to_string());
            false
        }

        for task in &self.tasks {
            if has_cycle(task.id(), &graph, &mut visiting, &mut visited) {
                return Err(WorkflowError::DependencyNotMet("Circular dependency detected".to_string()));
            }
        }

        Ok(())
    }
}

pub struct WorkflowOrchestrator {
    pipelines: HashMap<String, Pipeline>,
}

impl Default for WorkflowOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowOrchestrator {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    pub fn add_pipeline(&mut self, name: String, pipeline: Pipeline) {
        self.pipelines.insert(name, pipeline);
    }

    pub async fn execute_pipeline(&self, name: &str) -> Result<()> {
        if let Some(pipeline) = self.pipelines.get(name) {
            pipeline.execute().await
        } else {
            Err(WorkflowError::TaskFailed(format!("Pipeline {} not found", name)))
        }
    }

    pub fn list_pipelines(&self) -> Vec<String> {
        self.pipelines.keys().cloned().collect()
    }
}

// Example task implementation
pub struct CommandTask<E = ConcurrentExecutor>
where
    E: durable_run_parallel::CommandExecutor + Send + Sync,
{
    id: String,
    command: String,
    template: Option<String>,
    dependencies: Vec<String>,
    conditional: Option<ConditionalExecution>,
    retry: Option<RetryMechanism>,
    executor: E,
}

impl<E: durable_run_parallel::CommandExecutor + Send + Sync> CommandTask<E> {
    pub fn new(id: String, command: String, executor: E) -> Self {
        Self {
            id,
            command,
            template: None,
            dependencies: Vec::new(),
            conditional: None,
            retry: None,
            executor,
        }
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn with_conditional(mut self, conditional: ConditionalExecution) -> Self {
        self.conditional = Some(conditional);
        self
    }

    pub fn with_retry(mut self, retry: RetryMechanism) -> Self {
        self.retry = Some(retry);
        self
    }

    pub fn with_template(mut self, template: String) -> Self {
        self.template = Some(template);
        self
    }
}

#[async_trait]
impl<E: durable_run_parallel::CommandExecutor + Send + Sync> Task for CommandTask<E> {
    async fn execute(&self, context: Arc<Mutex<WorkflowContext>>) -> Result<()> {
        // Check conditional first
        if let Some(ref conditional) = self.conditional {
            let ctx = context.lock().unwrap();
            if !conditional.evaluate(&ctx) {
                return Ok(()); // Skip execution
            }
        }

        // Render template if present
        let command = if let Some(ref template_str) = self.template {
            let ctx = context.lock().unwrap();
            let mut template_engine = durable_run_template::TemplateEngine::new();
            let mut template_context = TemplateContext::new();
            // Add workflow context variables to template context
            for (key, value) in &ctx.variables {
                if let Some(s) = value.as_str() {
                    template_context = template_context.with_var(key.clone(), s.to_string());
                }
            }
            // Add results as variables
            for (key, value) in &ctx.results {
                if let Some(s) = value.as_str() {
                    template_context = template_context.with_var(format!("result_{}", key), s.to_string());
                }
            }
            drop(ctx); // Release lock before template rendering

            template_engine.render(template_str, &mut template_context)
                .map_err(|e| WorkflowError::TaskFailed(format!("Template rendering error: {}", e)))?
        } else {
            self.command.clone()
        };

        let last_result = Arc::new(Mutex::new(None));
        let command = Arc::new(command);
        let exec = {
            let last_result = Arc::clone(&last_result);
            let command = Arc::clone(&command);
            move || {
                let last_result = Arc::clone(&last_result);
                let command = Arc::clone(&command);
                async move {
                    let result = self.executor
                        .execute(&command)
                        .await
                        .map_err(|e| WorkflowError::TaskFailed(e.to_string()))?;
                    *last_result.lock().unwrap() = Some(result);
                    Ok(())
                }
            }
        };

        if let Some(ref retry) = self.retry {
            retry.execute_with_retry(exec).await?;
        } else {
            exec().await?;
        }

        if let Some(result) = last_result.lock().unwrap().take() {
            let mut ctx = context.lock().unwrap();
            ctx.set_result(self.id.clone(), serde_json::json!(result));
        }
        Ok(())
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn dependencies(&self) -> &[String] {
        &self.dependencies
    }
}

