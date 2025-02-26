import { search_events } from "$lib/utils.js";

export async function load({ url }) {
  const query = url.searchParams.get("q") || "";
  if (!query) {
    return {
      query: "",
      events: [],
    };
  }

  const events = await search_events(query);
  return {
    query,
    events,
  };
}
