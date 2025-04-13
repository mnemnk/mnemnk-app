<script lang="ts">
  interface Props {
    flowNames: string[];
    currentFlowName: string;
    changeFlowName: (flowName: string) => void;
  }

  let { flowNames, currentFlowName, changeFlowName }: Props = $props();
</script>

<div class="backdrop-blur-xs">
  <h4>Flows</h4>
  <hr />
  {#each flowNames as flowName}
    <button
      type="button"
      class="w-full text-left p-1 pl-3 text-gray-400 hover:text-black hover:bg-gray-200 dark:hover:bg-gray-400"
      onclick={() => changeFlowName(flowName)}
    >
      {#if flowName === currentFlowName}
        <span class="text-semibold text-gray-900 dark:text-white">{flowName}</span>
      {:else}
        <span>{flowName}</span>
      {/if}
    </button>
  {/each}
</div>
