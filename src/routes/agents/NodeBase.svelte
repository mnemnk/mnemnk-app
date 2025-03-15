<script lang="ts" generics="T">
  import type { Snippet } from "svelte";

  import { Handle, Position, useNodes } from "@xyflow/svelte";
  import type { NodeProps } from "@xyflow/svelte";
  import { Button, Card } from "flowbite-svelte";
  import { CloseOutline } from "flowbite-svelte-icons";

  type Props = NodeProps & {
    title: Snippet;
    contents: Snippet;
  };

  let { id, title, contents }: Props = $props();

  const nodes = useNodes();

  function deleteNode() {
    nodes.update((nodes) => {
      nodes = nodes.filter((node) => node.id !== id);
      return nodes;
    });
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
