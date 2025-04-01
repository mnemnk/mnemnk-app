<script lang="ts">
  import { Button, Input, NumberInput, Textarea, Toggle } from "flowbite-svelte";

  import Card from "@/components/Card.svelte";
  import { deserializeAgentConfig, serializeAgentFlowNodeConfig } from "@/lib/agent";
  import type { SAgentConfig, SAgentDefinition } from "@/lib/types";
  import { setAgentGlobalConfig } from "@/lib/utils";

  interface Props {
    agentName: string;
    agentConfig: SAgentConfig;
    agentDef: SAgentDefinition;
  }

  const { agentName, agentConfig, agentDef }: Props = $props();

  const config = $state(deserializeAgentConfig(agentConfig, agentDef.global_config));

  function saveConfig() {
    let sconfig = serializeAgentFlowNodeConfig(config, agentDef.global_config);
    if (sconfig) {
      setAgentGlobalConfig(agentName, sconfig);
    }
  }
</script>

<Card title={agentDef.title ?? agentName} subtitle={agentDef.description}>
  <form>
    {#each Object.keys(config) as key}
      {@const globalConfig = agentDef.global_config?.[key]}
      <label class="block mb-3 text-sm font-medium text-gray-900 dark:text-white">
        {globalConfig?.title || key}
        <p class="text-xs text-gray-500">{globalConfig?.description}</p>
        {#if globalConfig?.type === "boolean"}
          <Toggle bind:checked={config[key]} />
        {:else if globalConfig?.type === "integer"}
          <NumberInput bind:value={config[key]} />
        {:else if globalConfig?.type === "number"}
          <Input type="number" bind:value={config[key]} />
        {:else if globalConfig?.type === "string" || globalConfig?.type === "string?"}
          <Input type="text" bind:value={config[key]} />
        {:else if globalConfig?.type === "string[]"}
          <Textarea bind:value={config[key]} />
        {:else if globalConfig?.type === "object"}
          <Textarea bind:value={config[key]} />
        {:else}
          <Input type="text" value={JSON.stringify(config[key], null, 2)} disabled />
        {/if}
      </label>
    {/each}

    <Button onclick={saveConfig} class="mt-3 w-fit" outline>Save</Button>
  </form>
</Card>
