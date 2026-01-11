# PlexTools

Portable desktop toolbox for Plex TV libraries. PlexTools bundles three workflows in one app:
- Trash Selector: safe, selective trash purge for a show or season.
- Sub Uploader: bulk subtitle upload with automatic episode matching.
- Plexmatch Generator: build a .plexmatch file from TMDb episodes and local files.

## Why PlexTools
- Dark, focused UI tuned for Plex workflows.
- Preview before destructive actions.
- Focused on TV libraries (shows, seasons, episodes).
- Portable settings stored next to the executable (`settings.json`).
- Built-in activity log in Settings.
- Windows and Linux builds (separate executables).

## Quick start (prebuilt)
1. Download the latest release for your OS.
2. Run the executable (no installer).
3. Open Settings and enter your Plex URL and Plex token.
4. Optional: add a TMDb API key to enable Plexmatch Generator.

## Build from source
Requirements:
- Node.js (LTS)
- Rust toolchain
- Windows: Visual Studio Build Tools (MSVC)
- Linux: Tauri system dependencies (GTK/WebKit)

Install and run:
```powershell
npm install
npm run tauri dev
```

Build a portable release:
```powershell
npm run tauri build
```

Output:
- Windows: `src-tauri/target/release/plex-tools.exe`
- Linux: `src-tauri/target/release/plex-tools`

## Getting a Plex token
1. Open Plex Web in a browser and sign in.
2. Open DevTools -> Network.
3. Filter for `X-Plex-Token` or find a request URL that includes `X-Plex-Token=...`.
4. Copy the token into Settings.

## Usage tutorials

### Trash Selector (selective purge)
1. In Settings, save your Plex URL and token until you see "Connected".
2. In Trash Selector, choose a TV library, show, and optional season.
3. Click "Dry Run" to preview what will be removed.
4. Click "Purge Trash" to delete the trashed or missing parts for that scope.

Tips:
- Use Dry Run before every purge.
- The preview counts missing parts, not just full episodes.

### Sub Uploader (bulk subtitles)
1. Choose the TV library, show, and optional season.
2. Click "Choose folder" and select your subtitle folder.
3. Click "Preview Mapping" to see matches.
4. Click "Upload Subtitles" to send matched files to Plex.

Matching notes:
- Uses common patterns like `S01E02`, `1x02`, and similar.
- The status dot turns green when a subtitle is matched.

### Plexmatch Generator (TMDb to .plexmatch)
1. Add your TMDb API key in Settings.
2. Search for the series, then select it to load episodes.
3. Select episodes and add video files, then click "Map selected".
4. Drag to reorder file rows or mappings if needed.
5. Adjust "Path depth" to control how much parent path is written.
6. Click "Save .plexmatch" and choose a location.

## Data and portability
- Settings live next to the executable in `settings.json`.
- Delete `settings.json` to reset the app.
- No installers; the executable is self-contained.

## Tech stack
- Tauri (Rust backend)
- SvelteKit + Vite (frontend)

## License
MIT
