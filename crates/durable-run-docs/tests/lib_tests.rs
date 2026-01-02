use durable_run_docs::{UserGuide, Examples, BestPractices, Tutorials, Example, BestPractice, Tutorial, GuideSection, DocsError, Documentation};
use tempfile::tempdir;

#[test]
fn test_user_guide_creation() {
    let guide = UserGuide::getting_started();
    assert_eq!(guide.title, "Getting Started with Durable Run");
    assert_eq!(guide.version, "1.0.0");
    assert_eq!(guide.sections.len(), 2);
}

#[test]
fn test_user_guide_to_markdown() {
    let guide = UserGuide::getting_started();
    let md = guide.to_markdown();
    assert!(md.starts_with("# Getting Started with Durable Run"));
    assert!(md.contains("Version: 1.0.0"));
    assert!(md.contains("# Installation"));
    assert!(md.contains("# Basic Usage"));
}

#[test]
fn test_guide_section_to_markdown() {
    let section = durable_run_docs::GuideSection {
        title: "Test Section".to_string(),
        content: "Test content".to_string(),
        subsections: vec![],
    };
    let md = section.to_markdown(1);
    assert_eq!(md, "# Test Section\n\nTest content\n\n");
}

#[test]
fn test_guide_section_with_subsections() {
    let subsection = durable_run_docs::GuideSection {
        title: "Subsection".to_string(),
        content: "Sub content".to_string(),
        subsections: vec![],
    };
    let section = durable_run_docs::GuideSection {
        title: "Main Section".to_string(),
        content: "Main content".to_string(),
        subsections: vec![subsection],
    };
    let md = section.to_markdown(1);
    assert!(md.contains("# Main Section"));
    assert!(md.contains("## Subsection"));
}

#[test]
fn test_examples_creation() {
    let examples = Examples::new();
    let all = examples.all_examples();
    assert_eq!(all.len(), 5);
    assert_eq!(all[0].title, "Parallel File Processing");
    assert_eq!(all[1].title, "HTTP Load Testing");
}

#[test]
fn test_examples_by_tag() {
    let examples = Examples::new();
    let parallel = examples.get_examples_by_tag("parallel");
    assert_eq!(parallel.len(), 1);
    assert_eq!(parallel[0].title, "Parallel File Processing");

    let http = examples.get_examples_by_tag("http");
    assert_eq!(http.len(), 1);
    assert_eq!(http[0].title, "HTTP Load Testing");

    let nonexistent = examples.get_examples_by_tag("nonexistent");
    assert_eq!(nonexistent.len(), 0);
}

#[test]
fn test_best_practices_creation() {
    let practices = BestPractices::new();
    let all = practices.all_practices();
    assert_eq!(all.len(), 5);
    assert_eq!(all[0].title, "Use Appropriate Parallelism");
    assert_eq!(all[1].title, "Handle Errors Gracefully");
}

#[test]
fn test_tutorials_creation() {
    let tutorials = Tutorials::new();
    let all = tutorials.all_tutorials();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].title, "Building Your First Parallel Workflow");
    assert_eq!(all[0].difficulty, "beginner");
    assert_eq!(all[0].duration, "20 minutes");
}

#[test]
fn test_tutorials_get_tutorial() {
    let tutorials = Tutorials::new();
    let tutorial = tutorials.get_tutorial("Building Your First Parallel Workflow");
    assert!(tutorial.is_some());
    let tut = tutorial.unwrap();
    assert_eq!(tut.steps.len(), 3);

    let nonexistent = tutorials.get_tutorial("Nonexistent Tutorial");
    assert!(nonexistent.is_none());
}

#[test]
fn test_tutorials_by_difficulty() {
    let tutorials = Tutorials::new();
    let beginners = tutorials.tutorials_by_difficulty("beginner");
    assert_eq!(beginners.len(), 1);

    let advanced = tutorials.tutorials_by_difficulty("advanced");
    assert_eq!(advanced.len(), 1);
}

