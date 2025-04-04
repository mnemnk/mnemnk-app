<script lang="ts">
  import { MegaMenu } from "flowbite-svelte";

  interface Props {
    flowNames: string[];
    onChangeFlow?: (flowName: string) => void;
    open?: boolean;
  }

  let { flowNames, onChangeFlow, open = $bindable(false) }: Props = $props();

  let flowMenuItems = $derived(
    flowNames.map((key) => {
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
