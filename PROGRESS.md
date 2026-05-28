# Progress

## Current Story

None

## Completed Stories

- [x] S1 Project scaffolding
- [x] S2 Filesystem scanner
- [x] S3 App state and navigation
- [x] S4 Input handling
- [x] S5 Directory list panel
- [x] S6 Treemap layout algorithm
- [x] S7 Treemap rendering
- [x] S8 Treemap fit-to-panel behaviour
- [x] S9 Treemap container size labels
- [x] S10 Treemap small-entry aggregation block
- [x] S11 Cross-platform terminal lifecycle
- [x] S12 Error handling and permissions
- [x] S13 Polish, docs, and release checks
- [x] S14 macOS application icon generation
- [x] S15 Apple Silicon binary packaging
- [x] S16 Iced GUI application foundation
- [x] S17 Cross-platform GUI packaging and release
- [x] S18 GUI box heatmap treemap renderer
- [x] S19 GUI heatmap interaction and polish
- [x] S20 GUI keyboard parity and visual focus
- [x] S21 Default binary mode selection (GUI default, TUI flag)
- [x] S22 GUI treemap file-vs-folder visual distinction
- [x] S23 GUI treemap hover details
- [x] S24 GUI context menu: open location actions
- [x] S25 GUI context menu: delete with confirmation
- [x] S26 GUI start location and path selection UX
- [x] S27 GUI scan-in-progress loader and interaction policy
- [x] S28 GUI live scan progress output (current subfolder)

## Notes

- Implemented scanner with per-entry error capture and non-fatal traversal.
- Implemented list + treemap split layout (33/67) with status bar.
- Terminal lifecycle is managed via `TerminalGuard` and cleaned up on drop.
- Treemap blocks are clipped to panel bounds, tiny panel fallback message added, and fit/overlap/utilization tests added.
- Treemap labels now show container size where width allows, with tested truncation and small-block fallback.
- Treemap now aggregates tiny entries into a single synthetic block with combined size and deterministic label.
- Added deterministic macOS icon generation script and committed source SVG + generated ICNS.
- Added Apple Silicon packaging script that builds arm64 binary and creates a `.app` bundle.
- Added `iced` GUI binary (`treefold-gui`) with path scan, entry navigation, refresh, and size-based visualization bars.
- Added GUI packaging/check scripts for Apple Silicon app bundle and optional Linux/Windows target checks.
- Replaced GUI usage bars with a SequoiaView-style heatmap canvas backed by treemap layout + heat color scaling.
- Added heatmap hit-testing and click interaction to navigate into directories from boxes.
- Added GUI keyboard parity for arrows/enter/esc (q ignored), with synchronized visual selection across list and heatmap.
- Added CLI mode parser with GUI default, plus `-t`/`--tui` to force terminal mode while preserving optional path handling.
- Added folder/file-specific treemap styling, hover detail display, and right-click context menu actions.
- Added OS file-browser integration for open/view-in-parent and delete-with-confirmation workflow with refresh.
- GUI now starts at user home by default and supports both manual path entry and folder-picker selection.
- GUI scan state now reports active progress, including the currently scanned subfolder during long scans.
