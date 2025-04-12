<script lang="ts">
  import type { Snippet } from "svelte";

  import { Card, Heading, Tooltip } from "flowbite-svelte";

  interface Props {
    title: string;
    tooltip?: string;
    subtitle?: string | null;
    class?: string;
    children: Snippet;
  }

  let { title, tooltip, subtitle = "", class: clazz = "", children }: Props = $props();
  const uid = $props.id();
</script>

<Card size="xl" class="shadow-xs {clazz}">
  <div class="mb-6 mt-px">
    <div class="-ml-0.25 mb-2">
      <Heading id="{uid}-title" tag="h3" class="text-xl font-semibold dark:text-white w-fit">
        {title}
      </Heading>
      {#if tooltip}
        <Tooltip triggeredBy="#{uid}-title" arrow={false} placement="top-end">{tooltip}</Tooltip>
      {/if}
    </div>
    {#if subtitle}
      <div>
        <span class="text-sm font-normal text-gray-500 dark:text-gray-400">
          {subtitle}
        </span>
      </div>
    {/if}
  </div>
  {@render children()}
</Card>
