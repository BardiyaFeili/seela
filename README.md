# seela

A simple tmux session manager. Lets you fuzzy-find your projects,
and handles the window/pane layout based on your config.

## How to build

You can build using cargo

```bash
cargo build --release
```

## Usage

Run the binary

```bash
seela
```

Or specify a custom config path:

```bash
seela --config path/to/config.toml
```

### Tmux Integration

You can use `seela` in tmux like this

```tmux
bind g display-popup -w 80% -h 80% -E "seela"
```

## Configuration

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

### Layout Configuration

You can configure your tmux session's layout.
Based on the path of the project or its type.

#### Default Session

`default_session` is for projects that do not match any path or type.

```toml
[default_session]
windows = ["editor", "terminal"]
window_focus = "editor"
```

#### Custom Sessions

You can also define `custom_sessions`
Which can match a project based on the path or type.

The projects will math the session based on this order:

1. Exact Path
2. Type Match
3. Partial Path Match (The closest match will be chosen)
4. Default Session

```toml
[[custom_sessions]]
name = "Rust Development"
types = ["rust"]  # Match projects of type 'rust'
windows = ["editor", "bacon"]
window_focus = "editor"

[[custom_sessions]]
name = "Web Development"
paths = ["~/projects/web"] # Match by path prefix
types = ["web"]            # OR match by project type
windows = ["editor", "server", "logs"]
```

#### Project Types

You can define your types like this.

```toml
[[project_types]]
name = "rust"
files = ["Cargo.toml"]

[[project_types]]
name = "web"
files = ["tsconfig.json", "package.json", "node_modules"]
```

#### Window Layouts

```toml
[[windows]]
name = "editor"
[[windows.panes]]
exec = ["nvim"]
```

#### Deeply Nested Panes

The panes are nest-able Use `split = "vertical"` for side-by-side panes
and `split = "horizontal"` for top-to-bottom panes.

The `split` property on a "parent" pane tells `seela` how to lay out its
"children":

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

#### Special Operators

You can use special operators in the `exec` list to control command execution:

- **`@run <command>`**: Executes the command in the pane with several
  environment variables set:
  - `SEELA_SESSION_PATH`: Absolute path to the project.
  - `SEELA_SESSION_NAME`: Name of the tmux session.
  - `SEELA_WINDOW_NAME`: Name of the current window.
  - `SEELA_PANE_ID`: Unique ID of the current pane.
- **`@confirm <command>`**: Prompts for confirmation (`Run "command"? [Y/n]`)
  before executing.
- **`@wait <seconds>`**: Pauses execution for the specified number of seconds.
- **`@wait-milli <ms>`** (alias **`@wait-ms`**): Pauses execution for the
  specified number of milliseconds.
- **`@send-key <key>`** (alias **`@sk`**): Sends a raw key or key sequence to
  the pane (e.g., `Enter`, `Space`, `C-c`, `C-l`).

> [!NOTE]
> All panes need to be initialized before you you are attached to the session.
> This means using high `@wait` will the app just stall for that period.

## TODO

- [x] Implement TOML config loading (`src/config.rs`)
- [x] Recursive project discovery (look for `.git` folders)
- [x] `fzf` integration for project selection
- [x] Basic tmux session opening
- [x] Complex window/pane layout support
- [x] Different layouts based on project paths
- [x] Custom and inbuilt types and layout based on the type
