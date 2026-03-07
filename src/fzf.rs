use crate::config::FzfConfig;
use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn select_project(
    projects: &[String],
    config: &FzfConfig,
) -> Result<Option<String>, Box<dyn Error>> {
    let mut cmd = Command::new("fzf");

    if config.preview {
        cmd.arg("--preview").arg(&config.preview_command);
    }

    if let Some(opts) = &config.fzf_opts {
        for opt in opts.split_whitespace() {
            cmd.arg(opt);
        }
    }

    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

    let mut stdin = child.stdin.take().ok_or("Failed to open fzf stdin")?;
    for project in projects {
        writeln!(stdin, "{}", project)?;
    }
    drop(stdin);

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Ok(None); // User cancelled fzf
    }

    let selected = String::from_utf8(output.stdout)?.trim().to_string();
    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected))
    }
}
