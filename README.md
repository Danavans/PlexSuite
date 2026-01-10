# PlexTools

Unified desktop toolbox for Plex:
- Trash Selector (selective trash purge for a show or season)
- Sub Uploader (bulk subtitle upload with episode auto-detection)
- Plexmatch Generator (TMDb-based .plexmatch creation)

## Highlights
- Single Settings panel (Plex URL, Plex token, TMDb API key)
- Preview before destructive actions (Trash Selector, Sub Uploader)
- Portable settings storage next to the executable (`settings.json`)
- Global log panel in Settings for quick debugging
- Windows + Linux builds (separate executables per OS)

## Requirements
- Node.js (LTS recommended)
- Rust toolchain
- Windows: Visual Studio Build Tools (MSVC)
- Linux: Tauri system dependencies (GTK/WebKit). See Tauri prerequisites.

## Install
```powershell
npm install
```

## Run in dev mode
```powershell
npm run tauri dev
```

## Build release executable
```powershell
npm run tauri build
```

Output folder:
`src-tauri/target/release/bundle/`

## Usage overview
- **Trash Selector**: pick Library, Show, Season; run Dry Run then Purge.
- **Sub Uploader**: choose a subtitles folder; preview mapping; upload to Plex.
- **Plexmatch Generator**: search TMDb, select episodes, add video files, map, save `.plexmatch`.

## Plex token
A Plex token is required to connect. You can retrieve it from a browser session logged into Plex.

## Stack
- Tauri (Rust backend + desktop window)
- SvelteKit + Vite (frontend)

## License
MIT
