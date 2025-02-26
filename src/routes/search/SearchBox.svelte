<script lang="ts">
  interface Props {
    query?: string;
    onsearch?: (query: string) => void;
  }

  const { query, onsearch }: Props = $props();

  let search_input: HTMLInputElement;
  let q = $state(query);

  function onsubmit(event: Event) {
    event.preventDefault();
    search_input.blur();
    if (q) {
      onsearch?.(q);
    }
  }

  $effect(() => {
    q = query;
    search_input.focus();
  });
</script>

<div class="">
  <form class="relative" {onsubmit}>
    <input
      id="search-input"
      bind:this={search_input}
      class="w-full dark:bg-gray-900 rounded-md p-2"
      type="text"
      placeholder="Enter a query..."
      autocomplete="off"
      bind:value={q}
    />
  </form>
</div>
