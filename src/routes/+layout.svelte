<script>
  import Attribution from "@/components/Attribution.svelte";

  import "../app.css";

  /** @type {{children?: import('svelte').Snippet}} */
  let { children } = $props();
</script>

{@render children?.()}
<Attribution />
