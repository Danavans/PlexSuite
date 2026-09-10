<script lang="ts">
  import { appState } from "../appState.svelte.js";
  let { manage }: { manage: () => void } = $props();
  let trigger: HTMLButtonElement;
  let menu: HTMLDivElement;
  let expanded = $state(false);
  let dismissTriggerClick = false;
  function address(url: string) { try { return new URL(url.includes("://") ? url : `http://${url}`).origin; } catch { return "Edit URL in Settings"; } }
  function position() {
    const rect = trigger.getBoundingClientRect();
    const width = Math.min(300, window.innerWidth - 24);
    const height = Math.min(360, window.innerHeight - 24);
    menu.style.width = `${width}px`;
    menu.style.maxHeight = `${height}px`;
    menu.style.left = `${Math.max(12, Math.min(rect.left, window.innerWidth - width - 12))}px`;
    const actualHeight = Math.min(menu.scrollHeight, height);
    const preferredTop = rect.top >= actualHeight + 20 ? rect.top - actualHeight - 8 : rect.bottom + 8;
    menu.style.top = `${Math.max(12, Math.min(preferredTop, window.innerHeight - actualHeight - 12))}px`;
  }
  function open() { if (appState.switchBlocked) return; menu.showPopover(); position(); }
  function close() { menu?.hidePopover(); }
  function dismissFromTriggerPointer() {
    if (!menu.matches(":popover-open")) return;
    close();
    dismissTriggerClick = true;
  }
  function toggle() {
    if (dismissTriggerClick) { dismissTriggerClick = false; return; }
    menu.matches(":popover-open") ? close() : open();
  }
  $effect(() => { if (appState.switchBlocked) close(); });
</script>
<svelte:window onresize={close} onscroll={close} />
<button bind:this={trigger} class="connection-card server-trigger" onpointerdown={dismissFromTriggerPointer} onclick={toggle} disabled={appState.switchBlocked} aria-expanded={expanded} aria-controls="server-menu" title={appState.switchBlocked ? "Wait for the current operation" : "Select Plex server"}>
  <span class={`status-dot ${appState.isConnected ? "ok" : "down"}`}></span>
  <span class="server-current"><strong>{appState.activeServer?.name ?? "No Plex server"}</strong><small>{appState.switching ? "Connecting..." : appState.connectionState === "Available" ? "Server available" : appState.connectionState === "Unavailable" ? "Server unavailable" : appState.connectionState}</small></span>
  <span aria-hidden="true">▾</span>
</button>
<div bind:this={menu} id="server-menu" class="server-popover" popover="auto" ontoggle={(event) => expanded = event.newState === "open"}>
  <p class="nav-caption">PLEX SERVERS</p>
  {#each appState.servers as server (server.id)}
    <button class="server-option" class:selected-server={server.id === appState.activeServerId} aria-pressed={server.id === appState.activeServerId} disabled={appState.switchBlocked} onclick={() => { close(); appState.switchServer(server.id); }}>
      <span><strong>{server.name}</strong><small>{appState.serverStates[server.id] ?? "Unknown"}</small><small>{address(server.url)}</small></span><span aria-hidden="true">{server.id === appState.activeServerId ? "✓" : ""}</span>
    </button>
  {/each}
  <button class="server-option manage-servers" onclick={() => { close(); manage(); }}>Manage servers…</button>
</div>
