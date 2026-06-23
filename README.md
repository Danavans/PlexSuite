# PlexSuite

PlexSuite is a portable desktop toolbox for Plex TV libraries. It combines three workflows in one dark Tauri app:

- Trash Selector: preview and selectively purge trashed or missing media parts.
- Sub Uploader: bulk upload subtitles with automatic episode matching.
- Plexmatch Generator: create `.plexmatch` files from TMDb episode data and local video files.

## Status

Private project in early development. The app is usable for local workflows, but destructive actions should always be checked with preview/dry-run steps first.

## Platforms

- Windows: primary target, portable executable.
- Linux: supported target, portable executable.
- Installers are intentionally not part of the default build flow.

## Local Data

PlexSuite stores local settings next to the executable in `settings.json`. This file can contain a Plex token and a TMDb API key, so it is ignored by Git and must not be committed.

## Requirements

- Node.js LTS
- Rust toolchain
- Windows: Visual Studio Build Tools with MSVC
- Linux: Tauri system dependencies, including GTK/WebKit packages

## Development

Install dependencies:

```powershell
npm install
```

Run the app in development:

```powershell
npm run tauri dev
```

Run checks:

```powershell
npm run check
```

Build a portable release:

```powershell
npm run tauri build
```

Build outputs are generated under `src-tauri/target/` and are ignored by Git.

## Usage

1. Open Settings.
2. Enter the Plex server URL and Plex token.
3. Add a TMDb API key if you want to use Plexmatch Generator.
4. Use the relevant tab:
   - Trash Selector for dry-run previews and scoped trash purges.
   - Sub Uploader for subtitle folder matching and upload.
   - Plexmatch Generator for TMDb lookup, file mapping, and `.plexmatch` export.

## Tech Stack

- Tauri 2
- Rust
- SvelteKit
- Svelte 5
- Vite

## License

MIT
