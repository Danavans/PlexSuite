<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";

  let activeTab = $state("trash");
  let isBusy = $state(false);
  let status = $state({ type: "info", message: "" });
  let isConnected = $state(false);
  let connectionMessage = $state("");

  let serverUrl = $state("");
  let token = $state("");
  let tmdbKey = $state("");
  let showToken = $state(false);

  let libraries = $state([]);
  let selectedLibraryId = $state("");
  let selectedLibraryTitle = $state("");
  let shows = $state([]);
  let selectedShow = $state(null);
  let selectedShowKey = $state("");
  let seasons = $state([]);
  let selectedSeason = $state(null);
  let selectedSeasonKey = $state("");

  let preview = $state({ count: 0, titles: [] });
  let confirmOpen = $state(false);
  let confirmTargetLabel = $state("");
  let purgeSummary = $state({ removed: 0 });
  let purgeCompleteOpen = $state(false);
  let plexmatchSavedOpen = $state(false);
  let plexmatchSavedPath = $state("");

  let subsRoot = $state("");
  let subsPreview = $state({ total: 0, matched: 0, entries: [] });
  let subsUploadBusy = $state(false);
  let subsUploadPreConfirmOpen = $state(false);
  let subsUploadConfirmOpen = $state(false);
  let subsUploadSummary = $state({ uploaded: 0, failed: 0 });
  let subsExpandedKey = $state("");
  let appLogs = $state([]);

  let tmdbQuery = $state("");
  let tmdbResults = $state([]);
  let tmdbEpisodes = $state([]);
  let selectedSeries = $state(null);
  let selectedSeriesId = $state("");
  let selectedEpisodeCodes = $state([]);

  let pathDepth = $state(0);
  let plexmatchOutputPath = $state("");
  let videoFiles = $state([]);
  let mappings = $state([]);
  let plexmatchPreview = $state("");
  let showsRequestId = 0;
  const videoExts = new Set([
    ".mkv",
    ".mp4",
    ".avi",
    ".mov",
    ".flv",
    ".wmv",
    ".mpg",
    ".mpeg"
  ]);

  const filteredShows = $derived(shows);
  const trashFilteredShows = $derived(shows);
  const trashFilteredSeasons = $derived(seasons);
  const tabDescription = $derived(
    activeTab === "trash"
      ? "Purge Plex trash selectively for a show or season with a safe dry run."
      : activeTab === "subs"
        ? "Bulk upload subtitles by auto-matching episodes from filenames."
        : activeTab === "plexmatch"
          ? "Map TMDb episodes to local files and export a PlexMatch file."
          : "Manage Plex and TMDb credentials used by the app."
  );

  function setStatus(type, message) {
    status = { type, message };
    const timestamp = new Date().toLocaleTimeString("en-GB", { hour12: false });
    const tabLabel =
      activeTab === "trash"
        ? "Trash"
        : activeTab === "subs"
          ? "Subtitles"
          : activeTab === "plexmatch"
            ? "PlexMatch"
            : "Settings";
    appLogs = [
      { type, message, timestamp, tab: tabLabel },
      ...appLogs
    ].slice(0, 200);
  }

  function dirname(path) {
    return path.replace(/[/\\][^/\\]*$/, "");
  }

  function normalizePath(path) {
    return path.replace(/\\/g, "/");
  }

  function relativePath(full, root) {
    const normFull = normalizePath(full);
    const normRoot = normalizePath(root).replace(/\/$/, "");
    if (!normRoot) {
      return normFull;
    }
    if (normFull.toLowerCase().startsWith(normRoot.toLowerCase())) {
      return normFull.slice(normRoot.length).replace(/^\/+/, "");
    }
    return normFull;
  }

  function buildDisplayPath(filePath, fileName) {
    const normalized = normalizePath(filePath);
    const parts = normalized.split("/").filter(Boolean);
    const file = fileName || parts[parts.length - 1] || "";
    if (pathDepth < 0) {
      return file;
    }
    const parentParts = parts.slice(0, Math.max(0, parts.length - 1));
    const keepParents = Math.min(parentParts.length, pathDepth + 1);
    const start = Math.max(0, parentParts.length - keepParents);
    const selectedParents = parentParts.slice(start);
    return [...selectedParents, file].join("/");
  }

  function updatePlexmatchPreview() {
    let text = "#https://support.plex.tv/articles/plexmatch/\n\n";
    for (const item of mappings) {
      const displayPath = buildDisplayPath(item.filePath, item.fileName);
      text += `ep:${item.code}:${displayPath}\n`;
    }
    plexmatchPreview = text;
  }

  $effect(() => {
    updatePlexmatchPreview();
  });

  async function loadSettings() {
    try {
      const settings = await invoke("get_settings");
      serverUrl = settings.plex_url ?? "";
      token = settings.plex_token ?? "";
      tmdbKey = settings.tmdb_key ?? "";
      if (serverUrl && token) {
        await connectToPlex(true);
      } else {
        connectionMessage = "";
      }
    } catch (error) {
      setStatus("error", `Failed to load settings: ${error}`);
    }
  }

  async function saveSettings() {
    if (!serverUrl.trim() || !token.trim()) {
      setStatus("error", "Plex URL and token are required.");
      return;
    }
    isBusy = true;
    setStatus("info", "Saving settings...");
    try {
      await invoke("save_settings", {
        plexUrl: serverUrl,
        plexToken: token,
        tmdbKey
      });
      setStatus("success", "Settings saved.");
      await connectToPlex(false);
    } catch (error) {
      setStatus("error", `Save failed: ${error}`);
    } finally {
      isBusy = false;
    }
  }

  async function connectToPlex(auto) {
    if (!serverUrl.trim() || !token.trim()) {
      connectionMessage = "Enter Plex URL and token to connect.";
      isConnected = false;
      return;
    }
    isBusy = true;
    connectionMessage = auto ? "Connecting to Plex..." : "Connecting...";
    try {
      const data = await invoke("list_libraries", { serverUrl, token });
      libraries = data;
      selectedLibraryId = libraries[0]?.id ?? "";
      isConnected = true;
      connectionMessage = "Connected.";
      if (selectedLibraryId) {
        await loadShows();
      }
    } catch (error) {
      isConnected = false;
      connectionMessage = `Connection failed: ${error}`;
    } finally {
      isBusy = false;
    }
  }

  async function loadLibraries() {
    if (!serverUrl.trim() || !token.trim()) {
      setStatus("error", "Provide Plex URL and token first.");
      return;
    }
    isBusy = true;
    setStatus("info", "Loading libraries...");
    try {
      const data = await invoke("list_libraries", { serverUrl, token });
      libraries = data;
      selectedLibraryId = libraries[0]?.id ?? "";
      setStatus("success", "Libraries loaded.");
      if (selectedLibraryId) {
        await loadShows();
      }
    } catch (error) {
      setStatus("error", `Library load failed: ${error}`);
    } finally {
      isBusy = false;
    }
  }

  async function loadShows(query = null, libraryIdOverride = null) {
    const libraryId = libraryIdOverride ?? selectedLibraryId;
    if (!libraryId) {
      setStatus("error", "Select a library first.");
      return;
    }
    const requestId = ++showsRequestId;
    const currentLibrary = libraries.find((item) => item.id === libraryId);
    selectedLibraryTitle = currentLibrary?.title ?? "";
    isBusy = true;
    setStatus("info", "Loading shows...");
    try {
      const data = await invoke("list_shows", {
        serverUrl,
        token,
        libraryId,
        query
      });
      if (requestId !== showsRequestId) {
        return;
      }
      shows = data;
      selectedShow = null;
      selectedShowKey = "";
      seasons = [];
      selectedSeason = null;
      selectedSeasonKey = "";
      preview = { count: 0, titles: [] };
      setStatus("success", `Loaded ${shows.length} shows.`);
    } catch (error) {
      setStatus("error", `Show load failed: ${error}`);
    } finally {
      isBusy = false;
    }
  }

  async function loadSeasons(show) {
    if (!show) return;
    isBusy = true;
    setStatus("info", `Loading seasons for ${show.title}...`);
    try {
      const data = await invoke("list_seasons", {
        serverUrl,
        token,
        showRatingKey: show.rating_key
      });
      seasons = data;
      selectedSeason = null;
      selectedSeasonKey = "";
      setStatus("success", `Loaded ${seasons.length} seasons.`);
    } catch (error) {
      setStatus("error", `Season load failed: ${error}`);
    } finally {
      isBusy = false;
    }
  }

  function selectShow(show) {
    selectedShow = show;
    selectedShowKey = show?.rating_key ?? "";
    selectedSeason = null;
    selectedSeasonKey = "";
    loadSeasons(show);
  }

  function selectSeason(season) {
    selectedSeason = season;
    selectedSeasonKey = season?.rating_key ?? "";
  }

  async function previewTrash() {
    if (!selectedShow) {
      setStatus("error", "Choose a show first.");
      return;
    }
    isBusy = true;
    setStatus("info", "Previewing trash items...");
    try {
      const result = await invoke("preview_trash", {
        serverUrl,
        token,
        libraryId: selectedLibraryId,
        showRatingKey: selectedShow.rating_key,
        seasonRatingKey: selectedSeason?.rating_key ?? null
      });
      preview = result;
      setStatus("success", `Found ${result.count} trashed item(s).`);
    } catch (error) {
      setStatus("error", `Preview failed: ${error}`);
    } finally {
      isBusy = false;
    }
  }

  function openPurgeConfirm() {
    if (!selectedShow) {
      setStatus("error", "Choose a show first.");
      return;
    }
    confirmTargetLabel = selectedSeason
      ? `${selectedShow.title} - ${selectedSeason.title}`
      : selectedShow.title;
    confirmOpen = true;
  }

  async function confirmPurge() {
    confirmOpen = false;
    isBusy = true;
    setStatus("info", "Purging trash...");
    try {
      const result = await invoke("purge_trash", {
        serverUrl,
        token,
        libraryId: selectedLibraryId,
        showRatingKey: selectedShow.rating_key,
        seasonRatingKey: selectedSeason?.rating_key ?? null
      });
      purgeSummary = { removed: result.removed_count };
      purgeCompleteOpen = true;
      setStatus("success", `Removed ${result.removed_count} trashed item(s).`);
    } catch (error) {
      setStatus("error", `Purge failed: ${error}`);
    } finally {
      isBusy = false;
    }
  }

  async function previewSubtitles() {
    if (!selectedShow) {
      setStatus("error", "Choose a show first.");
      return;
    }
    if (!subsRoot.trim()) {
      setStatus("error", "Provide a subtitles folder.");
      return;
    }
    const episodeNumber = null;
    isBusy = true;
    setStatus("info", "Scanning subtitle files...");
    try {
      const result = await invoke("preview_subtitles", {
        serverUrl,
        token,
        showRatingKey: selectedShow.rating_key,
        seasonNumber: selectedSeason?.index ?? null,
        episodeNumber,
        subsRoot
      });
      subsPreview = result;
      setStatus(
        "success",
        `Matched ${result.matched}/${result.total} subtitle file(s).`
      );
    } catch (error) {
      setStatus("error", `Preview failed: ${error}`);
    } finally {
      isBusy = false;
    }
  }

  async function uploadSubtitles() {
    if (subsPreview.matched === 0) {
      setStatus("error", "No matched subtitles to upload.");
      return;
    }
    subsUploadConfirmOpen = false;
    subsUploadBusy = true;
    setStatus("info", "Uploading subtitles...");
    let result = null;
    try {
      const items = subsPreview.entries
        .filter((entry) => entry.episodeRatingKey)
        .map((entry) => ({
          path: entry.path,
          episodeRatingKey: entry.episodeRatingKey
        }));
      result = await invoke("upload_subtitles", {
        serverUrl,
        token,
        items
      });
      if (result.failed.length > 0) {
        setStatus(
          "error",
          `Uploaded ${result.uploaded} with ${result.failed.length} error(s).`
        );
      } else {
        setStatus("success", `Uploaded ${result.uploaded} subtitle file(s).`);
      }
    } catch (error) {
      setStatus("error", `Upload failed: ${error}`);
    } finally {
      subsUploadBusy = false;
    }
    if (result) {
      subsUploadSummary = {
        uploaded: result.uploaded,
        failed: result.failed.length
      };
      subsUploadConfirmOpen = true;
    }
  }

  async function searchTmdb() {
    if (!tmdbKey.trim()) {
      setStatus("error", "TMDb API key is required.");
      return;
    }
    if (!tmdbQuery.trim()) {
      setStatus("error", "Enter a series name.");
      return;
    }
    isBusy = true;
    setStatus("info", "Searching TMDb...");
    try {
      const result = await invoke("tmdb_search_series", {
        apiKey: tmdbKey,
        query: tmdbQuery
      });
      tmdbResults = result;
      selectedSeries = null;
      selectedSeriesId = "";
      tmdbEpisodes = [];
      setStatus("success", `Found ${result.length} result(s).`);
    } catch (error) {
      setStatus("error", `TMDb search failed: ${error}`);
    } finally {
      isBusy = false;
    }
  }

  async function selectSeries(series) {
    selectedSeries = series;
    selectedSeriesId = String(series?.id ?? "");
    isBusy = true;
    setStatus("info", "Loading episodes...");
    try {
      const result = await invoke("tmdb_list_episodes", {
        apiKey: tmdbKey,
        seriesId: series.id
      });
      tmdbEpisodes = result;
      selectedEpisodeCodes = [];
      setStatus("success", `Loaded ${result.length} episodes.`);
    } catch (error) {
      setStatus("error", `TMDb load failed: ${error}`);
    } finally {
      isBusy = false;
    }
  }

  function addFilePaths(paths) {
    const next = [...videoFiles];
    for (const fullPath of paths) {
      const name = fullPath.split(/[/\\]/).pop();
      const lower = name.toLowerCase();
      const ext = lower.slice(lower.lastIndexOf("."));
      if (!videoExts.has(ext)) continue;
      if (next.some((f) => f.path === fullPath)) continue;
      next.push({ name, path: fullPath });
    }
    videoFiles = next;
  }

  async function pickVideoFiles() {
    const result = await open({
      multiple: true,
      filters: [
        {
          name: "Video files",
          extensions: ["mkv", "mp4", "avi", "mov", "flv", "wmv", "mpg", "mpeg"]
        }
      ]
    });
    if (!result) return;
    const paths = Array.isArray(result) ? result : [result];
    addFilePaths(paths);
  }

  function increasePathDepth() {
    pathDepth += 1;
  }

  function decreasePathDepth() {
    pathDepth = Math.max(-1, pathDepth - 1);
  }

  async function pickSubsFolder() {
    const result = await open({
      directory: true,
      multiple: false
    });
    if (typeof result === "string") {
      subsRoot = result;
    }
  }

  function mapSelected() {
    const episodeItems = tmdbEpisodes.filter((ep) =>
      selectedEpisodeCodes.includes(ep.code)
    );
    const fileItems = [...videoFiles];
    const count = Math.min(episodeItems.length, fileItems.length);
    if (count === 0) return;
    const nextMappings = [...mappings];
    for (let i = 0; i < count; i += 1) {
      const ep = episodeItems[i];
      const file = fileItems[i];
      nextMappings.push({
        code: ep.code,
        fileName: file.name,
        filePath: file.path
      });
    }
    mappings = nextMappings;
    selectedEpisodeCodes = [];
    updatePlexmatchPreview();
  }

  function removeMapping(index) {
    mappings = mappings.filter((_, i) => i !== index);
    updatePlexmatchPreview();
  }

  async function savePlexmatch() {
    const result = await save({
      defaultPath: ".plexmatch",
      filters: [{ name: "PlexMatch", extensions: ["plexmatch"] }]
    });
    if (!result) return;
    plexmatchOutputPath = result;
    try {
      if (!plexmatchOutputPath.toLowerCase().endsWith(".plexmatch")) {
        plexmatchOutputPath = `${plexmatchOutputPath}.plexmatch`;
      }
      await invoke("save_text_file", {
        path: plexmatchOutputPath,
        content: plexmatchPreview
      });
      plexmatchSavedPath = plexmatchOutputPath;
      plexmatchSavedOpen = true;
    } catch (error) {
      setStatus("error", `Save failed: ${error}`);
    }
  }

  function clearEpisodeSelection() {
    selectedEpisodeCodes = [];
  }

  function selectAllEpisodes() {
    selectedEpisodeCodes = tmdbEpisodes.map((ep) => ep.code);
  }

  function clearVideoFiles() {
    videoFiles = [];
  }

  function selectLibrary(library) {
    selectedLibraryId = library.id;
    selectedLibraryTitle = library.title;
    shows = [];
    seasons = [];
    selectedShow = null;
    selectedShowKey = "";
    selectedSeason = null;
    selectedSeasonKey = "";
    loadShows(null, library.id);
  }


  let dragIndex = $state(null);
  let dragOverIndex = $state(null);
  let dragActive = $state(false);

  function startPointerDrag(event, index) {
    dragActive = true;
    dragIndex = index;
    dragOverIndex = index;
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function movePointerDrag(event) {
    if (!dragActive) return;
    const el = document.elementFromPoint(event.clientX, event.clientY);
    const row = el?.closest?.(".file-row");
    if (!row) return;
    const index = Number(row.dataset.index);
    if (Number.isNaN(index) || index === dragIndex) return;
    moveFile(dragIndex, index);
    dragIndex = index;
    dragOverIndex = index;
  }

  function endPointerDrag(event) {
    if (!dragActive) return;
    dragActive = false;
    dragIndex = null;
    dragOverIndex = null;
    event.currentTarget.releasePointerCapture(event.pointerId);
  }

  function moveFile(fromIndex, toIndex) {
    if (fromIndex < 0 || toIndex < 0) return;
    if (fromIndex === toIndex) return;
    const next = [...videoFiles];
    const [moved] = next.splice(fromIndex, 1);
    next.splice(toIndex, 0, moved);
    videoFiles = next;
  }

  function readSelectedValues(event) {
    return Array.from(event.currentTarget.selectedOptions).map((opt) => opt.value);
  }

  onMount(() => {
    loadSettings();
  });
</script>

<main
  class="app"
  class:app-blocked={subsUploadBusy}
  aria-busy={subsUploadBusy}
  inert={subsUploadBusy}
>
  <header class="hero">
    <div class="hero-top">
      <div class="hero-brand">
        <p class="badge">PlexTools</p>
        <h1 class="sr-only">PlexTools</h1>
      </div>
      <div class="status-row">
        <div class="status-pill">
          <span class={`status-dot ${isConnected ? "ok" : "down"}`}></span>
          <span class="status-text">
            {isConnected ? "Connected" : "Disconnected"}
          </span>
        </div>
      </div>
    </div>
    <div class="hero-bottom">
      <div class="tabs-wrap">
        <div class="tabs">
        <button
          class:active-tab={activeTab === "trash"}
          on:click={() => (activeTab = "trash")}
        >
          Trash Selector
        </button>
        <button
          class:active-tab={activeTab === "subs"}
          on:click={() => (activeTab = "subs")}
        >
          Sub Uploader
        </button>
        <button
          class:active-tab={activeTab === "plexmatch"}
          on:click={() => (activeTab = "plexmatch")}
        >
          Plexmatch Generator
        </button>
        <button
          class:active-tab={activeTab === "settings"}
          on:click={() => (activeTab = "settings")}
        >
          Settings
        </button>
        </div>
      </div>
      <p class="hero-desc">{tabDescription}</p>
    </div>
  </header>


    {#if activeTab === "settings"}
      <section class="grid module-grid">
        <div class="panel settings-panel lift-1">
          <h2 class="section-title">Credentials</h2>
          <p class="kicker">
            Save once. The app reconnects automatically on launch.
          </p>
          <div class="field">
            <label for="plex-url">Plex URL</label>
            <input
              id="plex-url"
              placeholder="http://192.168.1.100:32400"
              bind:value={serverUrl}
            />
          </div>
          <div class="field">
            <label for="plex-token">Plex Token</label>
            <input
              id="plex-token"
              type={showToken ? "text" : "password"}
              placeholder="X-Plex-Token"
              bind:value={token}
            />
          </div>
          <label class="toggle">
            <input type="checkbox" bind:checked={showToken} />
            Show token
          </label>
          <div class="field" style="margin-top: 16px;">
            <label for="tmdb-key">TMDb API Key</label>
            <input
              id="tmdb-key"
              placeholder="Your TMDb key"
              bind:value={tmdbKey}
            />
          </div>
        <div class="actions">
          <button data-variant="primary" on:click={saveSettings} disabled={isBusy}>
            Save & Connect
          </button>
        </div>
      </div>
      <div class="panel lift-2">
        <h2>Logs</h2>
        <div class="actions">
          <button
            data-variant="ghost"
            on:click={() => {
              appLogs = [];
            }}
          >
            Clear logs
          </button>
        </div>
        <div class="debug">
          {#if appLogs.length === 0}
            <p class="kicker">No logs yet.</p>
          {:else}
            {#each appLogs as entry}
              <p>
                [{entry.timestamp}] [{entry.tab}] {entry.type.toUpperCase()}: {entry.message}
              </p>
            {/each}
          {/if}
        </div>
      </div>
    </section>
    {:else if activeTab === "trash"}
      <section class="grid trash-grid">
        <div class="panel trash-library lift-1">
          <h2>Library Scope</h2>
          <div class="kicker" style="margin-bottom: 10px;">
            Choose a library, show, and optional season to define the purge scope.
          </div>
          <div class="field">
            <label for="library">TV Library</label>
            <select
              id="library"
              bind:value={selectedLibraryId}
              on:change={(event) => {
                const key = event.currentTarget.value;
                if (key) {
                  const library = libraries.find((item) => item.id === key);
                  if (library) {
                    selectLibrary(library);
                  }
                } else {
                  shows = [];
                  selectedShow = null;
                  selectedShowKey = "";
                  seasons = [];
                  selectedSeason = null;
                  selectedSeasonKey = "";
                }
              }}
            >
              <option value="">Select a library</option>
              {#each libraries as library}
                <option value={library.id}>{library.title}</option>
              {/each}
            </select>
          </div>
          <div class="field">
            <label for="trash-show">Show</label>
            <select
              id="trash-show"
              bind:value={selectedShowKey}
              on:change={(event) => {
                const key = event.currentTarget.value;
                const show = shows.find((item) => item.rating_key === key);
                if (show) {
                  selectShow(show);
                } else {
                  selectedShow = null;
                  selectedShowKey = "";
                  seasons = [];
                  selectedSeason = null;
                  selectedSeasonKey = "";
                }
              }}
            >
              <option value="">Select a show</option>
              {#each shows as show}
                <option value={show.rating_key}>{show.title}</option>
              {/each}
            </select>
          </div>
          <div class="field">
            <label for="trash-season">Season</label>
            <select
              id="trash-season"
              bind:value={selectedSeasonKey}
              on:change={(event) => {
                const key = event.currentTarget.value;
                const season = seasons.find((item) => item.rating_key === key);
                if (season) {
                  selectSeason(season);
                } else {
                  selectedSeason = null;
                  selectedSeasonKey = "";
                }
              }}
              disabled={!selectedShow}
            >
              <option value="">All seasons</option>
              {#each seasons as season}
                <option value={season.rating_key}>{season.title}</option>
              {/each}
            </select>
          </div>
          <p class="kicker" style="margin-top: 6px;">
            Selected: {selectedShow?.title || "None"}
            {#if selectedSeason}
              - {selectedSeason.title}
            {/if}
          </p>
        </div>

        <div class="panel trash-preview lift-2">
          <h2>Preview List</h2>
          <div class="debug">
            {#if preview.count === 0}
              <p class="kicker">Run a Dry Run to see results.</p>
            {:else}
              <p><strong>Items ({preview.count})</strong></p>
              {#each preview.titles as title}
                <p>{title}</p>
              {/each}
            {/if}
          </div>
        </div>

        <div class="panel trash-actions lift-3">
          <h2>Purge Controls</h2>
          <div class="kicker">
            Run a dry run to review matches, then purge trash for the selected show or season.
          </div>
          <div class="actions" style="margin-top: 18px;">
            <button
              data-variant="primary"
              on:click={openPurgeConfirm}
              disabled={!selectedShow || isBusy}
            >
              Purge Trash
            </button>
            <button
              data-variant="ghost"
              on:click={previewTrash}
              disabled={!selectedShow || isBusy}
            >
              Dry Run
            </button>
            <button
              data-variant="ghost"
              on:click={() => {
                selectedShow = null;
                selectedShowKey = "";
                selectedSeason = null;
                selectedSeasonKey = "";
                seasons = [];
                preview = { count: 0, titles: [] };
              }}
              disabled={isBusy}
            >
              Clear Selection
            </button>
          </div>
          <p class="kicker" style="margin-top: 12px;">
            Library: {selectedLibraryTitle || "--"}
          </p>
        </div>
      </section>
    {:else if activeTab === "subs"}
      <section class="grid subs-grid-wide">
        <div class="panel subs-scope lift-1">
          <h2>Library & Target</h2>
          <div class="field">
            <label for="library">TV Library</label>
            <select
              id="library"
              bind:value={selectedLibraryId}
              on:change={(event) => {
                const key = event.currentTarget.value;
                if (key) {
                  const library = libraries.find((item) => item.id === key);
                  if (library) {
                    selectLibrary(library);
                  }
                } else {
                  shows = [];
                  selectedShow = null;
                  selectedShowKey = "";
                  seasons = [];
                  selectedSeason = null;
                  selectedSeasonKey = "";
                }
              }}
            >
              <option value="">Select a library</option>
              {#each libraries as library}
                <option value={library.id}>{library.title}</option>
              {/each}
            </select>
          </div>
          <div class="field">
            <label for="show-select">Show</label>
            <select
              id="show-select"
              bind:value={selectedShowKey}
              on:change={(event) => {
                const key = event.currentTarget.value;
                const show = shows.find((item) => item.rating_key === key);
                if (show) {
                  selectShow(show);
                } else {
                  selectedShow = null;
                  seasons = [];
                  selectedSeason = null;
                  selectedSeasonKey = "";
                }
              }}
            >
              <option value="">Select a show</option>
              {#each filteredShows as show}
                <option value={show.rating_key}>{show.title}</option>
              {/each}
            </select>
          </div>
          <div class="field">
            <label for="season-select">Season</label>
            <select
              id="season-select"
              bind:value={selectedSeasonKey}
              on:change={(event) => {
                const key = event.currentTarget.value;
                const season = seasons.find((item) => item.rating_key === key);
                if (season) {
                  selectSeason(season);
                } else {
                  selectedSeason = null;
                  selectedSeasonKey = "";
                }
              }}
              disabled={!selectedShow}
            >
              <option value="">All seasons</option>
              {#each seasons as season}
                <option value={season.rating_key}>{season.title}</option>
              {/each}
            </select>
          </div>
        </div>

        <div class="panel subs-source lift-2">
          <h2>Subtitle Source</h2>
          <div class="actions">
            <button data-variant="ghost" on:click={pickSubsFolder}>
              Choose folder
            </button>
          </div>
          <div class="field" style="margin-top: 12px;">
            <label for="subs-root">Subtitles folder</label>
            <input
              id="subs-root"
              placeholder="D:\\Media\\Subtitles"
              bind:value={subsRoot}
            />
          </div>
          <div class="actions">
            <button
              data-variant="ghost"
              on:click={previewSubtitles}
              disabled={!selectedShow || isBusy}
            >
              Preview Mapping
            </button>
            <button
              data-variant="primary"
              on:click={() => (subsUploadPreConfirmOpen = true)}
              disabled={subsPreview.matched === 0 || subsUploadBusy}
            >
              Upload Subtitles
            </button>
          </div>
          <p class="kicker">
            Matching uses SxxEyy, 1x02, and other common patterns. Scope follows the
            selected show and season.
          </p>
        </div>

        <div class="panel subs-preview lift-3">
          <h2>Preview</h2>
          <div class="table">
            <div class="table-header subs-header">
              <span>Episode</span>
              <span>Subtitle</span>
              <span aria-hidden="true"></span>
            </div>
            {#if subsPreview.entries.length === 0}
              <p class="kicker">No preview yet.</p>
            {:else}
              {#each subsPreview.entries as entry}
                <div class="table-row subs-row">
                  <span class="subs-bubble subs-episode">
                    {entry.episodeTitle
                      ? `S${String(entry.season).padStart(2, "0")}E${String(entry.episode).padStart(2, "0")} - ${entry.episodeTitle}`
                      : "--"}
                  </span>
                  <span
                    class="subs-bubble subs-file"
                    class:expanded={subsExpandedKey === entry.path}
                    title={entry.fileName}
                    on:click={() => {
                      subsExpandedKey = subsExpandedKey === entry.path ? "" : entry.path;
                    }}
                  >
                    {entry.fileName}
                  </span>
                  <span class="kicker subs-status">
                    {#if entry.status.toLowerCase() === "matched"}
                      <span class="status-dot ok" aria-label="Matched"></span>
                    {:else}
                      <span class="status-dot" aria-label={entry.status}></span>
                    {/if}
                  </span>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      </section>
    {:else}
    <section class="grid plexmatch-row plexmatch-lookup">
      <div class="panel lift-1">
        <h2>TMDb Lookup</h2>
        <div class="lookup-row">
          <div class="field">
            <label for="tmdb-search">Series search</label>
            <input
              id="tmdb-search"
              placeholder="Type a series name"
              bind:value={tmdbQuery}
              on:keydown={(event) => {
                if (event.key === "Enter") {
                  searchTmdb();
                }
              }}
            />
          </div>
          <button data-variant="ghost" on:click={searchTmdb} disabled={isBusy}>
            Search TMDb
          </button>
          <div class="field">
            <label for="tmdb-series">Results</label>
            <select
              id="tmdb-series"
              bind:value={selectedSeriesId}
              on:change={(event) => {
                const id = Number(event.currentTarget.value);
                const series = tmdbResults.find((item) => item.id === id);
                if (series) {
                  selectSeries(series);
                } else {
                  selectedSeries = null;
                  selectedSeriesId = "";
                  tmdbEpisodes = [];
                }
              }}
            >
              <option value="">Select a series</option>
              {#each tmdbResults as series}
                <option value={String(series.id)}>
                  {series.name} ({series.first_air_year ?? "--"})
                </option>
              {/each}
            </select>
          </div>
        </div>
      </div>
    </section>

    <section class="grid plexmatch-row plexmatch-duo">
      <div class="panel lift-2">
        <h2>TMDb Episodes</h2>
        <div class="actions">
          <button data-variant="ghost" on:click={selectAllEpisodes}>
            Select all
          </button>
          <button data-variant="ghost" on:click={clearEpisodeSelection}>
            Clear selection
          </button>
        </div>
        <div class="field" style="margin-top: 10px;">
          <label>Episodes</label>
          <select
            id="tmdb-episodes"
            multiple
            size="16"
            bind:value={selectedEpisodeCodes}
            on:change={(event) => {
              selectedEpisodeCodes = readSelectedValues(event);
            }}
            class="fixed-list"
          >
            {#each tmdbEpisodes as ep}
              <option value={ep.code}>
                {ep.code} - {ep.name}
              </option>
            {/each}
          </select>
        </div>
      </div>

      <div class="panel lift-3">
        <h2>Video Files</h2>
        <div class="actions">
          <button data-variant="ghost" on:click={pickVideoFiles}>
            Add files
          </button>
          <button data-variant="ghost" on:click={clearVideoFiles}>
            Clear list
          </button>
        </div>
        <div class="field" style="margin-top: 10px;">
          <label>Files (drag to reorder)</label>
          <div
            class="select-list fixed-list"
            on:pointermove={movePointerDrag}
            on:pointerup={endPointerDrag}
            on:pointerleave={endPointerDrag}
          >
            {#if videoFiles.length === 0}
              <div class="kicker">No files added yet.</div>
            {:else}
              {#each videoFiles as file, index (file.path)}
                <div
                  class={`select-item file-row ${dragOverIndex === index ? "drag-over" : ""} ${dragIndex === index ? "dragging" : ""}`}
                  data-index={index}
                >
                  <span
                    class="drag-handle"
                    title="Drag"
                    on:pointerdown={(event) => startPointerDrag(event, index)}
                  >
                    ::
                  </span>
                  <span class="file-index">{index + 1}.</span>
                  <span class="file-name">{file.name}</span>
                  <div class="reorder-buttons">
                    <button
                      class="icon-btn small"
                      aria-label="Move up"
                      title="Move up"
                      disabled={index === 0}
                      on:click={() => moveFile(index, index - 1)}
                    >
                      ▲
                    </button>
                    <button
                      class="icon-btn small"
                      aria-label="Move down"
                      title="Move down"
                      disabled={index === videoFiles.length - 1}
                      on:click={() => moveFile(index, index + 1)}
                    >
                      ▼
                    </button>
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      </div>
    </section>

    <section class="grid plexmatch-row plexmatch-map">
      <div class="panel lift-2">
        <h2>Mapping</h2>
        <div class="actions">
          <button data-variant="mint" on:click={mapSelected}>
            Map selected
          </button>
          <button
            data-variant="ghost"
            on:click={() => {
              mappings = [];
              updatePlexmatchPreview();
            }}
          >
            Clear mapping
          </button>
        </div>
        <div class="table" style="margin-top: 10px;">
          <div class="table-header">
            <span>Episode</span>
            <span class="file-column">File</span>
            <span></span>
          </div>
          {#if mappings.length === 0}
            <p class="kicker">No mappings yet.</p>
          {:else}
            {#each mappings as map, index}
              <div class="table-row">
                <span class="mapping-episode">{map.code}</span>
                <span class="file-column mapping-file">
                  {buildDisplayPath(map.filePath, map.fileName)}
                </span>
                <button
                  class="icon-btn"
                  aria-label="Remove mapping"
                  title="Remove"
                  on:click={() => removeMapping(index)}
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path
                      d="M9 3h6l1 2h4v2H4V5h4l1-2zm1 6h2v9h-2V9zm4 0h2v9h-2V9zM7 9h2v9H7V9z"
                      fill="currentColor"
                    />
                  </svg>
                </button>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    </section>

    <section class="grid plexmatch-row plexmatch-preferences">
      <div class="panel lift-3">
        <h2>Mapping Preferences</h2>
        <div class="preferences-grid">
            <div>
              <div class="field">
                <label>Path depth</label>
                <div class="inline-field">
                  <button data-variant="ghost" on:click={decreasePathDepth}>
                    -
                  </button>
                  <div class="depth-pill">{pathDepth}</div>
                  <button data-variant="ghost" on:click={increasePathDepth}>
                    +
                  </button>
                  <span class="kicker">0 = parent folder only</span>
                </div>
              </div>
              <div class="field">
                <label>Export</label>
                <div class="actions">
                  <button data-variant="primary" on:click={savePlexmatch}>
                    Save .plexmatch
                  </button>
                </div>
              </div>
            </div>
            <div class="field">
              <label>Plexmatch preview</label>
              <textarea readonly rows="8" bind:value={plexmatchPreview}></textarea>
            </div>
          </div>
    </section>
    {/if}
</main>

<div class={`status-bar ${status.type}`}>
  {status.message || "Ready."}
</div>

{#if confirmOpen}
  <div class="modal-backdrop">
    <div class="modal">
      <h3>Confirm purge</h3>
      <p>
        This will remove all trashed items for <strong>{confirmTargetLabel}</strong>.
      </p>
      <div class="actions">
        <button data-variant="ghost" on:click={() => (confirmOpen = false)}>
          Cancel
        </button>
        <button data-variant="primary" on:click={confirmPurge} disabled={isBusy}>
          Purge Trash
        </button>
      </div>
    </div>
  </div>
{/if}

{#if purgeCompleteOpen}
  <div class="modal-backdrop">
    <div class="modal">
      <h3>Trash purge complete</h3>
      <p>Removed {purgeSummary.removed} trashed item(s).</p>
      <div class="actions">
        <button
          data-variant="primary"
          on:click={() => (purgeCompleteOpen = false)}
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}

{#if subsUploadBusy}
  <div class="modal-backdrop blocking">
    <div class="modal modal-loading">
      <div class="loader" aria-hidden="true"></div>
      <h3>Uploading subtitles</h3>
      <p>Please wait while PlexTools uploads your subtitle files.</p>
    </div>
  </div>
{/if}

{#if subsUploadPreConfirmOpen}
  <div class="modal-backdrop">
    <div class="modal">
      <h3>Confirm subtitle upload</h3>
      <p>
        Upload {subsPreview.matched} matched subtitle file(s) to Plex now?
      </p>
      <div class="actions">
        <button
          data-variant="ghost"
          on:click={() => (subsUploadPreConfirmOpen = false)}
        >
          Cancel
        </button>
        <button
          data-variant="primary"
          on:click={() => {
            subsUploadPreConfirmOpen = false;
            uploadSubtitles();
          }}
          disabled={subsUploadBusy}
        >
          Upload now
        </button>
      </div>
    </div>
  </div>
{/if}

{#if subsUploadConfirmOpen}
  <div class="modal-backdrop">
    <div class="modal">
      <h3>Subtitle upload complete</h3>
      {#if subsUploadSummary.failed > 0}
        <p>
          Uploaded {subsUploadSummary.uploaded} file(s) with
          {subsUploadSummary.failed} error(s).
        </p>
      {:else}
        <p>Uploaded {subsUploadSummary.uploaded} subtitle file(s) successfully.</p>
      {/if}

      <div class="actions">
        <button
          data-variant="primary"
          on:click={() => (subsUploadConfirmOpen = false)}
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}
{#if plexmatchSavedOpen}
  <div class="modal-backdrop">
    <div class="modal">
      <h3>PlexMatch saved</h3>
      <p>Saved to:</p>
      <p class="kicker">{plexmatchSavedPath}</p>
      <div class="actions">
        <button
          data-variant="primary"
          on:click={() => (plexmatchSavedOpen = false)}
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}

