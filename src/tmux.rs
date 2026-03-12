use crate::config::{Config, Session, SplitDirection};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

pub fn open_session(path: &Path, config: &Config, debug: bool) -> Result<(), Box<dyn Error>> {
    let session_name = path
        .file_name()
        .ok_or("Could not get directory name")?
        .to_string_lossy()
        .replace('.', "_");

    if debug {
        println!("Opening session: {session_name}");
    }

    let status = Command::new("tmux")
        .arg("has-session")
        .arg("-t")
        .arg(&session_name)
        .stderr(std::process::Stdio::null())
        .status();

    let session_exists = match status {
        Ok(s) => s.success(),
        Err(_) => false,
    };

    if !session_exists {
        if let Some(session_config) = config.get_session_for_path(path) {
            create_session_from_config(&session_name, path, config, session_config, debug)?;
        } else {
            let mut cmd = Command::new("tmux");
            cmd.arg("new-session")
                .arg("-d")
                .arg("-s")
                .arg(&session_name)
                .arg("-c")
                .arg(path.to_string_lossy().as_ref());
            cmd.status()?;
        }
    }

    if std::env::var("TMUX").is_ok() {
        Command::new("tmux")
            .arg("switch-client")
            .arg("-t")
            .arg(&session_name)
            .status()?;
    } else {
        Command::new("tmux")
            .arg("attach-session")
            .arg("-t")
            .arg(&session_name)
            .status()?;
    }

    Ok(())
}

