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

If no path is supplied, `treefold` scans the current working directory.

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
- App bundle: `dist/treefold.app`

Check architecture:

```bash
file target/aarch64-apple-darwin/release/treefold
```
