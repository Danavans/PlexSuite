# PlexSuite

PlexSuite is a portable desktop toolbox for Plex TV libraries. It brings together a few practical Plex maintenance tools in one small app, with a dark interface and no installer required.

## What It Does

PlexSuite includes three tools:

- Trash Selector: preview and selectively purge trashed or missing media from a Plex TV library.
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