fn get_command_output(mut cmd: Command, debug: bool) -> Result<String, Box<dyn Error>> {
    if debug {
        println!("Executing for output: {cmd:?}");
    }
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(format!(
            "Tmux command failed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

struct ExecTask {
    pane_id: String,
    commands: Vec<String>,
    path: PathBuf,
    session_name: String,
    window_name: String,
}

fn create_session_from_config(
    session_name: &str,
    path: &Path,
    config: &Config,
    session_config: &Session,
    debug: bool,
) -> Result<(), Box<dyn Error>> {
    let mut exec_tasks = Vec::new();

    for (win_idx, window_name) in session_config.windows.iter().enumerate() {
        let window_config = config.windows.iter().find(|w| &w.name == window_name);

        let root_pane_id = if win_idx == 0 {
            let mut cmd = Command::new("tmux");
            cmd.arg("new-session")
                .arg("-d")
                .arg("-s")
                .arg(session_name)
                .arg("-n")
                .arg(window_name)
                .arg("-c")
                .arg(path.to_string_lossy().as_ref())
                .arg("-P")
                .arg("-F")
                .arg("#{pane_id}");
            get_command_output(cmd, debug)?
        } else {
            let mut cmd = Command::new("tmux");
            cmd.arg("new-window")
                .arg("-t")
                .arg(session_name)
                .arg("-n")
                .arg(window_name)
                .arg("-c")
                .arg(path.to_string_lossy().as_ref())
                .arg("-P")
                .arg("-F")
                .arg("#{pane_id}");
            get_command_output(cmd, debug)?
        };

        if let Some(wc) = window_config {
            let mut pane_ids = vec![root_pane_id.clone()];
            let ratios: Vec<f32> = wc.panes.iter().map(|p| p.ratio.unwrap_or(1.0)).collect();
            let mut remaining_ratio: f32 = ratios.iter().sum();

            let mut current_pane_id = root_pane_id.clone();
            for i in 0..wc.panes.len() - 1 {
                let next_ratios_sum: f32 = ratios[i + 1..].iter().sum();
                let percentage = (next_ratios_sum / remaining_ratio) * 100.0;

                let mut cmd = Command::new("tmux");
                cmd.arg("split-window")
                    .arg("-d") // Add -d to avoid focus flickering
                    .arg("-h")
                    .arg("-l")
                    .arg(format!("{}%", percentage.round() as u32))
                    .arg("-t")
                    .arg(&current_pane_id)
                    .arg("-c")
                    .arg(path.to_string_lossy().as_ref())
                    .arg("-P")
                    .arg("-F")
                    .arg("#{pane_id}");
                let new_id = get_command_output(cmd, debug)?;
                pane_ids.push(new_id.clone());
                current_pane_id = new_id;
                remaining_ratio = next_ratios_sum;
            }

            for (i, pane_config) in wc.panes.iter().enumerate() {
                setup_pane(
                    &pane_ids[i],
                    pane_config,
                    path,
                    debug,
                    &mut exec_tasks,
                    session_name,
                    window_name,
                )?;
            }
        }
    }

    if !exec_tasks.is_empty() {
        if debug {
            println!("Waiting {}ms for tmux to stabilize...", config.tmux.startup_delay_ms);
        }
        thread::sleep(Duration::from_millis(config.tmux.startup_delay_ms));

        let tmux_cfg = config.tmux.clone();

        thread::scope(|s| {
            for (task_idx, task) in exec_tasks.into_iter().enumerate() {
                let tmux_cfg = tmux_cfg.clone();
                s.spawn(move || {
                    // Stagger start to avoid overwhelming tmux/shell
                    thread::sleep(Duration::from_millis(task_idx as u64 * 25));

                    for (cmd_idx, exec_cmd) in task.commands.iter().enumerate() {
                        let mut final_cmd = exec_cmd.clone();
                        let trimmed = exec_cmd.trim();

                        if trimmed.is_empty() {
                            continue;
                        }

                        if let Some((keyword, val)) = trimmed.split_once(' ') {
                            match keyword {
                                "@confirm" => {
                                    if let Ok(current_exe) = std::env::current_exe() {
                                        final_cmd = format!(
                                            "{} --run-command {:?}",
                                            current_exe.display(),
                                            val
                                        );
                                    }
                                }
                                "@run" => {
                                    final_cmd = format!(
                                        "SEELA_SESSION_PATH={:?} SEELA_SESSION_NAME={:?} SEELA_WINDOW_NAME={:?} SEELA_PANE_ID={:?} {}",
                                        task.path.display(),
                                        task.session_name,
                                        task.window_name,
                                        task.pane_id,
                                        val
                                    );
                                }
                                "@wait" => {
                                    if let Ok(secs) = val.parse::<u64>() {
                                        thread::sleep(Duration::from_secs(secs));
                                        continue;
                                    }
                                }
                                "@wait-milli" | "@wait-ms" => {
                                    if let Ok(ms) = val.parse::<u64>() {
                                        thread::sleep(Duration::from_millis(ms));
                                        continue;
                                    }
                                }
                                "@send-key" | "@sk" => {
                                    thread::sleep(Duration::from_millis(tmux_cfg.key_delay_ms));
                                    let mut key_cmd = Command::new("tmux");
                                    key_cmd
                                        .arg("send-keys")
                                        .arg("-t")
                                        .arg(&task.pane_id)
                                        .arg(val);
                                    let _ = key_cmd.status();
                                    continue;
                                }
                                _ => {}
                            }
                        }

                        // Robust execution sequence
                        // 1. Reset terminal and clear current line
                        if cmd_idx == 0 {
                            let mut reset_cmd = Command::new("tmux");
                            reset_cmd.arg("send-keys").arg("-t").arg(&task.pane_id).arg("-R").arg("C-c");
                            let _ = reset_cmd.status();
                            thread::sleep(Duration::from_millis(tmux_cfg.key_delay_ms));
                        }

                        let mut clear_cmd = Command::new("tmux");
                        clear_cmd.arg("send-keys").arg("-t").arg(&task.pane_id).arg("C-u");
                        let _ = clear_cmd.status();
                        thread::sleep(Duration::from_millis(tmux_cfg.key_delay_ms));

                        // 2. Send the command literally
                        let mut run_cmd = Command::new("tmux");
                        run_cmd.arg("send-keys")
                               .arg("-t")
                               .arg(&task.pane_id)
                               .arg("-l")
                               .arg(&final_cmd);
                        let _ = run_cmd.status();
                        thread::sleep(Duration::from_millis(tmux_cfg.key_delay_ms));

                        // 3. Send Enter
                        let mut enter_cmd = Command::new("tmux");
                        enter_cmd.arg("send-keys")
                                 .arg("-t")
                                 .arg(&task.pane_id)
                                 .arg("C-m");
                        let _ = enter_cmd.status();

                        thread::sleep(Duration::from_millis(tmux_cfg.action_delay_ms));
                    }
                });
            }
        });
    }

    if let Some(focus_name) = &session_config.window_focus {
        Command::new("tmux")
            .arg("select-window")
            .arg("-t")
            .arg(format!("{session_name}:{focus_name}"))
            .status()?;
    }

    Ok(())
}

fn setup_pane(
    pane_id: &str,
    config: &crate::config::Pane,
    path: &Path,
    debug: bool,
    exec_tasks: &mut Vec<ExecTask>,
    session_name: &str,
    window_name: &str,
) -> Result<(), Box<dyn Error>> {
    if !config.panes.is_empty() {
        let mut sub_pane_ids = vec![pane_id.to_string()];

        let split_arg = match config.split {
            Some(SplitDirection::Vertical) => "-h",
            _ => "-v",
        };

        let ratios: Vec<f32> = config.panes.iter().map(|p| p.ratio.unwrap_or(1.0)).collect();
        let mut remaining_ratio: f32 = ratios.iter().sum();

        let mut current_pane_id = pane_id.to_string();
        for i in 0..config.panes.len() - 1 {
            let next_ratios_sum: f32 = ratios[i + 1..].iter().sum();
            let percentage = (next_ratios_sum / remaining_ratio) * 100.0;

            let mut cmd = Command::new("tmux");
            cmd.arg("split-window")
                .arg("-d")
                .arg(split_arg)
                .arg("-l")
                .arg(format!("{}%", percentage.round() as u32))
                .arg("-t")
                .arg(&current_pane_id)
                .arg("-c")
                .arg(path.to_string_lossy().as_ref())
                .arg("-P")
                .arg("-F")
                .arg("#{pane_id}");
            let new_id = get_command_output(cmd, debug)?;
            sub_pane_ids.push(new_id.clone());
            current_pane_id = new_id;
            remaining_ratio = next_ratios_sum;
        }

        for (i, sub_pane_config) in config.panes.iter().enumerate() {
            setup_pane(
                &sub_pane_ids[i],
                sub_pane_config,
                path,
                debug,
                exec_tasks,
                session_name,
                window_name,
            )?;
        }
    } else if let Some(execs) = &config.exec
        && !execs.is_empty()
    {
        exec_tasks.push(ExecTask {
            pane_id: pane_id.to_string(),
            commands: execs.clone(),
            path: path.to_path_buf(),
            session_name: session_name.to_string(),
            window_name: window_name.to_string(),
        });
    }

    Ok(())
}
