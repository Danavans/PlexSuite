# Multi-Plex-server support

Implemented on 2026-09-10. The four tool layouts and business workflows are preserved; the sidebar is the sole active-server selector.

The architecture audit followed the settings commands, shared app state, every component invocation, Plex transport helpers, and TMDb integration.

| Before this change | Finding |
| --- | --- |
| Persistence | `settings.json` next to the executable held `plex_url`, `plex_token`, `tmdb_key`, and `client_id`. |
| Save & Connect | Required Plex URL/token, saved all three credentials, then called `list_libraries`. Saving happened before the connection attempt. No TMDb validation occurred. |
| TMDb | The global key was supplied to series/episode requests. Missing keys were rejected locally; HTTP errors were reported when searches or episode loads ran. There was no dedicated test. |
| Plex transport | One process-lifetime `Lazy<reqwest::Client>` shared GET, DELETE, POST, and PUT requests and pooled keep-alive connections. URL, token, and installation headers were supplied per request. |
| Captured server state | The HTTP client retained no base URL, token, server identity, server-specific default headers, authentication session, or cookie jar. Connectivity and Plex objects lived in frontend state. |
| Retained frontend data | Shared libraries, shows, seasons, selections and request IDs; component-local trash previews/dialogs, subtitle upload mappings/dialogs, subtitle scans/variants/cleanup reviews. |
| Busy handling | Most work used a global boolean. Uploads used a local flag. A boolean could also release too early during overlapping requests. |
| Plexmatch | Only TMDb requests and local file work; it does not communicate with Plex. |

The persisted model keeps `tmdb_key` and `client_id` global and adds `plex.active_server_id` and `plex.servers`. Each profile contains `id`, `name`, `url`, and `token`. New profiles use UUIDs; duplicate display names are permitted. Renaming preserves identity. Legacy URL/token values migrate without alteration, with a stable generated profile ID. Unrelated root settings are retained. Invalid JSON is reported without replacement. Serialized writes use a synchronized temporary file followed by atomic replacement; a failed replacement preserves the previous file.

`appState.activeServer` supplies credentials centrally. `plexInvoke`/`invokePlex` override caller credentials with the active profile, count pending requests, and update connectivity. Existing whole-operation busy scopes are counted rather than represented by one boolean. Uploads now participate in that scope. Switching and profile mutations are guarded in state methods as well as disabled in the UI. The transition itself holds a lock, and new tool requests are rejected during it.

The audit covers library/show/season/episode loading, trash preview/purge, subtitle preview/upload, subtitle stream scans, setting defaults, subtitle cleanup, and profile connection tests. Multi-request workflows hold their original busy scope between requests; backend loops retain the URL/token arguments from their initial invocation.

A switch first persists the chosen ID, then replaces the active context and clears its shared state. Failed persistence leaves the original context intact. A context version recreates the three Plex-dependent components, discarding their local results and confirmations. Plexmatch is preserved. The new server is checked immediately: Plex pages load libraries; Settings/Plexmatch only perform the connection check. Libraries load on entering a Plex page when needed. No library is automatically selected, and no inactive-server polling is introduced.

The shared generic HTTP transport remains alive, including its connection pool. There is no server-specific backend session to destroy. The frontend credentials, connectivity, selections, and cached results constitute the context that is invalidated. Existing request methods and timeouts are retained. Upload request errors now strip URLs, and frontend status/log handling redacts saved tokens and keys before retaining messages. Unsaved test errors redact the draft token.

Settings now provides a compact server list and one Add/Edit form with name, URL, masked token, show/hide, Test Connection, Save, and explicit deletion confirmation. Save is local and works offline; it does not test. The first profile becomes active. Saving changed active credentials clears old results and marks connectivity Unknown until tested or selected. An active profile rename preserves its context. Testing uses the current form credentials without saving and updates saved status only when those credentials match the saved profile.

