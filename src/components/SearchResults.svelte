<script lang="ts">
  import { goto } from "$app/navigation";

  import { dateString, formatTime } from "@/lib/utils";

  interface Props {
    events: any;
  }
  let { events }: Props = $props();

  let events_by_date = $derived(
    events.reduce((acc, ev) => {
      let d = dateString(new Date(ev.time));
      if (!acc[d]) {
        acc[d] = {};
      }
      if (!acc[d][ev.kind]) {
        acc[d][ev.kind] = [];
      }
      acc[d][ev.kind].push(ev);
      return acc;
    }, {}),
  );

  let dates: string[] = $derived(Object.keys(events_by_date).sort().reverse());

  function gotoItem(time: number) {
    let d = new Date(time);
    let date_str = `${d.getFullYear()}${(d.getMonth() + 1).toString().padStart(2, "0")}${d.getDate().toString().padStart(2, "0")}`;
    let time_str = `${d.getHours().toString().padStart(2, "0")}${d.getMinutes().toString().padStart(2, "0")}`;
    let tag = `t${date_str}${time_str}`;
    goto(`/daily/${date_str}/#${tag}`);
  }
</script>

<div>
  {#each dates as date}
    <div class="mb-4">
      <h3 class="text-xl font-bold mb-2">
        <a href="/daily?d={date}">{date.slice(0, 4)}/{date.slice(4, 6)}/{date.slice(6)}</a>
      </h3>
      {#each Object.keys(events_by_date[date]) as kind}
        <div class="mb-4">
          <h4 class="text-lg font-bold mt-1 mb-1">{kind}</h4>
          {#each events_by_date[date][kind] as ev}
            <div>
              <button class="text-left clip-text" onclick={() => gotoItem(ev.time)}>
                {formatTime(new Date(ev.time))}
                {#if ev.kind === "application"}
                  {ev.value.name} / {ev.value.title}
                {:else if ev.kind === "browser"}
                  {ev.value.title}
                {:else}
                  {JSON.stringify(ev, null, 2)}
                {/if}
              </button>
            </div>
          {/each}
        </div>
      {/each}
    </div>
  {/each}
</div>

<style>
  .clip-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
