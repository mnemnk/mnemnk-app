<script lang="ts">
  import { Accordion, AccordionItem } from "flowbite-svelte";

  import FlowListItems from "./FlowListItems.svelte";

  interface Props {
    directories: Record<string, any>;
    currentFlowName: string;
    flowActivities: Record<string, any>;
    changeFlowName: (flowName: string) => void;
  }

  let { directories, currentFlowName, flowActivities, changeFlowName }: Props = $props();

  const dirKeys = Object.keys(directories).sort();

  // Helper function to get display name (last part after slash)
  function getDisplayName(fullPath: string): string {
    const parts = fullPath.split("/");
    return parts[parts.length - 1];
  }
</script>

{#each dirKeys as key}
  {#if key === "."}
    {@const flowNames = directories[key].sort()}
    {#each flowNames as flowName}
      <button
        type="button"
        class="w-full text-left p-1 pl-3 text-gray-400 hover:text-black hover:bg-gray-200 dark:hover:bg-gray-400 flex items-center"
        onclick={() => changeFlowName(flowName)}
      >
        {#if flowName === currentFlowName}
          <span class="text-semibold text-gray-900 dark:text-white">{getDisplayName(flowName)}</span
          >
        {:else}
          <span>{getDisplayName(flowName)}</span>
        {/if}

        {#if flowActivities[flowName]}
          <span
            class="flex-none inline-block w-2 h-2 ml-1 bg-green-500 rounded-full mr-2"
            title="active"
          ></span>
        {/if}
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
        <FlowListItems
          directories={directories[key]}
          {currentFlowName}
          {flowActivities}
          {changeFlowName}
        />
      </Accordion>
    </AccordionItem>
  {/if}
{/each}
