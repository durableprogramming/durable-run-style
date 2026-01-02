use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io;
use thiserror::Error;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Error)]
pub enum InputError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Invalid input: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, InputError>;

#[async_trait]
pub trait InputSource: Send + Sync {
    async fn read_inputs(&self) -> Result<Vec<String>>;
}

#[async_trait]
pub trait InputProcessor {
    async fn process(&self, input: String) -> Result<Vec<String>>;
}

pub struct StdinSource;

impl StdinSource {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdinSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InputSource for StdinSource {
    async fn read_inputs(&self) -> Result<Vec<String>> {
        let stdin = tokio::io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();
        let mut result = Vec::new();

        while let Some(line) = lines.next_line().await? {
            result.push(line);
        }

        Ok(result)
    }
}

pub struct StdinProcessor;

impl StdinProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdinProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InputProcessor for StdinProcessor {
    async fn process(&self, input: String) -> Result<Vec<String>> {
        Ok(vec![input])
    }
}

pub struct FileSource {
    path: String,
}

impl FileSource {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

#[async_trait]
impl InputSource for FileSource {
    async fn read_inputs(&self) -> Result<Vec<String>> {
        let file = File::open(&self.path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut result = Vec::new();

        while let Some(line) = lines.next_line().await? {
            result.push(line);
        }

        Ok(result)
    }
}

pub struct JsonFileSource {
    path: String,
}

impl JsonFileSource {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

#[async_trait]
impl InputSource for JsonFileSource {
    async fn read_inputs(&self) -> Result<Vec<String>> {
        let content = tokio::fs::read_to_string(&self.path).await?;
        let value: serde_json::Value = serde_json::from_str(&content)?;

        let result = match value {
            serde_json::Value::Array(arr) => {
                // Extract strings from array elements
                arr.into_iter()
                    .filter_map(|v| match v {
                        serde_json::Value::String(s) => Some(s),
                        serde_json::Value::Number(n) => Some(n.to_string()),
                        serde_json::Value::Bool(b) => Some(b.to_string()),
                        _ => None, // Skip objects, arrays, null
                    })
                    .collect()
            }
            serde_json::Value::Object(obj) => {
                // Extract string values from object
                obj.into_iter()
                    .filter_map(|(_, v)| match v {
                        serde_json::Value::String(s) => Some(s),
                        serde_json::Value::Number(n) => Some(n.to_string()),
                        serde_json::Value::Bool(b) => Some(b.to_string()),
                        _ => None,
                    })
                    .collect()
            }
            serde_json::Value::String(s) => vec![s],
            serde_json::Value::Number(n) => vec![n.to_string()],
            serde_json::Value::Bool(b) => vec![b.to_string()],
            _ => Vec::new(),
        };

        Ok(result)
    }
}

pub struct FileProcessor {
    path: String,
}

impl FileProcessor {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

#[async_trait]
impl InputProcessor for FileProcessor {
    async fn process(&self, input: String) -> Result<Vec<String>> {
        // If input is a file path, read it; otherwise treat input as content
        let path = if input.is_empty() { &self.path } else { &input };
        let file = File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut result = Vec::new();

        while let Some(line) = lines.next_line().await? {
            result.push(line);
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputTransformation {
    pub filter: Option<String>,
    pub filter_regex: Option<String>,
    pub map: Option<String>,
    pub map_regex: Option<(String, String)>,
    pub limit: Option<usize>,
    pub skip: Option<usize>,
    #[serde(default)]
    pub unique: bool,
    #[serde(skip)]
    filter_regex_compiled: Option<Regex>,
    #[serde(skip)]
    map_regex_compiled: Option<Regex>,
}

impl InputTransformation {
    pub fn new() -> Self {
        Self {
            filter: None,
            filter_regex: None,
            map: None,
            map_regex: None,
            limit: None,
            skip: None,
            unique: false,
            filter_regex_compiled: None,
            map_regex_compiled: None,
        }
    }

    pub fn with_filter(mut self, filter: String) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn with_filter_regex(mut self, regex: String) -> Self {
        self.filter_regex = Some(regex);
        self
    }

    pub fn with_map(mut self, map: String) -> Self {
        self.map = Some(map);
        self
    }

    pub fn with_map_regex(mut self, regex: String, replacement: String) -> Self {
        self.map_regex = Some((regex, replacement));
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn with_unique(mut self, unique: bool) -> Self {
        self.unique = unique;
        self
    }

    pub async fn apply(&mut self, inputs: Vec<String>) -> Result<Vec<String>> {
        let mut result = inputs;

        // Skip
        if let Some(skip) = self.skip {
            if skip < result.len() {
                result = result.into_iter().skip(skip).collect();
            } else {
                result = Vec::new();
            }
        }

        // Filter by substring
        if let Some(filter) = &self.filter {
            result.retain(|s| s.contains(filter));
        }

        // Filter by regex
        if let Some(regex_str) = &self.filter_regex {
            if self.filter_regex_compiled.is_none() {
                self.filter_regex_compiled = Some(Regex::new(regex_str).map_err(|e| InputError::Invalid(format!("Invalid regex: {}", e)))?);
            }
            let regex = self.filter_regex_compiled.as_ref().unwrap();
            result.retain(|s| regex.is_match(s));
        }

        // Map with prefix
        if let Some(map) = &self.map {
            result = result.into_iter().map(|s| format!("{}{}", map, s)).collect();
        }

        // Map with regex replacement
        if let Some((regex_str, replacement)) = &self.map_regex {
            if self.map_regex_compiled.is_none() {
                self.map_regex_compiled = Some(Regex::new(regex_str).map_err(|e| InputError::Invalid(format!("Invalid regex: {}", e)))?);
            }
            let regex = self.map_regex_compiled.as_ref().unwrap();
            result = result.into_iter().map(|s| regex.replace_all(&s, replacement).to_string()).collect();
        }

        // Unique
        if self.unique {
            let mut seen = HashSet::new();
            result.retain(|s| seen.insert(s.clone()));
        }

        // Limit
        if let Some(limit) = self.limit {
            result.truncate(limit);
        }

        Ok(result)
    }
}

impl Default for InputTransformation {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ProcessingPipeline {
    processors: Vec<Box<dyn InputProcessor + Send + Sync>>,
    transformation: InputTransformation,
    concurrency_limit: usize,
}

impl ProcessingPipeline {
    pub fn new(transformation: InputTransformation) -> Self {
        Self::with_concurrency(transformation, 10)
    }

    pub fn with_concurrency(transformation: InputTransformation, concurrency_limit: usize) -> Self {
        Self {
            processors: Vec::new(),
            transformation,
            concurrency_limit,
        }
    }

    pub fn add_processor<P: InputProcessor + Send + Sync + 'static>(mut self, processor: P) -> Self {
        self.processors.push(Box::new(processor));
        self
    }

    pub async fn process(&mut self, inputs: Vec<String>) -> Result<Vec<String>> {
        let mut current = inputs;

        for processor in &self.processors {
            let futures = current.into_iter().map(|input| async {
                processor.process(input).await
            });
            let results = stream::iter(futures).buffer_unordered(self.concurrency_limit).collect::<Vec<_>>().await;
            let temp = results.into_iter().collect::<Result<Vec<_>>>()?;
            current = temp.into_iter().flatten().collect();
        }

        self.transformation.apply(current).await
    }

    pub async fn process_from_source(&mut self, source: &dyn InputSource) -> Result<Vec<String>> {
        let inputs = source.read_inputs().await?;
        self.process(inputs).await
    }
}

pub struct NoOpProcessor;

#[async_trait]
impl InputProcessor for NoOpProcessor {
    async fn process(&self, input: String) -> Result<Vec<String>> {
        Ok(vec![input])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_stdin_source() {
        let source = StdinSource::new();
        // Since we can't easily test stdin in unit tests without hanging,
        // just ensure the struct can be created
        // In real usage, stdin would be piped from external commands
        let _source = source;
    }

    #[tokio::test]
    async fn test_file_source() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "line1").unwrap();
        writeln!(temp_file, "line2").unwrap();
        writeln!(temp_file, "line3").unwrap();

        let source = FileSource::new(temp_file.path().to_string_lossy().to_string());
        let result = source.read_inputs().await.unwrap();
        assert_eq!(result, vec!["line1", "line2", "line3"]);
    }

    #[tokio::test]
    async fn test_json_file_source() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"["item1", "item2", "item3"]"#).unwrap();

        let source = JsonFileSource::new(temp_file.path().to_string_lossy().to_string());
        let result = source.read_inputs().await.unwrap();
        assert_eq!(result, vec!["item1", "item2", "item3"]);
    }

    #[tokio::test]
    async fn test_json_file_source_mixed() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"["item1", 123, "item2", true]"#).unwrap();

        let source = JsonFileSource::new(temp_file.path().to_string_lossy().to_string());
        let result = source.read_inputs().await.unwrap();
        assert_eq!(result, vec!["item1", "123", "item2", "true"]);
    }



    #[tokio::test]
    async fn test_stdin_processor() {
        let processor = StdinProcessor::new();
        let result = processor.process("test input".to_string()).await.unwrap();
        assert_eq!(result, vec!["test input"]);
    }

    #[tokio::test]
    async fn test_file_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "file line1").unwrap();
        writeln!(temp_file, "file line2").unwrap();

