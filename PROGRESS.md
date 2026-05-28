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
- [x] S10 Cross-platform terminal lifecycle
- [x] S11 Error handling and permissions
- [x] S12 Polish, docs, and release checks

## Notes

- Implemented scanner with per-entry error capture and non-fatal traversal.
- Implemented list + treemap split layout (33/67) with status bar.
- Terminal lifecycle is managed via `TerminalGuard` and cleaned up on drop.
- Treemap blocks are clipped to panel bounds, tiny panel fallback message added, and fit/overlap/utilization tests added.
- Treemap labels now show container size where width allows, with tested truncation and small-block fallback.
