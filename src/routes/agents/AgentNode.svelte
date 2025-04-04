<script lang="ts">
  import { onMount } from "svelte";
  import type { Unsubscriber } from "svelte/store";

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

  let { id, data, ...props }: Props = $props();

  const agentDefaultConfig = getAgentDefinitionsContext()?.[data.name]?.default_config;
  const agentDisplayConfig = getAgentDefinitionsContext()?.[data.name]?.display_config;

  onMount(() => {
    if (!agentDisplayConfig) return;

    let unsubscribers: Unsubscriber[] = [];
    agentDisplayConfig.forEach(([key, _]) => {
      unsubscribers.push(
        subscribeDisplayMessage(id, key, (value) => {
          const newDisplay = { ...data.display, [key]: value };
          updateNodeData(id, { display: newDisplay });
        }),
      );
    });

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
  <h4 class="flex-none text-xs text-gray-500 pl-4 pb-4">{data.description}</h4>

  {#if agentDefaultConfig}
    <form class="grow flex flex-col gap-1 pl-4 pr-4 pb-4">
      {#each agentDefaultConfig as [key, default_config]}
        {@const config = data.config[key]}
        {@const ty = default_config?.type}
        <h3 class="flex-none">{default_config?.title || key}</h3>
        {#if default_config?.description}
          <p class="flex-none text-xs text-gray-500">{default_config?.description}</p>
        {/if}
        {#if ty === "boolean"}
          <Toggle
            class="flex-none"
            checked={config}
            onchange={() => updateConfig(key, !data.config[key])}
          />
        {:else if ty === "integer"}
          <NumberInput
            class="flex-none"
            value={config}
            onchange={(evt) => updateConfig(key, evt.currentTarget.value)}
          />
        {:else if ty === "number"}
          <Input
            class="flex-none"
            type="text"
            value={config}
            onchange={(evt) => updateConfig(key, evt.currentTarget.value)}
          />
        {:else if ty === "string"}
          <Input
            class="flex-none"
            type="text"
            value={config}
            onchange={(evt) => updateConfig(key, evt.currentTarget.value)}
          />
        {:else if ty === "text"}
          <Textarea
            class="grow"
            value={config}
            onchange={(evt) => updateConfig(key, evt.currentTarget.value)}
          />
        {:else if ty === "object"}
          <Textarea
            class="grow"
            value={config}
            onchange={(evt) => updateConfig(key, evt.currentTarget.value)}
          />
        {:else}
          <Textarea class="grow" value={JSON.stringify(config, null, 2)} disabled />
        {/if}
      {/each}
    </form>
  {/if}

  {#if agentDisplayConfig}
    <div class="grow flex flex-col gap-1 pl-4 pr-4 pb-4">
      {#each agentDisplayConfig as [key, display_config]}
        <h3 class="flex-none">{display_config?.title || key}</h3>
        <p class="flex-none text-xs text-gray-500">{display_config?.description}</p>
        {@const display = data.display[key]}
        {@const ty = display_config?.type}
        {#if ty === "boolean"}
          {#if display}
            <div class="flex-none">true</div>
          {:else}
            <div class="flex-none">false</div>
          {/if}
        {:else if ty === "integer"}
          <div class="flex-none">{display}</div>
        {:else if ty === "number"}
          <div class="flex-none">{display}</div>
        {:else if ty === "string"}
          <pre class="flex-none">{display}</pre>
        {:else if ty === "text"}
          <pre class="grow">{display.join("\n")}</pre>
        {:else if ty === "object"}
          <pre class="grow">{JSON.stringify(display, null, 2)}</pre>
        {:else}
          <pre class="grow">{JSON.stringify(display, null, 2)}</pre>
        {/if}
      {/each}
    </div>
  {/if}
{/snippet}

<NodeBase {id} {data} {title} {contents} {...props} />
