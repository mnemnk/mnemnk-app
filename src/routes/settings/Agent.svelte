<script lang="ts">
  import { message } from "@tauri-apps/plugin-dialog";

  import { Button, Input, NumberInput, Textarea, Toggle } from "flowbite-svelte";

  import Card from "@/components/Card.svelte";
  import { deserializeAgentConfig, serializeAgentFlowNodeConfig } from "@/lib/agent";
  import type { SAgentConfig, SAgentDefinition } from "@/lib/types";
  import { exitApp, setAgentGlobalConfig } from "@/lib/utils";

  interface Props {
    agentName: string;
    agentConfig: SAgentConfig;
    agentDef: SAgentDefinition | null;
  }

  const { agentName, agentConfig, agentDef }: Props = $props();

  const config = $state(deserializeAgentConfig(agentConfig, agentDef?.global_config ?? null));

  async function saveConfig() {
    let sconfig = serializeAgentFlowNodeConfig(config, agentDef?.global_config ?? null);
    if (sconfig) {
      setAgentGlobalConfig(agentName, sconfig);
    }

    // confirm restart
    await message("Mnemnk will quit to apply changes.\n\nPlease restart.");
    await exitApp();
  }
</script>

{#if !agentDef}
  <Card title={agentName} subtitle="Agent not found">
    <p class="text-sm text-gray-500">This agent is not defined in the system.</p>
  </Card>
{:else if !agentDef.global_config}
  <Card title={agentName} subtitle="No global config">
    <p class="text-sm text-gray-500">This agent has no global config.</p>
  </Card>
{:else}
  <Card title={agentDef.title ?? agentName} subtitle={agentDef.description}>
    {#if agentDef.global_config}
      <form>
        {#each agentDef.global_config as [key, globalConfig]}
          {@const ty = globalConfig.type}
          <label class="block mb-3 text-sm font-medium text-gray-900 dark:text-white">
            {globalConfig?.title || key}
            <p class="text-xs text-gray-500">{globalConfig?.description}</p>
            {#if ty === "boolean"}
              <Toggle bind:checked={config[key]} />
            {:else if ty === "integer"}
              <NumberInput bind:value={config[key]} />
            {:else if ty === "number"}
              <Input type="number" bind:value={config[key]} />
            {:else if ty === "string"}
              <Input type="text" bind:value={config[key]} />
            {:else if ty === "text"}
              <Textarea bind:value={config[key]} />
            {:else if ty === "object"}
              <Textarea bind:value={config[key]} />
            {:else}
              <Input type="text" value={JSON.stringify(config[key], null, 2)} disabled />
            {/if}
          </label>
        {/each}

        <Button onclick={saveConfig} class="mt-3 w-fit" outline>Save</Button>
      </form>
    {/if}
  </Card>
{/if}
