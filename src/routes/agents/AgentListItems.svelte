<script lang="ts">
  import { Accordion, AccordionItem } from "flowbite-svelte";

  import type { SAgentDefinitions } from "@/lib/types";

  import AgentListItems from "./AgentListItems.svelte";

  interface Props {
    categories: Record<string, any>;
    agentDefs: SAgentDefinitions;
    onAddAgent: (agentName: string) => Promise<void>;
  }

  let { categories, agentDefs, onAddAgent }: Props = $props();

  const categoryKeys = Object.keys(categories).sort();
</script>

{#each categoryKeys as key}
  {#if key === "00agents"}
    {@const agentNames = categories[key].sort()}
    {#each agentNames as agentName}
      <button
        type="button"
        class="w-full pl-2 text-left text-gray-400 hover:text-black hover:bg-gray-200 dark:hover:bg-gray-400"
        onclick={() => onAddAgent(agentName)}
      >
        {agentDefs[agentName].title ?? agentName}
      </button>
    {/each}
  {:else}
    <AccordionItem
      borderBottomClass="border-b group-last:border-none"
      paddingFlush="pl-2 pr-2 py-1"
    >
      <div slot="header">
        {key}
      </div>
      <Accordion flush>
        <AgentListItems categories={categories[key]} {agentDefs} {onAddAgent} />
      </Accordion>
    </AccordionItem>
  {/if}
{/each}
