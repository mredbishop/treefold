# Agent Build Plan: Rust/Ratatui Cross-Platform Disk Usage TUI called "treefold"

## Objective

Build a cross-platform terminal disk usage tool in Rust using `ratatui`.

The application must scan disk usage, display the current directory contents as a navigable list on the left third of the terminal, and render a treemap of the current directory on the right two thirds.

The agent must continuously implement, test, and iterate until all user stories are complete.

---

## Execution Rules

You are starting in an empty directory.

Work story-by-story.

After completing each story:

1. Run all checks.
2. Update `PROGRESS.md`.
3. Commit the completed work if git is available.
4. Select the next incomplete story.
5. Continue until every story is complete.

If restarted, read `PROGRESS.md` first and resume from the next incomplete story.

Do not skip tests. If a test is impractical, write the smallest testable unit around the behaviour instead.

---

## Required Stack

* Rust stable
* `ratatui`
* `crossterm`
* `walkdir`
* `anyhow`
* `thiserror`
* `unicode-width`
* `tempfile` for tests
* Optional: `insta` for snapshots

---

## Initial Repository Setup

Create:

```text
.
├── Cargo.toml
├── README.md
├── PROGRESS.md
├── src
│   ├── main.rs
│   ├── app.rs
│   ├── fs_scan.rs
│   ├── layout.rs
│   ├── input.rs
│   ├── treemap.rs
│   ├── ui.rs
│   └── state.rs
└── tests
    └── integration.rs
```

Initialise:

```bash
cargo init --bin
cargo add ratatui crossterm walkdir anyhow thiserror unicode-width
cargo add --dev tempfile
```

Create `PROGRESS.md`:

```md
# Progress

## Current Story

None

## Completed Stories

- [ ] S1 Project scaffolding
- [ ] S2 Filesystem scanner
- [ ] S3 App state and navigation
- [ ] S4 Input handling
- [ ] S5 Directory list panel
- [ ] S6 Treemap layout algorithm
- [ ] S7 Treemap rendering
- [ ] S8 Cross-platform terminal lifecycle
- [ ] S9 Error handling and permissions
- [ ] S10 Polish, docs, and release checks

## Notes

No notes yet.
```

---

## Global Acceptance Criteria

The final product must:

* Build with `cargo build`.
* Pass `cargo test`.
* Pass `cargo clippy -- -D warnings`.
* Pass `cargo fmt --check`.
* Run with `cargo run -- <path>`.
* Default to current working directory when no path is provided.
* Work on Linux, macOS, and Windows.
* Allow navigation:

  * Down list: `Down`, `j`
  * Up list: `Up`, `k`
  * Enter selected directory: `Right`, `Enter`, `l`
  * Move to parent directory: `Left`, `Esc`, `h`
  * Quit: `q`, `Ctrl+C`
  * Refresh scan: `r`
  * Page down: `PageDown`, `Ctrl+D`
  * Page up: `PageUp`, `Ctrl+U`
  * Top: `g`
  * Bottom: `G`
* Render:

  * Left 1/3: scrollable directory/file list for current directory.
  * Right 2/3: treemap of current directory.
* Show file/folder sizes.
* Highlight selected list item.
* Keep selection valid after navigation.
* Handle unreadable files/directories without crashing.

---

# Stories

## S1 Project scaffolding

### User Story

As a developer, I want a clean Rust project structure so the application can be built, tested, and extended incrementally.

### Acceptance Criteria

* Project builds.
* Modules exist.
* `main.rs` calls into an app entrypoint.
* `README.md` explains the project.
* `PROGRESS.md` exists and tracks story completion.

### Tests

* `cargo build`
* `cargo test`
* `cargo fmt --check`

### Implementation Notes

Create a minimal app that parses an optional root path argument and exits cleanly.

---

## S2 Filesystem scanner

### User Story

As a user, I want the tool to calculate file and directory sizes so disk usage is accurately represented.

### Acceptance Criteria

* Scanner accepts a root path.
* Scanner returns a tree of entries.
* Each node contains:

  * path
  * display name
  * size in bytes
  * kind: file or directory
  * children
  * scan errors, if any
* Directory size equals recursive sum of readable children.
* Permission errors are captured, not fatal.

### Tests

Use `tempfile` to create:

```text
root/
├── a.txt      10 bytes
├── b.txt      20 bytes
└── sub/
    └── c.txt  30 bytes
```

Expected:

* `root.size == 60`
* `sub.size == 30`
* children sorted by descending size

### Suggested Types

```rust
pub enum EntryKind {
    File,
    Directory,
}

pub struct FsEntry {
    pub path: PathBuf,
    pub name: String,
    pub kind: EntryKind,
    pub size: u64,
    pub children: Vec<FsEntry>,
    pub errors: Vec<String>,
}
```

---

## S3 App state and navigation

### User Story

As a user, I want to move into and out of folders while preserving a coherent selection state.

### Acceptance Criteria

* App state tracks:

  * root tree
  * current directory path
  * current selected index
  * scroll offset
* Entering a directory updates current directory.
* Moving up updates current directory to parent when possible.
* Selection remains in bounds.
* Moving up from root does nothing.

### Tests

* Selecting a child directory and entering it changes current directory.
* Entering a file does nothing.
* Moving up from child returns to parent.
* Moving up from root stays at root.
* Selection clamp works when current directory has fewer children.

---

## S4 Input handling

### User Story

As a user, I want conventional arrow and vim-style keybindings.

### Acceptance Criteria

Map input to actions:

```rust
pub enum Action {
    Up,
    Down,
    PageUp,
    PageDown,
    Top,
    Bottom,
    Enter,
    Back,
    Refresh,
    Quit,
    None,
}
```

