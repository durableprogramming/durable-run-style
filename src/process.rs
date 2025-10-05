use std::io::Write;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

pub struct ProcessManager {
    pub child: tokio::process::Child,
    pub pid: u32,
    pub ppid: u32,
}

impl ProcessManager {
    pub fn new(command: &[String]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut child_cmd = Command::new("sh");
        child_cmd.args(["-c", &command.join(" ")]);
        child_cmd.stdout(std::process::Stdio::piped());
        child_cmd.stderr(std::process::Stdio::piped());
        let child = child_cmd.spawn()?;
        let pid = child.id().unwrap_or(0);
        let ppid = std::process::id();

        Ok(ProcessManager { child, pid, ppid })
    }

    pub fn start_output_reading(
        &mut self,
        log_file: Option<Arc<std::sync::Mutex<std::fs::File>>>,
    ) -> mpsc::UnboundedReceiver<String> {
        let (tx, rx) = mpsc::unbounded_channel();
        if let Some(stdout) = self.child.stdout.take() {
            let tx = tx.clone();
            let log_file = log_file.clone();
            tokio::spawn(async move {
                let mut reader = tokio_stream::wrappers::LinesStream::new(
                    tokio::io::BufReader::new(stdout).lines()
                );
                while let Some(line) = reader.next().await {
                    if let Ok(line) = line {
                        if let Some(file) = &log_file {
                            let mut file = file.lock().unwrap();
                            let _ = std::io::Write::write_all(&mut *file, format!("{line}\n").as_bytes());
                            let _ = file.flush();
                        }
                        let _ = tx.send(line);
                    }
                }
            });
        }

        if let Some(stderr) = self.child.stderr.take() {
            let tx = tx.clone();
            let log_file = log_file.clone();
            tokio::spawn(async move {
                let mut reader = tokio_stream::wrappers::LinesStream::new(
                    tokio::io::BufReader::new(stderr).lines()
                );
                while let Some(line) = reader.next().await {
                    if let Ok(line) = line {
                        if let Some(file) = &log_file {
                            let mut file = file.lock().unwrap();
                            let _ = std::io::Write::write_all(&mut *file, format!("{line}\n").as_bytes());
                            let _ = file.flush();
                        }
                        let _ = tx.send(line);
                    }
                }
            });
        }

        rx
    }

    pub async fn kill(&mut self) -> Result<(), std::io::Error> {
        self.child.kill().await
    }

    pub async fn wait(&mut self) -> Result<std::process::ExitStatus, std::io::Error> {
        self.child.wait().await
    }

    pub fn try_wait(&mut self) -> Result<Option<std::process::ExitStatus>, std::io::Error> {
        self.child.try_wait()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_logging_to_file() {
        // Create a temporary file for logging
        let temp_file = NamedTempFile::new().unwrap();
        let log_path = temp_file.path().to_path_buf();

        // Create a simple command that outputs to stdout
        let mut process_manager = ProcessManager::new(&["echo".to_string(), "test output".to_string()]).unwrap();

        // Open the log file
        let log_file = Some(Arc::new(std::sync::Mutex::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .unwrap()
        )));

        // Start output reading with logging
        let _rx = process_manager.start_output_reading(log_file);

        // Wait for the process to finish
        let _status = process_manager.wait().await.unwrap();

        // Check that the log file contains the output
        let log_contents = fs::read_to_string(&log_path).unwrap();
        assert!(log_contents.contains("test output"));
    }
}
