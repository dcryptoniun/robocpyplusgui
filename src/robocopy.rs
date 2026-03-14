use std::io::{BufReader, Read};
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread;

pub enum CopyEvent {
    Log(String),
    Progress(String),
    TotalProgress(usize, usize), // (copied, total)
    Finished(bool),              // true if success
}

pub fn start_copy(
    source: String,
    dest: String,
    action_type: String,
    file_filter: String,
    threads: i32,
    retries: i32,
    wait_time: i32,
    copy_empty_dirs: bool,
    restartable: bool,
    cancel_flag: Arc<AtomicBool>,
    tx: Sender<CopyEvent>,
) -> std::thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut args = vec![source.clone(), dest.clone(), file_filter.clone()];

        if copy_empty_dirs {
            args.push("/E".to_string()); // Copy subdirectories, including empty ones
        } else {
            args.push("/S".to_string()); // Copy subdirectories, excluding empty ones
        }

        if restartable {
            args.push("/Z".to_string()); // Restartable mode
        }

        args.push(format!("/MT:{}", threads));
        args.push(format!("/R:{}", retries));
        args.push(format!("/W:{}", wait_time));
        args.push("/BYTES".to_string()); // Print sizes as bytes

        if action_type == "Mirror (Sync)" {
            args.push("/MIR".to_string());
        } else if action_type == "Move (Cut)" {
            args.push("/MOVE".to_string());
        }

        let mut child = Command::new("robocopy")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // Prevent showing a console window on Windows
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .expect("Failed to start robocopy");

        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);

        let re = regex::Regex::new(r"(\d{1,3}(\.\d+)?)%").unwrap();

        let mut buffer = [0; 1024];
        let mut line_buf = String::new();
        let mut files_copied = 0;
        let mut total_files = 0;
        let total_re = regex::Regex::new(r"Files :\s+(\d+)\s+").unwrap();

        loop {
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = child.kill();
                let _ = tx.send(CopyEvent::Log(
                    "Copy operation forcefully stopped.".to_string(),
                ));
                let _ = tx.send(CopyEvent::Finished(false));
                return;
            }

            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buffer[..n]);
                    for c in text.chars() {
                        if c == '\n' || c == '\r' {
                            if !line_buf.trim().is_empty() {
                                let trimmed = line_buf.trim();

                                // Parse total files from initial scan header
                                if let Some(caps) = total_re.captures(trimmed) {
                                    if let Some(matched) = caps.get(1) {
                                        if let Ok(total) = matched.as_str().parse::<usize>() {
                                            total_files = total;
                                            let _ = tx.send(CopyEvent::TotalProgress(
                                                files_copied,
                                                total_files,
                                            ));
                                        }
                                    }
                                }

                                // Check for file percentage progress
                                if let Some(caps) = re.captures(trimmed) {
                                    if let Some(matched) = caps.get(0) {
                                        let _ = tx.send(CopyEvent::Progress(
                                            matched.as_str().to_string(),
                                        ));
                                    }
                                } else {
                                    // Identify a file actually being processed
                                    if !trimmed.starts_with("----------------")
                                        && !trimmed.starts_with("Total")
                                        && !trimmed.starts_with("Dirs")
                                        && trimmed.contains("\t")
                                    {
                                        files_copied += 1;
                                        let _ = tx.send(CopyEvent::TotalProgress(
                                            files_copied,
                                            total_files,
                                        ));
                                    }
                                    let log_msg =
                                        format!("{} (Files Processed: ~{})", trimmed, files_copied);
                                    let _ = tx.send(CopyEvent::Log(log_msg));
                                }
                            }
                            line_buf.clear();
                        } else {
                            line_buf.push(c);
                        }
                    }
                }
                Err(_) => break,
            }
        }

        let status = child.wait().expect("Failed to wait on robocopy");

        // Robocopy exit codes >= 8 indicate failure
        let code = status.code().unwrap_or(16);
        let success = code < 8;

        let _ = tx.send(CopyEvent::Finished(success));
    })
}
