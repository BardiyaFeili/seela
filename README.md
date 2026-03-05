# seela

A simple tmux session manager. Lets you fuzzy-find your projects,
and handles the window/pane layout based on your config.

## How to build

Just use cargo:

```bash
cargo build --release
```

## Usage

Run the binary to start searching for projects:

```bash
seela
```

Or specify a custom config path:

```bash
seela --config path/to/config.toml
```

## Tmux Integration

For the best experience, you can bind `seela` to a key in your `tmux.conf` to open it in a popup:

```tmux
bind g display-popup -w 80% -h 80% -E "seela"
```

### Configuration

`seela` looks for a config file in the following order:

1. `--config` flag
2. `$SEELA_CONFIG_HOME/config.toml`
3. `$XDG_CONFIG_HOME/seela/config.toml`
4. `~/.config/seela/config.toml`

Example `config.toml`:

```toml
[folders]
search_dirs = ["~/projects", "~/work"]
exclude_paths = ["~/projects/archive"]
force_include = ["~/special-project"]

[fzf]
preview = true
preview_command = "tree -C -L 2 {}"
fzf_opts = "--height 40% --layout=reverse"
```

### FZF Configuration

| Option            | Description                            | Default                           |
| ----------------- | -------------------------------------- | --------------------------------- |
| `preview`         | Show a preview pane in `fzf`           | `true`                            |
| `preview_command` | Command used for the preview           | `"tree -C -L 2 {}"`               |
| `fzf_opts`        | Additional flags for the `fzf` command | `None` (uses `$FZF_DEFAULT_OPTS`) |

## TODO

- [x] Implement TOML config loading (`src/config.rs`)
- [x] Recursive project discovery (look for `.git` folders)
- [x] `fzf` integration for project selection
- [x] Basic tmux session opening
- [ ] Complex window/pane layout support