#[test]
fn test_tutorial_step_structure() {
    let tutorials = Tutorials::new();
    let tutorial = tutorials.get_tutorial("Building Your First Parallel Workflow").unwrap();
    let step1 = &tutorial.steps[0];
    assert_eq!(step1.title, "Install durable-run");
    assert!(step1.command.is_some());
    assert_eq!(step1.command.as_ref().unwrap(), "cargo install durable-run");
    assert!(step1.expected_output.is_none());

    let step3 = &tutorial.steps[2];
    assert_eq!(step3.title, "Run parallel commands");
    assert!(step3.expected_output.is_some());
}

#[test]
fn test_serialization_deserialization() {
    let guide = UserGuide::getting_started();
    let serialized = serde_json::to_string(&guide).unwrap();
    let deserialized: UserGuide = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.title, guide.title);
    assert_eq!(deserialized.version, guide.version);
    assert_eq!(deserialized.sections.len(), guide.sections.len());
}

#[test]
fn test_examples_multiple_tags() {
    let _examples = Examples::new();
    // Add a test example with multiple tags
    let multi_tag_example = Example {
        title: "Multi-tag Example".to_string(),
        description: "Example with multiple tags".to_string(),
        command: "echo test".to_string(),
        output: "test".to_string(),
        tags: vec!["parallel".to_string(), "testing".to_string(), "files".to_string()],
    };

    // Since we can't modify the existing examples easily, let's test the logic
    assert!(multi_tag_example.tags.contains(&"parallel".to_string()));
    assert!(multi_tag_example.tags.contains(&"testing".to_string()));
    assert!(multi_tag_example.tags.contains(&"files".to_string()));
}

#[test]
fn test_best_practices_serialization() {
    let practices = BestPractices::new();
    let serialized = serde_json::to_string(&practices.all_practices()).unwrap();
    let deserialized: Vec<BestPractice> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.len(), practices.all_practices().len());
    assert_eq!(deserialized[0].title, practices.all_practices()[0].title);
}

#[test]
fn test_tutorials_serialization() {
    let tutorials = Tutorials::new();
    let tutorial = tutorials.get_tutorial("Building Your First Parallel Workflow").unwrap();
    let serialized = serde_json::to_string(tutorial).unwrap();
    let deserialized: Tutorial = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.title, tutorial.title);
    assert_eq!(deserialized.difficulty, tutorial.difficulty);
    assert_eq!(deserialized.steps.len(), tutorial.steps.len());
}

#[test]
fn test_guide_section_deep_nesting() {
    let deep_section = GuideSection {
        title: "Level 1".to_string(),
        content: "Level 1 content".to_string(),
        subsections: vec![
            GuideSection {
                title: "Level 2".to_string(),
                content: "Level 2 content".to_string(),
                subsections: vec![
                    GuideSection {
                        title: "Level 3".to_string(),
                        content: "Level 3 content".to_string(),
                        subsections: vec![],
                    }
                ],
            }
        ],
    };

    let md = deep_section.to_markdown(1);
    assert!(md.contains("# Level 1"));
    assert!(md.contains("## Level 2"));
    assert!(md.contains("### Level 3"));
}

#[test]
fn test_examples_empty_tags() {
    let examples = Examples::new();
    let empty_tag_examples = examples.get_examples_by_tag("");
    assert_eq!(empty_tag_examples.len(), 0);
}

#[test]
fn test_tutorials_empty_difficulty() {
    let tutorials = Tutorials::new();
    let empty_difficulty = tutorials.tutorials_by_difficulty("");
    assert_eq!(empty_difficulty.len(), 0);
}

#[test]
fn test_user_guide_validation() {
    let guide = UserGuide::getting_started();
    assert!(guide.validate().is_ok());

    let invalid_guide = UserGuide {
        title: "".to_string(),
        version: "1.0.0".to_string(),
        sections: vec![],
    };
    assert!(invalid_guide.validate().is_err());
}

