<script lang="ts">
  import { goto } from "$app/navigation";

  import type { MnemnkEvent } from "@/lib/types";
  import { dateString, formatTime } from "@/lib/utils";

  interface Props {
    events: MnemnkEvent[];
  }
  const { events }: Props = $props();

  const events_by_date = $derived(
    events.reduce(
      (acc, ev) => {
        const d = dateString(new Date(ev.time));
        if (!acc[d]) {
          acc[d] = {};
        }
        if (!acc[d][ev.kind]) {
          acc[d][ev.kind] = [];
        }
        acc[d][ev.kind].push(ev);
        return acc;
      },
      {} as Record<string, Record<string, MnemnkEvent[]>>,
    ),
  );

  const dates: string[] = $derived(Object.keys(events_by_date).sort().reverse());

  function gotoItem(local_ymd: number, time: number) {
    const d = new Date(time);
    const date_str = `${d.getFullYear()}${(d.getMonth() + 1).toString().padStart(2, "0")}${d.getDate().toString().padStart(2, "0")}`;
    const time_str = `${d.getHours().toString().padStart(2, "0")}${d.getMinutes().toString().padStart(2, "0")}`;
    const tag = `t${date_str}${time_str}`;
    goto(`/daily?d=${local_ymd}#${tag}`);
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
          {#each events_by_date[date][kind].reverse() as ev}
            <div>
              <button class="text-left clip-text" onclick={() => gotoItem(ev.local_ymd, ev.time)}>
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
