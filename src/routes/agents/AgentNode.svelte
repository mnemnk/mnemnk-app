<script lang="ts">
  import { onMount } from "svelte";

  import { useSvelteFlow, type NodeProps } from "@xyflow/svelte";
  import { Input, Label, NumberInput, Textarea, Toggle } from "flowbite-svelte";

  import {
    getAgentDefinitionsContext,
    serializeAgentFlowNodeConfig,
    setAgentConfig,
  } from "@/lib/agent";
  import { subscribeDisplayMessage } from "@/lib/shared.svelte";
  import type { AgentFlowNodeConfig, AgentFlowNodeDisplay } from "@/lib/types";

  import NodeBase from "./NodeBase.svelte";

  type Props = NodeProps & {
    data: {
      name: string;
      enabled: boolean;
      title?: string | null;
      description?: string | null;
      inputs: string[];
      outputs: string[];
      config: AgentFlowNodeConfig;
      display: AgentFlowNodeDisplay;
    };
  };

  let { id, data }: Props = $props();

  const agentDefaultConfig = getAgentDefinitionsContext()?.[data.name]?.default_config;
  const agentDisplayConfig = getAgentDefinitionsContext()?.[data.name]?.display_config;

  onMount(() => {
    if (!agentDisplayConfig) return;

    let unsubscribers = [];
    for (const key of Object.keys(agentDisplayConfig)) {
      unsubscribers.push(
        subscribeDisplayMessage(id, key, (value) => {
          const newDisplay = { ...data.display, [key]: value };
          updateNodeData(id, { display: newDisplay });
        }),
      );
    }

    return () => {
      for (const unsub of unsubscribers) {
        unsub();
      }
    };
  });

  const { updateNodeData } = useSvelteFlow();

  async function updateConfig(key: string, value: any) {
    const newConfig = { ...data.config, [key]: value };
    updateNodeData(id, { config: newConfig });
    const sConfig = serializeAgentFlowNodeConfig(newConfig, agentDefaultConfig);
    if (sConfig) {
      await setAgentConfig(id, sConfig);
    }
  }
</script>

{#snippet title()}
  <h3 class="text-xl pt-2">{data.title ?? data.name}</h3>
{/snippet}

{#snippet contents()}
  {#if data.description}
    <h4 class="text-sm pl-4 pb-4">{data.description}</h4>
  {/if}

  {#if agentDefaultConfig}
    <form class="grid grid-cols-6 gap-4 p-4">
      {#each agentDefaultConfig as [key, default_config]}
        {@const config = data.config[key]}
        <Label class="col-span-6 space-y-2">
          <h3>{default_config?.title || key}</h3>
          <p class="text-xs text-gray-500">{default_config?.description}</p>
          {#if default_config?.type === "boolean"}
            <Toggle checked={config} onchange={() => updateConfig(key, !data.config[key])} />
          {:else if default_config?.type === "integer"}
            <NumberInput
              value={config}
              onchange={(evt) => updateConfig(key, evt.currentTarget.value)}
            />
          {:else if default_config?.type === "number"}
            <Input
              type="text"
              value={config}
              onchange={(evt) => updateConfig(key, evt.currentTarget.value)}
            />
          {:else if default_config?.type === "string" || default_config?.type === "string?"}
            <Input
              type="text"
              value={config}
              onchange={(evt) => updateConfig(key, evt.currentTarget.value)}
            />
          {:else if default_config?.type === "string[]"}
            <Textarea
              value={config}
              onchange={(evt) => updateConfig(key, evt.currentTarget.value)}
              rows={4}
            />
          {:else if default_config?.type === "object"}
            <Textarea
              value={config}
              onchange={(evt) => updateConfig(key, evt.currentTarget.value)}
              rows={4}
            />
          {:else}
            <Input type="text" value={JSON.stringify(config, null, 2)} disabled />
          {/if}
        </Label>
      {/each}
    </form>
  {/if}

  {#if agentDisplayConfig}
    <div class="grid grid-cols-6 gap-4 p-4">
      {#each agentDisplayConfig as [key, display_config]}
        {@const display = data.display[key]}
        <Label class="col-span-6 space-y-2">
          <h3>{display_config?.title || key}</h3>
          <p class="text-xs text-gray-500">{display_config?.description}</p>
          {#if display_config?.type === "boolean"}
            {#if display}
              <div>true</div>
            {:else}
              <div>false</div>
            {/if}
          {:else if display_config?.type === "integer"}
            <div>{display}</div>
          {:else if display_config?.type === "number"}
            <div>{display}</div>
          {:else if display_config?.type === "string" || display_config?.type === "string?"}
            <pre>{display}</pre>
          {:else if display_config?.type === "string[]"}
            <pre>{display.join("\n")}</pre>
          {:else if display_config?.type === "object"}
            <pre>{JSON.stringify(display, null, 2)}</pre>
          {:else}
            <pre>{JSON.stringify(display, null, 2)}</pre>
          {/if}
        </Label>
      {/each}
    </div>
  {/if}
{/snippet}

<NodeBase
  {id}
  enabled={data.enabled}
  inputs={data.inputs}
  outputs={data.outputs}
  {title}
  {contents}
/>
