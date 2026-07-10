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
6. When all stories are complete, provide a final handoff that tells the user exactly how to build and run the app.

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
- [ ] S8 Treemap fit-to-panel behaviour
- [ ] S9 Treemap container size labels
- [ ] S10 Treemap small-entry aggregation block
- [ ] S11 Cross-platform terminal lifecycle
- [ ] S12 Error handling and permissions
- [ ] S13 Polish, docs, and release checks
- [ ] S14 macOS application icon generation
- [ ] S15 Apple Silicon binary packaging
- [ ] S16 Iced GUI application foundation
- [ ] S17 Cross-platform GUI packaging and release
- [ ] S18 GUI box heatmap treemap renderer
- [ ] S19 GUI heatmap interaction and polish
- [ ] S20 GUI keyboard parity and visual focus
- [ ] S21 Default binary mode selection (GUI default, TUI flag)
- [ ] S22 GUI treemap file-vs-folder visual distinction
- [ ] S23 GUI treemap hover details
- [ ] S24 GUI context menu: open location actions
- [ ] S25 GUI context menu: delete with confirmation
- [ ] S26 GUI start location and path selection UX
- [ ] S27 GUI scan-in-progress loader and interaction policy
- [ ] S28 GUI live scan progress output (current subfolder)
- [ ] S29 GUI cancel in-progress scan
- [ ] S30 GUI clear stale results on new scan start

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

## S8 Treemap fit-to-panel behaviour

### User Story

As a user, I want the treemap to use all available space in the right panel without overflowing, so the visualization remains readable at any terminal size.

### Acceptance Criteria

* Treemap rendering uses the exact inner rectangle of the right panel (after borders/padding).
* All treemap blocks are clipped to panel bounds.
* Treemap area utilization is maximized for visible non-zero entries:

  * no unintended blank rows/columns caused by rounding drift
  * no overlaps
* For tiny terminal sizes, rendering degrades gracefully:

  * still no panic
  * shows a clear fallback when blocks cannot be meaningfully drawn
* On terminal resize, treemap recomputes layout from the new panel size on the next draw.

### Tests

* Unit tests for fit logic:

  * all block rects are within the treemap panel inner rect
  * blocks do not overlap
  * sum of block area is bounded by panel area and close to it for normal sizes
* Regression test with narrow/tiny rectangles (for example `width < 8` or `height < 4`) verifies graceful fallback output.
* Resize-flow test (logic-level) verifies recomputation when dimensions change.

### Implementation Notes

* Keep using slice-and-dice if desired; add a deterministic remainder distribution strategy to reduce rounding gaps.
* Separate layout computation from rendering so fit/clipping can be tested independently.

---

## S9 Treemap container size labels

### User Story

As a user, I want each treemap container to show its size so I can read exact disk usage values without leaving the visual view.

### Acceptance Criteria

* Treemap blocks display both name and human-readable size when space allows.
* Label format is consistent (for example: `<name>  <size>`).
* Labels are truncated safely to fit container width.
* For small containers, size text is omitted gracefully instead of overflowing or corrupting borders.
* Selected container remains visually distinct even when showing size text.

### Tests

* Rendering smoke test verifies size text appears in sufficiently large treemap blocks.
* Unit test for label formatter/truncation includes:

  * large width: full name + size visible
  * medium width: truncated name + visible size
  * small width: omit size and avoid overflow
* Regression test confirms no panic on narrow/tiny blocks with size labels enabled.

### Implementation Notes

* Reuse existing `human_size` formatting for consistency with the left panel/status bar.
* Keep label composition in a small pure helper so it can be unit-tested.

---

## S10 Treemap small-entry aggregation block

### User Story

As a user, I want entries that are too small to render as meaningful individual blocks to be grouped into one block, so the treemap stays readable while still accounting for their total size.

### Acceptance Criteria

