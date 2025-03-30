<script lang="ts">
  import { SvelteFlowProvider } from "@xyflow/svelte";

  const { children } = $props();
</script>

<SvelteFlowProvider>
  {@render children?.()}
</SvelteFlowProvider>
