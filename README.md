# PlexSuite v1.2.0

PlexSuite is a portable desktop toolkit for maintaining Plex TV libraries. It brings four focused tools together in one app, with no installer required.

## Tools

### Trash Selector

Preview trashed or missing media entries for a selected TV show or season, then purge only the items you intend to remove.

### Sub Uploader

Match local subtitle files to Plex episodes, review or adjust the mapping, and upload the selected subtitles in bulk.

### Sub Selector

Scan subtitle variants, set a preferred variant as the default, and safely remove the external subtitle categories you select.

### Plexmatch Generator

Match TMDb episodes to local video files, review the mapping, and save a `.plexmatch` file.

## First setup

1. Download the portable executable and launch PlexSuite.
2. Open **Settings** from the Plex Toolkit sidebar.
3. Enter your Plex server URL and Plex token.
4. Add a TMDb API key to use Plexmatch Generator.

## Basic workflow

Choose a tool from the sidebar and follow its numbered guidance. Review every dry run, match, or cleanup selection before confirming it. The persistent Activity bar provides status updates and a shortcut to the session logs.

## Portability and privacy

PlexSuite stores its settings next to the executable in `settings.json`. This file can contain your Plex token and TMDb API key, so keep it private and do not share it. Logs are session-only and can be exported from Settings when needed.

## Platform status

- Windows: tested and published as a portable executable.
- Linux: supported by the Tauri-based codebase, but not currently tested or published.
- macOS: not tested or published.

## Safety

- Review previews before purging trash or changing subtitles.
- Subtitle cleanup removes only the external categories you explicitly select; embedded tracks are protected.
- Keep `settings.json` private.

## Tech stack

Tauri 2, Rust, SvelteKit, Svelte 5, and Vite.

## License

MIT
