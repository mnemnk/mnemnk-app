<script lang="ts">
  import { get } from "svelte/store";
  import type { Writable } from "svelte/store";

  import type { NodeProps } from "@xyflow/svelte";
  import { Input, Label, NumberInput, Textarea, Toggle } from "flowbite-svelte";

  import { getAgentDefinitionsContext } from "@/lib/agent";
  import type { AgentFlowNodeConfig } from "@/lib/types";

  import NodeBase from "./NodeBase.svelte";

  type Props = NodeProps & {
    data: {
      name: string;
      enabled: Writable<boolean>;
      inputs: string[];
      outputs: string[];
      config: AgentFlowNodeConfig;
    };
  };

  let { id, data }: Props = $props();

  const agent_default_config = getAgentDefinitionsContext()?.[data.name]?.default_config;
</script>

{#snippet title()}
  <h3 class="text-xl pt-2">{data.title ?? data.name}</h3>
{/snippet}

{#snippet contents()}
  {#if data.description}
    <h4 class="text-sm pl-4 pb-4">{data.description}</h4>
  {/if}
  <form class="grid grid-cols-6 gap-4 p-4">
    <Toggle bind:checked={() => get(data.enabled), (v) => data.enabled.set(v)} class="col-span-6"
    ></Toggle>
    {#each Object.keys(data.config) as key}
      {@const config = data.config[key]}
      {@const default_config = agent_default_config?.[key]}
      <Label class="col-span-6 space-y-2">
        <h3>{default_config?.title || key}</h3>
        <p class="text-xs text-gray-500">{default_config?.description}</p>
        {#if default_config?.type === "boolean"}
          <Toggle bind:checked={() => get(config), (v) => config.set(v)} />
        {:else if default_config?.type === "integer"}
          <NumberInput bind:value={() => get(config), (v) => config.set(v)} />
        {:else if default_config?.type === "number"}
          <Input type="text" bind:value={() => get(config), (v) => config.set(v)} />
        {:else if default_config?.type === "string" || default_config?.type === "string?"}
          <Input type="text" bind:value={() => get(config), (v) => config.set(v)} />
        {:else if default_config?.type === "string[]"}
          <Textarea bind:value={() => get(config), (v) => config.set(v)} rows={4} />
        {:else}
          <Input type="text" value={JSON.stringify(get(config))} disabled />
        {/if}
      </Label>
    {/each}
  </form>
{/snippet}

<NodeBase {id} inputs={data.inputs} outputs={data.outputs} {title} {contents} />
