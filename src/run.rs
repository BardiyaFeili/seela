use crate::config::Config;
use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

/// Expand ~ and return an absolute path
fn expand_path(path: &str) -> PathBuf {
    let expanded = shellexpand::tilde(path);
    PathBuf::from(expanded.to_string())
}

pub fn find_projects(config: &Config) -> Vec<PathBuf> {
    let mut projects = Vec::new();

    let search_dirs: Vec<PathBuf> = config
        .folders
        .search_dirs
        .iter()
        .map(|s| expand_path(s))
        .collect();
    let exclude_paths: Vec<PathBuf> = config
        .folders
        .exclude_paths
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .map(|s| expand_path(s))
        .collect();
    let force_include: Vec<PathBuf> = config
        .folders
        .force_include
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .map(|s| expand_path(s))
        .collect();

    // 1. Add force_include folders immediately if they exist
    for path in &force_include {
        if path.exists() && !projects.contains(path) {
            projects.push(path.clone());
        }
    }

    // 2. Search in search_dirs
    for root in &search_dirs {
        if !root.exists() {
            continue;
        }

        let mut it = WalkDir::new(root).into_iter();

        loop {
            let entry = match it.next() {
                None => break,
                Some(Ok(entry)) => entry,
                Some(Err(_)) => continue,
            };

            let path = entry.path();
            if !entry.file_type().is_dir() {
                continue;
            }

            let mut longest_rule_len = 0;
            let mut is_excluded = false;

            for ex in &exclude_paths {
                if path.starts_with(ex) && ex.as_os_str().len() > longest_rule_len {
                    longest_rule_len = ex.as_os_str().len();
                    is_excluded = true;
                }
            }

            for s in &search_dirs {
                if path.starts_with(s) && s.as_os_str().len() >= longest_rule_len {
                    longest_rule_len = s.as_os_str().len();
                    is_excluded = false;
                }
            }

            if is_excluded {
                let is_parent_of_search = search_dirs.iter().any(|s| s.starts_with(path));
                if !is_parent_of_search {
                    it.skip_current_dir();
                    continue;
                }
            }

            if path.join(".git").exists() {
                if !projects.contains(&path.to_path_buf()) {
                    projects.push(path.to_path_buf());
                }
                it.skip_current_dir();
                continue;
            }
        }
    }

    projects
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let projects = find_projects(&config);
    let input = projects
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<String>>()
        .join("\n");

    // Spawn fzf
    let mut child = Command::new("fzf")
        .arg("--preview")
        .arg("tree -C -L 2 {}")
        // .arg("--bind")
        // .arg("ctrl-j:down,ctrl-k:up")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Feed projects to fzf
    let mut stdin = child.stdin.take().ok_or("Failed to open fzf stdin")?;
    stdin.write_all(input.as_bytes())?;
    drop(stdin);

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Ok(()); // User cancelled fzf
    }

    let selected = String::from_utf8(output.stdout)?.trim().to_string();
    if !selected.is_empty() {
        open_in_tmux(Path::new(&selected))?;
    }

    Ok(())
}

fn open_in_tmux(path: &Path) -> Result<(), Box<dyn Error>> {
    let session_name = path
        .file_name()
        .ok_or("Could not get directory name")?
        .to_string_lossy()
        .replace('.', "_");

    // Create session if it doesn't exist
    let status = Command::new("tmux")
        .arg("has-session")
        .arg("-t")
        .arg(&session_name)
        .status();

    if status.is_err() || !status.unwrap().success() {
        Command::new("tmux")
            .arg("new-session")
            .arg("-d")
            .arg("-s")
            .arg(&session_name)
            .arg("-c")
            .arg(path.to_string_lossy().as_ref())
            .status()?;
    }

    // Switch or attach
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
