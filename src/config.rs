use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub folders: Folders,
    #[serde(default)]
    pub fzf: FzfConfig,
}

#[derive(Debug, Deserialize)]
pub struct FzfConfig {
    #[serde(default = "defaults::preview")]
    pub preview: bool,
    #[serde(default = "defaults::preview_command")]
    pub preview_command: String,
    pub fzf_opts: Option<String>,
}

impl Default for FzfConfig {
    fn default() -> Self {
        Self {
            preview: defaults::preview(),
            preview_command: defaults::preview_command(),
            fzf_opts: None,
        }
    }
}

mod defaults {
    pub fn preview() -> bool { true }
    pub fn preview_command() -> String { "tree -C -L 2 {}".to_string() }
}

impl Config {
    pub fn load(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Folders {
    pub search_dirs: Vec<String>,
    pub force_include: Option<Vec<String>>,
    pub exclude_paths: Option<Vec<String>>,
}

pub fn get_config_path(cli_path: Option<PathBuf>) -> Option<PathBuf> {
    // CLI given path
    if let Some(path) = cli_path.filter(|p| p.exists()) {
        return Some(path);
    }

    // $SEELA_CONFIG_HOME/config.toml
    if let Ok(seela_home) = env::var("SEELA_CONFIG_HOME") {
        let path = PathBuf::from(seela_home).join("config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    // $XDG_CONFIG_HOME/seela/config.toml
    if let Ok(xdg_home) = env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_home).join("seela/config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    // ~/.config/seela/config.toml
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".config/seela/config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    None
}
