use crate::config::Config;
use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub fn run_confirm(cmd: &str) -> Result<(), Box<dyn Error>> {
    print!("Run \"{cmd}\"? [Y/n] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    if input.is_empty() || input == "y" || input == "yes" {
        let status = Command::new("sh").arg("-c").arg(cmd).status()?;
        if !status.success() {
            eprintln!("Command exited with status: {}", status);
        }
    } else {
        println!("Skipped.");
    }

    Ok(())
}

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
                }
                continue;
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

pub fn run(config: &Config, debug: bool, headless: bool) -> Result<(), Box<dyn Error>> {
    if debug {
        println!("Loaded Config: {config:#?}");
    }

    let projects = find_projects(config);
    let project_strings = projects
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<String>>();

    if headless {
        println!("Headless mode enabled. Skipping fzf and tmux.");
        if debug {
            println!("Found {} projects", project_strings.len());
        }
        return Ok(());
    }

    if let Some(selected) = crate::fzf::select_project(&project_strings, &config.fzf)? {
        crate::tmux::open_session(Path::new(&selected), config, debug)?;
    }

    Ok(())
}
