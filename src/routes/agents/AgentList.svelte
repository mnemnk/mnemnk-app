<script lang="ts">
  import { Accordion } from "flowbite-svelte";

  import type { SAgentDefinitions } from "@/lib/types";

  import AgentListItems from "./AgentListItems.svelte";

  interface Props {
    agentDefs: SAgentDefinitions;
    onAddAgent: (agentName: string) => Promise<void>;
  }

  let { agentDefs, onAddAgent }: Props = $props();

  const categories = Object.keys(agentDefs).reduce(
    (acc, key) => {
      const categoryPath = (agentDefs[key].category ?? "_unknown_").split("/");
      let currentLevel = acc;

      for (const part of categoryPath) {
        if (!currentLevel[part]) {
          currentLevel[part] = {};
        }
        currentLevel = currentLevel[part];
      }

      if (!currentLevel["00agents"]) {
        currentLevel["00agents"] = [];
      }
      currentLevel["00agents"].push(key);

      return acc;
    },
    {} as Record<string, any>,
  );
</script>

<h4>Agents</h4>
<hr />
<Accordion flush class="w-full pr-2 bg-white dark:bg-black">
  <AgentListItems {categories} {agentDefs} {onAddAgent} />
</Accordion>
