<script lang="ts">
  import type { Snippet } from "svelte";

  import { Card, Heading } from "flowbite-svelte";

  interface Props {
    title: string;
    subtitle?: string;
    class?: string;
    children: Snippet;
  }

  let { title, subtitle = "", class: clazz = "", children }: Props = $props();
</script>

<Card size="xl" class="shadow-sm {clazz}">
  <div class="mb-4 mt-px lg:mb-0">
    <Heading tag="h3" class="-ml-0.25 mb-2 text-xl font-semibold dark:text-white">
      {title}
    </Heading>
    {#if subtitle}
      <span class="text-base font-normal text-gray-500 dark:text-gray-400">
        {subtitle}
      </span>
    {/if}
  </div>
  {@render children()}
</Card>
