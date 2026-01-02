//! # Durable Run Plugin
//!
//! A flexible plugin system for extending durable-run functionality.
//!
//! This crate provides:
//! - Dynamic library loading for plugins
//! - Plugin trait for custom functionality
//! - Plugin manager for registration and execution
//! - Built-in plugin types: Template, Processor, and Input plugins
//! - Integration with durable-run-input for data processing
//!
//! ## Features
//!
//! - **Dynamic Loading**: Load plugins from shared libraries at runtime
//! - **Type Safety**: Strongly typed plugin interfaces with async support
//! - **Extensibility**: Easy to create custom plugin types
//! - **Input Integration**: Seamless integration with input processing pipelines
//! - **Error Handling**: Comprehensive error types for robust plugin management
//!
//! ## Example
//!
//! ```rust,no_run
//! use durable_run_plugin::{PluginManager, TemplatePlugin, PluginHost};
//!
//! let mut manager = PluginManager::new();
//! let template_plugin = Box::new(TemplatePlugin::new(
//!     "greeting".to_string(),
//!     "Hello {name}!".to_string()
//! ));
//!
//! manager.register_plugin(template_plugin);
//!
//! // Execute the plugin
//! let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
//!     manager.execute_plugin("greeting", serde_json::json!({"name": "World"})).await
//! });
//!
//! println!("Result: {:?}", result);
//! ```

use async_trait::async_trait;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

// Re-export from durable-run-input for plugin convenience
pub use durable_run_input::{InputSource, InputProcessor, InputTransformation, ProcessingPipeline};

/// Errors that can occur during plugin operations
#[derive(Debug, Error)]
pub enum PluginError {
    /// Error loading a dynamic library
    #[error("Library loading error: {0}")]
    Library(#[from] libloading::Error),
    /// Plugin with the specified name was not found
    #[error("Plugin not found: {0}")]
    NotFound(String),
    /// Error occurred during plugin execution
    #[error("Plugin execution error: {0}")]
    Execution(String),
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, PluginError>;

/// Metadata describing a plugin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginMetadata {
    /// Unique name of the plugin
    pub name: String,
    /// Version string of the plugin
    pub version: String,
    /// Human-readable description of the plugin
    pub description: String,
    /// Type of the plugin
    pub plugin_type: PluginType,
    /// Configuration schema (JSON Schema)
    #[serde(default)]
    pub config_schema: Option<serde_json::Value>,
}

/// Types of plugins supported by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginType {
    /// Template substitution plugin
    Template,
    /// Data processing plugin
    Processor,
    /// Custom plugin type
    Custom(String),
}

/// Core trait that all plugins must implement
///
/// Plugins are executed asynchronously and can process JSON input to produce JSON output.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Execute the plugin with the given input
    ///
    /// # Arguments
    /// * `input` - JSON value to process
    ///
    /// # Returns
    /// Processed JSON value or an error
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value>;

    /// Get metadata about this plugin
    fn metadata(&self) -> PluginMetadata;
}

/// Factory function type for creating plugin instances
pub type PluginFactory = fn() -> Box<dyn Plugin>;

/// Manager for loading, registering, and executing plugins
///
/// The PluginManager handles both statically registered plugins and dynamically
/// loaded plugins from shared libraries.
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    libraries: Vec<Library>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            libraries: Vec::new(),
        }
    }

    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let library = unsafe {
            Library::new(path.as_ref().as_os_str())
                .map_err(PluginError::Library)?
        };

        // Get the constructor function
        let constructor: Symbol<PluginFactory> = unsafe {
            library.get(b"create_plugin")
                .map_err(PluginError::Library)?
        };

        // Create the plugin instance
        let plugin = constructor();

        let metadata = plugin.metadata();

        // Check for duplicate plugin names
        if self.plugins.contains_key(&metadata.name) {
            return Err(PluginError::Execution(format!("Plugin '{}' already loaded", metadata.name)));
        }

        self.plugins.insert(metadata.name.clone(), plugin);
        self.libraries.push(library);

        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub async fn execute_plugin(&self, name: &str, input: serde_json::Value) -> Result<serde_json::Value> {
        if let Some(plugin) = self.get_plugin(name) {
            plugin.execute(input).await
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }

    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }

    /// Get configuration schema for a specific plugin
    pub fn get_plugin_config_schema(&self, name: &str) -> Option<serde_json::Value> {
        self.plugins.get(name)
            .and_then(|p| p.metadata().config_schema)
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        // Libraries are dropped automatically
    }
}