* Entries below a defined visual threshold are aggregated into a single synthetic treemap block.
* Aggregated block has a clear label (for example: `Small entries (N)` or `Other small items`) and shows total aggregated size.
* Aggregation does not lose size accounting:

  * sum of visible treemap block sizes equals sum of rendered entries (including aggregated entries)
* If no entries are below threshold, no aggregation block is shown.
* Aggregation behavior is deterministic for the same input and panel size.
* Selected-item highlighting remains stable; if selected entry is inside the aggregated set, selected styling is handled gracefully (for example: no highlight or aggregate highlight by rule, but no crash).

### Tests

* Unit test for aggregation selector:

  * small entries are grouped
  * large entries remain individual
  * aggregated size equals sum of grouped entries
* Rendering/layout test verifies aggregate block appears with expected label and size text when threshold is triggered.
* Regression test verifies no overflow/panic with many tiny files.

### Implementation Notes

* Keep aggregation as a preprocessing step before treemap layout.
* Make threshold policy explicit and testable (area-based or minimum width/height based).

---

## S11 Cross-platform terminal lifecycle

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

## S12 Error handling and permissions

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

## S13 Polish, docs, and release checks

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

## S14 macOS application icon generation

### User Story

As a macOS user, I want a distinctive application icon derived from the `treefold` name so the app looks native and recognizable in Finder and the Dock.

### Acceptance Criteria

* Repository includes source icon artwork and exported macOS icon assets.
* Icon concept is clearly based on the product name `treefold`.
* A valid `.icns` file is produced for macOS app usage.
* Asset set includes required icon sizes for macOS app icons (via `iconset` or equivalent).
* Build/documentation explains how the icon is generated or regenerated.

### Tests

* Verify icon artifact exists (for example: `assets/treefold.icns`).
* Validate iconset generation command completes successfully on macOS.
* Manual visual check confirms icon is legible at small sizes (16x16, 32x32) and crisp at larger sizes.

### Implementation Notes

* Prefer a deterministic icon-generation script checked into the repo.
* Keep source artwork editable (for example, SVG or high-resolution PNG) and generate derivatives from it.

---

## S15 Apple Silicon binary packaging

### User Story

As an Apple Silicon Mac user, I want a runnable native executable (and optionally `.app` bundle) so I can run `treefold` without a Rust toolchain.

### Acceptance Criteria

* Produce a native `aarch64-apple-darwin` release build.
* Provide packaging steps for a standalone binary and optional `.app` bundle.
* If `.app` bundle is created, it includes:

  * executable in `Contents/MacOS/`
  * `Info.plist`
  * app icon in `Contents/Resources/`
* Documentation includes install/run instructions for Apple Silicon users.
* Output artifact path(s) are documented.

### Tests

* Build test:

  * `cargo build --release --target aarch64-apple-darwin`
* Binary architecture check:

  * `file <path-to-binary>` reports `arm64` / `aarch64`.
* Manual run test on Apple Silicon:

  * launch binary (or `.app`)
  * verify TUI starts and exits cleanly with `q`.

### Implementation Notes

* Keep packaging scripts reproducible (for example in `scripts/`).
* Optional future extension: universal binary (`x86_64` + `arm64`) after Apple Silicon-native flow is stable.

---

## S16 Iced GUI application foundation

### User Story

As a user, I want a desktop GUI version of `treefold` built with `iced` so I can use the app outside the terminal on macOS and other platforms.

### Acceptance Criteria

* Add an `iced`-based desktop application target to the project.
* GUI app can:

  * choose/start from a root path (default current directory)
  * display directory entries in a navigable list
  * show a treemap-like usage visualization panel
  * navigate into child directories and back to parent
  * refresh scan
* GUI shows size values consistently with existing formatting.
* Scan errors remain non-fatal and are visible in the GUI.
* Existing core scan/state logic is reused where practical (avoid duplicating business logic).

### Tests

