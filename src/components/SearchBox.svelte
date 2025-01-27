<script lang="ts">
  import IconsMdiMagnify from "~icons/mdi/magnify";

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
    <button
      type="submit"
      class="absolute right-3 top-1/2 transform -translate-y-1/2 focus:outline-none cursor-default"
      ><IconsMdiMagnify /></button
    >
  </form>
</div>
