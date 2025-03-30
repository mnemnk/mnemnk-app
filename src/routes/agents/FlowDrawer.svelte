<script lang="ts">
  import { Button, Drawer } from "flowbite-svelte";

  let { flows, flowIndex = $bindable() } = $props();
</script>

<Drawer
  activateClickOutside={false}
  backdrop={false}
  hidden={false}
  placement="right"
  class="w-200"
>
  {#each flows as flow, i}
    <div class="">
      <Button
        class="w-full"
        color={i === flowIndex ? "red" : "primary"}
        onclick={() => (flowIndex = i)}>{flow.name ?? "Untitled"}</Button
      >
    </div>
  {/each}
</Drawer>
