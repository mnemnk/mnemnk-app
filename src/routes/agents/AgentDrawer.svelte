<script lang="ts">
  import type { Writable } from "svelte/store";

  import type { NodeProps } from "@xyflow/svelte";
  import { Button, Drawer } from "flowbite-svelte";

  import type { SAgentDefinitions, AgentFlowNode } from "@/lib/types";

  type Props = NodeProps & {
    nodes: Writable<AgentFlowNode[]>;
    agent_defs: SAgentDefinitions;
    onAddAgent: (agent_name: string) => Promise<void>;
  };

  const { agent_defs, onAddAgent }: Props = $props();

  const agent_names = Object.keys(agent_defs).sort();
</script>

<Drawer
  activateClickOutside={false}
  backdrop={false}
  hidden={false}
  placement="right"
  class="w-200"
>
  {#each agent_names as agent_name}
    <div class="mb-4">
      <Button
        class="w-full"
        color={agent_name.startsWith("$") ? "blue" : "primary"}
        outline
        onclick={() => onAddAgent(agent_name)}>{agent_defs[agent_name].title ?? agent_name}</Button
      >
    </div>
  {/each}
</Drawer>