* Unit tests for any extracted GUI-independent adapters/view-model logic.
* Smoke test that GUI app state can be initialized from a path.
* Existing scanner/state tests continue passing.

### Implementation Notes

* Prefer sharing modules between TUI and GUI (scanner/state/layout helpers).
* Keep GUI-specific logic in separate files/modules (for example `src/gui/`).
* Start with functional layout over visual polish; polish can follow after parity.

---

## S17 Cross-platform GUI packaging and release

### User Story

As a user, I want installable/runnable GUI artifacts for Apple Silicon macOS and other major platforms so I can run `treefold` as a desktop app.

### Acceptance Criteria

* Provide documented build targets for:

  * macOS (Apple Silicon native)
  * Linux
  * Windows
* macOS GUI artifact supports app launching via `.app` bundle (or equivalent launcher workflow).
* GUI app icon is applied in packaged app where supported.
* Packaging scripts/configs are committed and reproducible.
* README includes GUI run/build/package instructions per platform.

### Tests

* macOS build test for GUI target (Apple Silicon).
* At least one CI-friendly verification per non-macOS platform path (for example compile-only checks).
* Manual launch verification checklist for macOS/Linux/Windows.

### Implementation Notes

* Keep TUI and GUI binaries separable (for example feature flags or multiple bin targets).
* Prefer standard Rust ecosystem tools for packaging where possible.
* Defer signing/notarization unless explicitly requested, but document where it would fit.

---

## S18 GUI box heatmap treemap renderer

### User Story

As a GUI user, I want the directory visualization to use a SequoiaView-style box heatmap treemap so large and small items are visually comparable by both area and color.

### Acceptance Criteria

* Replace the current GUI usage bars with a 2D box treemap panel.
* Each visible item is rendered as a rectangle:

  * area proportional to file/folder size
  * fill color based on a heat scale (for example cool-to-warm by relative size)
* Heatmap is deterministic for the same input.
* Labels are shown when space allows and hidden/truncated safely when not.
* Rendering handles:

  * empty folders
  * very small windows
  * many entries
* No overlaps or out-of-bounds rectangles in the rendered panel.

### Tests

* Unit tests for GUI treemap layout output:

  * rectangles remain within bounds
  * rectangles do not overlap
  * larger entries get larger/equal area than smaller entries (where comparable)
* Unit tests for heat color mapping:

  * lowest, mid, and highest buckets map to expected color bands
* GUI smoke test verifies heatmap view can render without panic.

### Implementation Notes

* Reuse existing treemap layout logic where possible; add GUI-specific rendering adapter.
* Keep color scale logic in a pure helper for deterministic tests.

---

## S19 GUI heatmap interaction and polish

### User Story

As a GUI user, I want to interact with heatmap boxes directly so navigation feels natural and informative.

### Acceptance Criteria

* Clicking a directory box navigates into that directory.
* Hovering or selecting a box reveals key metadata:

  * name
  * size
  * path (full or truncated with tooltip/detail panel)
* Selected box is visually distinct from unselected boxes.
* If an item is aggregated (small-entry bucket), interaction remains stable and non-crashing.
* Keyboard navigation remains available as fallback where implemented.

### Tests

* Interaction tests (or logic-level unit tests) for:

  * hit-testing box -> corresponding entry
  * click on directory updates current path
  * click on file does not break navigation state
* Regression test for resize + interaction sequence (no panic / stale selection).
* Manual verification checklist confirms expected UX on macOS/Linux/Windows.

### Implementation Notes

* Separate hit-testing from drawing so it can be unit-tested.
* Keep interaction model compatible with future zoom/animation enhancements.

---

## S20 GUI keyboard parity and visual focus

### User Story

As a GUI user, I want the same navigation keys as the TUI (arrow keys, `Enter`, `Esc`; excluding `q`) so keyboard navigation is consistent across interfaces.

### Acceptance Criteria

