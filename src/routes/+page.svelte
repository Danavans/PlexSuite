<script>
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount } from "svelte";
  import { appState } from "../lib/appState.svelte.js";
  import TrashTab from "../lib/components/TrashTab.svelte";
  import SubSelectorTab from "../lib/components/SubSelectorTab.svelte";
  import SubsTab from "../lib/components/SubsTab.svelte";
  import PlexmatchTab from "../lib/components/PlexmatchTab.svelte";
  import SettingsTab from "../lib/components/SettingsTab.svelte";
  import ToolIcon from "../lib/components/ToolIcon.svelte";

  let appVersion = $state("");
  onMount(() => { getVersion().then(version => appVersion = version).catch(() => {}); });

  const tools = [
    { id: "trash", title: "Trash Selector", hint: "Review & purge", description: "Review missing media. Clear only the trash in your selected scope.", steps: ["Choose scope", "Review dry run", "Confirm purge"] },
    { id: "subs", title: "Sub Uploader", hint: "Match & upload", description: "Bring your subtitle files into Plex, with every episode matched your way.", steps: ["Choose target", "Add subtitles", "Review & upload"] },
    { id: "subselector", title: "Sub Selector", hint: "Scan & select", description: "Find exact subtitle variants, set your defaults, and manage external tracks.", steps: ["Scan scope", "Choose variant", "Apply or clean up"] },
    { id: "plexmatch", title: "Plexmatch Generator", hint: "Map & export", description: "Connect local video files to TMDb episodes and create your .plexmatch file.", steps: ["Find series", "Arrange & map", "Review & export"] },
    { id: "settings", title: "Settings", hint: "Connections & activity", description: "Manage your connections and inspect activity across PlexSuite.", steps: [] }
  ];
  /** @param {string} id */
  function navigate(id) {
    if (appState.activeTab === "settings" && id !== "settings") {
      appState.connectionMessage = "";
    }
    appState.activeTab = id;
    window.scrollTo({ top: 0, behavior: "instant" });
  }

  const current = $derived(tools.find(tool => tool.id === appState.activeTab) || tools[0]);
</script>

<svelte:head><title>PlexSuite — Plex toolkit</title></svelte:head>

<div class="app" class:app-blocked={appState.isBusy} aria-busy={appState.isBusy} inert={appState.isBusy}>
  <aside class="sidebar">
    <div class="brand">
      <svg class="brand-mark" viewBox="0 0 32 32" fill="none" aria-hidden="true"><path d="M7 26V6h12l7 7-7 7H13" stroke="currentColor" stroke-width="3" stroke-linejoin="round"/><path d="m14 9 5 4-5 4" stroke="currentColor" stroke-width="2"/></svg>
      <div><span class="wordmark">Plex<span>Suite</span></span><span class="brand-caption">PLEX TOOLKIT</span></div>
    </div>
    <p class="nav-caption">TOOLS <span>04</span></p>
    <nav class="tool-nav" aria-label="Tools">
      {#each tools.slice(0, 4) as tool}
        <button class:active-tab={appState.activeTab === tool.id} aria-current={appState.activeTab === tool.id ? "page" : undefined} onclick={() => navigate(tool.id)}>
          <ToolIcon name={tool.id} /><span><strong>{tool.title}</strong><small>{tool.hint}</small></span>
        </button>
      {/each}
    </nav>
    <div class="sidebar-bottom">
      <nav class="tool-nav" aria-label="Application">
        <button class:active-tab={appState.activeTab === "settings"} aria-current={appState.activeTab === "settings" ? "page" : undefined} onclick={() => navigate("settings")}>
          <ToolIcon name="settings" /><span><strong>Settings</strong><small>Connections & activity</small></span>
        </button>
      </nav>
      <div class="connection-card">
        <span class={`status-dot ${appState.isConnected ? "ok" : "down"}`}></span>
        <div><strong>{appState.isConnected ? "Plex connected" : "Plex disconnected"}</strong><small>{appState.isConnected ? "Server available" : "Connect in Settings"}</small></div>
      </div>
      <div class="sidebar-meta"><span>PLEXSUITE</span><span>{appVersion ? `v${appVersion}` : "DESKTOP"}</span></div>
    </div>
  </aside>

  <main class="workspace" id="workspace">
    <header class="workspace-header">
      <div><h1>{current.title}</h1><p class="workspace-description">{current.description}</p></div>
      <span class="workspace-symbol" aria-hidden="true"><ToolIcon name={current.id} /></span>
    </header>
    {#if current.steps.length}
      <ol class="workflow" aria-label="Workflow overview">{#each current.steps as step, index}<li><span>0{index + 1}</span>{step}</li>{/each}</ol>
    {/if}
    {#if current.id !== "settings" && (current.id === "plexmatch" ? !appState.tmdbKey.trim() : !appState.isConnected)}
      <div class="connection-notice"><span><strong>Connect your services.</strong> {current.id === "plexmatch" ? "Add your TMDb API key in Settings to search for series." : "Connect your Plex server in Settings to load your libraries."}</span><button data-variant="ghost" onclick={() => navigate("settings")}>Open Settings <span aria-hidden="true">↗</span></button></div>
    {/if}
    <div class="tool-content">
      {#if appState.activeTab === "trash"}<TrashTab />
      {:else if appState.activeTab === "subselector"}<SubSelectorTab />
      {:else if appState.activeTab === "subs"}<SubsTab />
      {:else if appState.activeTab === "plexmatch"}<PlexmatchTab />
      {:else if appState.activeTab === "settings"}<SettingsTab />{/if}
    </div>
  </main>
</div>

{#if appState.activeTab === "subselector" && appState.isBusy}
  <div class="modal-backdrop blocking"><div class="modal modal-loading" role="status"><div class="loader" aria-hidden="true"></div><p>{appState.status.message}</p></div></div>
{/if}

<footer class={`status-bar ${appState.status.type}`}>
  <span class="activity-label"><span class:activity-busy={appState.isBusy} class="activity-dot"></span>{appState.isBusy ? "WORKING" : "ACTIVITY"}</span>
  <span class="status-message" role="status" aria-live="polite">{appState.status.message || "Ready for your next task."}</span>
  <button class="activity-link" disabled={appState.isBusy} onclick={() => navigate("settings")}>View logs <span aria-hidden="true">↗</span></button>
</footer>