// Template Plugin
pub struct TemplatePlugin {
    name: String,
    template: String,
    engine: Arc<Mutex<durable_run_template::TemplateEngine>>,
}

impl TemplatePlugin {
    pub fn new(name: String, template: String) -> Self {
        Self {
            name,
            template,
            engine: Arc::new(Mutex::new(durable_run_template::TemplateEngine::new())),
        }
    }
}

#[async_trait]
impl Plugin for TemplatePlugin {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        // Create context from input JSON
        let mut context = durable_run_template::Context::new();

        if let Some(obj) = input.as_object() {
            for (key, value) in obj {
                let value_str = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Array(arr) => {
                        // Convert array to space-separated string for positional args
                        arr.iter().map(|v| v.to_string().trim_matches('"').to_string()).collect::<Vec<_>>().join(" ")
                    }
                    _ => value.to_string(),
                };
                context = context.with_var(key.clone(), value_str);
            }
        }

        // Render the template
        let mut engine = self.engine.lock().unwrap();
        let result = engine.render(&self.template, &mut context)
            .map_err(|e| PluginError::Execution(format!("Template rendering error: {}", e)))?;

        Ok(serde_json::json!(result))
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            description: "Template substitution plugin".to_string(),
            plugin_type: PluginType::Template,
            config_schema: None,
        }
    }
}

// Processor Plugin
pub struct ProcessorPlugin {
    name: String,
    process_fn: Arc<dyn Fn(serde_json::Value) -> serde_json::Value + Send + Sync>,
}

impl ProcessorPlugin {
    pub fn new<F>(name: String, process_fn: F) -> Self
    where
        F: Fn(serde_json::Value) -> serde_json::Value + Send + Sync + 'static,
    {
        Self {
            name,
            process_fn: Arc::new(process_fn),
        }
    }
}

#[async_trait]
impl Plugin for ProcessorPlugin {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let result = (self.process_fn)(input);
        Ok(result)
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            description: "Data processing plugin".to_string(),
            plugin_type: PluginType::Processor,
            config_schema: None,
        }
    }
}

// Input Plugin - integrates with durable-run-input
pub struct InputPlugin {
    name: String,
    source: Box<dyn InputSource + Send + Sync>,
    transformation: InputTransformation,
}

impl InputPlugin {
    pub fn new(
        name: String,
        source: Box<dyn InputSource + Send + Sync>,
        transformation: InputTransformation,
    ) -> Self {
        Self {
            name,
            source,
            transformation,
        }
    }

    pub fn with_source<S: InputSource + Send + Sync + 'static>(name: String, source: S) -> Self {
        Self {
            name,
            source: Box::new(source),
            transformation: InputTransformation::new(),
        }
    }

    pub fn with_transformation(mut self, transformation: InputTransformation) -> Self {
        self.transformation = transformation;
        self
    }
}

#[async_trait]
impl Plugin for InputPlugin {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        // Use input as configuration if provided, otherwise use default
        let config = if input.is_object() && !input.as_object().unwrap().is_empty() {
            if let Ok(transform) = serde_json::from_value::<InputTransformation>(input.clone()) {
                transform
            } else {
                self.transformation.clone()
            }
        } else {
            self.transformation.clone()
        };

        let mut pipeline = ProcessingPipeline::new(config);

