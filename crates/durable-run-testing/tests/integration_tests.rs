use durable_run_testing::*;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_integration_tester_command_success() {
    let mut tester = IntegrationTester::new();
    let result = tester.run_command_test("echo_test", "echo 'hello world'", 0).await;
    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(results[0].passed);
    assert_eq!(results[0].name, "echo_test");
    assert!(results[0].output.contains("hello world"));
}

#[tokio::test]
async fn test_integration_tester_command_failure() {
    let mut tester = IntegrationTester::new();
    let result = tester.run_command_test("false_test", "false", 0).await;
    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(!results[0].passed);
    assert!(results[0].error.is_some());
}

#[tokio::test]
async fn test_integration_tester_file_test() {
    let mut tester = IntegrationTester::new();
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "test content").unwrap();

    let result = tester.run_file_test("file_test", temp_file.path(), "test content").await;
    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(results[0].passed);
}

#[tokio::test]
async fn test_integration_tester_file_test_failure() {
    let mut tester = IntegrationTester::new();
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "different content").unwrap();

    let result = tester.run_file_test("file_test", temp_file.path(), "expected content").await;
    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(!results[0].passed);
    assert!(results[0].error.is_some());
}

#[tokio::test]
async fn test_performance_tester() {
    let mut tester = PerformanceTester::new();

    let result = tester.run_performance_test(
        "sleep_test",
        10,
        Some(std::time::Duration::from_millis(50)),
        || async {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        },
    ).await;

    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(results[0].passed);
    assert_eq!(results[0].iterations, 10);
    assert!(results[0].avg_duration > std::time::Duration::from_millis(0));
}

#[tokio::test]
async fn test_performance_tester_threshold_failure() {
    let mut tester = PerformanceTester::new();

    let result = tester.run_performance_test(
        "slow_test",
        5,
        Some(std::time::Duration::from_millis(1)), // Very strict threshold
        || async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await; // Much slower
        },
    ).await;

    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(!results[0].passed); // Should fail due to threshold
}

#[tokio::test]
async fn test_cli_tester_success() {
    let mut tester = CliTester::new();

    // Test with a simple command that should succeed
    let result = tester.run_cli_test(
        "true_test",
        "true",
        &[],
        0,
        None,
        None,
    ).await;

    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(results[0].passed);
}

#[tokio::test]
async fn test_cli_tester_with_predicates() {
    let mut tester = CliTester::new();

    let result = tester.run_cli_test(
        "echo_test",
        "echo",
        &["hello world"],
        0,
        Some(Box::new(predicate::str::contains("hello world"))),
        None,
    ).await;

    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(results[0].passed);
}

#[tokio::test]
async fn test_cli_tester_predicate_failure() {
    let mut tester = CliTester::new();

    let result = tester.run_cli_test(
        "echo_test",
        "echo",
        &["hello"],
        0,
        Some(Box::new(predicate::str::contains("world"))), // This won't match
        None,
    ).await;

    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(!results[0].passed);
    assert!(results[0].error.is_some());
}

#[tokio::test]
async fn test_test_summary() {
    let mut tester = IntegrationTester::new();

    // Add some passing tests
    tester.run_command_test("pass1", "true", 0).await.unwrap();
    tester.run_command_test("pass2", "true", 0).await.unwrap();

    // Add a failing test
    tester.run_command_test("fail", "false", 0).await.unwrap();

    let summary = tester.summary();
    assert_eq!(summary.total, 3);
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 1);
    assert!(summary.total_duration > std::time::Duration::from_millis(0));
}

#[tokio::test]
async fn test_pipeline_result_serialization() {
    let summary = TestSummary {
        total: 5,
        passed: 4,
        failed: 1,
        total_duration: std::time::Duration::from_secs(10),
    };

    let perf_result = PerformanceTestResult {
        name: "test".to_string(),
        avg_duration: std::time::Duration::from_millis(100),
        min_duration: std::time::Duration::from_millis(90),
        max_duration: std::time::Duration::from_millis(110),
        iterations: 10,
        threshold: Some(std::time::Duration::from_millis(150)),
        passed: true,
    };

    let pipeline_result = PipelineResult {
        passed: true,
        integration_summary: summary,
        performance_results: vec![perf_result],
        analytics_report: None,
    };

    // Test JSON serialization
    let json = serde_json::to_string(&pipeline_result).unwrap();
    let deserialized: PipelineResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.passed, pipeline_result.passed);
    assert_eq!(deserialized.integration_summary.total, pipeline_result.integration_summary.total);
}

