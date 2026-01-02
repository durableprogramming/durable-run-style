//! # Durable Run Documentation
//!
//! A comprehensive documentation system for Durable Run, providing user guides,
//! examples, best practices, and tutorials.
//!
//! This crate provides:
//! - User guides with installation and usage instructions
//! - Curated examples for common use cases
//! - Best practices for efficient and reliable execution
//! - Step-by-step tutorials for different skill levels
//!
//! ## Features
//!
//! - **User Guide**: Structured documentation with sections and subsections
//! - **Examples**: Tagged examples for easy discovery and filtering
//! - **Best Practices**: Do's and don'ts with reasoning
//! - **Tutorials**: Guided learning paths with prerequisites and objectives
//! - **Search**: Full-text search across all documentation content
//! - **Validation**: Structural validation of documentation content
//! - **Serialization**: JSON support for documentation interchange
//!
//! ## Example
//!
//! ```rust
//! use durable_run_docs::{UserGuide, Examples, BestPractices, Tutorials};
//!
//! // Get the getting started guide
//! let guide = UserGuide::getting_started();
//! println!("{}", guide.to_markdown());
//!
//! // Find examples by tag or search
//! let examples = Examples::new();
//! let parallel_examples = examples.get_examples_by_tag("parallel");
//! let file_examples = examples.search_examples("file");
//!
//! // Get best practices
//! let practices = BestPractices::new();
//! for practice in practices.all_practices() {
//!     println!("{}: {}", practice.title, practice.description);
//! }
//!
//! // Access tutorials
//! let tutorials = Tutorials::new();
//! let beginner_tutorials = tutorials.tutorials_by_difficulty("beginner");
//! let workflow_tutorials = tutorials.search_tutorials("workflow");
//!
//! // Validate documentation
//! guide.validate()?;
//! examples.validate()?;
//! practices.validate()?;
//! tutorials.validate()?;
//! # Ok::<(), durable_run_docs::DocsError>(())
//! ```

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur in the documentation system
#[derive(Debug, Error)]
pub enum DocsError {
    /// Invalid markdown format
    #[error("Invalid markdown: {0}")]
    InvalidMarkdown(String),
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Tutorial not found
    #[error("Tutorial '{0}' not found")]
    TutorialNotFound(String),
    /// Example not found
    #[error("Example '{0}' not found")]
    ExampleNotFound(String),
    /// Invalid section structure
    #[error("Invalid section: {0}")]
    InvalidSection(String),
    /// Invalid regex pattern
    #[error("Invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),
}

/// Result type alias for documentation operations
pub type Result<T> = std::result::Result<T, DocsError>;

/// Search result containing matched content and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub content_type: String,
    pub path: Vec<String>,
    pub matched_text: String,
    pub context: String,
}

/// Combined documentation structure for unified operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    pub user_guide: UserGuide,
    pub examples: Vec<Example>,
    pub best_practices: Vec<BestPractice>,
    pub tutorials: Vec<Tutorial>,
}

impl Documentation {
    /// Create documentation from individual components
    pub fn new(user_guide: UserGuide, examples: Examples, best_practices: BestPractices, tutorials: Tutorials) -> Self {
        Self {
            user_guide,
            examples: examples.examples,
            best_practices: best_practices.practices,
            tutorials: tutorials.tutorials,
        }
    }

    /// Load documentation from a directory containing JSON files
    pub fn from_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        let user_guide = UserGuide::from_file(dir.join("user_guide.json"))?;
        let examples = Examples::from_file(dir.join("examples.json"))?;
        let best_practices = BestPractices::from_file(dir.join("best_practices.json"))?;
        let tutorials = Tutorials::from_file(dir.join("tutorials.json"))?;

