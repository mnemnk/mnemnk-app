<script lang="ts">
  interface Props {
    flowNames: string[];
    currentFlowName: string;
    changeFlowName: (flowName: string) => void;
  }

  let { flowNames, currentFlowName, changeFlowName }: Props = $props();
</script>

<h4>Flows</h4>
<hr />
{#each flowNames as flowName}
  <button
    type="button"
    class="w-full text-left p-1 text-gray-400 bg-white dark:bg-black hover:bg-gray-200 dark:hover:bg-gray-800 pl-3"
    onclick={() => changeFlowName(flowName)}
  >
    {#if flowName === currentFlowName}
      <span class="text-semibold text-gray-900 dark:text-white">{flowName}</span>
    {:else}
      {flowName}
    {/if}
  </button>
{/each}
