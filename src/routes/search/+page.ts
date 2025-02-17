import { invoke } from "@tauri-apps/api/core";

export async function load({ url }) {
  const query = url.searchParams.get("q");
  // console.log("searching for", query);
  let events = await invoke("search", { query });
  // console.log("found", events);
  return {
    query,
    events,
  };
}
