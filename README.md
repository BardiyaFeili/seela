# seela

A simple tmux session manager. Lets you fuzzy-find your projects,
and handles the window/pane layout based on your config.

## How to build

You can build using cargo

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

## Layout Configuration

`seela` lets you define exactly how your tmux session should look. You can set up multiple windows, complex pane splits, and even run a list of commands automatically when a pane opens.

### Sessions and Windows

A session is just a list of window names. You can also tell `seela` which window to focus on once everything is set up.

```toml
[session]
windows = ["editor", "server", "terminal"]
window_focus = "editor"
```

### Window Layouts

Each window can have a list of panes. By default, multiple top-level panes in a window are split vertically (side-by-side).

```toml
[[windows]]
name = "editor"
[[windows.panes]]
exec = ["nvim"]
```

### Deeply Nested Panes

You can nest panes as deep as you want to create any layout. Use `split = "vertical"` for side-by-side panes and `split = "horizontal"` for top-to-bottom panes.

The `split` property on a "parent" pane tells `seela` how to lay out its "children":

```toml
[[windows]]
name = "dev"

[[windows.panes]]
split = "vertical"  # The children below will be side-by-side

  [[windows.panes.panes]]
  exec = ["nvim"]   # Left side

  [[windows.panes.panes]]
  split = "horizontal" # Right side will be split top-to-bottom

    [[windows.panes.panes.panes]]
    exec = ["ls -la"] # Top right

    [[windows.panes.panes.panes]]
    exec = ["git status"] # Bottom right
```

## TODO

- [x] Implement TOML config loading (`src/config.rs`)
- [x] Recursive project discovery (look for `.git` folders)
- [x] `fzf` integration for project selection
- [x] Basic tmux session opening
- [x] Complex window/pane layout support
- [ ] Different layouts based on project types or paths
