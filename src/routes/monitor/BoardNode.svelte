<script lang="ts">
  import type { NodeProps } from "@xyflow/svelte";
  import { GradientButton } from "flowbite-svelte";

  // extends Props from NodeProps
  type Props = NodeProps & {
    data: {
      kind: string;
    };
  };

  let { id, data, ...props }: Props = $props();
</script>

<div {id}>
  <GradientButton outline color="cyanToBlue">
    {data.kind}
  </GradientButton>
</div>
