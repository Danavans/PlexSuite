<script>
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { appState } from "../appState.svelte.js";

  /**
   * @typedef {{ id: number, name: string, first_air_year?: number }} TmdbSeries
   * @typedef {{ code: string, name: string }} TmdbEpisode
   * @typedef {{ name: string, path: string }} VideoFile
   * @typedef {{ code: string, fileName: string, filePath: string }} PlexmatchMapping
   */

  let tmdbQuery = $state("");
  /** @type {TmdbSeries[]} */
  let tmdbResults = $state([]);
  /** @type {TmdbEpisode[]} */
  let tmdbEpisodes = $state([]);
  /** @type {TmdbSeries | null} */
  let selectedSeries = $state(null);
  let selectedSeriesId = $state("");
  /** @type {string[]} */
  let selectedEpisodeCodes = $state([]);

  let pathDepth = $state(0);
  let plexmatchOutputPath = $state("");
  /** @type {VideoFile[]} */
  let videoFiles = $state([]);
  /** @type {PlexmatchMapping[]} */
  let mappings = $state([]);
  let plexmatchPreview = $state("");
  let plexmatchSavedOpen = $state(false);
  let plexmatchSavedPath = $state("");

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

  /**
   * @param {string} path
   * @returns {string}
   */
  function normalizePath(path) {
    return path.replace(/\\/g, "/");
  }

  /**
   * @param {string} filePath
   * @param {string | undefined} fileName
   * @returns {string}
   */
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

  /**
   * @returns {number}
   */
  function getMaxPathDepth() {
    let maxDepth = 0;
    for (const file of videoFiles) {
      const parts = normalizePath(file.path).split("/").filter(Boolean);
      const parentCount = Math.max(0, parts.length - 1);
      const depth = Math.max(0, parentCount - 1);
      if (depth > maxDepth) {
        maxDepth = depth;
      }
    }
    return maxDepth;
  }

  $effect(() => {
    updatePlexmatchPreview();
  });

  $effect(() => {
    const maxDepth = getMaxPathDepth();
    if (pathDepth > maxDepth) {
      pathDepth = maxDepth;
    }
  });

  async function searchTmdb() {
    if (!appState.tmdbKey.trim()) {
      appState.setStatus("error", "TMDb API key is required.");
      return;
    }
    if (!tmdbQuery.trim()) {
      appState.setStatus("error", "Enter a series name.");
      return;
    }
    appState.isBusy = true;
    appState.setStatus("info", "Searching TMDb...");
    try {
      const result = await invoke("tmdb_search_series", {
        apiKey: appState.tmdbKey,
        query: tmdbQuery
      });
      tmdbResults = result;
      selectedSeries = null;
      selectedSeriesId = "";
      tmdbEpisodes = [];
      appState.setStatus("success", `Found ${result.length} result(s).`);
    } catch (error) {
      appState.setStatus("error", `TMDb search failed: ${error}`);
    } finally {
      appState.isBusy = false;
    }
  }

  /**
   * @param {TmdbSeries} series
   */
  async function selectSeries(series) {
    selectedSeries = series;
    selectedSeriesId = String(series?.id ?? "");
    appState.isBusy = true;
    appState.setStatus("info", "Loading episodes...");
    try {
      const result = await invoke("tmdb_list_episodes", {
        apiKey: appState.tmdbKey,
        seriesId: series.id
      });
      tmdbEpisodes = result;
      selectedEpisodeCodes = [];
      appState.setStatus("success", `Loaded ${result.length} episodes.`);
    } catch (error) {
      appState.setStatus("error", `TMDb load failed: ${error}`);
    } finally {
      appState.isBusy = false;
    }
  }

  /**
   * @param {string[]} paths
   */
  function addFilePaths(paths) {
    const next = [...videoFiles];
    for (const fullPath of paths) {
      // Normalize separators to '/' then split
      const name = fullPath.replace(/\\/g, "/").split("/").pop();
      if (!name) continue;
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
    const maxDepth = getMaxPathDepth();
    if (pathDepth < maxDepth) {
      pathDepth += 1;
    }
  }

  function decreasePathDepth() {
    pathDepth = Math.max(-1, pathDepth - 1);
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

  /**
   * @param {number} index
   */
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
      appState.setStatus("error", `Save failed: ${error}`);
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

  /**
   * @param {Event} event
   * @returns {string[]}
   */
  function readSelectedValues(event) {
    const target = /** @type {HTMLSelectElement} */ (event.currentTarget);
    return Array.from(target.selectedOptions).map((opt) => opt.value);
  }

  // Drag and Drop Logic
  /** @type {number | null} */
  let dragIndex = $state(null);
  /** @type {number | null} */
  let dragOverIndex = $state(null);
  let dragActive = $state(false);
  /** @type {number | null} */
  let mappingDragIndex = $state(null);
  /** @type {number | null} */
  let mappingDragOverIndex = $state(null);
  let mappingDragActive = $state(false);
  /** @type {number | null} */
  let mappingEpisodeDragIndex = $state(null);
  /** @type {number | null} */
  let mappingEpisodeDragOverIndex = $state(null);
  let mappingEpisodeDragActive = $state(false);

  /**
   * @param {PointerEvent} event
   * @param {number} index
   */
  function startPointerDrag(event, index) {
    if (event.button !== 0) return;
    const target = event.target;
    if (target instanceof HTMLElement) {
      if (target.closest("button")) {
        return;
      }
    }
    dragActive = true;
    dragIndex = index;
    dragOverIndex = index;
    
    // Explicit casting to HTMLElement to access .closest()
    const currentTarget = /** @type {HTMLElement} */ (event.currentTarget);
    const list = currentTarget.closest(".select-list");
    
    const captureTarget = list ?? currentTarget;
    captureTarget.setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  /**
   * @param {PointerEvent} event
   */
  function movePointerDrag(event) {
    if (!dragActive) return;
    if (dragIndex === null) return;
    const list = /** @type {HTMLElement | null} */ (event.currentTarget);
    if (list) {
      const rect = list.getBoundingClientRect();
      const threshold = 36;
      const minSpeed = 2;
      const maxSpeed = 12;
      if (event.clientY < rect.top + threshold) {
        const proximity = (rect.top + threshold - event.clientY) / threshold;
        const speed = Math.round(minSpeed + proximity * (maxSpeed - minSpeed));
        list.scrollTop -= speed;
      } else if (event.clientY > rect.bottom - threshold) {
        const proximity = (event.clientY - (rect.bottom - threshold)) / threshold;
        const speed = Math.round(minSpeed + proximity * (maxSpeed - minSpeed));
        list.scrollTop += speed;
      }
    }
    const el = document.elementFromPoint(event.clientX, event.clientY);
    const row = /** @type {HTMLElement | null} */ (
      el?.closest?.(".file-row") ?? null
    );
    if (!row) return;
    const index = Number(row.dataset.index);
    if (Number.isNaN(index) || index === dragIndex) return;
    moveFile(dragIndex, index);
    dragIndex = index;
    dragOverIndex = index;
  }

  /**
   * @param {PointerEvent} event
   */
  function endPointerDrag(event) {
    if (!dragActive) return;
    dragActive = false;
    dragIndex = null;
    dragOverIndex = null;
    const target = /** @type {HTMLElement} */ (event.currentTarget);
    target.releasePointerCapture(event.pointerId);
  }

  /**
   * @param {PointerEvent} event
   * @param {number} index
   */
  function startMappingDrag(event, index) {
    if (event.button !== 0) return;
    const target = event.target;
    if (target instanceof HTMLElement) {
      if (target.closest("button")) {
        return;
      }
    }
    mappingEpisodeDragActive = false;
    mappingEpisodeDragIndex = null;
    mappingEpisodeDragOverIndex = null;
    mappingDragActive = true;
    mappingDragIndex = index;
    mappingDragOverIndex = index;
    
    const currentTarget = /** @type {HTMLElement} */ (event.currentTarget);
    const list = currentTarget.closest(".mapping-table");
    
    const captureTarget = list ?? currentTarget;
    captureTarget.setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  /**
   * @param {PointerEvent} event
   * @param {number} index
   */
  function startMappingEpisodeDrag(event, index) {
    if (event.button !== 0) return;
    const target = event.target;
    if (target instanceof HTMLElement) {
      if (target.closest("button")) {
        return;
      }
    }
    mappingDragActive = false;
    mappingDragIndex = null;
    mappingDragOverIndex = null;
    mappingEpisodeDragActive = true;
    mappingEpisodeDragIndex = index;
    mappingEpisodeDragOverIndex = index;
    
    const currentTarget = /** @type {HTMLElement} */ (event.currentTarget);
    const list = currentTarget.closest(".mapping-table");
    
    const captureTarget = list ?? currentTarget;
    captureTarget.setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  /**
   * @param {PointerEvent} event
   */
  function moveMappingDrag(event) {
    if (!mappingDragActive && !mappingEpisodeDragActive) return;
    const list = /** @type {HTMLElement | null} */ (event.currentTarget);
    if (list) {
      const rect = list.getBoundingClientRect();
      const threshold = 36;
      const minSpeed = 2;
      const maxSpeed = 12;
      if (event.clientY < rect.top + threshold) {
        const proximity = (rect.top + threshold - event.clientY) / threshold;
        const speed = Math.round(minSpeed + proximity * (maxSpeed - minSpeed));
        list.scrollTop -= speed;
      } else if (event.clientY > rect.bottom - threshold) {
        const proximity = (event.clientY - (rect.bottom - threshold)) / threshold;
        const speed = Math.round(minSpeed + proximity * (maxSpeed - minSpeed));
        list.scrollTop += speed;
      }
    }
    const el = document.elementFromPoint(event.clientX, event.clientY);
    const row = /** @type {HTMLElement | null} */ (
      el?.closest?.(".mapping-row") ?? null
    );
    if (!row) return;
    const index = Number(row.dataset.index);
    if (Number.isNaN(index)) return;
    if (mappingDragActive) {
      if (mappingDragIndex === null || index === mappingDragIndex) return;
      moveMappingFile(mappingDragIndex, index);
      mappingDragIndex = index;
      mappingDragOverIndex = index;
      return;
    }
    if (mappingEpisodeDragIndex === null || index === mappingEpisodeDragIndex) return;
    moveMappingEpisode(mappingEpisodeDragIndex, index);
    mappingEpisodeDragIndex = index;
    mappingEpisodeDragOverIndex = index;
  }

  /**
   * @param {PointerEvent} event
   */
  function endMappingDrag(event) {
    if (!mappingDragActive && !mappingEpisodeDragActive) return;
    mappingDragActive = false;
    mappingDragIndex = null;
    mappingDragOverIndex = null;
    mappingEpisodeDragActive = false;
    mappingEpisodeDragIndex = null;
    mappingEpisodeDragOverIndex = null;
    const target = /** @type {HTMLElement} */ (event.currentTarget);
    target.releasePointerCapture(event.pointerId);
  }

  /**
   * @param {number} fromIndex
   * @param {number} toIndex
   */
  function moveFile(fromIndex, toIndex) {
    if (fromIndex < 0 || toIndex < 0) return;
    if (fromIndex === toIndex) return;
    const next = [...videoFiles];
    const [moved] = next.splice(fromIndex, 1);
    next.splice(toIndex, 0, moved);
    videoFiles = next;
  }

  /**
   * @param {number} fromIndex
   * @param {number} toIndex
   */
  function moveMappingFile(fromIndex, toIndex) {
    if (fromIndex < 0 || toIndex < 0) return;
    if (fromIndex === toIndex) return;
    const next = [...mappings];
    const files = next.map((item) => ({
      fileName: item.fileName,
      filePath: item.filePath
    }));
    const [moved] = files.splice(fromIndex, 1);
    if (!moved) return;
    files.splice(toIndex, 0, moved);
    mappings = next.map((item, index) => ({
      ...item,
      fileName: files[index].fileName,
      filePath: files[index].filePath
    }));
  }

  /**
   * @param {number} fromIndex
   * @param {number} toIndex
   */
  function moveMappingEpisode(fromIndex, toIndex) {
    if (fromIndex < 0 || toIndex < 0) return;
    if (fromIndex === toIndex) return;
    const next = [...mappings];
    const codes = next.map((item) => item.code);
    const [moved] = codes.splice(fromIndex, 1);
    if (!moved) return;
    codes.splice(toIndex, 0, moved);
    mappings = next.map((item, index) => ({
      ...item,
      code: codes[index]
    }));
  }
</script>

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
          onkeydown={/** @param {KeyboardEvent} event */ (event) => {
            if (event.key === "Enter") {
              searchTmdb();
            }
          }}
        />
      </div>
      <button data-variant="ghost" onclick={searchTmdb} disabled={appState.isBusy}>
        Search TMDb
      </button>
      <div class="field">
        <label for="tmdb-series">Results</label>
        <select
          id="tmdb-series"
          bind:value={selectedSeriesId}
          onchange={/** @param {Event} event */ (event) => {
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
      <button data-variant="ghost" onclick={selectAllEpisodes}>
        Select all
      </button>
      <button data-variant="ghost" onclick={clearEpisodeSelection}>
        Clear selection
      </button>
    </div>
    <div class="field" style="margin-top: 10px;">
      <label for="tmdb-episodes">Episodes</label>
      <select
        id="tmdb-episodes"
        multiple
        size="16"
        bind:value={selectedEpisodeCodes}
        onchange={/** @param {Event} event */ (event) => {
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
      <button data-variant="ghost" onclick={pickVideoFiles}>
        Add files
      </button>
      <button data-variant="ghost" onclick={clearVideoFiles}>
        Clear list
      </button>
    </div>
    <div class="field" style="margin-top: 10px;">
      <span class="field-label">Files (drag to reorder)</span>
      <div
        class="select-list fixed-list"
        onpointermove={movePointerDrag}
        onpointerup={endPointerDrag}
        onpointerleave={endPointerDrag}
      >
        {#if videoFiles.length === 0}
          <div class="kicker">No files added yet.</div>
        {:else}
          {#each videoFiles as file, index (file.path)}
            <div
              class={`select-item file-row ${dragOverIndex === index ? "drag-over" : ""} ${dragIndex === index ? "dragging" : ""}`}
              data-index={index}
              onpointerdown={/** @param {PointerEvent} event */ (event) => startPointerDrag(event, index)}
            >
              <span class="file-index">{index + 1}.</span>
              <span class="file-name">{file.name}</span>
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
      <button data-variant="mint" onclick={mapSelected}>
        Map selected
      </button>
      <button
        data-variant="ghost"
        onclick={() => {
          mappings = [];
          updatePlexmatchPreview();
        }}
      >
        Clear mapping
      </button>
    </div>
    <div
      class="table mapping-table"
      style="margin-top: 10px;"
      onpointermove={moveMappingDrag}
      onpointerup={endMappingDrag}
      onpointerleave={endMappingDrag}
    >
      <div class="table-header">
        <span>Episode</span>
        <span class="file-column">File</span>
        <span></span>
      </div>
      {#if mappings.length === 0}
        <p class="kicker">No mappings yet.</p>
      {:else}
        {#each mappings as map, index}
          <div
            class="table-row mapping-row"
            data-index={index}
          >
            <span
              class={`mapping-episode ${mappingEpisodeDragOverIndex === index ? "drag-over" : ""} ${mappingEpisodeDragIndex === index ? "dragging" : ""}`}
              onpointerdown={/** @param {PointerEvent} event */ (event) => startMappingEpisodeDrag(event, index)}
            >
              {map.code}
            </span>
            <span
              class={`file-column mapping-file ${mappingDragOverIndex === index ? "drag-over" : ""} ${mappingDragIndex === index ? "dragging" : ""}`}
              onpointerdown={/** @param {PointerEvent} event */ (event) => startMappingDrag(event, index)}
            >
              {buildDisplayPath(map.filePath, map.fileName)}
            </span>
            <button
              class="icon-btn"
              aria-label="Remove mapping"
              title="Remove"
              onclick={() => removeMapping(index)}
            >
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <circle
                  cx="12"
                  cy="12"
                  r="9"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                />
                <line
                  x1="7"
                  y1="7"
                  x2="17"
                  y2="17"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
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
            <span class="field-label">Path depth</span>
            <div class="inline-field">
              <button data-variant="ghost" onclick={decreasePathDepth}>
                -
              </button>
              <div class="depth-pill">{pathDepth}</div>
              <button data-variant="ghost" onclick={increasePathDepth}>
                +
              </button>
              <span class="kicker">0 = parent folder only</span>
            </div>
          </div>
          <div class="field">
            <span class="field-label">Export</span>
            <div class="actions">
              <button data-variant="primary" onclick={savePlexmatch}>
                Save .plexmatch
              </button>
            </div>
          </div>
        </div>
        <div class="field">
          <label for="plexmatch-preview">Plexmatch preview</label>
          <textarea id="plexmatch-preview" readonly rows="8" bind:value={plexmatchPreview}></textarea>
        </div>
      </div>
    </div>
</section>

{#if plexmatchSavedOpen}
  <div class="modal-backdrop">
    <div class="modal">
      <h3>PlexMatch saved</h3>
      <p>Saved to:</p>
      <p class="kicker">{plexmatchSavedPath}</p>
      <div class="actions">
        <button
          data-variant="primary"
          onclick={() => (plexmatchSavedOpen = false)}
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}
