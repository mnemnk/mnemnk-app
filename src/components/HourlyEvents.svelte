<script lang="ts">
  import { onMount } from "svelte";

  import { Popover } from "flowbite-svelte";

  import { mimgUrl } from "@/lib/utils";

  interface MnemnkEvent {
    id: { id: { String: string } };
    kind: string;
    time: number;
    value: any;
  }
  interface Props {
    date?: string;
    events?: MnemnkEvent[]; // assume events are sorted by time
  }

  let { date = "", events = [] }: Props = $props();

  // unique hours in events.time
  // let hours: number[] = $derived(
  //   [...new Set(events.map((ev) => new Date(ev.time).getHours()))].sort((a, b) => b - a),
  // );

  // const APPS_WINDOW_SIZE = 60;
  // const NUM_TRACKING_APPS = 4;

  // [dt, screenshot, applications, previous applications, others]
  type EventRow = [number, MnemnkEvent | null, MnemnkEvent[], MnemnkEvent[]];
  let events_apps: [Map<number, EventRow>, Set<string>] = $derived.by(() => {
    let events_by_dt = new Map<number, EventRow>();
    let last_app = "";
    // let apps = [] as string[];
    // let apps_window = [] as string[];
    let all_apps = new Set() as Set<string>;
    events.forEach((ev) => {
      let d = new Date(ev.time);
      let dt =
        d.getFullYear() * 100000000 +
        (d.getMonth() + 1) * 1000000 +
        d.getDate() * 10000 +
        d.getHours() * 100 +
        d.getMinutes();
      let event = events_by_dt.get(dt);
      if (!event) {
        // apps_window = [last_app, ...apps_window].slice(0, APPS_WINDOW_SIZE);
        // apps = apps.filter((a) => apps_window.includes(a));
        event = [dt, null, [], []] as EventRow;
      }
      if (ev.kind === "screen") {
        event[1] = ev;
      } else if (ev.kind === "application") {
        last_app = ev.value.name;
        // apps = apps.includes(last_app)
        //   ? [last_app, ...apps.filter((a) => a !== last_app)]
        //   : [last_app, ...apps].slice(0, NUM_TRACKING_APPS);
        // apps = apps.slice(0, NUM_TRACKING_APPS);
        event[2].unshift(ev);
        all_apps.add(last_app);
      } else {
        if (ev.kind === "browser") {
          let url = new URL(ev.value.url);
          ev.value.hostname = url.hostname;
        }
        event[3].unshift(ev);
      }
      events_by_dt.set(dt, event);
    });
    return [events_by_dt, all_apps];
  });
  let events_by_dt = $derived([...events_apps[0].values()].sort((a, b) => a[0] - b[0]));
  let rows = $derived(events_by_dt.reverse());
  let all_apps = $derived(events_apps[1]);

  // $inspect(events_by_dt);

  // const ROW_HEIGHT = 96;
  // const ROW_OFFSET = 16;
  // const COL_WIDTH = 100;
  // const NODE_HEIGHT = 15;

  const APP_COLOR_PALLETE = [
    "oklch(70% 0.1303 105.01 / 25%)", // yellow
    "oklch(70% 0.1303 28.25 / 25%)", // red
    "oklch(70% 0.1303 265.65 / 25%)", // blue
    "oklch(70% 0.1303 146.95 / 25%)", // green
    "oklch(70% 0.1303 67.34 / 25%)", // orange
    "oklch(70% 0.1303 222.98 / 25%)", // light blue
    "oklch(70% 0.1303 322.51 / 25%)", // purple
  ];

  let app_colors: Record<string, string> = $derived.by(() => {
    const colors = {} as Record<string, string>;
    [...all_apps].forEach((app, i) => {
      colors[app] = APP_COLOR_PALLETE[i % APP_COLOR_PALLETE.length];
    });
    return colors;
  });

  // let app_paths = $derived.by(() => {
  //   const paths = [] as { path: string; color: string }[];
  //   const appPositions = {} as Record<string, { row: number; col: number }[]>;

  //   let num_events = events_by_dt.length;
  //   events_by_dt.forEach((row, rowIndex) => {
  //     row[2][1].forEach((app, colIndex) => {
  //       if (!appPositions[app]) {
  //         appPositions[app] = [];
  //       }
  //       appPositions[app].push({ row: num_events - rowIndex - 1, col: colIndex });
  //     });
  //   });

  //   Object.entries(appPositions).forEach(([app, positions]) => {
  //     if (positions.length === 1) {
  //       const x = positions[0].col * COL_WIDTH + COL_WIDTH / 2;
  //       const y = positions[0].row * ROW_HEIGHT + ROW_OFFSET;
  //       paths.push({
  //         path: `M ${x} ${y + NODE_HEIGHT} L ${x} ${y - NODE_HEIGHT / 2}`,
  //         color: app_colors[app],
  //       });
  //     }
  //     const path = positions.reduce((acc, pos, i, arr) => {
  //       const x = pos.col * COL_WIDTH + COL_WIDTH / 2;
  //       const y = pos.row * ROW_HEIGHT + ROW_OFFSET;

  //       if (i === 0) return `M ${x} ${y + NODE_HEIGHT}`;

  //       const prevX = arr[i - 1].col * COL_WIDTH + COL_WIDTH / 2;
  //       const prevY = arr[i - 1].row * ROW_HEIGHT + ROW_OFFSET;

  //       if (arr[i - 1].row - 1 !== pos.row) {
  //         return acc + ` L ${prevX} ${prevY - NODE_HEIGHT / 2} M ${x} ${y + NODE_HEIGHT / 2}`;
  //       }

  //       const startY = prevY - NODE_HEIGHT / 2;
  //       const endY = y - NODE_HEIGHT / 2;
  //       const controlY = (startY + endY) / 2;
  //       return acc + ` L ${prevX} ${startY} C ${prevX} ${controlY}, ${x} ${controlY}, ${x} ${endY}`;
  //     }, "");

  //     paths.push({ path, color: app_colors[app] });
  //   });

  //   return paths;
  // });

  // $inspect(app_paths);

  function backgroundImage(screenshot) {
    if (screenshot) {
      return `url(${mimgUrl(`${screenshot.kind}/${screenshot.value.image_id}`)})`;
    }
    return "";
  }

  let screenshotOnly = $state(false);
  let scrollPos = 0;

  function toggleScreenshotOnly() {
    if (screenshotOnly) {
      screenshotOnly = false;
    } else {
      scrollPos = window.scrollY || document.documentElement.scrollTop;
      screenshotOnly = true;
    }
  }

  $effect(() => {
    if (!screenshotOnly) {
      window.scrollTo(0, scrollPos);
    }
  });

  onMount(() => {
    return () => {
      document.body.style.backgroundImage = "";
    };
  });
