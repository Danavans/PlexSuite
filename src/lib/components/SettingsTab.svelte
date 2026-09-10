<script>
  import { tick } from "svelte";
  import EmptyState from "./EmptyState.svelte";
  import { appState } from "../appState.svelte.js";
  import { save } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";

  let tmdbDraft = $state(appState.tmdbKey);
  let tmdbFeedback = $state("");
  let tmdbBusy = $state(false);
  let editing = $state(false);
  let deleting = $state(false);
  let editorBusy = $state(false);
  let feedback = $state("");
  let showToken = $state(false);
  let draft = $state({ id: "", name: "", url: "", token: "" });

  /** @param {{id: string, name: string, url: string, token: string} | null} [profile] */
  async function edit(profile = null) {
    draft = profile ? { ...profile } : { id: crypto.randomUUID(), name: "", url: "", token: "" };
    editing = true; deleting = false; feedback = ""; showToken = false;
    await tick(); document.getElementById("server-name")?.focus();
  }
  /** @param {boolean} requireName */
  function profileValues(requireName) {
    if ((requireName && !draft.name.trim()) || !draft.url.trim() || !draft.token.trim()) throw new Error("Name, URL, and token are required.");
    let url;
    try { url = new URL(draft.url.includes("://") ? draft.url.trim() : `http://${draft.url.trim()}`); }
    catch { throw new Error("Enter a valid Plex URL."); }
    if (!["http:", "https:"].includes(url.protocol) || !url.hostname || url.username || url.password || url.search || url.hash) {
      throw new Error("Use an HTTP(S) Plex URL without credentials, query parameters, or fragments.");
    }
    return { ...draft, name: draft.name.trim(), url: url.toString().replace(/\/+$/, ""), token: draft.token.trim() };
  }
  /** @param {"save" | "test" | "delete"} action */
  async function profileAction(action) {
    if (editorBusy || appState.switchBlocked) return;
    editorBusy = true; feedback = "";
    try {
      if (action === "delete") {
        await appState.deleteProfile(draft.id); editing = false;
        appState.setStatus("success", "Plex server profile deleted.");
      } else {
        const profile = profileValues(action === "save");
        if (action === "test") { await appState.testProfile(profile); feedback = "Connection successful."; }
        else { await appState.saveProfile(profile); draft = profile; feedback = "Server saved. Use Test Connection to check availability."; }
      }
    } catch (error) { feedback = String(error); }
    finally { editorBusy = false; deleting = false; }
  }
  /** @param {boolean} test */
  async function tmdbAction(test) {
    if (tmdbBusy) return;
    tmdbBusy = true; tmdbFeedback = "";
    try {
      if (test) { await invoke("test_tmdb_key", { apiKey: tmdbDraft }); tmdbFeedback = "TMDb API key is valid."; }
      else { await appState.saveTmdbKey(tmdbDraft); tmdbFeedback = "TMDb API key saved."; }
    } catch (error) { tmdbFeedback = String(error).split(tmdbDraft || "\0").join("[REDACTED]"); }
    finally { tmdbBusy = false; }
  }

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
  <div class="settings-connections">
    <div class="panel settings-panel">
      <h2 class="section-title">TMDb</h2>
      <p class="kicker">One global API key for Plexmatch Generator. Plex connectivity is not required.</p>
      <div class="field"><label for="tmdb-key">TMDb API Key</label><input id="tmdb-key" type="password" bind:value={tmdbDraft} disabled={tmdbBusy} autocomplete="off" /></div>
      <div class="actions"><button onclick={() => tmdbAction(true)} disabled={tmdbBusy}>Test API Key</button><button data-variant="primary" onclick={() => tmdbAction(false)} disabled={tmdbBusy || appState.settingsWriting}>Save</button></div>
      {#if tmdbBusy}<p role="status">Working...</p>{/if}
      {#if tmdbFeedback}<p class="connection-feedback" role="status">{tmdbFeedback}</p>{/if}
    </div>
    <div class="panel settings-panel lift-1" id="plex-servers">
      <h2 class="section-title">Plex Servers</h2>
      <p class="kicker">Save profiles here. Select the active server from the sidebar.</p>
      <div class="server-list">
        {#each appState.servers as server (server.id)}
          <div class="server-row">
            <div><strong>{server.name}</strong><small>{server.id === appState.activeServerId ? "Active · " : ""}{appState.serverStates[server.id] ?? "Unknown"}</small></div>
            <button data-variant="ghost" onclick={() => edit(server)} disabled={editorBusy || appState.switchBlocked}>Edit<span class="sr-only"> {server.name}</span></button>
          </div>
        {:else}<p class="kicker">No Plex server configured. Add your first server to get started.</p>{/each}
      </div>
      <div class="actions"><button onclick={() => edit()} disabled={editorBusy || appState.switchBlocked}>+ Add Server</button></div>
      {#if editing}
        <form class="server-editor" onsubmit={(event) => { event.preventDefault(); profileAction("save"); }}>
          <h3>{appState.servers.some(s => s.id === draft.id) ? "Edit Server" : "Add Server"}</h3>
          <fieldset disabled={editorBusy || appState.switchBlocked}>
            <div class="field"><label for="server-name">Server name</label><input id="server-name" bind:value={draft.name} required /></div>
            <div class="field"><label for="plex-url">Plex server URL</label><input id="plex-url" bind:value={draft.url} placeholder="http://192.168.1.100:32400" required /></div>
            <div class="field"><label for="plex-token">Plex token</label><input id="plex-token" type={showToken ? "text" : "password"} bind:value={draft.token} autocomplete="off" required /></div>
            <label class="toggle"><input type="checkbox" bind:checked={showToken} />Show token</label>
            <div class="actions">
              <button type="button" onclick={() => profileAction("test")}>Test Connection</button>
              <button type="submit" data-variant="primary">Save</button>
              <button type="button" data-variant="ghost" onclick={() => editing = false}>Close</button>
              {#if appState.servers.some(s => s.id === draft.id)}<button type="button" data-variant="danger" onclick={() => deleting = true}>Delete</button>{/if}
            </div>
            {#if deleting}
              <div class="delete-profile-confirm" role="alert">
                <p>Delete this saved server profile? {draft.id === appState.activeServerId ? "The active server will change to the first remaining profile, or no server." : "The active server will stay unchanged."}</p>
                <div class="actions"><button type="button" data-variant="danger" onclick={() => profileAction("delete")}>Delete profile</button><button type="button" onclick={() => deleting = false}>Cancel</button></div>
              </div>
            {/if}
          </fieldset>
          {#if editorBusy}<p role="status">Working...</p>{/if}
          {#if feedback}<p class="connection-feedback" role="status">{feedback}</p>{/if}
        </form>
      {/if}
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
