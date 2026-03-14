#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod robocopy;
mod windows_integration;

// Use config structure
use config::AppConfig;

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, mpsc};
use std::thread;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    // Register context menu
    let _ = windows_integration::register_context_menu();

    let ui = MainWindow::new()?;
    let ui_weak = ui.as_weak();

    // Setup global hotkeys using `global-hotkey` crate: Ctrl + Shift + R
    let manager = GlobalHotKeyManager::new().unwrap();
    let hotkey_shortcut = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyR);
    let _ = manager.register(hotkey_shortcut);

    let ui_weak_hotkey = ui.as_weak();
    thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                if event.id == hotkey_shortcut.id()
                    && event.state == global_hotkey::HotKeyState::Released
                {
                    let ui_weak_clone = ui_weak_hotkey.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak_clone.upgrade() {
                            ui.window().show().ok(); // attempt to bring to front
                        }
                    });
                }
            }
        }
    });

    // No longer mapped to MainWindow
    // The cancel logic will be mapped to the dynamically created ProgressWindow below.

    let ui_weak_save = ui.as_weak();
    ui.on_save_settings(
        move |source,
              dest,
              action_type,
              file_filter,
              threads,
              retries,
              wait_time,
              copy_empty_dirs,
              restartable| {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Save Robocopy+ Configuration")
                .add_filter("JSON", &["json"])
                .save_file()
            {
                let config = AppConfig {
                    source: source.to_string(),
                    dest: dest.to_string(),
                    action_type: action_type.to_string(),
                    file_filter: file_filter.to_string(),
                    thread_count: threads,
                    retry_count: retries,
                    wait_time: wait_time,
                    copy_empty_dirs,
                    restartable_mode: restartable,
                };
                let _ = config::save_config(&config, &path);
            }
        },
    );

    let ui_weak_load = ui.as_weak();
    ui.on_load_settings(move || {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Load Robocopy+ Configuration")
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            if let Ok(config) = config::load_config(&path) {
                if let Some(ui) = ui_weak_load.upgrade() {
                    ui.invoke_update_settings(
                        config.source.into(),
                        config.dest.into(),
                        config.action_type.into(),
                        config.file_filter.into(),
                        config.thread_count,
                        config.retry_count,
                        config.wait_time,
                        config.copy_empty_dirs,
                        config.restartable_mode,
                    );
                }
            }
        }
    });

    ui.on_start_copy(
        move |source,
              dest,
              action_type,
              file_filter,
              threads,
              retries,
              wait_time,
              copy_empty_dirs,
              restartable| {
            // Spawn a detached Progress Window
            let progress_ui = ProgressWindow::new().unwrap();
            let progress_weak = progress_ui.as_weak();

            progress_ui.set_is_running(true);
            progress_ui.set_robocopy_progress("Starting...".into());
            progress_ui.set_progress_value(0.0);
            progress_ui.set_log_text("".into());

            progress_ui.show().unwrap();

            // Create a new cancellation token for this independent copy process
            let cancel_flag = Arc::new(AtomicBool::new(false));
            let cancel_flag_inner = cancel_flag.clone();

            progress_ui.on_force_stop(move || {
                cancel_flag_inner.store(true, Ordering::Relaxed);
            });

            let (tx, rx) = mpsc::channel();

            // Spawn the robocopy execution thread
            let source_clone = source.to_string();
            let dest_clone = dest.to_string();
            let action_type_clone = action_type.to_string();
            let filter_clone = file_filter.to_string();

            thread::spawn(move || {
                robocopy::start_copy(
                    source_clone,
                    dest_clone,
                    action_type_clone,
                    filter_clone,
                    threads,
                    retries,
                    wait_time,
                    copy_empty_dirs,
                    restartable,
                    cancel_flag,
                    tx,
                );
            });

            // Spawn a thread to listen for events and update this specific ProgressWindow
            let ui_weak_clone = progress_weak.clone();
            thread::spawn(move || {
                for event in rx {
                    let ui_weak_inner = ui_weak_clone.clone();
                    match event {
                        robocopy::CopyEvent::Log(line) => {
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak_inner.upgrade() {
                                    let mut current_log = ui.get_log_text().to_string();
                                    current_log.push_str(&line);
                                    current_log.push('\n');
                                    ui.set_log_text(current_log.into());
                                }
                            });
                        }
                        robocopy::CopyEvent::Progress(prog_str) => {
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak_inner.upgrade() {
                                    ui.set_robocopy_progress(prog_str.clone().into());

                                    // parse progress string like "12.5%" or "100%"
                                    let prog_clean = prog_str.replace("%", "");
                                    if let Ok(val) = prog_clean.parse::<f32>() {
                                        ui.set_progress_value(val / 100.0);
                                    }
                                }
                            });
                        }
                        robocopy::CopyEvent::TotalProgress(copied, total) => {
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak_inner.upgrade() {
                                    ui.set_total_progress_text(
                                        format!("Total: {}/{}", copied, total).into(),
                                    );
                                    if total > 0 {
                                        ui.set_total_progress_value(copied as f32 / total as f32);
                                    }
                                }
                            });
                        }
                        robocopy::CopyEvent::Finished(success) => {
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak_inner.upgrade() {
                                    ui.set_is_running(false);
                                    if success {
                                        ui.set_robocopy_progress("Finished Successfully".into());
                                    } else {
                                        ui.set_robocopy_progress(
                                            "Finished with Errors or Cancelled".into(),
                                        );
                                    }
                                }
                            });
                            break;
                        }
                    }
                }
            });
        },
    );

    ui.run()
}
