use durable_run_parallel::execute_from_source;
use durable_run_input::{FileSource, InputTransformation};
use tempfile::NamedTempFile;
use std::io::Write;

#[tokio::test]
async fn test_execute_from_source_integration() {
    // Create a temporary file with test data
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "line1").unwrap();
    writeln!(temp_file, "line2").unwrap();
    writeln!(temp_file, "line3").unwrap();

    // Create input source
    let source = FileSource::new(temp_file.path().to_string_lossy().to_string());

    // Create transformation
    let transform = InputTransformation::new().with_limit(2);

    // Execute commands
    let results = execute_from_source(&source, transform, "echo {0}", 2).await;

    // Verify results
    assert_eq!(results.len(), 2);
    assert!(results[0].is_ok());
    assert!(results[1].is_ok());
    assert_eq!(results[0].as_ref().unwrap().trim(), "line1");
    assert_eq!(results[1].as_ref().unwrap().trim(), "line2");
}

#[tokio::test]
async fn test_execute_from_source_with_filter() {
    // Create a temporary file with test data
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "test1.txt").unwrap();
    writeln!(temp_file, "other").unwrap();
    writeln!(temp_file, "test2.txt").unwrap();

    // Create input source
    let source = FileSource::new(temp_file.path().to_string_lossy().to_string());

    // Create transformation with filter
    let transform = InputTransformation::new().with_filter("test".to_string());

    // Execute commands
    let results = execute_from_source(&source, transform, "echo 'processing {0}'", 2).await;

    // Verify results - should only process lines containing "test"
    assert_eq!(results.len(), 2);
    assert!(results[0].is_ok());
    assert!(results[1].is_ok());
    assert!(results[0].as_ref().unwrap().contains("test1.txt"));
    assert!(results[1].as_ref().unwrap().contains("test2.txt"));
}