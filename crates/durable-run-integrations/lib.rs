use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Integration error: {0}")]
    Integration(String),
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[async_trait]
pub trait Integration: Send + Sync {
    fn new() -> Self where Self: Sized;
    fn name(&self) -> &str;
    async fn execute(&self, args: Vec<String>) -> Result<String, IntegrationError>;
}

pub struct CargoIntegration;

#[async_trait]
impl Integration for CargoIntegration {
    fn new() -> Self {
        CargoIntegration
    }

    fn name(&self) -> &str {
        "cargo"
    }

    async fn execute(&self, args: Vec<String>) -> Result<String, IntegrationError> {
        if args.is_empty() {
            return Err(IntegrationError::Integration("No command provided".to_string()));
        }

        let command = &args[0];
        let rest_args = &args[1..];

        match command.as_str() {
            "version" | "--version" | "build" | "check" | "clean" | "test" | "run" | "doc" | "update" => {
                let output = Command::new("cargo")
                    .args(&[command])
                    .args(rest_args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .await?;

                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(IntegrationError::CommandFailed(
                        String::from_utf8_lossy(&output.stderr).to_string()
                    ))
                }
            }
            _ => Err(IntegrationError::Integration(format!(
                "Unknown Cargo command: {}",
                command
            ))),
        }
    }
}

pub struct DockerIntegration;

#[async_trait]
impl Integration for DockerIntegration {
    fn new() -> Self {
        DockerIntegration
    }

    fn name(&self) -> &str {
        "docker"
    }

    async fn execute(&self, args: Vec<String>) -> Result<String, IntegrationError> {
        if args.is_empty() {
            return Err(IntegrationError::Integration("No command provided".to_string()));
        }

        let command = &args[0];

        // For now, allow common docker commands. In a real implementation,
        // you might want to validate against a list of known commands.
        match command.as_str() {
            "build" | "run" | "pull" | "push" | "images" | "ps" | "stop" | "rm" | "rmi" | "--version" => {
                let output = Command::new("docker")
                    .args(&args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .await?;

                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(IntegrationError::CommandFailed(
                        String::from_utf8_lossy(&output.stderr).to_string()
                    ))
                }
            }
            _ => Err(IntegrationError::Integration(format!(
                "Unknown Docker command: {}",
                command
            ))),
        }
    }
}

pub struct NpmIntegration;

#[async_trait]
impl Integration for NpmIntegration {
    fn new() -> Self {
        NpmIntegration
    }

    fn name(&self) -> &str {
        "npm"
    }

    async fn execute(&self, args: Vec<String>) -> Result<String, IntegrationError> {
        if args.is_empty() {
            return Err(IntegrationError::Integration("No command provided".to_string()));
        }

        let command = &args[0];

        match command.as_str() {
            "install" | "run" | "test" | "start" | "build" | "init" | "--version" => {
                let output = Command::new("npm")
                    .args(&args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .await?;

                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(IntegrationError::CommandFailed(
                        String::from_utf8_lossy(&output.stderr).to_string()
                    ))
                }
            }
            _ => Err(IntegrationError::Integration(format!(
                "Unknown NPM command: {}",
                command
            ))),
        }
    }
}

pub struct GitIntegration;

#[async_trait]
impl Integration for GitIntegration {
    fn new() -> Self {
        GitIntegration
    }

    fn name(&self) -> &str {
        "git"
    }

    async fn execute(&self, args: Vec<String>) -> Result<String, IntegrationError> {
        if args.is_empty() {
            return Err(IntegrationError::Integration("No command provided".to_string()));
        }

        let command = &args[0];
        let rest_args = &args[1..];

        match command.as_str() {
            "status" | "add" | "commit" | "push" | "pull" | "clone" | "log" | "diff" | "branch" => {
                let output = Command::new("git")
                    .args(&[command])
                    .args(rest_args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .await?;

                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(IntegrationError::CommandFailed(
                        String::from_utf8_lossy(&output.stderr).to_string()
                    ))
                }
            }
            _ => Err(IntegrationError::Integration(format!(
                "Unknown Git command: {}",
                command
            ))),
        }
    }
}

pub struct MakeIntegration;

#[async_trait]
impl Integration for MakeIntegration {
    fn new() -> Self {
        MakeIntegration
    }

    fn name(&self) -> &str {
        "make"
    }

    async fn execute(&self, args: Vec<String>) -> Result<String, IntegrationError> {
        if args.is_empty() {
            return Err(IntegrationError::Integration("No command provided".to_string()));
        }

        let command = &args[0];

        match command.as_str() {
            "all" | "clean" | "install" | "test" | "--help" | "--version" => {
                let output = Command::new("make")
                    .args(&args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .await?;

                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(IntegrationError::CommandFailed(
                        String::from_utf8_lossy(&output.stderr).to_string()
                    ))
                }
            }
            _ => Err(IntegrationError::Integration(format!(
                "Unknown Make command: {}",
                command
            ))),
        }
    }
}

pub struct HttpIntegration;

#[async_trait]
impl Integration for HttpIntegration {
    fn new() -> Self {
        HttpIntegration
    }

    fn name(&self) -> &str {
        "http"
    }

