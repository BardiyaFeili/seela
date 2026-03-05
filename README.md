# seela

A simple tmux session manager. It scans your projects, lets you pick one via fzf,
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
```

## TODO

- [x] Implement TOML config loading (`src/config.rs`)
- [x] Recursive project discovery (look for `.git` folders)
- [x] `fzf` integration for project selection
- [x] Basic tmux session opening
- [ ] Complex window/pane layout support
