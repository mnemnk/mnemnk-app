<script lang="ts">
  import { Popover } from "flowbite-svelte";

  import type { MnemnkEvent, ScreenshotEvent } from "@/lib/types";
  import { mimgUrl } from "@/lib/utils";

  import EventsScrollbar from "./EventsScrollbar.svelte";

  interface Props {
    date: Date;
    events: MnemnkEvent[]; // assume events are sorted by time
  }

  let { date, events }: Props = $props();

  let date_str = $derived(
    `${date.getFullYear()} / ${(date.getMonth() + 1).toString().padStart(2, "0")} / ${date.getDate().toString().padStart(2, "0")}`,
  );

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
  let rows = $derived(events_by_dt);
  let all_apps = $derived(events_apps[1]);

  const APP_COLOR_PALLETE = [
    "oklch(100% 0.1303 105.01 / 40%)", // yellow
    "oklch(100% 0.1303 28.25 / 40%)", // red
    "oklch(100% 0.1303 265.65 / 40%)", // blue
    "oklch(100% 0.1303 146.95 / 40%)", // green
    "oklch(100% 0.1303 67.34 / 40%)", // orange
    "oklch(100% 0.1303 222.98 / 40%)", // light blue
    "oklch(100% 0.1303 322.51 / 40%)", // purple
  ];

  let app_colors: Record<string, string> = $derived.by(() => {
    const colors = {} as Record<string, string>;
    [...all_apps].forEach((app, i) => {
      colors[app] = APP_COLOR_PALLETE[i % APP_COLOR_PALLETE.length];
    });
    return colors;
  });

  let screenshotOnly = $state(false);

  function toggleScreenshotOnly() {
    if (screenshotOnly) {
      screenshotOnly = false;
    } else {
      screenshotOnly = true;
    }
  }

  function changeBackgroundImage(screenshot: ScreenshotEvent) {
    const img = new Image();
    img.src = mimgUrl(`${screenshot.kind}/${screenshot.value.image_id}`);
    img.onload = () => {
      document.body.style.backgroundImage = `url(${img.src})`;
    };
  }

  async function on_keydown(event: KeyboardEvent) {
    if (event.repeat) {
      return;
    }
    if (event.key === " ") {
      event.preventDefault();
      toggleScreenshotOnly();
    }
  }

  $effect(() => {
    return () => {
      document.body.style.backgroundImage = "";
    };
  });
</script>

<div class="static w-[100vw]">
  <div class={screenshotOnly ? "bg-transparent/0" : "bg-transparent/20"}>
    <div class="min-h-screen relative pt-13">
      <h1 class="text-3xl font-bold pb-8">{date_str}</h1>
      {#each rows as row}
        <div
          id="t{row[0]}"
          class="flex flex-cols gap-4 min-h-24"
          role="group"
          onmouseenter={() => {
            if (row[1]) {
              changeBackgroundImage(row[1]);
            }
          }}
        >
          <div class="w-[43px] p-[4px] flex-none bg-transparent/60 flex flex-rows items-center">
            <span class="text-[15px] font-semibold -mt-1">
              {(((row[0] / 100) | 0) % 100).toString().padStart(2, "0")}:{(row[0] % 100)
                .toString()
                .padStart(2, "0")}
            </span>
          </div>
          {#if screenshotOnly}
            <button
              class="w-[85px] flex-none"
              type="button"
              onclick={toggleScreenshotOnly}
              aria-label="back to timeline"
            ></button>
          {:else}
            <button class="w-[85px] flex-none" type="button" onclick={toggleScreenshotOnly}>
              {#if row[1]}
                <img
                  height="36"
                  width="85"
                  loading="lazy"
                  src={mimgUrl(`${row[1].kind}/${row[1].value.image_id}.t`)}
                  alt=""
                />
              {/if}
            </button>
          {/if}
          {#if screenshotOnly}
            <button
              class="w-[39vw] flex-none"
              type="button"
              onclick={toggleScreenshotOnly}
              aria-label="back to timeline"
            >
              {#each row[2] as app (app.id)}
                <div>&nbsp;</div>
              {/each}
            </button>
          {:else}
            <div class="w-[39vw] flex-none overflow-x-hidden">
              {#each row[2] as app (app.id)}
                <div
                  id="e-{app.id}"
                  class="text-nowrap drop-shadow"
                  style:background-color={app_colors[app.value.name]}
                >
                  {app.value.title}
                </div>
                <Popover
                  arrow={false}
                  offset={1}
                  placement="bottom-start"
                  trigger="click"
                  triggeredBy="#e-{app.id}"
                  class="!text-primary-50 !bg-gray-700 !bg-transparent/90 z-10 ml-4"
                >
                  <div class="p-3">
                    <div>{app.time.toLocaleString()}</div>
                    <pre>{JSON.stringify(app.value, null, 2)}</pre>
                  </div>
                </Popover>
              {/each}
            </div>
          {/if}
          {#if screenshotOnly}
            <button
              class="grow"
              type="button"
              onclick={toggleScreenshotOnly}
              aria-label="back to timeline"
            >
              {#each row[3] as event (event.id)}
                <div>&nbsp;</div>
              {/each}
            </button>
          {:else}
            <div class="grow text-nowrap overflow-x-visible">
              {#each row[3] as event (event.id)}
                {#if screenshotOnly}
                  <div>&nbsp;</div>
                {:else if event.kind === "browser"}
                  <div id="e-{event.id}">
                    <img
                      loading="lazy"
                      src={`http://www.google.com/s2/favicons?domain=${event.value.hostname}`}
                      alt=""
                      class="inline-block drop-shadow"
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
                    trigger="hover"
                    triggeredBy="#e-{event.id}"
                    class="!text-primary-50 !bg-gray-700 !bg-transparent/90 z-10 ml-4"
                  >
                    <div>
                      {event.value.url}
                    </div>
                  </Popover>
                {:else}
                  <div id="e-{event.id}" class="drop-shadow">
                    <div>{event.kind}</div>
                    <div>{event.time.toLocaleString()}</div>
                    <pre>{JSON.stringify(event.value, null, 2)}</pre>
                  </div>
                {/if}
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
    <div class="fixed top-0 right-0 z-10">
      <EventsScrollbar {events} />
    </div>
  </div>
</div>

<svelte:window on:keydown={on_keydown} />

<style>
  .drop-shadow {
    filter: drop-shadow(0 0 1.2px rgba(0, 0, 0, 0.8));
  }
</style>