Bindings:

| Key                   | Action   |
| --------------------- | -------- |
| `Up`, `k`             | Up       |
| `Down`, `j`           | Down     |
| `PageUp`, `Ctrl+U`    | PageUp   |
| `PageDown`, `Ctrl+D`  | PageDown |
| `g`                   | Top      |
| `G`                   | Bottom   |
| `Right`, `Enter`, `l` | Enter    |
| `Left`, `Esc`, `h`    | Back     |
| `r`                   | Refresh  |
| `q`, `Ctrl+C`         | Quit     |

### Tests

* Unit test every keybinding.
* Unknown keys map to `None`.

---

## S5 Directory list panel

### User Story

As a user, I want a scrollable list of the current directory contents on the left third of the terminal.

### Acceptance Criteria

* Left panel width is approximately one third of terminal width.
* List shows current directory children.
* Items show:

  * name
  * type indicator
  * human-readable size
* Selected item is highlighted.
* Scroll offset keeps selected item visible.

### Tests

* Layout split returns 1/3 and 2/3 areas.
* Human size formatting works:

  * `0 B`
  * `999 B`
  * `1.0 KiB`
  * `1.0 MiB`
* Scroll calculations keep index visible.

---

## S6 Treemap layout algorithm

### User Story

As a user, I want a proportional treemap so I can understand relative disk usage visually.

### Acceptance Criteria

* Treemap receives a rectangle and entries with sizes.
* Output contains one rectangle per visible child with non-zero size.
* Larger entries receive larger area.
* Total assigned area does not exceed input rectangle.
* Algorithm handles:

  * empty directories
  * zero-size files
  * very small terminal sizes
  * many entries

### Tests

* Single item occupies full rect.
* Two equal items split roughly equally.
* Larger item gets larger area.
* Zero-size items are omitted or assigned minimum visual treatment consistently.
* No rectangle has negative or overflowing dimensions.

### Implementation Notes

A simple slice-and-dice treemap is acceptable:

* Sort children by size descending.
* Alternate horizontal and vertical splits by depth.
* Allocate area proportional to size.
* Clamp minimum dimensions defensively.

Do not over-engineer squarified treemaps unless all stories are complete.

---

## S7 Treemap rendering

### User Story

As a user, I want the right two thirds of the screen to show a treemap of the current folder.

### Acceptance Criteria

* Right panel width is approximately two thirds.
* Treemap block exists for each file/folder that can fit.
* Blocks display truncated names where space permits.
* Selected list item is visually distinguishable in treemap if visible.
* Empty folders show a clear empty state.
* Unreadable/error entries do not crash rendering.

### Tests

* Rendering smoke test using `ratatui::backend::TestBackend`.
* Empty directory renders placeholder text.
* Selected item label appears when dimensions allow.

---

## S8 Cross-platform terminal lifecycle

### User Story

As a user, I want the app to reliably enter and leave terminal UI mode.

### Acceptance Criteria

* Enables raw mode on start.
* Enters alternate screen.
* Disables raw mode on exit.
* Leaves alternate screen on exit.
* Restores terminal after panic or error where practical.
* Supports Windows via `crossterm`.

### Tests

* Extract terminal lifecycle into small functions where possible.
* Unit-test non-terminal logic.
* Manual run test:

  * launch app
  * quit with `q`
  * verify shell is usable

---

## S9 Error handling and permissions

### User Story

As a user, I want scan errors to be visible but non-fatal.

### Acceptance Criteria

* Permission errors are stored on affected entries.
* UI shows an error marker for entries with scan errors.
* Status bar shows summary:

  * current path
  * total size
  * number of scan errors
  * key hints
* Invalid starting path produces a clear error and non-zero exit.

### Tests

* Invalid path returns an error.
* Error count aggregation works.
* Status text contains path and size.

---

## S10 Polish, docs, and release checks

### User Story

As a user, I want a usable CLI tool with documented controls.

### Acceptance Criteria

* `README.md` includes:

  * purpose
  * install/build instructions
  * usage
  * controls
  * known limitations
* `--help` works.
* Optional path argument works.
* No clippy warnings.
* Code is formatted.
* All tests pass.

### Tests

Run:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo run -- --help
cargo run -- .
```

---

# Continuous Iteration Protocol

Use this loop until complete:

```text
1. Read PROGRESS.md.
2. Find the first unchecked story.
3. Set "Current Story" to that story.
4. Implement only that story and any required refactor.
5. Add or update tests for that story.
6. Run:
   - cargo fmt
   - cargo test
   - cargo clippy -- -D warnings
7. Fix failures.
8. Repeat checks until green.
9. Mark story complete in PROGRESS.md.
10. Add notes about important implementation decisions.
11. Commit if git is available:
    git add .
    git commit -m "Complete S<N>: <story title>"
12. Continue with the next unchecked story.
```

If interrupted or restarted:

```text
1. Read PROGRESS.md.
2. If "Current Story" is incomplete, inspect working tree.
3. Run tests to establish current state.
4. Finish the current story before starting another.
5. If current story appears complete, verify acceptance criteria, then mark it complete.
```

---

# Definition of Done

The product is complete only when:

* Every story is checked in `PROGRESS.md`.
* All global acceptance criteria pass.
* Manual navigation works:

  * up/down list
  * enter directory
  * leave directory
  * vim controls
  * quit
* Treemap updates when navigating.
* Application exits cleanly.
* `README.md` accurately reflects behaviour.

---

# Final Verification Command

Run this before declaring completion:

```bash
cargo fmt --check \
  && cargo clippy -- -D warnings \
  && cargo test \
  && cargo run -- --help
```

Then manually run:

```bash
cargo run -- .
```

Verify the UI interactively.