* GUI supports keyboard commands equivalent to TUI (excluding `q` quit):

  * `Up` and `Down`: move selection/focus through current directory entries
  * `Left` and `Esc`: go to parent directory
  * `Right` and `Enter`: enter selected directory (file selection is non-crashing/no-op)
* Focus/selection movement is visually obvious in GUI:

  * list focus highlight updates on keypress
  * heatmap selected box updates to match focused item when visible
* Keyboard navigation and mouse interaction remain synchronized:

  * clicking updates keyboard focus target
  * keyboard updates selected heatmap/list state
* `q` does not trigger quit behavior in GUI.

### Tests

* Unit tests for GUI key-to-action mapping parity (arrow keys, `Enter`, `Esc`).
* State transition tests for:

  * up/down selection movement bounds
  * enter-directory and go-parent behavior
  * enter on file is safe no-op
* GUI smoke test verifies focus highlight updates after simulated key events.
* Regression test confirms `q` key does not close or crash GUI.

### Implementation Notes

* Reuse existing navigation/state methods from shared app state where possible.
* Keep key mapping in a small pure helper for deterministic tests.

---

## S21 Default binary mode selection (GUI default, TUI flag)

### User Story

As a user, I want the compiled `treefold` binary to launch the GUI by default, with an explicit `-t` or `--tui` flag to run terminal mode when needed.

### Acceptance Criteria

* Running `treefold` with no mode flags starts GUI mode by default.
* Running `treefold -t` starts TUI mode.
* Running `treefold --tui` starts TUI mode.
* Existing optional path argument continues to work in both modes.
* `--help` clearly documents:

  * default GUI behavior
  * `-t` / `--tui` behavior
  * path argument usage
* Invalid or conflicting mode arguments produce a clear, non-zero error.

### Tests

* CLI argument parsing unit tests for:

  * default -> GUI
  * `-t` -> TUI
  * `--tui` -> TUI
  * optional path parsing in both modes
* Help output test includes GUI default and TUI flag descriptions.
* Smoke run tests:

  * `cargo run -- --help`
  * `cargo run -- -t .`
  * `cargo run -- .` (GUI default, manual verification)

### Implementation Notes

* Keep mode parsing isolated in a small parser/helper to keep entrypoint logic clean.
* Preserve existing TUI behavior and keybindings when `--tui` is used.

---

## S22 GUI treemap file-vs-folder visual distinction

### User Story

As a GUI user, I want folders and files to look visually different in the treemap so I can quickly identify structure versus content.

### Acceptance Criteria

* Treemap blocks for directories and files are visually distinct.
* Distinction remains clear in normal, hover, and selected states.
* Distinction works alongside existing heatmap coloring (does not remove size heat cues).
* Distinction remains legible with many entries and small blocks.

### Tests

* Unit tests for style mapping by entry type (file vs directory).
* GUI render smoke test verifies both style variants are emitted.
* Regression test for tiny blocks confirms no panic or unreadable artifacts.

### Implementation Notes

* Prefer consistent visual channels (for example border style + icon/marker + subtle tint).
* Keep style decisions in a pure helper for deterministic tests.

---

## S23 GUI treemap hover details

### User Story

As a GUI user, I want details when hovering over a treemap block so I can inspect an item without clicking.

### Acceptance Criteria

* Hovering a block shows a detail view (tooltip or detail panel) including:

  * name
  * type (file/folder)
  * size
  * path (full or safely truncated)
* Hover details update as cursor moves between blocks.
* Hovering empty space clears or hides hover details.
* Hover behavior does not interfere with click selection/navigation.

### Tests

* Unit tests for hit-test -> hover detail data mapping.
* GUI interaction test verifies hover enter/leave transitions.
* Regression test for resize + hover sequence (no stale hover state / no panic).

### Implementation Notes

* Separate hover state computation from rendering for easier testing.
* Reuse existing formatting helpers for size/path display consistency.

