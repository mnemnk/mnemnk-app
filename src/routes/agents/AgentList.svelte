<script lang="ts">
  import { Drawer } from "flowbite-svelte";

  import type { SAgentDefinitions } from "@/lib/types";

  interface Props {
    agentDefs: SAgentDefinitions;
    onAddAgent: (agentName: string) => Promise<void>;
    hidden: boolean;
  }

  let { agentDefs, onAddAgent, hidden = $bindable(true) }: Props = $props();

  const agentNames = Object.keys(agentDefs).sort((a, b) => {
    const aTitle = agentDefs[a].title ?? a;
    const bTitle = agentDefs[b].title ?? b;
    return aTitle.localeCompare(bTitle);
  });
</script>

<Drawer bind:hidden backdrop={false} class="w-64" placement="right">
  <h3 class="pb-1 font-bold">Agents</h3>
  <hr />
  {#each agentNames as agentName}
    <button
      type="button"
      class="w-full text-left p-2 hover:bg-gray-200 dark:hover:bg-gray-800"
      onclick={() => onAddAgent(agentName)}
    >
      {agentDefs[agentName].title ?? agentName}
    </button>
  {/each}
</Drawer>