        Ok(Self::new(user_guide, examples, best_practices, tutorials))
    }

    /// Save documentation to a directory as JSON files
    pub fn save_to_directory<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir)?;

        self.user_guide.save_to_file(dir.join("user_guide.json"))?;
        Examples { examples: self.examples.clone() }.save_to_file(dir.join("examples.json"))?;
        BestPractices { practices: self.best_practices.clone() }.save_to_file(dir.join("best_practices.json"))?;
        Tutorials { tutorials: self.tutorials.clone() }.save_to_file(dir.join("tutorials.json"))?;

        Ok(())
    }

    /// Comprehensive search across all documentation
    pub fn search_all(&self, pattern: &str) -> Result<Vec<SearchResult>> {
        let regex = Regex::new(pattern)?;
        let mut results = Vec::new();

        // Search user guide
        self.user_guide.search_recursive(&regex, &mut results, vec!["user_guide".to_string()]);

        // Search examples
        for (i, example) in self.examples.iter().enumerate() {
            if regex.is_match(&example.title) {
                results.push(SearchResult {
                    content_type: "example_title".to_string(),
                    path: vec!["examples".to_string(), i.to_string()],
                    matched_text: example.title.clone(),
                    context: "Example title".to_string(),
                });
            }
            if regex.is_match(&example.description) {
                results.push(SearchResult {
                    content_type: "example_description".to_string(),
                    path: vec!["examples".to_string(), i.to_string()],
                    matched_text: example.description.clone(),
                    context: format!("Example: {}", example.title),
                });
            }
            if regex.is_match(&example.command) {
                results.push(SearchResult {
                    content_type: "example_command".to_string(),
                    path: vec!["examples".to_string(), i.to_string()],
                    matched_text: example.command.clone(),
                    context: format!("Example command: {}", example.title),
                });
            }
        }

        // Search best practices
        for (i, practice) in self.best_practices.iter().enumerate() {
            if regex.is_match(&practice.title) {
                results.push(SearchResult {
                    content_type: "practice_title".to_string(),
                    path: vec!["best_practices".to_string(), i.to_string()],
                    matched_text: practice.title.clone(),
                    context: "Best practice title".to_string(),
                });
            }
            if regex.is_match(&practice.description) {
                results.push(SearchResult {
                    content_type: "practice_description".to_string(),
                    path: vec!["best_practices".to_string(), i.to_string()],
                    matched_text: practice.description.clone(),
                    context: format!("Best practice: {}", practice.title),
                });
            }
        }

        // Search tutorials
        for (i, tutorial) in self.tutorials.iter().enumerate() {
            if regex.is_match(&tutorial.title) {
                results.push(SearchResult {
                    content_type: "tutorial_title".to_string(),
                    path: vec!["tutorials".to_string(), i.to_string()],
                    matched_text: tutorial.title.clone(),
                    context: "Tutorial title".to_string(),
                });
            }
            for (j, step) in tutorial.steps.iter().enumerate() {
                if regex.is_match(&step.title) {
                    results.push(SearchResult {
                        content_type: "tutorial_step_title".to_string(),
                        path: vec!["tutorials".to_string(), i.to_string(), "steps".to_string(), j.to_string()],
                        matched_text: step.title.clone(),
                        context: format!("Tutorial step: {} > {}", tutorial.title, step.title),
                    });
                }
                if regex.is_match(&step.content) {
                    results.push(SearchResult {
                        content_type: "tutorial_step_content".to_string(),
                        path: vec!["tutorials".to_string(), i.to_string(), "steps".to_string(), j.to_string()],
                        matched_text: step.content.clone(),
                        context: format!("Tutorial step content: {} > {}", tutorial.title, step.title),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Validate all documentation components
    pub fn validate(&self) -> Result<()> {
        self.user_guide.validate()?;
        Examples { examples: self.examples.clone() }.validate()?;
        BestPractices { practices: self.best_practices.clone() }.validate()?;
        Tutorials { tutorials: self.tutorials.clone() }.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGuide {
    pub title: String,
    pub sections: Vec<GuideSection>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideSection {
    pub title: String,
    pub content: String,
    pub subsections: Vec<GuideSection>,
}

impl UserGuide {
    pub fn getting_started() -> Self {
        Self {
            title: "Getting Started with Durable Run".to_string(),
            version: "1.0.0".to_string(),
            sections: vec![
                GuideSection {
                    title: "Installation".to_string(),
                    content: r#"
To install Durable Run, use Cargo:

```bash
cargo install durable-run
```

Or build from source:

```bash
git clone https://github.com/your-org/durable-run.git
cd durable-run
cargo build --release
```
                    "#.to_string(),
                    subsections: vec![],
                },
                GuideSection {
                    title: "Basic Usage".to_string(),
                    content: r#"
Run commands in parallel:

```bash
durable-run -j 4 'echo hello {1}' : 1..10
```

Use templates:

```bash
durable-run --template 'ping {host}' --input hosts.txt
```
                    "#.to_string(),
                    subsections: vec![],
                },
            ],
        }
    }

    /// Load user guide from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let guide: UserGuide = serde_json::from_str(&content)?;
        Ok(guide)
    }

    /// Save user guide to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Search the guide content using regex
    pub fn search(&self, pattern: &str) -> Result<Vec<SearchResult>> {
        let regex = Regex::new(pattern)?;
        let mut results = Vec::new();

        self.search_recursive(&regex, &mut results, Vec::new());
        Ok(results)
    }

    fn search_recursive(&self, regex: &Regex, results: &mut Vec<SearchResult>, path: Vec<String>) {
        // Search in title
        if regex.is_match(&self.title) {
            results.push(SearchResult {
                content_type: "guide_title".to_string(),
                path: path.clone(),
                matched_text: self.title.clone(),
                context: "Guide title".to_string(),
            });
        }

        // Search in sections
        for (i, section) in self.sections.iter().enumerate() {
            let mut section_path = path.clone();
            section_path.push(format!("section[{}]", i));
            section.search_recursive(regex, results, section_path);
        }
    }

    /// Validate the user guide structure
    pub fn validate(&self) -> Result<()> {
        if self.title.trim().is_empty() {
            return Err(DocsError::InvalidSection("Guide title cannot be empty".to_string()));
        }
        if self.sections.is_empty() {
            return Err(DocsError::InvalidSection("Guide must have at least one section".to_string()));
        }
        for section in &self.sections {
            section.validate()?;
        }
        Ok(())
    }

    pub fn to_markdown(&self) -> String {
        let mut md = format!("# {}\n\nVersion: {}\n\n", self.title, self.version);
        for section in &self.sections {
            md.push_str(&section.to_markdown(1));
        }
        md
    }
}

impl GuideSection {
    /// Validate the section structure
    pub fn validate(&self) -> Result<()> {
        if self.title.trim().is_empty() {
            return Err(DocsError::InvalidSection("Section title cannot be empty".to_string()));
        }
        if self.content.trim().is_empty() {
            return Err(DocsError::InvalidSection("Section content cannot be empty".to_string()));
        }
        for subsection in &self.subsections {
            subsection.validate()?;
        }
        Ok(())
    }

    pub fn to_markdown(&self, level: usize) -> String {
        let mut md = format!("{} {}\n\n{}\n\n", "#".repeat(level), self.title, self.content.trim());
        for subsection in &self.subsections {
            md.push_str(&subsection.to_markdown(level + 1));
        }
        md
    }

    fn search_recursive(&self, regex: &Regex, results: &mut Vec<SearchResult>, path: Vec<String>) {
        // Search in title
        if regex.is_match(&self.title) {
            results.push(SearchResult {
                content_type: "section_title".to_string(),
                path: path.clone(),
                matched_text: self.title.clone(),
                context: format!("Section: {}", path.join(" > ")),
            });
        }

        // Search in content
        if regex.is_match(&self.content) {
            results.push(SearchResult {
                content_type: "section_content".to_string(),
                path: path.clone(),
                matched_text: self.content.clone(),
                context: format!("Section content: {}", self.title),
            });
        }

        // Search in subsections
        for (i, subsection) in self.subsections.iter().enumerate() {
            let mut subsection_path = path.clone();
            subsection_path.push(format!("subsection[{}]", i));
            subsection.search_recursive(regex, results, subsection_path);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub title: String,
    pub description: String,
    pub command: String,
    pub output: String,
    pub tags: Vec<String>,
}

pub struct Examples {
    examples: Vec<Example>,
}

impl Default for Examples {
    fn default() -> Self {
        Self::new()
    }
}

impl Examples {
    pub fn new() -> Self {
        Self {
            examples: vec![
                Example {
                    title: "Parallel File Processing".to_string(),
                    description: "Process multiple files in parallel".to_string(),
                    command: "durable-run -j 4 'convert {file} {file}.png' --input files.txt".to_string(),
                    output: "Processes 4 files concurrently, converting each to PNG format.".to_string(),
                    tags: vec!["parallel".to_string(), "files".to_string()],
                },
                Example {
                    title: "HTTP Load Testing".to_string(),
                    description: "Test multiple endpoints simultaneously".to_string(),
                    command: "durable-run -j 10 'curl -s {url}' --input urls.txt".to_string(),
                    output: "Makes 10 concurrent HTTP requests to different URLs.".to_string(),
                    tags: vec!["http".to_string(), "testing".to_string()],
                },
                Example {
                    title: "Batch Configuration".to_string(),
                    description: "Use batch configuration for advanced job control".to_string(),
                    command: "durable-run --config batch.yaml 'process {file}' --input files.txt".to_string(),
                    output: "Executes commands with custom limits, timeouts, and resource controls.".to_string(),
                    tags: vec!["config".to_string(), "batch".to_string(), "advanced".to_string()],
                },
                Example {
                    title: "Analytics and Monitoring".to_string(),
                    description: "Collect performance metrics during execution".to_string(),
                    command: "durable-run --analytics 'heavy-computation {input}' --input data.txt".to_string(),
                    output: "Runs commands while collecting timing and resource usage statistics.".to_string(),
                    tags: vec!["analytics".to_string(), "monitoring".to_string(), "performance".to_string()],
                },
                Example {
                    title: "Workflow Orchestration".to_string(),
                    description: "Execute tasks with dependencies".to_string(),
                    command: "durable-run --workflow workflow.yaml".to_string(),
                    output: "Runs a series of dependent tasks in the correct order.".to_string(),
                    tags: vec!["workflow".to_string(), "dependencies".to_string(), "orchestration".to_string()],
                },
            ],
        }
    }

    pub fn get_examples_by_tag(&self, tag: &str) -> Vec<&Example> {
        self.examples.iter().filter(|e| e.tags.iter().any(|t| t == tag)).collect()
    }

    pub fn all_examples(&self) -> &[Example] {
        &self.examples
    }

    /// Search examples by keyword in title, description, or tags
    pub fn search_examples(&self, keyword: &str) -> Vec<&Example> {
        let keyword_lower = keyword.to_lowercase();
        self.examples.iter().filter(|e|
            e.title.to_lowercase().contains(&keyword_lower) ||
            e.description.to_lowercase().contains(&keyword_lower) ||
            e.tags.iter().any(|tag| tag.to_lowercase().contains(&keyword_lower))
        ).collect()
    }

    /// Get example by exact title
    pub fn get_example(&self, title: &str) -> Option<&Example> {
        self.examples.iter().find(|e| e.title == title)
    }

    /// Load examples from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let examples: Vec<Example> = serde_json::from_str(&content)?;
        Ok(Self { examples })
    }

    /// Save examples to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.examples)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Search examples using regex across all fields
    pub fn search_regex(&self, pattern: &str) -> Result<Vec<&Example>> {
        let regex = Regex::new(pattern)?;
        Ok(self.examples.iter().filter(|e|
            regex.is_match(&e.title) ||
            regex.is_match(&e.description) ||
            regex.is_match(&e.command) ||
            regex.is_match(&e.output) ||
            e.tags.iter().any(|tag| regex.is_match(tag))
        ).collect())
    }

    /// Validate all examples
    pub fn validate(&self) -> Result<()> {
        for example in &self.examples {
            if example.title.trim().is_empty() {
                return Err(DocsError::InvalidSection("Example title cannot be empty".to_string()));
            }
            if example.description.trim().is_empty() {
                return Err(DocsError::InvalidSection("Example description cannot be empty".to_string()));
            }
            if example.command.trim().is_empty() {
                return Err(DocsError::InvalidSection("Example command cannot be empty".to_string()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestPractice {
    pub title: String,
    pub description: String,
    pub do_example: String,
    pub dont_example: String,
    pub reasoning: String,
}

pub struct BestPractices {
    practices: Vec<BestPractice>,
}

impl Default for BestPractices {
    fn default() -> Self {
        Self::new()
    }
}

impl BestPractices {
    pub fn new() -> Self {
        Self {
            practices: vec![
                BestPractice {
                    title: "Use Appropriate Parallelism".to_string(),
                    description: "Choose the right number of parallel jobs for your workload".to_string(),
                    do_example: "durable-run -j $(nproc) 'process {file}' --input files.txt".to_string(),
                    dont_example: "durable-run -j 1000 'process {file}' --input files.txt".to_string(),
                    reasoning: "Too many jobs can overwhelm system resources, too few may not utilize available CPU cores effectively.".to_string(),
                },
                BestPractice {
                    title: "Handle Errors Gracefully".to_string(),
                    description: "Design commands to handle failures appropriately".to_string(),
                    do_example: "durable-run 'process {file} || echo \"Failed: {file}\" >&2' --input files.txt".to_string(),
                    dont_example: "durable-run 'process {file}' --input files.txt".to_string(),
                    reasoning: "Silent failures can make debugging difficult. Always log or handle errors.".to_string(),
                },
                BestPractice {
                    title: "Use Batch Configuration for Complex Jobs".to_string(),
                    description: "Leverage batch configuration for resource management".to_string(),
                    do_example: "durable-run --config batch.yaml -j 8 'intensive-task {input}' --input inputs.txt".to_string(),
                    dont_example: "durable-run -j 100 'intensive-task {input}' --input inputs.txt".to_string(),
                    reasoning: "Batch configs provide fine-grained control over resources, timeouts, and limits.".to_string(),
                },
                BestPractice {
                    title: "Monitor Performance with Analytics".to_string(),
                    description: "Collect metrics to optimize execution".to_string(),
                    do_example: "durable-run --analytics --output metrics.json 'workload {data}' --input dataset.txt".to_string(),
                    dont_example: "durable-run 'workload {data}' --input dataset.txt".to_string(),
                    reasoning: "Analytics help identify bottlenecks and optimize resource usage.".to_string(),
                },
                BestPractice {
                    title: "Design Workflows for Complex Tasks".to_string(),
                    description: "Use workflow orchestration for dependent tasks".to_string(),
                    do_example: "durable-run --workflow etl-workflow.yaml".to_string(),
                    dont_example: "durable-run 'step1 && step2 && step3' --input data.txt".to_string(),
                    reasoning: "Workflows provide better error handling, retries, and dependency management.".to_string(),
                },
            ],
        }
    }

    pub fn all_practices(&self) -> &[BestPractice] {
        &self.practices
    }

    /// Load best practices from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let practices: Vec<BestPractice> = serde_json::from_str(&content)?;
        Ok(Self { practices })
    }

    /// Save best practices to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.practices)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Search best practices using regex
    pub fn search_regex(&self, pattern: &str) -> Result<Vec<&BestPractice>> {
        let regex = Regex::new(pattern)?;
        Ok(self.practices.iter().filter(|p|
            regex.is_match(&p.title) ||
            regex.is_match(&p.description) ||
            regex.is_match(&p.do_example) ||
            regex.is_match(&p.dont_example) ||
            regex.is_match(&p.reasoning)
        ).collect())
    }

    /// Validate all best practices
    pub fn validate(&self) -> Result<()> {
        for practice in &self.practices {
            if practice.title.trim().is_empty() {
                return Err(DocsError::InvalidSection("Best practice title cannot be empty".to_string()));
            }
            if practice.description.trim().is_empty() {
                return Err(DocsError::InvalidSection("Best practice description cannot be empty".to_string()));
            }
            if practice.do_example.trim().is_empty() || practice.dont_example.trim().is_empty() {
                return Err(DocsError::InvalidSection("Best practice examples cannot be empty".to_string()));
            }
        }
        Ok(())
    }

    /// Search best practices by keyword
    pub fn search_practices(&self, keyword: &str) -> Vec<&BestPractice> {
        let keyword_lower = keyword.to_lowercase();
        self.practices.iter().filter(|p|
            p.title.to_lowercase().contains(&keyword_lower) ||
            p.description.to_lowercase().contains(&keyword_lower) ||
            p.reasoning.to_lowercase().contains(&keyword_lower)
        ).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    pub title: String,
    pub difficulty: String, // "beginner", "intermediate", "advanced"
    pub duration: String,   // e.g., "30 minutes"
    pub prerequisites: Vec<String>,
    pub steps: Vec<TutorialStep>,
    pub learning_objectives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStep {
    pub title: String,
    pub content: String,
    pub command: Option<String>,
    pub expected_output: Option<String>,
}

pub struct Tutorials {
    tutorials: Vec<Tutorial>,
}

impl Default for Tutorials {
    fn default() -> Self {
        Self::new()
    }
}

impl Tutorials {
    pub fn new() -> Self {
        Self {
            tutorials: vec![
                Tutorial {
                    title: "Building Your First Parallel Workflow".to_string(),
                    difficulty: "beginner".to_string(),
                    duration: "20 minutes".to_string(),
                    prerequisites: vec!["Basic command line knowledge".to_string()],
                    learning_objectives: vec![
                        "Understand basic durable-run syntax".to_string(),
                        "Run commands in parallel".to_string(),
                        "Use input files for dynamic commands".to_string(),
                    ],
                    steps: vec![
                        TutorialStep {
                            title: "Install durable-run".to_string(),
                            content: "First, install durable-run using Cargo.".to_string(),
                            command: Some("cargo install durable-run".to_string()),
                            expected_output: None,
                        },
                        TutorialStep {
                            title: "Create an input file".to_string(),
                            content: "Create a file with items to process.".to_string(),
                            command: Some("echo -e 'item1\nitem2\nitem3' > items.txt".to_string()),
                            expected_output: None,
                        },
                        TutorialStep {
                            title: "Run parallel commands".to_string(),
                            content: "Process each item in parallel.".to_string(),
                            command: Some("durable-run 'echo Processing {1}' : items.txt".to_string()),
                            expected_output: Some("Processing item1\nProcessing item2\nProcessing item3".to_string()),
                        },
                    ],
                },
                Tutorial {
                    title: "Using Batch Configuration".to_string(),
                    difficulty: "intermediate".to_string(),
                    duration: "25 minutes".to_string(),
                    prerequisites: vec!["Basic durable-run usage".to_string(), "YAML/JSON knowledge".to_string()],
                    learning_objectives: vec![
                        "Create batch configuration files".to_string(),
                        "Configure job limits and timeouts".to_string(),
                        "Manage resources effectively".to_string(),
                    ],
                    steps: vec![
                        TutorialStep {
                            title: "Create a batch config file".to_string(),
                            content: "Create a YAML file with job configuration.".to_string(),
                            command: Some(r#"cat > batch.yaml << 'EOF'
max_jobs: 10
concurrent_jobs: 4
job_timeout: 300
batch_timeout: 3600
retry_attempts: 3
resources:
  memory_limit: "1GB"
  cpu_priority: 0
EOF"#.to_string()),
                            expected_output: None,
                        },
                        TutorialStep {
                            title: "Run with batch config".to_string(),
                            content: "Execute commands using the batch configuration.".to_string(),
                            command: Some("durable-run --config batch.yaml 'sleep 1 && echo Done {1}' : 1..5".to_string()),
                            expected_output: Some("Done 1\nDone 2\nDone 3\nDone 4\nDone 5".to_string()),
                        },
                    ],
                },
                Tutorial {
                    title: "Performance Monitoring with Analytics".to_string(),
                    difficulty: "advanced".to_string(),
                    duration: "30 minutes".to_string(),
                    prerequisites: vec!["Intermediate durable-run usage".to_string(), "Basic statistics knowledge".to_string()],
                    learning_objectives: vec![
                        "Enable analytics collection".to_string(),
                        "Interpret performance metrics".to_string(),
                        "Optimize execution based on data".to_string(),
                    ],
                    steps: vec![
                        TutorialStep {
                            title: "Run with analytics".to_string(),
                            content: "Execute commands with performance monitoring.".to_string(),
                            command: Some("durable-run --analytics --output stats.json 'sleep {1} && echo Task {1}' : 0.1 0.2 0.3".to_string()),
                            expected_output: None,
                        },
                        TutorialStep {
                            title: "View analytics results".to_string(),
                            content: "Examine the collected performance data.".to_string(),
                            command: Some("cat stats.json | jq '.summary'".to_string()),
                            expected_output: None,
                        },
                    ],
                },
            ],
        }
    }

    pub fn get_tutorial(&self, title: &str) -> Option<&Tutorial> {
        self.tutorials.iter().find(|t| t.title == title)
    }

    pub fn tutorials_by_difficulty(&self, difficulty: &str) -> Vec<&Tutorial> {
        self.tutorials.iter().filter(|t| t.difficulty == difficulty).collect()
    }

    pub fn all_tutorials(&self) -> &[Tutorial] {
        &self.tutorials
    }

    /// Search tutorials by keyword in title, difficulty, prerequisites, or learning objectives
    pub fn search_tutorials(&self, keyword: &str) -> Vec<&Tutorial> {
        let keyword_lower = keyword.to_lowercase();
        self.tutorials.iter().filter(|t|
            t.title.to_lowercase().contains(&keyword_lower) ||
            t.difficulty.to_lowercase().contains(&keyword_lower) ||
            t.prerequisites.iter().any(|p| p.to_lowercase().contains(&keyword_lower)) ||
            t.learning_objectives.iter().any(|obj| obj.to_lowercase().contains(&keyword_lower))
        ).collect()
    }

    /// Load tutorials from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let tutorials: Vec<Tutorial> = serde_json::from_str(&content)?;
        Ok(Self { tutorials })
    }

    /// Save tutorials to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.tutorials)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Search tutorials using regex across all fields
    pub fn search_regex(&self, pattern: &str) -> Result<Vec<&Tutorial>> {
        let regex = Regex::new(pattern)?;
        Ok(self.tutorials.iter().filter(|t|
            regex.is_match(&t.title) ||
            regex.is_match(&t.difficulty) ||
            regex.is_match(&t.duration) ||
            t.prerequisites.iter().any(|p| regex.is_match(p)) ||
            t.learning_objectives.iter().any(|obj| regex.is_match(obj)) ||
            t.steps.iter().any(|s| regex.is_match(&s.title) || regex.is_match(&s.content) ||
                              s.command.as_ref().is_some_and(|c| regex.is_match(c)) ||
                              s.expected_output.as_ref().is_some_and(|o| regex.is_match(o)))
        ).collect())
    }

    /// Validate all tutorials
    pub fn validate(&self) -> Result<()> {
        for tutorial in &self.tutorials {
            if tutorial.title.trim().is_empty() {
                return Err(DocsError::InvalidSection("Tutorial title cannot be empty".to_string()));
            }
            if tutorial.steps.is_empty() {
                return Err(DocsError::InvalidSection("Tutorial must have at least one step".to_string()));
            }
            for step in &tutorial.steps {
                if step.title.trim().is_empty() || step.content.trim().is_empty() {
                    return Err(DocsError::InvalidSection("Tutorial step title and content cannot be empty".to_string()));
                }
            }
        }
        Ok(())
    }
}
