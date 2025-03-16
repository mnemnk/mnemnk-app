<script lang="ts">
  import type { Snippet } from "svelte";

  import { Handle, Position, useSvelteFlow } from "@xyflow/svelte";
  import type { NodeProps } from "@xyflow/svelte";
  import { Button, Card } from "flowbite-svelte";
  import { CloseOutline } from "flowbite-svelte-icons";

  type Props = NodeProps & {
    title: Snippet;
    contents: Snippet;
  };

  let { id, title, contents }: Props = $props();

  const { deleteElements, getNode } = useSvelteFlow();

  async function deleteNode(event: MouseEvent) {
    event.preventDefault();
    const node = getNode(id);
    if (node !== undefined) {
      await deleteElements({ nodes: [node] });
    }
  }
</script>

<div>
  <Handle type="target" position={Position.Left} />
  <Card padding="none">
    <div class="flex justify-between items-center pl-4 pr-0 mb-2">
      {@render title()}
      <Button onclick={deleteNode}><CloseOutline /></Button>
    </div>
    {@render contents()}
  </Card>
  <Handle type="source" position={Position.Right} />
</div>
