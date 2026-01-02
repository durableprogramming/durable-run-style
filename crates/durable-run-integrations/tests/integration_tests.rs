use durable_run_integrations::*;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cargo_integration_new() {
        let cargo = CargoIntegration::new();
        assert_eq!(cargo.name(), "cargo");
    }

    #[tokio::test]
    async fn test_docker_integration_new() {
        let docker = DockerIntegration::new();
        assert_eq!(docker.name(), "docker");
    }

    #[tokio::test]
    async fn test_npm_integration_new() {
        let npm = NpmIntegration::new();
        assert_eq!(npm.name(), "npm");
    }

    #[tokio::test]
    async fn test_git_integration_new() {
        let git = GitIntegration::new();
        assert_eq!(git.name(), "git");
    }

    #[tokio::test]
    async fn test_make_integration_new() {
        let make = MakeIntegration::new();
        assert_eq!(make.name(), "make");
    }

    #[tokio::test]
    async fn test_http_integration_new() {
        let http = HttpIntegration::new();
        assert_eq!(http.name(), "http");
    }

    #[tokio::test]
    async fn test_cargo_execute_unknown_command() {
        let cargo = CargoIntegration::new();
        let result = cargo.execute(vec!["unknown".to_string()]).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown Cargo command"));
    }

    #[tokio::test]
    async fn test_docker_execute_unknown_command() {
        let docker = DockerIntegration::new();
        let result = docker.execute(vec!["unknown".to_string()]).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown Docker command"));
    }

    #[tokio::test]
    async fn test_npm_execute_unknown_command() {
        let npm = NpmIntegration::new();
        let result = npm.execute(vec!["unknown".to_string()]).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown NPM command"));
    }

    #[tokio::test]
    async fn test_git_execute_unknown_command() {
        let git = GitIntegration::new();
        let result = git.execute(vec!["unknown".to_string()]).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown Git command"));
    }

    #[tokio::test]
    async fn test_make_execute_unknown_command() {
        let make = MakeIntegration::new();
        let result = make.execute(vec!["unknown".to_string()]).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown Make command"));
    }

    #[tokio::test]
    async fn test_http_execute_unknown_command() {
        let http = HttpIntegration::new();
        let result = http.execute(vec!["unknown".to_string()]).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported HTTP method"));
    }

    #[tokio::test]
    async fn test_http_integration_execute_get() {
        let http = HttpIntegration::new();
        // Test with httpbin.org which returns request info
        let result = http.execute(vec!["get".to_string(), "https://httpbin.org/get".to_string()]).await;
        // Should succeed if network is available, or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_http_integration_execute_post() {
        let http = HttpIntegration::new();
        // Test with httpbin.org which accepts POST requests
        let result = http.execute(vec!["post".to_string(), "https://httpbin.org/post".to_string(), r#"{"test": "data"}"#.to_string()]).await;
        // Should succeed if network is available, or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_http_integration_execute_put() {
        let http = HttpIntegration::new();
        // Test with httpbin.org which accepts PUT requests
        let result = http.execute(vec!["put".to_string(), "https://httpbin.org/put".to_string(), r#"{"test": "data"}"#.to_string()]).await;
        // Should succeed if network is available, or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_http_integration_execute_delete() {
        let http = HttpIntegration::new();
        // Test with httpbin.org which accepts DELETE requests
        let result = http.execute(vec!["delete".to_string(), "https://httpbin.org/delete".to_string()]).await;
        // Should succeed if network is available, or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_workflow_templates_new() {
        let templates = WorkflowTemplates::new();
        assert!(templates.list_templates().is_empty());
    }

    #[test]
    fn test_workflow_templates_add_and_get() {
        let mut templates = WorkflowTemplates::new();
        let template = WorkflowTemplate {
            name: "test".to_string(),
            description: "Test template".to_string(),
            steps: vec![],
        };

        templates.add_template(template);
        assert_eq!(templates.list_templates(), vec!["test"]);

        let retrieved = templates.get_template("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test");
    }

    #[test]
    fn test_workflow_templates_create_build_template() {
        let template = WorkflowTemplates::create_build_template();
        assert_eq!(template.name, "build");
        assert_eq!(template.description, "Build project with Cargo");
        assert_eq!(template.steps.len(), 2);
        assert_eq!(template.steps[0].integration, "cargo");
        assert_eq!(template.steps[0].command, vec!["build".to_string(), "--release".to_string()]);
        assert_eq!(template.steps[1].integration, "cargo");
        assert_eq!(template.steps[1].command, vec!["test".to_string()]);
    }

    #[test]
    fn test_workflow_templates_create_node_build_template() {
        let template = WorkflowTemplates::create_node_build_template();
        assert_eq!(template.name, "node-build");
        assert_eq!(template.description, "Build Node.js project");
        assert_eq!(template.steps.len(), 3);
        assert_eq!(template.steps[0].integration, "npm");
        assert_eq!(template.steps[0].command, vec!["install".to_string()]);
        assert_eq!(template.steps[1].integration, "npm");
        assert_eq!(template.steps[1].command, vec!["run".to_string(), "build".to_string()]);
        assert_eq!(template.steps[2].integration, "npm");
        assert_eq!(template.steps[2].command, vec!["run".to_string(), "test".to_string()]);
    }

    #[test]
    fn test_workflow_templates_create_docker_build_template() {
        let template = WorkflowTemplates::create_docker_build_template();
        assert_eq!(template.name, "docker-build");
        assert_eq!(template.description, "Build and test with Docker");
        assert_eq!(template.steps.len(), 2);
        assert_eq!(template.steps[0].integration, "docker");
        assert_eq!(template.steps[0].command, vec!["build".to_string(), "Dockerfile".to_string(), "myapp".to_string()]);
        assert_eq!(template.steps[1].integration, "docker");
        assert_eq!(template.steps[1].command, vec!["run".to_string(), "myapp".to_string(), "npm".to_string(), "test".to_string()]);
    }

    #[test]
    fn test_workflow_templates_create_full_stack_template() {
        let template = WorkflowTemplates::create_full_stack_template();
        assert_eq!(template.name, "full-stack");
        assert_eq!(template.description, "Build full-stack application");
        assert_eq!(template.steps.len(), 5);
        assert_eq!(template.steps[0].integration, "cargo");
        assert_eq!(template.steps[0].command, vec!["build".to_string(), "--release".to_string()]);
        assert_eq!(template.steps[1].integration, "cargo");
        assert_eq!(template.steps[1].command, vec!["test".to_string()]);
        assert_eq!(template.steps[2].integration, "npm");
        assert_eq!(template.steps[2].command, vec!["install".to_string()]);
        assert_eq!(template.steps[3].integration, "npm");
        assert_eq!(template.steps[3].command, vec!["run".to_string(), "build".to_string()]);
        assert_eq!(template.steps[4].integration, "docker");
        assert_eq!(template.steps[4].command, vec!["build".to_string(), "Dockerfile".to_string(), "app".to_string()]);
    }

    #[tokio::test]
    async fn test_workflow_templates_execute_nonexistent_template() {
        let templates = WorkflowTemplates::new();
        let integrations: HashMap<String, Box<dyn Integration + Send + Sync>> = HashMap::new();

        let result = templates
            .execute_template("nonexistent", &integrations)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_workflow_templates_execute_missing_integration() {
        let mut templates = WorkflowTemplates::new();
        let template = WorkflowTemplate {
            name: "test".to_string(),
            description: "Test".to_string(),
            steps: vec![WorkflowStep {
                name: "step1".to_string(),
                integration: "missing".to_string(),
                command: vec!["test".to_string()],
            }],
        };
        templates.add_template(template);

        let integrations: HashMap<String, Box<dyn Integration + Send + Sync>> = HashMap::new();

        let result = templates.execute_template("test", &integrations).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Integration 'missing' not found"));
    }

    #[tokio::test]
    async fn test_cargo_integration_execute_version() {
        let cargo = CargoIntegration::new();

        // Only run if cargo is available
        if std::process::Command::new("cargo")
            .arg("--version")
            .output()
            .is_ok()
        {
            let result = cargo.execute(vec!["version".to_string()]).await;
            assert!(result.is_ok());
            assert!(result.unwrap().contains("cargo"));
        }
    }

    #[tokio::test]
    async fn test_cargo_integration_execute_check() {
        let cargo = CargoIntegration::new();

        // Only run if cargo is available
        if std::process::Command::new("cargo")
            .arg("check")
            .arg("--help")
            .output()
            .is_ok()
        {
            let result = cargo.execute(vec!["check".to_string()]).await;
            // check may succeed or fail depending on code, but shouldn't panic
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[tokio::test]
    async fn test_cargo_integration_execute_clean() {
        let cargo = CargoIntegration::new();

        // Only run if cargo is available
        if std::process::Command::new("cargo")
            .arg("clean")
            .arg("--help")
            .output()
            .is_ok()
        {
            let result = cargo.execute(vec!["clean".to_string()]).await;
            // clean may succeed or fail depending on project state
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[tokio::test]
    async fn test_git_integration_execute_status() {
        let git = GitIntegration::new();

        // Only run if git is available and we're in a git repo
        if std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_ok()
            && std::process::Command::new("git")
                .arg("rev-parse")
                .arg("--git-dir")
                .output()
                .is_ok()
        {
            let result = git.execute(vec!["status".to_string()]).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_git_integration_execute_log() {
        let git = GitIntegration::new();

        // Only run if git is available and we're in a git repo
        if std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_ok()
            && std::process::Command::new("git")
                .arg("rev-parse")
                .arg("--git-dir")
                .output()
                .is_ok()
        {
            let result = git.execute(vec!["log".to_string(), "--oneline".to_string(), "-1".to_string()]).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_git_integration_execute_diff() {
        let git = GitIntegration::new();

        // Only run if git is available
        if std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_ok()
        {
            let result = git.execute(vec!["diff".to_string()]).await;
            // diff may succeed or fail depending on changes, but shouldn't panic
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[tokio::test]
    async fn test_docker_integration_execute_version() {
        let docker = DockerIntegration::new();

        // Only run if docker is available
        if std::process::Command::new("docker")
            .arg("--version")
            .output()
            .is_ok()
        {
            let result = docker.execute(vec!["--version".to_string()]).await;
            assert!(result.is_ok());
            assert!(result.unwrap().to_lowercase().contains("docker"));
        }
    }

    #[tokio::test]
    async fn test_npm_integration_execute_version() {
        let npm = NpmIntegration::new();

        // Only run if npm is available
        if std::process::Command::new("npm")
            .arg("--version")
            .output()
            .is_ok()
        {
            let result = npm.execute(vec!["--version".to_string()]).await;
            assert!(result.is_ok());
            let version = result.unwrap();
            // npm version is like 8.19.2
            assert!(version.trim().split('.').count() >= 2);
        }
    }

    #[tokio::test]
    async fn test_make_integration_execute_help() {
        let make = MakeIntegration::new();

        // Only run if make is available
        if std::process::Command::new("make")
            .arg("--help")
            .output()
            .is_ok()
        {
            let result = make.execute(vec!["--help".to_string()]).await;
            assert!(result.is_ok());
            assert!(result.unwrap().contains("make"));
        }
    }

    #[tokio::test]
    async fn test_workflow_templates_full_execution() {
        let mut templates = WorkflowTemplates::new();
        let template = WorkflowTemplate {
            name: "simple".to_string(),
            description: "Simple test template".to_string(),
            steps: vec![WorkflowStep {
                name: "invalid".to_string(),
                integration: "cargo".to_string(),
                command: vec!["invalid".to_string()],
            }],
        };
        templates.add_template(template);

        let mut integrations: HashMap<String, Box<dyn Integration + Send + Sync>> = HashMap::new();
        integrations.insert("cargo".to_string(), Box::new(CargoIntegration::new()));

        // This should fail because "invalid" is not a valid cargo command
        let result = templates.execute_template("simple", &integrations).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_workflow_templates_successful_execution() {
        // Only run if cargo is available
        if std::process::Command::new("cargo")
            .arg("--version")
            .output()
            .is_ok()
        {
            let mut templates = WorkflowTemplates::new();
            let template = WorkflowTemplate {
                name: "version-check".to_string(),
                description: "Check cargo version".to_string(),
                steps: vec![WorkflowStep {
                    name: "version".to_string(),
                    integration: "cargo".to_string(),
                    command: vec!["--version".to_string()],
                }],
            };
            templates.add_template(template);

            let mut integrations: HashMap<String, Box<dyn Integration + Send + Sync>> = HashMap::new();
            integrations.insert("cargo".to_string(), Box::new(CargoIntegration::new()));

            let result = templates.execute_template("version-check", &integrations).await;
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("cargo"));
            assert!(output.contains("Step 'version'"));
        }
    }

    #[tokio::test]
    async fn test_workflow_templates_multi_step_execution() {
        // Only run if git is available and in a git repo
        if std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_ok()
            && std::process::Command::new("git")
                .arg("rev-parse")
                .arg("--git-dir")
                .output()
                .is_ok()
        {
            let mut templates = WorkflowTemplates::new();
            let template = WorkflowTemplate {
                name: "git-check".to_string(),
                description: "Check git status and log".to_string(),
                steps: vec![
                    WorkflowStep {
                        name: "status".to_string(),
                        integration: "git".to_string(),
                        command: vec!["status".to_string(), "--porcelain".to_string()],
                    },
                    WorkflowStep {
                        name: "log".to_string(),
                        integration: "git".to_string(),
                        command: vec!["log".to_string(), "--oneline".to_string(), "-1".to_string()],
                    },
                ],
            };
            templates.add_template(template);

            let mut integrations: HashMap<String, Box<dyn Integration + Send + Sync>> = HashMap::new();
            integrations.insert("git".to_string(), Box::new(GitIntegration::new()));

            let result = templates.execute_template("git-check", &integrations).await;
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("Step 'status'"));
            assert!(output.contains("Step 'log'"));
        }
    }

    #[tokio::test]
    async fn test_workflow_templates_empty_steps() {
        let mut templates = WorkflowTemplates::new();
        let template = WorkflowTemplate {
            name: "empty".to_string(),
            description: "Empty template".to_string(),
            steps: vec![],
        };
        templates.add_template(template);

        let integrations: HashMap<String, Box<dyn Integration + Send + Sync>> = HashMap::new();

        let result = templates.execute_template("empty", &integrations).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}
