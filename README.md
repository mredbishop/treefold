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
