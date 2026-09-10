# PlexSuite - Project Context

## Product and operating constraints

- PlexSuite is a portable Tauri + Svelte desktop toolkit for Plex TV libraries. The UI is English-only and provides four tools: Trash Selector, Sub Uploader, Sub Selector, and Plexmatch Generator.
- Keep portable builds; do not introduce installers. Preserve behavior parity across Windows and Linux unless a platform requires a specific flow.
- Read `AGENTS.md` for operational rules. If it conflicts with this document, follow `AGENTS.md`.
- The current `experiment/rebrand` interface is the approved production design. Do not restore the former top-tab/header layout.

## Current UI and design system

- The app uses a persistent left Plex Toolkit sidebar/tool rail for the four tools; Settings is separate in the sidebar. The main workspace contains each tool title, description, and numbered workflow guidance.
- A persistent Activity bar shows global status and includes a View logs shortcut to Settings. Pages return to the top when a tool is selected.
- Use layered graphite/dark surfaces with a Plex-inspired amber accent. Success uses a distinct green; red is reserved for destructive actions. Keep all panels dark.
- `src/app.css` is the centralized theme and layout layer. Its semantic CSS tokens cover surfaces, text, accent interaction, success, danger, warning, focus, and overlays.
- Use `ToolIcon.svelte` for navigation/workspace icons and `EmptyState.svelte` for shared empty-state presentation. SVG UI icons use `currentColor` where applicable. There are no remote fonts or unnecessary UI frameworks.
- Desktop layouts use the sidebar and responsive content grids. Narrow windows collapse the tool grids and turn navigation into a compact grid; short desktop windows can scroll the sidebar. Keyboard focus is visible and reduced-motion preferences are respected.
- Page/footer clearance is sized so a page that fits its content does not gain an unnecessary page-level vertical scrollbar.
- `static/app-icon.svg` is the editable application-icon source. Browser assets use `static/favicon.png`; Tauri desktop/window/taskbar assets are in `src-tauri/icons`.
- Tauri defaults to a 1200 x 830 window. `bundle.active` stays false for portable packaging while `bundle.icon` remains configured.

## Tool layouts and UX

- **Trash Selector:** Library scope and Purge controls occupy the left column; Dry run preview spans the right column. Long dry-run result lists are bounded and internally scrollable, use the available card height without growing the page, and episode labels use `S01E01 - Episode Title`.
- **Sub Uploader:** Library & Target and Subtitle Source are side by side, with a full-width preview below. Subtitle files are selected through a folder button, then matched to episodes; users can adjust the mapping before upload.
- **Sub Selector:** A responsive two-column layout keeps Library & Target and compact Subtitle Cleanup on the left, with available subtitle variants and Set as Default on the right. On narrow screens the columns stack.
- **Plexmatch Generator:** TMDb lookup sits above episode and video-file lists, followed by mapping, preferences, and preview. Video files can be reordered and the `.plexmatch` export always uses a save dialog and confirmation.

## Architecture and safety

- Svelte 5 components: `TrashTab.svelte`, `SubsTab.svelte`, `SubSelectorTab.svelte`, `PlexmatchTab.svelte`, and `SettingsTab.svelte`. `src/routes/+page.svelte` is the current shell and layout controller.
- `src/lib/appState.svelte.js` owns shared state and business coordination. Backend behavior, Tauri window behavior, persistence, confirmations, and tool workflows are preserved by the rebrand.
- Settings stores Plex URL/token and TMDb key next to the executable. Session logs are fed by `setStatus()`, are not persisted automatically, and can be exported as plain text. Debug mode is off by default; retention is 200 logs normally and 10,000 in Debug mode.
- Trash Selector performs a dry run before purge and uses themed confirmation/completion dialogs. Do not change purge or dry-run semantics.
- Sub Uploader uses the Plex `/library/metadata/{ratingKey}/subtitles` endpoint with a raw body and title/format parameters. Keep its current matching and upload behavior.
- Sub Selector supports exact language/region/script plus Forced and SDH variants. Set as Default changes only matching streams for the Plex user associated with the token; each media version/part is handled independently and episodes without a match remain unchanged.
- Subtitle Cleanup has unchecked-by-default Physical Sidecar, Plex Uploaded, and Unknown External categories; Unknown External is hidden at count zero. Embedded streams are always protected. Confirmation warns by category, and removal uses Plex DELETE only after revalidating scope, identity, and category for each stream.
- `scripts/diagnose-subtitle-metadata.ps1` is a PowerShell 7, GET-only development diagnostic for subtitle metadata classification.
- One process-lifetime shared `reqwest::Client` handles Plex GET, DELETE, subtitle-upload POST, and subtitle-selection PUT operations. It creates no background traffic while inactive; preserve its existing request behavior and timeouts. TMDb retains its own client creation.

## Build and cleanup

- Run `npm run tauri build` for a portable Windows build; the executable is `src-tauri/target/release/plex-suite.exe`.
- Safe generated directories to remove when needed: `src-tauri/target`, `node_modules`, `.svelte-kit`, and `build`.

## Changelog

- 2026-09-10: v1.2.0 finalization: approved PlexSuite UI/UX rebrand with persistent sidebar/tool rail, graphite/amber identity, shared icon and empty-state presentation, responsive layout improvements, Activity/status presentation, refreshed application icons, Trash Selector episode identifiers, and final scrollbar/layout polish.
- 2026-09-09: Sub Selector added exact-variant scanning, Set as Default by version/part, and safe handling for episodes without a match.
- 2026-09-09: Subtitle Cleanup finalized Physical Sidecar / Plex Uploaded / Unknown External classification, embedded-stream protection, category warnings, and Plex DELETE revalidation.
- 2026-09-09: Sub Selector desktop two-column responsive layout and compact cleanup panel stabilized.
- 2026-09-09: Logs gained optional Debug mode, text export, and timing diagnostics for Sub Selector and Trash Selector scans.
- 2026-09-09: Plex GET, DELETE, subtitle-upload POST, and subtitle-selection PUT operations moved to a shared `reqwest::Client` without changing request behavior or timeouts.
- 2026-01-13: Startup defaults to no selected library; clearing a library resets its child selections.
- 2026-01-12: Application split into Svelte 5 components with unified drag-and-drop and click-to-expand interactions for subtitle upload and Plexmatch workflows.
- 2026-01-11: Plexmatch drag reorder, mapping behavior, long-filename handling, path-depth limits, and save confirmation refined.
- 2026-01-09: Trash/Sub Uploader layouts, subtitle upload endpoint, settings logs, icon generation, and dark-theme refinements introduced.