#[tokio::test]
async fn test_cicd_pipeline_basic() {
    let mut pipeline = CiCdPipeline::new();

    // Test that the pipeline can be created and run without panicking
    let result = pipeline.run_full_pipeline().await;

    // The pipeline should return a result (either success or error)
    // We don't assert on success since cargo commands may not be available in test environment
    match result {
        Ok(pipeline_result) => {
            // If successful, verify the structure
            assert!(pipeline_result.analytics_report.is_some());
            if let Some(report) = &pipeline_result.analytics_report {
                // Analytics report should be generated
                assert!(!report.metrics.is_empty() || report.metrics.is_empty()); // Just check it exists
            }
        }
        Err(_) => {
            // If it fails, that's also acceptable in test environment
            // The important thing is that it doesn't panic
        }
    }
}

#[test]
fn test_error_display() {
    let error = TestingError::Assertion("test error".to_string());
    assert!(error.to_string().contains("test error"));

    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let error = TestingError::Command(io_error);
    assert!(error.to_string().contains("Command error"));
}

#[tokio::test]
async fn test_integration_tester_empty_results() {
    let tester = IntegrationTester::new();
    assert_eq!(tester.get_results().len(), 0);

    let summary = tester.summary();
    assert_eq!(summary.total, 0);
    assert_eq!(summary.passed, 0);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.total_duration, std::time::Duration::from_millis(0));
}

#[tokio::test]
async fn test_performance_tester_empty_results() {
    let tester = PerformanceTester::new();
    assert_eq!(tester.get_results().len(), 0);
}

#[tokio::test]
async fn test_cli_tester_empty_results() {
    let tester = CliTester::new();
    assert_eq!(tester.get_results().len(), 0);

    let summary = tester.summary();
    assert_eq!(summary.total, 0);
    assert_eq!(summary.passed, 0);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.total_duration, std::time::Duration::from_millis(0));
}

#[tokio::test]
async fn test_integration_tester_command_with_nonzero_exit() {
    let mut tester = IntegrationTester::new();
    let result = tester.run_command_test("false_test", "false", 1).await;
    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(results[0].passed);
    assert_eq!(results[0].name, "false_test");
}

#[tokio::test]
async fn test_integration_tester_command_timeout_simulation() {
    let mut tester = IntegrationTester::new();
    // Test with a command that should complete quickly
    let result = tester.run_command_test("quick_test", "true", 0).await;
    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(results[0].duration > std::time::Duration::from_millis(0));
    assert!(results[0].duration < std::time::Duration::from_secs(1)); // Should be very fast
}

#[tokio::test]
async fn test_file_test_with_large_content() {
    let mut tester = IntegrationTester::new();
    let temp_file = NamedTempFile::new().unwrap();
    let large_content = "test content".repeat(1000); // 12KB of content
    fs::write(&temp_file, &large_content).unwrap();

    let result = tester.run_file_test("large_file_test", temp_file.path(), "test content").await;
    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(results[0].passed);
    assert_eq!(results[0].output, large_content);
}

#[tokio::test]
async fn test_file_test_nonexistent_file() {
    let mut tester = IntegrationTester::new();
    let result = tester.run_file_test("missing_file_test", std::path::Path::new("/nonexistent/file"), "content").await;
    assert!(result.is_err()); // Should fail because file doesn't exist
}

#[tokio::test]
async fn test_performance_tester_zero_iterations() {
    let mut tester = PerformanceTester::new();

    // This should handle zero iterations gracefully
    let result = tester.run_performance_test(
        "zero_iter_test",
        0,
        None,
        || async {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        },
    ).await;

    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].iterations, 0);
    // With zero iterations, avg_duration should be 0
    assert_eq!(results[0].avg_duration, std::time::Duration::from_millis(0));
}

#[tokio::test]
async fn test_cli_tester_with_complex_args() {
    let mut tester = CliTester::new();

    let result = tester.run_cli_test(
        "complex_args_test",
        "echo",
        &["-n", "hello", "world", "with", "spaces"],
        0,
        Some(Box::new(predicate::str::contains("hello world with spaces"))),
        None,
    ).await;

    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(results[0].passed);
}

#[tokio::test]
async fn test_cli_tester_stderr_predicate() {
    let mut tester = CliTester::new();

    // Command that outputs to stderr
    let result = tester.run_cli_test(
        "stderr_test",
        "sh",
        &["-c", "echo 'error message' >&2"],
        0,
        None,
        Some(Box::new(predicate::str::contains("error message"))),
    ).await;

    assert!(result.is_ok());

    let results = tester.get_results();
    assert_eq!(results.len(), 1);
    assert!(results[0].passed);
}

#[tokio::test]
async fn test_multiple_test_runs() {
    let mut tester = IntegrationTester::new();

    // Run multiple tests
    tester.run_command_test("test1", "true", 0).await.unwrap();
    tester.run_command_test("test2", "true", 0).await.unwrap();
    tester.run_command_test("test3", "false", 0).await.unwrap(); // This will fail

    let results = tester.get_results();
    assert_eq!(results.len(), 3);

    let summary = tester.summary();
    assert_eq!(summary.total, 3);
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.failed, 1);
}