        let processor = FileProcessor::new(temp_file.path().to_string_lossy().to_string());
        let result = processor.process("".to_string()).await.unwrap();
        assert_eq!(result, vec!["file line1", "file line2"]);
    }

    #[tokio::test]
    async fn test_noop_processor() {
        let processor = NoOpProcessor;
        let result = processor.process("test".to_string()).await.unwrap();
        assert_eq!(result, vec!["test"]);
    }

    #[tokio::test]
    async fn test_input_transformation_filter() {
        let mut transform = InputTransformation::new().with_filter("test".to_string());
        let inputs = vec!["test1".to_string(), "other".to_string(), "test2".to_string()];
        let result = transform.apply(inputs).await.unwrap();
        assert_eq!(result, vec!["test1", "test2"]);
    }

    #[tokio::test]
    async fn test_input_transformation_filter_regex() {
        let mut transform = InputTransformation::new().with_filter_regex(r"\d+".to_string());
        let inputs = vec!["abc".to_string(), "123".to_string(), "def".to_string()];
        let result = transform.apply(inputs).await.unwrap();
        assert_eq!(result, vec!["123"]);
    }

    #[tokio::test]
    async fn test_input_transformation_map() {
        let mut transform = InputTransformation::new().with_map("prefix: ".to_string());
        let inputs = vec!["a".to_string(), "b".to_string()];
        let result = transform.apply(inputs).await.unwrap();
        assert_eq!(result, vec!["prefix: a", "prefix: b"]);
    }

