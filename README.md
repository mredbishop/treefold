# treefold

`treefold` is a cross-platform terminal disk usage explorer written in Rust with `ratatui`.

## Build

```bash
cargo build
```

## Usage

```bash
cargo run -- <optional-path>
```

Default mode is GUI. If no path is supplied, GUI opens at the user home directory.

Run in TUI mode:

```bash
cargo run -- --tui <optional-path>
```

In GUI mode you can change location by:

- Typing a path and pressing `Enter` or `Scan`
- Clicking `Browse` to choose a folder

## Controls

- `Up`, `k`: move up
- `Down`, `j`: move down
- `PageUp`, `Ctrl+U`: page up
- `PageDown`, `Ctrl+D`: page down
- `g`: top
- `G`: bottom
- `Right`, `Enter`, `l`: enter directory
- `Left`, `Esc`, `h`: go to parent
- `r`: refresh scan
- `q`, `Ctrl+C`: quit

## Known Limitations

- Treemap uses a simple slice-and-dice layout, not a squarified treemap.
- Very small terminal sizes may hide labels.

## macOS Icon

The app icon is derived from the `treefold` name and stored as source artwork:

- `assets/treefold-icon.svg`

Generate macOS icon assets:

```bash
chmod +x scripts/generate_macos_icon.sh
scripts/generate_macos_icon.sh
```

This creates:

- `assets/treefold.iconset/`
- `assets/treefold.icns`

## Apple Silicon Packaging

Build a native Apple Silicon binary and `.app` bundle:

```bash
chmod +x scripts/build_apple_silicon_app.sh
scripts/build_apple_silicon_app.sh
```

Artifacts:

- Binary: `target/aarch64-apple-darwin/release/treefold`
- App bundle: `dist/treefold-tui.app`

Check architecture:

```bash
file target/aarch64-apple-darwin/release/treefold
```

## GUI (iced)

Run the GUI version:

```bash
cargo run --bin treefold-gui
```

Build GUI binary:

```bash
cargo build --release --bin treefold-gui
```

Build Apple Silicon macOS GUI app bundle:

```bash
chmod +x scripts/build_gui_apple_silicon_app.sh
scripts/build_gui_apple_silicon_app.sh
```

GUI artifact:

- App bundle: `dist/treefold.app`

Cross-platform GUI target checks (when targets are installed):

```bash
chmod +x scripts/check_gui_targets.sh
scripts/check_gui_targets.sh
```
