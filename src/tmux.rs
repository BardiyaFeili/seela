use crate::config::{Config, Session, SplitDirection};
use std::error::Error;
use std::path::Path;
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

    if status.is_err() || !status.unwrap().success() {
        if let Some(session_config) = &config.session {
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
            let mut pane_ids = Vec::new();
            pane_ids.push(root_pane_id.clone());

            let mut last_pane_id = root_pane_id.clone();
            for _ in 1..wc.panes.len() {
                let mut cmd = Command::new("tmux");
                cmd.arg("split-window")
                    .arg("-h")
                    .arg("-t")
                    .arg(&last_pane_id)
                    .arg("-c")
                    .arg(path.to_string_lossy().as_ref())
                    .arg("-P")
                    .arg("-F")
                    .arg("#{pane_id}");
                let new_id = get_command_output(cmd, debug)?;
                pane_ids.push(new_id.clone());
                last_pane_id = new_id;
            }

            for (i, pane_config) in wc.panes.iter().enumerate() {
                setup_pane(&pane_ids[i], pane_config, path, debug, &mut exec_tasks)?;
            }
        }
    }
    // Run all execution tasks after layout is complete
    if !exec_tasks.is_empty() {
        if debug {
            println!("Executing {} command tasks...", exec_tasks.len());
        }
        thread::sleep(Duration::from_millis(500)); // Wait for shells to be fully ready

        for task in exec_tasks {
            for exec_cmd in task.commands {
                // 1. Clear current line
                let mut clear_cmd = Command::new("tmux");
                clear_cmd
                    .arg("send-keys")
                    .arg("-t")
                    .arg(&task.pane_id)
                    .arg("C-u");
                clear_cmd.status()?;
                thread::sleep(Duration::from_millis(50));

                // 2. Send command literally
                let mut run_cmd = Command::new("tmux");
                run_cmd
                    .arg("send-keys")
                    .arg("-t")
                    .arg(&task.pane_id)
                    .arg("-l")
                    .arg(&exec_cmd);
                run_cmd.status()?;
                thread::sleep(Duration::from_millis(50));

                // 3. Execute
                let mut enter_cmd = Command::new("tmux");
                enter_cmd
                    .arg("send-keys")
                    .arg("-t")
                    .arg(&task.pane_id)
                    .arg("C-m");

                if debug {
                    println!("Running: {exec_cmd} in {pane_id}", pane_id = task.pane_id);
                }
                enter_cmd.status()?;

                thread::sleep(Duration::from_millis(150));
            }
        }
    }

    // Focus the requested window if specified
    if let Some(focus_name) = &session_config.window_focus {
        if debug {
            println!("Focusing window: {focus_name}");
        }
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
) -> Result<(), Box<dyn Error>> {
    if !config.panes.is_empty() {
        let mut sub_pane_ids = Vec::new();
        sub_pane_ids.push(pane_id.to_string());

        let split_arg = match config.split {
            Some(SplitDirection::Vertical) => "-h",
            _ => "-v",
        };

        let mut last_id = pane_id.to_string();
        for _ in 1..config.panes.len() {
            let mut cmd = Command::new("tmux");
            cmd.arg("split-window")
                .arg(split_arg)
                .arg("-t")
                .arg(&last_id)
                .arg("-c")
                .arg(path.to_string_lossy().as_ref())
                .arg("-P")
                .arg("-F")
                .arg("#{pane_id}");
            let new_id = get_command_output(cmd, debug)?;
            sub_pane_ids.push(new_id.clone());
            last_id = new_id;
        }

        for (i, sub_pane_config) in config.panes.iter().enumerate() {
            setup_pane(&sub_pane_ids[i], sub_pane_config, path, debug, exec_tasks)?;
        }
    } else if let Some(execs) = &config.exec
        && !execs.is_empty()
    {
        exec_tasks.push(ExecTask {
            pane_id: pane_id.to_string(),
            commands: execs.clone(),
        });
    }

    Ok(())
}
