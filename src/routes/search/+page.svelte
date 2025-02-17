<script lang="ts">
  import { goto } from "$app/navigation";

  import SearchBox from "@/components/SearchBox.svelte";
  import SearchResults from "@/components/SearchResults.svelte";

  let { data } = $props();
  let query = $derived(data.query);
  let events = $derived(data.events);

  function onsearch(query: string) {
    if (!query) {
      return;
    }
    goto(`search?q=${query}`);
  }
</script>

<main class="container mx-auto p-8 space-y-8 mt-20">
  <SearchBox {query} {onsearch} />
  <SearchResults {events} />
</main>