---

## S24 GUI context menu: open location actions

### User Story

As a GUI user, I want a right-click menu with context-appropriate open actions so I can jump to a location in my OS file browser.

### Acceptance Criteria

* Right-clicking a block opens a context menu.
* If target is a directory: show `Open this directory`.
* If target is a file: show `View in parent`.
* Action opens the relevant location in the platform file browser:

  * macOS Finder
  * Windows File Explorer
  * Linux default file manager
* Failures are handled with a non-crashing error message.

### Tests

* Unit tests for menu-option selection logic by entry type.
* Unit tests for OS command resolution/routing (mocked execution).
* Manual verification checklist for macOS/Linux/Windows behavior.

### Implementation Notes

* Isolate OS-launch logic behind a small adapter for testability.
* Avoid blocking UI thread while launching external file browser command.

---

## S25 GUI context menu: delete with confirmation

### User Story

As a GUI user, I want a right-click delete action with confirmation so I can safely remove files/folders from the treemap.

### Acceptance Criteria

* Context menu includes `Delete this folder` or `Delete this file` based on entry type.
* Selecting delete always shows a confirmation dialog before executing.
* Confirmation dialog includes clear target name/path and irreversible warning.
* Cancel leaves filesystem unchanged.
* Confirm executes deletion and refreshes scan/view.
* Deletion failures are surfaced as non-fatal errors.

### Tests

* Unit tests for delete-label selection by entry type.
* Unit tests for confirmation workflow state transitions (open/cancel/confirm).
* Integration-style test with `tempfile` verifies:

  * cancel preserves file/folder
  * confirm removes file/folder
* Regression test for deletion errors (permissions/path race) confirms no crash.

### Implementation Notes

* Wrap filesystem mutation in a small service layer to simplify mocking/testing.
* Consider optional future enhancement for trash/recycle-bin behavior.

---

## S26 GUI start location and path selection UX

### User Story

As a GUI user, I want the app to open in my home directory by default, while still allowing me to navigate above it or choose another path via picker or manual entry.

### Acceptance Criteria

* GUI default start path is the current user home directory (not project cwd).
* User can navigate upward beyond home directory using existing navigation controls.
* GUI provides folder picker action to choose a new root/current path.
* GUI still supports manual path entry and scan from typed path.
* Invalid typed or picked paths show a clear non-fatal error message.
* After selecting or typing a path, directory list + heatmap refresh to that location.

### Tests

* Unit test for default GUI start-path resolver returns home directory when available.
* State/navigation tests confirm parent traversal works above home.
* Unit/integration tests for manual path submission:

  * valid path loads
  * invalid path surfaces error and preserves stable state
* Folder picker flow test (logic-level or mocked callback) verifies selected path is applied.

### Implementation Notes

* Keep path-resolution and validation in a testable helper.
* Abstract folder-picker integration behind a small adapter to keep platform differences isolated.
* Preserve CLI path override behavior: explicit path argument should still take precedence over GUI default home path.

---

## S27 GUI scan-in-progress loader and interaction policy

### User Story

As a GUI user, I want a clear loader while scanning is in progress so I know the app is working and not frozen.

### Acceptance Criteria

* When a scan starts, GUI shows an explicit loading indicator (spinner/progress text/overlay).
* Loading state clearly indicates which path is being scanned.
* While scanning, interaction policy is explicit and consistent:

  * either disable conflicting actions (recommended), or
  * allow safe interactions but prevent confusing stale-state actions
* Scan completion hides loader and updates list/treemap atomically.
* Scan failure hides loader and shows non-fatal error state.
* Repeated scans (path change/refresh) do not leave loader stuck.

### Tests

* Unit tests for scan state transitions:

  * idle -> scanning -> success -> idle
  * idle -> scanning -> error -> idle
* GUI interaction test verifies loader visibility during async scan.
* Regression test ensures no stale loader after rapid scan requests.

