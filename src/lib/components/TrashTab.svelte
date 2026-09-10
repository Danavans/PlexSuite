<script>
  import EmptyState from "./EmptyState.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../appState.svelte.js";

  /**
   * @typedef {{ count: number, titles: string[], diagnostics?: string[] }} TrashPreview
   * @typedef {{ removed: number }} PurgeSummary
   */

  /** @type {TrashPreview} */
  let preview = $state({ count: 0, titles: [] });
  let confirmOpen = $state(false);
  let confirmTargetLabel = $state("");
  /** @type {PurgeSummary} */
  let purgeSummary = $state({ removed: 0 });
  let purgeCompleteOpen = $state(false);

  async function previewTrash() {
    if (!appState.selectedShow) {
      appState.setStatus("error", "Choose a show first.");
      return;
    }
    appState.isBusy = true;
    appState.setStatus("info", "Previewing trash items...");
    try {
      appState.addDebugLog(`Preview started: ${appState.selectedLibraryTitle} / ${appState.selectedShow.title} / ${appState.selectedSeason?.title || "All Seasons"}`, "Trash");
      const result = await invoke("preview_trash", {
        serverUrl: appState.serverUrl,
        token: appState.token,
        libraryId: appState.selectedLibraryId,
        showRatingKey: appState.selectedShow.rating_key,
        seasonRatingKey: appState.selectedSeason?.rating_key ?? null,
        debug: appState.debugMode,
      });
      for (const line of result.diagnostics ?? []) appState.addDebugLog(line, "Trash");
      preview = result;
      appState.setStatus("success", `Found ${result.count} trashed item(s).`);
    } catch (error) {
      appState.setStatus("error", `Preview failed: ${error}`);
    } finally {
      appState.isBusy = false;
    }
  }

  function openPurgeConfirm() {
    if (!appState.selectedShow) {
      appState.setStatus("error", "Choose a show first.");
      return;
    }
    confirmTargetLabel = appState.selectedSeason
      ? `${appState.selectedShow.title} - ${appState.selectedSeason.title}`
      : appState.selectedShow.title;
    confirmOpen = true;
  }

  async function confirmPurge() {
    if (!appState.selectedShow) {
      appState.setStatus("error", "Choose a show first.");
      return;
    }
    confirmOpen = false;
    appState.isBusy = true;
    appState.setStatus("info", "Purging trash...");
    try {
      const result = await invoke("purge_trash", {
        serverUrl: appState.serverUrl,
        token: appState.token,
        libraryId: appState.selectedLibraryId,
        showRatingKey: appState.selectedShow.rating_key,
        seasonRatingKey: appState.selectedSeason?.rating_key ?? null,
      });
      purgeSummary = { removed: result.removed_count };
      purgeCompleteOpen = true;
      appState.setStatus("success", `Removed ${result.removed_count} trashed item(s).`);
    } catch (error) {
      appState.setStatus("error", `Purge failed: ${error}`);
    } finally {
      appState.isBusy = false;
    }
  }
</script>

<section class="grid trash-grid">
  <div class="panel trash-library lift-1">
    <h2><span class="step-number">01</span> Library scope</h2>
    <div class="kicker" style="margin-bottom: 10px;">
      Choose a library, show, and optional season to define the purge scope.
    </div>
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
             // Reset logic handled in appState.selectLibrary or manually if needed, 
             // but appState.selectLibrary handles the reset of children.
             // If empty, we might want to clear manually.
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
      <label for="trash-show">Show</label>
      <select
        id="trash-show"
        bind:value={appState.selectedShowKey}
        onchange={/** @param {Event} event */ (event) => {
          const key = event.currentTarget.value;
          const show = appState.shows.find((item) => item.rating_key === key);
          if (show) {
            appState.selectShow(show);
          } else {
            appState.selectedShow = null;
            appState.selectedShowKey = "";
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
      <label for="trash-season">Season</label>
      <select
        id="trash-season"
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
    <p class="kicker" style="margin-top: 6px;">
      Selected: {appState.selectedShow?.title || "None"}
      {#if appState.selectedSeason}
        - {appState.selectedSeason.title}
      {/if}
    </p>
  </div>

  <div class="panel trash-preview lift-2">
    <h2><span class="step-number">02</span> Dry run preview</h2>
    <div class="debug">
      {#if preview.count === 0}
        <EmptyState title="Review before you remove" description="Choose a show and run a Dry Run. Matching trash appears here; the activity bar reports the result." />
      {:else}
        <p><strong>Items ({preview.count})</strong></p>
        {#each preview.titles as title}
          <p>{title}</p>
        {/each}
      {/if}
    </div>
  </div>

  <div class="panel trash-actions lift-3">
    <h2><span class="step-number">03</span> Purge controls</h2>
    <div class="kicker">
      Run a dry run to review matches, then purge trash for the selected show or season.
    </div>
    <div class="actions" style="margin-top: 18px;">
      <button
        data-variant="primary"
        onclick={previewTrash}
        disabled={!appState.selectedShow || appState.isBusy}
      >
        Dry Run
      </button>
      <button
        data-variant="danger"
        onclick={openPurgeConfirm}
        disabled={!appState.selectedShow || appState.isBusy}
      >
        Purge Trash
      </button>
      <button
        data-variant="ghost"
        onclick={() => {
          appState.selectedShow = null;
          appState.selectedShowKey = "";
          appState.selectedSeason = null;
          appState.selectedSeasonKey = "";
          appState.seasons = [];
          preview = { count: 0, titles: [] };
        }}
        disabled={appState.isBusy}
      >
        Clear Selection
      </button>
    </div>
    <p class="kicker" style="margin-top: 12px;">
      Library: {appState.selectedLibraryTitle || "--"}
    </p>
  </div>
</section>

{#if confirmOpen}
  <div class="modal-backdrop">
    <div class="modal">
      <h3>Confirm purge</h3>
      <p>
        This will remove all trashed items for <strong>{confirmTargetLabel}</strong>.
      </p>
      <div class="actions">
        <button data-variant="ghost" onclick={() => (confirmOpen = false)}>
          Cancel
        </button>
        <button data-variant="danger" onclick={confirmPurge} disabled={appState.isBusy}>
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
          onclick={() => (purgeCompleteOpen = false)}
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}
