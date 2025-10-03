use procfs::process::Process;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

fn get_child_pids(pid: u32) -> Vec<u32> {
    let mut pids = vec![pid];
    let mut to_check = vec![pid];

    while let Some(current_pid) = to_check.pop() {
        if let Ok(process) = Process::new(current_pid as i32) {
            if let Ok(tasks) = process.tasks() {
                for task in tasks {
                    if let Ok(task) = task {
                        if let Ok(stat) = task.stat() {
                            // Check if this is a child process
                            if stat.ppid == current_pid as i32 {
                                let child_pid = stat.pid as u32;
                                if !pids.contains(&child_pid) {
                                    pids.push(child_pid);
                                    to_check.push(child_pid);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pids
}

pub async fn monitor_disk(pid: u32, tx: mpsc::UnboundedSender<(u64, u64)>) {
    let mut prev_read = 0u64;
    let mut prev_write = 0u64;
    loop {
        let pids = get_child_pids(pid);
        let mut total_read = 0u64;
        let mut total_write = 0u64;

        for &current_pid in &pids {
            if let Ok(process) = Process::new(current_pid as i32) {
                if let Ok(io) = process.io() {
                    total_read += io.read_bytes;
                    total_write += io.write_bytes;
                }
            }
        }

        let read_delta = total_read.saturating_sub(prev_read);
        let write_delta = total_write.saturating_sub(prev_write);
        prev_read = total_read;
        prev_write = total_write;
        let _ = tx.send((read_delta, write_delta));
        sleep(Duration::from_secs(1)).await;
    }
}