    #[tokio::test]
    async fn test_input_transformation_limit() {
        let mut transform = InputTransformation::new().with_limit(2);
        let inputs = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = transform.apply(inputs).await.unwrap();
        assert_eq!(result, vec!["a", "b"]);
    }

    #[tokio::test]
    async fn test_input_transformation_skip() {
        let mut transform = InputTransformation::new().with_skip(1);
        let inputs = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = transform.apply(inputs).await.unwrap();
        assert_eq!(result, vec!["b", "c"]);
    }

    #[tokio::test]
    async fn test_input_transformation_unique() {
        let mut transform = InputTransformation::new().with_unique(true);
        let inputs = vec!["a".to_string(), "b".to_string(), "a".to_string(), "c".to_string()];
        let result = transform.apply(inputs).await.unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn test_input_transformation_map_regex() {
        let mut transform = InputTransformation::new().with_map_regex(r"\d+".to_string(), "NUM".to_string());
        let inputs = vec!["abc123".to_string(), "def".to_string(), "456ghi".to_string()];
        let result = transform.apply(inputs).await.unwrap();
        assert_eq!(result, vec!["abcNUM", "def", "NUMghi"]);
    }

    #[tokio::test]
    async fn test_processing_pipeline() {
        let transform = InputTransformation::new().with_limit(2);
        let mut pipeline = ProcessingPipeline::new(transform);
        pipeline = pipeline.add_processor(NoOpProcessor);

        let inputs = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = pipeline.process(inputs).await.unwrap();
        assert_eq!(result, vec!["a", "b"]);
    }

    #[tokio::test]
    async fn test_processing_pipeline_from_source() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "line1").unwrap();
        writeln!(temp_file, "line2").unwrap();

        let source = FileSource::new(temp_file.path().to_string_lossy().to_string());
        let transform = InputTransformation::new().with_limit(1);
        let mut pipeline = ProcessingPipeline::new(transform);

        let result = pipeline.process_from_source(&source).await.unwrap();
        assert_eq!(result, vec!["line1"]);
    }

    #[tokio::test]
    async fn test_processing_pipeline_with_concurrency() {
        let transform = InputTransformation::new().with_limit(2);
        let mut pipeline = ProcessingPipeline::with_concurrency(transform, 5);
        pipeline = pipeline.add_processor(NoOpProcessor);

        let inputs = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = pipeline.process(inputs).await.unwrap();
        assert_eq!(result, vec!["a", "b"]);
    }
}