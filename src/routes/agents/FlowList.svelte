<script lang="ts">
  import { Button } from "flowbite-svelte";

  let { flows, flowIndex = $bindable() } = $props();
</script>

<div class="w-200">
  {#each flows as flow, i}
    <Button class="w-full" onclick={() => (flowIndex = i)}>
      <span>{flow.name ?? "Untitled"}{i === flowIndex ? " *" : ""}</span>
    </Button>
  {/each}
</div>