### Implementation Notes

* Track scan lifecycle in explicit GUI state (for example `is_scanning` + `scan_path`).
* Use a request token/version to ignore stale async scan completions.
* Keep loader rendering separate from data rendering to avoid partial-update flicker.

---

## S28 GUI live scan progress output (current subfolder)

### User Story

As a GUI user, I want to see live progress showing which subfolder is currently being scanned so long scans feel transparent and trustworthy.

### Acceptance Criteria

* During scan, GUI displays active progress text with the current subfolder/path being processed.
* Progress text updates continuously as scanning advances through subfolders.
* Progress display is cleared or finalized when scan completes or fails.
* Progress updates are tied to the active scan request only (no stale updates from previous scans).
* UI remains responsive while progress updates are shown.

### Tests

* Unit tests for scan-progress state transitions:

  * start scan -> progress updates -> complete
  * start scan -> progress updates -> error
  * stale progress events are ignored when request id no longer matches
* GUI logic test verifies progress text changes when progress events are received.
* Regression test ensures rapid scan restarts do not interleave progress output incorrectly.

### Implementation Notes

* Introduce scanner progress callback/event channel (for example periodic message with current path).
* Keep progress events lightweight and throttled if needed to avoid UI flooding.
* Preserve non-fatal error behavior and request-token guard for async scans.

---

## S29 GUI cancel in-progress scan

### User Story

As a GUI user, I want to stop an active scan so I can quickly correct path mistakes or avoid waiting for a long scan I no longer need.

### Acceptance Criteria

* GUI provides a clear `Stop`/`Cancel scan` action while scanning is active.
* Triggering cancel stops further progress updates from that scan and returns UI to idle scan state.
* Canceled scan results are never applied to list/treemap (no stale completion overwrite).
* Cancel is safe to invoke repeatedly and does not panic.
* User can start a new scan immediately after cancel.

### Tests

* Unit tests for scan lifecycle transitions:

  * scanning -> canceled -> idle
  * canceled scan completion event is ignored
* Regression test for rapid `scan -> cancel -> rescan` ensures only latest request updates UI.
* GUI logic test verifies cancel control is visible only during active scan.

### Implementation Notes

* Use request-id/token invalidation so canceled tasks cannot mutate active state.
* Prefer cooperative cancel behavior (ignore stale results/progress) unless hard cancellation is practical.
* Ensure loader/progress text clears consistently on cancel.

---

## S30 GUI clear stale results on new scan start

### User Story

As a GUI user, I want list and treemap to clear when starting a new scan so I do not mistake old results for the new target path.

### Acceptance Criteria

* Starting a new scan clears current list and treemap immediately.
* UI shows loading/progress state instead of previous folder contents during scan.
* On scan success, fresh results replace cleared state.
* On scan failure/cancel, stale prior results remain cleared and UI shows appropriate error/idle state.
* Behavior is consistent for startup scan, manual path scan, refresh, and browse-selected path scan.

### Tests

* GUI state test verifies `state` is cleared at scan start before completion.
* Regression test ensures previous folder entries are not visible during active scan.
* Scan-failure test verifies stale content does not reappear after failed new scan.

### Implementation Notes

* Clear view-model data at scan initiation, not at completion.
* Keep error rendering separate from content rendering to avoid fallback to previous state.
* Pair with request-id guard to avoid out-of-order result rehydration.

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
* Final response includes explicit user-facing commands to:

  * build the app
  * run the TUI app
  * run the GUI app (if present)
  * run Apple Silicon packaging scripts (if present)

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

Then provide a final user handoff section with exact commands, for example:

```bash
# Build
cargo build

# Run TUI
cargo run -- .

# Run GUI
cargo run --bin treefold-gui

# Apple Silicon packaging
scripts/build_apple_silicon_app.sh
scripts/build_gui_apple_silicon_app.sh
```
