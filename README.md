# PlexSuite

PlexSuite is a portable desktop toolbox for maintaining Plex TV libraries. It brings four focused tools together in one small, dark-themed app with no installer required.

It is useful for Plex users who want more control over missing media, subtitles, and local episode matching.

## Features

### Trash Selector

Preview and selectively purge trashed or missing media entries from Plex TV libraries.

### Sub Uploader

Match subtitle files with episodes and upload them to Plex in bulk.

### Sub Selector

Scan available subtitle variants, set preferred subtitles as default, and remove selected external subtitles.

### Plexmatch Generator

Match TMDb episodes with local video files and generate `.plexmatch` files.

## Getting Started

1. Download the portable Windows executable and launch PlexSuite.
2. Open **Settings**.
3. Enter your Plex URL and Plex token.
4. Add a TMDb API key if you plan to use Plexmatch Generator.

## Usage

### Trash Selector

Select a TV library, show, and optional season. Run a dry-run preview, review the results, then purge only the entries you intend to remove.

### Sub Uploader

Select the Plex library, show, and season, choose a subtitle folder, review the automatic episode matching, make any needed adjustments, then upload.

### Sub Selector

Select a library, show, and season (or all seasons), then scan subtitles. Choose a variant to set as the default, or select the external subtitle categories you want to remove and confirm the cleanup.

### Plexmatch Generator

Search for a show with TMDb, select its episodes and local video files, review or adjust the matching, then save the generated `.plexmatch` file.

## Portable Data

PlexSuite stores settings next to the executable in `settings.json`. This file can contain your Plex token and TMDb API key, so keep it private.

Logs are session-only. Debug mode is available in Settings when you need additional scan diagnostics, and the current logs can be exported as plain text.

## Platform Support

- Windows: tested and published.
- Linux: expected to be possible with Tauri, but not currently tested or published.
- macOS: not tested or published.

## Safety

- Review previews before any destructive action.
- Subtitle cleanup can remove the external subtitle categories you select.
- Keep `settings.json` private.

## Tech Stack

- Tauri 2
- Rust
- SvelteKit
- Svelte 5
- Vite

## License

MIT
