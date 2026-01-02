
use assert_cmd::Command;
use chrono::Utc;
use durable_run_analytics::{AnalyticsDashboard, DataPoint};
use durable_run_parallel::{CommandExecutor, ConcurrentExecutor, ParallelError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use thiserror::Error;
use tokio::process::Command as TokioCommand;

#[derive(Debug, Error)]
pub enum TestingError {
    #[error("Command error: {0}")]
    Command(#[from] std::io::Error),
    #[error("Assertion error: {0}")]
    Assertion(String),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Cargo error: {0}")]
    Cargo(#[from] assert_cmd::cargo::CargoError),
}

pub type Result<T> = std::result::Result<T, TestingError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration: std::time::Duration,
    pub output: String,
    pub error: Option<String>,
}

pub struct IntegrationTester {
    test_results: Vec<TestResult>,
}

impl Default for IntegrationTester {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegrationTester {
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
        }
    }

    pub async fn run_command_test(&mut self, name: &str, command: &str, expected_exit_code: i32) -> Result<()> {
        let start = std::time::Instant::now();

        let output = TokioCommand::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let duration = start.elapsed();
        let passed = output.status.code() == Some(expected_exit_code);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let output_combined = format!("stdout: {}\nstderr: {}", stdout, stderr);
        let error = if passed { None } else { Some(format!("Expected exit code {}, got {:?}", expected_exit_code, output.status.code())) };

        let result = TestResult {
            name: name.to_string(),
            passed,
            duration,
            output: output_combined,
            error,
        };

        self.test_results.push(result);
        Ok(())
    }

    pub async fn run_file_test(&mut self, name: &str, file_path: &Path, expected_content: &str) -> Result<()> {
        let start = std::time::Instant::now();

        let content = tokio::fs::read_to_string(file_path).await?;
        let duration = start.elapsed();
        let passed = content.contains(expected_content);
        let error = if passed { None } else { Some(format!("Expected content not found in {}", file_path.display())) };

        let result = TestResult {
            name: name.to_string(),
            passed,
            duration,
            output: content,
            error,
        };

        self.test_results.push(result);
        Ok(())
    }

    pub fn get_results(&self) -> &[TestResult] {
        &self.test_results
    }

    pub fn summary(&self) -> TestSummary {
        let total = self.test_results.len();
        let passed = self.test_results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration = self.test_results.iter().map(|r| r.duration).sum();

        TestSummary {
            total,
            passed,
            failed,
            total_duration,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_duration: std::time::Duration,
}

pub struct PerformanceTester {
    results: Vec<PerformanceTestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestResult {
    pub name: String,
    pub avg_duration: std::time::Duration,
    pub min_duration: std::time::Duration,
    pub max_duration: std::time::Duration,
    pub iterations: usize,
    pub threshold: Option<std::time::Duration>,
    pub passed: bool,
}

impl Default for PerformanceTester {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceTester {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub async fn run_performance_test<F, Fut>(
        &mut self,
        name: &str,
        iterations: usize,
        threshold: Option<std::time::Duration>,
        test_fn: F,
    ) -> Result<()>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let mut durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = std::time::Instant::now();
            test_fn().await;
            durations.push(start.elapsed());
        }

        let total: std::time::Duration = durations.iter().sum();
        let avg_duration = if iterations > 0 {
            total / iterations as u32
        } else {
            std::time::Duration::from_millis(0)
        };
        let min_duration = durations.iter().min().copied().unwrap_or_default();
        let max_duration = durations.iter().max().copied().unwrap_or_default();

        let passed = threshold.is_none_or(|t| avg_duration <= t);

        let result = PerformanceTestResult {
            name: name.to_string(),
            avg_duration,
            min_duration,
            max_duration,
            iterations,
            threshold,
            passed,
        };

        self.results.push(result);
        Ok(())
    }

    pub fn get_results(&self) -> &[PerformanceTestResult] {
        &self.results
    }
}

pub struct CliTester {
    results: Vec<TestResult>,
}

impl Default for CliTester {
    fn default() -> Self {
        Self::new()
    }
}

impl CliTester {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub async fn run_cli_test(
        &mut self,
        name: &str,
        command: &str,
        args: &[&str],
        expected_exit_code: i32,
        stdout_predicate: Option<Box<dyn predicates::Predicate<str> + Send + Sync>>,
        stderr_predicate: Option<Box<dyn predicates::Predicate<str> + Send + Sync>>,
    ) -> Result<()> {
        let start = std::time::Instant::now();

        let mut cmd = Command::new(command);
        cmd.args(args);

        let assert = cmd.assert();

        let (passed, error_msg) = if let Some(expected_code) = assert.get_output().status.code() {
            if expected_code == expected_exit_code {
                let stdout = std::str::from_utf8(&assert.get_output().stdout).unwrap_or("");
                let stderr = std::str::from_utf8(&assert.get_output().stderr).unwrap_or("");

                let stdout_ok = stdout_predicate.as_ref().is_none_or(|p| p.eval(stdout));
                let stderr_ok = stderr_predicate.as_ref().is_none_or(|p| p.eval(stderr));

                if stdout_ok && stderr_ok {
                    (true, None)
                } else {
                    let mut msg = Vec::new();
                    if let Some(p) = &stdout_predicate {
                        if !p.eval(stdout) {
                            msg.push("stdout predicate failed".to_string());
                        }
                    }
                    if let Some(p) = &stderr_predicate {
                        if !p.eval(stderr) {
                            msg.push("stderr predicate failed".to_string());
                        }
                    }
                    (false, Some(msg.join("; ")))
                }
            } else {
                (false, Some(format!("Expected exit code {}, got {}", expected_exit_code, expected_code)))
            }
        } else {
            (false, Some("Command terminated by signal".to_string()))
        };

        let duration = start.elapsed();
        let stdout = std::str::from_utf8(&assert.get_output().stdout).unwrap_or("").to_string();
        let stderr = std::str::from_utf8(&assert.get_output().stderr).unwrap_or("").to_string();
        let output = format!("stdout: {}\nstderr: {}", stdout, stderr);

        let result = TestResult {
            name: name.to_string(),
            passed,
            duration,
            output,
            error: error_msg,
        };

        self.results.push(result);
        Ok(())
    }

    pub fn get_results(&self) -> &[TestResult] {
        &self.results
    }

    pub fn summary(&self) -> TestSummary {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration = self.results.iter().map(|r| r.duration).sum();

        TestSummary {
            total,
            passed,
            failed,
            total_duration,
        }
    }
}

pub struct CiCdPipeline {
    integration_tester: IntegrationTester,
    performance_tester: PerformanceTester,
    cli_tester: CliTester,
    analytics: AnalyticsDashboard,
}

impl Default for CiCdPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl CiCdPipeline {
    pub fn new() -> Self {
        Self {
            integration_tester: IntegrationTester::new(),
            performance_tester: PerformanceTester::new(),
            cli_tester: CliTester::new(),
            analytics: AnalyticsDashboard::new(),
        }
    }

    pub async fn run_full_pipeline(&mut self) -> Result<PipelineResult> {
        // Define test commands
        let test_commands = vec![
            ("build", "cargo build --release", 0),
            ("unit_tests", "cargo test", 0),
            ("integration_tests", "cargo test --test integration", 0),
        ];

        // Run integration tests sequentially for accurate timing
        let executor = ConcurrentExecutor::new(1); // Sequential execution for timing

        // Process results and add to integration tester
        for (name, cmd, expected_code) in test_commands {
            let start = std::time::Instant::now();
            let result = executor.execute(cmd).await;
            let duration = start.elapsed();

            let (passed, output, error) = match result {
                Ok(output) => {
                    // Command succeeded (exit code 0)
                    let passed = expected_code == 0;
                    let error = if passed { None } else { Some(format!("Expected exit code {}, got 0", expected_code)) };
                    (passed, output, error)
                }
                Err(ParallelError::CommandFailed { command: _, exit_code, stderr }) => {
                    // Command failed with specific exit code
                    let passed = exit_code == Some(expected_code);
                    let error = if passed { None } else { Some(format!("Expected exit code {}, got {:?}", expected_code, exit_code)) };
                    (passed, stderr.clone(), error)
                }
                Err(e) => {
                    // Other execution error
                    (false, String::new(), Some(format!("Command execution failed: {}", e)))
                }
            };

            let test_result = TestResult {
                name: name.to_string(),
                passed,
                duration,
                output: output.clone(),
                error: error.clone(),
            };
            self.integration_tester.test_results.push(test_result);

            // Record analytics data
            let point = DataPoint {
                timestamp: Utc::now(),
                metric: "test_execution_time".to_string(),
                value: duration.as_millis() as f64,
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("test_name".to_string(), name.to_string());
                    tags.insert("test_type".to_string(), "integration".to_string());
                    tags.insert("passed".to_string(), passed.to_string());
                    tags
                },
            };
            self.analytics.collection().record_data_point(point);
        }

        // Performance tests
        self.performance_tester.run_performance_test(
            "basic_operation",
            100,
            Some(std::time::Duration::from_millis(10)),
            || async {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            },
        ).await?;

        // Record performance analytics
        for perf_result in self.performance_tester.get_results() {
            let point = DataPoint {
                timestamp: Utc::now(),
                metric: "performance_test_avg_duration".to_string(),
                value: perf_result.avg_duration.as_millis() as f64,
                tags: {
                    let mut tags = HashMap::new();
                    tags.insert("test_name".to_string(), perf_result.name.clone());
                    tags.insert("iterations".to_string(), perf_result.iterations.to_string());
                    tags.insert("passed".to_string(), perf_result.passed.to_string());
                    if let Some(threshold) = perf_result.threshold {
                        tags.insert("threshold_ms".to_string(), threshold.as_millis().to_string());
                    }
                    tags
                },
            };
            self.analytics.collection().record_data_point(point);
        }

        let integration_summary = self.integration_tester.summary();
        let performance_results = self.performance_tester.get_results().to_vec();

        let passed = integration_summary.failed == 0 && performance_results.iter().all(|r| r.passed);

        // Generate analytics report
        let analytics_report = self.analytics.generate_dashboard_report(vec![
            "test_execution_time".to_string(),
            "performance_test_avg_duration".to_string(),
        ]);

        Ok(PipelineResult {
            passed,
            integration_summary,
            performance_results,
            analytics_report: Some(analytics_report),
        })
    }

    pub fn integration_tester(&mut self) -> &mut IntegrationTester {
        &mut self.integration_tester
    }

    pub fn performance_tester(&mut self) -> &mut PerformanceTester {
        &mut self.performance_tester
    }

    pub fn cli_tester(&mut self) -> &mut CliTester {
        &mut self.cli_tester
    }

    pub fn analytics(&self) -> &AnalyticsDashboard {
        &self.analytics
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub passed: bool,
    pub integration_summary: TestSummary,
    pub performance_results: Vec<PerformanceTestResult>,
    pub analytics_report: Option<durable_run_analytics::PerformanceReport>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_result_serialization() {
        let result = TestResult {
            name: "test".to_string(),
            passed: true,
            duration: std::time::Duration::from_secs(1),
            output: "output".to_string(),
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, result.name);
        assert_eq!(deserialized.passed, result.passed);
    }

    #[test]
    fn test_test_summary_calculation() {
        let summary = TestSummary {
            total: 5,
            passed: 3,
            failed: 2,
            total_duration: std::time::Duration::from_secs(10),
        };

        assert_eq!(summary.total, 5);
        assert_eq!(summary.passed, 3);
        assert_eq!(summary.failed, 2);
    }

    #[test]
    fn test_performance_test_result_serialization() {
        let result = PerformanceTestResult {
            name: "perf_test".to_string(),
            avg_duration: std::time::Duration::from_millis(100),
            min_duration: std::time::Duration::from_millis(90),
            max_duration: std::time::Duration::from_millis(110),
            iterations: 10,
            threshold: Some(std::time::Duration::from_millis(150)),
            passed: true,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: PerformanceTestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, result.name);
        assert_eq!(deserialized.passed, result.passed);
    }
}
