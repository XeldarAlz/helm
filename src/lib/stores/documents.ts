import { writable } from "svelte/store";

export const documents = writable<Record<string, string>>({});
