<script lang="ts">
  interface Props {
    query?: string;
    onsearch?: (query: string) => void;
  }

  let { query = $bindable(""), onsearch }: Props = $props();

  function onsubmit(event: Event) {
    event.preventDefault();
    onsearch?.(query);
  }
</script>

<div class="">
  <form class="relative" {onsubmit}>
    <input
      id="search-input"
      class="w-full bg-surface-900 rounded-md p-2"
      type="text"
      placeholder="Enter a query..."
      autocomplete="off"
      bind:value={query}
    />
  </form>
</div>
