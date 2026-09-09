<script>
  import { appState } from "../lib/appState.svelte.js";
  import TrashTab from "../lib/components/TrashTab.svelte";
  import SubSelectorTab from "../lib/components/SubSelectorTab.svelte";
  import SubsTab from "../lib/components/SubsTab.svelte";
  import PlexmatchTab from "../lib/components/PlexmatchTab.svelte";
  import SettingsTab from "../lib/components/SettingsTab.svelte";

  const tabDescription = $derived(
    appState.activeTab === "trash"
      ? "Purge Plex trash selectively for a show or season with a safe dry run."
      : appState.activeTab === "subselector" ? "Scan subtitle variants, set defaults, and remove external subtitles." : appState.activeTab === "subs"
        ? "Bulk upload subtitles by auto-matching episodes from filenames."
        : appState.activeTab === "plexmatch"
          ? "Map TMDb episodes to local files and export a PlexMatch file."
          : "Manage Plex and TMDb credentials used by the app."
  );
</script>

<main
  class="app"
  class:app-blocked={appState.isBusy}
  aria-busy={appState.isBusy}
  inert={appState.isBusy}
>
  <header class="hero">
    <div class="hero-top">
      <div class="hero-brand">
        <p class="badge">PlexSuite</p>
        <h1 class="sr-only">PlexSuite</h1>
      </div>
      <div class="status-row">
        <div class="status-pill">
          <span class={`status-dot ${appState.isConnected ? "ok" : "down"}`}></span>
          <span class="status-text">
            {appState.isConnected ? "Connected" : "Disconnected"}
          </span>
        </div>
      </div>
    </div>
    <div class="hero-bottom">
      <div class="tabs-wrap">
        <div class="tabs">
        <button
          class:active-tab={appState.activeTab === "trash"}
          onclick={() => (appState.activeTab = "trash")}
        >
          Trash Selector
        </button>
        <button
          class:active-tab={appState.activeTab === "subs"}
          onclick={() => (appState.activeTab = "subs")}
        >
          Sub Uploader
        </button>
        <button class:active-tab={appState.activeTab === "subselector"} onclick={() => (appState.activeTab = "subselector")}>
          Sub Selector
        </button>
        <button
          class:active-tab={appState.activeTab === "plexmatch"}
          onclick={() => (appState.activeTab = "plexmatch")}
        >
          Plexmatch Generator
        </button>
        <button
          class:active-tab={appState.activeTab === "settings"}
          onclick={() => (appState.activeTab = "settings")}
        >
          Settings
        </button>
        </div>
      </div>
      <p class="hero-desc">{tabDescription}</p>
    </div>
  </header>

  {#if appState.activeTab === "trash"}
    <TrashTab />
  {:else if appState.activeTab === "subselector"}
    <SubSelectorTab />
  {:else if appState.activeTab === "subs"}
    <SubsTab />
  {:else if appState.activeTab === "plexmatch"}
    <PlexmatchTab />
  {:else if appState.activeTab === "settings"}
    <SettingsTab />
  {/if}
</main>

{#if appState.activeTab === "subselector" && appState.isBusy}
  <div class="modal-backdrop blocking">
    <div class="modal modal-loading" role="status">
      <div class="loader" aria-hidden="true"></div>
      <p>{appState.status.message}</p>
    </div>
  </div>
{/if}

<div class={`status-bar ${appState.status.type}`}>
  {appState.status.message || "Ready."}
</div>
