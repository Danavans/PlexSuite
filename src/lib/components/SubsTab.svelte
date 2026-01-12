<script>
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
   */

  let subsRoot = $state("");
  /** @type {SubsPreview} */
  let subsPreview = $state({ total: 0, matched: 0, entries: [] });
  let subsUploadBusy = $state(false);
  let subsUploadPreConfirmOpen = $state(false);
  let subsUploadConfirmOpen = $state(false);
  /** @type {UploadSummary} */
  let subsUploadSummary = $state({ uploaded: 0, failed: 0 });
  let subsExpandedKey = $state("");

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
    const episodeNumber = null;
    appState.isBusy = true;
    appState.setStatus("info", "Scanning subtitle files...");
    try {
      const result = await invoke("preview_subtitles", {
        serverUrl: appState.serverUrl,
        token: appState.token,
        showRatingKey: appState.selectedShow.rating_key,
        seasonNumber: appState.selectedSeason?.index ?? null,
        episodeNumber,
        subsRoot
      });
      subsPreview = result;
      appState.setStatus(
        "success",
        `Matched ${result.matched}/${result.total} subtitle file(s).`
      );
    } catch (error) {
      appState.setStatus("error", `Preview failed: ${error}`);
    } finally {
      appState.isBusy = false;
    }
  }

  async function uploadSubtitles() {
    if (subsPreview.matched === 0) {
      appState.setStatus("error", "No matched subtitles to upload.");
      return;
    }
    subsUploadConfirmOpen = false;
    subsUploadBusy = true;
    appState.setStatus("info", "Uploading subtitles...");
    let result = null;
    try {
      const items = subsPreview.entries
        .filter((entry) => entry.episodeRatingKey)
        .map((entry) => ({
          path: entry.path,
          episodeRatingKey: entry.episodeRatingKey
        }));
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
</script>

<section class="grid subs-grid-wide" class:app-blocked={subsUploadBusy} aria-busy={subsUploadBusy} inert={subsUploadBusy}>
  <div class="panel subs-scope lift-1">
    <h2>Library & Target</h2>
    <div class="field">
      <label for="library">TV Library</label>
      <select
        id="library"
        bind:value={appState.selectedLibraryId}
        onchange={/** @param {Event} event */ (event) => {
          const key = event.currentTarget.value;
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
          const key = event.currentTarget.value;
          const show = appState.shows.find((item) => item.rating_key === key);
          if (show) {
            appState.selectShow(show);
          } else {
            appState.selectedShow = null;
            appState.seasons = [];
            appState.selectedSeason = null;
            appState.selectedSeasonKey = "";
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
          const key = event.currentTarget.value;
          const season = appState.seasons.find((item) => item.rating_key === key);
          if (season) {
            appState.selectSeason(season);
          } else {
            appState.selectedSeason = null;
            appState.selectedSeasonKey = "";
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
    <h2>Subtitle Source</h2>
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
      <button
        data-variant="primary"
        onclick={() => (subsUploadPreConfirmOpen = true)}
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
              role="button"
              tabindex="0"
              onclick={() => {
                subsExpandedKey = subsExpandedKey === entry.path ? "" : entry.path;
              }}
              onkeydown={/** @param {KeyboardEvent} event */ (event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  subsExpandedKey =
                    subsExpandedKey === entry.path ? "" : entry.path;
                }
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
        Upload {subsPreview.matched} matched subtitle file(s) to Plex now?
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