        let inputs = self.source.read_inputs().await
            .map_err(|e| PluginError::Execution(format!("Input source error: {}", e)))?;

        let processed = pipeline.process(inputs).await
            .map_err(|e| PluginError::Execution(format!("Processing error: {}", e)))?;

        Ok(serde_json::json!(processed))
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            description: "Input processing plugin".to_string(),
            plugin_type: PluginType::Custom("input".to_string()),
            config_schema: None,
        }
    }
}

// Plugin Architecture for Extensibility
pub trait PluginHost {
    fn register_plugin(&mut self, plugin: Box<dyn Plugin>);
    fn unregister_plugin(&mut self, name: &str);
    fn list_registered_plugins(&self) -> Vec<String>;
}

impl PluginHost for PluginManager {
    fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        let name = plugin.metadata().name;
        self.plugins.insert(name, plugin);
    }

    fn unregister_plugin(&mut self, name: &str) {
        self.plugins.remove(name);
    }

    fn list_registered_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}

// Example of creating a plugin library
// Note: Dynamic loading functionality is available through PluginManager::load_plugin
// For creating plugins programmatically, use the Plugin constructors directly

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio;

    #[tokio::test]
    async fn test_template_plugin() {
        let plugin = TemplatePlugin::new("test_template".to_string(), "Hello {name}, you are {age} years old!".to_string());

        let input = json!({"name": "Alice", "age": "30"});
        let result = plugin.execute(input).await.unwrap();

        assert_eq!(result, json!("Hello Alice, you are 30 years old!"));

        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "test_template");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.description, "Template substitution plugin");
        assert_eq!(metadata.plugin_type, PluginType::Template);
    }

    #[tokio::test]
    async fn test_processor_plugin() {
        let plugin = ProcessorPlugin::new("test_processor".to_string(), |input: serde_json::Value| {
            if let Some(arr) = input.as_array() {
                let sum: i64 = arr.iter().filter_map(|v| v.as_i64()).sum();
                json!(sum)
            } else {
                json!(0)
            }
        });

        let input = json!([1, 2, 3, 4, 5]);
        let result = plugin.execute(input).await.unwrap();
        assert_eq!(result, json!(15));

        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "test_processor");
        assert_eq!(metadata.plugin_type, PluginType::Processor);
    }

    #[test]
    fn test_plugin_manager_new() {
        let manager = PluginManager::new();
        assert!(manager.plugins.is_empty());
        assert!(manager.libraries.is_empty());
    }

    #[test]
    fn test_plugin_manager_register_plugin() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TemplatePlugin::new("test".to_string(), "template".to_string()));

        manager.register_plugin(plugin);

        assert_eq!(manager.plugins.len(), 1);
        assert!(manager.get_plugin("test").is_some());
    }

    #[tokio::test]
    async fn test_plugin_manager_execute_plugin() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TemplatePlugin::new("test".to_string(), "Hello {name}!".to_string()));
        manager.register_plugin(plugin);

        let input = json!({"name": "World"});
        let result = manager.execute_plugin("test", input).await.unwrap();

        assert_eq!(result, json!("Hello World!"));
    }

    #[tokio::test]
    async fn test_plugin_manager_execute_plugin_not_found() {
        let manager = PluginManager::new();
        let input = json!({});

        let result = manager.execute_plugin("nonexistent", input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::NotFound(_)));
    }

    #[test]
    fn test_plugin_manager_list_plugins() {
        let mut manager = PluginManager::new();
        let plugin1 = Box::new(TemplatePlugin::new("plugin1".to_string(), "template1".to_string()));
        let plugin2 = Box::new(ProcessorPlugin::new("plugin2".to_string(), |_| json!("processed")));

        manager.register_plugin(plugin1);
        manager.register_plugin(plugin2);

        let metadata_list = manager.list_plugins();
        assert_eq!(metadata_list.len(), 2);

        let names: Vec<String> = metadata_list.iter().map(|m| m.name.clone()).collect();
        assert!(names.contains(&"plugin1".to_string()));
        assert!(names.contains(&"plugin2".to_string()));
    }

    #[test]
    fn test_plugin_manager_unregister_plugin() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TemplatePlugin::new("test".to_string(), "template".to_string()));
        manager.register_plugin(plugin);

        assert!(manager.get_plugin("test").is_some());

        manager.unregister_plugin("test");
        assert!(manager.get_plugin("test").is_none());
    }

    #[test]
    fn test_plugin_manager_list_registered_plugins() {
        let mut manager = PluginManager::new();
        let plugin1 = Box::new(TemplatePlugin::new("plugin1".to_string(), "template1".to_string()));
        let plugin2 = Box::new(ProcessorPlugin::new("plugin2".to_string(), |_| json!("processed")));

        manager.register_plugin(plugin1);
        manager.register_plugin(plugin2);

        let registered = manager.list_registered_plugins();
        assert_eq!(registered.len(), 2);
        assert!(registered.contains(&"plugin1".to_string()));
        assert!(registered.contains(&"plugin2".to_string()));
    }

    #[test]
    fn test_plugin_manager_get_config_schema() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TemplatePlugin::new("test".to_string(), "template".to_string()));
        manager.register_plugin(plugin);

        let schema = manager.get_plugin_config_schema("test");
        assert!(schema.is_none()); // TemplatePlugin doesn't have a schema

        let nonexistent = manager.get_plugin_config_schema("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_plugin_metadata_serialization() {
        let metadata = PluginMetadata {
            name: "test_plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            plugin_type: PluginType::Custom("special".to_string()),
            config_schema: None,
        };

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: PluginMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, metadata);
    }

    #[test]
    fn test_plugin_type_serialization() {
        let types = vec![
            PluginType::Template,
            PluginType::Processor,
            PluginType::Custom("test".to_string()),
        ];

        for plugin_type in types {
            let serialized = serde_json::to_string(&plugin_type).unwrap();
            let deserialized: PluginType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized, plugin_type);
        }
    }



    #[tokio::test]
    async fn test_input_plugin() {
        use durable_run_input::FileSource;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "line1").unwrap();
        writeln!(temp_file, "line2").unwrap();

        let source = FileSource::new(temp_file.path().to_string_lossy().to_string());
        let plugin = InputPlugin::with_source("test_input".to_string(), source);

        let input = serde_json::json!({});
        let result = plugin.execute(input).await.unwrap();

        let expected = serde_json::json!(["line1", "line2"]);
        assert_eq!(result, expected);

        let metadata = plugin.metadata();
        assert_eq!(metadata.name, "test_input");
        assert_eq!(metadata.plugin_type, PluginType::Custom("input".to_string()));
    }

    #[tokio::test]
    async fn test_input_plugin_with_transformation() {
        use durable_run_input::FileSource;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "apple").unwrap();
        writeln!(temp_file, "banana").unwrap();
        writeln!(temp_file, "cherry").unwrap();

        let source = FileSource::new(temp_file.path().to_string_lossy().to_string());
        let transformation = InputTransformation::new().with_filter("a".to_string()).with_limit(1);
        let plugin = InputPlugin::new("test_input".to_string(), Box::new(source), transformation);

        let input = serde_json::json!({});
        let result = plugin.execute(input).await.unwrap();

        let expected = serde_json::json!(["apple"]);
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_input_plugin_with_config_input() {
        use durable_run_input::FileSource;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "apple").unwrap();
        writeln!(temp_file, "banana").unwrap();
        writeln!(temp_file, "cherry").unwrap();

        let source = FileSource::new(temp_file.path().to_string_lossy().to_string());
        let plugin = InputPlugin::with_source("test_input".to_string(), source);

        let input = serde_json::json!({"filter": "a", "limit": 1});
        let result = plugin.execute(input).await.unwrap();

        let expected = serde_json::json!(["apple"]);
        assert_eq!(result, expected);
    }


}
