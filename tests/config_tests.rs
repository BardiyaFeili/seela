use seela::config::{Config, Folders, Session, FzfConfig, expand_path, ProjectType};

#[test]
fn test_get_session_for_path() {
    let config = Config {
        folders: Folders {
            search_dirs: vec!["~/projects".to_string()],
            force_include: None,
            exclude_paths: None,
        },
        fzf: FzfConfig::default(),
        windows: vec![],
        custom_sessions: vec![
            Session {
                name: Some("Rust Project".to_string()),
                paths: Some(vec!["~/projects/rust".to_string()]),
                types: None,
                windows: vec!["editor".to_string()],
                window_focus: None,
            },
            Session {
                name: Some("Python Project".to_string()),
                paths: Some(vec!["~/projects/python".to_string()]),
                types: None,
                windows: vec!["ide".to_string()],
                window_focus: None,
            },
        ],
        default_session: Some(Session {
            name: Some("Default".to_string()),
            paths: None,
            types: None,
            windows: vec!["shell".to_string()],
            window_focus: None,
        }),
        project_types: vec![],
    };

    let rust_path = expand_path("~/projects/rust/my-rust-app");
    let rust_session = config.get_session_for_path(&rust_path).unwrap();
    assert_eq!(rust_session.windows[0], "editor");

    let python_path = expand_path("~/projects/python/my-python-app");
    let python_session = config.get_session_for_path(&python_path).unwrap();
    assert_eq!(python_session.windows[0], "ide");

    let other_path = expand_path("~/projects/other/my-other-app");
    let default_session = config.get_session_for_path(&other_path).unwrap();
    assert_eq!(default_session.windows[0], "shell");
}

#[test]
fn test_longest_prefix_matching() {
    let config = Config {
        folders: Folders {
            search_dirs: vec![],
            force_include: None,
            exclude_paths: None,
        },
        fzf: FzfConfig::default(),
        windows: vec![],
        custom_sessions: vec![
            Session {
                name: Some("Short".to_string()),
                paths: Some(vec!["~/projects".to_string()]),
                types: None,
                windows: vec!["short".to_string()],
                window_focus: None,
            },
            Session {
                name: Some("Long".to_string()),
                paths: Some(vec!["~/projects/specific".to_string()]),
                types: None,
                windows: vec!["long".to_string()],
                window_focus: None,
            },
        ],
        default_session: None,
        project_types: vec![],
    };

    let path = expand_path("~/projects/specific/my-app");
    let session = config.get_session_for_path(&path).unwrap();
    assert_eq!(session.name.as_ref().unwrap(), "Long");

    let path = expand_path("~/projects/general/my-app");
    let session = config.get_session_for_path(&path).unwrap();
    assert_eq!(session.name.as_ref().unwrap(), "Short");
}

#[test]
fn test_hierarchy_and_types() {
    use std::fs;
    let temp_dir = std::env::temp_dir().join("seela_test_hierarchy");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let project_path = temp_dir.join("my-project");
    fs::create_dir_all(&project_path).unwrap();
    fs::write(project_path.join("Cargo.toml"), "").unwrap();

    let config = Config {
        folders: Folders { search_dirs: vec![], force_include: None, exclude_paths: None },
        fzf: FzfConfig::default(),
        windows: vec![],
        project_types: vec![
            ProjectType {
                name: "rust".to_string(),
                files: vec!["Cargo.toml".to_string()],
            }
        ],
        custom_sessions: vec![
            Session {
                name: Some("Prefix Match".to_string()),
                paths: Some(vec![temp_dir.to_string_lossy().to_string()]),
                types: None,
                windows: vec!["prefix".to_string()],
                window_focus: None,
            },
            Session {
                name: Some("Type Match".to_string()),
                paths: None,
                types: Some(vec!["rust".to_string()]),
                windows: vec!["type".to_string()],
                window_focus: None,
            },
            Session {
                name: Some("Exact Match".to_string()),
                paths: Some(vec![project_path.to_string_lossy().to_string()]),
                types: None,
                windows: vec!["exact".to_string()],
                window_focus: None,
            },
        ],
        default_session: None,
    };

    // 1. Exact match should win over Type match
    let session = config.get_session_for_path(&project_path).unwrap();
    assert_eq!(session.name.as_ref().unwrap(), "Exact Match");

    // 2. Remove Exact Match from config to test Type match vs Prefix match
    let mut config2 = config.clone();
    config2.custom_sessions.pop(); // Remove Exact Match
    
    let session = config2.get_session_for_path(&project_path).unwrap();
    assert_eq!(session.name.as_ref().unwrap(), "Type Match");

    // 3. Remove Type Match to test Prefix match
    config2.custom_sessions.remove(1); // Remove Type Match
    let session = config2.get_session_for_path(&project_path).unwrap();
    assert_eq!(session.name.as_ref().unwrap(), "Prefix Match");

    fs::remove_dir_all(&temp_dir).unwrap();
}
