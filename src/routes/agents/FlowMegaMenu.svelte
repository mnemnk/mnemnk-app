<script lang="ts">
  import { MegaMenu } from "flowbite-svelte";

  interface Props {
    flows: Record<string, any>;
    onChangeFlow?: (flowName: string) => void;
    open?: boolean;
  }

  let { flows, onChangeFlow, open = $bindable(false) }: Props = $props();

  let flowMenuItems = $derived(
    Object.keys(flows).map((key) => {
      return { name: key };
    }),
  );
</script>

<MegaMenu
  bind:open
  items={flowMenuItems}
  let:item
  class="!bg-gray-100 dark:!bg-gray-900 border-none"
>
  <button type="button" onclick={() => onChangeFlow?.(item.name)} class="m-1">
    {item.name}
  </button>
</MegaMenu>
