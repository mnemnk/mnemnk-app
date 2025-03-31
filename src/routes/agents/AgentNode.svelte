<script lang="ts">
  import { useSvelteFlow, type NodeProps } from "@xyflow/svelte";
  import { Input, Label, NumberInput, Textarea, Toggle } from "flowbite-svelte";

  import { getAgentDefinitionsContext } from "@/lib/agent";
  import type { AgentFlowNodeConfig } from "@/lib/types";

  import NodeBase from "./NodeBase.svelte";

  type Props = NodeProps & {
    data: {
      name: string;
      enabled: boolean;
      inputs: string[];
      outputs: string[];
      config: AgentFlowNodeConfig;
    };
  };

  let { id, data }: Props = $props();

  const agent_default_config = getAgentDefinitionsContext()?.[data.name]?.default_config;

  const { updateNodeData } = useSvelteFlow();
</script>

{#snippet title()}
  <h3 class="text-xl pt-2">{data.title ?? data.name}</h3>
{/snippet}

{#snippet contents()}
  {#if data.description}
    <h4 class="text-sm pl-4 pb-4">{data.description}</h4>
  {/if}
  <form class="grid grid-cols-6 gap-4 p-4">
    <Toggle
      checked={data.enabled}
      onchange={(evt) => updateNodeData(id, { enabled: evt.currentTarget.value })}
      class="col-span-6"
    ></Toggle>
    {#each Object.keys(data.config) as key}
      {@const default_config = agent_default_config?.[key]}
      {@const config = data.config[key]}
      <Label class="col-span-6 space-y-2">
        <h3>{default_config?.title || key}</h3>
        <p class="text-xs text-gray-500">{default_config?.description}</p>
        {#if default_config?.type === "boolean"}
          <Toggle
            checked={config}
            onchange={(evt) =>
              updateNodeData(id, { config: { ...data.config, [key]: evt.currentTarget.value } })}
          />
        {:else if default_config?.type === "integer"}
          <NumberInput
            value={config}
            onchange={(evt) =>
              updateNodeData(id, { config: { ...data.config, [key]: evt.currentTarget.value } })}
          />
        {:else if default_config?.type === "number"}
          <Input
            type="text"
            value={config}
            onchange={(evt) =>
              updateNodeData(id, { config: { ...data.config, [key]: evt.currentTarget.value } })}
          />
        {:else if default_config?.type === "string" || default_config?.type === "string?"}
          <Input
            type="text"
            value={config}
            onchange={(evt) =>
              updateNodeData(id, { config: { ...data.config, [key]: evt.currentTarget.value } })}
          />
        {:else if default_config?.type === "string[]"}
          <Textarea
            value={config}
            onchange={(evt) =>
              updateNodeData(id, { config: { ...data.config, [key]: evt.currentTarget.value } })}
            rows={4}
          />
        {:else if default_config?.type === "object"}
          <Textarea
            value={config}
            onchange={(evt) =>
              updateNodeData(id, { config: { ...data.config, [key]: evt.currentTarget.value } })}
            rows={4}
          />
        {:else}
          <Input type="text" value={JSON.stringify(config, null, 2)} disabled />
        {/if}
      </Label>
    {/each}
  </form>
{/snippet}

<NodeBase {id} inputs={data.inputs} outputs={data.outputs} {title} {contents} />
