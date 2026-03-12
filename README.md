# seela

A simple tmux session manager. Lets you fuzzy-find your projects,
and handles the window/pane layout based on your config.

## Installation

You can install `seela` directly from [crates.io](https://crates.io/crates/seela):

```bash
cargo install seela
```

Make sure that your cargo bin directory (usually `~/.cargo/bin`) is in your `PATH`.

### Build from source

If you prefer to build from source:

```bash
git clone https://github.com/BardiyaFeili/seela.git
cd seela
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

[tmux]
# Time to wait for tmux to stabilize before sending commands (ms)
startup_delay_ms = 1000
# Delay between keys and minor actions (ms)
key_delay_ms = 100
# Delay after major actions like Enter or C-c (ms)
action_delay_ms = 200
```

> [!TIP]
> If your `exec` commands are not running correctly (e.g., being sent before the
> shell is ready), try increasing the values in the `[tmux]` section.
> Different operating systems, shells, and hardware may require different
> timings to ensure commands are processed correctly.


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
"children". You can use the `ratio` property (a float) to define proportional
sizes for panes. If omitted, panes share the available space equally.

```toml
[[windows]]
name = "dev"

[[windows.panes]]
split = "vertical"  # The children below will be side-by-side

  [[windows.panes.panes]]
  exec = ["nvim"]
  ratio = 0.7       # 70% width

  [[windows.panes.panes]]
  split = "horizontal"
  ratio = 0.3       # 30% width

    [[windows.panes.panes.panes]]
    exec = ["ls -la"]
    ratio = 0.4      # 40% height of the 30% width

    [[windows.panes.panes.panes]]
    exec = ["git status"]
    ratio = 0.6      # 60% height of the 30% width
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
> This means using high `@wait` will make the app just stall for that period.

## TODO

- [x] Implement TOML config loading (`src/config.rs`)
- [x] Recursive project discovery (look for `.git` folders)
- [x] `fzf` integration for project selection
- [x] Basic tmux session opening
- [x] Complex window/pane layout support
- [x] Different layouts based on project paths
- [x] Custom and inbuilt types and layout based on the type
