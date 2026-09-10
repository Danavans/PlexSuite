<script lang="ts">
  let { title, description }: { title: string; description: string } = $props();
</script>

<div class="empty-state">
  <span class="empty-mark" aria-hidden="true">◇</span>
  <strong>{title}</strong>
  <p>{description}</p>
</div>