</script>

<div class={screenshotOnly ? "bg-transparent/0" : "bg-transparent/60"}>
  {#if screenshotOnly}
    <div class="fixed inset-0">
      <button
        type="button"
        class="fixed inset-0 w-full h-full"
        aria-label="back to timeline"
        onclick={toggleScreenshotOnly}
      ></button>
    </div>
  {:else}
    <div class="w-full overflow-x-auto relative">
      <!-- <div class="pointer-events-none absolute top-0 left-36">
        <svg width={COL_WIDTH * NUM_TRACKING_APPS} height={ROW_HEIGHT * events_by_dt.length}>
          {#each app_paths as pathData, i}
            <path
              d={pathData.path}
              stroke={pathData.color}
              stroke-width="12"
              fill="none"
              opacity="0.5"
            />
          {/each}
        </svg>
      </div> -->

      <div class="ml-4">
        <!-- <h1 class="text-3xl font-bold bg-transparent/60 pt-2 pb-4">{date}</h1> -->
        {#each rows as row}
          <div
            id="t{row[0]}"
            class="flex flex-nowrap space-y-1 min-h-24 event-row"
            role="group"
            onmouseenter={() => {
              document.body.style.backgroundImage = backgroundImage(row[1]);
            }}
          >
            <div class="flex-none w-12">
              {(((row[0] / 100) | 0) % 100).toString().padStart(2, "0")}:{(row[0] % 100)
                .toString()
                .padStart(2, "0")}
            </div>
            <div class="flex-none w-36">
              {#if row[1]}
                <button type="button" class="screenshot-button" onclick={toggleScreenshotOnly}>
                  <img
                    height="36"
                    width="85"
                    loading="lazy"
                    src={mimgUrl(`${row[1].kind}/${row[1].value.image_id}.t`)}
                    alt=""
                  />
                </button>
              {/if}
            </div>
            <div class="flex-none w-1/2 overflow-x-hidden mr-4">
              {#each row[2] as app (app.id.id.String)}
                <div
                  id="e-{app.id.id.String}"
                  class="text-nowrap"
                  style="background-color: {app_colors[app.value.name]}"
                >
                  {app.value.title}
                </div>
                <Popover
                  arrow={false}
                  offset={1}
                  placement="bottom-start"
                  trigger="click"
                  triggeredBy="#e-{app.id.id.String}"
                  class="!text-primary-50 !bg-gray-700"
                >
                  <div class="p-3">
                    <div>{app.time.toLocaleString()}</div>
                    <pre>{JSON.stringify(app.value, null, 2)}</pre>
                  </div>
                </Popover>
              {/each}
            </div>
            <div class="flex-auto w-96 text-nowrap">
              {#each row[3] as event (event.id.id.String)}
                {#if event.kind === "browser"}
                  <div id="e-{event.id.id.String}">
                    <img
                      src={`http://www.google.com/s2/favicons?domain=${event.value.hostname}`}
                      alt=""
                      class="inline-block"
                      width="16"
                      height="16"
                    />
                    <a href={event.value.url} target="_blank" rel="noopener noreferrer"
                      >{event.value.title}</a
                    >
                  </div>
                  <Popover
                    arrow={false}
                    offset={1}
                    placement="bottom-start"
                    trigger="click"
                    triggeredBy="#e-{event.id.id.String}"
                    class="!text-primary-50 !bg-gray-700"
                  >
                    <div>
                      {event.value.url}
                    </div>
                  </Popover>
                {:else}
                  <div id="e-{event.id.id.String}">
                    <div>{event.kind}</div>
                    <div>{event.time.toLocaleString()}</div>
                    <pre>{JSON.stringify(event.value, null, 2)}</pre>
                  </div>
                {/if}
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .event-row {
    filter: drop-shadow(0 0 1.2px rgba(0, 0, 0, 0.8));
  }
</style>