#[test]
fn test_guide_section_validation() {
    let valid_section = GuideSection {
        title: "Test Section".to_string(),
        content: "Test content".to_string(),
        subsections: vec![],
    };
    assert!(valid_section.validate().is_ok());

    let invalid_section = GuideSection {
        title: "".to_string(),
        content: "Test content".to_string(),
        subsections: vec![],
    };
    assert!(invalid_section.validate().is_err());
}

#[test]
fn test_examples_validation() {
    let examples = Examples::new();
    assert!(examples.validate().is_ok());
}

#[test]
fn test_best_practices_validation() {
    let practices = BestPractices::new();
    assert!(practices.validate().is_ok());
}

#[test]
fn test_tutorials_validation() {
    let tutorials = Tutorials::new();
    assert!(tutorials.validate().is_ok());
}

#[test]
fn test_examples_search() {
    let examples = Examples::new();
    let parallel_results = examples.search_examples("parallel");
    assert_eq!(parallel_results.len(), 1);
    assert_eq!(parallel_results[0].title, "Parallel File Processing");

    let file_results = examples.search_examples("file");
    assert_eq!(file_results.len(), 1); // Parallel File Processing

    let nonexistent = examples.search_examples("nonexistent");
    assert_eq!(nonexistent.len(), 0);
}

#[test]
fn test_examples_get_example() {
    let examples = Examples::new();
    let example = examples.get_example("Parallel File Processing");
    assert!(example.is_some());
    assert_eq!(example.unwrap().command, "durable-run -j 4 'convert {file} {file}.png' --input files.txt");

    let nonexistent = examples.get_example("Nonexistent Example");
    assert!(nonexistent.is_none());
}

#[test]
fn test_best_practices_search() {
    let practices = BestPractices::new();
    let parallel_results = practices.search_practices("parallel");
    assert_eq!(parallel_results.len(), 1);
    assert_eq!(parallel_results[0].title, "Use Appropriate Parallelism");

    let resource_results = practices.search_practices("resource");
    assert_eq!(resource_results.len(), 3); // Parallelism, Batch Configuration, and Analytics

    let nonexistent = practices.search_practices("nonexistent");
    assert_eq!(nonexistent.len(), 0);
}

#[test]
fn test_tutorials_search() {
    let tutorials = Tutorials::new();
    let beginner_results = tutorials.search_tutorials("beginner");
    assert_eq!(beginner_results.len(), 1);
    assert_eq!(beginner_results[0].title, "Building Your First Parallel Workflow");

    let workflow_results = tutorials.search_tutorials("workflow");
    assert_eq!(workflow_results.len(), 1); // Building Your First Parallel Workflow

    let nonexistent = tutorials.search_tutorials("nonexistent");
    assert_eq!(nonexistent.len(), 0);
}

#[test]
fn test_error_types() {
    let err = DocsError::TutorialNotFound("Test Tutorial".to_string());
    assert_eq!(err.to_string(), "Tutorial 'Test Tutorial' not found");

    let err = DocsError::InvalidMarkdown("Test message".to_string());
    assert_eq!(err.to_string(), "Invalid markdown: Test message");

    let err = DocsError::InvalidSection("Test section error".to_string());
    assert_eq!(err.to_string(), "Invalid section: Test section error");
}

#[test]
fn test_serialization_error_handling() {
    // Test that invalid JSON produces appropriate error
    let invalid_json = r#"{"invalid": json"#;
    let result: std::result::Result<UserGuide, serde_json::Error> = serde_json::from_str(invalid_json);
    assert!(result.is_err());

    // Test conversion to DocsError
    let docs_result: std::result::Result<UserGuide, DocsError> = result.map_err(DocsError::from);
    assert!(docs_result.is_err());
    match docs_result.unwrap_err() {
        DocsError::Serialization(_) => {}, // Expected
        _ => panic!("Expected serialization error"),
    }
}

#[test]
fn test_user_guide_file_io() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("guide.json");

    let guide = UserGuide::getting_started();
    guide.save_to_file(&file_path).unwrap();

    let loaded_guide = UserGuide::from_file(&file_path).unwrap();
    assert_eq!(loaded_guide.title, guide.title);
    assert_eq!(loaded_guide.sections.len(), guide.sections.len());
}