    async fn execute(&self, args: Vec<String>) -> Result<String, IntegrationError> {
        if args.is_empty() {
            return Err(IntegrationError::Integration("No command provided".to_string()));
        }

        let method = &args[0].to_uppercase();
        let client = reqwest::Client::new();

        match method.as_str() {
            "GET" => {
                let url = args.get(1).ok_or_else(|| {
                    IntegrationError::Integration("URL required for HTTP requests".to_string())
                })?;
                let response = client.get(url).send().await.map_err(|e| {
                    IntegrationError::CommandFailed(e.to_string())
                })?;
                let text = response.text().await.map_err(|e| {
                    IntegrationError::CommandFailed(e.to_string())
                })?;
                Ok(text)
            }
            "POST" => {
                let url = args.get(1).ok_or_else(|| {
                    IntegrationError::Integration("URL required for HTTP requests".to_string())
                })?;
                let body = args.get(2).unwrap_or(&"".to_string()).clone();
                let response = client.post(url).body(body).send().await.map_err(|e| {
                    IntegrationError::CommandFailed(e.to_string())
                })?;
                let text = response.text().await.map_err(|e| {
                    IntegrationError::CommandFailed(e.to_string())
                })?;
                Ok(text)
            }
            "PUT" => {
                let url = args.get(1).ok_or_else(|| {
                    IntegrationError::Integration("URL required for HTTP requests".to_string())
                })?;
                let body = args.get(2).unwrap_or(&"".to_string()).clone();
                let response = client.put(url).body(body).send().await.map_err(|e| {
                    IntegrationError::CommandFailed(e.to_string())
                })?;
                let text = response.text().await.map_err(|e| {
                    IntegrationError::CommandFailed(e.to_string())
                })?;
                Ok(text)
            }
            "DELETE" => {
                let url = args.get(1).ok_or_else(|| {
                    IntegrationError::Integration("URL required for HTTP requests".to_string())
                })?;
                let response = client.delete(url).send().await.map_err(|e| {
                    IntegrationError::CommandFailed(e.to_string())
                })?;
                let text = response.text().await.map_err(|e| {
                    IntegrationError::CommandFailed(e.to_string())
                })?;
                Ok(text)
            }
            _ => Err(IntegrationError::Integration(format!(
                "Unsupported HTTP method: {}",
                method
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub name: String,
    pub integration: String,
    pub command: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowTemplate {
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
}

pub struct WorkflowTemplates {
    templates: std::collections::HashMap<String, WorkflowTemplate>,
}

impl WorkflowTemplates {
    pub fn new() -> Self {
        WorkflowTemplates {
            templates: std::collections::HashMap::new(),
        }
    }

    pub fn add_template(&mut self, template: WorkflowTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    pub fn get_template(&self, name: &str) -> Option<&WorkflowTemplate> {
        self.templates.get(name)
    }

    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    pub async fn execute_template(
        &self,
        name: &str,
        integrations: &std::collections::HashMap<String, Box<dyn Integration + Send + Sync>>,
    ) -> Result<String, IntegrationError> {
        let template = self.get_template(name).ok_or_else(|| {
            IntegrationError::Integration(format!("Template '{}' not found", name))
        })?;

        let mut results = Vec::new();

        for step in &template.steps {
            let integration = integrations.get(&step.integration).ok_or_else(|| {
                IntegrationError::Integration(format!("Integration '{}' not found", step.integration))
            })?;

            let result = integration.execute(step.command.clone()).await?;
            results.push(format!("Step '{}': {}", step.name, result));
        }

        Ok(results.join("\n"))
    }

    pub fn create_build_template() -> WorkflowTemplate {
        WorkflowTemplate {
            name: "build".to_string(),
            description: "Build project with Cargo".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "build".to_string(),
                    integration: "cargo".to_string(),
                    command: vec!["build".to_string(), "--release".to_string()],
                },
                WorkflowStep {
                    name: "test".to_string(),
                    integration: "cargo".to_string(),
                    command: vec!["test".to_string()],
                },
            ],
        }
    }

    pub fn create_node_build_template() -> WorkflowTemplate {
        WorkflowTemplate {
            name: "node-build".to_string(),
            description: "Build Node.js project".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "install".to_string(),
                    integration: "npm".to_string(),
                    command: vec!["install".to_string()],
                },
                WorkflowStep {
                    name: "build".to_string(),
                    integration: "npm".to_string(),
                    command: vec!["run".to_string(), "build".to_string()],
                },
                WorkflowStep {
                    name: "test".to_string(),
                    integration: "npm".to_string(),
                    command: vec!["run".to_string(), "test".to_string()],
                },
            ],
        }
    }

    pub fn create_docker_build_template() -> WorkflowTemplate {
        WorkflowTemplate {
            name: "docker-build".to_string(),
            description: "Build and test with Docker".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "build".to_string(),
                    integration: "docker".to_string(),
                    command: vec!["build".to_string(), "Dockerfile".to_string(), "myapp".to_string()],
                },
                WorkflowStep {
                    name: "test".to_string(),
                    integration: "docker".to_string(),
                    command: vec!["run".to_string(), "myapp".to_string(), "npm".to_string(), "test".to_string()],
                },
            ],
        }
    }

    pub fn create_full_stack_template() -> WorkflowTemplate {
        WorkflowTemplate {
            name: "full-stack".to_string(),
            description: "Build full-stack application".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "build-backend".to_string(),
                    integration: "cargo".to_string(),
                    command: vec!["build".to_string(), "--release".to_string()],
                },
                WorkflowStep {
                    name: "test-backend".to_string(),
                    integration: "cargo".to_string(),
                    command: vec!["test".to_string()],
                },
                WorkflowStep {
                    name: "install-frontend".to_string(),
                    integration: "npm".to_string(),
                    command: vec!["install".to_string()],
                },
                WorkflowStep {
                    name: "build-frontend".to_string(),
                    integration: "npm".to_string(),
                    command: vec!["run".to_string(), "build".to_string()],
                },
                WorkflowStep {
                    name: "build-docker".to_string(),
                    integration: "docker".to_string(),
                    command: vec!["build".to_string(), "Dockerfile".to_string(), "app".to_string()],
                },
            ],
        }
    }
}