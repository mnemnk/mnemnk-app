<script lang="ts">
  import { Accordion, AccordionItem } from "flowbite-svelte";

  import type { SAgentDefinitions } from "@/lib/types";

  interface Props {
    agentDefs: SAgentDefinitions;
    onAddAgent: (agentName: string) => Promise<void>;
  }

  let { agentDefs, onAddAgent }: Props = $props();

  const categories = Object.keys(agentDefs).reduce(
    (acc, key) => {
      const category = agentDefs[key].category ?? "_unknown_";
      if (!acc[category]) {
        acc[category] = [];
      }
      acc[category].push(key);
      return acc;
    },
    {} as Record<string, string[]>,
  );

  // sort agents in each category
  for (const category in categories) {
    categories[category].sort((a, b) => {
      const aTitle = agentDefs[a].title ?? a;
      const bTitle = agentDefs[b].title ?? b;
      return aTitle.localeCompare(bTitle);
    });
  }

  const categoryNames = Object.keys(categories).sort();
</script>

<Accordion flush class="w-full bg-white dark:bg-black">
  <div>Agents</div>
  <hr />
  {#each categoryNames as categoryName}
    <AccordionItem class="px-2">
      <span slot="header">
        {categoryName}
      </span>
      {#each categories[categoryName] as agentName}
        <button
          type="button"
          class="w-full text-left p-1 hover:bg-gray-200 dark:hover:bg-gray-800 pl-3"
          onclick={() => onAddAgent(agentName)}
        >
          {agentDefs[agentName].title ?? agentName}
        </button>
      {/each}
    </AccordionItem>
  {/each}
</Accordion>
