<script lang="ts">
  let { name }: { name: string } = $props();
</script>

<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  {#if name === "trash"}
    <path d="M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13M10 11v5m4-5v5" />
  {:else if name === "subs"}
    <rect x="3" y="5" width="18" height="14" rx="2" /><path d="M6 15h5m3 0h4M12 12V8m-3 3 3-3 3 3" />
  {:else if name === "subselector"}
    <path d="M4 6h16M4 12h7M4 18h7m4-5 2 2 4-5m-6 8 2 2 4-5" />
  {:else if name === "plexmatch"}
    <rect x="3" y="3" width="6" height="6" rx="1" /><rect x="15" y="15" width="6" height="6" rx="1" /><path d="M9 6h6a3 3 0 0 1 3 3v6M6 9v9h5m-3-3 3 3-3 3" />
  {:else}
    <path d="M4 6h16M4 12h16M4 18h16" /><path d="M8 3v6m8 0v6m-6 0v6" stroke-width="3" />
  {/if}
</svg>
