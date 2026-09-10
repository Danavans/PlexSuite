<script>
  import EmptyState from "./EmptyState.svelte";
  import { appState } from "../appState.svelte.js";
  import { save } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";

  function exportableLogs() {
    return [...appState.appLogs].reverse().map((entry) =>
      `[${entry.timestamp}] [${entry.tab}] ${entry.type.toUpperCase()}: ${entry.message}`
    ).join("\n");
  }

  /** @param {string} text */
  function sanitizedLogText(text) {
    return text
      .replace(/([?&](?:X-Plex-Token|token|api_key)=)[^&\s]+/gi, "$1[REDACTED]")
      .replace(/(X-Plex-Token[=:]\s*)[^\s,]+/gi, "$1[REDACTED]");
  }

  async function exportLogs() {
    const now = new Date();
    /** @param {number} value */
    const part = (value) => String(value).padStart(2, "0");
    const stamp = `${now.getFullYear()}${part(now.getMonth() + 1)}${part(now.getDate())}-${part(now.getHours())}${part(now.getMinutes())}${part(now.getSeconds())}`;
    const path = await save({ defaultPath: `plexsuite-debug-${stamp}.txt`, filters: [{ name: "Text", extensions: ["txt"] }] });
    if (!path) return;
    try {
      await invoke("save_text_file", { path, content: sanitizedLogText(exportableLogs()) });
      appState.setStatus("success", "Logs exported.");
    } catch (error) {
      appState.setStatus("error", `Log export failed: ${error}`);
    }
  }
</script>

<section class="grid settings-grid">
  <div class="panel settings-panel lift-1">
    <h2 class="section-title">Connections</h2>
    <p class="kicker">
      Save once. The app reconnects automatically on launch.
    </p>
    {#if appState.connectionMessage}<p class="connection-feedback" role="status">{appState.connectionMessage}</p>{/if}
    <div class="field">
      <label for="plex-url">Plex URL</label>
      <input
        id="plex-url"
        placeholder="http://192.168.1.100:32400"
        bind:value={appState.serverUrl}
      />
    </div>
    <div class="field">
      <label for="plex-token">Plex Token</label>
      <input
        id="plex-token"
        type={appState.showToken ? "text" : "password"}
        placeholder="X-Plex-Token"
        bind:value={appState.token}
      />
    </div>
    <label class="toggle">
      <input type="checkbox" bind:checked={appState.showToken} />
      Show token
    </label>
    <div class="field" style="margin-top: 16px;">
      <label for="tmdb-key">TMDb API Key</label>
      <input
        id="tmdb-key"
        placeholder="Your TMDb key"
        bind:value={appState.tmdbKey}
      />
    </div>
    <div class="actions">
      <button
        data-variant="primary"
        onclick={() => appState.saveSettings()}
        disabled={appState.isBusy}
      >
        Save & Connect
      </button>
    </div>
  </div>
  <div class="panel logs-panel lift-2">
    <h2>Session activity</h2><p class="kicker">Logs stay in this session. Export a copy to keep them. Debug mode includes detailed scan diagnostics.</p>
    <div class="logs-controls">
      <div class="actions">
        <button data-variant="ghost" onclick={exportLogs}>Export logs</button>
        <button
          data-variant="ghost"
          onclick={() => {
            appState.appLogs = [];
          }}
        >
          Clear logs
        </button>
      </div>
      <label class="toggle">
        <input type="checkbox" checked={appState.debugMode} onchange={(event) => appState.setDebugMode(event.currentTarget.checked)} />
        Debug mode
      </label>
    </div>
    <div class="debug">
      {#if appState.appLogs.length === 0}
        <EmptyState title="A clear view of your activity" description="Connection messages and tool results appear here as you work." />
      {:else}
        {#each appState.appLogs as entry}
          <p class={`log-entry ${entry.type}`}>
            [{entry.timestamp}] [{entry.tab}] {entry.type.toUpperCase()}: {entry.message}
          </p>
        {/each}
      {/if}
    </div>
  </div>
</section>
