import { invoke } from "@tauri-apps/api/core";

export async function load({ url }) {
  const query = url.searchParams.get("q");
  if (!query) {
    return {
      query: "",
      events: [],
    };
  }

  let events = await invoke("search", { query });
  return {
    query,
    events,
  };
}
