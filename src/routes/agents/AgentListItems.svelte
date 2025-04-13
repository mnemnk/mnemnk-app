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
    {#each categories[key] as agentName}
      <button
        type="button"
        class="w-full text-left p-1 hover:bg-gray-200 dark:hover:bg-gray-800 pl-3"
        onclick={() => onAddAgent(agentName)}
      >
        {agentDefs[agentName].title ?? agentName}
      </button>
    {/each}
  {:else}
    <AccordionItem borderBottomClass="border-b group-last:border-none" paddingFlush="pl-2 py-1">
      <div slot="header">
        {key}
      </div>
      <Accordion flush class="w-full pr-2 bg-white dark:bg-black">
        <AgentListItems categories={categories[key]} {agentDefs} {onAddAgent} />
      </Accordion>
    </AccordionItem>
  {/if}
{/each}