Plex Test Connection makes an authenticated GET to `library/sections` through the existing shared client and checks for a Plex `MediaContainer`. The global TMDb test makes one GET to the documented [Validate Key endpoint](https://developer.themoviedb.org/reference/authentication-validate-key), using the existing v3 API-key integration and a 15-second timeout. Both tests show success/failure. TMDb Save and Test work without any configured or reachable Plex server.

Unavailable servers remain selected; there is no failover. Deleting an inactive server leaves the active context unchanged. Deleting the active server chooses the first remaining profile and checks it. Deleting the last profile clears the active ID, credentials, connectivity, selections, and results while preserving TMDb settings.

The sidebar uses a dark, bounded, scrolling popover anchored to the connectivity area. It stays inside the window, overlays content without moving the rail, indicates the active server, closes on selection/outside click/Escape, and includes Manage servers. The selector remains displayed while operations run but cannot switch. View logs is retained.

| Changed files | Purpose |
| --- | --- |
| `src-tauri/src/settings.rs` | Profile schema, migration, validation, independent saves, atomic persistence, five new backend tests. |
| `src-tauri/src/lib.rs` | Register independent profile-save, TMDb-save, and test commands. |
| `src-tauri/src/plex.rs` | Connection test, library response validation, URL-free network errors. |
| `src-tauri/src/tmdb.rs` | Explicit API-key validation and URL-free request errors. |
| `src/lib/appState.svelte.js` | Active profile, counted busy/request scopes, guarded switching and CRUD, invalidation, status and redaction. |
| `src/lib/components/SettingsTab.svelte` | Profile manager and independent global TMDb controls; existing logs retained. |
| `src/lib/components/ServerSelector.svelte` | Global sidebar popover. |
| `src/routes/+page.svelte` | Selector integration, context-keyed Plex components, lazy library loading. |
| `src/app.css` | Scoped dark profile/editor/popover styling and compact-layout visibility. |
| `src/lib/components/TrashTab.svelte`, `SubsTab.svelte`, `SubSelectorTab.svelte` | Route calls through the central wrapper; include uploads in global operation busy state. No layout changes. |
| `tests/plex-profiles.test.mjs`, `package.json` | Tests compile the real Svelte state module with a mocked Tauri boundary; `npm test` entry point. |
| `PROJECT_CONTEXT.md`, `docs/multi-server-support.md` | Updated architecture and implementation report. |

Validation completed:

- `cargo test --manifest-path src-tauri/Cargo.toml`: 15 backend tests passed, including all 10 pre-existing tests and five configuration/persistence tests.
- `npm test`: all 10 profile/context scenarios passed (11 Node test entries including the parent). Coverage includes every Plex command's busy guard, overlapping scopes, transition rejection, failed persistence, failed connectivity, CRUD, stable rename, restart selection, independent offline saves, draft tests, stale data clearing, active routing, and redaction.
- `npm run check`: zero errors and zero warnings.
- `npm run tauri build`: production frontend and portable Windows executable built; no installer. Output: `src-tauri/target/release/plex_suite.exe`.
- `git diff --check`: no whitespace errors.
- Isolated browser fixture: checked a 25-server popover, bounded scrolling, offline selection, offline profile Save, dark Settings layout, and compact navigation at 580px. Temporary fixtures were removed.

The environment's default `npm` shim was broken, so checks used `C:\Program Files\nodejs\npm.cmd`; the build placed that directory first on PATH for its nested frontend build.

No live Plex/TMDb credentials were used, and no real media operations were performed. Network/UI behavior was exercised with mocks plus existing backend tests; real-server end-to-end verification and Linux runtime testing remain unperformed. Credentials retain the application's existing plaintext portable-file storage model. Connectivity is checked on demand, not continuously. Existing global UI busy behavior also blocks switching during TMDb work, conservatively.

Suggested Conventional Commit: `feat: add global Plex server profiles and independent connection settings`
