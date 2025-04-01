<script lang="ts">
  import { MegaMenu } from "flowbite-svelte";

  import type { SAgentDefinitions } from "@/lib/types";

  interface Props {
    agentDefs: SAgentDefinitions;
    onAddAgent: (agentName: string) => Promise<void>;
    open?: boolean;
  }

  let { agentDefs, onAddAgent, open = $bindable(false) }: Props = $props();

  const agentMenuItems = Object.keys(agentDefs)
    .sort()
    .map((key) => {
      return { name: key };
    });
</script>

<MegaMenu
  bind:open
  items={agentMenuItems}
  let:item
  class="!bg-gray-100 dark:!bg-gray-900 border-none"
>
  <button type="button" onclick={() => onAddAgent(item.name)}>
    {agentDefs[item.name].title ?? item.name}
  </button>
</MegaMenu>