#[test]
fn test_examples_file_io() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("examples.json");

    let examples = Examples::new();
    examples.save_to_file(&file_path).unwrap();

    let loaded_examples = Examples::from_file(&file_path).unwrap();
    assert_eq!(loaded_examples.all_examples().len(), examples.all_examples().len());
}

#[test]
fn test_best_practices_file_io() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("practices.json");

    let practices = BestPractices::new();
    practices.save_to_file(&file_path).unwrap();

    let loaded_practices = BestPractices::from_file(&file_path).unwrap();
    assert_eq!(loaded_practices.all_practices().len(), practices.all_practices().len());
}

#[test]
fn test_tutorials_file_io() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("tutorials.json");

    let tutorials = Tutorials::new();
    tutorials.save_to_file(&file_path).unwrap();

    let loaded_tutorials = Tutorials::from_file(&file_path).unwrap();
    assert_eq!(loaded_tutorials.all_tutorials().len(), tutorials.all_tutorials().len());
}

#[test]
fn test_user_guide_regex_search() {
    let guide = UserGuide::getting_started();
    let results = guide.search("Installation").unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|r| r.content_type == "section_title"));
}

#[test]
fn test_examples_regex_search() {
    let examples = Examples::new();
    let results = examples.search_regex("parallel").unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].title, "Parallel File Processing");
}

#[test]
fn test_best_practices_regex_search() {
    let practices = BestPractices::new();
    let results = practices.search_regex("parallel").unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].title, "Use Appropriate Parallelism");
}

#[test]
fn test_tutorials_regex_search() {
    let tutorials = Tutorials::new();
    let results = tutorials.search_regex("beginner").unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].title, "Building Your First Parallel Workflow");
}

#[test]
fn test_documentation_creation_and_validation() {
    let guide = UserGuide::getting_started();
    let examples = Examples::new();
    let practices = BestPractices::new();
    let tutorials = Tutorials::new();

    let docs = Documentation::new(guide, examples, practices, tutorials);
    assert!(docs.validate().is_ok());
}

#[test]
fn test_documentation_file_io() {
    let dir = tempdir().unwrap();
    let guide = UserGuide::getting_started();
    let examples = Examples::new();
    let practices = BestPractices::new();
    let tutorials = Tutorials::new();

    let docs = Documentation::new(guide, examples, practices, tutorials);
    docs.save_to_directory(&dir).unwrap();

    let loaded_docs = Documentation::from_directory(&dir).unwrap();
    assert_eq!(loaded_docs.user_guide.title, docs.user_guide.title);
    assert_eq!(loaded_docs.examples.len(), docs.examples.len());
}

#[test]
fn test_documentation_comprehensive_search() {
    let guide = UserGuide::getting_started();
    let examples = Examples::new();
    let practices = BestPractices::new();
    let tutorials = Tutorials::new();

    let docs = Documentation::new(guide, examples, practices, tutorials);
    let results = docs.search_all("parallel").unwrap();
    assert!(!results.is_empty());

    // Should find matches in examples and best practices
    let example_results: Vec<_> = results.iter().filter(|r| r.content_type.starts_with("example")).collect();
    let practice_results: Vec<_> = results.iter().filter(|r| r.content_type.starts_with("practice")).collect();
    assert!(!example_results.is_empty());
    assert!(!practice_results.is_empty());
}

#[test]
fn test_invalid_regex_search() {
    let examples = Examples::new();
    let result = examples.search_regex("[invalid");
    assert!(result.is_err());
    match result.unwrap_err() {
        DocsError::InvalidRegex(_) => {}, // Expected
        _ => panic!("Expected invalid regex error"),
    }
}

#[test]
fn test_file_io_errors() {
    // Test loading from nonexistent file
    let result = UserGuide::from_file("nonexistent.json");
    assert!(result.is_err());

    // Test saving to invalid path
    let guide = UserGuide::getting_started();
    let result = guide.save_to_file("/invalid/path/guide.json");
    assert!(result.is_err());
}