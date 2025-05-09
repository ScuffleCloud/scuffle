import { writable } from 'svelte/store';
export const hoveredDate = writable<Date | null>(null);
