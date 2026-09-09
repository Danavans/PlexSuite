# PlexSuite

PlexSuite is a portable desktop toolbox for Plex TV libraries. It brings together a few practical Plex maintenance tools in one small app, with a dark interface and no installer required.

## What It Does

PlexSuite includes four tools:

- Trash Selector: preview and selectively purge trashed or missing media from a Plex TV library.
- Sub Selector: select exact subtitle variants and remove explicitly selected external subtitle categories.
- Sub Uploader: match subtitle files to episodes and upload them to Plex in bulk.
- Plexmatch Generator: use TMDb episode data and local video files to create a `.plexmatch` file.

## Platforms

- Windows: tested and currently published.
- Linux: expected to be possible with Tauri, but not tested for this release.

The app is portable. Download the Windows executable, keep it wherever you want, and run it directly. No installer is required.

## First Setup

Open PlexSuite, go to Settings, and enter:

- Plex URL: the address of your Plex server, for example `http://192.168.1.10:32400`.
- Plex token: required so PlexSuite can talk to your Plex server.
- TMDb API key: optional, only needed for Plexmatch Generator.

When the connection works, the app shows a connected status.

## Local Data

PlexSuite saves its settings next to the executable in a `settings.json` file. This can include your Plex token and TMDb API key, so keep that file private.

To reset the app, close PlexSuite and delete `settings.json`.

## How To Use

### Trash Selector

Use this when Plex has trashed or missing media entries and you want to clean them carefully.

1. Select a TV library.
2. Select a show and, if needed, a season.
3. Run a dry-run preview first.
4. Review the list.
5. Purge only when the preview matches what you expect.

### Sub Uploader

Use this when you have subtitle files in a folder and want to send them to Plex.

1. Select the matching Plex library, show, and season.
2. Choose the subtitle folder.
3. Preview the automatic matching.
4. Adjust the mapping if needed.
5. Upload the matched subtitles.

### Sub Selector

1. Select a TV library, show, and season (or All Seasons), then click **Scan Subtitles**.
2. Expand a language and select an exact region/script, Forced, and SDH variant. Missing episodes and scan errors can be inspected separately.
3. **Set as Default** selects matching subtitle streams for the Plex user associated with the token. It leaves episodes without a match unchanged and matches each media version/part independently. If equivalent streams coexist in a part, an already selected stream is preferred, otherwise the first matching stream is used.
4. Under **Subtitle Cleanup**, select **Physical Sidecar**, **Plex Uploaded**, **Unknown External**, or any combination, then click **Remove Selected**. Nothing is selected by default. The confirmation shows the scope, category counts, and relevant warnings. Embedded tracks have no checkbox and can never be deleted.

Classification uses subtitle stream metadata, not titles. An absent `index` defaults to -1, matching python-plexapi; explicit malformed/null indexes remain uncertain. A nonnegative index is embedded and always protected. External streams require a numeric `/library/streams/<id>` key matching their ID. An explicit numeric/string zero `transient` identifies Plex uploads; absent/null `transient` identifies physical sidecars under the observed server convention; other values remain Unknown External. These are metadata-based provenance labels, not a universal Plex guarantee.

Physical Sidecar removal may permanently delete subtitle files next to the video. Unknown External is selectable but marked less safe because it may also include physical files. Each relevant warning appears in the confirmation. All removal uses the Plex stream DELETE endpoint; PlexSuite never deletes subtitle files through filesystem code.

Only reviewed keys are eligible. The backend scans fresh metadata, then re-fetches the owning episode immediately before each DELETE, validates library/show/season membership and matching stream key/ID, reclassifies the stream, and requires its current category to be explicitly selected. Embedded tracks, invalid identifiers, malformed indexes, conflicting observations and changed/unselected categories are skipped. A classification of Unknown External alone does not bypass these checks. Selections are cleared after actions/scans and scope changes.

The September 2026 read-only comparison on a Barry Season 3 episode found the same attributes in JSON and XML: omitted `index`, matching stream key/ID, and `transient: "0"`. The old parser required explicit `index == -1`, which explains the Unknown External result; python-plexapi had supplied that default during earlier tests. No XML migration is needed. This live comparison covered an upload; sidecar metadata is covered by fixtures matching the reported observations. To repeat the diagnostic with PowerShell 7, run `./scripts/diagnose-subtitle-metadata.ps1 -SettingsPath <settings.json> -Show "Barry" -Season 3`. It compares the first episode of that season using GET only and prints an allowlist of classification fields without tokens, URLs or raw response bodies. It is a development aid, not an application dependency.

The scan and actions use direct Rust HTTP requests. No Python runtime is required. Global language preferences and audio selections are not changed. A failed episode/stream is reported and processing continues. Refresh the scan after external changes in Plex.

### Plexmatch Generator

Use this to create a `.plexmatch` file from TMDb episode data and your local video files.

1. Add a TMDb API key in Settings.
2. Search for the show.
3. Select the episodes and video files.
4. Check or adjust the mapping.
5. Save the `.plexmatch` file.

## Safety Notes

- Always use previews before deleting or uploading.
- PlexSuite is focused on TV libraries.
- The Windows version is the only tested release target for now.
- Linux may work, but it is not part of the published release yet.

## Tech Stack

- Tauri 2
- Rust
- SvelteKit
- Svelte 5
- Vite

## License

MIT
