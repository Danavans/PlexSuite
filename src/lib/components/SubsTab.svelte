<script>
  import EmptyState from "./EmptyState.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { appState } from "../appState.svelte.js";

  /**
   * @typedef {{ uploaded: number, failed: number }} UploadSummary
   * @typedef {{ total: number, matched: number, entries: SubsPreviewEntry[] }} SubsPreview
   * @typedef {{
   *   episodeTitle?: string,
   *   season?: number,
   *   episode?: number,
   *   fileName: string,
   *   path: string,
   *   status: string,
   *   episodeRatingKey?: string
   * }} SubsPreviewEntry
   * @typedef {{ rating_key: string, title: string, season_number: number, episode_number: number }} PlexEpisode
   * @typedef {{ episode: PlexEpisode | null, file: SubsPreviewEntry | null }} MappingRow
   */

  let subsRoot = $state("");
  /** @type {MappingRow[]} */
  let mappingRows = $state([]);
  let subsUploadBusy = $state(false);
  let subsUploadPreConfirmOpen = $state(false);
  let subsUploadConfirmOpen = $state(false);
  /** @type {UploadSummary} */
  let subsUploadSummary = $state({ uploaded: 0, failed: 0 });

  // UI State
  /** @type {number | null} */
  let expandedIndex = $state(null); // Track which row is expanded

  // Drag & Drop State
  /** @type {number | null} */
  let dragIndex = $state(null);
  /** @type {number | null} */
  let dragOverIndex = $state(null);
  let dragActive = $state(false);
  let hasMoved = $state(false); // To distinguish click from drag

  async function pickSubsFolder() {
    const result = await open({
      directory: true,
      multiple: false
    });
    if (typeof result === "string") {
      subsRoot = result;
    }
  }

  async function previewSubtitles() {
    if (!appState.selectedShow) {
      appState.setStatus("error", "Choose a show first.");
      return;
    }
    if (!subsRoot.trim()) {
      appState.setStatus("error", "Provide a subtitles folder.");
      return;
    }
    
    appState.isBusy = true;
    appState.setStatus("info", "Scanning and mapping...");
    
    try {
      /** @type {PlexEpisode[]} */
      const episodes = await invoke("list_episodes", {
        serverUrl: appState.serverUrl,
        token: appState.token,
        showRatingKey: appState.selectedShow.rating_key,
        seasonRatingKey: appState.selectedSeason?.rating_key ?? null,
      });

      /** @type {SubsPreview} */
      const scanResult = await invoke("preview_subtitles", {
        serverUrl: appState.serverUrl,
        token: appState.token,
        showRatingKey: appState.selectedShow.rating_key,
        seasonNumber: appState.selectedSeason?.index ?? null,
        episodeNumber: null,
        subsRoot
      });

      /** @type {MappingRow[]} */
      const newRows = episodes.map(ep => ({
        episode: ep,
        file: null
      }));

      const usedFilePaths = new Set();
      for (const entry of scanResult.entries) {
        if (entry.episodeRatingKey) {
          const row = newRows.find(r => r.episode?.rating_key === entry.episodeRatingKey);
          if (row && !row.file) {
            row.file = entry;
            usedFilePaths.add(entry.path);
          }
        }
      }

      for (const entry of scanResult.entries) {
        if (!usedFilePaths.has(entry.path)) {
          newRows.push({
            episode: null,
            file: entry
          });
        }
      }

      mappingRows = newRows;
      expandedIndex = null; // Reset expansion
      
      const matchedCount = newRows.filter(r => r.episode && r.file).length;
      appState.setStatus(
        "success",
        `Mapped ${matchedCount} file(s) to ${episodes.length} episodes.`
      );

    } catch (error) {
      appState.setStatus("error", `Preview failed: ${error}`);
    } finally {
      appState.isBusy = false;
    }
  }

  async function uploadSubtitles() {
    const validMappings = mappingRows.filter(r => r.episode && r.file);
    if (validMappings.length === 0) {
      appState.setStatus("error", "No mapped subtitles to upload.");
      return;
    }
    
    subsUploadConfirmOpen = false;
    subsUploadBusy = true;
    appState.setStatus("info", "Uploading subtitles...");
    
    let result = null;
    try {
      const items = validMappings.map(row => ({
        path: row.file ? row.file.path : "",
        episodeRatingKey: row.episode ? row.episode.rating_key : ""
      })).filter(item => item.path && item.episodeRatingKey);
      
      result = await invoke("upload_subtitles", {
        serverUrl: appState.serverUrl,
        token: appState.token,
        items
      });
      
      if (result.failed.length > 0) {
        appState.setStatus(
          "error",
          `Uploaded ${result.uploaded} with ${result.failed.length} error(s).`
        );
      } else {
        appState.setStatus("success", `Uploaded ${result.uploaded} subtitle file(s).`);
      }
    } catch (error) {
      appState.setStatus("error", `Upload failed: ${error}`);
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

  // --- Drag & Drop Logic ---

  /**
   * @param {PointerEvent} event
   * @param {number} index
   */
  function startDrag(event, index) {
    if (event.button !== 0) return;
    const row = mappingRows[index];
    if (!row || !row.file) return;

    dragActive = true;
    dragIndex = index;
    dragOverIndex = index;
    hasMoved = false; // Reset movement flag

    const currentTarget = /** @type {HTMLElement} */ (event.currentTarget);
    const list = currentTarget.closest(".mapping-table");
    const captureTarget = list ?? currentTarget;
    captureTarget.setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  /**
   * @param {PointerEvent} event
   */
  function moveDrag(event) {
    if (!dragActive) return;
    
    const currentTarget = /** @type {HTMLElement} */ (event.currentTarget);
    const rect = currentTarget.getBoundingClientRect();
    const threshold = 36;
    if (event.clientY < rect.top + threshold) {
      currentTarget.scrollTop -= 8;
    } else if (event.clientY > rect.bottom - threshold) {
      currentTarget.scrollTop += 8;
    }

    const el = document.elementFromPoint(event.clientX, event.clientY);
    const rowEl = /** @type {HTMLElement | null} */ (el?.closest?.(".mapping-row") ?? null);
    if (!rowEl) return;
    
    const index = Number(rowEl.dataset.index);
    if (Number.isNaN(index) || index === dragIndex) return;

    // If we reach here, we are swapping, so it's definitely a drag, not a click.
    hasMoved = true;
    if (dragIndex !== null) {
        swapFiles(dragIndex, index);
        dragIndex = index;
        dragOverIndex = index;
    }
  }

  /**
   * @param {PointerEvent} event
   */
  function endDrag(event) {
    if (!dragActive) return;
    
    // If we didn't move/swap, treat it as a click to expand
    if (!hasMoved && dragIndex !== null) {
      if (expandedIndex === dragIndex) {
        expandedIndex = null;
      } else {
        expandedIndex = dragIndex;
      }
    }

    dragActive = false;
    dragIndex = null;
    dragOverIndex = null;
    const target = /** @type {HTMLElement} */ (event.currentTarget);
    target.releasePointerCapture(event.pointerId);
  }

  /**
   * @param {number} fromIndex
   * @param {number} toIndex
   */
  function swapFiles(fromIndex, toIndex) {
    if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) return;
    const next = [...mappingRows];
    const fileFrom = next[fromIndex].file;
    const fileTo = next[toIndex].file;
    next[fromIndex] = { ...next[fromIndex], file: fileTo };
    next[toIndex] = { ...next[toIndex], file: fileFrom };
    mappingRows = next;
  }
</script>

<section class="grid subs-grid-wide" class:app-blocked={subsUploadBusy} aria-busy={subsUploadBusy} inert={subsUploadBusy}>
  <div class="panel subs-scope lift-1">
    <h2><span class="step-number">01</span> Library & target</h2>
    <div class="field">
      <label for="library">TV Library</label>
      <select
        id="library"
        bind:value={appState.selectedLibraryId}
        onchange={/** @param {Event} event */ (event) => {
          const target = /** @type {HTMLSelectElement} */ (event.target);
          const key = target.value;
          if (key) {
            const library = appState.libraries.find((item) => item.id === key);
            if (library) {
              appState.selectLibrary(library);
            }
          } else {
            appState.shows = [];
            appState.selectedShow = null;
            appState.selectedShowKey = "";
            appState.seasons = [];
            appState.selectedSeason = null;
            appState.selectedSeasonKey = "";
          }
        }}
      >
        <option value="">Select a library</option>
        {#each appState.libraries as library}
          <option value={library.id}>{library.title}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="show-select">Show</label>
      <select
        id="show-select"
        bind:value={appState.selectedShowKey}
        onchange={/** @param {Event} event */ (event) => {
          const target = /** @type {HTMLSelectElement} */ (event.target);
          const key = target.value;
          const show = appState.shows.find((item) => item.rating_key === key);
          if (show) {
            appState.selectShow(show);
          }
        }}
      >
        <option value="">Select a show</option>
        {#each appState.shows as show}
          <option value={show.rating_key}>{show.title}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="season-select">Season</label>
      <select
        id="season-select"
        bind:value={appState.selectedSeasonKey}
        onchange={/** @param {Event} event */ (event) => {
          const target = /** @type {HTMLSelectElement} */ (event.target);
          const key = target.value;
          const season = appState.seasons.find((item) => item.rating_key === key);
          if (season) {
            appState.selectSeason(season);
          }
        }}
        disabled={!appState.selectedShow}
      >
        <option value="">All seasons</option>
        {#each appState.seasons as season}
          <option value={season.rating_key}>{season.title}</option>
        {/each}
      </select>
    </div>
  </div>

  <div class="panel subs-source lift-2">
    <h2><span class="step-number">02</span> Subtitle source</h2>
    <div class="actions">
      <button data-variant="ghost" onclick={pickSubsFolder}>
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
        onclick={previewSubtitles}
        disabled={!appState.selectedShow || appState.isBusy}
      >
        Preview Mapping
      </button>
    </div>
    <p class="kicker">
      Drag files to reorder mapping. Click a file to view full name.
    </p>
  </div>

  <div class="panel subs-preview lift-3">
    <h2><span class="step-number">03</span> Review mapping</h2>
    <div 
      class="table mapping-table"
      onpointermove={moveDrag}
      onpointerup={endDrag}
      onpointerleave={endDrag}
    >
      <div class="table-header subs-header">
        <span>Episode</span>
        <span>Subtitle File</span>
        <span aria-hidden="true"></span>
      </div>
      {#if mappingRows.length === 0}
        <EmptyState title="Every subtitle, in its place" description="Choose a target and subtitle folder, then Preview Mapping. Drag files between rows to adjust matches before uploading." />
      {:else}
        {#each mappingRows as row, index}
          <div 
            class="table-row mapping-row subs-row"
            data-index={index}
          >
            <!-- Episode Column -->
            <span class="subs-bubble subs-episode">
              {#if row.episode}
                S{String(row.episode.season_number).padStart(2, "0")}E{String(row.episode.episode_number).padStart(2, "0")} - {row.episode.title}
              {:else}
                <span class="unassigned-text">Unassigned</span>
              {/if}
            </span>

            <!-- Draggable File Column -->
            <span
              class="subs-bubble subs-file"
              class:empty={!row.file}
              class:expanded={expandedIndex === index}
              class:drag-over={dragOverIndex === index}
              class:dragging={dragIndex === index}
              title={row.file?.fileName || "Empty slot"}
              role="button"
              tabindex="0"
              onpointerdown={/** @param {PointerEvent} event */ (event) => startDrag(event, index)}
            >
              {#if row.file}
                {row.file.fileName}
              {:else}
                <span class="empty-text">-- Empty --</span>
              {/if}
            </span>

            <!-- Status Icon -->
            <span class="kicker subs-status">
              {#if row.episode && row.file}
                <span class="status-dot ok" aria-label="Ready"></span>
              {:else if row.file}
                 <span class="status-dot warn" aria-label="Unassigned"></span>
              {:else}
                 <span class="status-dot" aria-label="Missing"></span>
              {/if}
            </span>
          </div>
        {/each}
      {/if}
    </div>
    <div class="review-actions"><p class="kicker">Review episode matches before uploading to Plex.</p><div class="actions">
      <button
        data-variant="primary"
        onclick={() => (subsUploadPreConfirmOpen = true)}
        disabled={mappingRows.filter(r => r.episode && r.file).length === 0 || subsUploadBusy}
      >
        Upload Subtitles
      </button>
    </div></div>
  </div>
</section>

{#if subsUploadBusy}
  <div class="modal-backdrop blocking">
    <div class="modal modal-loading">
      <div class="loader" aria-hidden="true"></div>
      <h3>Uploading subtitles</h3>
      <p>Please wait while PlexSuite uploads your subtitle files.</p>
    </div>
  </div>
{/if}

{#if subsUploadPreConfirmOpen}
  <div class="modal-backdrop">
    <div class="modal">
      <h3>Confirm subtitle upload</h3>
      <p>
        Upload {mappingRows.filter(r => r.episode && r.file).length} matched subtitle file(s) to Plex now?
      </p>
      <div class="actions">
        <button
          data-variant="ghost"
          onclick={() => (subsUploadPreConfirmOpen = false)}
        >
          Cancel
        </button>
        <button
          data-variant="primary"
          onclick={() => {
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
          onclick={() => (subsUploadConfirmOpen = false)}
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .unassigned-text {
    color: var(--muted);
    font-style: italic;
  }
  .empty-text {
    color: var(--muted);
  }
  .subs-file {
    cursor: grab;
    transition: transform 0.1s ease, box-shadow 0.1s ease, background 0.1s ease;
    user-select: none;
    white-space: nowrap; /* Default: truncate */
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .subs-file:active {
    cursor: grabbing;
  }
  .subs-file.expanded {
    white-space: normal; /* Clicked: wrap text */
    overflow: visible;
  }
  .subs-file.empty {
    border: 1px dashed var(--stroke);
    background: transparent;
  }
  .subs-file.dragging {
    opacity: 0.6;
    transform: scale(1.02);
    box-shadow: 0 10px 20px rgba(0, 0, 0, 0.3);
    z-index: 10;
  }
  .subs-file.drag-over {
    border-color: var(--ok-green);
    background: rgba(39, 241, 120, 0.1);
  }
  .warn {
    background-color: #f5a623;
    box-shadow: 0 0 0 2px rgba(245, 166, 35, 0.28);
  }
  /* Ensure column alignment */
  .subs-row {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) 20px;
    gap: 4px;
  }
  .subs-header {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) 20px;
    padding: 0 12px;
  }
</style>
