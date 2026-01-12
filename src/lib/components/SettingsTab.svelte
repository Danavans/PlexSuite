<script>
  import { appState } from "../appState.svelte.js";
</script>

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
  <div class="panel lift-2">
    <h2>Logs</h2>
    <div class="actions">
      <button
        data-variant="ghost"
        onclick={() => {
          appState.appLogs = [];
        }}
      >
        Clear logs
      </button>
    </div>
    <div class="debug">
      {#if appState.appLogs.length === 0}
        <p class="kicker">No logs yet.</p>
      {:else}
        {#each appState.appLogs as entry}
          <p>
            [{entry.timestamp}] [{entry.tab}] {entry.type.toUpperCase()}: {entry.message}
          </p>
        {/each}
      {/if}
    </div>
  </div>
</section>