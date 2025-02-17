<script lang="ts">
  import { AppBar } from "@skeletonlabs/skeleton";
  import IconsMdiChevronLeft from "~icons/mdi/chevron-left";

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

<div>
  <AppBar class="!bg-transparent">
    {#snippet lead()}
      <a href="/"><IconsMdiChevronLeft /></a>
    {/snippet}
    <div class="text-xl"></div>
  </AppBar>
  <main class="container mx-auto p-8 space-y-8">
    <SearchBox {query} {onsearch} />
    <SearchResults {events} />
  </main>
</div>
