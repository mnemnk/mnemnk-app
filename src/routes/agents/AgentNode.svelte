<script lang="ts" module>
  const titleColorMap: Record<string, string> = {
    Board: "bg-green-500",
    Builtin: "bg-blue-500",
    Command: "bg-amber-500",
    Database: "bg-teal-500",
    default: "bg-purple-500",
  };
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import type { Unsubscriber } from "svelte/store";

  import { useSvelteFlow, type NodeProps } from "@xyflow/svelte";
  import { Button, Input, NumberInput, Popover, Textarea, Toggle } from "flowbite-svelte";
  import { ExclamationCircleOutline } from "flowbite-svelte-icons";

  import Messages from "@/components/Messages.svelte";
  import {
    getAgentDefinitionsContext,
    serializeAgentFlowNodeConfig,
    setAgentConfig,
  } from "@/lib/agent";
  import {
    subscribeDisplayMessage,
    subscribeErrorMessage,
    subscribeInputMessage,
  } from "@/lib/shared.svelte";
  import type {
    AgentFlowNodeConfig,
    AgentFlowNodeDisplay,
    SAgentConfigEntry,
    SAgentDisplayConfigEntry,
  } from "@/lib/types";

  import NodeBase from "./NodeBase.svelte";

  type Props = NodeProps & {
    data: {
      name: string;
      title: string | null;
      enabled: boolean;
      config: AgentFlowNodeConfig;
      display: AgentFlowNodeDisplay;
    };
  };

  let { id, data, ...props }: Props = $props();

  const agentDef = getAgentDefinitionsContext()?.[data.name];
  const agentDefaultConfig = agentDef?.default_config;
  const agentDisplayConfig = agentDef?.display_config;
  const description = agentDef?.description;

  let errorMessages = $state<string[]>([]);
  let inputMessage = $state<string>("");
  let inputCount = $state(0);

  onMount(() => {
    let unsubscribers: Unsubscriber[] = [];

    if (agentDisplayConfig) {
      // Subscribe to display messages for each display config key
      agentDisplayConfig.forEach(([key, _]) => {
        unsubscribers.push(
          subscribeDisplayMessage(id, key, (value) => {
            const newDisplay = { ...data.display, [key]: value };
            updateNodeData(id, { display: newDisplay });
          }),
        );
      });
    }

    // Add subscription for error messages
    unsubscribers.push(
      subscribeErrorMessage(id, (message) => {
        if (!message) return;
        errorMessages.push(message);
      }),
    );

    unsubscribers.push(
      subscribeInputMessage(id, ({ ch, t }) => {
        if (!ch || ch === "") return;
        inputMessage = ch;
        inputCount += 1;
      }),
    );

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

  function clearError() {
    errorMessages = [];
  }

  let editTitle = $state(false);
  let titleColor = $derived(titleColorMap[agentDef?.kind ?? "default"] ?? titleColorMap.default);

  const uid = $props.id();
</script>

{#snippet title()}
  <div class="flex-none mt-1">
    <div class="flex flex-nowrap">
      {#if agentDef}
        {#if editTitle}
          <Input
            class="mt-1"
            type="text"
            value={data.title ?? agentDef?.title ?? data.name}
            onblur={() => (editTitle = false)}
            onkeydown={(evt) => {
              if (evt.key === "Enter") {
                const newTitle = evt.currentTarget.value;
                if (newTitle === "" || newTitle === (agentDef?.title ?? data.name)) {
                  updateNodeData(id, { title: null });
                } else if (newTitle !== data.title) {
                  updateNodeData(id, { title: newTitle });
                }
                editTitle = false;
              }
            }}
          />
        {:else}
          <button
            id="t-{uid}"
            type="button"
            onclick={() => (editTitle = true)}
            class="flex-none"
            tabindex={-1}
          >
            <h3 class="text-xl">
              {data.title ?? agentDef?.title ?? data.name}
            </h3>
          </button>
        {/if}
      {:else}
        <h3 class="text-xl">
          <s>{data.name}</s>
        </h3>
      {/if}
      {#if errorMessages.length > 0}
        <ExclamationCircleOutline id="e-{uid}" class="ml-2 pt-1 w-6 h-6 text-red-500" />
      {/if}
    </div>
  </div>
{/snippet}

{#snippet displayItem(ty: string | null, value: any)}
  {#if ty === "boolean"}
    {#if value}
      <div class="flex-none border-1 p-2">true</div>
    {:else}
      <div class="flex-none border-1 p-2">false</div>
    {/if}
  {:else if ty === "integer"}
    <div class="flex-none border-1 p-2">{value}</div>
  {:else if ty === "number"}
    <div class="flex-none border-1 p-2">{value}</div>
  {:else if ty === "string"}
    <Input
      type="text"
      class="nodrag nowheel flex-none text-wrap"
      {value}
      onkeydown={(evt) => {
        if (evt.ctrlKey && (evt.key === "a" || evt.key === "c")) {
          return;
        }
        evt.preventDefault();
      }}
    />
  {:else if ty === "text"}
    <Textarea
      class="nodrag nowheel flex-1 text-wrap"
      {value}
      onkeydown={(evt) => {
        if (evt.ctrlKey && (evt.key === "a" || evt.key === "c")) {
          return;
        }
        evt.preventDefault();
      }}
    />
  {:else if ty === "image"}
    <img class="flex-1 object-scale-down" src={value} alt="" />
  {:else if ty === "object"}
    <Textarea
      class="nodrag nowheel flex-1 text-wrap"
      value={JSON.stringify(value, null, 2)}
      onkeydown={(evt) => {
        if (evt.ctrlKey && (evt.key === "a" || evt.key === "c")) {
          return;
        }
        evt.preventDefault();
      }}
    />
  {:else if ty === "message" || ty === "messages"}
    <Messages messages={value} />
  {:else}
    <Textarea
      class="nodrag nowheel flex-1 text-wrap"
      value={JSON.stringify(value, null, 2)}
      onkeydown={(evt) => {
        if (evt.ctrlKey && (evt.key === "a" || evt.key === "c")) {
          return;
        }
        evt.preventDefault();
      }}
    />
  {/if}
{/snippet}

{#snippet display(
  key: string,
  data: { kind: string; value: any },
  display_config: SAgentDisplayConfigEntry,
)}
  {#if display_config?.hideTitle === true}
    <h3 class="flex-none">{display_config?.title || key}</h3>
    <p class="flex-none text-xs text-gray-500">{display_config?.description}</p>
  {/if}
  {@const dct = display_config?.type}
  {@const ty = dct === "*" ? data?.kind : dct}
  {@const value = data?.value}
  {#if value instanceof Array && ty !== "object" && ty !== "message"}
    <div class="flex-none flex flex-col gap-2">
      {#each value as v}
        {@render displayItem(ty, v)}
      {/each}
    </div>
  {:else}
    {@render displayItem(ty, value)}
  {/if}
{/snippet}

{#snippet inputItem(key: string, default_config: SAgentConfigEntry)}
  {#if default_config?.hidden !== true}
    {@const config = data.config[key]}
    {@const ty = default_config?.type}
    <h3 class="flex-none">{default_config?.title || key}</h3>
    {#if default_config?.description}
      <p class="flex-none text-xs text-gray-500">{default_config?.description}</p>
    {/if}
    {#if ty === "unit"}
      <Button color="alternative" class="flex-none" onclick={() => updateConfig(key, {})} />
    {:else if ty === "boolean"}
      <Toggle
        class="flex-none"
        checked={config}
        onchange={() => updateConfig(key, !data.config[key])}
      />
    {:else if ty === "integer"}
      <NumberInput
        class="nodrag flex-none"
        value={config}
        onkeydown={(evt) => {
          if (evt.key === "Enter") {
            updateConfig(key, evt.currentTarget.value);
          }
        }}
        onchange={(evt) => {
          if (evt.currentTarget.value !== data.config[key]) {
            updateConfig(key, evt.currentTarget.value);
          }
        }}
      />
    {:else if ty === "number"}
      <Input
        class="nodrag flex-none"
        type="text"
        value={config}
        onkeydown={(evt) => {
          if (evt.key === "Enter") {
            updateConfig(key, evt.currentTarget.value);
          }
        }}
        onchange={(evt) => {
          if (evt.currentTarget.value !== data.config[key]) {
            updateConfig(key, evt.currentTarget.value);
          }
        }}
      />
    {:else if ty === "string"}
      <Input
        class="nodrag flex-none"
        type="text"
        value={config}
        onkeydown={(evt) => {
          if (evt.key === "Enter") {
            updateConfig(key, evt.currentTarget.value);
          }
        }}
        onchange={(evt) => {
          if (evt.currentTarget.value !== data.config[key]) {
            updateConfig(key, evt.currentTarget.value);
          }
        }}
      />
    {:else if ty === "text"}
      <Textarea
        class="nodrag nowheel flex-1"
        value={config}
        onkeydown={(evt) => {
          if (evt.ctrlKey && evt.key === "Enter") {
            evt.preventDefault();
            updateConfig(key, evt.currentTarget.value);
          }
        }}
        onchange={(evt) => {
          if (evt.currentTarget.value !== data.config[key]) {
            updateConfig(key, evt.currentTarget.value);
          }
        }}
      />
    {:else if ty === "object"}
      <Textarea
        class="nodrag nowheel flex-1"
        value={config}
        onkeydown={(evt) => {
          if (evt.ctrlKey && evt.key === "Enter") {
            evt.preventDefault();
            updateConfig(key, evt.currentTarget.value);
          }
        }}
        onchange={(evt) => {
          if (evt.currentTarget.value !== data.config[key]) {
            updateConfig(key, evt.currentTarget.value);
          }
        }}
      />
    {:else}
      <Textarea class="nodrag nowheel flex-1" value={JSON.stringify(config, null, 2)} disabled />
    {/if}
  {/if}
{/snippet}

{#snippet contents()}
  {#if agentDefaultConfig}
    <form class="grow flex flex-col gap-1 pl-4 pr-4 pb-4">
      {#each agentDefaultConfig as [key, default_config]}
        {@render inputItem(key, default_config)}
      {/each}
    </form>
  {/if}

  {#if agentDisplayConfig}
    <div class="grow flex flex-col gap-1 pl-4 pr-4 pb-4">
      {#each agentDisplayConfig as [key, display_config]}
        {@render display(key, data.display[key], display_config)}
      {/each}
    </div>
  {/if}
{/snippet}

<NodeBase {id} {data} {agentDef} {titleColor} {inputCount} {title} {contents} {...props} />

{#if description || data.title}
  <Popover triggeredBy="#t-{uid}" placement="top-start" arrow={false} class="z-40">
    {#if data.title}
      <p class="text-sm font-semibold pb-1">{agentDef?.title ?? data.name}</p>
    {/if}
    {#if description}
      <p class="text-sm text-gray-500">{description}</p>
    {/if}
  </Popover>
{/if}

{#if errorMessages.length > 0}
  <Popover
    triggeredBy="#e-{uid}"
    placement="bottom"
    arrow={false}
    class="w-96 min-h-60 z-40 text-xs font-light text-gray-500 bg-white dark:bg-gray-800 dark:border-gray-600 dark:text-gray-400 flex flex-col"
  >
    <div class="grow flex flex-col gap-2">
      <Textarea
        class="grow nodrag nowheel text-wrap"
        value={errorMessages.join("\n")}
        onkeydown={(evt) => {
          evt.preventDefault();
        }}
        rows={8}
      />
      <Button size="xs" color="red" class="w-10 flex-none" onclick={clearError}>Clear</Button>
    </div>
  </Popover>
{/if}
