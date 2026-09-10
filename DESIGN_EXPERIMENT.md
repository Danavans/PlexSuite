# PlexSuite — Plex toolkit experiment

Implemented on `experiment/rebrand`. This document describes the experimental UI; historical orange-theme layout notes in PROJECT_CONTEXT.md describe the stable design.

## Direction

A compact media workbench: graphite surfaces, a warm amber identity with distinct success green, a geometric P mark, a persistent tool rail, and numbered workflow guides. Red is reserved for destructive actions. All UI text stays in English and every surface stays dark.

## User experience

- Four tools remain named and individually accessible; Settings sits separately in the rail.
- Each tool has a clear title, short purpose, and static workflow overview (the numbers are guidance, not progress indicators).
- Dry Run precedes Purge Trash. Upload Subtitles follows the mapping preview. Existing confirmation gates and enablement rules remain intact.
- Empty states explain the next action. Settings exposes connection feedback and session activity, including Debug mode and export.
- A persistent activity strip shows global status and provides a shortcut to logs. Navigation starts the new workspace at the top.
- Two-column layouts collapse for narrow windows; navigation becomes a compact grid below 620 px. Long names and paths wrap or truncate within their own controls. Short desktop windows can scroll the tool rail.
- Keyboard focus is visible and reduced-motion preferences are respected.

## Implementation boundaries

The frontend shell and shared CSS were replaced. Small `ToolIcon` and `EmptyState` components centralize visual presentation. There are no new dependencies or remote fonts.

The five existing tool scripts are unchanged except for the EmptyState import. The polish pass only adjusts copy, CSS tokens, spacing, an accent variant name, and icon assets. appState.svelte.js, Rust sources, HTTP behavior, persistence, Tauri window configuration are unchanged. Desktop PNG/ICO/ICNS assets and the browser favicon now match the geometric in-app logo. The editable source is static/app-icon.svg; its colors match the panel and accent tokens in src/app.css. To regenerate, run the Tauri icon command into a temporary directory and copy the existing desktop filenames into src-tauri/icons, plus 32x32.png to static/favicon.png. Portable packaging remains enabled through the existing no-installer configuration.

## Validation

- Svelte check: zero errors and warnings.
- Production frontend build and Cargo check pass.
- Portable Windows build verified through the Tauri CLI, using the already-built frontend (the local npm launcher points to a missing CLI; direct invocation of the installed npm CLI works).
- Browser smoke checks use mocked Tauri IPC only: all five screens at 390, 768, 900, 1200 and 1440 px; long filenames; exact pt-BR Forced + SDH selection payload; unchecked cleanup categories and protected embedded tracks; cancellation of destructive/upload confirmations; TMDb mapping; Debug mode and log export.
- Real Plex purge, upload and cleanup were not executed. Linux native rendering was not tested on this Windows host.

Suggested commit: `feat(ui): redesign PlexSuite as a focused media workspace`

## Theme polish

The rail uses “Plex toolkit” and “Tools”; the extra header tagline is removed. Header and workflow margins are tighter without changing responsive breakpoints. Shared semantic tokens cover surfaces, text, accent interactions, success, danger, warning, focus, and overlays; SVG UI icons retain currentColor. View logs stays as a direct path from activity status to its details. No handlers, enablement rules, workflows, APIs, or Rust sources change.
