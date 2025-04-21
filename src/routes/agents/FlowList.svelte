<script lang="ts">
  import { Accordion, AccordionItem } from "flowbite-svelte";

  import FlowListItems from "./FlowListItems.svelte";

  interface Props {
    flowNames: string[];
    currentFlowName: string;
    flowActivities: Record<string, any>;
    changeFlowName: (flowName: string) => void;
  }

  let { flowNames, currentFlowName, flowActivities, changeFlowName }: Props = $props();

  const directories = $derived.by(() => {
    const result: Record<string, any> = {
      ".": [], // Special directory for top-level flows (no slashes)
    };

    // Process each flow name
    for (const flowName of flowNames) {
      if (!flowName.includes("/")) {
        // Top-level flow, no directory
        result["."].push(flowName);
        continue;
      }

      const parts = flowName.split("/");
      const dir = parts[0];

      if (parts.length === 2) {
        // Direct child of the dir
        if (!result[dir]) {
          result[dir] = {
            ".": [],
          };
        }
        result[dir]["."].push(flowName);
      } else {
        // Nested flow with multiple levels
        if (!result[dir]) {
          result[dir] = {};
        }

        // Create or navigate to sub-directory
        let current = result[dir];
        for (let i = 1; i < parts.length - 1; i++) {
          const part = parts[i];
          if (!current[part]) {
            current[part] = {
              ".": [],
            };
          }
          current = current[part];
        }

        // Add flow to the deepest sub-directory
        if (!current["."]) {
          current["."] = [];
        }
        current["."].push(flowName);
      }
    }

    return result;
  });
</script>

<div class="backdrop-blur-xs">
  <h4>Flows</h4>
  <hr />
  <Accordion flush>
    {#each directories["."] as flowName}
      <button
        type="button"
        class="w-full text-left p-1 pl-3 text-gray-400 hover:text-black hover:bg-gray-200 dark:hover:bg-gray-400 flex items-center"
        onclick={() => changeFlowName(flowName)}
      >
        {#if flowName === currentFlowName}
          <span class="text-semibold text-gray-900 dark:text-white">{flowName}</span>
        {:else}
          <span>{flowName}</span>
        {/if}
        {#if flowActivities[flowName]}
          <span
            class="flex-none inline-block w-2 h-2 ml-1 bg-green-500 rounded-full mr-2"
            title="active"
          ></span>
        {/if}
      </button>
    {/each}

    {#each Object.keys(directories)
      .filter((key) => key !== ".")
      .sort() as dir}
      <AccordionItem
        borderBottomClass="border-b group-last:border-none"
        paddingFlush="pl-2 pr-2 py-1"
      >
        <div slot="header">
          {dir}
        </div>
        <Accordion flush>
          <FlowListItems
            directories={directories[dir]}
            {currentFlowName}
            {flowActivities}
            {changeFlowName}
          />
        </Accordion>
      </AccordionItem>
    {/each}
  </Accordion>
</div>
